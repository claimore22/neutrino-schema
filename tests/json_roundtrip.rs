use neutrino_schema::*;

/// Build a moderately complex SchemaIR for roundtrip testing.
fn make_test_schema() -> SchemaIR {
    let fields = vec![
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
            name: "email".into(),
            ty: DbType::String,
            nullable: false,
            raw_type: "VARCHAR(255)".into(),
            default_value: None,
            generated: false,
            comment: Some("user email address".into()),
        },
        FieldIR {
            name: "role".into(),
            ty: DbType::Enum(types::EnumRef {
                rust_name: "UserRole".into(),
            }),
            nullable: true,
            raw_type: "VARCHAR(20)".into(),
            default_value: Some("'user'".into()),
            generated: false,
            comment: None,
        },
        FieldIR {
            name: "created_at".into(),
            ty: DbType::Timestamp,
            nullable: false,
            raw_type: "TIMESTAMP".into(),
            default_value: Some("CURRENT_TIMESTAMP".into()),
            generated: false,
            comment: None,
        },
    ];

    let users = TableIR {
        name: "users".into(),
        fields: fields.clone(),
        constraints: vec![],
        comment: Some("Registered users".into()),
        indexes: vec![],
    };

    let posts = TableIR {
        name: "posts".into(),
        fields: vec![
            FieldIR {
                name: "id".into(),
                ty: DbType::BigInt,
                nullable: false,
                raw_type: "BIGINT".into(),
                default_value: None,
                generated: true,
                comment: None,
            },
            FieldIR {
                name: "author_id".into(),
                ty: DbType::Integer,
                nullable: false,
                raw_type: "INTEGER".into(),
                default_value: None,
                generated: false,
                comment: None,
            },
            FieldIR {
                name: "title".into(),
                ty: DbType::String,
                nullable: false,
                raw_type: "TEXT".into(),
                default_value: None,
                generated: false,
                comment: None,
            },
            FieldIR {
                name: "status".into(),
                ty: DbType::Enum(types::EnumRef {
                    rust_name: "PostStatus".into(),
                }),
                nullable: false,
                raw_type: "post_status".into(),
                default_value: None,
                generated: false,
                comment: None,
            },
        ],
        constraints: vec![ConstraintIR {
            name: "fk_author".into(),
            kind: ConstraintKind::ForeignKey {
                columns: vec!["author_id".into()],
                referenced_table: "users".into(),
                referenced_columns: vec!["id".into()],
                on_delete: ReferentialAction::Cascade,
                on_update: ReferentialAction::NoAction,
                match_type: None,
            },
        }],
        comment: None,
        indexes: vec![IndexIR {
            name: "idx_posts_author".into(),
            table_name: "posts".into(),
            kind: IndexKind::BTree,
            entries: vec![IndexEntryIR::Column {
                name: "author_id".into(),
                descending: false,
            }],
            unique: false,
            predicate: None,
        }],
    };

    let user_role_enum = EnumIR {
        database_name: "user_role".into(),
        rust_name: "UserRole".into(),
        variants: vec![
            EnumVariantIR {
                database_name: "admin".into(),
                rust_name: "Admin".into(),
            },
            EnumVariantIR {
                database_name: "user".into(),
                rust_name: "User".into(),
            },
        ],
        schema: Some("public".into()),
    };

    let post_status_enum = EnumIR {
        database_name: "post_status".into(),
        rust_name: "PostStatus".into(),
        variants: vec![
            EnumVariantIR {
                database_name: "draft".into(),
                rust_name: "Draft".into(),
            },
            EnumVariantIR {
                database_name: "published".into(),
                rust_name: "Published".into(),
            },
            EnumVariantIR {
                database_name: "archived".into(),
                rust_name: "Archived".into(),
            },
        ],
        schema: Some("public".into()),
    };

    SchemaIR::with_enums(
        vec![users, posts],
        vec![user_role_enum, post_status_enum],
        RelationStrategy::NamingHeuristic,
    )
}

#[test]
fn json_roundtrip_pretty() {
    let schema = make_test_schema();
    let json = schema.to_json_pretty().expect("serialize pretty");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize pretty");
    assert_eq!(schema, deser);
}

#[test]
fn json_roundtrip_compact() {
    let schema = make_test_schema();
    let json = schema.to_json().expect("serialize compact");
    assert!(!json.contains('\n'), "compact JSON should be single-line");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize compact");
    assert_eq!(schema, deser);
}

#[test]
fn json_roundtrip_file() {
    let schema = make_test_schema();
    let dir = std::env::temp_dir().join("ns_json_roundtrip");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test_schema.json");

    // Write to file
    let file = std::fs::File::create(&path).expect("create file");
    schema
        .write_json_to(file, true)
        .expect("write to file");

    // Read back
    let text = std::fs::read_to_string(&path).expect("read file");
    let deser = SchemaIR::from_json_str(&text).expect("deserialize from file");
    assert_eq!(schema, deser);

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn json_roundtrip_with_validate() {
    let schema = make_test_schema();
    let json = schema.to_json().expect("serialize");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize");

    // Validate the deserialized schema
    let report = validate(&deser);
    assert!(!report.has_errors(), "deserialized schema should be valid");
    assert!(!report.has_warnings(), "deserialized schema should have no warnings");
}

#[test]
fn json_roundtrip_preserves_metadata() {
    let mut schema = make_test_schema();
    schema.metadata.provider = Some(config::DatabaseProvider::Postgres);
    schema.metadata.database_name = Some("testdb".into());

    let json = schema.to_json().expect("serialize");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize");

    assert_eq!(deser.metadata.provider, Some(config::DatabaseProvider::Postgres));
    assert_eq!(
        deser.metadata.database_name,
        Some("testdb".to_string())
    );
    // generated_at is set at construction time, not in equality check
    assert_eq!(schema.ir_version, deser.ir_version);
}

#[test]
fn json_roundtrip_handles_special_chars() {
    let mut schema = make_test_schema();
    // Add a field with atypical characters
    schema.tables[0].fields.push(FieldIR {
        name: "user's data".into(),
        ty: DbType::Unknown("JSON".into()),
        nullable: true,
        raw_type: "JSONB".into(),
        default_value: None,
        generated: false,
        comment: Some("contains user's info & preferences".into()),
    });

    let json = schema.to_json_pretty().expect("serialize");
    let deser = SchemaIR::from_json_str(&json).expect("deserialize");
    assert_eq!(schema, deser);
}
