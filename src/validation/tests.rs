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
