use std::collections::HashMap;

use sqlx::{Row, SqlitePool};

use crate::ir::{ConstraintIR, ConstraintKind, FieldIR, ReferentialAction};
use crate::introspect::Column;
use crate::types::{self, SqliteType};

/// SQLite implementation of [`DatabaseIntrospector`](crate::introspect::DatabaseIntrospector).
///
/// Uses `PRAGMA table_info` and `sqlite_master` to discover tables
/// and their columns.
pub struct SqliteIntrospector {
    /// SQLx connection pool for the target database.
    pub pool: SqlitePool,
}

impl SqliteIntrospector {
    /// Create a new introspector from an existing SQLite connection pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl super::DatabaseIntrospector for SqliteIntrospector {
    fn column_to_field(&self, col: &Column) -> FieldIR {
        let sqlite_ty = SqliteType::map_sqlite_type(&col.data_type);
        let db_ty = types::sqlite_to_db_type(sqlite_ty);
        FieldIR {
            name: col.column_name.clone(),
            ty: db_ty,
            nullable: col.nullable,
            raw_type: col.data_type.clone(),
        }
    }
    async fn list_tables(&self) -> anyhow::Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT name
            FROM sqlite_master
            WHERE type = 'table'
              AND name NOT LIKE 'sqlite_%'
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| r.get::<Option<String>, _>("name"))
            .collect())
    }

    async fn list_columns(&self, table: &str) -> anyhow::Result<Vec<Column>> {
        let rows = sqlx::query("SELECT * FROM pragma_table_info(?1)")
            .bind(table)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let raw: Option<String> = r.get("type");
                Column {
                    table_name: table.to_string(),
                    column_name: r.get("name"),
                    data_type: raw.unwrap_or_else(|| "TEXT".to_string()),
                    nullable: r.get::<i32, _>("notnull") == 0,
                }
            })
            .collect())
    }

    async fn list_constraints(&self, table: &str) -> anyhow::Result<Vec<ConstraintIR>> {
        let mut constraints: Vec<ConstraintIR> = Vec::new();

        // Primary key — from pragma_table_info (pk > 0)
        let pk_rows = sqlx::query("SELECT name, pk FROM pragma_table_info(?1) WHERE pk > 0 ORDER BY pk")
            .bind(table)
            .fetch_all(&self.pool)
            .await?;

        let pk_cols: Vec<String> = pk_rows.into_iter().map(|r| r.get("name")).collect();
        if !pk_cols.is_empty() {
            constraints.push(ConstraintIR {
                name: format!("{}_pk", table),
                kind: ConstraintKind::PrimaryKey { columns: pk_cols },
            });
        }

        // Foreign keys — from pragma_foreign_key_list
        let fk_rows = sqlx::query("SELECT * FROM pragma_foreign_key_list(?1)")
            .bind(table)
            .fetch_all(&self.pool)
            .await?;

        let mut fk_groups: HashMap<i32, FkBuilder> = HashMap::new();
        for r in fk_rows {
            let id: i32 = r.get("id");
            let entry = fk_groups.entry(id).or_insert_with(|| FkBuilder {
                columns: Vec::new(),
                ref_table: String::new(),
                ref_columns: Vec::new(),
                update_rule: None,
                delete_rule: None,
            });
            entry.columns.push(r.get::<String, _>("from"));
            entry.ref_table = r.get::<String, _>("table");
            entry.ref_columns.push(r.get::<String, _>("to"));
            if entry.update_rule.is_none() {
                entry.update_rule = r.try_get::<Option<String>, _>("on_update").ok().flatten();
            }
            if entry.delete_rule.is_none() {
                entry.delete_rule = r.try_get::<Option<String>, _>("on_delete").ok().flatten();
            }
        }
        for (_id, fb) in fk_groups {
            constraints.push(ConstraintIR {
                name: format!("{}_{}_fk", table, fb.columns.join("_")),
                kind: ConstraintKind::ForeignKey {
                    columns: fb.columns,
                    referenced_table: fb.ref_table,
                    referenced_columns: fb.ref_columns,
                    on_delete: sqlite_referential_action(fb.delete_rule.as_deref()),
                    on_update: sqlite_referential_action(fb.update_rule.as_deref()),
                    match_type: None,
                },
            });
        }

        // Unique constraints — from pragma_index_list (origin = 'u')
        let idx_rows = sqlx::query("SELECT * FROM pragma_index_list(?1) WHERE origin = 'u'")
            .bind(table)
            .fetch_all(&self.pool)
            .await?;

        for r in idx_rows {
            let idx_name: String = r.get("name");
            let col_rows = sqlx::query("SELECT name FROM pragma_index_info(?1)")
                .bind(&idx_name)
                .fetch_all(&self.pool)
                .await?;
            let columns: Vec<String> = col_rows.into_iter().map(|cr| cr.get("name")).collect();
            constraints.push(ConstraintIR {
                name: idx_name,
                kind: ConstraintKind::Unique { columns },
            });
        }

        // Check constraints — best-effort from CREATE TABLE parsing
        let sql_row = sqlx::query("SELECT sql FROM sqlite_master WHERE type='table' AND name=?1")
            .bind(table)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = sql_row {
            let sql: String = row.get("sql");
            if let Some(checks) = parse_sqlite_checks(&sql) {
                for (name, expr) in checks {
                    constraints.push(ConstraintIR {
                        name,
                        kind: ConstraintKind::Check { expression: expr },
                    });
                }
            }
        }

        Ok(constraints)
    }
}

struct FkBuilder {
    columns: Vec<String>,
    ref_table: String,
    ref_columns: Vec<String>,
    update_rule: Option<String>,
    delete_rule: Option<String>,
}

fn sqlite_referential_action(s: Option<&str>) -> ReferentialAction {
    match s {
        Some("CASCADE" | "CASCADE ") => ReferentialAction::Cascade,
        Some("SET NULL") => ReferentialAction::SetNull,
        Some("SET DEFAULT") => ReferentialAction::SetDefault,
        Some("RESTRICT") => ReferentialAction::Restrict,
        _ => ReferentialAction::NoAction,
    }
}

/// Best-effort extraction of CHECK constraints from a SQLite CREATE TABLE statement.
///
/// Returns `None` if parsing fails (caller should degrade gracefully).
fn parse_sqlite_checks(sql: &str) -> Option<Vec<(String, String)>> {
    let mut checks = Vec::new();
    let upper = sql.to_uppercase();
    let bytes = sql.as_bytes();

    let mut search_start = 0usize;
    loop {
        // Look for "CHECK" followed by optional whitespace and "("
        let remaining = &upper[search_start..];
        let check_pos = match remaining.find("CHECK") {
            Some(p) => search_start + p,
            None => break,
        };
        let after_check = check_pos + 5;
        // skip whitespace between CHECK and (
        let mut paren_start = after_check;
        while paren_start < bytes.len()
            && (bytes[paren_start] == b' ' || bytes[paren_start] == b'\t' || bytes[paren_start] == b'\n')
        {
            paren_start += 1;
        }
        if paren_start >= bytes.len() || bytes[paren_start] != b'(' {
            search_start = after_check;
            continue;
        }

        // Backtrack to find optional CONSTRAINT name
        let before = &sql[..check_pos].trim_end();
        let name = if let Some(con_pos) = before.to_uppercase().rfind("CONSTRAINT") {
            let name_part = before[con_pos + 10..].trim();
            name_part.split_whitespace().next().unwrap_or("").to_string()
        } else {
            "check".to_string()
        };

        // Find matching closing paren
        let expr_start = paren_start + 1; // after '('
        let mut depth = 1u32;
        let mut expr_end = expr_start;
        let mut found = false;
        for i in expr_start..bytes.len() {
            match bytes[i] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        expr_end = i;
                        found = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !found {
            return None; // unbalanced parens
        }
        let expression = sql[expr_start..expr_end].trim().to_string();
        checks.push((name, expression));

        search_start = expr_end + 1;
        if search_start >= sql.len() {
            break;
        }
    }

    Some(checks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sqlite_checks_basic() {
        let sql = "CREATE TABLE users (id INTEGER, age INTEGER CHECK(age > 0))";
        let result = parse_sqlite_checks(sql);
        assert!(result.is_some());
        let checks = result.unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].1, "age > 0");
    }

    #[test]
    fn test_parse_sqlite_checks_named() {
        let sql = "CREATE TABLE users (id INTEGER, age INTEGER CONSTRAINT age_check CHECK(age > 0))";
        let result = parse_sqlite_checks(sql);
        assert!(result.is_some());
        let checks = result.unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].0, "age_check");
        assert_eq!(checks[0].1, "age > 0");
    }

    #[test]
    fn test_parse_sqlite_checks_multiple() {
        let sql = "CREATE TABLE t (a INT CHECK(a > 0), b INT CHECK(b < 100))";
        let result = parse_sqlite_checks(sql);
        assert!(result.is_some());
        let checks = result.unwrap();
        assert_eq!(checks.len(), 2);
    }

    #[test]
    fn test_parse_sqlite_checks_none() {
        let sql = "CREATE TABLE users (id INTEGER, name TEXT)";
        let result = parse_sqlite_checks(sql);
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_sqlite_checks_nested_parens() {
        let sql = "CREATE TABLE t (a INT CHECK((a > 0) AND (a < 100)))";
        let result = parse_sqlite_checks(sql);
        assert!(result.is_some());
        let checks = result.unwrap();
        assert_eq!(checks.len(), 1);
        assert!(checks[0].1.contains("(a > 0) AND (a < 100)"));
    }
}
