# Handoff: SQL Functions Implementation Blocked by facet-styx Bug

## Completed
- Updated `dibs-query-schema/src/lib.rs` ValueExpr to use `#[facet(other)]` pattern
- Updated dibs AST ValueExpr with FunctionCall variant
- Updated `parse.rs` to convert schema to AST function calls
- Updated `sql.rs` to generate SQL function calls
- Created test file `/Users/amos/bearcove/styx/crates/facet-styx/src/value_expr_test.rs` to isolate the bug

## Active Work

### Origin
User wanted to implement general SQL function call syntax (005-sql-functions) in dibs to replace hardcoded `@now` with a flexible mechanism supporting arbitrary SQL functions like `@coalesce($a $b)`, `@lower($x)`, etc.

User quote:
> "so we want: @default / @funcname / @funcname(args...) / or a bare scalar like: $name / "literal" / 123"

### The Problem
We discovered a bug in facet-styx (or facet-format) where `#[facet(other)]` with `#[facet(tag)] tag: Option<String>` doesn't correctly handle bare scalars.

**Expected behavior:**
- `@now` → `tag: Some("now"), content: None` (nullary function)
- `@coalesce($a $b)` → `tag: Some("coalesce"), content: Some(Seq([...]))` (function with args)
- `$name` → `tag: None, content: Some(Scalar("$name"))` (bare scalar)

**Actual behavior:**
- `@now` → CORRECT
- `$name` → `tag: Some("$name"), content: None` (WRONG - scalar is being put in tag field)

User confirmed:
> "god fucking damn it facet-styx is wrong"
> "yeah Idk what's wrong but it's not respecting facet(tag) at all"
> "time to fix styx :)"

### Current State
- Branch: main (no separate branch created)
- No PR yet - blocked on facet bug
- Test file created at `/Users/amos/bearcove/styx/crates/facet-styx/src/value_expr_test.rs`

User said:
> "well uhh if your fix is in the facet crate... hold up b/c we're also changing it here"

This means the user is actively working on the facet crate themselves. **Wait for them before making changes to facet.**

### Technical Context

The test schema in `/Users/amos/bearcove/styx/crates/facet-styx/src/value_expr_test.rs`:

```rust
#[derive(Facet, Debug, PartialEq)]
#[facet(untagged)]
#[repr(u8)]
pub enum Payload {
    Scalar(String),
    Seq(Vec<ValueExpr>),
}

#[derive(Facet, Debug, PartialEq)]
#[facet(rename_all = "lowercase")]
#[repr(u8)]
pub enum ValueExpr {
    Default,
    #[facet(other)]
    Other {
        #[facet(tag)]
        tag: Option<String>,
        #[facet(content)]
        content: Option<Payload>,
    },
}
```

The bug is in facet-format's enum deserializer, likely in `deserialize_other_variant_with_captured_tag` function in `/Users/amos/bearcove/facet/facet-format/src/deserializer/eenum.rs:1433`.

The styx two-dimensional value model (user explained):
- Every value has a `tag` (Option - None for bare scalars, Some("name") for @name)
- Every value has a `payload` (unit/scalar/sequence)

For bare scalars: tag=None, payload=Scalar
For @tag: tag=Some("tag"), payload=unit
For @tag(args): tag=Some("tag"), payload=Seq

### Success Criteria
1. Test `test_bare_scalar` in `/Users/amos/bearcove/styx/crates/facet-styx/src/value_expr_test.rs` passes
2. All other tests in that file pass (`test_default_tag`, `test_nullary_function`)
3. Once facet is fixed, complete SQL function implementation in dibs

### Files to Touch
- `/Users/amos/bearcove/facet/facet-format/src/deserializer/eenum.rs:1433` - fix `deserialize_other_variant_with_captured_tag`
- `/Users/amos/bearcove/styx/crates/facet-styx/src/value_expr_test.rs` - test file (already created)
- After fix: `/Users/amos/bearcove/dibs/crates/dibs-query-schema/src/lib.rs` - finalize ValueExpr schema

### Decisions Made
- Use `#[facet(other)]` with `#[facet(tag)]` and `#[facet(content)]` instead of multiple variants
- Use `Option<Payload>` instead of `Payload::Unit` - user said "remove the Unit variant"
- Use `#[facet(untagged)]` on Payload enum to match based on structure not tag names

### What NOT to Do
- **DO NOT modify the facet crate without checking with user** - they said they're "also changing it here"
- Don't add multiple `#[facet(other)]` variants - user said "I don't think _two_ other is ever going to work"
- Don't oversimplify the tests - user explicitly said "stop, do not simplify the test"

### Blockers/Gotchas
- User is actively making changes to facet crate - coordinate before touching `/Users/amos/bearcove/facet/`
- The bug is that bare scalars are being interpreted as tags, not as content
- facet-styx depends on facet-format, so the fix needs to be in facet-format

## Bootstrap
```bash
# In styx repo
cd /Users/amos/bearcove/styx
cargo test -p facet-styx value_expr_test

# In dibs repo (after facet is fixed)
cd /Users/amos/bearcove/dibs
pnpm run check
```
