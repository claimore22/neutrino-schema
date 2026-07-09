use crate::config::GeneratorConfig;
use crate::ir::{EnumIR, FieldIR, SchemaIR, TableIR};
use crate::types::{DbType, EnumRef};
use crate::util::naming::to_struct_name;

/// Controls whether generated structs include debug annotations.
///
/// Used by [`generate_struct`] and [`generate_files`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "cli", serde(rename_all = "lowercase"))]
pub enum RenderMode {
    /// Clean output — no comments on fields.
    Clean,
    /// Include `// RawType, NULL/NOT NULL` comments on each field.
    Debug,
}

fn render_field_default(f: &FieldIR, mode: RenderMode) -> String {
    let ty = crate::types::dbtype_to_rust(&f.ty, f.nullable);
    match mode {
        RenderMode::Clean => format!("    pub {}: {},\n", f.name, ty),
        RenderMode::Debug => {
            let null_label = if f.nullable { "NULL" } else { "NOT NULL" };
            format!("    pub {}: {}, // {}, {}\n", f.name, ty, f.raw_type, null_label)
        }
    }
}

fn render_field_with_enum_prefix(f: &FieldIR, mode: RenderMode) -> String {
    let ty = match &f.ty {
        DbType::Enum(EnumRef { rust_name }) => {
            let base = format!("super::enums::{}", rust_name);
            if f.nullable {
                format!("Option<{}>", base)
            } else {
                base
            }
        }
        _ => crate::types::dbtype_to_rust(&f.ty, f.nullable),
    };
    match mode {
        RenderMode::Clean => format!("    pub {}: {},\n", f.name, ty),
        RenderMode::Debug => {
            let null_label = if f.nullable { "NULL" } else { "NOT NULL" };
            format!("    pub {}: {}, // {}, {}\n", f.name, ty, f.raw_type, null_label)
        }
    }
}

/// Render a single [`TableIR`] as a Rust struct definition.
///
/// The struct is `#[derive(Debug, Clone)]` and all fields are `pub`.
/// When `mode` is [`RenderMode::Debug`], each field gets a comment with
/// its raw SQL type and nullability.
///
/// Fields with [`DbType::Enum`] use the bare enum name (no module prefix).
/// For multi-file code generation (including a separate `enums.rs`), use
/// [`generate_files`] instead, which emits `super::enums::` prefixed types.
///
/// # Example
///
/// ```rust
/// use neutrino_schema::*;
///
/// let table = TableIR {
///     name: "users".into(),
///     fields: vec![FieldIR {
///         name: "email".into(),
///         ty: DbType::String,
///         nullable: false,
///         raw_type: "Varchar".into(),
///     }],
/// };
///
/// let out = generate_struct(&table, RenderMode::Clean);
/// assert!(out.contains("pub struct Users"));
/// assert!(out.contains("email: String"));
/// ```
pub fn generate_struct(table: &TableIR, mode: RenderMode) -> String {
    let mut out = String::new();
    let struct_name = to_struct_name(&table.name);

    out.push_str("#[derive(Debug, Clone)]\n");
    out.push_str(&format!("pub struct {struct_name} {{\n"));

    for f in &table.fields {
        out.push_str(&render_field_default(f, mode));
    }

    out.push_str("}\n");
    out
}

/// Generate Rust enum definitions from introspection results.
///
/// Each enum is rendered as a `pub enum` with `#[derive(Debug, Clone, Copy,
/// PartialEq, Eq, Hash, PartialOrd, Ord)]`.
///
/// Returns an empty string when `enums` is empty.
///
/// # Example
///
/// ```rust
/// use neutrino_schema::*;
///
/// let enums = vec![
///     EnumIR::new("mood", &["happy".into(), "sad".into()], None),
/// ];
/// let out = generate_enum_defs(&enums);
/// assert!(out.contains("pub enum Mood"));
/// assert!(out.contains("Happy,"));
/// assert!(out.contains("Sad,"));
/// ```
pub fn generate_enum_defs(enums: &[EnumIR]) -> String {
    if enums.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for enm in enums {
        out.push_str(
            "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\n",
        );
        out.push_str(&format!("pub enum {} {{\n", enm.rust_name));
        for variant in &enm.variants {
            out.push_str(&format!("    {},\n", variant.rust_name));
        }
        out.push_str("}\n\n");
    }
    out
}

/// Write generated Rust files to disk.
///
/// Creates one `.rs` file per table in `config.output_dir`, named after the
/// table (e.g. `users.rs`), plus a `mod.rs` that declares each sub-module.
/// When the schema contains enums, also writes an `enums.rs` file and adds
/// `pub mod enums;` to `mod.rs`.
///
/// Creates the output directory if it does not exist.
///
/// # Errors
///
/// Returns `Err` if the output directory cannot be created, or if any file
/// cannot be written.
pub fn generate_files(schema: &SchemaIR, config: &GeneratorConfig) -> std::io::Result<()> {
    std::fs::create_dir_all(&config.output_dir)?;

    let mut mod_decls: Vec<String> = Vec::new();

    // Write enums.rs if there are any enums
    if !schema.enums.is_empty() {
        let content = generate_enum_defs(&schema.enums);
        std::fs::write(config.output_dir.join("enums.rs"), content)?;
        mod_decls.push("enums".into());
    }

    for table in &schema.tables {
        let file_name = format!("{}.rs", table.name.replace('-', "_"));
        let content = generate_struct_file(table, config.render_mode);
        std::fs::write(config.output_dir.join(&file_name), content)?;
        mod_decls.push(table.name.replace('-', "_"));
    }

    let mut mod_rs = format!("// Generated by neutrino-schema — {} module\n\n", config.module_name);
    for decl in &mod_decls {
        mod_rs.push_str(&format!("pub mod {decl};\n"));
    }
    std::fs::write(config.output_dir.join("mod.rs"), mod_rs)?;

    Ok(())
}

/// Like [`generate_struct`] but emits `super::enums::Name` for enum-typed fields.
fn generate_struct_file(table: &TableIR, mode: RenderMode) -> String {
    let mut out = String::new();
    let struct_name = to_struct_name(&table.name);

    out.push_str("#[derive(Debug, Clone)]\n");
    out.push_str(&format!("pub struct {struct_name} {{\n"));

    for f in &table.fields {
        out.push_str(&render_field_with_enum_prefix(f, mode));
    }

    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_enum_defs_empty() {
        let result = generate_enum_defs(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn generate_enum_defs_single() {
        let enums = vec![EnumIR::new("status", &["active".into(), "inactive".into(), "pending".into()], None)];
        let result = generate_enum_defs(&enums);
        assert!(result.contains("pub enum Status"));
        assert!(result.contains("Active,"));
        assert!(result.contains("Inactive,"));
    }

    #[test]
    fn generate_enum_defs_multiple() {
        let enums = vec![
            EnumIR::new("mood", &["happy".into(), "sad".into()], None),
            EnumIR::new("color", &["red".into(), "green".into(), "blue".into()], None),
        ];
        let result = generate_enum_defs(&enums);
        assert!(result.contains("pub enum Mood"));
        assert!(result.contains("pub enum Color"));
        assert!(result.contains("Happy,"));
        assert!(result.contains("Blue,"));
    }

    #[test]
    fn generate_enum_defs_variant_with_hyphens() {
        let enums = vec![EnumIR::new("review_status", &["needs-review".into(), "in-progress".into()], None)];
        let result = generate_enum_defs(&enums);
        assert!(result.contains("NeedsReview,"));
        assert!(result.contains("InProgress,"));
    }

    #[test]
    fn generate_struct_file_uses_enum_prefix() {
        let enm = EnumIR::new("mood", &["happy".into(), "sad".into()], None);
        let field = FieldIR {
            name: "current_mood".into(),
            ty: DbType::Enum(EnumRef { rust_name: enm.rust_name.clone() }),
            nullable: false,
            raw_type: "mood".into(),
        };
        let table = TableIR {
            name: "users".into(),
            fields: vec![field],
        };
        let result = generate_struct_file(&table, RenderMode::Clean);
        assert!(result.contains("super::enums::Mood"));
        assert!(!result.contains("super::enums::Option"));
    }

    #[test]
    fn generate_struct_file_nullable_enum() {
        let enm = EnumIR::new("mood", &["happy".into(), "sad".into()], None);
        let field = FieldIR {
            name: "current_mood".into(),
            ty: DbType::Enum(EnumRef { rust_name: enm.rust_name.clone() }),
            nullable: true,
            raw_type: "mood".into(),
        };
        let table = TableIR {
            name: "users".into(),
            fields: vec![field],
        };
        let result = generate_struct_file(&table, RenderMode::Clean);
        assert!(result.contains("Option<super::enums::Mood>"));
    }

    #[test]
    fn generate_struct_keeps_bare_enum_name() {
        let enm = EnumIR::new("mood", &["happy".into(), "sad".into()], None);
        let field = FieldIR {
            name: "current_mood".into(),
            ty: DbType::Enum(EnumRef { rust_name: enm.rust_name.clone() }),
            nullable: false,
            raw_type: "mood".into(),
        };
        let table = TableIR {
            name: "users".into(),
            fields: vec![field],
        };
        let result = generate_struct(&table, RenderMode::Clean);
        assert!(result.contains("Mood"));
        assert!(!result.contains("super::enums"));
    }
}
