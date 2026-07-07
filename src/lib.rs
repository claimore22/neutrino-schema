//! Schema-to-Rust compiler pipeline.
//!
//! Introspects a live PostgreSQL database, normalises column types into a
//! database-agnostic Intermediate Representation ([`SchemaIR`]), and generates
//! strongly typed Rust model structs.
//!
//! # Pipeline
//!
//! 1. **Introspect** — queries `information_schema.columns` via [`DatabaseIntrospector`]
//!    to discover tables, columns, types, and nullability.
//! 2. **Normalise** — raw [`PgType`] values are mapped to [`DbType`] (database-agnostic).
//!    Columns are collected into [`FieldIR`] → [`TableIR`] → [`SchemaIR`]. Optional
//!    relation inference uses naming heuristics (column ending in `_id`).
//! 3. **Generate** — [`SchemaIR`] is rendered into `.rs` files via [`generate_files`]
//!    or printed per-table with [`generate_struct`].
//!
//! # Quick start (programmatic)
//!
//! ```rust,no_run
//! use neutrino_schema::*;
//!
//! let tables = vec![
//!     TableIR {
//!         name: "users".into(),
//!         fields: vec![FieldIR {
//!             name: "email".into(),
//!             ty: DbType::String,
//!             nullable: false,
//!             raw_type: "Varchar".into(),
//!         }],
//!     },
//! ];
//!
//! let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);
//!
//! let generated = generate_struct(&schema.tables[0], RenderMode::Clean);
//! assert!(generated.contains("pub struct Users"));
//! ```
//!
//! # Quick start (CLI)
//!
//! ```bash
//! # Inspect and print one table
//! neutrino-schema inspect "postgres://localhost/mydb" users
//!
//! # Generate all tables to ./src/models
//! neutrino-schema generate --database-url "postgres://localhost/mydb" --output ./src/models
//! ```

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
pub use codegen::{generate_files, generate_struct, to_struct_name, RenderMode};

#[cfg(feature = "postgres")]
pub use introspect::{Column, DatabaseIntrospector, PostgresIntrospector};
