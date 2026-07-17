use crate::ir::{ConstraintKind, FieldIR, TableIR};
use crate::types::{DbType, EnumRef};
use crate::GenerateOptions;

use super::resolver::RustTypeResolver;

/// Render `value` as a multi-line `///` doc comment.
fn render_doc_comment(value: &str) -> String {
    value
        .lines()
        .map(|line| format!("    /// {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render a single field with bare enum names (no module prefix).
pub(super) fn render_field_default(
    f: &FieldIR,
    options: &GenerateOptions,
    resolver: &RustTypeResolver,
) -> String {
    let mut out = String::new();
    if let Some(comment) = &f.comment {
        let trimmed = comment.trim();
        if !trimmed.is_empty() {
            out.push_str(&render_doc_comment(trimmed));
            out.push('\n');
        }
    }
    let sanitized = crate::sanitize_identifier(&f.name);
    let ty = resolver.resolve(&f.ty, f.nullable);
    match options.render_mode {
        crate::RenderMode::Clean => out + &format!("    pub {sanitized}: {ty},\n"),
        crate::RenderMode::Debug => {
            let null_label = if f.nullable { "NULL" } else { "NOT NULL" };
            out + &format!(
                "    pub {sanitized}: {ty}, // {}, {null_label}\n",
                f.raw_type
            )
        }
    }
}

/// Render a single field with `super::enums::Name` for enum-typed fields.
pub(super) fn render_field_with_enum_prefix(
    f: &FieldIR,
    options: &GenerateOptions,
    resolver: &RustTypeResolver,
) -> String {
    let mut out = String::new();
    if let Some(comment) = &f.comment {
        let trimmed = comment.trim();
        if !trimmed.is_empty() {
            out.push_str(&render_doc_comment(trimmed));
            out.push('\n');
        }
    }
    let sanitized = crate::sanitize_identifier(&f.name);
    let ty = match &f.ty {
        DbType::Enum(EnumRef { rust_name }) => {
            let base = format!("super::enums::{}", rust_name);
            if f.nullable {
                format!("Option<{base}>")
            } else {
                base
            }
        }
        _ => resolver.resolve(&f.ty, f.nullable),
    };
    match options.render_mode {
        crate::RenderMode::Clean => out + &format!("    pub {sanitized}: {ty},\n"),
        crate::RenderMode::Debug => {
            let null_label = if f.nullable { "NULL" } else { "NOT NULL" };
            out + &format!(
                "    pub {sanitized}: {ty}, // {}, {null_label}\n",
                f.raw_type
            )
        }
    }
}

/// Render a `pub const` holding the primary-key column names for a table.
pub(super) fn render_primary_key_metadata(table: &TableIR) -> String {
    let Some(pk) = table.primary_key() else {
        return String::new();
    };
    let ConstraintKind::PrimaryKey { columns } = &pk.kind else {
        return String::new();
    };
    let const_name = format!("{}_PRIMARY_KEY", table.name.to_uppercase());
    let cols: Vec<String> = columns.iter().map(|c| format!("\"{c}\"")).collect();
    let cols_joined = cols.join(", ");
    format!("\npub const {const_name}: &[&str] = &[{cols_joined}];\n")
}

/// Generate a single table struct definition (with `super::enums::` prefix).
pub(super) fn generate_table(
    table: &TableIR,
    options: &GenerateOptions,
    resolver: &RustTypeResolver,
) -> String {
    let mut out = String::new();
    let struct_name = crate::to_struct_name(&table.name);

    if let Some(comment) = &table.comment {
        let trimmed = comment.trim();
        if !trimmed.is_empty() {
            for line in trimmed.lines() {
                out.push_str(&format!("/// {}\n", line));
            }
        }
    }

    let extra_derive = if options.rust.derive_from_row {
        ", sqlx::FromRow"
    } else {
        ""
    };
    out.push_str(&format!("#[derive(Debug, Clone{extra_derive})]\n"));
    out.push_str(&format!("pub struct {struct_name} {{\n"));

    for f in &table.fields {
        out.push_str(&render_field_with_enum_prefix(f, options, resolver));
    }

    out.push_str("}\n");
    out.push_str(&render_primary_key_metadata(table));
    out
}

/// Generate a single table struct definition (bare enum names, no module prefix).
///
/// This is a convenience wrapper used by [`generate_struct`].
pub(super) fn generate_table_bare(
    table: &TableIR,
    options: &GenerateOptions,
    resolver: &RustTypeResolver,
) -> String {
    let mut out = String::new();
    let struct_name = crate::to_struct_name(&table.name);

    if let Some(comment) = &table.comment {
        let trimmed = comment.trim();
        if !trimmed.is_empty() {
            for line in trimmed.lines() {
                out.push_str(&format!("/// {}\n", line));
            }
        }
    }

    let extra_derive = if options.rust.derive_from_row {
        ", sqlx::FromRow"
    } else {
        ""
    };
    out.push_str(&format!("#[derive(Debug, Clone{extra_derive})]\n"));
    out.push_str(&format!("pub struct {struct_name} {{\n"));

    for f in &table.fields {
        out.push_str(&render_field_default(f, options, resolver));
    }

    out.push_str("}\n");
    out.push_str(&render_primary_key_metadata(table));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ConstraintIR, FieldIR};
    use crate::types::TypeRegistry;

    fn setup() -> (TypeRegistry, RustTypeResolver) {
        let reg = TypeRegistry::default();
        let r = RustTypeResolver::new(reg.clone());
        (reg, r)
    }

    fn default_opts() -> GenerateOptions {
        GenerateOptions::default()
    }

    #[test]
    fn pk_metadata_single_column() {
        let (_, resolver) = setup();
        let table = TableIR {
            name: "users".into(),
            fields: vec![FieldIR {
                name: "id".into(),
                ty: DbType::Integer,
                nullable: false,
                raw_type: "INTEGER".into(),
                default_value: None,
                generated: false,
                comment: None,
            }],
            constraints: vec![ConstraintIR {
                name: "users_pkey".into(),
                kind: ConstraintKind::PrimaryKey {
                    columns: vec!["id".into()],
                },
            }],
            comment: None,
            indexes: vec![],
        };
        let out = generate_table_bare(&table, &default_opts(), &resolver);
        assert!(out.contains("pub const USERS_PRIMARY_KEY: &[&str] = &[\"id\"];"));
    }

    #[test]
    fn pk_metadata_composite() {
        let (_, resolver) = setup();
        let table = TableIR {
            name: "post_tags".into(),
            fields: vec![
                FieldIR {
                    name: "post_id".into(),
                    ty: DbType::Integer,
                    nullable: false,
                    raw_type: "INTEGER".into(),
                    default_value: None,
                    generated: false,
                    comment: None,
                },
                FieldIR {
                    name: "tag_id".into(),
                    ty: DbType::Integer,
                    nullable: false,
                    raw_type: "INTEGER".into(),
                    default_value: None,
                    generated: false,
                    comment: None,
                },
            ],
            constraints: vec![ConstraintIR {
                name: "post_tags_pkey".into(),
                kind: ConstraintKind::PrimaryKey {
                    columns: vec!["post_id".into(), "tag_id".into()],
                },
            }],
            comment: None,
            indexes: vec![],
        };
        let out = generate_table_bare(&table, &default_opts(), &resolver);
        assert!(
            out.contains("pub const POST_TAGS_PRIMARY_KEY: &[&str] = &[\"post_id\", \"tag_id\"];")
        );
    }

    #[test]
    fn pk_metadata_none_when_no_constraint() {
        let (_, resolver) = setup();
        let table = TableIR {
            name: "users".into(),
            fields: vec![],
            constraints: vec![],
            comment: None,
            indexes: vec![],
        };
        let out = generate_table_bare(&table, &default_opts(), &resolver);
        assert!(!out.contains("PRIMARY_KEY"));
    }

    #[test]
    fn generate_struct_file_uses_enum_prefix() {
        let (_, resolver) = setup();
        let enm = crate::ir::EnumIR::new("mood", &["happy".into(), "sad".into()], None);
        let field = FieldIR {
            name: "current_mood".into(),
            ty: DbType::Enum(EnumRef {
                rust_name: enm.rust_name.clone(),
            }),
            nullable: false,
            raw_type: "mood".into(),
            default_value: None,
            generated: false,
            comment: None,
        };
        let table = TableIR {
            name: "users".into(),
            fields: vec![field],
            constraints: vec![],
            comment: None,
            indexes: vec![],
        };
        let out = generate_table(&table, &default_opts(), &resolver);
        assert!(out.contains("super::enums::Mood"));
        assert!(!out.contains("super::enums::Option"));
    }

    #[test]
    fn generate_struct_file_nullable_enum() {
        let (_, resolver) = setup();
        let enm = crate::ir::EnumIR::new("mood", &["happy".into(), "sad".into()], None);
        let field = FieldIR {
            name: "current_mood".into(),
            ty: DbType::Enum(EnumRef {
                rust_name: enm.rust_name.clone(),
            }),
            nullable: true,
            raw_type: "mood".into(),
            default_value: None,
            generated: false,
            comment: None,
        };
        let table = TableIR {
            name: "users".into(),
            fields: vec![field],
            constraints: vec![],
            comment: Some(" Users table".into()),
            indexes: vec![],
        };
        let out = generate_table(&table, &default_opts(), &resolver);
        assert!(out.contains("Option<super::enums::Mood>"));
    }

    #[test]
    fn generate_struct_keeps_bare_enum_name() {
        let (_, resolver) = setup();
        let enm = crate::ir::EnumIR::new("mood", &["happy".into(), "sad".into()], None);
        let field = FieldIR {
            name: "current_mood".into(),
            ty: DbType::Enum(EnumRef {
                rust_name: enm.rust_name.clone(),
            }),
            nullable: false,
            raw_type: "mood".into(),
            default_value: None,
            generated: false,
            comment: None,
        };
        let table = TableIR {
            name: "users".into(),
            fields: vec![field],
            constraints: vec![],
            comment: None,
            indexes: vec![],
        };
        let out = generate_table_bare(&table, &default_opts(), &resolver);
        assert!(out.contains("Mood"));
        assert!(!out.contains("super::enums"));
    }

    #[test]
    fn type_registry_overrides_are_respected() {
        use std::collections::HashMap;
        let mut overrides = HashMap::new();
        overrides.insert("Uuid".into(), "MyUuid".into());
        let registry = TypeRegistry::with_overrides(overrides);
        let resolver = RustTypeResolver::new(registry);

        let result = resolver.resolve(&DbType::Uuid, false);
        assert_eq!(result, "MyUuid");
    }

    #[test]
    fn type_registry_override_with_nullable() {
        use std::collections::HashMap;
        let mut overrides = HashMap::new();
        overrides.insert("Uuid".into(), "MyUuid".into());
        let registry = TypeRegistry::with_overrides(overrides);
        let resolver = RustTypeResolver::new(registry);

        let result = resolver.resolve(&DbType::Uuid, true);
        assert_eq!(result, "Option<MyUuid>");
    }
}
