use std::collections::BTreeSet;

use crate::ir::{SchemaIR, TableIR};
use crate::types::TypeRegistry;

/// Collect all unique import lines needed for the types used across the schema.
///
/// Returns a sorted, deduplicated list of `use` statements (without trailing newlines).
pub fn generate_imports(schema: &SchemaIR, registry: &TypeRegistry) -> Vec<String> {
    let mut imports = BTreeSet::new();
    for table in &schema.tables {
        for field in &table.fields {
            let rt = registry.resolve(&field.ty);
            for import in &rt.imports {
                imports.insert(import.clone());
            }
        }
    }
    imports.into_iter().collect()
}

/// Collect all unique import lines needed for a single table's fields.
pub fn generate_imports_for_table(table: &TableIR, registry: &TypeRegistry) -> Vec<String> {
    let mut imports = BTreeSet::new();
    for field in &table.fields {
        let rt = registry.resolve(&field.ty);
        for import in &rt.imports {
            imports.insert(import.clone());
        }
    }
    imports.into_iter().collect()
}

/// Build an imports block string for a single table.
pub(super) fn build_imports_block_for_table(table: &TableIR, registry: &TypeRegistry) -> String {
    let imports = generate_imports_for_table(table, registry);
    if imports.is_empty() {
        String::new()
    } else {
        imports.join("\n") + "\n\n"
    }
}
