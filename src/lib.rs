//! Multi-database schema-to-Rust compiler pipeline.
//!
//! Introspects a live PostgreSQL, MySQL/MariaDB, or SQLite database, normalises
//! column types into a database-agnostic Intermediate Representation
//! ([`SchemaIR`]), and generates strongly typed Rust model structs.
//!
//! # Pipeline
//!
//! 1. **Introspect** — reads schema metadata via [`DatabaseIntrospector`]
//!    (querying `information_schema` for PostgreSQL/MySQL or `sqlite_master` /
//!    `PRAGMA table_info` for SQLite).
//! 2. **Normalise** — raw [`PgType`], [`MysqlType`], or [`SqliteType`] values
//!    are mapped to [`DbType`] (database-agnostic).  Columns are collected into
//!    [`FieldIR`] → [`TableIR`] → [`SchemaIR`].  Optional relation inference
//!    uses naming heuristics (column ending in `_id`).
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
//! # PostgreSQL
//! neutrino-schema generate --database-url "postgres://localhost/mydb" --output src/models
//!
//! # MySQL
//! neutrino-schema generate --database-url "mysql://user:pass@localhost/mydb" --output src/models
//!
//! # SQLite
//! neutrino-schema generate --database-url "sqlite:./dev.db" --output src/models
//! ```

pub mod types;
pub mod ir;
pub mod codegen;
pub mod config;

#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
pub mod introspect;

#[cfg(feature = "cli")]
pub mod cli;

pub use types::{dbtype_to_rust, mysql_to_db_type, sqlite_to_db_type, to_db_type, DbType, MysqlType, PgType, SqliteType};
pub use ir::{FieldIR, RelationIR, RelationStrategy, SchemaIR, TableIR};
pub use codegen::{generate_files, generate_struct, to_struct_name, RenderMode};

#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
pub use introspect::{Column, DatabaseIntrospector};

#[cfg(feature = "postgres")]
pub use introspect::PostgresIntrospector;

#[cfg(feature = "sqlite")]
pub use introspect::SqliteIntrospector;

#[cfg(feature = "mysql")]
pub use introspect::MysqlIntrospector;
