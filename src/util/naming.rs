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
