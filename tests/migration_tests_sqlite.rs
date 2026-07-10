mod common;

use common::migrations_common::{execute_sqlite_batch, load_migration_sql, MigrationBackend};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn find_constraint<'a>(
    table: &'a neutrino_schema::ir::TableIR,
    name: &str,
) -> Option<&'a neutrino_schema::ir::ConstraintIR> {
    table.constraints.iter().find(|c| c.name == name)
}

fn has_fk(table: &neutrino_schema::ir::TableIR, target: &str) -> bool {
    table.constraints.iter().any(|c| {
        matches!(
            &c.kind,
            neutrino_schema::ir::ConstraintKind::ForeignKey {
                referenced_table,
                ..
            } if referenced_table == target
        )
    })
}

async fn build_schema() -> neutrino_schema::SchemaIR {
    use neutrino_schema::introspect::DatabaseIntrospector;
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite in-memory connection");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .ok();

    let sql = load_migration_sql(MigrationBackend::Sqlite).expect("load sqlite migrations");
    execute_sqlite_batch(&pool, &sql)
        .await
        .expect("execute sqlite migrations");

    let introspector = neutrino_schema::introspect::SqliteIntrospector::new(pool);

    let table_names = introspector.list_tables().await.unwrap();
    let mut tables = Vec::new();
    for name in &table_names {
        let columns = introspector.list_columns(name).await.unwrap();
        let fields: Vec<_> = columns
            .iter()
            .map(|c| introspector.column_to_field(c))
            .collect();
        let constraints = introspector.list_constraints(name).await.unwrap();
        tables.push(neutrino_schema::ir::TableIR {
            name: name.clone(),
            fields,
            constraints,
        });
    }

    neutrino_schema::SchemaIR::from_tables(tables, neutrino_schema::ir::RelationStrategy::Disabled)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn sqlite_migration_discovery() {
    use neutrino_schema::introspect::DatabaseIntrospector;
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite in-memory connection");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .ok();

    let sql = load_migration_sql(MigrationBackend::Sqlite).expect("load sqlite migrations");
    execute_sqlite_batch(&pool, &sql)
        .await
        .expect("execute sqlite migrations");

    let introspector = neutrino_schema::introspect::SqliteIntrospector::new(pool);
    let tables = introspector.list_tables().await.unwrap();

    assert_eq!(tables.len(), 28, "expected 28 tables");

    for name in &[
        "users",
        "roles",
        "user_roles",
        "user_security_settings",
        "user_trusted_devices",
        "sessions",
        "user_sessions",
        "user_allowed_ips",
        "oauth_clients",
        "oauth_client_redirect_uris",
        "oauth_client_grants",
        "oauth_providers",
        "oauth_accounts",
        "oauth_access_tokens",
        "oauth_refresh_tokens",
        "oauth_authorization_codes",
        "oauth_device_codes",
        "personal_access_tokens",
        "password_reset_tokens",
        "email_verification_tokens",
        "remember_me_tokens",
        "security_events",
        "login_attempts",
        "account_lockouts",
        "ip_rate_limits",
        "network_restrictions",
        "network_restrictions_history",
        "two_factor_codes",
    ] {
        assert!(tables.contains(&name.to_string()), "missing table: {name}");
    }
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn sqlite_migration_column_order() {
    use neutrino_schema::introspect::DatabaseIntrospector;
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite in-memory connection");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .ok();

    let sql = load_migration_sql(MigrationBackend::Sqlite).expect("load sqlite migrations");
    execute_sqlite_batch(&pool, &sql)
        .await
        .expect("execute sqlite migrations");

    let introspector = neutrino_schema::introspect::SqliteIntrospector::new(pool);

    // Column order matters for generated structs
    let users_cols = introspector.list_columns("users").await.unwrap();
    assert_eq!(users_cols[0].column_name, "id");
    assert_eq!(users_cols[1].column_name, "public_id");
    assert_eq!(users_cols[2].column_name, "first_name");

    let sessions_cols = introspector.list_columns("sessions").await.unwrap();
    assert_eq!(sessions_cols[0].column_name, "session_id");
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn sqlite_migration_relations() {
    let schema = build_schema().await;

    // Self-referencing FK: oauth_refresh_tokens.previous_token_id -> oauth_refresh_tokens(id)
    let tokens = schema.table("oauth_refresh_tokens").unwrap();
    assert!(
        has_fk(tokens, "oauth_refresh_tokens"),
        "expected self-referencing FK on oauth_refresh_tokens"
    );

    // Multiple FKs to same target: roles.created_by/updated_by/deleted_by -> users(id)
    let roles = schema.table("roles").unwrap();
    let fks_to_users: Vec<_> = roles
        .constraints
        .iter()
        .filter(|c| {
            matches!(
                &c.kind,
                neutrino_schema::ir::ConstraintKind::ForeignKey {
                    referenced_table,
                    ..
                } if referenced_table == "users"
            )
        })
        .collect();
    assert_eq!(fks_to_users.len(), 3, "roles should have 3 FKs to users");

    // Bare REFERENCES without ON DELETE -> NoAction (all 52 SQLite FKs)
    let mut bare_fk_count = 0;
    for table in &schema.tables {
        for c in &table.constraints {
            if let neutrino_schema::ir::ConstraintKind::ForeignKey { on_delete, .. } = &c.kind {
                assert_eq!(
                    *on_delete,
                    neutrino_schema::ir::ReferentialAction::NoAction,
                    "FK {} on {} should be NoAction (SQLite bare REFERENCES)",
                    c.name,
                    table.name
                );
                bare_fk_count += 1;
            }
        }
    }
    assert_eq!(bare_fk_count, 52, "expected 52 FK constraints across all tables");
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn sqlite_migration_types() {
    use neutrino_schema::types::DbType;

    // SQLite affinity-based type mapping + sqlite_declared_to_db_type
    let schema = build_schema().await;

    // users table — type coverage
    let users = schema.table("users").unwrap();

    let public_id = users.fields.iter().find(|f| f.name == "public_id").unwrap();
    assert_eq!(public_id.ty, DbType::Binary, "public_id BLOB -> DbType::Binary");
    assert!(!public_id.nullable);

    let email = users.fields.iter().find(|f| f.name == "email").unwrap();
    assert_eq!(email.ty, DbType::String, "email TEXT -> DbType::String");
    assert!(!email.nullable);

    let is_active = users.fields.iter().find(|f| f.name == "is_active").unwrap();
    assert_eq!(is_active.ty, DbType::Integer, "is_active INTEGER -> DbType::Integer");
    assert!(!is_active.nullable);

    let last_login_ip = users.fields.iter().find(|f| f.name == "last_login_ip").unwrap();
    assert_eq!(last_login_ip.ty, DbType::String, "last_login_ip TEXT -> DbType::String");
    assert!(last_login_ip.nullable);

    let created_at = users.fields.iter().find(|f| f.name == "created_at").unwrap();
    assert_eq!(created_at.ty, DbType::String, "created_at TEXT -> DbType::String");
    assert!(!created_at.nullable);

    let deleted_at = users.fields.iter().find(|f| f.name == "deleted_at").unwrap();
    assert_eq!(deleted_at.ty, DbType::String, "deleted_at TEXT -> DbType::String");
    assert!(deleted_at.nullable);

    // roles table — non-autoincrement PK
    let roles = schema.table("roles").unwrap();
    let id = roles.fields.iter().find(|f| f.name == "id").unwrap();
    assert_eq!(id.ty, DbType::Integer, "roles.id INTEGER -> DbType::Integer");
    assert!(!id.nullable);

    // sessions — BLOB PK (session_id)
    let sessions = schema.table("sessions").unwrap();
    let session_id = sessions.fields.iter().find(|f| f.name == "session_id").unwrap();
    assert_eq!(session_id.ty, DbType::Binary, "session_id BLOB -> DbType::Binary");
    assert!(!session_id.nullable, "session_id PK should be NOT NULL");

    let session_data = sessions.fields.iter().find(|f| f.name == "session_data").unwrap();
    assert_eq!(session_data.ty, DbType::String, "session_data TEXT -> DbType::String");

    // oauth_device_codes — CHECK constraint status column
    let device_codes = schema.table("oauth_device_codes").unwrap();
    let status = device_codes.fields.iter().find(|f| f.name == "status").unwrap();
    assert_eq!(status.ty, DbType::String, "status TEXT -> DbType::String");
    assert!(!status.nullable);
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn sqlite_migration_constraints() {
    use neutrino_schema::ir::ConstraintKind;

    let schema = build_schema().await;

    // Every table should have a PK constraint
    for table in &schema.tables {
        let has_pk = table.constraints.iter().any(|c| {
            matches!(c.kind, ConstraintKind::PrimaryKey { .. })
        });
        assert!(has_pk, "table {} has no primary key", table.name);
    }

    // Check constraints: 4 tables with CHECK IN(...)
    let mut check_count = 0;
    for table in &schema.tables {
        for c in &table.constraints {
            if let ConstraintKind::Check { expression } = &c.kind {
                assert!(
                    expression.contains("IN ("),
                    "CHECK expression should contain IN (): {}",
                    expression
                );
                check_count += 1;
            }
        }
    }
    assert_eq!(check_count, 4, "expected 4 CHECK constraints");

    // Unique constraints: count across all tables.
    //
    // SQLite index names are unique per **database**, not per table. The migration
    // files use generic names like `idx_public_id` and `idx_expires_at` that collide
    // across tables, so only the first alphabetically succeeds; later duplicates
    // are silently skipped by IF NOT EXISTS. This is a migration-design concern,
    // not a neutrino-schema bug — we report what SQLite actually stores.
    //
    // The query uses `"unique" = 1 AND origin != 'pk'` to capture both inline
    // UNIQUE constraints (origin='u') and CREATE UNIQUE INDEX entries (origin='c').
    let mut unique_count = 0;
    for table in &schema.tables {
        for c in &table.constraints {
            if matches!(c.kind, ConstraintKind::Unique { .. }) {
                unique_count += 1;
            }
        }
    }
    assert_eq!(unique_count, 17, "expected 17 unique constraints, got {}", unique_count);
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn sqlite_migration_defaults() {
    use sqlx::Row;
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite in-memory connection");

    let sql = load_migration_sql(MigrationBackend::Sqlite).expect("load sqlite migrations");
    execute_sqlite_batch(&pool, &sql)
        .await
        .expect("execute sqlite migrations");

    // Query PRAGMA table_info for users to check dflt_value column
    // This column exists in PRAGMA output but is not captured by Column
    // (which has no default_value field yet).
    let rows = sqlx::query("SELECT name, dflt_value FROM pragma_table_info('users')")
        .fetch_all(&pool)
        .await
        .expect("PRAGMA table_info");

    let created_at_default: Option<String> = rows
        .iter()
        .find(|r| r.get::<String, _>("name") == "created_at")
        .map(|r| r.get("dflt_value"));

    assert!(
        created_at_default.is_some(),
        "created_at should have a default value (dflt_value)"
    );
    let default_str = created_at_default.unwrap();
    assert!(
        default_str.contains("strftime"),
        "expected strftime in created_at default, got: {}",
        default_str
    );

    // Check nullable columns have no default
    let deleted_at_default: Option<String> = rows
        .iter()
        .find(|r| r.get::<String, _>("name") == "deleted_at")
        .map(|r| r.get("dflt_value"));

    assert!(
        deleted_at_default.is_none() || deleted_at_default.as_deref() == Some("NULL"),
        "deleted_at should have no default, got: {:?}",
        deleted_at_default
    );
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn sqlite_migration_indexes() {
    use sqlx::Row;
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite in-memory connection");

    let sql = load_migration_sql(MigrationBackend::Sqlite).expect("load sqlite migrations");
    execute_sqlite_batch(&pool, &sql)
        .await
        .expect("execute sqlite migrations");

    // Count total indexes via PRAGMA.
    //
    // SQLite index names are unique per **database**, not per table. Many migration
    // files use the same index names (`idx_public_id` in ALL 28 files, `idx_user_id`
    // in many, `idx_expires_at` in 4), so only the first alphabetically succeeds.
    // Users is the last file alphabetically → most shared index names already taken.
    let rows = sqlx::query("SELECT COUNT(*) AS cnt FROM pragma_index_list('users')")
        .fetch_one(&pool)
        .await
        .expect("PRAGMA index_list for users");
    let users_index_count: i64 = rows.get("cnt");
    assert_eq!(users_index_count, 3, "users should have 3 indexes (idx_email_active, idx_email_unique, idx_email_verified_at)");

    // Roles: autoindex for inline UNIQUE on `name`
    let rows = sqlx::query("SELECT COUNT(*) AS cnt FROM pragma_index_list('roles')")
        .fetch_one(&pool)
        .await
        .expect("PRAGMA index_list for roles");
    let roles_index_count: i64 = rows.get("cnt");
    assert_eq!(roles_index_count, 1, "roles should have 1 index (autoindex for name UNIQUE)");

    // Sessions: only the BLOB PK autoindex; idx_expires_at and idx_public_id collide
    // with earlier files (oauth_access_tokens, account_lockouts respectively).
    let rows = sqlx::query("SELECT COUNT(*) AS cnt FROM pragma_index_list('sessions')")
        .fetch_one(&pool)
        .await
        .expect("PRAGMA index_list for sessions");
    let sessions_index_count: i64 = rows.get("cnt");
    assert_eq!(sessions_index_count, 1, "sessions should have 1 index (BLOB PK autoindex)");
}
