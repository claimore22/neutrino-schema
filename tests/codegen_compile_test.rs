mod common;

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn codegen_compile_sqlite_fixtures() {
    use neutrino_schema::{
        codegen::{generate_files_with_registry, RenderMode},
        config::GeneratorConfig,
        introspect::DatabaseIntrospector,
        types::TypeRegistry,
        RelationStrategy, SchemaIR,
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
            sqlx::query(trimmed).execute(&pool).await.unwrap();
        }
    }

    let introspector =
        neutrino_schema::introspect::SqliteIntrospector::new(pool);

    // ── 2. Introspect ─────────────────────────────────────────────────
    let table_infos = introspector.list_tables_with_info().await.unwrap();
    let mut tables = Vec::new();
    for info in &table_infos {
        let columns = introspector.list_columns(&info.name).await.unwrap();
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
        let constraints = introspector.list_constraints(&info.name).await.unwrap();
        tables.push(neutrino_schema::ir::TableIR {
            name: info.name.to_string(),
            fields,
            constraints,
            comment: info.comment.clone(),
        });
    }

    let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);
    schema.validate().expect("schema validation should pass");

    // ── 3. Generate code ───────────────────────────────────────────────
    let tmp = std::env::temp_dir().join("ns_codegen_compile_test");
    let _ = std::fs::remove_dir_all(&tmp);
    let out_dir = tmp.join("src").join("entities");

    let config = GeneratorConfig {
        output_dir: out_dir.clone(),
        module_name: "entities".into(),
        render_mode: RenderMode::Debug,
    };
    let registry = TypeRegistry::default();

    generate_files_with_registry(&schema, &config, &registry).expect("generate files");

    // ── 4. Write Cargo.toml ────────────────────────────────────────────
    // SQLite fixture types all resolve to standard Rust types (i32, i64, f64, String, Vec<u8>),
    // so no external dependencies are needed.
    std::fs::write(
        tmp.join("Cargo.toml"),
        r#"[package]
name = "test-generated-code"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .unwrap();

    let src_dir = tmp.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "mod entities;\n").unwrap();

    // ── 5. cargo check ─────────────────────────────────────────────────
    let status = std::process::Command::new("cargo")
        .args(["check"])
        .current_dir(&tmp)
        .status()
        .expect("cargo check");

    let _ = std::fs::remove_dir_all(&tmp);

    assert!(status.success(), "generated code should compile");
}
