/// PostgreSQL column data type returned by `information_schema.columns`.
///
/// This is the raw representation directly from the database.  It is never used
/// downstream — it is always mapped to [`DbType`] via [`to_db_type`].
#[derive(Debug, Clone)]
pub enum PgType {
    /// 32-bit integer (`integer`, `int`, `int4`, `smallint`).
    Int,
    /// 64-bit integer (`bigint`, `int8`).
    BigInt,
    /// Variable-length string (`character varying`, `varchar`).
    Varchar,
    /// Unlimited text (`text`).
    Text,
    /// UUID (`uuid`).
    Uuid,
    /// Boolean (`boolean`, `bool`).
    Bool,
    /// Timestamp with time zone (`timestamp with time zone`, `timestamptz`).
    TimestampTz,
    /// IPv4/IPv6 address (`inet`).
    Inet,
    /// Binary JSON (`jsonb`).
    Jsonb,
    /// Unrecognised type — passed through verbatim as a fallback.
    Unknown(String),
}

impl PgType {
    /// Parse a raw SQL type name from `information_schema.columns.data_type`.
    ///
    /// Returns [`PgType::Unknown`] when the string does not match any known type.
    pub fn map_pg_type(t: &str) -> Self {
        match t {
            "integer" | "int" | "int4" | "smallint" => Self::Int,
            "bigint" | "int8" => Self::BigInt,
            "character varying" | "varchar" => Self::Varchar,
            "text" => Self::Text,
            "uuid" => Self::Uuid,
            "boolean" | "bool" => Self::Bool,
            "timestamp with time zone" | "timestamptz" => Self::TimestampTz,
            "inet" => Self::Inet,
            "jsonb" => Self::Jsonb,
            other => Self::Unknown(other.to_string()),
        }
    }
}
