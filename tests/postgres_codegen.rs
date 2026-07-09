use neutrino_schema::{
    generate_struct, FieldIR, RelationStrategy, RenderMode, SchemaIR, TableIR,
    types::DbType,
};

#[test]
fn generates_user_struct() {
    let fields = vec![
        FieldIR {
            name: "id".to_string(),
            ty: DbType::Int,
            nullable: false,
            raw_type: "integer".to_string(),
        },
        FieldIR {
            name: "email".to_string(),
            ty: DbType::String,
            nullable: false,
            raw_type: "character varying".to_string(),
        },
        FieldIR {
            name: "bio".to_string(),
            ty: DbType::String,
            nullable: true,
            raw_type: "text".to_string(),
        },
    ];

    let tables = vec![TableIR {
        name: "users".to_string(),
        fields,
    }];

    let schema = SchemaIR::from_tables(tables, RelationStrategy::Disabled);
    let output = generate_struct(&schema.tables[0], RenderMode::Clean);

    assert!(output.contains("pub struct Users"));
    assert!(output.contains("pub id: i64,"));
    assert!(output.contains("pub email: String,"));
    assert!(output.contains("pub bio: Option<String>,"));
}

#[test]
fn generates_debug_comments() {
    let fields = vec![FieldIR {
        name: "id".to_string(),
        ty: DbType::Int,
        nullable: false,
        raw_type: "integer".to_string(),
    }];

    let tables = vec![TableIR {
        name: "items".to_string(),
        fields,
    }];

    let output = generate_struct(&tables[0], RenderMode::Debug);

    assert!(output.contains("// integer, NOT NULL"));
}

#[test]
fn pascal_case_table_names() {
    use neutrino_schema::to_struct_name;

    assert_eq!(to_struct_name("users"), "Users");
    assert_eq!(to_struct_name("user_profiles"), "UserProfiles");
    assert_eq!(to_struct_name("email_verification_tokens"), "EmailVerificationTokens");
}
