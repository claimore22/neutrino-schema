use sqlx::{MySqlPool, Row};

use crate::ir::{EnumIR, EnumVariantIR, FieldIR};
use crate::introspect::{parse_mysql_enum, Column, DatabaseIntrospector};
use crate::types::{self, MysqlType};
use crate::util::naming::{enum_variant_name, to_struct_name};

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
            SELECT TABLE_NAME AS `table_name`
            FROM information_schema.tables
            WHERE TABLE_SCHEMA = DATABASE()
              AND TABLE_TYPE = 'BASE TABLE'
            ORDER BY TABLE_NAME
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.get("table_name")).collect())
    }

    async fn list_columns(&self, table: &str) -> anyhow::Result<Vec<Column>> {
        let rows = sqlx::query(
            r#"
            SELECT COLUMN_NAME  AS `column_name`,
                   DATA_TYPE    AS `data_type`,
                   IS_NULLABLE  AS `is_nullable`
            FROM information_schema.columns
            WHERE TABLE_SCHEMA = DATABASE()
              AND TABLE_NAME = ?
            ORDER BY ORDINAL_POSITION
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

    async fn introspect_enums(&self) -> anyhow::Result<Vec<EnumIR>> {
        let rows = sqlx::query(
            r#"
            SELECT TABLE_NAME   AS `table_name`,
                   COLUMN_NAME  AS `column_name`,
                   COLUMN_TYPE  AS `column_type`
            FROM information_schema.columns
            WHERE TABLE_SCHEMA = DATABASE()
              AND DATA_TYPE = 'enum'
            ORDER BY TABLE_NAME, ORDINAL_POSITION
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut enums = Vec::new();
        for r in rows {
            let table_name: String = r.get("table_name");
            let column_name: String = r.get("column_name");
            let column_type: String = r.get("column_type");

            let Some(variants) = parse_mysql_enum(&column_type) else {
                continue;
            };

            // Generate a unique Rust name from table + column
            let db_name = format!("{}.{}", table_name, column_name);
            let rust_name = to_struct_name(&db_name);
            let schema = None; // MySQL does not have schema-qualified enums

            let variants: Vec<EnumVariantIR> = variants
                .into_iter()
                .map(|v| EnumVariantIR {
                    database_name: v.clone(),
                    rust_name: enum_variant_name(&v),
                })
                .collect();

            enums.push(EnumIR {
                database_name: db_name,
                rust_name,
                variants,
                schema,
            });
        }

        Ok(enums)
    }
}
