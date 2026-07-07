use crate::ir::{RelationIR, RelationStrategy, TableIR};

#[derive(Debug)]
pub struct SchemaIR {
    pub tables: Vec<TableIR>,
    pub relations: Vec<RelationIR>,
}

impl SchemaIR {
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
