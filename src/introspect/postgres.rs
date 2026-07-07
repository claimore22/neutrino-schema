use sqlx::{PgPool, Row};

use crate::ir::FieldIR;
use crate::introspect::Column;
use crate::types::{self, PgType};

/// Abstraction for database introspection.
///
/// Implementors provide table and column metadata from a live database,
/// typically by querying `information_schema`.
#[async_trait::async_trait]
pub trait DatabaseIntrospector: Send + Sync {
    /// List all user-accessible table names in the `public` schema.
    async fn list_tables(&self) -> anyhow::Result<Vec<String>>;
    /// List all columns for a given table, in ordinal position order.
    async fn list_columns(&self, table: &str) -> anyhow::Result<Vec<Column>>;
}

/// PostgreSQL implementation of [`DatabaseIntrospector`].
///
/// Queries `information_schema.tables` and `information_schema.columns`
/// filtered to the `public` schema.
pub struct PostgresIntrospector {
    /// SQLx connection pool for the target database.
    pub pool: PgPool,
}

impl PostgresIntrospector {
    /// Create a new introspector from an existing connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Convert an introspected [`Column`] into a [`FieldIR`] for the IR pipeline.
    ///
    /// This is the boundary between the introspection and IR layers:
    /// the raw [`PgType`] is normalised to [`DbType`] here.
    pub fn column_to_field(col: &Column) -> FieldIR {
        let db_ty = types::to_db_type(col.data_type.clone());
        FieldIR {
            name: col.column_name.clone(),
            ty: db_ty,
            nullable: col.nullable,
            raw_type: format!("{:?}", col.data_type),
        }
    }
}

#[async_trait::async_trait]
impl DatabaseIntrospector for PostgresIntrospector {
    async fn list_tables(&self) -> anyhow::Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
            ORDER BY table_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| r.get::<Option<String>, _>("table_name"))
            .collect())
    }

    async fn list_columns(&self, table: &str) -> anyhow::Result<Vec<Column>> {
        let rows = sqlx::query(
            r#"
            SELECT column_name, data_type, is_nullable
            FROM information_schema.columns
            WHERE table_schema = 'public' AND table_name = $1
            ORDER BY ordinal_position
            "#,
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let raw = r.get::<String, _>("data_type");
                Column {
                    table_name: table.to_string(),
                    column_name: r.get("column_name"),
                    data_type: PgType::map_pg_type(raw.as_str()),
                    nullable: r.get::<String, _>("is_nullable") == "YES",
                }
            })
            .collect())
    }
}
