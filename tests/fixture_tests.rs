mod common;

// ---------------------------------------------------------------------------
// SQLite — always runs, in-memory
// ---------------------------------------------------------------------------

#[cfg(feature = "sqlite")]
mod sqlite {
    use neutrino_schema::{
        introspect::{DatabaseIntrospector, SqliteIntrospector},
        ir::ConstraintKind,
        types::DbType,
        RelationStrategy, SchemaIR,
    };
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup() -> SqliteIntrospector {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("sqlite in-memory connection");

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .ok();

        let sql = super::common::fixture::load_fixture("sqlite");
        let sql: &'static str = Box::leak(sql.into_boxed_str());
        for stmt in sql.split(';') {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(&pool).await.unwrap();
            }
        }

        SqliteIntrospector::new(pool)
    }

    #[tokio::test]
    async fn tables_exist() {
        let introspector = setup().await;
        let tables = introspector.list_tables().await.unwrap();
        for name in &["users", "posts", "tags", "post_tags", "profiles", "all_types"] {
            assert!(tables.contains(&name.to_string()), "missing table: {name}");
        }
    }

    #[tokio::test]
    async fn users_fields() {
        let introspector = setup().await;
        let columns = introspector.list_columns("users").await.unwrap();
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();

        let email = fields.iter().find(|f| f.name == "email").unwrap();
        assert_eq!(email.ty, DbType::String);
        assert!(!email.nullable);

        let bio = fields.iter().find(|f| f.name == "bio").unwrap();
        assert_eq!(bio.ty, DbType::String);
        assert!(bio.nullable);

        let age = fields.iter().find(|f| f.name == "age").unwrap();
        assert_eq!(age.ty, DbType::Integer);
        assert!(!age.nullable);

        let salary = fields.iter().find(|f| f.name == "salary").unwrap();
        assert_eq!(salary.ty, DbType::Float64);
        assert!(salary.nullable);

        let is_active = fields.iter().find(|f| f.name == "is_active").unwrap();
        assert_eq!(is_active.ty, DbType::Integer);
    }

    #[tokio::test]
    async fn all_types_fields() {
        let introspector = setup().await;
        let columns = introspector.list_columns("all_types").await.unwrap();
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();

        assert_eq!(
            fields.iter().find(|f| f.name == "small_int_value").unwrap().ty,
            DbType::Integer
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "integer_value").unwrap().ty,
            DbType::Integer
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "bigint_value").unwrap().ty,
            DbType::Integer
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "real_value").unwrap().ty,
            DbType::Float64
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "text_value").unwrap().ty,
            DbType::String
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "varchar_value").unwrap().ty,
            DbType::String
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "blob_value").unwrap().ty,
            DbType::Binary
        );

        assert!(matches!(
            fields.iter().find(|f| f.name == "json_value").unwrap().ty,
            DbType::Unknown(_)
        ));
        assert!(matches!(
            fields.iter().find(|f| f.name == "date_value").unwrap().ty,
            DbType::Unknown(_)
        ));
        assert!(matches!(
            fields.iter().find(|f| f.name == "datetime_value").unwrap().ty,
            DbType::Unknown(_)
        ));
    }

    #[tokio::test]
    async fn constraints() {
        let introspector = setup().await;

        let user_constraints = introspector.list_constraints("users").await.unwrap();

        // Primary key (generated name: users_pk)
        assert!(user_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { .. })));
        assert!(user_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { columns } if columns == &vec!["id"])));
 
         // UNIQUE on email — SQLite PRAGMA returns auto-generated index name
        assert!(user_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::Unique { columns } if columns == &vec!["email"])));

        // CHECK constraint parsed from CREATE TABLE SQL preserves the name
        assert!(user_constraints.iter().any(|c| c.name == "users_age_check"));

        // mood CHECK (inline, no CONSTRAINT name)
        assert!(user_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::Check { .. })));

        let post_constraints = introspector.list_constraints("posts").await.unwrap();
        assert!(post_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { .. })));

        // FK name is auto-generated as {table}_{column}_fk
        assert!(post_constraints
            .iter()
            .any(|c| c.name == "posts_user_id_fk"));

        // CHECK constraint parsed from SQL preserves the name
        assert!(post_constraints.iter().any(|c| c.name == "posts_title_check"));

        let tag_constraints = introspector.list_constraints("tags").await.unwrap();
        // UNIQUE on name — auto-generated name
        assert!(tag_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::Unique { columns } if columns == &vec!["name"])));

        let pt_constraints = introspector.list_constraints("post_tags").await.unwrap();
        assert!(pt_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { columns } if columns.len() == 2)));

        let profile_constraints = introspector.list_constraints("profiles").await.unwrap();
        // UNIQUE on email — auto-generated
        assert!(profile_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::Unique { columns } if columns == &vec!["email"])));
     }

    #[tokio::test]
    async fn fk_relations_and_table_accessor() {
        let introspector = setup().await;
        let table_names = introspector.list_tables().await.unwrap();
        let mut tables = Vec::new();
        for name in &table_names {
            let columns = introspector.list_columns(name).await.unwrap();
            let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
            let constraints = introspector.list_constraints(name).await.unwrap();
            tables.push(neutrino_schema::ir::TableIR {
                name: name.clone(),
                fields,
                constraints,
            });
        }
        let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

        let fk = schema
            .relations
            .iter()
            .find(|r| r.from_table == "posts");
        assert!(fk.is_some());
        assert_eq!(fk.unwrap().from_field, "user_id");

        assert!(schema.table("users").is_some());
        assert!(schema.table("nonexistent").is_none());
    }
}

// ---------------------------------------------------------------------------
// PostgreSQL — runs if DATABASE_URL is set
// ---------------------------------------------------------------------------

#[cfg(feature = "postgres")]
mod postgres {
    use neutrino_schema::{
        introspect::{DatabaseIntrospector, PostgresIntrospector},
        ir::ConstraintKind,
        types::DbType,
        RelationStrategy, SchemaIR,
    };
    use sqlx::PgPool;

    fn database_url() -> Option<String> {
        std::env::var("DATABASE_URL").ok()
    }

    async fn setup(db_name: &str) -> Option<PostgresIntrospector> {
        let url = database_url()?;
        let admin = PgPool::connect(&url).await.ok()?;

        let sql: &'static str = &*Box::leak(format!("DROP DATABASE IF EXISTS \"{db_name}\"").into_boxed_str());
        sqlx::query(sql).execute(&admin).await.ok();
        let sql: &'static str = &*Box::leak(format!("CREATE DATABASE \"{db_name}\"").into_boxed_str());
        sqlx::query(sql).execute(&admin).await.ok()?;
        admin.close().await;

        let fixture_url = format!("{}/{}", url.trim_end_matches('/'), db_name);
        let pool = PgPool::connect(&fixture_url).await.ok()?;

        let sql = crate::common::fixture::load_fixture("postgres");
        let sql: &'static str = Box::leak(sql.into_boxed_str());
        for stmt in sql.split(';') {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(&pool).await.unwrap();
            }
        }

        Some(PostgresIntrospector::new(pool))
    }

    async fn teardown(db_name: &str) {
        if let Some(url) = database_url() {
            if let Ok(admin) = PgPool::connect(&url).await {
                let s: &'static str = &*Box::leak(format!("DROP DATABASE IF EXISTS \"{db_name}\"").into_boxed_str());
                sqlx::query(s).execute(&admin).await.ok();
            }
        }
    }

    #[tokio::test]
    async fn tables_exist() {
        let Some(introspector) = setup("ns_fixture_pg_1").await else {
            eprintln!("Skipping postgres::tables_exist (DATABASE_URL not set)");
            return;
        };
        let tables = introspector.list_tables().await.unwrap();
        for name in &["users", "posts", "tags", "post_tags", "profiles", "all_types"] {
            assert!(tables.contains(&name.to_string()), "missing table: {name}");
        }
        drop(introspector);
        teardown("ns_fixture_pg_1").await;
    }

    #[tokio::test]
    async fn users_fields() {
        let Some(introspector) = setup("ns_fixture_pg_2").await else {
            eprintln!("Skipping postgres::users_fields (DATABASE_URL not set)");
            return;
        };
        let columns = introspector.list_columns("users").await.unwrap();
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();

        let email = fields.iter().find(|f| f.name == "email").unwrap();
        assert_eq!(email.ty, DbType::String);
        assert!(!email.nullable);

        let full_name = fields.iter().find(|f| f.name == "full_name").unwrap();
        assert_eq!(full_name.ty, DbType::Text);
        assert!(!full_name.nullable);

        let bio = fields.iter().find(|f| f.name == "bio").unwrap();
        assert_eq!(bio.ty, DbType::Text);
        assert!(bio.nullable);

        let age = fields.iter().find(|f| f.name == "age").unwrap();
        assert_eq!(age.ty, DbType::Integer);
        assert!(!age.nullable);

        let salary = fields.iter().find(|f| f.name == "salary").unwrap();
        assert_eq!(salary.ty, DbType::Decimal);
        assert!(salary.nullable);

        let is_active = fields.iter().find(|f| f.name == "is_active").unwrap();
        assert_eq!(is_active.ty, DbType::Boolean);
        assert!(!is_active.nullable);

        drop(introspector);
        teardown("ns_fixture_pg_2").await;
    }

    #[tokio::test]
    async fn all_types_fields() {
        let Some(introspector) = setup("ns_fixture_pg_3").await else {
            eprintln!("Skipping postgres::all_types_fields (DATABASE_URL not set)");
            return;
        };
        let columns = introspector.list_columns("all_types").await.unwrap();
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();

        assert_eq!(
            fields.iter().find(|f| f.name == "small_int_value").unwrap().ty,
            DbType::SmallInt
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "integer_value").unwrap().ty,
            DbType::Integer
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "bigint_value").unwrap().ty,
            DbType::BigInt
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "serial_value").unwrap().ty,
            DbType::Integer
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "bigserial_value").unwrap().ty,
            DbType::BigInt
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "numeric_value").unwrap().ty,
            DbType::Decimal
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "real_value").unwrap().ty,
            DbType::Float32
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "double_value").unwrap().ty,
            DbType::Float64
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "varchar_value").unwrap().ty,
            DbType::String
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "text_value").unwrap().ty,
            DbType::Text
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "boolean_value").unwrap().ty,
            DbType::Boolean
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "bytea_value").unwrap().ty,
            DbType::Binary
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "date_value").unwrap().ty,
            DbType::Date
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "time_value").unwrap().ty,
            DbType::Time
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "timestamp_value").unwrap().ty,
            DbType::Timestamp
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "timestamptz_value").unwrap().ty,
            DbType::TimestampTz
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "json_value").unwrap().ty,
            DbType::Json
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "jsonb_value").unwrap().ty,
            DbType::Jsonb
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "uuid_value").unwrap().ty,
            DbType::Uuid
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "inet_value").unwrap().ty,
            DbType::Inet
        );

        let mood = fields.iter().find(|f| f.name == "mood_value").unwrap();
        assert!(matches!(mood.ty, DbType::Unknown(_)));
        assert_eq!(mood.raw_type, "USER-DEFINED");

        let arr = fields.iter().find(|f| f.name == "text_array_value").unwrap();
        assert!(matches!(arr.ty, DbType::Array(_)));

        drop(introspector);
        teardown("ns_fixture_pg_3").await;
    }

    #[tokio::test]
    async fn constraints() {
        let Some(introspector) = setup("ns_fixture_pg_4").await else {
            eprintln!("Skipping postgres::constraints (DATABASE_URL not set)");
            return;
        };

        let user_constraints = introspector.list_constraints("users").await.unwrap();
        assert!(user_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { .. })));
        assert!(user_constraints.iter().any(|c| c.name == "users_email_unique"));
        assert!(user_constraints.iter().any(|c| c.name == "users_age_check"));

        let post_constraints = introspector.list_constraints("posts").await.unwrap();
        assert!(post_constraints.iter().any(|c| c.name == "posts_user_id_fkey"));
        assert!(post_constraints.iter().any(|c| c.name == "posts_title_check"));

        let pt_constraints = introspector.list_constraints("post_tags").await.unwrap();
        assert!(pt_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { columns } if columns.len() == 2)));

        let profile_constraints = introspector.list_constraints("profiles").await.unwrap();
        assert!(profile_constraints.iter().any(|c| c.name == "profiles_email_unique"));

        drop(introspector);
        teardown("ns_fixture_pg_4").await;
    }

    #[tokio::test]
    async fn enums_discovered() {
        let Some(introspector) = setup("ns_fixture_pg_5").await else {
            eprintln!("Skipping postgres::enums_discovered (DATABASE_URL not set)");
            return;
        };
        let enums = introspector.introspect_enums().await.unwrap();
        let mood = enums.iter().find(|e| e.database_name == "mood");
        assert!(mood.is_some(), "mood enum should be discovered");
        let mood = mood.unwrap();
        assert_eq!(mood.variants.len(), 3);
        assert_eq!(mood.variants[0].database_name, "sad");
        assert_eq!(mood.variants[1].database_name, "ok");
        assert_eq!(mood.variants[2].database_name, "happy");

        drop(introspector);
        teardown("ns_fixture_pg_5").await;
    }

    #[tokio::test]
    async fn fk_relations_and_table_accessor() {
        let Some(introspector) = setup("ns_fixture_pg_6").await else {
            eprintln!("Skipping postgres::fk_relations (DATABASE_URL not set)");
            return;
        };
        let table_names = introspector.list_tables().await.unwrap();
        let mut tables = Vec::new();
        for name in &table_names {
            let columns = introspector.list_columns(name).await.unwrap();
            let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
            let constraints = introspector.list_constraints(name).await.unwrap();
            tables.push(neutrino_schema::ir::TableIR {
                name: name.clone(),
                fields,
                constraints,
            });
        }
        let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

        assert!(schema.table("users").is_some());
        assert!(schema.table("posts").is_some());
        assert!(schema.table("nonexistent").is_none());

        let fk = schema
            .relations
            .iter()
            .find(|r| r.from_table == "posts");
        assert!(fk.is_some(), "FK relation from posts should exist");
        assert_eq!(fk.unwrap().from_field, "user_id");

        drop(introspector);
        teardown("ns_fixture_pg_6").await;
    }
}

// ---------------------------------------------------------------------------
// MySQL — runs if DATABASE_URL is set
// ---------------------------------------------------------------------------

#[cfg(feature = "mysql")]
mod mysql {
    use neutrino_schema::{
        introspect::{DatabaseIntrospector, MysqlIntrospector},
        ir::ConstraintKind,
        types::DbType,
        RelationStrategy, SchemaIR,
    };
    use sqlx::MySqlPool;

    fn admin_url() -> Option<String> {
        std::env::var("DATABASE_URL")
            .ok()
            .or_else(|| Some("mysql://root@localhost:3306".to_string()))
    }

    async fn setup(db_name: &str) -> Option<MysqlIntrospector> {
        let admin_url = admin_url()?;
        let admin = MySqlPool::connect(&format!("{admin_url}/mysql")).await.ok()?;

        let sql: &'static str = &*Box::leak(format!("DROP DATABASE IF EXISTS `{db_name}`").into_boxed_str());
        sqlx::query(sql).execute(&admin).await.ok();
        let sql: &'static str = &*Box::leak(format!("CREATE DATABASE `{db_name}`").into_boxed_str());
        sqlx::query(sql).execute(&admin).await.ok()?;
        admin.close().await;

        let pool = MySqlPool::connect(&format!("{admin_url}/{db_name}"))
            .await
            .ok()?;

        let sql = crate::common::fixture::load_fixture("mysql");
        let sql: &'static str = Box::leak(sql.into_boxed_str());
        for stmt in sql.split(';') {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(&pool).await.unwrap();
            }
        }

        Some(MysqlIntrospector::new(pool))
    }

    async fn teardown(db_name: &str) {
        if let Some(admin_url) = admin_url() {
            if let Ok(admin) = MySqlPool::connect(&format!("{admin_url}/mysql")).await {
                let s: &'static str = &*Box::leak(format!("DROP DATABASE IF EXISTS `{db_name}`").into_boxed_str());
                sqlx::query(s).execute(&admin).await.ok();
            }
        }
    }

    #[tokio::test]
    async fn tables_exist() {
        let Some(introspector) = setup("ns_fixture_my_1").await else {
            eprintln!("Skipping mysql::tables_exist (MySQL unreachable)");
            return;
        };
        let tables = introspector.list_tables().await.unwrap();
        for name in &["users", "posts", "tags", "post_tags", "profiles", "all_types"] {
            assert!(tables.contains(&name.to_string()), "missing table: {name}");
        }
        drop(introspector);
        teardown("ns_fixture_my_1").await;
    }

    #[tokio::test]
    async fn users_fields() {
        let Some(introspector) = setup("ns_fixture_my_2").await else {
            eprintln!("Skipping mysql::users_fields (MySQL unreachable)");
            return;
        };
        let columns = introspector.list_columns("users").await.unwrap();
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();

        let email = fields.iter().find(|f| f.name == "email").unwrap();
        assert_eq!(email.ty, DbType::String);
        assert!(!email.nullable);

        let bio = fields.iter().find(|f| f.name == "bio").unwrap();
        assert_eq!(bio.ty, DbType::Text);
        assert!(bio.nullable);

        let age = fields.iter().find(|f| f.name == "age").unwrap();
        assert_eq!(age.ty, DbType::Integer);
        assert!(!age.nullable);

        let salary = fields.iter().find(|f| f.name == "salary").unwrap();
        assert_eq!(salary.ty, DbType::Decimal);
        assert!(salary.nullable);

        let is_active = fields.iter().find(|f| f.name == "is_active").unwrap();
        assert_eq!(is_active.ty, DbType::SmallInt);
        assert!(!is_active.nullable);

        drop(introspector);
        teardown("ns_fixture_my_2").await;
    }

    #[tokio::test]
    async fn all_types_fields() {
        let Some(introspector) = setup("ns_fixture_my_3").await else {
            eprintln!("Skipping mysql::all_types_fields (MySQL unreachable)");
            return;
        };
        let columns = introspector.list_columns("all_types").await.unwrap();
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();

        assert_eq!(
            fields.iter().find(|f| f.name == "tiny_int_value").unwrap().ty,
            DbType::SmallInt
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "small_int_value").unwrap().ty,
            DbType::SmallInt
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "medium_int_value").unwrap().ty,
            DbType::Integer
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "integer_value").unwrap().ty,
            DbType::Integer
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "bigint_value").unwrap().ty,
            DbType::BigInt
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "decimal_value").unwrap().ty,
            DbType::Decimal
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "float_value").unwrap().ty,
            DbType::Float32
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "double_value").unwrap().ty,
            DbType::Float64
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "varchar_value").unwrap().ty,
            DbType::String
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "text_value").unwrap().ty,
            DbType::Text
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "enum_value").unwrap().ty,
            DbType::String
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "enum_value").unwrap().raw_type,
            "enum"
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "json_value").unwrap().ty,
            DbType::Json
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "blob_value").unwrap().ty,
            DbType::Binary
        );

        assert_eq!(
            fields.iter().find(|f| f.name == "datetime_value").unwrap().ty,
            DbType::Timestamp
        );
        assert_eq!(
            fields.iter().find(|f| f.name == "timestamp_value").unwrap().ty,
            DbType::Timestamp
        );

        drop(introspector);
        teardown("ns_fixture_my_3").await;
    }

    #[tokio::test]
    async fn constraints() {
        let Some(introspector) = setup("ns_fixture_my_4").await else {
            eprintln!("Skipping mysql::constraints (MySQL unreachable)");
            return;
        };

        let user_constraints = introspector.list_constraints("users").await.unwrap();
        assert!(user_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { .. })));
        assert!(user_constraints.iter().any(|c| c.name == "users_email_unique"));
        assert!(user_constraints.iter().any(|c| c.name == "users_age_check"));

        let post_constraints = introspector.list_constraints("posts").await.unwrap();
        assert!(post_constraints.iter().any(|c| c.name == "posts_user_id_fkey"));
        assert!(post_constraints.iter().any(|c| c.name == "posts_title_check"));

        let pt_constraints = introspector.list_constraints("post_tags").await.unwrap();
        assert!(pt_constraints
            .iter()
            .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { columns } if columns.len() == 2)));

        let profile_constraints = introspector.list_constraints("profiles").await.unwrap();
        assert!(profile_constraints.iter().any(|c| c.name == "profiles_email_unique"));

        drop(introspector);
        teardown("ns_fixture_my_4").await;
    }

    #[tokio::test]
    async fn fk_relations_and_table_accessor() {
        let Some(introspector) = setup("ns_fixture_my_5").await else {
            eprintln!("Skipping mysql::fk_relations (MySQL unreachable)");
            return;
        };
        let table_names = introspector.list_tables().await.unwrap();
        let mut tables = Vec::new();
        for name in &table_names {
            let columns = introspector.list_columns(name).await.unwrap();
            let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
            let constraints = introspector.list_constraints(name).await.unwrap();
            tables.push(neutrino_schema::ir::TableIR {
                name: name.clone(),
                fields,
                constraints,
            });
        }
        let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

        assert!(schema.table("users").is_some());
        assert!(schema.table("posts").is_some());

        let fk = schema
            .relations
            .iter()
            .find(|r| r.from_table == "posts");
        assert!(fk.is_some(), "FK relation from posts should exist");
        assert_eq!(fk.unwrap().from_field, "user_id");

        drop(introspector);
        teardown("ns_fixture_my_5").await;
    }
}
