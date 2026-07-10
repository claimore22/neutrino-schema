/// How a relation was discovered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationSource {
    /// Derived from a real foreign key constraint in the database.
    ForeignKey(String),
    /// Inferred via naming heuristic (column ends with `_id`).
    NamingHeuristic,
}

/// Controls whether [`SchemaIR::from_tables`](crate::ir::SchemaIR::from_tables) attempts to infer foreign-key-like
/// relationships between tables.
///
/// Relation inference is always best-effort and does **not** query database
/// foreign key constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationStrategy {
    /// No relation inference — relations vec will be empty.
    Disabled,
    /// Infer relations using naming convention heuristic (column ends with `_id`).
    /// These are best-effort guesses, not verified foreign key constraints.
    NamingHeuristic,
}

/// A relationship between two tables.
///
/// Produced either from a real [`ForeignKey`](crate::ir::ConstraintKind::ForeignKey)
/// constraint or via the naming heuristic fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationIR {
    /// Source table name.
    pub from_table: String,
    /// Source column name (the `_id` column).
    pub from_field: String,
    /// Target table name.
    pub to_table: String,
    /// Target column name (always `"id"` with current heuristics).
    pub to_field: String,
    /// How this relation was discovered.
    pub source: RelationSource,
}
