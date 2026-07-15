use crate::ir::{EnumIR, Metadata, RelationIR, RelationOrigin, RelationStrategy, TableIR};
#[cfg(test)]
use crate::types::DbType;

/// Full database schema — the top-level IR object consumed by code generation.
///
/// `ir_version` tracks the JSON serialization format (not the crate version).
/// `metadata` records the database provider and generation time.
///
/// Construct via [`SchemaIR::from_tables`] (without enums) or
/// [`SchemaIR::with_enums`] for the full schema including enum definitions.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaIR {
    /// SchemaIR JSON format version. Bumped only on breaking JSON structure
    /// changes. Currently `1`.
    pub ir_version: u16,
    /// Provenance and generation metadata.
    pub metadata: Metadata,
    /// All tables discovered or provided.
    pub tables: Vec<TableIR>,
    /// Inferred inter-table relationships (may be empty).
    pub relations: Vec<RelationIR>,
    /// Database enum types (e.g. PostgreSQL `CREATE TYPE`, MySQL `ENUM` columns).
    pub enums: Vec<EnumIR>,
}

/// Current SchemaIR JSON format version.
///
/// Incremented only on **breaking** changes to the JSON structure
/// (field removals, renames, type changes). Additions of optional fields
/// should NOT bump this version.
pub const IR_VERSION: u16 = 1;

impl Default for SchemaIR {
    fn default() -> Self {
        Self {
            ir_version: IR_VERSION,
            metadata: Metadata::default(),
            tables: Vec::new(),
            relations: Vec::new(),
            enums: Vec::new(),
        }
    }
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
            ir_version: IR_VERSION,
            metadata: Metadata::default(),
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
            ir_version: IR_VERSION,
            metadata: Metadata::default(),
            tables,
            relations,
            enums,
        }
    }

    /// Look up a table by name.
    pub fn table(&self, name: &str) -> Option<&TableIR> {
        self.tables.iter().find(|t| t.name == name)
    }

    /// Mutable lookup of a table by name (for IR transforms).
    pub fn table_mut(&mut self, name: &str) -> Option<&mut TableIR> {
        self.tables.iter_mut().find(|t| t.name == name)
    }

    /// Build a [`SchemaIR`] by introspecting a live database.
    ///
    /// Convenience wrapper around [`introspect_schema`](crate::introspect::introspect_schema)
    /// that also populates [`Metadata::provider`] and [`Metadata::database_name`].
    #[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
    pub async fn from_database(
        introspector: &dyn crate::introspect::DatabaseIntrospector,
        table_infos: &[crate::introspect::TableInfo],
        strategy: RelationStrategy,
        provider: Option<crate::config::DatabaseProvider>,
        database_name: Option<String>,
    ) -> anyhow::Result<Self> {
        let mut schema = crate::introspect::introspect_schema(introspector, table_infos, strategy).await?;
        schema.metadata.provider = provider;
        schema.metadata.database_name = database_name;
        Ok(schema)
    }

    // ------------------------------------------------------------------
    // JSON serialization
    // ------------------------------------------------------------------

    /// Serialize to compact (non-pretty) JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from a JSON string.
    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Write JSON (pretty or compact) to any [`std::io::Write`] implementor.
    pub fn write_json_to(
        &self,
        writer: impl std::io::Write,
        pretty: bool,
    ) -> Result<(), serde_json::Error> {
        if pretty {
            serde_json::to_writer_pretty(writer, self)
        } else {
            serde_json::to_writer(writer, self)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FieldIR, TableIR};

    #[test]
    fn json_roundtrip() {
        let table = TableIR {
            name: "users".into(),
            fields: vec![FieldIR {
                name: "id".into(),
                ty: DbType::Integer,
                nullable: false,
                raw_type: "INTEGER".into(),
                default_value: None,
                generated: true,
                comment: None,
            }],
            constraints: vec![],
            comment: None,
            indexes: vec![],
        };
        let schema = SchemaIR::from_tables(vec![table], RelationStrategy::Disabled);

        let json = schema.to_json_pretty().expect("serialize");
        println!("\n=== JSON output ===\n{json}\n====================");

        let schema2 = SchemaIR::from_json_str(&json).expect("deserialize");
        assert_eq!(schema, schema2);
    }

    #[test]
    fn json_compact() {
        let table = TableIR {
            name: "t".into(),
            fields: vec![FieldIR {
                name: "id".into(),
                ty: DbType::BigInt,
                nullable: true,
                raw_type: "BIGINT".into(),
                default_value: None,
                generated: false,
                comment: None,
            }],
            constraints: vec![],
            comment: None,
            indexes: vec![],
        };
        let schema = SchemaIR::from_tables(vec![table], RelationStrategy::Disabled);

        let json = schema.to_json().expect("compact");
        assert!(!json.contains('\n'), "compact JSON should be single-line");
        let deser = SchemaIR::from_json_str(&json).expect("deserialize compact");
        assert_eq!(schema, deser);
    }
}
