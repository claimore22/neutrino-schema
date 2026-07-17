use crate::ir::EnumIR;

/// Generate Rust enum definitions from introspection results.
///
/// Each enum is rendered as a `pub enum` with `#[derive(Debug, Clone, Copy,
/// PartialEq, Eq, Hash, PartialOrd, Ord)]`.
///
/// Returns an empty string when `enums` is empty.
pub fn generate_enum_defs(enums: &[EnumIR]) -> String {
    if enums.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for enm in enums {
        out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\n");
        out.push_str(&format!("pub enum {} {{\n", enm.rust_name));
        for variant in &enm.variants {
            out.push_str(&format!("    {},\n", variant.rust_name));
        }
        out.push_str("}\n\n");
    }
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
        let enums = vec![EnumIR::new(
            "status",
            &["active".into(), "inactive".into(), "pending".into()],
            None,
        )];
        let result = generate_enum_defs(&enums);
        assert!(result.contains("pub enum Status"));
        assert!(result.contains("Active,"));
        assert!(result.contains("Inactive,"));
    }

    #[test]
    fn generate_enum_defs_multiple() {
        let enums = vec![
            EnumIR::new("mood", &["happy".into(), "sad".into()], None),
            EnumIR::new(
                "color",
                &["red".into(), "green".into(), "blue".into()],
                None,
            ),
        ];
        let result = generate_enum_defs(&enums);
        assert!(result.contains("pub enum Mood"));
        assert!(result.contains("pub enum Color"));
        assert!(result.contains("Happy,"));
        assert!(result.contains("Blue,"));
    }

    #[test]
    fn generate_enum_defs_variant_with_hyphens() {
        let enums = vec![EnumIR::new(
            "review_status",
            &["needs-review".into(), "in-progress".into()],
            None,
        )];
        let result = generate_enum_defs(&enums);
        assert!(result.contains("NeedsReview,"));
        assert!(result.contains("InProgress,"));
    }
}
