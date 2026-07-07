# Changelog

All notable changes to this project will be documented here.

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
