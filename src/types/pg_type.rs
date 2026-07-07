#[derive(Debug, Clone)]
pub enum PgType {
    Int,
    BigInt,
    Varchar,
    Text,
    Uuid,
    Bool,
    TimestampTz,
    Inet,
    Jsonb,
    Unknown(String),
}

impl PgType {
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
