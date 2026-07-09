use neutrino_schema::types::*;

#[test]
fn pg_smallint() {
    let pg = PgType::map_pg_type("smallint");
    assert_eq!(to_db_type(pg), DbType::SmallInt);
}

#[test]
fn pg_integer() {
    let pg = PgType::map_pg_type("integer");
    assert_eq!(to_db_type(pg), DbType::Integer);
}

#[test]
fn pg_bigint() {
    let pg = PgType::map_pg_type("bigint");
    assert_eq!(to_db_type(pg), DbType::BigInt);
}

#[test]
fn pg_serial() {
    let pg = PgType::map_pg_type("serial");
    assert_eq!(to_db_type(pg), DbType::Serial);
}

#[test]
fn pg_bigserial() {
    let pg = PgType::map_pg_type("bigserial");
    assert_eq!(to_db_type(pg), DbType::BigSerial);
}

#[test]
fn pg_numeric() {
    let pg = PgType::map_pg_type("numeric");
    assert_eq!(to_db_type(pg), DbType::Decimal);
}

#[test]
fn pg_decimal() {
    let pg = PgType::map_pg_type("decimal");
    assert_eq!(to_db_type(pg), DbType::Decimal);
}

#[test]
fn pg_real() {
    let pg = PgType::map_pg_type("real");
    assert_eq!(to_db_type(pg), DbType::Float32);
}

#[test]
fn pg_double() {
    let pg = PgType::map_pg_type("double precision");
    assert_eq!(to_db_type(pg), DbType::Float64);
}

#[test]
fn pg_varchar() {
    let pg = PgType::map_pg_type("character varying");
    assert_eq!(to_db_type(pg), DbType::String);
}

#[test]
fn pg_text() {
    let pg = PgType::map_pg_type("text");
    assert_eq!(to_db_type(pg), DbType::Text);
}

#[test]
fn pg_boolean() {
    let pg = PgType::map_pg_type("boolean");
    assert_eq!(to_db_type(pg), DbType::Boolean);
}

#[test]
fn pg_bytea() {
    let pg = PgType::map_pg_type("bytea");
    assert_eq!(to_db_type(pg), DbType::Binary);
}

#[test]
fn pg_date() {
    let pg = PgType::map_pg_type("date");
    assert_eq!(to_db_type(pg), DbType::Date);
}

#[test]
fn pg_timestamp() {
    let pg = PgType::map_pg_type("timestamp");
    assert_eq!(to_db_type(pg), DbType::Timestamp);
}

#[test]
fn pg_timestamptz() {
    let pg = PgType::map_pg_type("timestamp with time zone");
    assert_eq!(to_db_type(pg), DbType::TimestampTz);
}

#[test]
fn pg_jsonb() {
    let pg = PgType::map_pg_type("jsonb");
    assert_eq!(to_db_type(pg), DbType::Jsonb);
}

#[test]
fn pg_uuid() {
    let pg = PgType::map_pg_type("uuid");
    assert_eq!(to_db_type(pg), DbType::Uuid);
}

#[test]
fn pg_inet() {
    let pg = PgType::map_pg_type("inet");
    assert_eq!(to_db_type(pg), DbType::Inet);
}

#[test]
fn mysql_tinyint() {
    let ty = MysqlType::map_mysql_type("tinyint");
    assert_eq!(mysql_to_db_type(ty), DbType::SmallInt);
}

#[test]
fn mysql_int() {
    let ty = MysqlType::map_mysql_type("int");
    assert_eq!(mysql_to_db_type(ty), DbType::Integer);
}

#[test]
fn mysql_bigint() {
    let ty = MysqlType::map_mysql_type("bigint");
    assert_eq!(mysql_to_db_type(ty), DbType::BigInt);
}

#[test]
fn mysql_decimal() {
    let ty = MysqlType::map_mysql_type("decimal");
    assert_eq!(mysql_to_db_type(ty), DbType::Decimal);
}

#[test]
fn mysql_float() {
    let ty = MysqlType::map_mysql_type("float");
    assert_eq!(mysql_to_db_type(ty), DbType::Float32);
}

#[test]
fn mysql_double() {
    let ty = MysqlType::map_mysql_type("double");
    assert_eq!(mysql_to_db_type(ty), DbType::Float64);
}

#[test]
fn mysql_varchar() {
    let ty = MysqlType::map_mysql_type("varchar");
    assert_eq!(mysql_to_db_type(ty), DbType::String);
}

#[test]
fn mysql_text() {
    let ty = MysqlType::map_mysql_type("text");
    assert_eq!(mysql_to_db_type(ty), DbType::Text);
}

#[test]
fn mysql_blob() {
    let ty = MysqlType::map_mysql_type("blob");
    assert_eq!(mysql_to_db_type(ty), DbType::Binary);
}

#[test]
fn mysql_json() {
    let ty = MysqlType::map_mysql_type("json");
    assert_eq!(mysql_to_db_type(ty), DbType::Json);
}

#[test]
fn mysql_date() {
    let ty = MysqlType::map_mysql_type("date");
    assert_eq!(mysql_to_db_type(ty), DbType::Date);
}

#[test]
fn mysql_datetime() {
    let ty = MysqlType::map_mysql_type("datetime");
    assert_eq!(mysql_to_db_type(ty), DbType::Timestamp);
}

#[test]
fn sqlite_integer() {
    let ty = SqliteType::map_sqlite_type("INTEGER");
    assert_eq!(sqlite_to_db_type(ty), DbType::Integer);
}

#[test]
fn sqlite_real() {
    let ty = SqliteType::map_sqlite_type("REAL");
    assert_eq!(sqlite_to_db_type(ty), DbType::Float64);
}

#[test]
fn sqlite_text() {
    let ty = SqliteType::map_sqlite_type("TEXT");
    assert_eq!(sqlite_to_db_type(ty), DbType::String);
}

#[test]
fn sqlite_blob() {
    let ty = SqliteType::map_sqlite_type("BLOB");
    assert_eq!(sqlite_to_db_type(ty), DbType::Binary);
}

#[test]
fn sqlite_decimal_is_unknown() {
    let ty = SqliteType::map_sqlite_type("DECIMAL");
    // SQLite does not enforce DECIMAL — map to Unknown
    assert_eq!(ty, SqliteType::Unknown("decimal".into()));
}

#[test]
fn registry_default_resolves() {
    let registry = TypeRegistry::default();
    assert_eq!(registry.resolve(&DbType::Integer).name, "i32");
    assert_eq!(registry.resolve(&DbType::Decimal).name, "rust_decimal::Decimal");
    assert_eq!(registry.resolve(&DbType::Uuid).name, "uuid::Uuid");
    assert_eq!(registry.resolve(&DbType::TimestampTz).name, "chrono::DateTime<chrono::Utc>");
    assert_eq!(registry.resolve(&DbType::Binary).name, "Vec<u8>");
    assert_eq!(registry.resolve(&DbType::Unknown("foo".into())).name, "String");
}

#[test]
fn registry_overrides_work() {
    let overrides = [("Integer".into(), "my_crate::MyInt".into())]
        .into_iter()
        .collect();
    let registry = TypeRegistry::with_overrides(overrides);
    assert_eq!(
        registry.resolve(&DbType::Integer).name,
        "my_crate::MyInt"
    );
}

#[test]
fn registry_generates_imports() {
    let registry = TypeRegistry::default();
    let rt = registry.resolve(&DbType::Decimal);
    assert!(rt.imports.contains(&"use rust_decimal::Decimal;".into()));
    let rt = registry.resolve(&DbType::Uuid);
    assert!(rt.imports.contains(&"use uuid::Uuid;".into()));
    let rt = registry.resolve(&DbType::Binary);
    assert!(rt.imports.is_empty());
}
