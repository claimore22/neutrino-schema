use std::collections::HashSet;

fn pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut upper = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == '.' {
            upper = true;
        } else if upper {
            out.push(ch.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Convert a `snake_case` table name to `PascalCase` for use as a Rust struct name.
///
/// # Examples
///
/// ```
/// use neutrino_schema::to_struct_name;
/// assert_eq!(to_struct_name("users"), "Users");
/// assert_eq!(to_struct_name("blog_posts"), "BlogPosts");
/// assert_eq!(to_struct_name("user_profile_data"), "UserProfileData");
/// ```
pub fn to_struct_name(table_name: &str) -> String {
    pascal_case(table_name)
}

/// Convert a `snake_case` database value to `PascalCase` for use as a Rust enum variant.
///
/// # Examples
///
/// ```
/// use neutrino_schema::enum_variant_name;
/// assert_eq!(enum_variant_name("active"), "Active");
/// assert_eq!(enum_variant_name("needs_review"), "NeedsReview");
/// ```
pub fn enum_variant_name(input: &str) -> String {
    pascal_case(input)
}

/// Sanitize a database identifier into a valid Rust identifier.
///
/// Rules:
/// - Non-alphanumeric characters (except `_`) are replaced with `_`
/// - Leading digits are stripped
/// - If the result is empty or a Rust keyword, append `_`
///
/// # Examples
///
/// ```
/// use neutrino_schema::sanitize_identifier;
/// assert_eq!(sanitize_identifier("user-profile"), "user_profile");
/// assert_eq!(sanitize_identifier("123name"), "name");
/// assert_eq!(sanitize_identifier("type"), "type_");
/// assert_eq!(sanitize_identifier("first name"), "first_name");
/// ```
pub fn sanitize_identifier(input: &str) -> String {
    let sanitized: String = input
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .skip_while(|c| c.is_ascii_digit())
        .collect();

    if sanitized.is_empty() || is_rust_keyword(&sanitized) {
        format!("{sanitized}_")
    } else {
        sanitized
    }
}

/// Deduplicate identifiers against a set of already-used names.
///
/// If `name` is already in `used`, appends `_2`, `_3`, etc. until unique.
/// Returns the deduplicated name and inserts it into `used`.
pub fn deduplicate_identifier(name: String, used: &mut HashSet<String>) -> String {
    if used.insert(name.clone()) {
        return name;
    }
    let mut counter = 2;
    loop {
        let candidate = format!("{name}_{counter}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        counter += 1;
    }
}

/// Rust 2024 edition reserved keywords.
fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "abstract"
            | "as"
            | "async"
            | "await"
            | "become"
            | "box"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "do"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "final"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "macro"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "override"
            | "priv"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "try"
            | "type"
            | "typeof"
            | "unsafe"
            | "unsized"
            | "use"
            | "virtual"
            | "where"
            | "while"
            | "yield"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_hyphens() {
        assert_eq!(sanitize_identifier("user-profile"), "user_profile");
    }

    #[test]
    fn sanitize_strips_leading_digits() {
        assert_eq!(sanitize_identifier("123name"), "name");
    }

    #[test]
    fn sanitize_appends_underscore_for_keyword() {
        assert_eq!(sanitize_identifier("type"), "type_");
        assert_eq!(sanitize_identifier("self"), "self_");
        assert_eq!(sanitize_identifier("pub"), "pub_");
    }

    #[test]
    fn sanitize_replaces_spaces() {
        assert_eq!(sanitize_identifier("first name"), "first_name");
    }

    #[test]
    fn sanitize_handles_empty() {
        assert_eq!(sanitize_identifier(""), "_");
    }

    #[test]
    fn deduplicate_increments() {
        let mut used = HashSet::new();
        let a = deduplicate_identifier("user_id".into(), &mut used);
        let b = deduplicate_identifier("user_id".into(), &mut used);
        assert_eq!(a, "user_id");
        assert_eq!(b, "user_id_2");
    }
}
