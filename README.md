# neutrino-schema

[![CI](https://github.com/claimore22/neutrino-schema/actions/workflows/ci.yml/badge.svg)](https://github.com/claimore22/neutrino-schema/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/neutrino-schema.svg?refresh=1)](https://crates.io/crates/neutrino-schema)
[![docs.rs](https://docs.rs/neutrino-schema/badge.svg?refresh=1)](https://docs.rs/neutrino-schema)
[![License](https://img.shields.io/crates/l/neutrino-schema.svg?refresh=1)](https://github.com/claimore22/neutrino-schema)
[![sqlx](https://img.shields.io/badge/sqlx-0.9-blue)](https://crates.io/crates/sqlx)

Compile relational database schemas into strongly typed Rust types.

```sql
CREATE TABLE users (
    id         BIGSERIAL    PRIMARY KEY,
    email      VARCHAR(255) NOT NULL,
    full_name  VARCHAR(100)
);
```

```bash
neutrino-schema generate --database-url $DATABASE_URL --output src/types
```

```rust
#[derive(Debug, Clone)]
pub struct Users {
    pub id: i64,
    pub email: String,
    pub display_name: Option<String>,
}
```

## Supported databases

- PostgreSQL
- MySQL / MariaDB
- SQLite

## Overview

`neutrino-schema` simplifies Rust application development by converting database schemas into strongly typed Rust structures.

The tool introspects database structures — PostgreSQL, MySQL/MariaDB, and SQLite — including tables, columns, data types, nullability, and relationships. The extracted schema is transformed into an intermediate representation (`SchemaIR`) and then used to generate Rust source code.

## How does it work?

```
Database
   |
   v
Introspection
   |
   v
SchemaIR
   |
   v
Rust Code Generation
   |
   v
Strongly typed Rust code
```

## Features

- PostgreSQL, MySQL/MariaDB, and SQLite introspection
- Strongly typed schema representation
- Intermediate representation (`SchemaIR`)
- Automatic Rust struct generation
- Relationship inference via naming heuristics
- CLI tooling for `inspect` and `generate`

## Installation

```bash
cargo install neutrino-schema
```

## Usage

```bash
neutrino-schema generate \
    --database-url $DATABASE_URL \
    --output src/types
```

## Design goals

`neutrino-schema` is designed as a compiler pipeline rather than a runtime ORM. It focuses on generating predictable Rust code from your database definition while keeping the generated output easy to understand and maintain.

## License

MIT OR Apache-2.0
