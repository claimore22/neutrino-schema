use sqlx::{PgPool, Row};

use crate::ir::{EnumIR, EnumVariantIR, FieldIR};
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
}
