//! PostgreSQL type representation and normalisation.
//!
//! Raw SQL type strings (e.g. `"integer"`, `"character varying"`) are parsed
//! into [`PgType`], then mapped to a database-agnostic [`DbType`] for use
//! downstream in the pipeline.  Nullability is never encoded in the type enum —
//! it is always represented as `Option<T>` at codegen time.

mod pg_type;
mod db_type;

pub use pg_type::*;
pub use db_type::*;
