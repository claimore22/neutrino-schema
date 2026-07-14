mod common;

// ---------------------------------------------------------------------------
// SQLite — always runs, in-memory
// ---------------------------------------------------------------------------

#[cfg(feature = "sqlite")]
mod sqlite {
    use neutrino_schema::{
        RelationStrategy, SchemaIR,
        introspect::{DatabaseIntrospector, SqliteIntrospector},
        ir::ConstraintKind,
        types::DbType,
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
                sqlx::query(trimmed)
                    .execute(&pool)
                    .await
                    .expect("fixture SQL should execute");
            }
        }

        SqliteIntrospector::new(pool)
    }

    #[tokio::test]
    async fn tables_exist() {
        let introspector = setup().await;
        let table_infos = introspector
            .list_tables_with_info()
            .await
            .expect("list_tables failed");
        for name in &[
            "users",
            "posts",
            "tags",
            "post_tags",
            "profiles",
            "all_types",
        ] {
            assert!(
                table_infos.iter().any(|ti| ti.name == *name),
                "missing table: {name}"
            );
        }
    }

    #[tokio::test]
    async fn users_fields() {
        let introspector = setup().await;
        let columns = introspector
            .list_columns("users")
            .await
            .expect("list_columns for users failed");
        let fields: Vec<_> = columns
            .iter()
            .map(|c| introspector.column_to_field(c))
            .collect();

        let email = fields
            .iter()
            .find(|f| f.name == "email")
            .expect("users.email field not found");
        assert_eq!(email.ty, DbType::String);
        assert!(!email.nullable);

        let bio = fields
            .iter()
            .find(|f| f.name == "bio")
            .expect("users.bio field not found");
        assert_eq!(bio.ty, DbType::String);
        assert!(bio.nullable);

        let age = fields
            .iter()
            .find(|f| f.name == "age")
            .expect("users.age field not found");
        assert_eq!(age.ty, DbType::Integer);
        assert!(!age.nullable);

        let salary = fields
            .iter()
            .find(|f| f.name == "salary")
            .expect("users.salary field not found");
        assert_eq!(salary.ty, DbType::Float64);
        assert!(salary.nullable);

        let is_active = fields
            .iter()
            .find(|f| f.name == "is_active")
            .expect("users.is_active field not found");
        assert_eq!(is_active.ty, DbType::Integer);
    }

    #[tokio::test]
    async fn all_types_fields() {
        let introspector = setup().await;
        let columns = introspector
            .list_columns("all_types")
            .await
            .expect("list_columns for all_types failed");
        let fields: Vec<_> = columns
            .iter()
            .map(|c| introspector.column_to_field(c))
            .collect();

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "small_int_value")
                .expect("all_types.small_int_value field not found")
                .ty,
            DbType::SmallInt
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "integer_value")
                .expect("all_types.integer_value field not found")
                .ty,
            DbType::Integer
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "bigint_value")
                .expect("all_types.bigint_value field not found")
                .ty,
            DbType::BigInt
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "real_value")
                .expect("all_types.real_value field not found")
                .ty,
            DbType::Float64
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "text_value")
                .expect("all_types.text_value field not found")
                .ty,
            DbType::String
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "varchar_value")
                .expect("all_types.varchar_value field not found")
                .ty,
            DbType::String
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "blob_value")
                .expect("all_types.blob_value field not found")
                .ty,
            DbType::Binary
        );

        assert!(matches!(
            fields
                .iter()
                .find(|f| f.name == "json_value")
                .expect("all_types.json_value field not found")
                .ty,
            DbType::Unknown(_)
        ));
        assert!(matches!(
            fields
                .iter()
                .find(|f| f.name == "date_value")
                .expect("all_types.date_value field not found")
                .ty,
            DbType::Unknown(_)
        ));
        assert!(matches!(
            fields
                .iter()
                .find(|f| f.name == "datetime_value")
                .expect("all_types.datetime_value field not found")
                .ty,
            DbType::Unknown(_)
        ));

        // Nullable columns
        let nullable_bool = fields
            .iter()
            .find(|f| f.name == "nullable_bool")
            .expect("all_types.nullable_bool field not found");
        assert_eq!(nullable_bool.ty, DbType::Boolean);
        assert!(nullable_bool.nullable);

        let nullable_text = fields
            .iter()
            .find(|f| f.name == "nullable_text")
            .expect("all_types.nullable_text field not found");
        assert_eq!(nullable_text.ty, DbType::String);
        assert!(nullable_text.nullable);

        let nullable_blob = fields
            .iter()
            .find(|f| f.name == "nullable_blob")
            .expect("all_types.nullable_blob field not found");
        assert_eq!(nullable_blob.ty, DbType::Binary);
        assert!(nullable_blob.nullable);
    }

    #[tokio::test]
    async fn constraints() {
        let introspector = setup().await;

        let user_constraints = introspector
            .list_constraints("users")
            .await
            .expect("list_constraints for users failed");

        // Primary key (generated name: users_pk)
        assert!(
            user_constraints
                .iter()
                .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { .. }))
        );
        assert!(user_constraints.iter().any(
            |c| matches!(&c.kind, ConstraintKind::PrimaryKey { columns } if columns == &vec!["id"])
        ));

        // UNIQUE on email — SQLite PRAGMA returns auto-generated index name
        assert!(user_constraints.iter().any(
            |c| matches!(&c.kind, ConstraintKind::Unique { columns } if columns == &vec!["email"])
        ));

        // CHECK constraint parsed from CREATE TABLE SQL preserves the name
        assert!(user_constraints.iter().any(|c| c.name == "users_age_check"));

        // mood CHECK (inline, no CONSTRAINT name)
        assert!(
            user_constraints
                .iter()
                .any(|c| matches!(&c.kind, ConstraintKind::Check { .. }))
        );

        let post_constraints = introspector
            .list_constraints("posts")
            .await
            .expect("list_constraints for posts failed");
        assert!(
            post_constraints
                .iter()
                .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { .. }))
        );

        // FK name is auto-generated as {table}_{column}_fk
        assert!(
            post_constraints
                .iter()
                .any(|c| c.name == "posts_user_id_fk")
        );

        // CHECK constraint parsed from SQL preserves the name
        assert!(
            post_constraints
                .iter()
                .any(|c| c.name == "posts_title_check")
        );

        let tag_constraints = introspector
            .list_constraints("tags")
            .await
            .expect("list_constraints for tags failed");
        // UNIQUE on name — auto-generated name
        assert!(tag_constraints.iter().any(
            |c| matches!(&c.kind, ConstraintKind::Unique { columns } if columns == &vec!["name"])
        ));

        let pt_constraints = introspector
            .list_constraints("post_tags")
            .await
            .expect("list_constraints for post_tags failed");
        assert!(pt_constraints.iter().any(
            |c| matches!(&c.kind, ConstraintKind::PrimaryKey { columns } if columns.len() == 2)
        ));

        let profile_constraints = introspector
            .list_constraints("profiles")
            .await
            .expect("list_constraints for profiles failed");
        // UNIQUE on email — auto-generated
        assert!(profile_constraints.iter().any(
            |c| matches!(&c.kind, ConstraintKind::Unique { columns } if columns == &vec!["email"])
        ));
    }

    #[tokio::test]
    async fn fk_relations_and_table_accessor() {
        let introspector = setup().await;
        let table_infos = introspector
            .list_tables_with_info()
            .await
            .expect("list_tables failed");
        let mut tables = Vec::new();
        for table_info in table_infos {
            let columns = introspector
                .list_columns(&table_info.name)
                .await
                .expect("list_columns failed");
            let fields: Vec<_> = columns
                .iter()
                .map(|c| introspector.column_to_field(c))
                .collect();
            let constraints = introspector
                .list_constraints(&table_info.name)
                .await
                .expect("list_constraints failed");
            tables.push(neutrino_schema::ir::TableIR {
                name: table_info.name.clone(),
                fields,
                constraints,
                comment: table_info.comment.clone(),
                indexes: vec![],
            });
        }
        let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

        let fk = schema.relations.iter().find(|r| r.from_table == "posts");
        assert!(fk.is_some());
        assert_eq!(
            fk.expect("FK relation from posts should exist")
                .from_columns[0],
            "user_id"
        );

        let users = schema.table("users").expect("users table");
        let id = users
            .fields
            .iter()
            .find(|f| f.name == "id")
            .expect("users.id");
        assert!(
            id.generated,
            "users.id should be generated (INTEGER PRIMARY KEY)"
        );
        let age = users
            .fields
            .iter()
            .find(|f| f.name == "age")
            .expect("users.age");
        assert_eq!(
            age.default_value.as_deref(),
            Some("0"),
            "users.age default should be 0"
        );

        assert!(schema.table("nonexistent").is_none());
    }

    #[tokio::test]
    async fn indexes() {
        use neutrino_schema::{IndexEntryIR, IndexKind};
        let introspector = setup().await;

        // --- users ---
        let indexes = introspector
            .list_indexes("users")
            .await
            .expect("list_indexes users");
        let created_at = indexes
            .iter()
            .find(|i| i.name == "idx_users_created_at")
            .expect("idx_users_created_at");
        assert!(!created_at.unique);
        assert_eq!(created_at.kind, IndexKind::BTree);
        assert_eq!(created_at.entries.len(), 1);
        assert_eq!(
            created_at.entries[0],
            IndexEntryIR::Column {
                name: "created_at".into(),
                descending: false
            }
        );

        // Expression index idx_users_email_lower on LOWER(email) — entries intentionally omitted
        let email_lower = indexes
            .iter()
            .find(|i| i.name == "idx_users_email_lower")
            .expect("idx_users_email_lower");
        assert!(email_lower.unique);
        assert_eq!(email_lower.kind, IndexKind::BTree);
        assert!(
            email_lower.entries.is_empty(),
            "expression entries omitted pending sqlite_master parsing"
        );

        // --- posts ---
        let indexes = introspector
            .list_indexes("posts")
            .await
            .expect("list_indexes posts");
        let user_created = indexes
            .iter()
            .find(|i| i.name == "idx_posts_user_id_created")
            .expect("idx_posts_user_id_created");
        assert!(!user_created.unique);
        assert_eq!(user_created.entries.len(), 2);
        assert_eq!(
            user_created.entries[0],
            IndexEntryIR::Column {
                name: "user_id".into(),
                descending: false
            }
        );
        assert_eq!(
            user_created.entries[1],
            IndexEntryIR::Column {
                name: "created_at".into(),
                descending: false
            }
        );

        // --- profiles ---
        let indexes = introspector
            .list_indexes("profiles")
            .await
            .expect("list_indexes profiles");
        let score = indexes
            .iter()
            .find(|i| i.name == "idx_profiles_score")
            .expect("idx_profiles_score");
        assert!(!score.unique);
        assert_eq!(score.entries.len(), 1);
        assert_eq!(
            score.entries[0],
            IndexEntryIR::Column {
                name: "score".into(),
                descending: false
            }
        );
    }
}

// ---------------------------------------------------------------------------
// PostgreSQL — runs if DATABASE_URL is set
// ---------------------------------------------------------------------------

#[cfg(feature = "postgres")]
mod postgres {
    use neutrino_schema::{
        RelationStrategy, SchemaIR,
        introspect::{DatabaseIntrospector, PostgresIntrospector},
        ir::ConstraintKind,
        types::DbType,
    };
    use sqlx::PgPool;

    fn database_url() -> Option<String> {
        std::env::var("DATABASE_URL").ok()
    }

    async fn setup(db_name: &str) -> Option<PostgresIntrospector> {
        let url = database_url()?;
        let admin = PgPool::connect(&url).await.ok()?;

        let sql: &'static str =
            &*Box::leak(format!("DROP DATABASE IF EXISTS \"{db_name}\"").into_boxed_str());
        sqlx::query(sql).execute(&admin).await.ok();
        let sql: &'static str =
            &*Box::leak(format!("CREATE DATABASE \"{db_name}\"").into_boxed_str());
        sqlx::query(sql).execute(&admin).await.ok()?;
        admin.close().await;

        let fixture_url = format!("{}/{}", url.trim_end_matches('/'), db_name);
        let pool = PgPool::connect(&fixture_url).await.ok()?;

        let sql = crate::common::fixture::load_fixture("postgres");
        let sql: &'static str = Box::leak(sql.into_boxed_str());
        for stmt in sql.split(';') {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed)
                    .execute(&pool)
                    .await
                    .expect("fixture SQL should execute");
            }
        }

        Some(PostgresIntrospector::new(pool))
    }

    async fn teardown(db_name: &str) {
        if let Some(url) = database_url() {
            if let Ok(admin) = PgPool::connect(&url).await {
                let s: &'static str =
                    &*Box::leak(format!("DROP DATABASE IF EXISTS \"{db_name}\"").into_boxed_str());
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
        let tables = introspector
            .list_tables_with_info()
            .await
            .expect("list_tables failed");
        for table_info in tables {
            assert!(
                [
                    "users",
                    "posts",
                    "tags",
                    "post_tags",
                    "profiles",
                    "all_types"
                ]
                .contains(&table_info.name.as_str()),
                "missing table: {}",
                table_info.name
            );
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
        let columns = introspector
            .list_columns("users")
            .await
            .expect("list_columns for users failed");
        let fields: Vec<_> = columns
            .iter()
            .map(|c| introspector.column_to_field(c))
            .collect();

        let email = fields
            .iter()
            .find(|f| f.name == "email")
            .expect("users.email field not found");
        assert_eq!(email.ty, DbType::String);
        assert!(!email.nullable);

        let full_name = fields
            .iter()
            .find(|f| f.name == "full_name")
            .expect("users.full_name field not found");
        assert_eq!(full_name.ty, DbType::Text);
        assert!(!full_name.nullable);

        let bio = fields
            .iter()
            .find(|f| f.name == "bio")
            .expect("users.bio field not found");
        assert_eq!(bio.ty, DbType::Text);
        assert!(bio.nullable);

        let age = fields
            .iter()
            .find(|f| f.name == "age")
            .expect("users.age field not found");
        assert_eq!(age.ty, DbType::Integer);
        assert!(!age.nullable);

        let salary = fields
            .iter()
            .find(|f| f.name == "salary")
            .expect("users.salary field not found");
        assert_eq!(salary.ty, DbType::Decimal);
        assert!(salary.nullable);

        let is_active = fields
            .iter()
            .find(|f| f.name == "is_active")
            .expect("users.is_active field not found");
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
        let columns = introspector
            .list_columns("all_types")
            .await
            .expect("list_columns for all_types failed");
        let fields: Vec<_> = columns
            .iter()
            .map(|c| introspector.column_to_field(c))
            .collect();

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "small_int_value")
                .expect("all_types.small_int_value field not found")
                .ty,
            DbType::SmallInt
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "integer_value")
                .expect("all_types.integer_value field not found")
                .ty,
            DbType::Integer
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "bigint_value")
                .expect("all_types.bigint_value field not found")
                .ty,
            DbType::BigInt
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "serial_value")
                .expect("all_types.serial_value field not found")
                .ty,
            DbType::Integer
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "bigserial_value")
                .expect("all_types.bigserial_value field not found")
                .ty,
            DbType::BigInt
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "numeric_value")
                .expect("all_types.numeric_value field not found")
                .ty,
            DbType::Decimal
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "real_value")
                .expect("all_types.real_value field not found")
                .ty,
            DbType::Float32
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "double_value")
                .expect("all_types.double_value field not found")
                .ty,
            DbType::Float64
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "varchar_value")
                .expect("all_types.varchar_value field not found")
                .ty,
            DbType::String
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "text_value")
                .expect("all_types.text_value field not found")
                .ty,
            DbType::Text
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "boolean_value")
                .expect("all_types.boolean_value field not found")
                .ty,
            DbType::Boolean
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "bytea_value")
                .expect("all_types.bytea_value field not found")
                .ty,
            DbType::Binary
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "date_value")
                .expect("all_types.date_value field not found")
                .ty,
            DbType::Date
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "time_value")
                .expect("all_types.time_value field not found")
                .ty,
            DbType::Time
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "timestamp_value")
                .expect("all_types.timestamp_value field not found")
                .ty,
            DbType::Timestamp
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "timestamptz_value")
                .expect("all_types.timestamptz_value field not found")
                .ty,
            DbType::TimestampTz
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "json_value")
                .expect("all_types.json_value field not found")
                .ty,
            DbType::Json
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "jsonb_value")
                .expect("all_types.jsonb_value field not found")
                .ty,
            DbType::Jsonb
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "uuid_value")
                .expect("all_types.uuid_value field not found")
                .ty,
            DbType::Uuid
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "inet_value")
                .expect("all_types.inet_value field not found")
                .ty,
            DbType::Inet
        );

        let mood = fields
            .iter()
            .find(|f| f.name == "mood_value")
            .expect("all_types.mood_value field not found");
        assert!(matches!(mood.ty, DbType::Unknown(_)));
        assert_eq!(mood.raw_type, "USER-DEFINED");

        let arr = fields
            .iter()
            .find(|f| f.name == "text_array_value")
            .expect("all_types.text_array_value field not found");
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

        let user_constraints = introspector
            .list_constraints("users")
            .await
            .expect("list_constraints for users failed");
        assert!(
            user_constraints
                .iter()
                .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { .. }))
        );
        assert!(
            user_constraints
                .iter()
                .any(|c| c.name == "users_email_unique")
        );
        assert!(user_constraints.iter().any(|c| c.name == "users_age_check"));

        let post_constraints = introspector
            .list_constraints("posts")
            .await
            .expect("list_constraints for posts failed");
        assert!(
            post_constraints
                .iter()
                .any(|c| c.name == "posts_user_id_fkey")
        );
        assert!(
            post_constraints
                .iter()
                .any(|c| c.name == "posts_title_check")
        );

        let pt_constraints = introspector
            .list_constraints("post_tags")
            .await
            .expect("list_constraints for post_tags failed");
        assert!(pt_constraints.iter().any(
            |c| matches!(&c.kind, ConstraintKind::PrimaryKey { columns } if columns.len() == 2)
        ));

        let profile_constraints = introspector
            .list_constraints("profiles")
            .await
            .expect("list_constraints for profiles failed");
        assert!(
            profile_constraints
                .iter()
                .any(|c| c.name == "profiles_email_unique")
        );

        drop(introspector);
        teardown("ns_fixture_pg_4").await;
    }

    #[tokio::test]
    async fn enums_discovered() {
        let Some(introspector) = setup("ns_fixture_pg_5").await else {
            eprintln!("Skipping postgres::enums_discovered (DATABASE_URL not set)");
            return;
        };
        let enums = introspector
            .introspect_enums()
            .await
            .expect("introspect_enums failed");
        let mood = enums.iter().find(|e| e.database_name == "mood");
        assert!(mood.is_some(), "mood enum should be discovered");
        let mood = mood.expect("mood enum should be discovered");
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
        let table_infos = introspector
            .list_tables_with_info()
            .await
            .expect("list_tables failed");
        let mut tables = Vec::new();
        for table_info in table_infos {
            let columns = introspector
                .list_columns(&table_info.name)
                .await
                .expect("list_columns failed");
            let fields: Vec<_> = columns
                .iter()
                .map(|c| introspector.column_to_field(c))
                .collect();
            let constraints = introspector
                .list_constraints(&table_info.name)
                .await
                .expect("list_constraints failed");
            tables.push(neutrino_schema::ir::TableIR {
                name: table_info.name.clone(),
                fields,
                constraints,
                comment: table_info.comment.clone(),
                indexes: vec![],
            });
        }
        let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

        assert!(schema.table("users").is_some());
        assert!(schema.table("posts").is_some());
        assert!(schema.table("nonexistent").is_none());

        let fk = schema.relations.iter().find(|r| r.from_table == "posts");
        assert!(fk.is_some(), "FK relation from posts should exist");
        assert_eq!(
            fk.expect("FK relation from posts should exist")
                .from_columns[0],
            "user_id"
        );

        drop(introspector);
        teardown("ns_fixture_pg_6").await;
    }

    #[tokio::test]
    async fn indexes() {
        use neutrino_schema::{IndexEntryIR, IndexKind};
        let Some(introspector) = setup("ns_fixture_pg_7").await else {
            eprintln!("Skipping postgres::indexes (DATABASE_URL not set)");
            return;
        };

        // --- users ---
        let indexes = introspector
            .list_indexes("users")
            .await
            .expect("list_indexes users");
        let created_at = indexes
            .iter()
            .find(|i| i.name == "idx_users_created_at")
            .expect("idx_users_created_at");
        assert!(!created_at.unique);
        assert_eq!(created_at.kind, IndexKind::BTree);
        assert_eq!(created_at.entries.len(), 1);
        assert_eq!(
            created_at.entries[0],
            IndexEntryIR::Column {
                name: "created_at".into(),
                descending: false
            }
        );

        let is_active = indexes
            .iter()
            .find(|i| i.name == "idx_users_is_active")
            .expect("idx_users_is_active");
        assert!(!is_active.unique);
        assert_eq!(is_active.kind, IndexKind::BTree);
        assert_eq!(
            is_active.predicate.as_deref(),
            Some("is_active = TRUE"),
            "partial index with predicate"
        );

        let lower_email = indexes
            .iter()
            .find(|i| i.name == "idx_users_lower_email")
            .expect("idx_users_lower_email");
        assert!(lower_email.unique);
        assert_eq!(lower_email.entries.len(), 1);
        assert_eq!(
            lower_email.entries[0],
            IndexEntryIR::Expression {
                expression: "lower((email)::text)".into()
            }
        );

        // --- posts ---
        let indexes = introspector
            .list_indexes("posts")
            .await
            .expect("list_indexes posts");
        let user_created = indexes
            .iter()
            .find(|i| i.name == "idx_posts_user_id_created")
            .expect("idx_posts_user_id_created");
        assert!(!user_created.unique);
        assert_eq!(user_created.entries.len(), 2);
        assert_eq!(
            user_created.entries[0],
            IndexEntryIR::Column {
                name: "user_id".into(),
                descending: false
            }
        );
        assert_eq!(
            user_created.entries[1],
            IndexEntryIR::Column {
                name: "created_at".into(),
                descending: true
            }
        );

        let metadata = indexes
            .iter()
            .find(|i| i.name == "idx_posts_metadata")
            .expect("idx_posts_metadata");
        assert!(!metadata.unique);
        assert_eq!(metadata.kind, IndexKind::Gin);

        // --- profiles ---
        let indexes = introspector
            .list_indexes("profiles")
            .await
            .expect("list_indexes profiles");
        let score = indexes
            .iter()
            .find(|i| i.name == "idx_profiles_score")
            .expect("idx_profiles_score");
        assert!(!score.unique);
        assert_eq!(score.entries.len(), 1);
        assert_eq!(
            score.entries[0],
            IndexEntryIR::Column {
                name: "score".into(),
                descending: true
            }
        );

        drop(introspector);
        teardown("ns_fixture_pg_7").await;
    }
}

// ---------------------------------------------------------------------------
// MySQL — runs if DATABASE_URL is set
// ---------------------------------------------------------------------------

#[cfg(feature = "mysql")]
mod mysql {
    use neutrino_schema::{
        RelationStrategy, SchemaIR,
        introspect::{DatabaseIntrospector, MysqlIntrospector},
        ir::ConstraintKind,
        types::DbType,
    };
    use sqlx::MySqlPool;

    fn admin_url() -> Option<String> {
        std::env::var("DATABASE_URL")
            .ok()
            .or_else(|| Some("mysql://root@localhost:3306".to_string()))
    }

    async fn setup(db_name: &str) -> Option<MysqlIntrospector> {
        let admin_url = admin_url()?;
        let admin = MySqlPool::connect(&format!("{admin_url}/mysql"))
            .await
            .ok()?;

        let sql: &'static str =
            &*Box::leak(format!("DROP DATABASE IF EXISTS `{db_name}`").into_boxed_str());
        sqlx::query(sql).execute(&admin).await.ok();
        let sql: &'static str =
            &*Box::leak(format!("CREATE DATABASE `{db_name}`").into_boxed_str());
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
                sqlx::query(trimmed)
                    .execute(&pool)
                    .await
                    .expect("fixture SQL should execute");
            }
        }

        Some(MysqlIntrospector::new(pool))
    }

    async fn teardown(db_name: &str) {
        if let Some(admin_url) = admin_url() {
            if let Ok(admin) = MySqlPool::connect(&format!("{admin_url}/mysql")).await {
                let s: &'static str =
                    &*Box::leak(format!("DROP DATABASE IF EXISTS `{db_name}`").into_boxed_str());
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
        let tables = introspector
            .list_tables_with_info()
            .await
            .expect("list_tables failed");
        for table_info in tables {
            assert!(
                [
                    "users",
                    "posts",
                    "tags",
                    "post_tags",
                    "profiles",
                    "all_types"
                ]
                .contains(&table_info.name.as_str()),
                "missing table: {}",
                table_info.name
            );
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
        let columns = introspector
            .list_columns("users")
            .await
            .expect("list_columns for users failed");
        let fields: Vec<_> = columns
            .iter()
            .map(|c| introspector.column_to_field(c))
            .collect();

        let email = fields
            .iter()
            .find(|f| f.name == "email")
            .expect("users.email field not found");
        assert_eq!(email.ty, DbType::String);
        assert!(!email.nullable);

        let bio = fields
            .iter()
            .find(|f| f.name == "bio")
            .expect("users.bio field not found");
        assert_eq!(bio.ty, DbType::Text);
        assert!(bio.nullable);

        let age = fields
            .iter()
            .find(|f| f.name == "age")
            .expect("users.age field not found");
        assert_eq!(age.ty, DbType::Integer);
        assert!(!age.nullable);

        let salary = fields
            .iter()
            .find(|f| f.name == "salary")
            .expect("users.salary field not found");
        assert_eq!(salary.ty, DbType::Decimal);
        assert!(salary.nullable);

        let is_active = fields
            .iter()
            .find(|f| f.name == "is_active")
            .expect("users.is_active field not found");
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
        let columns = introspector
            .list_columns("all_types")
            .await
            .expect("list_columns for all_types failed");
        let fields: Vec<_> = columns
            .iter()
            .map(|c| introspector.column_to_field(c))
            .collect();

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "tiny_int_value")
                .expect("all_types.tiny_int_value field not found")
                .ty,
            DbType::SmallInt
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "small_int_value")
                .expect("all_types.small_int_value field not found")
                .ty,
            DbType::SmallInt
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "medium_int_value")
                .expect("all_types.medium_int_value field not found")
                .ty,
            DbType::Integer
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "integer_value")
                .expect("all_types.integer_value field not found")
                .ty,
            DbType::Integer
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "bigint_value")
                .expect("all_types.bigint_value field not found")
                .ty,
            DbType::BigInt
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "decimal_value")
                .expect("all_types.decimal_value field not found")
                .ty,
            DbType::Decimal
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "float_value")
                .expect("all_types.float_value field not found")
                .ty,
            DbType::Float32
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "double_value")
                .expect("all_types.double_value field not found")
                .ty,
            DbType::Float64
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "varchar_value")
                .expect("all_types.varchar_value field not found")
                .ty,
            DbType::String
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "text_value")
                .expect("all_types.text_value field not found")
                .ty,
            DbType::Text
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "enum_value")
                .expect("all_types.enum_value field not found")
                .ty,
            DbType::String
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "enum_value")
                .expect("all_types.enum_value field not found")
                .raw_type,
            "enum"
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "json_value")
                .expect("all_types.json_value field not found")
                .ty,
            DbType::Json
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "blob_value")
                .expect("all_types.blob_value field not found")
                .ty,
            DbType::Binary
        );

        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "datetime_value")
                .expect("all_types.datetime_value field not found")
                .ty,
            DbType::Timestamp
        );
        assert_eq!(
            fields
                .iter()
                .find(|f| f.name == "timestamp_value")
                .expect("all_types.timestamp_value field not found")
                .ty,
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

        let user_constraints = introspector
            .list_constraints("users")
            .await
            .expect("list_constraints for users failed");
        assert!(
            user_constraints
                .iter()
                .any(|c| matches!(&c.kind, ConstraintKind::PrimaryKey { .. }))
        );
        assert!(
            user_constraints
                .iter()
                .any(|c| c.name == "users_email_unique")
        );
        assert!(user_constraints.iter().any(|c| c.name == "users_age_check"));

        let post_constraints = introspector
            .list_constraints("posts")
            .await
            .expect("list_constraints for posts failed");
        assert!(
            post_constraints
                .iter()
                .any(|c| c.name == "posts_user_id_fkey")
        );
        assert!(
            post_constraints
                .iter()
                .any(|c| c.name == "posts_title_check")
        );

        let pt_constraints = introspector
            .list_constraints("post_tags")
            .await
            .expect("list_constraints for post_tags failed");
        assert!(pt_constraints.iter().any(
            |c| matches!(&c.kind, ConstraintKind::PrimaryKey { columns } if columns.len() == 2)
        ));

        let profile_constraints = introspector
            .list_constraints("profiles")
            .await
            .expect("list_constraints for profiles failed");
        assert!(
            profile_constraints
                .iter()
                .any(|c| c.name == "profiles_email_unique")
        );

        drop(introspector);
        teardown("ns_fixture_my_4").await;
    }

    #[tokio::test]
    async fn fk_relations_and_table_accessor() {
        let Some(introspector) = setup("ns_fixture_my_5").await else {
            eprintln!("Skipping mysql::fk_relations (MySQL unreachable)");
            return;
        };
        let table_infos = introspector
            .list_tables_with_info()
            .await
            .expect("list_tables failed");
        let mut tables = Vec::new();
        for table_info in table_infos {
            let columns = introspector
                .list_columns(&table_info.name)
                .await
                .expect("list_columns failed");
            let fields: Vec<_> = columns
                .iter()
                .map(|c| introspector.column_to_field(c))
                .collect();
            let constraints = introspector
                .list_constraints(&table_info.name)
                .await
                .expect("list_constraints failed");
            tables.push(neutrino_schema::ir::TableIR {
                name: table_info.name.clone(),
                fields,
                constraints,
                indexes: vec![],
                comment: table_info.comment.clone(),
            });
        }
        let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

        assert!(schema.table("users").is_some());
        assert!(schema.table("posts").is_some());

        let fk = schema.relations.iter().find(|r| r.from_table == "posts");
        assert!(fk.is_some(), "FK relation from posts should exist");
        assert_eq!(
            fk.expect("FK relation from posts should exist")
                .from_columns[0],
            "user_id"
        );

        drop(introspector);
        teardown("ns_fixture_my_5").await;
    }

    #[tokio::test]
    async fn indexes() {
        use neutrino_schema::{IndexEntryIR, IndexKind};
        let Some(introspector) = setup("ns_fixture_my_6").await else {
            eprintln!("Skipping mysql::indexes (MySQL unreachable)");
            return;
        };

        // --- users ---
        let indexes = introspector
            .list_indexes("users")
            .await
            .expect("list_indexes users");
        let created_at = indexes
            .iter()
            .find(|i| i.name == "idx_users_created_at")
            .expect("idx_users_created_at");
        assert!(!created_at.unique);
        assert_eq!(created_at.kind, IndexKind::BTree);
        assert_eq!(created_at.entries.len(), 1);
        assert_eq!(
            created_at.entries[0],
            IndexEntryIR::Column {
                name: "created_at".into(),
                descending: false
            }
        );

        let is_active = indexes
            .iter()
            .find(|i| i.name == "idx_users_is_active")
            .expect("idx_users_is_active");
        assert!(!is_active.unique);
        assert_eq!(is_active.kind, IndexKind::BTree);

        // MySQL functional index on LOWER(email) — INDEX_TYPE is BTREE, unique
        let email_lower = indexes
            .iter()
            .find(|i| i.name == "idx_users_email_lower")
            .expect("idx_users_email_lower");
        assert!(email_lower.unique);
        assert_eq!(email_lower.kind, IndexKind::BTree);
        assert_eq!(email_lower.entries.len(), 1);
        assert!(matches!(
            &email_lower.entries[0],
            IndexEntryIR::Expression { .. }
        ));

        // --- posts ---
        let indexes = introspector
            .list_indexes("posts")
            .await
            .expect("list_indexes posts");
        let user_created = indexes
            .iter()
            .find(|i| i.name == "idx_posts_user_id_created")
            .expect("idx_posts_user_id_created");
        assert!(!user_created.unique);
        assert_eq!(user_created.entries.len(), 2);
        assert_eq!(
            user_created.entries[0],
            IndexEntryIR::Column {
                name: "user_id".into(),
                descending: false
            }
        );
        assert_eq!(
            user_created.entries[1],
            IndexEntryIR::Column {
                name: "created_at".into(),
                descending: true
            }
        );

        let body_ft = indexes
            .iter()
            .find(|i| i.name == "idx_posts_body_fulltext")
            .expect("idx_posts_body_fulltext");
        assert!(!body_ft.unique);
        assert_eq!(body_ft.kind, IndexKind::FullText);

        // --- profiles ---
        let indexes = introspector
            .list_indexes("profiles")
            .await
            .expect("list_indexes profiles");
        let score = indexes
            .iter()
            .find(|i| i.name == "idx_profiles_score")
            .expect("idx_profiles_score");
        assert!(!score.unique);
        assert_eq!(score.entries.len(), 1);
        assert_eq!(
            score.entries[0],
            IndexEntryIR::Column {
                name: "score".into(),
                descending: true
            }
        );

        drop(introspector);
        teardown("ns_fixture_my_6").await;
    }
}
