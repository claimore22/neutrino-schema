use crate::ir::{RelationIR, RelationStrategy, TableIR};

/// Full database schema — the top-level IR object consumed by code generation.
///
/// Construct via [`SchemaIR::from_tables`], which populates the relations
/// vector according to the chosen [`RelationStrategy`].
#[derive(Debug)]
pub struct SchemaIR {
    /// All tables discovered or provided.
    pub tables: Vec<TableIR>,
    /// Inferred inter-table relationships (may be empty).
    pub relations: Vec<RelationIR>,
}

impl SchemaIR {
    /// Build a [`SchemaIR`] from a set of tables, inferring relations
    /// according to `strategy`.
    pub fn from_tables(tables: Vec<TableIR>, strategy: RelationStrategy) -> Self {
        let relations = match strategy {
            RelationStrategy::Disabled => Vec::new(),
            RelationStrategy::NamingHeuristic => Self::infer_relations_heuristic(&tables),
        };
        SchemaIR { tables, relations }
    }

    fn table_exists(tables: &[TableIR], name: &str) -> bool {
        tables.iter().any(|t| t.name == name)
    }

    fn infer_relations_heuristic(tables: &[TableIR]) -> Vec<RelationIR> {
        let mut relations = Vec::new();

        for table in tables {
            for field in &table.fields {
                let Some(prefix) = field.name.strip_suffix("_id") else {
                    continue;
                };

                let to_table = if Self::table_exists(tables, prefix) {
                    prefix.to_string()
                } else {
                    let plural = format!("{}s", prefix);
                    if Self::table_exists(tables, &plural) {
                        plural
                    } else {
                        continue;
                    }
                };

                relations.push(RelationIR {
                    from_table: table.name.clone(),
                    from_field: field.name.clone(),
                    to_table,
                    to_field: "id".to_string(),
                });
            }
        }

        relations
    }
}
