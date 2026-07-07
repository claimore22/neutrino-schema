use crate::types::DbType;

/// MySQL/MariaDB column data type returned by `information_schema.columns`.
#[derive(Debug, Clone)]
pub enum MysqlType {
    /// Integer types (`tinyint`, `smallint`, `mediumint`, `int`, `integer`, `bigint`).
    Int,
    /// Floating-point types (`float`, `double`, `decimal`, `numeric`).
    Float,
    /// String types (`char`, `varchar`, `text`, `mediumtext`, `longtext`, `enum`, `set`).
    String,
    /// Boolean (`boolean`, `bool` — aliases for `tinyint(1)`).
    Bool,
    /// Binary blob types (`binary`, `varbinary`, `blob`, `mediumblob`, `longblob`).
    Bytes,
    /// Date/time types (`date`, `datetime`, `timestamp`, `time`, `year`).
    DateTime,
    /// JSON type (`json`).
    Json,
    /// UUID type (`uuid` — MariaDB 11.6+; MySQL stores as `char(36)`).
    Uuid,
    /// Unrecognised type — passed through verbatim as a fallback.
    Unknown(String),
}

impl MysqlType {
    /// Parse a raw SQL type name from `information_schema.columns.data_type`.
    pub fn map_mysql_type(t: &str) -> Self {
        match t.to_lowercase().as_str() {
            "tinyint" | "smallint" | "mediumint" | "int" | "integer" | "bigint" => Self::Int,
            "float" | "double" | "double precision" | "real" | "decimal" | "numeric"
            | "fixed" => Self::Float,
            "char" | "varchar" | "tinytext" | "text" | "mediumtext" | "longtext" | "enum"
            | "set" => Self::String,
            "boolean" | "bool" => Self::Bool,
            "binary" | "varbinary" | "tinyblob" | "blob" | "mediumblob" | "longblob" => Self::Bytes,
            "date" | "datetime" | "timestamp" | "time" | "year" => Self::DateTime,
            "json" => Self::Json,
            "uuid" => Self::Uuid,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// Map a [`MysqlType`] to its database-agnostic [`DbType`].
pub fn mysql_to_db_type(ty: MysqlType) -> DbType {
    match ty {
        MysqlType::Int => DbType::Int,
        MysqlType::Float => DbType::Float,
        MysqlType::String => DbType::String,
        MysqlType::Bool => DbType::Bool,
        MysqlType::Bytes => DbType::Bytes,
        MysqlType::DateTime => DbType::DateTime,
        MysqlType::Json => DbType::Json,
        MysqlType::Uuid => DbType::Uuid,
        MysqlType::Unknown(s) => DbType::Unknown(s),
    }
}
