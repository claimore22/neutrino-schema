use neutrino_schema::*;

fn simple_schema() -> SchemaIR {
    SchemaIR::from_tables(
        vec![TableIR {
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
        }],
        RelationStrategy::Disabled,
    )
}

#[test]
fn valid_schema_passes_validate() {
    let schema = simple_schema();
    let report = validate(&schema);
    assert!(!report.has_errors());
    assert!(!report.has_warnings());
}

#[test]
fn validate_on_deserialized_schema() {
    let schema = simple_schema();
    let json = schema.to_json().expect("serialize");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize");
    let report = validate(&deser);
    assert!(!report.has_errors());
}

#[test]
fn fk_validates_after_json_roundtrip() {
    let posts = TableIR {
        name: "posts".into(),
        fields: vec![
            FieldIR {
                name: "id".into(),
                ty: DbType::Integer,
                nullable: false,
                raw_type: "INTEGER".into(),
                default_value: None,
                generated: true,
                comment: None,
            },
            FieldIR {
                name: "user_id".into(),
                ty: DbType::Integer,
                nullable: false,
                raw_type: "INTEGER".into(),
                default_value: None,
                generated: false,
                comment: None,
            },
        ],
        constraints: vec![ConstraintIR {
            name: "fk_user".into(),
            kind: ConstraintKind::ForeignKey {
                columns: vec!["user_id".into()],
                referenced_table: "users".into(),
                referenced_columns: vec!["id".into()],
                on_delete: ReferentialAction::Cascade,
                on_update: ReferentialAction::NoAction,
                match_type: None,
            },
        }],
        comment: None,
        indexes: vec![],
    };

    let schema = SchemaIR::from_tables(
        vec![
            TableIR {
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
            },
            posts,
        ],
        RelationStrategy::Disabled,
    );

    let json = schema.to_json().expect("serialize");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize");
    let report = validate(&deser);
    assert!(
        !report.has_errors(),
        "valid FK should pass after JSON roundtrip"
    );
}

#[test]
fn orphan_enum_detected_after_json_roundtrip() {
    let schema = SchemaIR {
        tables: vec![],
        enums: vec![EnumIR {
            database_name: "unused".into(),
            rust_name: "Unused".into(),
            variants: vec![],
            schema: None,
        }],
        ..SchemaIR::default()
    };

    let json = schema.to_json().expect("serialize");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize");
    let report = validate(&deser);
    assert!(!report.has_errors(), "orphan enum should not be an error");
    assert!(report.has_warnings(), "orphan enum should warn");
}

#[test]
fn duplicate_tables_detected_after_json_roundtrip() {
    let schema = SchemaIR {
        tables: vec![
            TableIR {
                name: "users".into(),
                ..simple_schema().tables.into_iter().next().expect("simple_schema has at least one table")
            },
            TableIR {
                name: "users".into(),
                ..simple_schema().tables.into_iter().next().expect("simple_schema has at least one table")
            },
        ],
        ..SchemaIR::default()
    };

    let json = schema.to_json().expect("serialize");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize");
    let report = validate(&deser);
    assert!(report.has_errors(), "duplicate tables should error");
    assert!(report
        .entries
        .iter()
        .any(|e| e.message.contains("duplicate")));
}

#[test]
fn empty_name_detected_after_json_roundtrip() {
    let schema = SchemaIR {
        tables: vec![TableIR {
            name: "".into(),
            fields: vec![],
            constraints: vec![],
            comment: None,
            indexes: vec![],
        }],
        ..SchemaIR::default()
    };

    let json = schema.to_json().expect("serialize");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize");
    let report = validate(&deser);
    assert!(report.has_errors());
}

#[test]
fn missing_enum_ref_detected_after_json_roundtrip() {
    let mut t = simple_schema().tables.into_iter().next().expect("simple_schema has at least one table");
    t.fields.push(FieldIR {
        name: "role".into(),
        ty: DbType::Enum(types::EnumRef {
            rust_name: "NonExistent".into(),
        }),
        nullable: true,
        raw_type: "VARCHAR".into(),
        default_value: None,
        generated: false,
        comment: None,
    });

    let schema = SchemaIR::from_tables(vec![t], RelationStrategy::Disabled);

    let json = schema.to_json().expect("serialize");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize");
    let report = validate(&deser);
    assert!(report.has_errors());
    assert!(report.entries.iter().any(|e| e.message.contains("NonExistent")));
}
