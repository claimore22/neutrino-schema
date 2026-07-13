use std::collections::HashMap;

use sqlx::{PgPool, Row};

use crate::ir::{ConstraintIR, ConstraintKind, EnumIR, EnumVariantIR, FieldIR, IndexEntryIR, IndexIR, IndexKind, MatchType};
use crate::introspect::{parse_referential_action, Column, TableInfo};
use crate::introspect::DatabaseIntrospector;
use crate::types::{self, PgType};
use crate::util::naming::to_struct_name;

/// PostgreSQL implementation of [`DatabaseIntrospector`](crate::introspect::DatabaseIntrospector).
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

}

#[async_trait::async_trait]
impl DatabaseIntrospector for PostgresIntrospector {
    fn column_to_field(&self, col: &Column) -> FieldIR {
        let pg = PgType::map_pg_type(&col.data_type);
        let db_ty = types::to_db_type(pg);
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
            SELECT t.table_name,
                pg_catalog.obj_description(c.oid) AS table_comment
            FROM information_schema.tables t
            JOIN pg_catalog.pg_class c ON c.relname = t.table_name
            JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
            WHERE t.table_schema = 'public' AND n.nspname = 'public'
            ORDER BY t.table_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let rows: Vec<TableInfo> = rows
            .into_iter()
            .filter_map(|r| r.get::<Option<String>, _>("table_name").map(|name| TableInfo {
                name,
                comment: r.get("table_comment"),
            }))
            .collect();

        Ok(rows)
    }

    async fn list_columns(&self, table: &str) -> anyhow::Result<Vec<Column>> {
        let rows = sqlx::query(
            r#"
            SELECT column_name, data_type, udt_name, is_nullable,
            pg_catalog.col_description(
                (
                    SELECT c.oid
                    FROM pg_catalog.pg_class c
                    JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relname = $1 AND n.nspname = 'public'
                ),
                ordinal_position::int
            ) AS column_comment
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
                let raw_data_type: String = r.get("data_type");
                let udt_name: String = r.get("udt_name");
                // PostgreSQL reports enums as data_type = 'USER-DEFINED' and
                // arrays as data_type = 'ARRAY'. Use udt_name for these.
                let data_type = if raw_data_type == "USER-DEFINED" || raw_data_type == "ARRAY" {
                    udt_name
                } else {
                    raw_data_type
                };
                Column {
                    table_name: table.to_string(),
                    column_name: r.get("column_name"),
                    data_type,
                    nullable: r.get::<String, _>("is_nullable") == "YES",
                    comment: r.get("column_comment"),
                }
            })
            .collect())
    }

    async fn introspect_enums(&self) -> anyhow::Result<Vec<EnumIR>> {
        let rows = sqlx::query(
            r#"
            SELECT t.typname      AS enum_name,
                   e.enumlabel    AS variant,
                   ns.nspname     AS schema
            FROM pg_enum e
            JOIN pg_type     t  ON e.enumtypid = t.oid
            JOIN pg_namespace ns ON t.typnamespace = ns.oid
            WHERE ns.nspname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY t.oid, e.enumsortorder
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        // Group rows by (schema, enum_name)
        let mut raw: Vec<(String, String, String)> = rows
            .into_iter()
            .map(|r| {
                (
                    r.get::<String, _>("schema"),
                    r.get::<String, _>("enum_name"),
                    r.get::<String, _>("variant"),
                )
            })
            .collect();
        raw.sort_by(|a, b| a.1.cmp(&b.1));

        // Collect unique (schema, enum_name) pairs and their variants
        #[derive(Clone)]
        struct RawEnum {
            schema: String,
            db_name: String,
            variants: Vec<String>,
        }
        let mut raw_enums: Vec<RawEnum> = Vec::new();
        for (schema, db_name, variant) in raw {
            let last = raw_enums.last_mut();
            if let Some(e) = last {
                if e.schema == schema && e.db_name == db_name {
                    e.variants.push(variant);
                    continue;
                }
            }
            raw_enums.push(RawEnum {
                schema,
                db_name,
                variants: vec![variant],
            });
        }

        // Resolve Rust names — if a name is unique across schemas, use bare name;
        // if duplicated, prefix with schema name.
        let mut name_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for e in &raw_enums {
            *name_counts.entry(e.db_name.clone()).or_insert(0) += 1;
        }

        let enums: Vec<EnumIR> = raw_enums
            .into_iter()
            .map(|e| {
                let rust_name = if name_counts.get(&e.db_name).copied().unwrap_or(0) > 1 {
                    // Collision — prefix with schema name
                    let prefixed = format!("{}.{}", e.schema, e.db_name);
                    to_struct_name(&prefixed)
                } else {
                    to_struct_name(&e.db_name)
                };

                let variants: Vec<EnumVariantIR> = e
                    .variants
                    .iter()
                    .map(|v| EnumVariantIR {
                        database_name: v.clone(),
                        rust_name: crate::util::naming::enum_variant_name(v),
                    })
                    .collect();

                EnumIR {
                    database_name: e.db_name,
                    rust_name,
                    variants,
                    schema: Some(e.schema),
                }
            })
            .collect();

        Ok(enums)
    }

    async fn list_constraints(&self, table: &str) -> anyhow::Result<Vec<ConstraintIR>> {
        let rows = sqlx::query(
            r#"
            SELECT
                tc.constraint_name,
                tc.constraint_type,
                ccu.column_name,
                ccu.ordinal_position,
                ccu.table_schema      AS ref_table_schema,
                ccu.table_name        AS ref_table_name,
                ccu.column_name       AS ref_column_name,
                rc.update_rule,
                rc.delete_rule,
                cc.check_clause
            FROM information_schema.table_constraints tc
            LEFT JOIN information_schema.key_column_usage ccu
                ON tc.constraint_catalog = ccu.constraint_catalog
                AND tc.constraint_schema = ccu.constraint_schema
                AND tc.constraint_name = ccu.constraint_name
            LEFT JOIN information_schema.referential_constraints rc
                ON tc.constraint_catalog = rc.constraint_catalog
                AND tc.constraint_schema = rc.constraint_schema
                AND tc.constraint_name = rc.constraint_name
            LEFT JOIN information_schema.check_constraints cc
                ON tc.constraint_catalog = cc.constraint_catalog
                AND tc.constraint_schema = cc.constraint_schema
                AND tc.constraint_name = cc.constraint_name
            WHERE tc.table_schema = current_schema()
              AND tc.table_name = $1
            ORDER BY tc.constraint_name, ccu.ordinal_position
            "#,
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let mut grouped: HashMap<String, ConstraintBuilder> = HashMap::new();
        for r in rows {
            let name: String = r.get("constraint_name");
            let entry = grouped.entry(name.clone()).or_insert_with(|| {
                let raw: String = r.get("constraint_type");
                ConstraintBuilder {
                    name,
                    constraint_type: raw,
                    columns: Vec::new(),
                    ref_table: None,
                    ref_columns: Vec::new(),
                    update_rule: None,
                    delete_rule: None,
                    check_clause: None,
                    match_type: None,
                }
            });
            if let Ok(col) = r.try_get::<Option<String>, _>("column_name") {
                if let Some(col) = col {
                    entry.columns.push(col);
                }
            }
            if entry.ref_table.is_none() {
                if let Ok(Some(ref_table)) = r.try_get::<Option<String>, _>("ref_table_name") {
                    entry.ref_table = Some(ref_table);
                }
            }
            if let Ok(Some(col)) = r.try_get::<Option<String>, _>("ref_column_name") {
                if !entry.ref_columns.contains(&col) {
                    entry.ref_columns.push(col);
                }
            }
            if entry.update_rule.is_none() {
                if let Ok(Some(rule)) = r.try_get::<Option<String>, _>("update_rule") {
                    entry.update_rule = Some(rule);
                }
            }
            if entry.delete_rule.is_none() {
                if let Ok(Some(rule)) = r.try_get::<Option<String>, _>("delete_rule") {
                    entry.delete_rule = Some(rule);
                }
            }
            if entry.check_clause.is_none() {
                if let Ok(Some(clause)) = r.try_get::<Option<String>, _>("check_clause") {
                    entry.check_clause = Some(clause);
                }
            }
        }

        Ok(grouped.into_values().filter_map(|b| b.build()).collect())
    }

    async fn list_indexes(&self, table: &str) -> anyhow::Result<Vec<IndexIR>> {
        let rows = sqlx::query(
            r#"
            SELECT
                c.relname                        AS index_name,
                i.indisunique                    AS is_unique,
                am.amname                        AS index_type,
                pg_get_expr(i.indpred, i.indrelid) AS predicate,
                i.indkey::text                   AS indkey_str,
                i.indoption::text                AS indoption_str,
                pg_get_expr(i.indexprs, i.indrelid) AS indexprs
            FROM pg_index i
            JOIN pg_class c  ON c.oid = i.indexrelid
            JOIN pg_class t  ON t.oid = i.indrelid
            JOIN pg_am    am ON am.oid = c.relam
            JOIN pg_namespace n ON n.oid = t.relnamespace
            WHERE t.relname = $1
              AND n.nspname = 'public'
            ORDER BY c.relname
            "#,
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        // Batch-resolve pg_attribute for all columns in this table
        let attr_rows = sqlx::query(
            r#"
            SELECT a.attnum, a.attname
            FROM pg_attribute a
            JOIN pg_class c ON c.oid = a.attrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE c.relname = $1
              AND n.nspname = 'public'
              AND a.attnum > 0
              AND NOT a.attisdropped
            "#,
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;

        let attr_map: std::collections::HashMap<i16, String> = attr_rows
            .into_iter()
            .filter_map(|r| {
                let num: i16 = r.get("attnum");
                let name: String = r.get("attname");
                Some((num, name))
            })
            .collect();

        let mut indexes = Vec::new();
        for r in rows {
            let name: String = r.get("index_name");
            let unique: bool = r.get("is_unique");
            let idx_type: String = r.get("index_type");
            let kind = pg_index_kind(&idx_type);
            let predicate: Option<String> = r.get("predicate");
            let predicate = if predicate.as_deref() == Some("") { None } else { predicate };

            let indkey_str: String = r.get("indkey_str");
            let indoption_str: String = r.get("indoption_str");
            let expr_str: Option<String> = r.get("indexprs");

            let key_attnums: Vec<i16> = indkey_str
                .split_whitespace()
                .filter_map(|s| s.parse::<i16>().ok())
                .collect();
            let options: Vec<i16> = indoption_str
                .split_whitespace()
                .filter_map(|s| s.parse::<i16>().ok())
                .collect();

            let expr_parts: Vec<&str> = expr_str
                .as_deref()
                .map(|s| s.split(',').map(|p| p.trim()).collect())
                .unwrap_or_default();

            let mut entries = Vec::new();
            for (i, &attnum) in key_attnums.iter().enumerate() {
                let desc = options.get(i).copied().unwrap_or(0) & 0x01 != 0;

                if attnum == 0 {
                    if let Some(expr) = expr_parts.get(i) {
                        entries.push(IndexEntryIR::Expression {
                            expression: expr.to_string(),
                        });
                    }
                } else if let Some(col_name) = attr_map.get(&attnum) {
                    entries.push(IndexEntryIR::Column {
                        name: col_name.clone(),
                        descending: desc,
                    });
                }
            }

            indexes.push(IndexIR {
                name,
                table_name: table.to_string(),
                entries,
                unique,
                kind,
                predicate,
            });
        }
        Ok(indexes)
    }
}

/// Map PostgreSQL access method name to [`IndexKind`].
fn pg_index_kind(am: &str) -> IndexKind {
    match am {
        "btree" => IndexKind::BTree,
        "hash" => IndexKind::Hash,
        "gin" => IndexKind::Gin,
        "gist" => IndexKind::Gist,
        "brin" => IndexKind::Brin,
        other => IndexKind::Other(other.to_string()),
    }
}

struct ConstraintBuilder {
    name: String,
    constraint_type: String,
    columns: Vec<String>,
    ref_table: Option<String>,
    ref_columns: Vec<String>,
    update_rule: Option<String>,
    delete_rule: Option<String>,
    check_clause: Option<String>,
    match_type: Option<MatchType>,
}

impl ConstraintBuilder {
    fn build(self) -> Option<ConstraintIR> {
        let kind = match self.constraint_type.as_str() {
            "PRIMARY KEY" => ConstraintKind::PrimaryKey { columns: self.columns },
            "UNIQUE" => ConstraintKind::Unique { columns: self.columns },
            "FOREIGN KEY" => ConstraintKind::ForeignKey {
                columns: self.columns,
                referenced_table: self.ref_table.unwrap_or_default(),
                referenced_columns: self.ref_columns,
                on_delete: parse_referential_action(self.delete_rule.as_deref()),
                on_update: parse_referential_action(self.update_rule.as_deref()),
                match_type: self.match_type,
            },
            "CHECK" => ConstraintKind::Check {
                expression: self.check_clause.unwrap_or_default(),
            },
            _ => return None,
        };
        Some(ConstraintIR { name: self.name, kind })
    }
}
