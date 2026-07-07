use crate::types::PgType;

/// A raw column descriptor returned by [`DatabaseIntrospector::list_columns`].
///
/// Fields map directly to `information_schema.columns` rows.
/// `data_type` is the raw PostgreSQL type — see [`PgType`] for mapping.
#[derive(Debug, Clone)]
pub struct Column {
    /// Parent table name.
    pub table_name: String,
    /// Column name.
    pub column_name: String,
    /// Raw PostgreSQL data type (parsed but not yet normalised).
    pub data_type: PgType,
    /// Whether the column allows `NULL` (based on `is_nullable`).
    pub nullable: bool,
}
