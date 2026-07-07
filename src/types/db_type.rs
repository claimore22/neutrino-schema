use crate::types::PgType;

#[derive(Debug, Clone)]
pub enum DbType {
    Int,
    String,
    Bool,
    Uuid,
    DateTime,
    Bytes,
    Json,
    Inet,
    Unknown(String),
}

pub fn to_db_type(pg: PgType) -> DbType {
    match pg {
        PgType::Int | PgType::BigInt => DbType::Int,
        PgType::Varchar | PgType::Text => DbType::String,
        PgType::Uuid => DbType::Uuid,
        PgType::Bool => DbType::Bool,
        PgType::TimestampTz => DbType::DateTime,
        PgType::Inet => DbType::Inet,
        PgType::Jsonb => DbType::Json,
        PgType::Unknown(s) => DbType::Unknown(s),
    }
}

/// Convert a DbType + nullability into a Rust type string.
///
/// Note: This is a Rust codegen concern, not a pure schema concept.
/// It lives here for v0.1 simplicity but may migrate to the generator crate later.
pub fn dbtype_to_rust(ty: &DbType, nullable: bool) -> String {
    let base = match ty {
        DbType::String => "String".to_string(),
        DbType::Int => "i64".to_string(),
        DbType::Uuid => "uuid::Uuid".to_string(),
        DbType::DateTime => "chrono::DateTime<chrono::Utc>".to_string(),
        DbType::Bool => "bool".to_string(),
        DbType::Inet => "std::net::IpAddr".to_string(),
        DbType::Bytes => "Vec<u8>".to_string(),
        DbType::Json => "serde_json::Value".to_string(),
        DbType::Unknown(s) => s.clone(),
    };

    if nullable {
        format!("Option<{}>", base)
    } else {
        base
    }
}
