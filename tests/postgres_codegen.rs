use neutrino_schema::{
    FieldIR, RelationStrategy, RenderMode, SchemaIR, TableIR, generate_struct, types::DbType,
};

#[test]
fn generates_user_struct() {
    let fields = vec![
        FieldIR {
            name: "id".to_string(),
            ty: DbType::Integer,
            nullable: false,
            raw_type: "integer".to_string(),
            comment: None,
        },
        FieldIR {
            name: "email".to_string(),
            ty: DbType::String,
            nullable: false,
            raw_type: "character varying".to_string(),
            comment: None,
        },
        FieldIR {
            name: "bio".to_string(),
            ty: DbType::String,
            nullable: true,
            raw_type: "text".to_string(),
            comment: None,
        },
    ];

    let tables = vec![TableIR {
        name: "users".to_string(),
        fields,
        constraints: vec![],
        comment: None,
        indexes: vec![],
    }];

    let schema = SchemaIR::from_tables(tables, RelationStrategy::Disabled);
    let output = generate_struct(&schema.tables[0], RenderMode::Clean);

    assert!(output.contains("pub struct Users"));
    assert!(output.contains("pub id: i32,"));
    assert!(output.contains("pub email: String,"));
    assert!(output.contains("pub bio: Option<String>,"));
}

#[test]
fn generates_debug_comments() {
    let table = TableIR {
        name: "items".to_string(),
        fields: vec![
            FieldIR {
                name: "id".to_string(),
                ty: DbType::Integer,
                nullable: false,
                raw_type: "integer".to_string(),
                comment: Some("Primary key".into()),
            },
            FieldIR {
                name: "label".to_string(),
                ty: DbType::String,
                nullable: true,
                raw_type: "text".to_string(),
                comment: None,
            },
        ],
        constraints: vec![],
        comment: Some(" Represents an inventory item".into()),
        indexes: vec![],
    };

    let output = generate_struct(&table, RenderMode::Debug);

    // Debug mode comment on id field
    assert!(output.contains("// integer, NOT NULL"));
    // Doc comment on id field
    assert!(output.contains("/// Primary key"));
    // Doc comment on table (leading space trimmed)
    assert!(output.contains("/// Represents an inventory item"));
    // Field with no comment — just the debug comment
    assert!(output.contains("pub label: Option<String>, // text, NULL"));
    // No stray doc comment on label
    assert!(!output.contains("/// ///"), "no double doc-comment prefix");
}

#[test]
fn pascal_case_table_names() {
    use neutrino_schema::to_struct_name;

    assert_eq!(to_struct_name("users"), "Users");
    assert_eq!(to_struct_name("user_profiles"), "UserProfiles");
    assert_eq!(
        to_struct_name("email_verification_tokens"),
        "EmailVerificationTokens"
    );
}
