//! Multi-database schema-to-Rust compiler pipeline.
//!
//! Introspects a live PostgreSQL, MySQL/MariaDB, or SQLite database, normalises
//! column types into a database-agnostic Intermediate Representation
//! ([`SchemaIR`]), and generates strongly typed Rust model structs.
//!
//! # Pipeline
//!
//! 1. **Introspect** — reads schema metadata via [`DatabaseIntrospector`]
//! 2. **Normalise** — raw column types map to [`DbType`]; fields, tables,
//!    constraints, enums, and indexes collect into [`SchemaIR`]
//! 3. **Generate** — [`SchemaIR`] is consumed by [`generate()`] which returns
//!    a [`GeneratedOutput`] containing file contents; the CLI writes them via
//!    [`OutputWriter`]
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
//!             default_value: None,
//!             generated: false,
//!             comment: None,
//!         }],
//!         constraints: vec![],
//!         comment: None,
//!         indexes: vec![],
//!     },
//! ];
//!
//! let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);
//!
//! let output = generate(&schema, &GenerateOptions::default());
//! let generated = output.file("users.rs").map(|f| &f.content[..]).unwrap_or("");
//! assert!(generated.contains("pub struct Users"));
//! ```
//!
//! # Quick start (CLI)
//!
//! ```bash
//! # First run — creates neutrino-schema.toml
//! neutrino-schema generate
//! # → prompts for database URL, saves it, generates types
//!
//! # Subsequent runs — just works from config
//! neutrino-schema generate
//!
//! # Explicit setup (CI / scripting)
//! neutrino-schema init --database-url "postgres://localhost/mydb"
//! neutrino-schema generate
//!
//! # All flags still work
//! neutrino-schema generate --database-url "mysql://user:pass@localhost/mydb" --output src/entities
//! ```

pub mod codegen;
pub mod config;
pub mod inference;
pub mod ir;
pub mod types;
pub mod util;
pub mod validation;

#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
pub mod introspect;

#[cfg(feature = "cli")]
pub mod cli;

pub use codegen::{
    generate, generate_enum_defs, generate_files, generate_files_with_registry, generate_imports,
    generate_struct, GenerateOptions, GeneratedFile, GeneratedOutput, OutputWriter, RenderMode,
    RustGeneratorConfig,
};
pub use ir::{
    ConstraintIR, ConstraintKind, EnumIR, EnumVariantIR, FieldIR, IndexEntryIR, IndexIR, IndexKind,
    MatchType, ReferentialAction, RelationCardinality, RelationIR, RelationInferenceStrategy,
    RelationOrigin, RelationStrategy, SchemaIR, SemanticRelationIR, TableIR,
};
pub use validation::{validate, ValidationEntry, ValidationLevel, ValidationReport};
pub use types::{
    dbtype_to_rust, mysql_to_db_type, sqlite_declared_to_db_type, sqlite_to_db_type, to_db_type,
    DbType, EnumRef, MysqlType, PgType, RustType, SqliteType, TypeRegistry,
};
pub use util::naming::{
    deduplicate_identifier, enum_variant_name, sanitize_identifier, to_struct_name,
};

#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
pub use introspect::{Column, DatabaseIntrospector};

#[cfg(feature = "postgres")]
pub use introspect::PostgresIntrospector;

#[cfg(feature = "sqlite")]
pub use introspect::SqliteIntrospector;

#[cfg(feature = "mysql")]
pub use introspect::MysqlIntrospector;
