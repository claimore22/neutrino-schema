use crate::introspect::Column;
use crate::introspect::table::TableInfo;
use crate::ir::{ConstraintIR, EnumIR, FieldIR, IndexIR};

/// Abstraction for database introspection.
///
/// Implementors provide table and column metadata from a live database,
/// typically by querying `information_schema` (PostgreSQL) or
/// `sqlite_master` / `PRAGMA table_info` (SQLite).
#[async_trait::async_trait]
pub trait DatabaseIntrospector: Send + Sync {
    /// List all user-accessible table information (in `public` schema for supported databases,
    /// excluding internal `sqlite_%` tables for SQLite).
    async fn list_tables_with_info(&self) -> anyhow::Result<Vec<TableInfo>>;
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
    /// List all constraints (PK, FK, UNIQUE, CHECK) for a given table.
    async fn list_constraints(&self, _table: &str) -> anyhow::Result<Vec<ConstraintIR>> {
        Ok(Vec::new())
    }
    /// List all physical indexes for a given table.
    ///
    /// Default implementation returns an empty vec (for backends that don't
    /// yet support index introspection).
    async fn list_indexes(&self, _table: &str) -> anyhow::Result<Vec<IndexIR>> {
        Ok(Vec::new())
    }
}
