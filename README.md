# neutrino-schema

[![CI](https://github.com/claimore22/neutrino-schema/actions/workflows/ci.yml/badge.svg)](https://github.com/claimore22/neutrino-schema/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/neutrino-schema.svg?refresh=1)](https://crates.io/crates/neutrino-schema)
[![docs.rs](https://docs.rs/neutrino-schema/badge.svg?refresh=1)](https://docs.rs/neutrino-schema)
[![License](https://img.shields.io/crates/l/neutrino-schema.svg?refresh=1)](https://github.com/claimore22/neutrino-schema)
[![sqlx](https://img.shields.io/badge/sqlx-0.9-blue)](https://crates.io/crates/sqlx)

> **Note:** This project is under active development. The SchemaIR is stabilizing but may still see breaking changes in minor releases. Pin your version and check the [CHANGELOG](CHANGELOG.md) before upgrading.

Compile relational database schemas into strongly typed Rust types.

```sql
CREATE TABLE users (
    id         BIGSERIAL    PRIMARY KEY,
    email      VARCHAR(255) NOT NULL,
    full_name  VARCHAR(100)
);
```

```bash
neutrino-schema generate --database-url $DATABASE_URL --output src/entities
```

```rust
#[derive(Debug, Clone)]
pub struct Users {
    pub id: i64,
    pub email: String,
    pub full_name: Option<String>,
}
```

## Supported databases

| Database | Minimum version | Notes |
|---|---|---|
| PostgreSQL | 8.1 (recommended: 14+) | Enum introspection requires 8.1+ |
| MySQL | 8.0.16 (5.7+ partial) | Full support on 8.0.16+. On 5.7, CHECK constraint introspection is not available. MariaDB 10.2.1+ (recommended: 11.0+) |
| SQLite | 3.8.0 (recommended: 3.35+) | sqlx bundles its own libsqlite3 3.45+ so this is effectively always satisfied |

## Overview

`neutrino-schema` is a compiler pipeline that introspects database schemas — PostgreSQL, MySQL/MariaDB, and SQLite — including tables, columns, data types, nullability, enums, and relationships. The extracted schema is transformed through an intermediate representation (`SchemaIR`) with a configurable type-resolution layer (`TypeRegistry`) and then used to generate Rust source code.

## How does it work?

```
                Database
                   |
                   v
         Database Introspection
                   |
                   v
           Database Types
      (PgType / MysqlType / SqliteType)
                   |
                   v
                SchemaIR
      (tables, fields, relations,
       enums, constraints, metadata)
                   |
                   v
           Type Resolution
        (DbType ──TypeRegistry──> RustType)
                   |
                   v
         Rust Code Generation
                   |
                   v
         Strongly typed Rust code
```

Raw database types are normalised into a database-agnostic `DbType` enum, then resolved to `RustType` (name + imports) by a `TypeRegistry`. Users can override any mapping in their config file.

## Features

- PostgreSQL, MySQL/MariaDB, and SQLite introspection
- Enum introspection & generation (PostgreSQL `CREATE TYPE`, MySQL `enum(...)` column definitions)
- Normalised intermediate representation (`SchemaIR`)
- Configurable type resolution (`TypeRegistry`) with `[types]` overrides — e.g. `money = "rust_decimal::Decimal"`
- Feature-gated Rust types: `uuid::Uuid`, `rust_decimal::Decimal`, `chrono::DateTime<Utc>`, etc.
- Automatic Rust struct generation with `Option<T>` for nullable columns
- CLI tooling: `init`, `inspect`, `export`, and `generate` commands
- **JSON export** — serialize a normalized SchemaIR to a versioned JSON file
- **Offline code generation** — `generate --from-ir` works without a database connection
- **Schema validation** — semantic checks for duplicate tables, missing references, orphan enums
- Config file workflow (`neutrino-schema.toml` with `[databases.*]`, `[generator]`, `[types]` sections)
- `--table` flag for selective generation
- Relationship inference via naming heuristics

## Installation

```bash
cargo install neutrino-schema
```

## Usage

```bash
# Quick start — introspect and generate from a database URL
neutrino-schema generate \
    --database-url $DATABASE_URL \
    --output src/entities

# Interactive config file bootstrap
neutrino-schema init

# Generate from an existing config file
neutrino-schema generate

# Inspect schema without generating code
neutrino-schema inspect --database-url $DATABASE_URL

# Export the schema to a versioned JSON file
neutrino-schema export --database-url $DATABASE_URL --pretty -o my_schema.json

# Generate code from the exported JSON (no database connection needed)
neutrino-schema generate --from-ir my_schema.json --output src/entities

# Generate only specific tables
neutrino-schema generate --database-url $DATABASE_URL --table users,orders
```

### Validation

The `validate()` function checks a `SchemaIR` for consistency before code generation. It returns a `ValidationReport` with errors (blocking) and warnings (non-blocking):

```rust
use neutrino_schema::{validate, ValidationLevel};

let report = validate(&schema);
if report.has_errors() {
    for entry in report.errors() {
        eprintln!("[error] {}: {}", entry.location.as_deref().unwrap_or("(global)"), entry.message);
    }
}
```

Detection rules:
- Empty or whitespace-only identifiers — Error
- Duplicate table names — Error
- Duplicate enum Rust names — Error
- Unresolved `EnumRef` references — Error
- Foreign key references to non-existent tables — Error
- Orphan enums (defined but never referenced) — Warning

### Configuration

Create a `neutrino-schema.toml` file (generated by `neutrino-schema init`):

```toml
[generator]
output = "src/entities"

[databases.default]
url = "postgresql://localhost:5432/mydb"

[types]
money = "rust_decimal::Decimal"
my_custom_type = "my_crate::MyType"
```

## Design goals

`neutrino-schema` is designed as a compiler pipeline rather than a runtime ORM. It focuses on generating predictable Rust code from your database definition while keeping the generated output easy to understand and maintain.

## License

MIT OR Apache-2.0
