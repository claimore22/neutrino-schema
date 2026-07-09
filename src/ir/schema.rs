use std::collections::HashSet;

use crate::ir::{EnumIR, RelationIR, RelationStrategy, TableIR};
use crate::types::{DbType, EnumRef};

/// Full database schema — the top-level IR object consumed by code generation.
///
/// Construct via [`SchemaIR::from_tables`] (without enums) or
/// [`SchemaIR::with_enums`] for the full schema including enum definitions.
#[derive(Debug)]
pub struct SchemaIR {
    /// All tables discovered or provided.
    pub tables: Vec<TableIR>,
    /// Inferred inter-table relationships (may be empty).
    pub relations: Vec<RelationIR>,
    /// Database enum types (e.g. PostgreSQL `CREATE TYPE`, MySQL `ENUM` columns).
    pub enums: Vec<EnumIR>,
}

impl SchemaIR {
    /// Build a [`SchemaIR`] from a set of tables without enums.
    pub fn from_tables(tables: Vec<TableIR>, strategy: RelationStrategy) -> Self {
        let relations = match strategy {
            RelationStrategy::Disabled => Vec::new(),
            RelationStrategy::NamingHeuristic => Self::infer_relations_heuristic(&tables),
        };
        SchemaIR {
            tables,
            relations,
            enums: Vec::new(),
        }
    }

    /// Build a [`SchemaIR`] from tables, enums, and a relation strategy.
    pub fn with_enums(
        tables: Vec<TableIR>,
        enums: Vec<EnumIR>,
        strategy: RelationStrategy,
    ) -> Self {
        let relations = match strategy {
            RelationStrategy::Disabled => Vec::new(),
            RelationStrategy::NamingHeuristic => Self::infer_relations_heuristic(&tables),
        };
        SchemaIR {
            tables,
            relations,
            enums,
        }
    }

    /// Validate that all [`EnumRef`] references in table fields resolve to
    /// defined enums, and that no two enums share the same Rust name.
    ///
    /// Call before code generation to catch configuration errors early.
    pub fn validate(&self) -> Result<(), SchemaError> {
        let mut seen = HashSet::new();
        for enm in &self.enums {
            if !seen.insert(enm.rust_name.as_str()) {
                return Err(SchemaError::DuplicateEnum(enm.rust_name.clone()));
            }
        }

        let enum_names: HashSet<&str> =
            self.enums.iter().map(|e| e.rust_name.as_str()).collect();

        for table in &self.tables {
            for field in &table.fields {
                if let DbType::Enum(EnumRef { rust_name }) = &field.ty {
                    if !enum_names.contains(rust_name.as_str()) {
                        return Err(SchemaError::MissingEnum(rust_name.clone()));
                    }
                }
            }
        }

        Ok(())
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

/// Errors detected during [`SchemaIR::validate`].
#[derive(Debug)]
pub enum SchemaError {
    /// A field references an enum that does not exist in [`SchemaIR::enums`].
    MissingEnum(String),
    /// Two or more enums share the same [`rust_name`](EnumIR::rust_name).
    DuplicateEnum(String),
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaError::MissingEnum(name) => {
                write!(f, "Schema references enum \"{name}\" but no matching EnumIR was found")
            }
            SchemaError::DuplicateEnum(name) => {
                write!(f, "Duplicate enum Rust name \"{name}\" — enums must have unique names")
            }
        }
    }
}

impl std::error::Error for SchemaError {}
