use crate::types::{EnumRef, PgType, TypeRegistry};

/// Database-agnostic type representation.
///
/// This is the type system used throughout the IR and codegen layers.
/// Raw database types (PostgreSQL, MySQL, SQLite) are normalised into this
/// enum by [`to_db_type`], [`mysql_to_db_type`], or [`sqlite_to_db_type`].
/// Nullability is NEVER encoded here — always `Option<T>` at codegen time.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum DbType {
    // Numeric
    SmallInt,
    Integer,
    BigInt,
    SmallSerial,
    Serial,
    BigSerial,
    Decimal,
    Float32,
    Float64,

    // Text
    String,
    Text,
    Boolean,

    // Binary
    Binary,

    // Date/time
    Date,
    Time,
    Timestamp,
    TimestampTz,

    // Structured
    Json,
    Jsonb,
    Uuid,
    Inet,

    // Collections
    Array(Box<DbType>),

    // Schema references
    Enum(EnumRef),

    /// Unrecognised type — mapped to `String` with a warning.
    Unknown(String),
}

/// Map a raw [`PgType`] to its database-agnostic [`DbType`].
///
/// This is the boundary between the introspection and IR layers.
/// Raw SQL type distinctions (e.g. `SmallInt` vs `BigInt`) are preserved
/// so the downstream pipeline can generate precise Rust types.
///
/// # Examples
///
/// ```
/// use neutrino_schema::{PgType, to_db_type, DbType};
///
/// assert_eq!(to_db_type(PgType::SmallInt), DbType::SmallInt);
/// assert_eq!(to_db_type(PgType::Varchar), DbType::String);
/// assert_eq!(to_db_type(PgType::TimestampTz), DbType::TimestampTz);
/// ```
pub fn to_db_type(pg: PgType) -> DbType {
    match pg {
        PgType::SmallInt => DbType::SmallInt,
        PgType::Integer => DbType::Integer,
        PgType::BigInt => DbType::BigInt,
        PgType::SmallSerial => DbType::SmallSerial,
        PgType::Serial => DbType::Serial,
        PgType::BigSerial => DbType::BigSerial,
        PgType::Numeric | PgType::Decimal => DbType::Decimal,
        PgType::Real => DbType::Float32,
        PgType::Double => DbType::Float64,
        PgType::Varchar | PgType::Char => DbType::String,
        PgType::Text => DbType::Text,
        PgType::Boolean => DbType::Boolean,
        PgType::Bytea => DbType::Binary,
        PgType::Date => DbType::Date,
        PgType::Time => DbType::Time,
        PgType::Timestamp => DbType::Timestamp,
        PgType::TimestampTz => DbType::TimestampTz,
        PgType::Json => DbType::Json,
        PgType::Jsonb => DbType::Jsonb,
        PgType::Uuid => DbType::Uuid,
        PgType::Inet => DbType::Inet,
        PgType::Unknown(s) => DbType::Unknown(s),
    }
}

/// Convert a [`DbType`] + nullability into a Rust type expression string.
///
/// Returns a Rust type path (e.g. `"i64"`, `"String"`, `"uuid::Uuid"`).
/// When `nullable` is `true` the type is wrapped in `Option<...>`.
///
/// This is a convenience wrapper around [`TypeRegistry::default().resolve()`].
/// For full control over imports and type overrides, use [`TypeRegistry`] directly.
///
/// # Examples
///
/// ```
/// use neutrino_schema::{DbType, dbtype_to_rust};
///
/// assert_eq!(dbtype_to_rust(&DbType::Integer, false), "i32");
/// assert_eq!(dbtype_to_rust(&DbType::String, true), "Option<String>");
/// assert_eq!(dbtype_to_rust(&DbType::TimestampTz, false), "chrono::DateTime<chrono::Utc>");
/// ```
pub fn dbtype_to_rust(ty: &DbType, nullable: bool) -> String {
    let registry = TypeRegistry::default();
    let rt = registry.resolve(ty);
    if nullable {
        format!("Option<{}>", rt.name)
    } else {
        rt.name
    }
}
