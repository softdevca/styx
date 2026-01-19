//! Generate the meta-schema from Rust types.
//!
//! Run with: cargo run -p styx-schema --bin gen-meta-schema > schema/meta.gen.styx

use styx_schema::SchemaFile;

fn main() {
    let schema = facet_styx::schema_from_type::<SchemaFile>();
    print!("{}", schema);
}
