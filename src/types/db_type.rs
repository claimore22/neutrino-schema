use crate::types::PgType;

/// Database-agnostic type representation.
///
/// This is the type system used throughout the IR and codegen layers.
/// Raw database types (PostgreSQL, MySQL, SQLite) are normalised into this
/// enum by [`to_db_type`](crate::types::to_db_type),
/// [`mysql_to_db_type`](crate::types::mysql_to_db_type), or
/// [`sqlite_to_db_type`](crate::types::sqlite_to_db_type).
/// Nullability is NEVER encoded here — always `Option<T>` at codegen time.
#[derive(Debug, Clone, PartialEq)]
pub enum DbType {
    /// Integer (signed 64-bit).
    Int,
    /// Floating-point (64-bit).
    Float,
    /// UTF-8 string.
    String,
    /// Boolean.
    Bool,
    /// UUID.
    Uuid,
    /// Date/time with time zone.
    DateTime,
    /// Binary blob.
    Bytes,
    /// Arbitrary JSON value.
    Json,
    /// IP address (v4 or v6).
    Inet,
    /// Unrecognised type — stored as a raw Rust type path.
    Unknown(String),
}

/// Map a raw [`PgType`] to its database-agnostic [`DbType`].
///
/// This is the boundary between the introspection and IR layers.
/// Raw SQL type distinctions (e.g. `Varchar` vs `Text`) are collapsed.
///
/// # Examples
///
/// ```
/// use neutrino_schema::{PgType, to_db_type, DbType};
///
/// assert_eq!(to_db_type(PgType::Int), DbType::Int);
/// assert_eq!(to_db_type(PgType::Varchar), DbType::String);
/// assert_eq!(to_db_type(PgType::TimestampTz), DbType::DateTime);
/// ```
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

/// Convert a [`DbType`] + nullability into a Rust type expression string.
///
/// Returns a Rust type path (e.g. `"i64"`, `"String"`, `"uuid::Uuid"`).
/// When `nullable` is `true` the type is wrapped in `Option<...>`.
///
/// # Examples
///
/// ```
/// use neutrino_schema::{DbType, dbtype_to_rust};
///
/// assert_eq!(dbtype_to_rust(&DbType::Int, false), "i64");
/// assert_eq!(dbtype_to_rust(&DbType::String, true), "Option<String>");
/// assert_eq!(dbtype_to_rust(&DbType::DateTime, false), "chrono::DateTime<chrono::Utc>");
/// ```
pub fn dbtype_to_rust(ty: &DbType, nullable: bool) -> String {
    let base = match ty {
        DbType::String => "String".to_string(),
        DbType::Int => "i64".to_string(),
        DbType::Float => "f64".to_string(),
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
