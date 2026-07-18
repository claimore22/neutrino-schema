#![allow(clippy::unwrap_used, unused)]
//!
//! SQLite CLI integration test — creates a file-based SQLite DB from the
//! migration fixtures, then runs every CLI subcommand against it.
//!

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sqlx::ConnectOptions;

fn migrations_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/migrations/sqlite")
}

fn bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_neutrino-schema"))
}

fn tmp_path(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("cli_test");
    fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn sqlite_url(db_path: &Path) -> String {
    format!("sqlite:{}", db_path.display())
}

/// Create a file-based SQLite database by executing every `.sql` migration
/// in sorted order.
fn create_db(db_path: &Path) {
    if db_path.exists() {
        fs::remove_file(db_path).unwrap();
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(migrations_dir())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "sql"))
        .collect();
    entries.sort();

    // Concatenate all migration files.
    let mut all_sql = String::new();
    for path in &entries {
        let sql = fs::read_to_string(path).unwrap();
        all_sql.push_str(&sql);
        all_sql.push('\n');
    }

    // Leak the concatenated SQL so it becomes &'static (required by sqlx 0.9).
    let static_sql: &'static str = Box::leak(all_sql.into_boxed_str());

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let mut conn = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .connect()
            .await
            .unwrap();

        for raw_stmt in static_sql.split(';') {
            let trimmed = raw_stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Skip pure comment lines
            let non_comment_lines: Vec<&str> = trimmed
                .lines()
                .filter(|l| !l.trim().starts_with("--"))
                .collect();
            if non_comment_lines.is_empty() || non_comment_lines.iter().all(|l| l.trim().is_empty())
            {
                continue;
            }
            let clean = non_comment_lines.join("\n");
            if clean.trim().is_empty() {
                continue;
            }
            let stmt: &'static str = Box::leak(clean.into_boxed_str());
            sqlx::query(stmt)
                .execute(&mut conn)
                .await
                .unwrap_or_else(|e| panic!("exec stmt: {e}\nSQL: {stmt}"));
        }
    });
    drop(rt);

    eprintln!("created {}", db_path.display());
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run with DATABASE_URL env var (for generate / export).
fn run_with_db(args: &[&str], db_path: &Path) -> (String, bool) {
    let output = Command::new(bin_path())
        .args(args)
        .env("DATABASE_URL", sqlite_url(db_path))
        .output()
        .expect("failed to run neutrino-schema");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let ok = output.status.success();
    if !ok {
        eprintln!("FAILED: neutrino-schema {}", args.join(" "));
        eprintln!("stderr:\n{stderr}");
    }
    (format!("{stdout}\n{stderr}"), ok)
}

/// Run with database URL as positional arg (for inspect).
fn run_inspect(args: &[&str], db_path: &Path) -> (String, bool) {
    let url = sqlite_url(db_path);
    let mut full_args: Vec<&str> = vec![&url];
    full_args.extend_from_slice(args);

    let output = Command::new(bin_path())
        .arg("inspect")
        .args(&full_args)
        .output()
        .expect("failed to run neutrino-schema inspect");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let ok = output.status.success();
    if !ok {
        eprintln!("FAILED: neutrino-schema inspect {}", args.join(" "));
        eprintln!("stderr:\n{stderr}");
    }
    (format!("{stdout}\n{stderr}"), ok)
}

/// Run without any DB env (for --help, --version, init).
fn run_no_env(args: &[&str]) -> (String, bool) {
    let output = Command::new(bin_path())
        .args(args)
        .output()
        .expect("failed to run neutrino-schema");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let ok = output.status.success();
    if !ok {
        eprintln!("FAILED: neutrino-schema {}", args.join(" "));
        eprintln!("stderr:\n{stderr}");
    }
    (format!("{stdout}\n{stderr}"), ok)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn cli_help() {
    let (out, ok) = run_no_env(&["--help"]);
    assert!(ok, "--help should succeed\n{out}");
    assert!(out.contains("generate"), "help should mention generate");
    assert!(out.contains("inspect"), "help should mention inspect");
    assert!(out.contains("export"), "help should mention export");
    assert!(out.contains("init"), "help should mention init");
    eprintln!("=== help ===\n{out}");
}

#[test]
fn cli_inspect_sqlite_tables_list() {
    let db = tmp_path("inspect.sqlite");
    create_db(&db);

    // No table arg → should list all tables
    let (out, ok) = run_inspect(&[], &db);
    assert!(ok, "inspect should succeed\n{out}");
    assert!(out.contains("users"), "should list users table");
    assert!(out.contains("roles"), "should list roles table");
    assert!(out.contains("user_roles"), "should list user_roles table");
    eprintln!("=== inspect (list tables) ===\n{out}");
}

#[test]
fn cli_inspect_sqlite_single_table() {
    let db = tmp_path("inspect_single.sqlite");
    create_db(&db);

    // Inspect a single table
    let (out, ok) = run_inspect(&["users"], &db);
    assert!(ok, "inspect users should succeed\n{out}");
    assert!(out.contains("pub struct Users"), "should contain Users struct\n{out}");
    eprintln!("=== inspect users ===\n{out}");
}

#[test]
fn cli_inspect_sqlite_all() {
    let db = tmp_path("inspect_all.sqlite");
    create_db(&db);

    // --all should generate to generated/ directory
    let (out, ok) = run_inspect(&["--all"], &db);
    assert!(ok, "inspect --all should succeed\n{out}");
    assert!(out.contains("Generated"), "should print generation summary\n{out}");
    assert!(out.contains("Relations"), "should print relation info\n{out}");
    eprintln!("=== inspect --all ===\n{out}");
}

#[test]
fn cli_generate_sqlite() {
    let db = tmp_path("generate.sqlite");
    create_db(&db);

    let out_dir = tmp_path("generate_out");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).unwrap();
    }

    let (out, ok) = run_with_db(
        &["generate", "--output", out_dir.to_str().unwrap()],
        &db,
    );
    assert!(ok, "generate should succeed\n{out}");

    let files: Vec<String> = fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert!(!files.is_empty(), "should generate at least one file");
    assert!(files.iter().any(|f| f.contains("user")), "should have a users file\n{files:?}");
    assert!(files.iter().any(|f| f.contains("role")), "should have a roles file\n{files:?}");
    eprintln!("=== generate ===\n{out}\nfiles: {files:?}");
}

#[test]
fn cli_generate_with_table_filter() {
    let db = tmp_path("gen_filter.sqlite");
    create_db(&db);

    let out_dir = tmp_path("gen_filter_out");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).unwrap();
    }

    let (out, ok) = run_with_db(
        &[
            "generate",
            "--output",
            out_dir.to_str().unwrap(),
            "--table",
            "users",
        ],
        &db,
    );
    assert!(ok, "generate --table users should succeed\n{out}");

    let files: Vec<String> = fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert!(files.iter().any(|f| f.contains("user")), "should have users file");
    assert!(!files.iter().any(|f| f.contains("role")), "should NOT have roles file when filtered");
    eprintln!("=== generate --table users ===\n{out}\nfiles: {files:?}");
}

#[test]
fn cli_generate_code_with_debug() {
    let db = tmp_path("gen_code.sqlite");
    create_db(&db);

    let out_dir = tmp_path("gen_code_out");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).unwrap();
    }

    let (out, ok) = run_with_db(
        &[
            "generate",
            "--output",
            out_dir.to_str().unwrap(),
            "--debug",
        ],
        &db,
    );
    assert!(ok, "generate --debug should succeed\n{out}");

    let files: Vec<PathBuf> = fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    assert!(!files.is_empty(), "should have generated .rs files");

    let all_code: String = files
        .iter()
        .map(|f| fs::read_to_string(f).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        all_code.contains("// ") || all_code.contains("///"),
        "debug mode should produce type comments\n{all_code}"
    );
    eprintln!("=== generate --debug ===\n{out}");
}

#[test]
fn cli_export_sqlite() {
    let db = tmp_path("export.sqlite");
    create_db(&db);

    let json_path = tmp_path("export.json");

    let (out, ok) = run_with_db(
        &["export", "--output", json_path.to_str().unwrap()],
        &db,
    );
    assert!(ok, "export should succeed\n{out}");
    assert!(json_path.exists(), "JSON file should be created");

    let json_str = fs::read_to_string(&json_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.get("tables").is_some(), "JSON should have 'tables' key");
    let tables = parsed["tables"].as_array().unwrap();
    let table_names: Vec<&str> = tables
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(table_names.contains(&"users"), "should export users");
    assert!(table_names.contains(&"roles"), "should export roles");
    eprintln!("=== export ===\n{out}\ntables: {table_names:?}");
}

#[test]
fn cli_export_generate_roundtrip() {
    let db = tmp_path("roundtrip.sqlite");
    create_db(&db);

    let json_path = tmp_path("roundtrip.json");
    let out_dir = tmp_path("roundtrip_out");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).unwrap();
    }

    // Step 1: Export
    let (out, ok) = run_with_db(
        &["export", "--output", json_path.to_str().unwrap()],
        &db,
    );
    assert!(ok, "export should succeed\n{out}");

    // Step 2: Generate from IR (--from-ir)
    let (out, ok) = run_with_db(
        &[
            "generate",
            "--from-ir",
            json_path.to_str().unwrap(),
            "--output",
            out_dir.to_str().unwrap(),
        ],
        &db,
    );
    assert!(ok, "generate --from-ir should succeed\n{out}");

    let files: Vec<String> = fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert!(!files.is_empty(), "roundtrip should produce files");
    eprintln!("=== roundtrip ===\n{out}\nfiles: {files:?}");
}

#[test]
fn cli_generate_from_json_alias() {
    let db = tmp_path("alias.sqlite");
    create_db(&db);

    let json_path = tmp_path("alias.json");
    let out_dir = tmp_path("alias_out");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).unwrap();
    }

    // Export first
    let (_, ok) = run_with_db(
        &["export", "--output", json_path.to_str().unwrap()],
        &db,
    );
    assert!(ok);

    // Use --from-json alias
    let (out, ok) = run_with_db(
        &[
            "generate",
            "--from-json",
            json_path.to_str().unwrap(),
            "--output",
            out_dir.to_str().unwrap(),
        ],
        &db,
    );
    assert!(ok, "generate --from-json alias should succeed\n{out}");

    let files: Vec<String> = fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert!(!files.is_empty(), "--from-json should produce files");
    eprintln!("=== --from-json ===\n{out}");
}

#[test]
fn cli_init() {
    let dir = tmp_path("init_dir");
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();

    let output = Command::new(bin_path())
        .arg("init")
        .current_dir(&dir)
        .output()
        .expect("failed to run neutrino-schema init");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let out = format!("{stdout}\n{stderr}");
    assert!(output.status.success(), "init should succeed\n{out}");

    let toml_path = dir.join("neutrino-schema.toml");
    assert!(toml_path.exists(), "neutrino-schema.toml should be created");
    let content = fs::read_to_string(&toml_path).unwrap();
    assert!(content.contains("database"), "TOML should have database section");
    eprintln!("=== init ===\n{out}\nTOML:\n{content}");
}
