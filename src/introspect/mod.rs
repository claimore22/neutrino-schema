//! Database introspection — reads live schema metadata.
//!
//! The [`DatabaseIntrospector`] trait abstracts the introspection API.
//! [`PostgresIntrospector`], [`SqliteIntrospector`], and [`MysqlIntrospector`]
//! are the built-in implementations using `sqlx`.

mod column;
mod helpers;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "mysql")]
mod mysql_enum;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;
mod table;
mod traits;

pub use column::*;
pub(crate) use helpers::*;
#[cfg(feature = "mysql")]
pub use mysql::*;
#[cfg(feature = "mysql")]
pub use mysql_enum::*;
#[cfg(feature = "postgres")]
pub use postgres::*;
#[cfg(feature = "sqlite")]
pub use sqlite::*;
pub use table::*;
pub use traits::*;
