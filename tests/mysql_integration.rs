use neutrino_schema::{
    introspect::{DatabaseIntrospector, MysqlIntrospector},
    ir::SchemaIR,
    types::DbType,
    RelationStrategy,
};

const MYSQL_URL: &str = "mysql://root:1qaz2wsx@localhost:3306";

/// Helper: create a test database with tables, return the introspector.
async fn setup_introspector(db_name: &str) -> MysqlIntrospector {
    // Connect without a database first, create & seed the test DB
    let admin_pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(1)
        .connect(&format!("{MYSQL_URL}/mysql"))
        .await
        .expect("connect to MySQL (admin)");

    sqlx::query(
        sqlx::AssertSqlSafe(format!("DROP DATABASE IF EXISTS `{db_name}`")),
    )
    .execute(&admin_pool)
    .await
    .ok();
    sqlx::query(
        sqlx::AssertSqlSafe(format!("CREATE DATABASE `{db_name}`")),
    )
    .execute(&admin_pool)
    .await
    .expect("create test database");
    admin_pool.close().await;

    // Connect to the test database
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(1)
        .connect(&format!("{MYSQL_URL}/{db_name}"))
        .await
        .expect("connect to test database");

    sqlx::query(
        "CREATE TABLE users (
            id         INT             NOT NULL AUTO_INCREMENT PRIMARY KEY,
            email      VARCHAR(255)    NOT NULL UNIQUE,
            full_name  VARCHAR(255),
            age        INT             NOT NULL DEFAULT 0,
            salary     DECIMAL(10,2),
            avatar     MEDIUMBLOB,
            is_active  TINYINT(1)      NOT NULL DEFAULT 1,
            created_at DATETIME        NOT NULL DEFAULT CURRENT_TIMESTAMP,
            bio        TEXT
        ) ENGINE=InnoDB",
    )
    .execute(&pool)
    .await
    .expect("create users table");

    sqlx::query(
        "CREATE TABLE posts (
            id         INT          NOT NULL AUTO_INCREMENT PRIMARY KEY,
            user_id    INT          NOT NULL,
            title      VARCHAR(255) NOT NULL,
            body       TEXT,
            created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        ) ENGINE=InnoDB",
    )
    .execute(&pool)
    .await
    .expect("create posts table");

    MysqlIntrospector::new(pool)
}

async fn teardown(db_name: &str) {
    let admin_pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(1)
        .connect(&format!("{MYSQL_URL}/mysql"))
        .await
        .expect("connect to MySQL (admin)");
    sqlx::query(
        sqlx::AssertSqlSafe(format!("DROP DATABASE IF EXISTS `{db_name}`")),
    )
    .execute(&admin_pool)
    .await
    .ok();
    admin_pool.close().await;
}

#[tokio::test]
async fn mysql_list_tables() {
    let db_name = "ns_test_list_tables";
    let introspector = setup_introspector(db_name).await;
    let tables = introspector.list_tables().await.expect("list tables");
    assert!(tables.contains(&"users".to_string()));
    assert!(tables.contains(&"posts".to_string()));
    drop(introspector);
    teardown(db_name).await;
}

#[tokio::test]
async fn mysql_list_columns() {
    let db_name = "ns_test_list_columns";
    let introspector = setup_introspector(db_name).await;
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
    assert!(col_names.contains(&"bio"));

    let id_col = columns.iter().find(|c| c.column_name == "id").unwrap();
    assert!(!id_col.nullable, "id PRIMARY KEY is NOT NULL");

    let email_col = columns.iter().find(|c| c.column_name == "email").unwrap();
    assert!(!email_col.nullable, "email has explicit NOT NULL");

    let name_col = columns.iter().find(|c| c.column_name == "full_name").unwrap();
    assert!(name_col.nullable, "full_name has no NOT NULL");

    drop(introspector);
    teardown(db_name).await;
}

#[tokio::test]
async fn mysql_column_to_field() {
    let db_name = "ns_test_column_to_field";
    let introspector = setup_introspector(db_name).await;
    let columns = introspector.list_columns("users").await.expect("list columns");
    let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();

    let id = fields.iter().find(|f| f.name == "id").unwrap();
    assert_eq!(id.ty, DbType::Int);
    assert!(!id.nullable);

    let email = fields.iter().find(|f| f.name == "email").unwrap();
    assert_eq!(email.ty, DbType::String);
    assert!(!email.nullable);

    let salary = fields.iter().find(|f| f.name == "salary").unwrap();
    assert_eq!(salary.ty, DbType::Float);

    let avatar = fields.iter().find(|f| f.name == "avatar").unwrap();
    assert_eq!(avatar.ty, DbType::Bytes);

    let is_active = fields.iter().find(|f| f.name == "is_active").unwrap();
    assert_eq!(is_active.ty, DbType::Int);

    let created_at = fields.iter().find(|f| f.name == "created_at").unwrap();
    assert_eq!(created_at.ty, DbType::DateTime);

    let bio = fields.iter().find(|f| f.name == "bio").unwrap();
    assert_eq!(bio.ty, DbType::String);
    assert!(bio.nullable, "bio has no NOT NULL");

    drop(introspector);
    teardown(db_name).await;
}

#[tokio::test]
async fn mysql_full_pipeline() {
    let db_name = "ns_test_full_pipeline";
    let introspector = setup_introspector(db_name).await;

    let table_names = introspector.list_tables().await.expect("list tables");
    let mut tables = Vec::new();
    for name in &table_names {
        let columns = introspector.list_columns(name).await.expect("list columns");
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
        tables.push(neutrino_schema::ir::TableIR {
            name: name.clone(),
            fields,
        });
    }

    let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);
    let rel = schema.relations.iter().find(|r| r.from_table == "posts");
    assert!(rel.is_some(), "posts.user_id -> users.id relation should be inferred");
    if let Some(r) = rel {
        assert_eq!(r.from_field, "user_id");
        assert_eq!(r.to_table, "users");
        assert_eq!(r.to_field, "id");
    }

    let users_table = schema.tables.iter().find(|t| t.name == "users").unwrap();
    let output = neutrino_schema::generate_struct(users_table, neutrino_schema::RenderMode::Clean);
    assert!(output.contains("pub struct Users"));
    assert!(output.contains("pub id: i64,"));
    assert!(output.contains("pub email: String,"));
    assert!(output.contains("pub full_name: Option<String>,"));
    assert!(output.contains("pub salary: Option<f64>,"));
    assert!(output.contains("pub avatar: Option<Vec<u8>>,"));
    assert!(output.contains("pub bio: Option<String>,"));

    drop(introspector);
    teardown(db_name).await;
}
