/// PostgreSQL column data type returned by `information_schema.columns`.
///
/// This is the raw representation directly from the database.  It is never used
/// downstream — it is always mapped to [`DbType`](crate::types::DbType) via
/// [`to_db_type`](crate::types::to_db_type).
#[derive(Debug, Clone)]
pub enum PgType {
    // Integer types
    SmallInt,
    Integer,
    BigInt,

    // Serial types (auto-incrementing)
    SmallSerial,
    Serial,
    BigSerial,

    // Exact numeric
    Numeric,
    Decimal,

    // Floating-point
    Real,
    Double,

    // String types
    Varchar,
    Char,
    Text,

    // Boolean
    Boolean,

    // Binary
    Bytea,

    // Date/time
    Date,
    Time,
    Timestamp,
    TimestampTz,

    // JSON
    Json,
    Jsonb,

    // Network
    Inet,

    // UUID
    Uuid,

    /// Unrecognised type — passed through verbatim as a fallback.
    Unknown(String),
}

impl PgType {
    /// Parse a raw SQL type name from `information_schema.columns.data_type`.
    ///
    /// Returns [`PgType::Unknown`] when the string does not match any known type.
    pub fn map_pg_type(t: &str) -> Self {
        match t {
            "smallint" | "int2" => Self::SmallInt,
            "integer" | "int" | "int4" => Self::Integer,
            "bigint" | "int8" => Self::BigInt,
            "smallserial" | "serial2" => Self::SmallSerial,
            "serial" | "serial4" => Self::Serial,
            "bigserial" | "serial8" => Self::BigSerial,
            "numeric" => Self::Numeric,
            "decimal" => Self::Decimal,
            "real" | "float4" => Self::Real,
            "double precision" | "float8" => Self::Double,
            "character varying" | "varchar" => Self::Varchar,
            "character" | "char" => Self::Char,
            "text" => Self::Text,
            "boolean" | "bool" => Self::Boolean,
            "bytea" => Self::Bytea,
            "date" => Self::Date,
            "time without time zone" | "time" => Self::Time,
            "timestamp without time zone" | "timestamp" => Self::Timestamp,
            "timestamp with time zone" | "timestamptz" => Self::TimestampTz,
            "json" => Self::Json,
            "jsonb" => Self::Jsonb,
            "inet" => Self::Inet,
            "uuid" => Self::Uuid,
            other => Self::Unknown(other.to_string()),
        }
    }
}
