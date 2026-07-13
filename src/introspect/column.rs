/// A raw column descriptor returned by
/// [`DatabaseIntrospector::list_columns`](crate::introspect::DatabaseIntrospector::list_columns).
///
/// Fields map directly to database metadata views/PRAGMAs.
/// `data_type` is the raw type name string from the database —
/// use [`PgType::map_pg_type`](crate::PgType) or
/// [`SqliteType::map_sqlite_type`](crate::SqliteType) to parse it.
#[derive(Debug, Clone)]
pub struct Column {
    /// Parent table name.
    pub table_name: String,
    /// Column name.
    pub column_name: String,
    /// Raw type name string as reported by the database.
    pub data_type: String,
    /// Whether the column allows `NULL` (based on `is_nullable`).
    pub nullable: bool,
    /// Comment string for the column, if any.
    pub comment: Option<String>,
}
