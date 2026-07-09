//! Database introspection — reads live schema metadata.
//!
//! The [`DatabaseIntrospector`] trait abstracts the introspection API.
//! [`PostgresIntrospector`], [`SqliteIntrospector`], and [`MysqlIntrospector`]
//! are the built-in implementations using `sqlx`.

mod column;
mod traits;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "mysql")]
mod mysql_enum;

pub use column::*;
pub use traits::*;
#[cfg(feature = "postgres")]
pub use postgres::*;
#[cfg(feature = "sqlite")]
pub use sqlite::*;
#[cfg(feature = "mysql")]
pub use mysql::*;
#[cfg(feature = "mysql")]
pub use mysql_enum::*;
