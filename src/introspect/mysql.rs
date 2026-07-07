use sqlx::{MySqlPool, Row};

use crate::ir::FieldIR;
use crate::introspect::Column;
use crate::introspect::DatabaseIntrospector;
use crate::types::{self, MysqlType};

/// MySQL/MariaDB implementation of [`DatabaseIntrospector`](crate::introspect::DatabaseIntrospector).
///
/// Queries `information_schema.tables` and `information_schema.columns`
/// filtered to the current database (`DATABASE()`).
pub struct MysqlIntrospector {
    /// SQLx connection pool for the target database.
    pub pool: MySqlPool,
}

impl MysqlIntrospector {
    /// Create a new introspector from an existing connection pool.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DatabaseIntrospector for MysqlIntrospector {
    fn column_to_field(&self, col: &Column) -> FieldIR {
        let mysql = MysqlType::map_mysql_type(&col.data_type);
        let db_ty = types::mysql_to_db_type(mysql);
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
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = DATABASE()
              AND table_type = 'BASE TABLE'
            ORDER BY table_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.get("table_name")).collect())
    }

    async fn list_columns(&self, table: &str) -> anyhow::Result<Vec<Column>> {
        let rows = sqlx::query(
            r#"
            SELECT column_name, data_type, is_nullable
            FROM information_schema.columns
            WHERE table_schema = DATABASE()
              AND table_name = ?
            ORDER BY ordinal_position
            "#,
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let raw: String = r.get("data_type");
                Column {
                    table_name: table.to_string(),
                    column_name: r.get("column_name"),
                    data_type: raw,
                    nullable: r.get::<String, _>("is_nullable") == "YES",
                }
            })
            .collect())
    }
}
