use std::collections::HashSet;

use crate::ir::{EnumIR, RelationIR, RelationOrigin, RelationStrategy, TableIR};
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
        let fk_relations = Self::derive_fk_relations(&tables);
        let mut relations = fk_relations.clone();
        if strategy == RelationStrategy::NamingHeuristic {
            relations.extend(Self::infer_relations_heuristic(&tables, &fk_relations));
        }
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
        let fk_relations = Self::derive_fk_relations(&tables);
        let mut relations = fk_relations.clone();
        if strategy == RelationStrategy::NamingHeuristic {
            relations.extend(Self::infer_relations_heuristic(&tables, &fk_relations));
        }
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

        let enum_names: HashSet<&str> = self.enums.iter().map(|e| e.rust_name.as_str()).collect();

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

    /// Look up a table by name.
    pub fn table(&self, name: &str) -> Option<&TableIR> {
        self.tables.iter().find(|t| t.name == name)
    }

    /// Mutable lookup of a table by name (for IR transforms).
    pub fn table_mut(&mut self, name: &str) -> Option<&mut TableIR> {
        self.tables.iter_mut().find(|t| t.name == name)
    }

    fn table_exists(tables: &[TableIR], name: &str) -> bool {
        tables.iter().any(|t| t.name == name)
    }

    fn derive_fk_relations(tables: &[TableIR]) -> Vec<RelationIR> {
        tables
            .iter()
            .flat_map(|table| {
                table
                    .constraints
                    .iter()
                    .flat_map(|c| c.fk_relations(&table.name))
            })
            .collect()
    }

    /// Try to infer the singular table name from a known table in `tables`.
    /// Returns `None` if no match is found after trying all plural strategies.
    fn resolve_plural(tables: &[TableIR], prefix: &str) -> Option<String> {
        if Self::table_exists(tables, prefix) {
            return Some(prefix.to_string());
        }
        // Append "s"
        let plural_s = format!("{}s", prefix);
        if Self::table_exists(tables, &plural_s) {
            return Some(plural_s);
        }
        // Append "es"
        let plural_es = format!("{}es", prefix);
        if Self::table_exists(tables, &plural_es) {
            return Some(plural_es);
        }
        // Replace trailing "y" with "ies"
        if let Some(stem) = prefix.strip_suffix('y') {
            let plural_ies = format!("{}ies", stem);
            if Self::table_exists(tables, &plural_ies) {
                return Some(plural_ies);
            }
        }
        None
    }

    /// Return the primary-key column name of a table, or `"id"` as fallback.
    fn pk_column(tables: &[TableIR], table_name: &str) -> Vec<String> {
        if let Some(tbl) = tables.iter().find(|t| t.name == table_name) {
            if let Some(pk) = tbl.primary_key() {
                if let crate::ir::ConstraintKind::PrimaryKey { columns } = &pk.kind {
                    if columns.len() == 1 {
                        return columns.clone();
                    }
                }
            }
        }
        vec!["id".to_string()]
    }

    fn infer_relations_heuristic(tables: &[TableIR], existing: &[RelationIR]) -> Vec<RelationIR> {
        let mut relations = Vec::new();

        for table in tables {
            for field in &table.fields {
                // Skip fields already covered by an FK constraint — FK metadata is authoritative
                if existing
                    .iter()
                    .any(|r| r.from_table == table.name && r.from_columns.contains(&field.name))
                {
                    continue;
                }

                let Some(prefix) = field.name.strip_suffix("_id") else {
                    continue;
                };

                let Some(to_table) = Self::resolve_plural(tables, prefix) else {
                    continue;
                };

                let to_columns = Self::pk_column(tables, &to_table);

                relations.push(RelationIR {
                    from_table: table.name.clone(),
                    from_columns: vec![field.name.clone()],
                    to_table,
                    to_columns,
                    origin: RelationOrigin::Inferred,
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
                write!(
                    f,
                    "Schema references enum \"{name}\" but no matching EnumIR was found"
                )
            }
            SchemaError::DuplicateEnum(name) => {
                write!(
                    f,
                    "Duplicate enum Rust name \"{name}\" — enums must have unique names"
                )
            }
        }
    }
}

impl std::error::Error for SchemaError {}
