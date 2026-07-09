use crate::util::naming::{enum_variant_name, to_struct_name};

/// A database enum type in the intermediate representation.
///
/// Carries both the original database identity and the resolved Rust
/// identifiers so that codegen has all information it needs without
/// referencing the introspector or database again.
#[derive(Debug, Clone)]
pub struct EnumIR {
    /// The original database enum name (e.g. `"status"`, `"mood"`).
    pub database_name: String,
    /// The PascalCase Rust identifier for the generated enum.
    pub rust_name: String,
    /// The enum variants in ordinal (declaration) order.
    pub variants: Vec<EnumVariantIR>,
    /// The database schema this enum belongs to (e.g. `"public"`).
    /// `None` when the database does not have schema-qualified names (MySQL, SQLite).
    pub schema: Option<String>,
}

/// A single variant of a database enum.
#[derive(Debug, Clone)]
pub struct EnumVariantIR {
    /// The original database value (e.g. `"needs_review"`, `"'it''s ok'"`).
    pub database_name: String,
    /// The PascalCase Rust variant identifier (e.g. `"NeedsReview"`).
    pub rust_name: String,
}

impl EnumIR {
    /// Construct an [`EnumIR`] from a database enum name and its raw variant strings.
    ///
    /// The Rust name is derived automatically via [`to_struct_name`] (so the enum
    /// name follows the same PascalCase convention as table names).
    /// Each variant's Rust identifier is derived via [`enum_variant_name`].
    pub fn new(database_name: &str, variants: &[String], schema: Option<&str>) -> Self {
        let variants = variants
            .iter()
            .map(|v| EnumVariantIR {
                database_name: v.clone(),
                rust_name: enum_variant_name(v),
            })
            .collect();

        EnumIR {
            database_name: database_name.to_string(),
            rust_name: to_struct_name(database_name),
            variants,
            schema: schema.map(String::from),
        }
    }
}
