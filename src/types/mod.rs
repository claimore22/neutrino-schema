//! Database type representation and normalisation.
//!
//! Raw column type strings from PostgreSQL (`PgType`) or SQLite (`SqliteType`)
//! are mapped to a database-agnostic [`DbType`] for use downstream in the
//! pipeline.  Nullability is never encoded in the type enum — it is always
//! represented as `Option<T>` at codegen time.

mod db_type;
mod enum_ref;
mod mysql_type;
mod pg_type;
mod sqlite_type;
mod type_registry;

pub use db_type::*;
pub use enum_ref::*;
pub use mysql_type::*;
pub use pg_type::*;
pub use sqlite_type::*;
pub use type_registry::*;
