use std::path::PathBuf;

/// Backend selector for migration file loading.
pub enum MigrationBackend {
    Sqlite,
    Postgres,
    Mysql,
}

impl MigrationBackend {
    fn dir_name(&self) -> &str {
        match self {
            MigrationBackend::Sqlite => "sqlite",
            MigrationBackend::Postgres => "postgresql",
            MigrationBackend::Mysql => "mysql",
        }
    }
}

fn migrations_root() -> PathBuf {
    let manifest = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    manifest.join("tests/migrations")
}

/// Load all migration SQL files for the given backend, concatenated in
/// file-name order.
///
/// Backend-specific cleanup:
/// - **PostgreSQL**: skips lines containing `CREATE EXTENSION` (not needed
///   for introspection and would fail on other databases).
/// - **SQLite / MySQL**: no cleanup — real migrations are executed as-is.
pub fn load_migration_sql(backend: MigrationBackend) -> anyhow::Result<String> {
    let dir = migrations_root().join(backend.dir_name());
    let mut entries: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut parts = Vec::new();
    for entry in entries {
        let sql = std::fs::read_to_string(entry.path())?;
        // Only strip CREATE EXTENSION for PostgreSQL — it's a setup step
        // that fails on other databases and isn't needed for introspection.
        let sql = match backend {
            MigrationBackend::Postgres => sql
                .lines()
                .filter(|l| !l.trim().to_uppercase().starts_with("CREATE EXTENSION"))
                .collect::<Vec<_>>()
                .join("\n"),
            _ => sql,
        };
        parts.push(sql);
    }
    Ok(parts.join("\n\n"))
}

/// Execute a batch of SQL statements against an SQLite pool.
///
/// Handles semicolons inside string literals and `--` line comments.
/// Note: `sqlx::query` requires `&'static str` in sqlx 0.9, so we leak the
/// allocated statements. This is acceptable for test-only code.
pub async fn execute_sqlite_batch(pool: &sqlx::SqlitePool, sql: &str) -> anyhow::Result<()> {
    for stmt in split_sql_statements(sql) {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            let leaked: &'static str = Box::leak(trimmed.to_string().into_boxed_str());
            sqlx::query(leaked).execute(pool).await?;
        }
    }
    Ok(())
}

/// Split SQL text into individual statements, respecting quoted strings
/// and line comments.
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut stmts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_line_comment = false;
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Line comment start `--` — skip both `-` chars and the rest of the line
        if !in_single_quote && c == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
            in_line_comment = true;
            i += 2; // skip both `--`
            continue;
        }
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }

        // Single-quoted string — track escape `''`
        if c == '\'' {
            current.push(c);
            if !in_single_quote {
                in_single_quote = true;
            } else if i + 1 < chars.len() && chars[i + 1] == '\'' {
                // Escaped single quote `''` inside string
                current.push(chars[i + 1]);
                i += 2;
                continue;
            } else {
                in_single_quote = false;
            }
            i += 1;
            continue;
        }

        // Statement separator `;` outside strings
        if c == ';' && !in_single_quote {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                stmts.push(trimmed);
            }
            current = String::new();
            i += 1;
            continue;
        }

        current.push(c);
        i += 1;
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        stmts.push(trimmed);
    }

    stmts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_basic() {
        let sql = "SELECT 1; SELECT 2;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn split_with_quoted_semicolon() {
        let sql = "SELECT 'hello; world' AS x; SELECT 2";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts, vec!["SELECT 'hello; world' AS x", "SELECT 2"]);
    }

    #[test]
    fn split_with_escaped_quote() {
        let sql = "SELECT 'it''s; ok' AS x; SELECT 2";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts, vec!["SELECT 'it''s; ok' AS x", "SELECT 2"]);
    }

    #[test]
    fn split_skips_line_comment() {
        let sql = "SELECT 1; -- comment; here\nSELECT 2;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn load_migration_sqlite() {
        let sql = load_migration_sql(MigrationBackend::Sqlite).unwrap();
        // Should contain users table
        assert!(sql.contains("CREATE TABLE"));
        assert!(sql.contains("users"));
        assert!(sql.contains("oauth_refresh_tokens"));
    }
}
