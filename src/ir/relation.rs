#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationStrategy {
    /// No relation inference — relations vec will be empty.
    Disabled,
    /// Infer relations using naming convention heuristic (column ends with `_id`).
    /// These are best-effort guesses, not verified foreign key constraints.
    NamingHeuristic,
}

#[derive(Debug)]
pub struct RelationIR {
    pub from_table: String,
    pub from_field: String,
    pub to_table: String,
    pub to_field: String,
}
