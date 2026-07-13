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
    let ct = column_type.trim().to_lowercase();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_normal_enum() {
        let result = parse_mysql_enum("enum('a','b','c')");
        assert_eq!(result, Some(vec!["a".into(), "b".into(), "c".into()]));
    }

    #[test]
    fn parses_single_variant() {
        let result = parse_mysql_enum("enum('only')");
        assert_eq!(result, Some(vec!["only".into()]));
    }

    #[test]
    fn returns_none_for_non_enum() {
        assert_eq!(parse_mysql_enum("varchar(255)"), None);
        assert_eq!(parse_mysql_enum("text"), None);
        assert_eq!(parse_mysql_enum(""), None);
    }

    #[test]
    fn handles_whitespace() {
        let result = parse_mysql_enum("enum( 'x' , 'y' )");
        assert_eq!(result, Some(vec!["x".into(), "y".into()]));
    }

    #[test]
    fn handles_escaped_quotes() {
        let result = parse_mysql_enum("enum('it''s ok','normal')");
        assert_eq!(result, Some(vec!["it's ok".into(), "normal".into()]));
    }

    #[test]
    fn rejects_unclosed_quote() {
        assert_eq!(parse_mysql_enum("enum('unclosed"), None);
    }

    #[test]
    fn rejects_missing_opening_quote() {
        assert_eq!(parse_mysql_enum("enum(x)"), None);
    }

    #[test]
    fn trims_input() {
        let result = parse_mysql_enum("  enum('a','b')  ");
        assert_eq!(result, Some(vec!["a".into(), "b".into()]));
    }

    #[test]
    fn handles_uppercase_enum() {
        let result = parse_mysql_enum("ENUM('x','y')");
        assert_eq!(result, Some(vec!["x".into(), "y".into()]));
    }

    #[test]
    fn handles_hyphenated_values() {
        let result = parse_mysql_enum("enum('needs-review','in-progress')");
        assert_eq!(
            result,
            Some(vec!["needs-review".into(), "in-progress".into()])
        );
    }
}
