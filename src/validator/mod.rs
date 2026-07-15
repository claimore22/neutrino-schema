//! Semantic validation for [`SchemaIR`] documents.
//!
//! Validates an IR against consistency rules (duplicate tables, missing
//! references, orphan enums, etc.) and produces a [`ValidationReport`]
//! containing errors and warnings.
//!
//! This is a standalone operation over a fully constructed [`SchemaIR`];
//! validation does not modify the IR or depend on the database.

use std::collections::{HashMap, HashSet};

use crate::ir::SchemaIR;
use crate::types::DbType;

/// Result of validating a [`SchemaIR`] document.
///
/// Reports all detected issues as [`ValidationEntry`] items, each tagged
/// with a severity [`level`](ValidationEntry::level).  Consumers should
/// check [`has_errors`](ValidationReport::has_errors) before proceeding
/// with code generation or export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// All diagnostic entries found during validation.
    pub entries: Vec<ValidationEntry>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl ValidationReport {
    /// Returns `true` if at least one entry has [`ValidationLevel::Error`].
    pub fn has_errors(&self) -> bool {
        self.entries
            .iter()
            .any(|e| matches!(e.level, ValidationLevel::Error))
    }

    /// Returns an iterator over all error-level entries.
    pub fn errors(&self) -> impl Iterator<Item = &ValidationEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.level, ValidationLevel::Error))
    }

    /// Returns `true` if at least one entry has [`ValidationLevel::Warning`].
    pub fn has_warnings(&self) -> bool {
        self.entries
            .iter()
            .any(|e| matches!(e.level, ValidationLevel::Warning))
    }

    /// Returns an iterator over all warning-level entries.
    pub fn warnings(&self) -> impl Iterator<Item = &ValidationEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.level, ValidationLevel::Warning))
    }

    fn error(&mut self, message: impl Into<String>, location: Option<String>) {
        self.entries.push(ValidationEntry {
            level: ValidationLevel::Error,
            message: message.into(),
            location,
        });
    }

    fn warning(&mut self, message: impl Into<String>, location: Option<String>) {
        self.entries.push(ValidationEntry {
            level: ValidationLevel::Warning,
            message: message.into(),
            location,
        });
    }
}

/// A single diagnostic entry produced by validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationEntry {
    /// Severity level.
    pub level: ValidationLevel,
    /// Human-readable description of the issue.
    pub message: String,
    /// Optional location string (e.g. `"table 'users'"`, `"table 'users'.field 'email'"`).
    pub location: Option<String>,
}

/// Severity level for a validation entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationLevel {
    /// Document cannot be used for code generation or export.
    Error,
    /// Potential issue but does not block code generation or export.
    Warning,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Validate a [`SchemaIR`] document against consistency rules.
///
/// Returns a [`ValidationReport`] containing all detected errors and warnings.
/// The report is always valid — an empty [`entries`](ValidationReport::entries)
/// list means the document passed all checks.
pub fn validate(schema: &SchemaIR) -> ValidationReport {
    let mut report = ValidationReport::default();

    // ------------------------------------------------------------------
    // Pass 1: collect symbols
    // ------------------------------------------------------------------
    let mut table_names: HashMap<String, usize> = HashMap::new();
    let mut enum_names: HashMap<String, usize> = HashMap::new();
    let mut used_enums: HashSet<String> = HashSet::new();

    for (i, table) in schema.tables.iter().enumerate() {
        let name = table.name.trim().to_string();
        if name.is_empty() {
            report.error("table name is empty", Some(format!("tables[{}]", i)));
        } else if let Some(prev) = table_names.insert(name.clone(), i) {
            report.error(
                format!("duplicate table name '{}'", name),
                Some(format!("tables[{}] (already defined at tables[{}])", i, prev)),
            );
        }

        for (j, field) in table.fields.iter().enumerate() {
            let field_name = field.name.trim().to_string();
            if field_name.is_empty() {
                report.error("field name is empty", Some(format!("table '{}'.fields[{}]", table.name, j)));
            }
            collect_enum_refs(&field.ty, &mut used_enums);
        }
    }

    for (i, enm) in schema.enums.iter().enumerate() {
        let db_name = enm.database_name.trim().to_string();
        if db_name.is_empty() {
            report.error("enum database_name is empty", Some(format!("enums[{}]", i)));
        }
        let rust_name = enm.rust_name.trim().to_string();
        if rust_name.is_empty() {
            report.error("enum rust_name is empty", Some(format!("enums[{}]", i)));
        } else if let Some(prev) = enum_names.insert(rust_name.clone(), i) {
            report.error(
                format!("duplicate enum rust_name '{}'", rust_name),
                Some(format!("enums[{}] (already defined at enums[{}])", i, prev)),
            );
        }
    }

    // ------------------------------------------------------------------
    // Pass 2: validate references
    // ------------------------------------------------------------------

    // 4. Unresolved EnumRef
    for rust_name in &used_enums {
        if !enum_names.contains_key(rust_name) {
            report.error(
                format!("unresolved enum reference '{}'", rust_name),
                None,
            );
        }
    }

    // 5. FK references missing table
    for table in &schema.tables {
        for constraint in &table.constraints {
            if let crate::ir::ConstraintKind::ForeignKey {
                referenced_table, ..
            } = &constraint.kind
            {
                if !table_names.contains_key(referenced_table.trim()) {
                    report.error(
                        format!(
                            "foreign key references non-existent table '{}'",
                            referenced_table
                        ),
                        Some(format!("table '{}'", table.name)),
                    );
                }
            }
        }
    }

    // 6. Orphan enums
    for rust_name in enum_names.keys() {
        if !used_enums.contains(rust_name.as_str()) {
            report.warning(
                format!("enum '{}' is defined but never used", rust_name),
                None,
            );
        }
    }

    report
}

/// Recursively collect all [`DbType::Enum`] reference names from a type.
fn collect_enum_refs(db_type: &DbType, used: &mut HashSet<String>) {
    match db_type {
        DbType::Enum(enm) => {
            used.insert(enm.rust_name.trim().to_string());
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;
    use crate::types::{DbType, EnumRef};

    fn table(name: &str) -> TableIR {
        TableIR {
            name: name.to_string(),
            fields: vec![],
            constraints: vec![],
            comment: None,
            indexes: vec![],
        }
    }

    fn enum_ir(db_name: &str, rust_name: &str) -> EnumIR {
        EnumIR {
            database_name: db_name.to_string(),
            rust_name: rust_name.to_string(),
            variants: vec![],
            schema: None,
        }
    }

    fn field_with_enum(name: &str, enum_rust_name: &str) -> FieldIR {
        FieldIR {
            name: name.to_string(),
            ty: DbType::Enum(EnumRef {
                rust_name: enum_rust_name.to_string(),
            }),
            nullable: false,
            raw_type: String::new(),
            default_value: None,
            generated: false,
            comment: None,
        }
    }

    fn field_with_type(name: &str, ty: DbType) -> FieldIR {
        FieldIR {
            name: name.to_string(),
            ty,
            nullable: false,
            raw_type: String::new(),
            default_value: None,
            generated: false,
            comment: None,
        }
    }

    #[test]
    fn valid_schema_has_no_issues() {
        let schema = SchemaIR::from_tables(vec![table("users")], RelationStrategy::Disabled);
        let report = validate(&schema);
        assert!(report.entries.is_empty());
        assert!(!report.has_errors());
        assert!(!report.has_warnings());
    }

    #[test]
    fn empty_table_name_is_error() {
        let schema = SchemaIR::from_tables(vec![table("")], RelationStrategy::Disabled);
        let report = validate(&schema);
        assert!(report.has_errors());
        assert!(report.entries.iter().any(|e| e.message.contains("empty")));
    }

    #[test]
    fn whitespace_table_name_is_error() {
        let schema = SchemaIR::from_tables(vec![table("   ")], RelationStrategy::Disabled);
        let report = validate(&schema);
        assert!(report.has_errors());
        assert!(report.entries.iter().any(|e| e.message.contains("empty")));
    }

    #[test]
    fn empty_field_name_is_error() {
        let mut t = table("users");
        t.fields.push(field_with_type("", DbType::Integer));
        let schema = SchemaIR::from_tables(vec![t], RelationStrategy::Disabled);
        let report = validate(&schema);
        assert!(report.has_errors());
        assert!(report.entries.iter().any(|e| e.message.contains("empty")));
    }

    #[test]
    fn duplicate_table_names_is_error() {
        let schema = SchemaIR {
            tables: vec![table("users"), table("users")],
            ..SchemaIR::default()
        };
        let report = validate(&schema);
        assert!(report.has_errors());
        assert!(report.entries.iter().any(|e| e.message.contains("duplicate")));
    }

    #[test]
    fn duplicate_enum_names_is_error() {
        let schema = SchemaIR {
            enums: vec![enum_ir("status", "Status"), enum_ir("old_status", "Status")],
            ..SchemaIR::default()
        };
        let report = validate(&schema);
        assert!(report.has_errors());
        assert!(report
            .entries
            .iter()
            .any(|e| e.message.contains("duplicate")));
    }

    #[test]
    fn missing_enum_ref_is_error() {
        let mut t = table("users");
        t.fields
            .push(field_with_enum("role", "UserRole"));
        let schema = SchemaIR::from_tables(vec![t], RelationStrategy::Disabled);
        let report = validate(&schema);
        assert!(report.has_errors());
        assert!(report
            .entries
            .iter()
            .any(|e| e.message.contains("unresolved")));
    }

    #[test]
    fn fk_missing_target_table_is_error() {
        let mut t = table("posts");
        t.constraints.push(ConstraintIR {
            name: "fk_category".into(),
            kind: ConstraintKind::ForeignKey {
                columns: vec!["category_id".into()],
                referenced_table: "categories".into(),
                referenced_columns: vec!["id".into()],
                on_delete: ReferentialAction::NoAction,
                on_update: ReferentialAction::NoAction,
                match_type: None,
            },
        });
        let schema = SchemaIR::from_tables(vec![t], RelationStrategy::Disabled);
        let report = validate(&schema);
        assert!(report.has_errors());
        assert!(report
            .entries
            .iter()
            .any(|e| e.message.contains("non-existent")));
    }

    #[test]
    fn orphan_enum_is_warning() {
        let schema = SchemaIR {
            enums: vec![enum_ir("mood", "Mood")],
            ..SchemaIR::default()
        };
        let report = validate(&schema);
        assert!(!report.has_errors());
        assert!(report.has_warnings());
        assert!(report.entries.iter().any(|e| e.message.contains("never used")));
    }

    #[test]
    fn warning_does_not_make_schema_invalid() {
        let schema = SchemaIR {
            enums: vec![enum_ir("unused", "Unused")],
            ..SchemaIR::default()
        };
        let report = validate(&schema);
        assert!(!report.has_errors());
        assert!(report.has_warnings());
    }

    #[test]
    fn valid_enum_ref_passes() {
        let mut t = table("users");
        t.fields
            .push(field_with_enum("status", "Status"));
        let schema = SchemaIR {
            tables: vec![t],
            enums: vec![enum_ir("status", "Status")],
            ..SchemaIR::default()
        };
        let report = validate(&schema);
        assert!(report.entries.is_empty());
    }

    #[test]
    fn empty_enum_database_name_is_error() {
        let schema = SchemaIR {
            enums: vec![enum_ir("", "Status")],
            ..SchemaIR::default()
        };
        let report = validate(&schema);
        assert!(report.has_errors());
        assert!(report
            .entries
            .iter()
            .any(|e| e.message.contains("database_name")));
    }

    #[test]
    fn empty_enum_rust_name_is_error() {
        let schema = SchemaIR {
            enums: vec![enum_ir("status", "")],
            ..SchemaIR::default()
        };
        let report = validate(&schema);
        assert!(report.has_errors());
        assert!(report
            .entries
            .iter()
            .any(|e| e.message.contains("rust_name")));
    }
}
