use crate::types::DbType;

/// MySQL/MariaDB column data type returned by `information_schema.columns`.
#[derive(Debug, Clone)]
pub enum MysqlType {
    // Integer types
    TinyInt,
    SmallInt,
    MediumInt,
    Int,
    BigInt,

    // Floating-point / decimal
    Float,
    Double,
    Decimal,
    Numeric,

    // String types
    Char,
    Varchar,
    TinyText,
    Text,
    MediumText,
    LongText,
    Enum,
    Set,

    // Boolean
    Boolean,

    // Binary types
    Binary,
    VarBinary,
    TinyBlob,
    Blob,
    MediumBlob,
    LongBlob,

    // Date/time
    Date,
    Datetime,
    Timestamp,
    Time,
    Year,

    // Structured
    Json,
    Uuid,

    /// Unrecognised type — passed through verbatim as a fallback.
    Unknown(String),
}

impl MysqlType {
    /// Parse a raw SQL type name from `information_schema.columns.data_type`.
    pub fn map_mysql_type(t: &str) -> Self {
        match t.to_lowercase().as_str() {
            "tinyint" => Self::TinyInt,
            "smallint" => Self::SmallInt,
            "mediumint" => Self::MediumInt,
            "int" | "integer" => Self::Int,
            "bigint" => Self::BigInt,
            "float" => Self::Float,
            "double" | "double precision" | "real" => Self::Double,
            "decimal" | "dec" | "fixed" => Self::Decimal,
            "numeric" => Self::Numeric,
            "char" => Self::Char,
            "varchar" => Self::Varchar,
            "tinytext" => Self::TinyText,
            "text" => Self::Text,
            "mediumtext" => Self::MediumText,
            "longtext" => Self::LongText,
            "enum" => Self::Enum,
            "set" => Self::Set,
            "boolean" | "bool" => Self::Boolean,
            "binary" => Self::Binary,
            "varbinary" => Self::VarBinary,
            "tinyblob" => Self::TinyBlob,
            "blob" => Self::Blob,
            "mediumblob" => Self::MediumBlob,
            "longblob" => Self::LongBlob,
            "date" => Self::Date,
            "datetime" => Self::Datetime,
            "timestamp" => Self::Timestamp,
            "time" => Self::Time,
            "year" => Self::Year,
            "json" => Self::Json,
            "uuid" => Self::Uuid,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// Map a [`MysqlType`] to its database-agnostic [`DbType`].
pub fn mysql_to_db_type(ty: MysqlType) -> DbType {
    match ty {
        MysqlType::TinyInt | MysqlType::SmallInt => DbType::SmallInt,
        MysqlType::MediumInt | MysqlType::Int => DbType::Integer,
        MysqlType::BigInt => DbType::BigInt,
        MysqlType::Float => DbType::Float32,
        MysqlType::Double => DbType::Float64,
        MysqlType::Decimal | MysqlType::Numeric => DbType::Decimal,
        MysqlType::Char | MysqlType::Varchar => DbType::String,
        MysqlType::TinyText | MysqlType::Text | MysqlType::MediumText | MysqlType::LongText => {
            DbType::Text
        }
        MysqlType::Enum => {
            // ENUM columns are handled by introspect_enums() -> DbType::Enum after post-processing
            DbType::String
        }
        MysqlType::Set => DbType::String,
        MysqlType::Boolean => DbType::Boolean,
        MysqlType::Binary | MysqlType::VarBinary => DbType::Binary,
        MysqlType::TinyBlob | MysqlType::Blob | MysqlType::MediumBlob | MysqlType::LongBlob => {
            DbType::Binary
        }
        MysqlType::Date => DbType::Date,
        MysqlType::Datetime | MysqlType::Timestamp => DbType::Timestamp,
        MysqlType::Time => DbType::Time,
        MysqlType::Year => DbType::Integer,
        MysqlType::Json => DbType::Json,
        MysqlType::Uuid => DbType::Uuid,
        MysqlType::Unknown(s) => DbType::Unknown(s),
    }
}
