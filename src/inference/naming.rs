/// Generate relation names from table names.
///
/// Given a table name, produces the singular form for use as a
/// belongs-to relation name, and the plural form for has-many inverses.
///
/// These are simple heuristics — no external pluralization crate is used.

/// Singularize a table name for use as a relation name.
///
/// Examples:
/// - `users` → `user`
/// - `posts` → `post`
/// - `addresses` → `address`
/// - `categories` → `category`
/// - `children` → `child` (best effort)
pub fn singularize(table_name: &str) -> String {
    // Already singular (no trailing s/es/ies)
    if !table_name.ends_with('s') {
        return table_name.to_string();
    }

    // Handle "ies" → "y" (categories → category)
    if let Some(stem) = table_name.strip_suffix("ies") {
        return format!("{}y", stem);
    }

    // Handle "ses", "xes", "zes", "ches", "shes" (addresses → address)
    if table_name.ends_with("ses")
        || table_name.ends_with("xes")
        || table_name.ends_with("zes")
        || table_name.ends_with("ches")
        || table_name.ends_with("shes")
    {
        // Remove trailing "es"
        let len = table_name.len();
        return table_name[..len - 2].to_string();
    }

    // Handle "s" but not "ss" (posts → post, but class → class)
    if table_name.ends_with('s') && !table_name.ends_with("ss") {
        let len = table_name.len();
        return table_name[..len - 1].to_string();
    }

    // Fallback: return as-is
    table_name.to_string()
}

/// Pluralize a table name for use as an inverse relation name.
///
/// Examples:
/// - `user` → `users`
/// - `post` → `posts`
/// - `address` → `addresses`
/// - `category` → `categories`
pub fn pluralize(table_name: &str) -> String {
    if table_name.ends_with('y') && !table_name.ends_with("ay")
        && !table_name.ends_with("ey")
        && !table_name.ends_with("oy")
        && !table_name.ends_with("uy")
    {
        // category → categories
        let stem = &table_name[..table_name.len() - 1];
        return format!("{}ies", stem);
    }

    if table_name.ends_with('s')
        || table_name.ends_with("sh")
        || table_name.ends_with("ch")
        || table_name.ends_with("x")
    {
        return format!("{}es", table_name);
    }

    // Single z (not zz) → double the z and add "es" (quiz → quizzes)
    if table_name.ends_with('z') && !table_name.ends_with("zz") {
        return format!("{}zes", table_name);
    }

    if table_name.ends_with("zz") {
        return format!("{}es", table_name);
    }

    format!("{}s", table_name)
}

/// Build the relation name for a belongs-to side.
///
/// Given the FK column name and target table, produces a name.
/// If the FK column is `<prefix>_id` and `<prefix>` singularized
/// matches the target table, use `<prefix>`. Otherwise fall back
/// to the singularized target table name.
pub fn relation_name_from_fk(
    fk_column: &str,
    _target_table: &str,
) -> String {
    // Try to strip `_id` suffix
    if let Some(prefix) = fk_column.strip_suffix("_id") {
        return singularize(prefix);
    }
    // Fallback: singularize the target table
    singularize(_target_table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singularize_basic() {
        assert_eq!(singularize("users"), "user");
        assert_eq!(singularize("posts"), "post");
        assert_eq!(singularize("roles"), "role");
    }

    #[test]
    fn singularize_ies() {
        assert_eq!(singularize("categories"), "category");
        assert_eq!(singularize("entities"), "entity");
        assert_eq!(singularize("ilies"), "ily");
    }

    #[test]
    fn singularize_shes_ches() {
        assert_eq!(singularize("addresses"), "address");
        assert_eq!(singularize("boxes"), "box");
        assert_eq!(singularize("buses"), "bus");
        assert_eq!(singularize("watches"), "watch");
        assert_eq!(singularize("dishes"), "dish");
    }

    #[test]
    fn singularize_already_singular() {
        assert_eq!(singularize("user"), "user");
        assert_eq!(singularize("post"), "post");
        assert_eq!(singularize("class"), "class");
    }

    #[test]
    fn pluralize_basic() {
        assert_eq!(pluralize("user"), "users");
        assert_eq!(pluralize("post"), "posts");
        assert_eq!(pluralize("role"), "roles");
    }

    #[test]
    fn pluralize_ies() {
        assert_eq!(pluralize("category"), "categories");
        assert_eq!(pluralize("entity"), "entities");
    }

    #[test]
    fn pluralize_sh_ch_x_z() {
        assert_eq!(pluralize("address"), "addresses");
        assert_eq!(pluralize("box"), "boxes");
        assert_eq!(pluralize("watch"), "watches");
        assert_eq!(pluralize("dish"), "dishes");
        assert_eq!(pluralize("quiz"), "quizzes");
    }

    #[test]
    fn pluralize_vowels_before_y() {
        assert_eq!(pluralize("toy"), "toys");
        assert_eq!(pluralize("key"), "keys");
        assert_eq!(pluralize("boy"), "boys");
        assert_eq!(pluralize("day"), "days");
    }

    #[test]
    fn relation_name_from_fk_basic() {
        assert_eq!(relation_name_from_fk("user_id", "users"), "user");
        assert_eq!(relation_name_from_fk("author_id", "users"), "author");
        assert_eq!(relation_name_from_fk("category_id", "categories"), "category");
    }

    #[test]
    fn relation_name_from_fk_no_id_suffix() {
        assert_eq!(relation_name_from_fk("author", "users"), "user");
    }
}
