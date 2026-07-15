# Changelog

All notable changes to this project will be documented here.

## [0.5.5] - 2026-07-14

### Added
- **`neutrino-schema generate --from-ir <FILE>`** — generate code from a
  SchemaIR JSON file without a live database connection. Loads, validates,
  then proceeds through the normal codegen pipeline. (`src/cli/generate.rs`)
- **`neutrino-schema export`** — introspect a database and write a
  versioned SchemaIR JSON file. Supports `--output`, `--pretty`.
  URL resolution via CLI flag, `DATABASE_URL` env, config file, or
  interactive prompt. (`src/cli/export.rs`)
- **`SchemaIR::from_database()`** — convenience constructor that
  introspects a live database and populates `Metadata.provider`.
  (`src/ir/schema.rs`)
- **Validator module** (`src/validator/`) — standalone semantic validation
  returning a `ValidationReport` with `ValidationEntry` items.
  - Empty/whitespace identifiers (table, field, enum names) → Error
  - Duplicate table names → Error
  - Duplicate enum `rust_name` → Error
  - Unresolved `DbType::Enum` references → Error
  - FK references to non-existent tables → Error
  - Orphan enums (defined but never referenced) → Warning
  - `ValidationReport` helpers: `has_errors()`, `errors()`, `has_warnings()`, `warnings()`
- **Public introspection API** — `introspect_tables()` and
  `introspect_schema()` moved from `cli/mod.rs` to `introspect/mod.rs`
  as `pub` functions.

### Changed
- `SchemaIR::validate()` removed — validation is now a standalone
  operation over the IR, not a method on `SchemaIR`.
- `SchemaError` enum removed entirely (was only used by the old `validate()`).

## [0.5.4] - 2026-07-14

Same as 0.5.5 — superseded during development.

## [0.5.3] - 2026-07-13

### Added
- **Column default values** — `default_value: Option<String>` on `Column` and
  `FieldIR` preserving raw SQL default expressions from all backends (PostgreSQL
  `column_default`, MySQL `COLUMN_DEFAULT`, SQLite `dflt_value`).
- **Generated column detection** — `generated: bool` on `Column` and `FieldIR`
  for identity columns: PostgreSQL (`is_identity` / `nextval(`), MySQL
  (`auto_increment`), SQLite (`INTEGER PRIMARY KEY`).

## [0.5.0] - 2026-07-13

### Added
- **PK metadata constants** — every generated struct now gets a
  `pub const TABLE_NAME_PRIMARY_KEY: &[&str]` listing primary key column names.
  Composite PKs (e.g. `post_tags(post_id, tag_id)`) include all columns.
  No PK → no constant emitted.
- **Composite FK support** — `RelationIR::from_columns` and
  `RelationIR::to_columns` are now `Vec<String>`, so multi-column foreign keys
  (e.g. `user_sessions(user_id, device_id)`) produce a single `RelationIR`
  instead of being split into per-column pairs.
- **Better plural resolution** — `infer_relations_heuristic` now tries `"es"`
  and `"ies"` suffix strategies in addition to `"s"` (e.g. `status_id` → `statuses`,
  `category_id` → `categories`).
- **Dynamic `to_columns` via PK lookup** — the naming heuristic now reads the
  target table's primary key columns instead of always assuming `"id"`.

### Changed (breaking)
- `RelationSource` renamed to `RelationOrigin` — describes provenance
  (`ForeignKey` / `Inferred`) rather than nature of the relation.
- `RelationIR::source` → `RelationIR::origin`.
- `RelationIR::from_field` (String) → `RelationIR::from_columns` (Vec\<String\>).
- `RelationIR::to_field` (String) → `RelationIR::to_columns` (Vec\<String\>).
- `RelationSource::ForeignKey(String)` → `RelationOrigin::ForeignKey` —
  the constraint name was never consumed downstream and added noise.
- `RelationSource::NamingHeuristic` → `RelationOrigin::Inferred`.
- `ConstraintIR::fk_relations()` now returns one `RelationIR` per FK constraint
  (not one per column pair), correctly representing composite foreign keys.

## [0.4.7] - 2026-07-13

### Added
- **IndexIR introspection** for PostgreSQL, MySQL, and SQLite — every index
  (unique, expression, partial, multi-column) is captured as `IndexIR` in
  the schema IR.
- **Expanded enum support** — ENUM columns on PostgreSQL and MySQL are now
  introspected as proper EnumIR with named variants; codegen derives
  `EnumString`/`Display` and uses the enum type instead of raw strings.
- **ConstraintIR for all backends** — FOREIGN KEY, UNIQUE, PRIMARY KEY, and CHECK
  constraints are introspected into a unified `ConstraintIR` per table.
- **Cross-backend constraint parity** — the same suite of CHECK constraints now
  works on PostgreSQL, MySQL, and SQLite (MySQL filtered via CHECK_CONSTRAINTS
  + TABLE_CONSTRAINTS JOIN).
- **`parse_referential_action` shared helper** — extracted from `postgres.rs`
  into `introspect/helpers.rs` for use by all backends.
- **Level 4A syn validation** — generated Rust code is parsed with `syn`
  during the roundtrip test to catch syntax errors before cargo execution.
- **`sanitize_identifier()` / `deduplicate_identifier()`** in `util::naming` —
  sanitizes field/struct names for safe Rust codegen (removes leading digits,
  non-alphanumeric chars, keyword collisions).

### Fixed
- **FK heuristic suppression** — `infer_relations_heuristic` now skips relations
  already covered by a FOREIGN KEY constraint, avoiding duplicate relations.
- **Comment injection via doc comments** — multiline database comments are now
  escaped line-by-line with `///` prefix instead of a single `///` block,
  preventing injected Rust syntax.
- **PG enum/array column types** — `list_columns` uses `udt_name` for
  `USER-DEFINED` (enum) and `ARRAY` types, resolving to actual type names.
- **PG expression index parsing** — replaced `pg_get_expr(i.indexprs)` comma-split
  with per-key `pg_get_indexdef(indexrelid, kpno, true)` via LATERAL join,
  robust against commas inside function arguments.
- **CLI config priority** — `generate` now reads `[generator]` section from
  `neutrino-schema.toml` with CLI > config > default priority.
- **`--table` validation** — CLI checks that requested table names exist before
  introspecting, with a clear error message.
- **SQLite unique constraint count** — filter by `origin='u'` to correctly
  report user-defined unique constraints vs internal unique indexes.
- **`sqlite_full_pipeline` test** — updated to expect heuristic relations are
  suppressed when FK metadata already covers the same pair.

### Changed
- **MySQL unique constraints** — documented cross-backend difference: MySQL/InnoDB
  reports all unique indexes as both `ConstraintIR::Unique` and `IndexIR` since
  they are physically the same object.

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
