/// Controls whether [`SchemaIR::from_tables`] attempts to infer foreign-key-like
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

/// An inferred relationship between two tables.
///
/// Currently produced only by [`RelationStrategy::NamingHeuristic`]:
/// a column ending in `_id` is assumed to reference the `id` column of
/// the table named by the prefix (or its plural).
#[derive(Debug)]
pub struct RelationIR {
    /// Source table name.
    pub from_table: String,
    /// Source column name (the `_id` column).
    pub from_field: String,
    /// Target table name.
    pub to_table: String,
    /// Target column name (always `"id"` with current heuristics).
    pub to_field: String,
}
