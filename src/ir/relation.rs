/// Where a relation definition originated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationOrigin {
    /// Derived from a real foreign key constraint in the database.
    ForeignKey,
    /// Inferred via naming heuristic (column ends with `_id`).
    Inferred,
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
///
/// One constraint = one [`RelationIR`]. Composite foreign keys appear as a
/// single entry with multiple columns in [`from_columns`](RelationIR::from_columns)
/// and [`to_columns`](RelationIR::to_columns).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationIR {
    /// Source table name.
    pub from_table: String,
    /// Source column(s) — multiple for composite foreign keys.
    pub from_columns: Vec<String>,
    /// Target table name.
    pub to_table: String,
    /// Target column(s) — multiple for composite foreign keys.
    pub to_columns: Vec<String>,
    /// How this relation was discovered.
    pub origin: RelationOrigin,
}
