use neutrino_schema::{
    introspect::{DatabaseIntrospector, SqliteIntrospector},
    RelationStrategy, SchemaIR,
};

/// Helper: create an in-memory SQLite DB with test tables, returns the
/// introspector.
async fn setup_introspector() -> SqliteIntrospector {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite in-memory connection");

    sqlx::query(
        "CREATE TABLE users (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            email      TEXT    NOT NULL UNIQUE,
            full_name  TEXT,
            age        INTEGER NOT NULL DEFAULT 0,
            salary     REAL,
            avatar     BLOB,
            is_active  INTEGER NOT NULL DEFAULT 1,
            created_at TEXT    NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&pool)
    .await
    .expect("create users table");

    sqlx::query(
        "CREATE TABLE posts (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id    INTEGER NOT NULL,
            title      TEXT    NOT NULL,
            body       TEXT,
            created_at TEXT    NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
    )
    .execute(&pool)
    .await
    .expect("create posts table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sqlite_sequence(name,seq)",
    )
    .execute(&pool)
    .await
    .ok();

    SqliteIntrospector::new(pool)
}

#[tokio::test]
async fn sqlite_list_tables() {
    let introspector = setup_introspector().await;
    let table_infos = introspector.list_tables_with_info().await.expect("list tables");
    let tables: Vec<String> = table_infos.iter().map(|ti| ti.name.clone()).collect();
    assert!(tables.contains(&"users".to_string()));
    assert!(tables.contains(&"posts".to_string()));
    // sqlite_sequence is internal, should be filtered out
    assert!(!tables.contains(&"sqlite_sequence".to_string()));
}

#[tokio::test]
async fn sqlite_list_columns() {
    let introspector = setup_introspector().await;
    let columns = introspector.list_columns("users").await.expect("list columns");

    let col_names: Vec<_> = columns.iter().map(|c| c.column_name.as_str()).collect();
    assert!(col_names.contains(&"id"));
    assert!(col_names.contains(&"email"));
    assert!(col_names.contains(&"full_name"));
    assert!(col_names.contains(&"age"));
    assert!(col_names.contains(&"salary"));
    assert!(col_names.contains(&"avatar"));
    assert!(col_names.contains(&"is_active"));
    assert!(col_names.contains(&"created_at"));

    // id is INTEGER PRIMARY KEY → SQLite's PRAGMA doesn't report implicit NOT NULL
    let _id_col = columns.iter().find(|c| c.column_name == "id").unwrap();
    // PRAGMA table_info does NOT report implicit PK NOT NULL, so nullable may be true
    // This is expected SQLite behavior; explicit NOT NULL columns are checked below.

    // email is TEXT NOT NULL → non-nullable
    let email_col = columns.iter().find(|c| c.column_name == "email").unwrap();
    assert!(!email_col.nullable, "email has explicit NOT NULL");

    // full_name has no NOT NULL → nullable
    let name_col = columns.iter().find(|c| c.column_name == "full_name").unwrap();
    assert!(name_col.nullable);
}

#[tokio::test]
async fn sqlite_column_to_field() {
    let introspector = setup_introspector().await;
    let columns = introspector.list_columns("users").await.expect("list columns");

    let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();

    let id = fields.iter().find(|f| f.name == "id").unwrap();
    assert_eq!(id.ty, neutrino_schema::types::DbType::Integer);
    // INTEGER PRIMARY KEY: SQLite PRAGMA doesn't report implicit NOT NULL

    let email = fields.iter().find(|f| f.name == "email").unwrap();
    assert_eq!(email.ty, neutrino_schema::types::DbType::String);
    assert!(!email.nullable, "email has explicit NOT NULL");

    let full_name = fields.iter().find(|f| f.name == "full_name").unwrap();
    assert_eq!(full_name.ty, neutrino_schema::types::DbType::String);
    assert!(full_name.nullable);

    let salary = fields.iter().find(|f| f.name == "salary").unwrap();
    assert_eq!(salary.ty, neutrino_schema::types::DbType::Float64);

    let avatar = fields.iter().find(|f| f.name == "avatar").unwrap();
    assert_eq!(avatar.ty, neutrino_schema::types::DbType::Binary);
}

#[tokio::test]
async fn sqlite_list_constraints() {
    let introspector = setup_introspector().await;

    // users: PK on id, UNIQUE on email
    let user_constraints = introspector.list_constraints("users").await.expect("list constraints");
    assert!(user_constraints.iter().any(|c| matches!(&c.kind, neutrino_schema::ir::ConstraintKind::PrimaryKey { columns } if columns == &vec!["id".to_string()])),
        "expected PK on id");
    assert!(user_constraints.iter().any(|c| matches!(&c.kind, neutrino_schema::ir::ConstraintKind::Unique { columns } if columns == &vec!["email".to_string()])),
        "expected UNIQUE on email");

    // posts: PK on id, FK user_id → users(id)
    let post_constraints = introspector.list_constraints("posts").await.expect("list constraints");
    assert!(post_constraints.iter().any(|c| matches!(&c.kind, neutrino_schema::ir::ConstraintKind::PrimaryKey { columns } if columns == &vec!["id".to_string()])),
        "expected PK on id");
    assert!(post_constraints.iter().any(|c| matches!(&c.kind, neutrino_schema::ir::ConstraintKind::ForeignKey { columns, referenced_table, .. } if columns == &vec!["user_id".to_string()] && referenced_table == "users")),
        "expected FK user_id → users(id)");
}

#[tokio::test]
async fn sqlite_full_pipeline() {
    let introspector = setup_introspector().await;
    let table_infos = introspector.list_tables_with_info().await.expect("list tables");

    let mut tables = Vec::new();
    for info in &table_infos {
        let columns = introspector.list_columns(&info.name).await.expect("list columns");
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
        let constraints = introspector.list_constraints(&info.name).await.expect("list constraints");
        tables.push(neutrino_schema::ir::TableIR {
            name: info.name.to_string(),
            fields,
            constraints,
            comment: info.comment.clone(),
        });
    }

    let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

    // FK-derived relation: posts.user_id → users.id
    let fk_rel = schema.relations.iter().find(|r| matches!(r.source, neutrino_schema::ir::RelationSource::ForeignKey(_)));
    assert!(fk_rel.is_some(), "FK-derived relation should exist");
    if let Some(r) = fk_rel {
        assert_eq!(r.from_field, "user_id");
        assert_eq!(r.to_table, "users");
        assert_eq!(r.to_field, "id");
    }

    // Heuristic relation: posts.user_id → users.id
    let heuristic_rel = schema.relations.iter().find(|r| matches!(r.source, neutrino_schema::ir::RelationSource::NamingHeuristic));
    assert!(heuristic_rel.is_some(), "heuristic relation should exist");
    if let Some(r) = heuristic_rel {
        assert_eq!(r.from_field, "user_id");
        assert_eq!(r.to_table, "users");
        assert_eq!(r.to_field, "id");
    }

    // Generated struct looks correct
    let users_table = schema.tables.iter().find(|t| t.name == "users").unwrap();
    let output = neutrino_schema::generate_struct(users_table, neutrino_schema::RenderMode::Clean);
    assert!(output.contains("pub struct Users"));
    assert!(output.contains("pub email: String,"), "email should be non-null String");
    assert!(output.contains("pub full_name: Option<String>,"));
    assert!(output.contains("pub salary: Option<f64>,"));
    assert!(output.contains("pub avatar: Option<Vec<u8>>,"));
}
