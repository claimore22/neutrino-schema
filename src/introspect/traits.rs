use crate::introspect::Column;
use crate::ir::{EnumIR, FieldIR};

/// Abstraction for database introspection.
///
/// Implementors provide table and column metadata from a live database,
/// typically by querying `information_schema` (PostgreSQL) or
/// `sqlite_master` / `PRAGMA table_info` (SQLite).
#[async_trait::async_trait]
pub trait DatabaseIntrospector: Send + Sync {
    /// List all user-accessible table names (in `public` schema for Postgres,
    /// excluding internal `sqlite_%` tables for SQLite).
    async fn list_tables(&self) -> anyhow::Result<Vec<String>>;
    /// List all columns for a given table, in ordinal position order.
    async fn list_columns(&self, table: &str) -> anyhow::Result<Vec<Column>>;
    /// Convert an introspected [`Column`] into a [`FieldIR`] for the IR pipeline.
    fn column_to_field(&self, col: &Column) -> FieldIR;
    /// Introspect all enum types defined in the database.
    ///
    /// Returns an empty vec for databases without native enum support (SQLite).
    async fn introspect_enums(&self) -> anyhow::Result<Vec<EnumIR>> {
        Ok(Vec::new())
    }
}
