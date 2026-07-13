mod common;

/// Level 4A — Generated models actually query the database.
///
/// 1. Create SQLite :memory: database
/// 2. Apply fixture schema + seed data
/// 3. Introspect → SchemaIR
/// 4. Generate Rust structs via `generate_struct()`
/// 5. Post-process: add `sqlx::FromRow` derive
/// 6. Write standalone Cargo project with the structs + a roundtrip test
/// 7. `cargo test` on the generated project
/// 8. Assert decoded row counts, field values, and NULL handling
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn generated_models_roundtrip_sqlite() {
    use neutrino_schema::{
        codegen::{generate_struct, RenderMode},
        introspect::DatabaseIntrospector,
    };
    use sqlx::sqlite::SqlitePoolOptions;

    // ── 1. In-memory SQLite with fixture data ──────────────────────────
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite in-memory connection");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .ok();

    let sql = common::fixture::load_fixture("sqlite");
    let sql: &'static str = Box::leak(sql.into_boxed_str());
    for stmt in sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await.expect("failed to execute fixture SQL statement");
        }
    }

    let introspector =
        neutrino_schema::introspect::SqliteIntrospector::new(pool);

    // ── 2. Introspect ─────────────────────────────────────────────────
    let table_infos = introspector.list_tables_with_info().await.expect("list_tables failed");
    let mut struct_strs = Vec::new();
    for info in &table_infos {
        let columns = introspector.list_columns(&info.name).await.expect("list_columns failed");
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
        let table = neutrino_schema::ir::TableIR {
            name: info.name.to_string(),
            fields,
            constraints: vec![],
            comment: info.comment.clone(),
            indexes: vec![],
        };
        let struct_src = generate_struct(&table, RenderMode::Debug);
        // Post-process: add sqlx::FromRow to derives
        let struct_src = struct_src.replace(
            "#[derive(Debug, Clone)]",
            "#[derive(Debug, Clone, sqlx::FromRow)]",
        );
        struct_strs.push(struct_src);
    }

    // ── Level 4A: Validate generated Rust parses with syn ──────────
    let all_structs = struct_strs.join("\n");
    let parsed: syn::File = syn::parse_file(&all_structs)
        .unwrap_or_else(|e| panic!("Generated Rust failed to parse: {e}\n---\n{all_structs}\n---"));

    // Verify every item is a struct with at least one field
    let struct_count = parsed
        .items
        .iter()
        .filter(|item| matches!(item, syn::Item::Struct(_)))
        .count();
    assert!(
        struct_count >= table_infos.len(),
        "Expected at least {} structs, got {}",
        table_infos.len(),
        struct_count,
    );

    // ── 3. Write generated Cargo project ──────────────────────────────
    let tmp = std::env::temp_dir().join("ns_roundtrip_sqlite");
    let _ = std::fs::remove_dir_all(&tmp);

    std::fs::create_dir_all(tmp.join("src")).expect("failed to create temp dir");
    std::fs::create_dir_all(tmp.join("tests")).expect("failed to create temp dir");

    // Cargo.toml
    std::fs::write(
        tmp.join("Cargo.toml"),
        r#"[package]
name = "roundtrip-test"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { version = "0.9", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
"#,
    )
    .expect("failed to write temp file");

    // src/lib.rs
    std::fs::write(tmp.join("src").join("lib.rs"), "pub mod models;\n").expect("failed to write temp file");

    // src/models.rs
    let models_content = struct_strs.join("\n");
    std::fs::write(tmp.join("src").join("models.rs"), models_content).expect("failed to write temp file");

    // ── 4. Write roundtrip test ───────────────────────────────────────
    // We embed the fixture SQL (schema + seed data) directly in the test
    // so that the generated project is completely standalone.
    let fixture_sql = common::fixture::load_fixture("sqlite");

    let roundtrip_test = format!(
        r##"use roundtrip_test::models::*;

const SCHEMA_SQL: &str = r#"
{fixture_sql}
"#;

#[tokio::test]
async fn users_roundtrip() {{
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("pool");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .ok();

    for stmt in SCHEMA_SQL.split(';') {{
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {{
            sqlx::query(trimmed).execute(&pool).await.expect("failed to execute fixture SQL statement");
        }}
    }}

    let rows = sqlx::query_as::<_, Users>("SELECT * FROM users")
        .fetch_all(&pool)
        .await
        .expect("fetch users");

    assert_eq!(rows.len(), 2, "users row count");

    let alice = rows.iter().find(|u| u.email == "alice@example.com").expect("alice not found in users");
    assert_eq!(alice.id, 1);
    assert_eq!(alice.age, 30);
    assert_eq!(alice.is_active, 1);

    let bob = rows.iter().find(|u| u.email == "bob@example.com").expect("bob not found in users");
    assert_eq!(bob.id, 2);
    assert_eq!(bob.age, 25);
}}

#[tokio::test]
async fn posts_roundtrip() {{
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("pool");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .ok();

    for stmt in SCHEMA_SQL.split(';') {{
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {{
            sqlx::query(trimmed).execute(&pool).await.expect("failed to execute fixture SQL statement");
        }}
    }}

    let rows = sqlx::query_as::<_, Posts>("SELECT * FROM posts")
        .fetch_all(&pool)
        .await
        .expect("fetch posts");

    assert_eq!(rows.len(), 3, "posts row count");

    let first = rows.iter().find(|p| p.title == "First Post").expect("first post not found");
    assert_eq!(first.user_id, 1);
}}

#[tokio::test]
async fn tags_roundtrip() {{
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("pool");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .ok();

    for stmt in SCHEMA_SQL.split(';') {{
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {{
            sqlx::query(trimmed).execute(&pool).await.expect("failed to execute fixture SQL statement");
        }}
    }}

    let rows = sqlx::query_as::<_, Tags>("SELECT * FROM tags")
        .fetch_all(&pool)
        .await
        .expect("fetch tags");

    assert_eq!(rows.len(), 4, "tags row count");

    let tech = rows.iter().find(|t| t.name == "tech").expect("tech tag not found");
    assert_eq!(tech.description.as_deref(), Some("Technology-related posts"));
}}

#[tokio::test]
async fn post_tags_roundtrip() {{
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("pool");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .ok();

    for stmt in SCHEMA_SQL.split(';') {{
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {{
            sqlx::query(trimmed).execute(&pool).await.expect("failed to execute fixture SQL statement");
        }}
    }}

    let rows = sqlx::query_as::<_, PostTags>("SELECT * FROM post_tags")
        .fetch_all(&pool)
        .await
        .expect("fetch post_tags");

    assert_eq!(rows.len(), 4, "post_tags row count");
}}

#[tokio::test]
async fn profiles_roundtrip() {{
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("pool");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .ok();

    for stmt in SCHEMA_SQL.split(';') {{
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {{
            sqlx::query(trimmed).execute(&pool).await.expect("failed to execute fixture SQL statement");
        }}
    }}

    let rows = sqlx::query_as::<_, Profiles>("SELECT * FROM profiles")
        .fetch_all(&pool)
        .await
        .expect("fetch profiles");

    assert_eq!(rows.len(), 2, "profiles row count");

    let alice = rows.iter().find(|p| p.email == "alice@example.com").expect("alice profile not found");
    assert_eq!(alice.display_name, "Alice");
    assert!(alice.avatar_url.is_none());
}}

#[tokio::test]
async fn all_types_roundtrip() {{
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("pool");

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .ok();

    for stmt in SCHEMA_SQL.split(';') {{
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {{
            sqlx::query(trimmed).execute(&pool).await.expect("failed to execute fixture SQL statement");
        }}
    }}

    let rows = sqlx::query_as::<_, AllTypes>("SELECT * FROM all_types")
        .fetch_all(&pool)
        .await
        .expect("fetch all_types");

    assert_eq!(rows.len(), 1, "all_types row count");

    let row = &rows[0];
    assert_eq!(row.small_int_value, Some(100i16));
    assert_eq!(row.integer_value, Some(2000000i32));
    assert_eq!(row.bigint_value, Some(9000000000i64));
    assert_eq!(row.real_value, Some(3.14f64));
    assert_eq!(row.text_value.as_deref(), Some("This is a long text value."));
    assert_eq!(row.varchar_value.as_deref(), Some("hello"));

    // Nullable columns not in the seed INSERT — decode as None
    assert_eq!(row.nullable_bool, None);
    assert_eq!(row.nullable_text, None);
    assert_eq!(row.nullable_blob, None);
}}
"##,
    );

    std::fs::write(tmp.join("tests").join("roundtrip.rs"), roundtrip_test).expect("failed to write temp file");

    // ── 5. cargo test on the generated project ─────────────────────────
    let status = std::process::Command::new("cargo")
        .args(["test"])
        .current_dir(&tmp)
        .status()
        .expect("cargo test on generated project");

    let _ = std::fs::remove_dir_all(&tmp);

    assert!(status.success(), "generated roundtrip tests should pass");
}
