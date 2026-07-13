mod common;

use std::collections::HashMap;

use common::migrations_common::{load_migration_sql, MigrationBackend};
use neutrino_schema::introspect::DatabaseIntrospector;
use neutrino_schema::types::DbType;
use neutrino_schema::{ConstraintIR, SchemaIR};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

async fn build_schema_sqlite(db_suffix: &str) -> SchemaIR {
    use sqlx::SqlitePool;

    let pool = SqlitePool::connect(":memory:").await.expect("failed to connect to in-memory SQLite pool");
    let sql = load_migration_sql(MigrationBackend::Sqlite).expect("load_migration_sql failed for Sqlite");
    common::migrations_common::execute_sqlite_batch(&pool, &sql)
        .await
        .expect("execute_sqlite_batch failed");

    let introspector = neutrino_schema::introspect::SqliteIntrospector::new(pool);
    build_schema_from(introspector).await
}

async fn build_schema_postgres(db_name: &str) -> Option<SchemaIR> {
    use sqlx::PgPool;

    let url = database_url()?;
    let admin = PgPool::connect(&url).await.ok()?;
    let drop_sql: &'static str =
        &*Box::leak(format!("DROP DATABASE IF EXISTS \"{db_name}\"").into_boxed_str());
    sqlx::query(drop_sql).execute(&admin).await.ok();
    let create_sql: &'static str =
        &*Box::leak(format!("CREATE DATABASE \"{db_name}\"").into_boxed_str());
    sqlx::query(create_sql).execute(&admin).await.ok()?;
    admin.close().await;

    let fixture_url = format!("{}/{}", url.trim_end_matches('/'), db_name);
    let pool = PgPool::connect(&fixture_url).await.ok()?;
    let sql = load_migration_sql(MigrationBackend::Postgres).expect("load_migration_sql failed for Postgres");
    let sql: &'static str = Box::leak(sql.into_boxed_str());
    for stmt in sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await.ok();
        }
    }
    let introspector = neutrino_schema::introspect::PostgresIntrospector::new(pool);
    Some(build_schema_from(introspector).await)
}

async fn build_schema_mysql(db_name: &str) -> Option<SchemaIR> {
    use sqlx::MySqlPool;

    let url = database_url()?;
    let admin = MySqlPool::connect(&url).await.ok()?;
    let drop_sql: &'static str =
        &*Box::leak(format!("DROP DATABASE IF EXISTS `{db_name}`").into_boxed_str());
    sqlx::query(drop_sql).execute(&admin).await.ok();
    let create_sql: &'static str =
        &*Box::leak(format!("CREATE DATABASE `{db_name}`").into_boxed_str());
    sqlx::query(create_sql).execute(&admin).await.ok()?;
    admin.close().await;

    let fixture_url = format!("{}/{}", url.trim_end_matches('/'), db_name);
    let pool = MySqlPool::connect(&fixture_url).await.ok()?;
    let sql = load_migration_sql(MigrationBackend::Mysql).expect("load_migration_sql failed for Mysql");
    let sql: &'static str = Box::leak(sql.into_boxed_str());
    for stmt in sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await.ok();
        }
    }
    let introspector = neutrino_schema::introspect::MysqlIntrospector::new(pool);
    Some(build_schema_from(introspector).await)
}

async fn build_schema_from(
    introspector: impl DatabaseIntrospector,
) -> SchemaIR {
    let table_infos = introspector.list_tables_with_info().await.expect("list_tables failed");
    let mut tables = Vec::new();
    for table_info in table_infos {
        let columns = introspector.list_columns(&table_info.name).await.expect("list_columns failed");
        let fields: Vec<_> = columns
            .iter()
            .map(|c| introspector.column_to_field(c))
            .collect();
        let constraints = introspector.list_constraints(&table_info.name).await.unwrap();
        tables.push(neutrino_schema::ir::TableIR {
            name: table_info.name.clone(),
            fields,
            constraints,
            comment: table_info.comment.clone(),
            indexes: vec![],
        });
    }
    SchemaIR::from_tables(tables, neutrino_schema::ir::RelationStrategy::Disabled)
}

/// Index by table + field name for comparison.
fn field_map(schema: &SchemaIR) -> HashMap<(String, String), (DbType, bool)> {
    let mut m = HashMap::new();
    for table in &schema.tables {
        for f in &table.fields {
            m.insert((table.name.clone(), f.name.clone()), (f.ty.clone(), f.nullable));
        }
    }
    m
}

/// Index by table + constraint name for FK comparison.
fn fk_map(schema: &SchemaIR) -> HashMap<String, Vec<&neutrino_schema::ir::ConstraintIR>> {
    let mut m: HashMap<String, Vec<_>> = HashMap::new();
    for table in &schema.tables {
        for c in &table.constraints {
            if matches!(c.kind, neutrino_schema::ir::ConstraintKind::ForeignKey { .. }) {
                m.entry(table.name.clone()).or_default().push(c);
            }
        }
    }
    m
}

// ---------------------------------------------------------------------------
// Cross-backend type normalization tests
// ---------------------------------------------------------------------------

/// PG and MySQL should agree on common `VARCHAR` / `BIGINT` / `SMALLINT` /
/// `TEXT` / `BOOLEAN` / `JSON` / `BIGINT` type mappings for equivalent columns.
///
/// Known differences we *expect*:
/// | Column              | PG                          | MySQL                        |
/// |---------------------|-----------------------------|------------------------------|
/// | public_id           | `DbType::Uuid`              | `DbType::Binary`             |
/// | created_at          | `DbType::TimestampTz`       | `DbType::Timestamp`          |
/// | last_login_ip       | `DbType::Inet`              | `DbType::Binary`             |
/// | user_type           | `DbType::SmallInt`          | `DbType::SmallInt` (via TINYINT UNSIGNED → SmallInt, unsigned info lost) |
#[tokio::test]
async fn pg_mysql_type_equivalence() {
    let Some(pg) = build_schema_postgres("xb_pg_mysql").await else {
        eprintln!("Skipping pg_mysql_type_equivalence (DATABASE_URL or PG unreachable)");
        return;
    };
    let Some(mysql) = build_schema_mysql("xb_pg_mysql").await else {
        eprintln!("Skipping pg_mysql_type_equivalence (MySQL unreachable)");
        teardown_pg("xb_pg_mysql").await;
        return;
    };

    let pg_map = field_map(&pg);
    let mysql_map = field_map(&mysql);

    // Columns that should have IDENTICAL DbType across PG and MySQL
    let equivalent = &[
        (("users", "id"), DbType::BigInt),
        (("users", "first_name"), DbType::String),
        (("users", "last_name"), DbType::String),
        (("users", "email"), DbType::String),
        (("users", "password"), DbType::String),
        (("users", "is_verified"), DbType::SmallInt),
        (("users", "is_active"), DbType::SmallInt),
        (("users", "remember_token"), DbType::String),

        // oauth_refresh_tokens
        (("oauth_refresh_tokens", "id"), DbType::BigInt),
        (("oauth_refresh_tokens", "token"), DbType::Binary),
        (("oauth_refresh_tokens", "revoked"), DbType::Boolean),

        // user_sessions
        (("user_sessions", "id"), DbType::BigInt),
        (("user_sessions", "is_revoked"), DbType::Boolean),
        (("user_sessions", "is_2fa_verified"), DbType::Boolean),
    ];

    for &((ref table, ref col), ref expected_ty) in equivalent {
        let pg_key = (table.to_string(), col.to_string());
        let mysql_key = (table.to_string(), col.to_string());
        let pg_val = pg_map.get(&pg_key).map(|(ty, _)| ty);
        let mysql_val = mysql_map.get(&mysql_key).map(|(ty, _)| ty);
        assert_eq!(
            pg_val,
            Some(expected_ty),
            "PG {table}.{col}: expected {expected_ty:?}, got {:?}",
            pg_val
        );
        assert_eq!(
            mysql_val,
            Some(expected_ty),
            "MySQL {table}.{col}: expected {expected_ty:?}, got {:?}",
            mysql_val
        );
    }

    // Known-different columns
    let differences: &[((&str, &str), DbType, DbType)] = &[
        (("users", "public_id"), DbType::Uuid, DbType::Binary),
        (("users", "created_at"), DbType::TimestampTz, DbType::Timestamp),
        (("users", "updated_at"), DbType::TimestampTz, DbType::Timestamp),
        (("users", "deleted_at"), DbType::TimestampTz, DbType::Timestamp),
        (("users", "last_login_ip"), DbType::Inet, DbType::Binary),
        (("users", "email_verified_at"), DbType::TimestampTz, DbType::Timestamp),
        (("users", "last_login_at"), DbType::TimestampTz, DbType::Timestamp),
        (("user_sessions", "metadata"), DbType::Jsonb, DbType::Json),
        (("user_sessions", "created_at"), DbType::TimestampTz, DbType::Timestamp),
        (("user_sessions", "updated_at"), DbType::TimestampTz, DbType::Timestamp),
        (("user_sessions", "last_activity"), DbType::TimestampTz, DbType::Timestamp),
        (("user_sessions", "expires_at"), DbType::TimestampTz, DbType::Timestamp),
        (("user_sessions", "revoked_at"), DbType::TimestampTz, DbType::Timestamp),
        (("user_sessions", "deleted_at"), DbType::TimestampTz, DbType::Timestamp),
        (("user_sessions", "ip_address"), DbType::Inet, DbType::Binary),
        (("oauth_refresh_tokens", "expires_at"), DbType::TimestampTz, DbType::Timestamp),
    ];

    for &((ref table, ref col), ref pg_ty, ref mysql_ty) in differences {
        let pg_key = (table.to_string(), col.to_string());
        let mysql_key = (table.to_string(), col.to_string());
        assert_eq!(
            pg_map.get(&pg_key).map(|(ty, _)| ty),
            Some(pg_ty),
            "PG {table}.{col}: expected {pg_ty:?}"
        );
        assert_eq!(
            mysql_map.get(&mysql_key).map(|(ty, _)| ty),
            Some(mysql_ty),
            "MySQL {table}.{col}: expected {mysql_ty:?}"
        );
    }

    drop(pg);
    drop(mysql);
    teardown_pg("xb_pg_mysql").await;
    teardown_mysql("xb_pg_mysql").await;
}

/// SQLite shares a subset of type mappings with PG/MySQL.
/// Where SQLite uses TEXT for everything, mapping to `DbType::String`,
/// compare with PG/MySQL String and Text columns.
#[tokio::test]
async fn sqlite_common_types() {
    let sqlite = build_schema_sqlite("xb_sqlite").await;
    let sqlite_map = field_map(&sqlite);

    // SQLite integer types map correctly
    let check_sqlite = &[
        (("users", "id"), DbType::Integer),
        (("users", "is_verified"), DbType::Integer),
        (("users", "user_type"), DbType::Integer),
        (("users", "is_active"), DbType::Integer),
    ];
    for &((ref table, ref col), ref expected) in check_sqlite {
        let key = (table.to_string(), col.to_string());
        assert_eq!(
            sqlite_map.get(&key).map(|(ty, _)| ty),
            Some(expected),
            "SQLite {table}.{col}: expected {expected:?}, got {:?}",
            sqlite_map.get(&key).map(|(ty, _)| ty)
        );
    }

    // SQLite TEXT maps to DbType::String
    let text_cols = &[
        (("users", "first_name"), false),
        (("users", "email"), false),
        (("users", "public_id"), false), // BLOB → DbType::Binary
    ];
    for &((ref table, ref col), nullable) in text_cols {
        let key = (table.to_string(), col.to_string());
        let (ty, n) = sqlite_map.get(&key).unwrap();
        assert_eq!(*n, nullable, "SQLite {table}.{col} nullable mismatch");
        if *col == "public_id" {
            assert_eq!(*ty, DbType::Binary, "SQLite {table}.{col}: expected Binary, got {ty:?}");
        } else {
            assert_eq!(*ty, DbType::String, "SQLite {table}.{col}: expected String, got {ty:?}");
        }
    }
}

/// Constraint parity: FK counts should agree across backends
/// for structurally identical schemas.
#[tokio::test]
async fn fk_parity_across_backends() {
    let sqlite = build_schema_sqlite("xb_fk").await;
    let sqlite_fk = fk_map(&sqlite);

    let pg = build_schema_postgres("xb_fk").await;
    let pg_fk = pg.as_ref().map(fk_map);

    let mysql = build_schema_mysql("xb_fk").await;
    let mysql_fk = mysql.as_ref().map(fk_map);

    // Every table that has FKs in SQLite should have them in PG and MySQL
    for (table, fks) in &sqlite_fk {
        let count = fks.len();
        if let Some(ref pg_fk) = pg_fk {
            let pg_count = pg_fk.get(table).map(|v| v.len()).unwrap_or(0);
            assert!(
                pg_count >= count,
                "PG table {table}: expected >= {count} FKs, got {pg_count}"
            );
        }
        if let Some(ref mysql_fk) = mysql_fk {
            let mysql_count = mysql_fk.get(table).map(|v| v.len()).unwrap_or(0);
            assert!(
                mysql_count >= count,
                "MySQL table {table}: expected >= {count} FKs, got {mysql_count}"
            );
        }
    }

    // Self-referencing FK on oauth_refresh_tokens in all backends
    let has_self_fk = |fk_map: &HashMap<String, Vec<_>>, table: &str| -> bool {
        fk_map.get(table).map_or(false, |fks| {
            fks.iter().any(|c:&&ConstraintIR| {
                matches!(&c.kind, neutrino_schema::ir::ConstraintKind::ForeignKey {
                    referenced_table, ..
                } if referenced_table == table)
            })
        })
    };

    assert!(
        has_self_fk(&sqlite_fk, "oauth_refresh_tokens"),
        "SQLite: expected self-FK on oauth_refresh_tokens"
    );
    if let Some(ref pg_fk) = pg_fk {
        assert!(
            has_self_fk(pg_fk, "oauth_refresh_tokens"),
            "PG: expected self-FK on oauth_refresh_tokens"
        );
    }
    if let Some(ref mysql_fk) = mysql_fk {
        assert!(
            has_self_fk(mysql_fk, "oauth_refresh_tokens"),
            "MySQL: expected self-FK on oauth_refresh_tokens"
        );
    }

    // Composite FK: user_sessions -> user_trusted_devices
    let has_composite = |fk_map: &HashMap<String, Vec<_>>| -> bool {
        fk_map.get("user_sessions").map_or(false, |fks| {
            fks.iter().any(|c:&&ConstraintIR| {
                matches!(&c.kind, neutrino_schema::ir::ConstraintKind::ForeignKey {
                    columns, referenced_table, referenced_columns, ..
                } if columns.len() == 2
                    && referenced_table == "user_trusted_devices"
                    && referenced_columns.len() == 2)
            })
        })
    };

    if let Some(ref pg_fk) = pg_fk {
        assert!(has_composite(pg_fk), "PG: expected composite FK on user_sessions");
    }
    if let Some(ref mysql_fk) = mysql_fk {
        assert!(has_composite(mysql_fk), "MySQL: expected composite FK on user_sessions");
    }

    if let Some(pg) = pg {
        drop(pg);
    }
    if let Some(mysql) = mysql {
        drop(mysql);
    }
    teardown_pg("xb_fk").await;
    teardown_mysql("xb_fk").await;
}

// ---------------------------------------------------------------------------
// Teardown
// ---------------------------------------------------------------------------

async fn teardown_pg(db_name: &str) {
    use sqlx::PgPool;
    if let Some(url) = database_url() {
        if let Ok(admin) = PgPool::connect(&url).await {
            let s: &'static str =
                &*Box::leak(format!("DROP DATABASE IF EXISTS \"{db_name}\"").into_boxed_str());
            sqlx::query(s).execute(&admin).await.ok();
        }
    }
}

async fn teardown_mysql(db_name: &str) {
    use sqlx::MySqlPool;
    if let Some(url) = database_url() {
        if let Ok(admin) = MySqlPool::connect(&url).await {
            let s: &'static str =
                &*Box::leak(format!("DROP DATABASE IF EXISTS `{db_name}`").into_boxed_str());
            sqlx::query(s).execute(&admin).await.ok();
        }
    }
}
