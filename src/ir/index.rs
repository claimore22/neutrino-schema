/// A physical database index in the intermediate representation.
///
/// Describes the storage-level index structure. Logical constraints
/// (PK, FK, UNIQUE, CHECK) are captured in [`ConstraintIR`] — an index
/// may implement a constraint, but the index is not the constraint itself.
#[derive(Debug, Clone)]
pub struct IndexIR {
    /// Index name (e.g. `"users_email_idx"`, `"idx_lower_email"`).
    pub name: String,
    /// The table this index belongs to.
    pub table_name: String,
    /// Indexed entries in ordinal order (columns or expressions).
    pub entries: Vec<IndexEntryIR>,
    /// Whether the index enforces uniqueness.
    pub unique: bool,
    /// Index access method / structure.
    pub kind: IndexKind,
    /// Partial index predicate (`WHERE` clause), if any.
    pub predicate: Option<String>,
}

/// A single entry in an index definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexEntryIR {
    /// A simple column reference (optionally descending).
    Column {
        name: String,
        descending: bool,
    },
    /// A scalar expression (e.g. `lower(email)`, `(metadata jsonb_path_ops)`).
    Expression {
        expression: String,
    },
}

/// The access method / index structure type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexKind {
    BTree,
    Hash,
    Gin,
    Gist,
    Brin,
    FullText,
    Spatial,
    /// Any other index type not covered above (e.g. `Other("bloom".into())`).
    Other(String),
}
