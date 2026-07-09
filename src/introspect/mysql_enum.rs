/// Minimal parser for MySQL `COLUMN_TYPE` values of the form `enum('a','b','c')`.
///
/// Supports normal cases. Escaped quotes inside variant values
/// (e.g. `enum('it''s ok')`) are handled by a future, more robust parser.
///
/// # Examples
///
/// ```
/// use neutrino_schema::introspect::parse_mysql_enum;
///
/// let variants = parse_mysql_enum("enum('active','inactive','pending')");
/// assert_eq!(variants, Some(vec!["active".into(), "inactive".into(), "pending".into()]));
///
/// assert_eq!(parse_mysql_enum("varchar(255)"), None);
/// assert_eq!(parse_mysql_enum("enum()"), Some(vec![]));
/// ```
pub fn parse_mysql_enum(column_type: &str) -> Option<Vec<String>> {
    let ct = column_type.trim();

    // Must start with "enum("
    let body = ct.strip_prefix("enum(")?.strip_suffix(')')?;
    if body.is_empty() {
        return Some(Vec::new());
    }

    let mut variants = Vec::new();
    let mut chars = body.chars().peekable();
    loop {
        // Skip whitespace before a variant
        while chars.peek().copied() == Some(' ') {
            chars.next();
        }
        // Expect opening quote
        if chars.next() != Some('\'') {
            return None;
        }
        // Read until closing quote (handle basic '' escaping)
        let mut value = String::new();
        loop {
            match chars.next() {
                None => return None,
                Some('\'') => {
                    if chars.peek() == Some(&'\'') {
                        chars.next();
                        value.push('\'');
                    } else {
                        break;
                    }
                }
                Some(c) => value.push(c),
            }
        }
        variants.push(value);
        // Skip whitespace after closing quote
        while chars.peek().copied() == Some(' ') {
            chars.next();
        }
        match chars.next() {
            None => break,
            Some(',') => continue,
            _ => return None,
        }
    }

    Some(variants)
}
