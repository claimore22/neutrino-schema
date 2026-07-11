mod common;

use common::migrations_common::{load_migration_sql, MigrationBackend};

const MYSQL_DB_PREFIX: &str = "ns_mig_my_";

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

async fn setup(db_suffix: &str) -> Option<neutrino_schema::introspect::MysqlIntrospector> {
    use sqlx::MySqlPool;

    let url = database_url()?;
    let admin = MySqlPool::connect(&url).await.ok()?;
    let db_name = format!("{}{}", MYSQL_DB_PREFIX, db_suffix);

    let drop_sql: &'static str =
        &*Box::leak(format!("DROP DATABASE IF EXISTS `{db_name}`").into_boxed_str());
    sqlx::query(drop_sql).execute(&admin).await.ok();
    let create_sql: &'static str =
        &*Box::leak(format!("CREATE DATABASE `{db_name}`").into_boxed_str());
    sqlx::query(create_sql).execute(&admin).await.ok()?;
    admin.close().await;

    let fixture_url = format!("{}/{}", url.trim_end_matches('/'), db_name);
    let pool = MySqlPool::connect(&fixture_url).await.ok()?;

    let sql = load_migration_sql(MigrationBackend::Mysql).ok()?;
    let sql: &'static str = Box::leak(sql.into_boxed_str());
    for stmt in sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await.ok();
        }
    }

    Some(neutrino_schema::introspect::MysqlIntrospector::new(pool))
}

async fn teardown(db_suffix: &str) {
    use sqlx::MySqlPool;
    if let Some(url) = database_url() {
        if let Ok(admin) = MySqlPool::connect(&url).await {
            let db_name = format!("{}{}", MYSQL_DB_PREFIX, db_suffix);
            let s: &'static str =
                &*Box::leak(format!("DROP DATABASE IF EXISTS `{db_name}`").into_boxed_str());
            sqlx::query(s).execute(&admin).await.ok();
        }
    }
}

async fn build_schema(db_suffix: &str) -> Option<neutrino_schema::SchemaIR> {
    use neutrino_schema::introspect::DatabaseIntrospector;

    let introspector = setup(db_suffix).await?;
    let table_infos = introspector.list_tables_with_info().await.ok()?;
    let mut tables = Vec::new();
    for info in &table_infos {
        let columns = introspector.list_columns(&info.name).await.ok()?;
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
        let constraints = introspector.list_constraints(&info.name).await.ok()?;
        tables.push(neutrino_schema::ir::TableIR {
            name: info.name.to_string(),
            fields,
            constraints,
            comment: info.comment.clone(),
        });
    }
    let schema = neutrino_schema::SchemaIR::from_tables(
        tables,
        neutrino_schema::ir::RelationStrategy::Disabled,
    );
    drop(introspector);
    Some(schema)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(feature = "mysql")]
#[tokio::test]
async fn mysql_migration_discovery() {
    use neutrino_schema::introspect::{DatabaseIntrospector, TableInfo};

    let Some(introspector) = setup("discovery").await else {
        eprintln!("Skipping mysql::migration_discovery (DATABASE_URL not set)");
        return;
    };
    let tables: Vec<TableInfo> = introspector.list_tables_with_info().await.expect("list_tables failed");
    assert_eq!(tables.len(), 28, "expected 28 tables");
    let table_names: Vec<&str> = tables.iter().map(|ti| ti.name.as_str()).collect();

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
        assert!(
            table_names.contains(name),
            "missing table: {name}"
        );
    }
    drop(introspector);
    teardown("discovery").await;
}

#[cfg(feature = "mysql")]
#[tokio::test]
async fn mysql_migration_column_order() {
    use neutrino_schema::introspect::DatabaseIntrospector;

    let Some(introspector) = setup("columns").await else {
        eprintln!("Skipping mysql::migration_column_order (DATABASE_URL not set)");
        return;
    };
    let cols = introspector.list_columns("users").await.expect("list_columns for users failed");
    assert_eq!(cols[0].column_name, "id");
    assert_eq!(cols[1].column_name, "public_id");
    assert_eq!(cols[2].column_name, "first_name");
    drop(introspector);
    teardown("columns").await;
}

#[cfg(feature = "mysql")]
#[tokio::test]
async fn mysql_migration_types() {
    use neutrino_schema::types::DbType;

    let Some(schema) = build_schema("types").await else {
        eprintln!("Skipping mysql::migration_types (DATABASE_URL not set)");
        return;
    };

    let users = schema.table("users").expect("users table not found in schema");

    let id = users.fields.iter().find(|f| f.name == "id").expect("users.id field not found");
    assert_eq!(id.ty, DbType::BigInt, "id BIGINT -> DbType::BigInt");
    assert!(!id.nullable);

    let public_id = users.fields.iter().find(|f| f.name == "public_id").expect("users.public_id field not found");
    assert_eq!(public_id.ty, DbType::Binary, "public_id BINARY(16) -> DbType::Binary");
    assert!(!public_id.nullable);

    let first_name = users.fields.iter().find(|f| f.name == "first_name").expect("users.first_name field not found");
    assert_eq!(first_name.ty, DbType::String, "first_name VARCHAR -> DbType::String");
    assert!(!first_name.nullable);

    let is_active = users.fields.iter().find(|f| f.name == "is_active").expect("users.is_active field not found");
    assert_eq!(is_active.ty, DbType::SmallInt, "is_active TINYINT(1) -> DbType::SmallInt");
    assert!(!is_active.nullable);

    let user_agent = users.fields.iter().find(|f| f.name == "user_agent").expect("users.user_agent field not found");
    assert_eq!(user_agent.ty, DbType::Text, "user_agent TEXT -> DbType::Text");
    assert!(user_agent.nullable);

    let created_at = users.fields.iter().find(|f| f.name == "created_at").expect("users.created_at field not found");
    assert_eq!(created_at.ty, DbType::Timestamp, "created_at TIMESTAMP -> DbType::Timestamp");
    assert!(!created_at.nullable);

    let deleted_at = users.fields.iter().find(|f| f.name == "deleted_at").expect("users.deleted_at field not found");
    assert_eq!(deleted_at.ty, DbType::Timestamp, "deleted_at TIMESTAMP -> DbType::Timestamp");
    assert!(deleted_at.nullable);

    // user_sessions — JSON (not JSONB in MySQL)
    let sessions = schema.table("user_sessions").expect("user_sessions table not found in schema");
    let metadata = sessions.fields.iter().find(|f| f.name == "metadata").expect("user_sessions.metadata field not found");
    assert_eq!(metadata.ty, DbType::Json, "metadata JSON -> DbType::Json");
    assert!(metadata.nullable);

    // security_events — ip_address stored as VARBINARY(16)
    let events = schema.table("security_events").expect("security_events table not found in schema");
    let ip = events.fields.iter().find(|f| f.name == "ip_address").expect("security_events.ip_address field not found");
    assert_eq!(ip.ty, DbType::Binary, "ip_address VARBINARY(16) -> DbType::Binary");
    assert!(ip.nullable);

    // oauth_device_codes — poll_interval INT UNSIGNED
    let codes = schema.table("oauth_device_codes").expect("oauth_device_codes table not found in schema");
    let poll = codes.fields.iter().find(|f| f.name == "poll_interval").expect("oauth_device_codes.poll_interval field not found");
    assert_eq!(poll.ty, DbType::Integer, "poll_interval INT UNSIGNED -> DbType::Integer");
    assert!(!poll.nullable);

    teardown("types").await;
}

#[cfg(feature = "mysql")]
#[tokio::test]
async fn mysql_migration_constraints() {
    use neutrino_schema::ir::ConstraintKind;

    let Some(schema) = build_schema("constraints").await else {
        eprintln!("Skipping mysql::migration_constraints (DATABASE_URL not set)");
        return;
    };

    // Every table should have a PK constraint
    for table in &schema.tables {
        let has_pk = table
            .constraints
            .iter()
            .any(|c| matches!(c.kind, ConstraintKind::PrimaryKey { .. }));
        assert!(has_pk, "table {} has no primary key", table.name);
    }

    // Unique constraints: count across all tables
    let mut unique_count = 0;
    for table in &schema.tables {
        for c in &table.constraints {
            if matches!(c.kind, ConstraintKind::Unique { .. }) {
                unique_count += 1;
            }
        }
    }
    assert!(
        unique_count >= 20,
        "expected at least 20 unique constraints, got {}",
        unique_count
    );

    teardown("constraints").await;
}

#[cfg(feature = "mysql")]
#[tokio::test]
async fn mysql_migration_relations() {
    use neutrino_schema::ir::ConstraintKind;

    let Some(schema) = build_schema("relations").await else {
        eprintln!("Skipping mysql::migration_relations (DATABASE_URL not set)");
        return;
    };

    // Self-referencing FK: oauth_refresh_tokens -> self
    let tokens = schema.table("oauth_refresh_tokens").expect("oauth_refresh_tokens table not found in schema");
    let self_fk = tokens.constraints.iter().find(|c| {
        matches!(&c.kind, ConstraintKind::ForeignKey { referenced_table, .. }
            if referenced_table == "oauth_refresh_tokens")
    });
    assert!(
        self_fk.is_some(),
        "expected self-referencing FK on oauth_refresh_tokens"
    );

    // Multiple FKs to users: roles.created_by/updated_by/deleted_by -> users
    let roles = schema.table("roles").expect("roles table not found in schema");
    let fks_to_users: Vec<_> = roles
        .constraints
        .iter()
        .filter(|c| {
            matches!(&c.kind, ConstraintKind::ForeignKey { referenced_table, .. }
                if referenced_table == "users")
        })
        .collect();
    assert_eq!(fks_to_users.len(), 3, "roles should have 3 FKs to users");

    // Composite FK: user_sessions(user_id, device_id) -> user_trusted_devices(user_id, device_id)
    let sessions = schema.table("user_sessions").expect("user_sessions table not found in schema");
    let composite_fk = sessions.constraints.iter().find(|c| {
        matches!(&c.kind, ConstraintKind::ForeignKey {
            columns, referenced_table, referenced_columns, ..
        } if columns.len() == 2
            && referenced_table == "user_trusted_devices"
            && referenced_columns.len() == 2)
    });
    assert!(
        composite_fk.is_some(),
        "expected composite FK on user_sessions -> user_trusted_devices"
    );

    // FK total count across all tables (minimum)
    let mut fk_count = 0;
    for table in &schema.tables {
        for c in &table.constraints {
            if matches!(c.kind, ConstraintKind::ForeignKey { .. }) {
                fk_count += 1;
            }
        }
    }
    assert!(
        fk_count >= 50,
        "expected at least 50 FK constraints, got {}",
        fk_count
    );

    teardown("relations").await;
}
