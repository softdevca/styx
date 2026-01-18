# Handoff: Integrate Schema Extraction with CLI/LSP/MCP

## Completed
- Rebuilt `facet-styx/src/schema_gen.rs` - schema generation from Facet types (was accidentally deleted)
- Added `styx @extract <binary>` CLI command in `crates/styx-cli/src/main.rs:579-604`
- Fixed `styx-embed-macros` proc_macro2 conversion for unsynn compatibility
- Simplified `docs/content/tools/schema-distribution.md` to one recommended workflow
- Pushed commit `799a876`

## Active Work

### Origin
User said:
> "push, then add seamless integration with CLI validate subcommands, LSP, MCP etc."

This follows the styx-embed work where we added the ability to embed schemas in binaries and extract them. Now the tooling needs to USE that extraction automatically when resolving schemas.

### The Problem
When a config file declares `@schema {source crate:myapp-config@1, cli myapp}`, the tooling should automatically try to extract embedded schemas from the `myapp` binary FIRST, before falling back to running `myapp @dump-styx-schema` or fetching from crates.io.

Currently, the CLI validation (`--validate` flag) and LSP don't try embedded extraction at all.

### Current State
- Branch: `main`
- The `@extract` command works standalone
- `styx_embed::extract_schemas_from_file()` is available with `mmap` feature
- Integration with validation/LSP is NOT done

The resolution order should be:
1. **Scan binary** - extract embedded schema (no execution)
2. **Run CLI** - `myapp @dump-styx-schema` (if embedded not found)
3. **Fetch crate** - download from crates.io (future)

### Technical Context

**CLI validation code** is in `crates/styx-cli/src/main.rs`:
- `run_validation()` at ~line 315 handles `--validate` flag
- `find_schema_declaration()` at ~line 393 parses `@schema` from document
- `resolve_schema_path()` at ~line 420 resolves paths
- Currently only handles file paths and inline schemas

The `SchemaRef` enum at line 364:
```rust
enum SchemaRef {
    External(String),  // file path or URL
    Inline(Value),     // inline schema object
}
```

This needs to be extended to handle `crate:foo@1, cli bar` syntax.

**LSP code** is in `crates/styx-lsp/` - I didn't explore this yet.

**MCP** - user mentioned this but I'm not sure what MCP integration exists. May refer to Model Context Protocol server in the codebase.

**styx-embed extraction API**:
```rust
// In styx_embed crate
pub fn extract_schemas_from_file(path: &Path) -> Result<Vec<String>, Box<dyn Error>>
```
Requires `mmap` feature. Returns all embedded schemas as strings.

**Binary location**: Need to find the binary from the `cli` field. Use `which` crate or shell out to `which myapp`.

### Success Criteria
1. `styx config.styx --validate` works when config has `@schema {source crate:foo@1, cli myapp}` and `myapp` binary has embedded schema
2. LSP provides validation/completions using embedded schemas
3. Falls back gracefully if binary not found or no embedded schema
4. MCP integration (if applicable) uses same resolution

### Files to Touch
- `crates/styx-cli/src/main.rs` - extend schema resolution in `find_schema_declaration()` and add new resolution logic
- `crates/styx-cli/Cargo.toml` - already has `styx-embed` with `mmap` feature
- `crates/styx-lsp/src/` - integrate extraction (need to explore structure)
- Possibly create shared schema resolution module if logic is duplicated

### Decisions Made
- Use `styx_embed::extract_schemas_from_file()` with mmap for efficient scanning
- Resolution order: embedded → CLI → crates.io (per docs)
- User wanted ONE good workflow, not multiple options

### What NOT to Do
- Don't implement crates.io fetching yet - user said "we're not even allowed to publish on crates.io yet"
- Don't overcomplicate - keep it simple
- Don't make big architectural decisions without asking (I got told off for removing unsynn)

### Blockers/Gotchas
- The `@schema` directive syntax needs proper parsing - it's an object `{source crate:foo@1, cli myapp}` not just a string
- Need to locate binary by name - probably use `which` command or look in PATH
- LSP might have different constraints (async, can't block on file I/O)

## Bootstrap
```bash
git status
cargo build -p styx-cli
cargo build -p styx-lsp
# Look at current schema resolution:
grep -n "find_schema_declaration\|SchemaRef\|resolve_schema" crates/styx-cli/src/main.rs | head -20
```
