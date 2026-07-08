# Changelog

All notable changes to this project will be documented here.

## [0.3.1] - 2026-07-07

### Changed
- Replaced `atty` (unmaintained, RUSTSEC-2024-0375) with `std::io::IsTerminal`
  (stable since Rust 1.70).  One fewer dependency, zero behavior change.

## [0.3.0] - 2026-07-07

### Added
- **`neutrino-schema init`** — creates `neutrino-schema.toml` with
  `[databases.default]` and `[generator]` sections (`--database-url` to
  pre-fill, `--force` to overwrite).
- **Auto-bootstrap** — `neutrino-schema generate` prompts for a database URL
  on first run, saves it to `neutrino-schema.toml`, and reuses it thereafter.
- **Named database connections** — `[databases.*]` HashMap in config supports
  future multi-database projects; `--database <name>` flag selects which
  connection to use.
- **`version = 1`** — top-level config version for future migration paths.
- **`DatabaseProvider` enum** — `Postgres`, `MySql`, `Sqlite`, inferred from
  URL scheme (or explicitly set). Provider mismatch produces a clear error.
- **`--save` flag** — explicitly persists a CLI-provided URL to config.
- **`--non-interactive` / `--all` flags** — reserved for CI and multi-db.
- **Shared `url_to_introspector()`** — extracted from duplicated connect
  helpers in `generate.rs` / `inspect.rs`.
- **Config test suite** (`tests/config.rs`) — 12 tests covering defaults,
  round-trip, provider inference, TOML rename, partial configs,
  multiple databases.

### Changed
- `GeneratorConfig` and `RenderMode` gain serde derives (behind `cli` feature)
  for config file serialization.
- `GeneratorConfig.output_dir` serializes as `output` in TOML.
- Reduced CLI boilerplate: `--database-url` is optional (env + config + prompt
  chain fills the gap).

## [0.2.2] - 2026-07-07

### Added
- `examples/saas-subscription/` — 7-table SaaS subscription platform schema
  and README showcase demonstrating neutrino-schema output across all types
  (BOOLEAN, JSONB, TIMESTAMPTZ, VARCHAR, TEXT, BIGINT, DECIMAL/cents).

## [0.2.1] - 2026-07-07

### Fixed
- Crate and README documentation updated from PostgreSQL-only to reflect
  multi-database support (PostgreSQL, MySQL/MariaDB, SQLite).
- MySQL integration tests gracefully skip when server is unreachable.

## [0.2.0] - 2026-07-07

### Added
- SQLite support: `SqliteType` enum, `SqliteIntrospector` (via `PRAGMA table_info`),
  feature gate (`sqlite`), CLI auto-detection of `sqlite:` URLs.
- MySQL/MariaDB support: `MysqlType` enum, `MysqlIntrospector` (via
  `information_schema.columns`), feature gate (`mysql`),
  CLI auto-detection of `mysql:` URLs.
- `DbType::Float` variant for SQLite `REAL` / MySQL `FLOAT` / `DECIMAL`.
- Integration tests for SQLite (in-memory) and MySQL (real database).
- `DatabaseIntrospector::column_to_field` moved into trait for polymorphic dispatch.

### Changed
- `Column.data_type` from `PgType` to `String` so the shared struct works
  across all three database backends.
- `DatabaseIntrospector` trait extracted into own `traits.rs` (not feature-gated).
- Default features now include `postgres`, `sqlite`, `mysql`, and `cli`.
- All intra-doc links use `crate::` paths (zero doc warnings).

## [0.1.3] - 2026-07-06

### Added
- `command-line-utilities` category in Cargo.toml for crates.io discovery.
- Before/after SQL-to-Rust example in README for instant value demonstration.
- Cache-busting query parameters on shields.io and docs.rs badge URLs.

## [0.1.2] - 2026-07-06

### Added
- LICENSE-MIT and LICENSE-APACHE files (missing from standalone repo).
- `documentation = "https://docs.rs/neutrino-schema"` in Cargo.toml.
- `deny.toml` for cargo-deny CI check.
- Standalone CI workflow without `--workspace` flags.

### Fixed
- Workspace-inherited Cargo.toml fields (`edition`, `license`, `repository`,
  `homepage`, `lints`) hardcoded to resolve independently.

## [0.1.1] - 2026-07-06

### Added
- Full documentation comments on all public API items (module-level `//!`,
  structs, enums, variants, fields, functions, traits) for docs.rs.
- Comprehensive crate-level docs with pipeline explanation and usage examples.
- Doc-test examples on `to_struct_name`, `to_db_type`, `dbtype_to_rust`,
  `generate_struct`.
- `to_struct_name` is now re-exported from the crate root.

### Fixed
- `DbType` now derives `PartialEq` (needed for doc-test assertions).

## [0.1.0] - 2026-07-06

### Added
- Schema introspection pipeline: PostgresIntrospector, DatabaseIntrospector trait.
- Intermediate Representation: FieldIR, TableIR, SchemaIR, RelationIR.
- Relation inference via naming heuristics (`_id` suffix) or disabled.
- Type system: PgType (raw), DbType (normalised).
- Rust code generation: `generate_struct`, `generate_files`, `RenderMode`.
- CLI: `inspect` (single/all tables, with/without comments) and `generate` commands.
- CI pipeline (check, clippy, test, deny).
- License files (MIT / Apache-2.0).
