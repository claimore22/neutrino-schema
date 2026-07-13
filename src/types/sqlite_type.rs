use crate::types::DbType;

/// SQLite column type reported by `PRAGMA table_info`.
///
/// SQLite has only five storage classes; the reported type string is
/// the column's declared affinity.
#[derive(Debug, Clone, PartialEq)]
pub enum SqliteType {
    /// 64-bit signed integer (`INTEGER`).
    Integer,
    /// 64-bit floating point (`REAL`, `FLOAT`, `DOUBLE`).
    Real,
    /// UTF-8 string (`TEXT`, `VARCHAR`, `CHAR`).
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
        let base = t
            .trim()
            .split('(')
            .next()
            .unwrap_or(t)
            .trim()
            .to_lowercase();
        match base.as_str() {
            "int" | "integer" | "tinyint" | "smallint" | "mediumint" | "bigint" | "int2"
            | "int8" | "boolean" | "bool" => Self::Integer,
            "real" | "double" | "double precision" | "float" | "number" => Self::Real,
            // DECIMAL/NUMERIC in SQLite are not enforced — map to Unknown
            "numeric" | "decimal" | "dec" => Self::Unknown(base),
            "char" | "varchar" | "text" | "clob" | "character" | "varying character"
            | "native character" | "nchar" | "nvarchar" | "string" => Self::Text,
            "blob" | "binary" | "varbinary" => Self::Blob,
            "null" => Self::Null,
            other => {
                // SQLite date/time affinities — stored as TEXT, keep as Unknown
                if matches!(other, "date" | "datetime" | "timestamp" | "time") {
                    Self::Unknown(other.to_string())
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
/// assert_eq!(sqlite_to_db_type(SqliteType::Integer), DbType::Integer);
/// assert_eq!(sqlite_to_db_type(SqliteType::Real), DbType::Float64);
/// assert_eq!(sqlite_to_db_type(SqliteType::Text), DbType::String);
/// ```
pub fn sqlite_to_db_type(ty: SqliteType) -> DbType {
    match ty {
        SqliteType::Integer => DbType::Integer,
        SqliteType::Real => DbType::Float64,
        SqliteType::Text => DbType::String,
        SqliteType::Blob => DbType::Binary,
        SqliteType::Null => DbType::Unknown("()".into()),
        SqliteType::Unknown(s) => DbType::Unknown(s),
    }
}

/// Map a declared SQLite column type string to [`DbType`], preserving
/// the user's declared intent rather than falling back to SQLite affinity.
///
/// SQLite's five storage classes lose information (e.g. `BIGINT` and `SMALLINT`
/// both map to `Integer` affinity).  This function checks the declared type
/// name first so that `BIGINT` → `DbType::BigInt` → `i64` instead of `i32`.
///
/// Unknown type names fall through to the existing affinity-based
/// [`sqlite_to_db_type`] / [`SqliteType::map_sqlite_type`] pipeline.
///
/// # Examples
///
/// ```
/// use neutrino_schema::{sqlite_declared_to_db_type, DbType};
///
/// assert_eq!(sqlite_declared_to_db_type("BIGINT"), DbType::BigInt);
/// assert_eq!(sqlite_declared_to_db_type("SMALLINT"), DbType::SmallInt);
/// assert_eq!(sqlite_declared_to_db_type("INTEGER"), DbType::Integer);
/// assert_eq!(sqlite_declared_to_db_type("TEXT"), DbType::String);
/// ```
pub fn sqlite_declared_to_db_type(declared: &str) -> DbType {
    match declared.trim().to_uppercase().as_str() {
        "INT" | "INTEGER" | "INT4" => return DbType::Integer,
        "TINYINT" => return DbType::SmallInt,
        "SMALLINT" | "INT2" => return DbType::SmallInt,
        "MEDIUMINT" => return DbType::Integer,
        "BIGINT" | "INT8" => return DbType::BigInt,
        "BOOLEAN" | "BOOL" => return DbType::Boolean,
        _ => {}
    }
    let sqlite_ty = SqliteType::map_sqlite_type(declared);
    sqlite_to_db_type(sqlite_ty)
}
