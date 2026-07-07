# neutrino-schema

[![CI](https://github.com/claimore22/neutrino-schema/actions/workflows/ci.yml/badge.svg)](https://github.com/claimore22/neutrino-schema/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/neutrino-schema.svg?refresh=1)](https://crates.io/crates/neutrino-schema)
[![docs.rs](https://docs.rs/neutrino-schema/badge.svg?refresh=1)](https://docs.rs/neutrino-schema)
[![License](https://img.shields.io/crates/l/neutrino-schema.svg?refresh=1)](https://github.com/claimore22/neutrino-schema)

A schema-to-Rust compiler pipeline for generating strongly typed Rust code from databases.

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL,
    display_name VARCHAR(100)
);
```

```bash
neutrino-schema generate --database-url $DATABASE_URL --output src/models
```

```rust
#[derive(Debug, Clone)]
pub struct Users {
    pub id: i64,
    pub email: String,
    pub display_name: Option<String>,
}
```

## Overview

`neutrino-schema` simplifies Rust application development by converting database schemas into strongly typed Rust structures.

The tool currently introspects PostgreSQL database structures, including tables, columns, data types, nullability, and relationships. The extracted schema is transformed into an intermediate representation (`SchemaIR`) and then used to generate Rust source code.

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

- PostgreSQL schema introspection
- Strongly typed schema representation
- Intermediate representation (`SchemaIR`)
- Automatic Rust struct generation
- Relationship inference
- CLI tooling

## Installation

```bash
cargo install neutrino-schema
```

## Usage

```bash
neutrino-schema generate \
    --database-url $DATABASE_URL \
    --output src/models
```

## Design goals

`neutrino-schema` is designed as a compiler pipeline rather than a runtime ORM. It focuses on generating predictable Rust code from your database definition while keeping the generated output easy to understand and maintain.

## License

MIT OR Apache-2.0
