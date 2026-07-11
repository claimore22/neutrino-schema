use std::collections::HashMap;

use sqlx::ColumnOrigin::Table;
use sqlx::{MySqlPool, Row};

use crate::introspect::parse_referential_action;
use crate::introspect::table::TableInfo;
use crate::ir::{ConstraintIR, ConstraintKind, EnumIR, EnumVariantIR, FieldIR};
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
            comment: col.comment.clone(),
        }
    }

    async fn list_tables_with_info(&self) -> anyhow::Result<Vec<TableInfo>> {
        let rows = sqlx::query(
            r#"
                SELECT TABLE_NAME AS table_name,
                    TABLE_COMMENT AS table_comment
                FROM information_schema.tables
                WHERE TABLE_SCHEMA = DATABASE()
                AND TABLE_TYPE = 'BASE TABLE'
                ORDER BY TABLE_NAME
            "#,
        )
        .fetch_all(&self.pool)
        .await?;


        let rows: Vec<TableInfo> = rows
            .into_iter()
            .map(|r| TableInfo {
                name: r.get("table_name"),
                comment: r.get("table_comment"),
            })
            .collect();
    

        Ok(rows)
    }

    async fn list_columns(&self, table: &str) -> anyhow::Result<Vec<Column>> {
        let rows = sqlx::query(
            r#"
            SELECT COLUMN_NAME  AS `column_name`,
                   DATA_TYPE    AS `data_type`,
                   IS_NULLABLE  AS `is_nullable`,
                   COLUMN_COMMENT AS `column_comment`
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
                    comment: r.get("column_comment"),
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

    async fn list_constraints(&self, table: &str) -> anyhow::Result<Vec<ConstraintIR>> {
        let mut constraints: Vec<ConstraintIR> = Vec::new();

        // Primary key — from STATISTICS
        let pk_rows = sqlx::query(
            r#"
            SELECT COLUMN_NAME
            FROM information_schema.STATISTICS
            WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND INDEX_NAME = 'PRIMARY'
            ORDER BY SEQ_IN_INDEX
            "#,
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let pk_cols: Vec<String> = pk_rows.into_iter().map(|r| r.get("COLUMN_NAME")).collect();
        if !pk_cols.is_empty() {
            constraints.push(ConstraintIR {
                name: format!("{}_pkey", table),
                kind: ConstraintKind::PrimaryKey { columns: pk_cols },
            });
        }

        // Foreign keys — from KEY_COLUMN_USAGE + REFERENTIAL_CONSTRAINTS
        let fk_rows = sqlx::query(
            r#"
            SELECT
                kcu.CONSTRAINT_NAME,
                kcu.COLUMN_NAME,
                kcu.REFERENCED_TABLE_NAME,
                kcu.REFERENCED_COLUMN_NAME,
                kcu.ORDINAL_POSITION,
                rc.UPDATE_RULE,
                rc.DELETE_RULE
            FROM information_schema.KEY_COLUMN_USAGE kcu
            JOIN information_schema.REFERENTIAL_CONSTRAINTS rc
                ON kcu.CONSTRAINT_NAME = rc.CONSTRAINT_NAME
                AND kcu.CONSTRAINT_SCHEMA = rc.CONSTRAINT_SCHEMA
            WHERE kcu.TABLE_SCHEMA = DATABASE()
              AND kcu.TABLE_NAME = ?
              AND kcu.REFERENCED_TABLE_NAME IS NOT NULL
            ORDER BY kcu.CONSTRAINT_NAME, kcu.ORDINAL_POSITION
            "#,
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let mut fk_groups: HashMap<String, FkBuilder> = HashMap::new();
        for r in fk_rows {
            let name: String = r.get("CONSTRAINT_NAME");
            let entry = fk_groups.entry(name).or_insert_with(|| FkBuilder {
                columns: Vec::new(),
                ref_table: String::new(),
                ref_columns: Vec::new(),
                update_rule: None,
                delete_rule: None,
            });
            entry.columns.push(r.get("COLUMN_NAME"));
            entry.ref_table = r.get("REFERENCED_TABLE_NAME");
            entry.ref_columns.push(r.get("REFERENCED_COLUMN_NAME"));
            if entry.update_rule.is_none() {
                entry.update_rule = r.try_get::<Option<String>, _>("UPDATE_RULE").ok().flatten();
            }
            if entry.delete_rule.is_none() {
                entry.delete_rule = r.try_get::<Option<String>, _>("DELETE_RULE").ok().flatten();
            }
        }
        for (name, fb) in fk_groups {
            constraints.push(ConstraintIR {
                name,
                kind: ConstraintKind::ForeignKey {
                    columns: fb.columns,
                    referenced_table: fb.ref_table,
                    referenced_columns: fb.ref_columns,
                    on_delete: parse_referential_action(fb.delete_rule.as_deref()),
                    on_update: parse_referential_action(fb.update_rule.as_deref()),
                    match_type: None,
                },
            });
        }

        // Unique constraints — from STATISTICS (non-unique=0, not primary)
        let uq_rows = sqlx::query(
            r#"
            SELECT INDEX_NAME, COLUMN_NAME, SEQ_IN_INDEX
            FROM information_schema.STATISTICS
            WHERE TABLE_SCHEMA = DATABASE()
              AND TABLE_NAME = ?
              AND NON_UNIQUE = 0
              AND INDEX_NAME != 'PRIMARY'
            ORDER BY INDEX_NAME, SEQ_IN_INDEX
            "#,
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let mut uq_groups: HashMap<String, Vec<String>> = HashMap::new();
        for r in uq_rows {
            let name: String = r.get("INDEX_NAME");
            uq_groups.entry(name).or_default().push(r.get("COLUMN_NAME"));
        }
        for (name, columns) in uq_groups {
            constraints.push(ConstraintIR {
                name,
                kind: ConstraintKind::Unique { columns },
            });
        }

        // Check constraints (MySQL 8.0.16+)
        let ck_rows = sqlx::query(
            r#"
            SELECT CONSTRAINT_NAME, CHECK_CLAUSE
            FROM information_schema.CHECK_CONSTRAINTS
            WHERE CONSTRAINT_SCHEMA = DATABASE()
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for r in ck_rows {
            // CHECK_CONSTRAINTS doesn't directly link to a table in MySQL.
            // Match by constraint name prefix or skip.
            let ck_name: String = r.get("CONSTRAINT_NAME");
            let clause: String = r.get("CHECK_CLAUSE");
            constraints.push(ConstraintIR {
                name: ck_name,
                kind: ConstraintKind::Check { expression: clause },
            });
        }

        Ok(constraints)
    }
}

struct FkBuilder {
    columns: Vec<String>,
    ref_table: String,
    ref_columns: Vec<String>,
    update_rule: Option<String>,
    delete_rule: Option<String>,
}
