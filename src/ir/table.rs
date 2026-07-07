use crate::ir::FieldIR;

/// A database table with its columns represented as [`FieldIR`] entries.
#[derive(Debug)]
pub struct TableIR {
    /// Table name (e.g. `"users"`, `"blog_posts"`).
    pub name: String,
    /// Columns in ordinal position order.
    pub fields: Vec<FieldIR>,
}
