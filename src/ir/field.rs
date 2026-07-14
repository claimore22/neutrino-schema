use crate::types::DbType;

/// A single column in a table, represented in the database-agnostic type system.
#[derive(Debug)]
pub struct FieldIR {
    /// Column name (e.g. `"email"`, `"created_at"`).
    pub name: String,
    /// Normalised type — one of the database-agnostic [`DbType`] variants.
    pub ty: DbType,
    /// Whether the column allows `NULL`.
    pub nullable: bool,
    /// Raw SQL type name, only used for debug/CLI display. Never consulted by type pipeline.
    pub raw_type: String,
    /// Raw SQL default expression, `None` means no default was defined.
    pub default_value: Option<String>,
    /// Whether the database generates this column's value (auto-increment,
    /// identity, etc.).
    pub generated: bool,
    /// Column comment, if any.
    pub comment: Option<String>,
}
