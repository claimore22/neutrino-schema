use crate::types::DbType;

/// SQLite column type reported by `PRAGMA table_info`.
///
/// SQLite has only five storage classes; the reported type string is
/// the column's declared affinity.
#[derive(Debug, Clone, PartialEq)]
pub enum SqliteType {
    /// 64-bit signed integer (`INTEGER`).
    Int,
    /// 64-bit floating point (`REAL`).
    Real,
    /// UTF-8 string (`TEXT`).
    Text,
    /// Binary blob (`BLOB`).
    Blob,
    /// NULL value.
    Null,
    /// Unrecognised type name passed through verbatim.
    Unknown(String),
}

impl SqliteType {
    /// Parse a SQLite type name from `PRAGMA table_info.type`.
    ///
    /// Type names are case-insensitive and may carry a parenthesised
    /// width (e.g. `"VARCHAR(255)"` → [`SqliteType::Text`]).
    pub fn map_sqlite_type(t: &str) -> Self {
        let base = t.trim().split('(').next().unwrap_or(t).trim().to_lowercase();
        match base.as_str() {
            "int" | "integer" | "tinyint" | "smallint" | "mediumint" | "bigint"
            | "int2" | "int8" | "boolean" | "bool" => Self::Int,
            "real" | "double" | "double precision" | "float" | "numeric"
            | "decimal" | "number" | "dec" => Self::Real,
            "char" | "varchar" | "text" | "clob" | "character" | "varying character"
            | "native character" | "nchar" | "nvarchar" | "string" => Self::Text,
            "blob" | "binary" | "varbinary" => Self::Blob,
            "null" => Self::Null,
            other => {
                // SQLite date/time affinities
                if matches!(other, "date" | "datetime" | "timestamp" | "time") {
                    Self::Text
                } else {
                    Self::Unknown(t.trim().to_string())
                }
            }
        }
    }
}

/// Map a [`SqliteType`] to its database-agnostic [`DbType`].
///
/// # Examples
///
/// ```
/// use neutrino_schema::{SqliteType, sqlite_to_db_type, DbType};
///
/// assert_eq!(sqlite_to_db_type(SqliteType::Int), DbType::Int);
/// assert_eq!(sqlite_to_db_type(SqliteType::Real), DbType::Float);
/// assert_eq!(sqlite_to_db_type(SqliteType::Text), DbType::String);
/// ```
pub fn sqlite_to_db_type(ty: SqliteType) -> DbType {
    match ty {
        SqliteType::Int => DbType::Int,
        SqliteType::Real => DbType::Float,
        SqliteType::Text => DbType::String,
        SqliteType::Blob => DbType::Bytes,
        SqliteType::Null => DbType::Unknown("()".into()),
        SqliteType::Unknown(s) => DbType::Unknown(s),
    }
}
