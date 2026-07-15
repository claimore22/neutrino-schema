use crate::ir::{ConstraintIR, ConstraintKind, FieldIR, IndexIR};

/// A database table with its columns represented as [`FieldIR`] entries
/// and constraints as [`ConstraintIR`] entries.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableIR {
    /// Table name (e.g. `"users"`, `"blog_posts"`).
    pub name: String,
    /// Columns in ordinal position order.
    pub fields: Vec<FieldIR>,
    /// Constraints (PK, FK, UNIQUE, CHECK).
    pub constraints: Vec<ConstraintIR>,
    /// Optional comment string for the table, if any.
    pub comment: Option<String>,
    /// Physical indexes on the table.
    pub indexes: Vec<IndexIR>,
}

impl TableIR {
    /// Look up a physical index by name.
    pub fn index(&self, name: &str) -> Option<&IndexIR> {
        self.indexes.iter().find(|i| i.name == name)
    }

    /// Return the primary key constraint, if any.
    ///
    /// Logical constraint, not an index — see [`ConstraintKind::PrimaryKey`].
    pub fn primary_key(&self) -> Option<&ConstraintIR> {
        self.constraints
            .iter()
            .find(|c| matches!(c.kind, ConstraintKind::PrimaryKey { .. }))
    }
}
