//! Schema-to-Rust compiler pipeline.
//!
//! Provides database introspection, type normalization,
//! SchemaIR generation, and Rust model code generation.

pub mod types;
pub mod ir;
pub mod codegen;
pub mod config;

#[cfg(feature = "postgres")]
pub mod introspect;

#[cfg(feature = "cli")]
pub mod cli;

pub use types::{dbtype_to_rust, to_db_type, DbType, PgType};
pub use ir::{FieldIR, RelationIR, RelationStrategy, SchemaIR, TableIR};
pub use codegen::{generate_files, generate_struct, RenderMode};

#[cfg(feature = "postgres")]
pub use introspect::{Column, DatabaseIntrospector, PostgresIntrospector};
