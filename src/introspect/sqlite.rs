use sqlx::{Row, SqlitePool};

use crate::ir::FieldIR;
use crate::introspect::Column;
use crate::types::{self, SqliteType};

/// SQLite implementation of [`DatabaseIntrospector`](crate::introspect::DatabaseIntrospector).
///
/// Uses `PRAGMA table_info` and `sqlite_master` to discover tables
/// and their columns.
pub struct SqliteIntrospector {
    /// SQLx connection pool for the target database.
    pub pool: SqlitePool,
}

impl SqliteIntrospector {
    /// Create a new introspector from an existing SQLite connection pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl super::DatabaseIntrospector for SqliteIntrospector {
    fn column_to_field(&self, col: &Column) -> FieldIR {
        let sqlite_ty = SqliteType::map_sqlite_type(&col.data_type);
        let db_ty = types::sqlite_to_db_type(sqlite_ty);
        FieldIR {
            name: col.column_name.clone(),
            ty: db_ty,
            nullable: col.nullable,
            raw_type: col.data_type.clone(),
        }
    }
    async fn list_tables(&self) -> anyhow::Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT name
            FROM sqlite_master
            WHERE type = 'table'
              AND name NOT LIKE 'sqlite_%'
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| r.get::<Option<String>, _>("name"))
            .collect())
    }

    async fn list_columns(&self, table: &str) -> anyhow::Result<Vec<Column>> {
        let rows = sqlx::query("SELECT * FROM pragma_table_info(?1)")
            .bind(table)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let raw: Option<String> = r.get("type");
                Column {
                    table_name: table.to_string(),
                    column_name: r.get("name"),
                    data_type: raw.unwrap_or_else(|| "TEXT".to_string()),
                    nullable: r.get::<i32, _>("notnull") == 0,
                }
            })
            .collect())
    }
}
