use crate::ir::{ConstraintIR, FieldIR};

/// A database table with its columns represented as [`FieldIR`] entries
/// and constraints as [`ConstraintIR`] entries.
#[derive(Debug)]
pub struct TableIR {
    /// Table name (e.g. `"users"`, `"blog_posts"`).
    pub name: String,
    /// Columns in ordinal position order.
    pub fields: Vec<FieldIR>,
    /// Constraints (PK, FK, UNIQUE, CHECK).
    pub constraints: Vec<ConstraintIR>,
    /// Optional comment string for the table, if any.
    pub comment: Option<String>,
}
