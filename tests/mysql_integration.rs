use neutrino_schema::{
    introspect::{DatabaseIntrospector, MysqlIntrospector},
    ir::SchemaIR,
    types::DbType,
    RelationStrategy,
};

const MYSQL_URL: &str = "mysql://root:1qaz2wsx@localhost:3306";

async fn try_admin() -> Option<sqlx::mysql::MySqlPool> {
    sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(1)
        .connect(&format!("{MYSQL_URL}/mysql"))
        .await
        .ok()
}

async fn setup(db_name: &str) -> Option<MysqlIntrospector> {
    let admin = try_admin().await?;

    sqlx::query(sqlx::AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS `{db_name}`"
    )))
    .execute(&admin)
    .await
    .ok();
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE DATABASE `{db_name}`"
    )))
    .execute(&admin)
    .await
    .ok()?;
    admin.close().await;

    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(1)
        .connect(&format!("{MYSQL_URL}/{db_name}"))
        .await
        .ok()?;

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
    .ok()?;

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
    .ok()?;

    Some(MysqlIntrospector::new(pool))
}

async fn teardown(db_name: &str) {
    if let Some(admin) = try_admin().await {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "DROP DATABASE IF EXISTS `{db_name}`"
        )))
        .execute(&admin)
        .await
        .ok();
    }
}

#[tokio::test]
async fn mysql_list_tables() {
    let Some(introspector) = setup("ns_list_tables").await else {
        eprintln!("MySQL unreachable — skipping mysql_list_tables");
        return;
    };
    let tables = introspector.list_tables().await.expect("list tables");
    assert!(tables.contains(&"users".to_string()));
    assert!(tables.contains(&"posts".to_string()));
    drop(introspector);
    teardown("ns_list_tables").await;
}

#[tokio::test]
async fn mysql_list_columns() {
    let Some(introspector) = setup("ns_list_columns").await else {
        eprintln!("MySQL unreachable — skipping mysql_list_columns");
        return;
    };
    let columns = introspector.list_columns("users").await.expect("list columns");
    assert!(!columns.iter().find(|c| c.column_name == "id").unwrap().nullable);
    assert!(!columns.iter().find(|c| c.column_name == "email").unwrap().nullable);
    assert!(columns.iter().find(|c| c.column_name == "full_name").unwrap().nullable);
    drop(introspector);
    teardown("ns_list_columns").await;
}

#[tokio::test]
async fn mysql_column_to_field() {
    let Some(introspector) = setup("ns_column_to_field").await else {
        eprintln!("MySQL unreachable — skipping mysql_column_to_field");
        return;
    };
    let columns = introspector.list_columns("users").await.expect("list columns");
    let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
    assert_eq!(fields.iter().find(|f| f.name == "id").unwrap().ty, DbType::Integer);
    assert_eq!(fields.iter().find(|f| f.name == "email").unwrap().ty, DbType::String);
    assert_eq!(fields.iter().find(|f| f.name == "salary").unwrap().ty, DbType::Decimal);
    assert_eq!(fields.iter().find(|f| f.name == "avatar").unwrap().ty, DbType::Binary);
    drop(introspector);
    teardown("ns_column_to_field").await;
}

#[tokio::test]
async fn mysql_full_pipeline() {
    let Some(introspector) = setup("ns_full_pipeline").await else {
        eprintln!("MySQL unreachable — skipping mysql_full_pipeline");
        return;
    };
    let table_names = introspector.list_tables().await.expect("list tables");
    let mut tables = Vec::new();
    for name in &table_names {
        let columns = introspector.list_columns(name).await.expect("list columns");
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
        tables.push(neutrino_schema::ir::TableIR { name: name.clone(), fields, constraints: vec![] });
    }
    let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);
    assert!(schema.relations.iter().any(|r| r.from_table == "posts"));
    let output = neutrino_schema::generate_struct(
        schema.tables.iter().find(|t| t.name == "users").unwrap(),
        neutrino_schema::RenderMode::Clean,
    );
    assert!(output.contains("pub struct Users"));
    drop(introspector);
    teardown("ns_full_pipeline").await;
}
