use std::collections::HashMap;

use sqlx::{PgPool, Row};

use crate::ir::{ConstraintIR, ConstraintKind, EnumIR, EnumVariantIR, FieldIR, MatchType, ReferentialAction};
use crate::introspect::Column;
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
        }
    }
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

pub(crate) fn parse_referential_action(s: Option<&str>) -> ReferentialAction {
    match s {
        Some("CASCADE") => ReferentialAction::Cascade,
        Some("SET NULL") => ReferentialAction::SetNull,
        Some("SET DEFAULT") => ReferentialAction::SetDefault,
        Some("RESTRICT") => ReferentialAction::Restrict,
        _ => ReferentialAction::NoAction,
    }
}
