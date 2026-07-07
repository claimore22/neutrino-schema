//! Database type representation and normalisation.
//!
//! Raw column type strings from PostgreSQL (`PgType`) or SQLite (`SqliteType`)
//! are mapped to a database-agnostic [`DbType`] for use downstream in the
//! pipeline.  Nullability is never encoded in the type enum — it is always
//! represented as `Option<T>` at codegen time.

mod pg_type;
mod sqlite_type;
mod mysql_type;
mod db_type;

pub use pg_type::*;
pub use sqlite_type::*;
pub use mysql_type::*;
pub use db_type::*;
