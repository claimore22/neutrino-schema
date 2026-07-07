//! Database introspection — reads live schema metadata via PostgreSQL's
//! `information_schema` views.
//!
//! The [`DatabaseIntrospector`] trait abstracts the introspection API.
//! [`PostgresIntrospector`] is the built-in implementation using `sqlx`.

mod column;
mod postgres;

pub use column::*;
pub use postgres::*;
