# neutrino-schema

![Rust](https://img.shields.io/badge/rust-1.85+-blue.svg)

neutrino-schema aims to simplify Rust application development by automatically generating Rust types from your database schema. The tool introspects your database structure, including tables, columns, data types, nullability, and relationships, then builds an intermediate schema representation before generating Rust source code.

## How does it work?

neutrino-schema follows a compiler-style pipeline:

**Database introspection** — Connects to your database and analyzes its structure.
**Schema representation** — Converts database metadata into a strongly typed intermediate representation (SchemaIR).
**Code generation** — Generates Rust structs and modules based on your schema.

The generated code is designed to provide a type-safe bridge between your database and Rust applications while keeping the generated models predictable and easy to maintain.

## Pipeline

```
PostgreSQL
   ↓
PostgresIntrospector  (list_tables, list_columns)
   ↓
Column
   ↓
types::PgType → types::DbType  (type normalization)
   ↓
ir::SchemaIR  (FieldIR, TableIR, RelationIR)
   ↓
codegen::generate_files(config)
   ↓
output/*.rs + mod.rs
```

## CLI

Install:

```bash
cargo install neutrino-schema
```

### Generate

Generate model files for all tables:

```bash
neutrino-schema generate \
  --database-url postgres://user:pass@localhost/mydb \
  --output src/models
```

Generate only specific tables:

```bash
neutrino-schema generate \
  --database-url postgres://localhost/mydb \
  --table users \
  --table posts
```

Use `DATABASE_URL` environment variable instead of `--database-url`:

```bash
export DATABASE_URL=postgres://localhost/mydb
neutrino-schema generate
```

Include raw SQL type and nullability comments:

```bash
neutrino-schema generate --database-url postgres://localhost/mydb --debug
```

### Inspect

Print a single table struct to stdout:

```bash
neutrino-schema inspect postgres://localhost/mydb users
```

With type/nullable comments:

```bash
neutrino-schema inspect postgres://localhost/mydb users -c
```

Generate all tables to `./generated/` directory:

```bash
neutrino-schema inspect postgres://localhost/mydb --all
```

List available tables:

```bash
neutrino-schema inspect postgres://localhost/mydb
```

## Library (build.rs or programmatic)

Add to `Cargo.toml`:

```toml
[dependencies]
neutrino-schema = "0.1"
```

Without PostgreSQL support (types and IR only):

```toml
neutrino-schema = { version = "0.1", default-features = false }
```

### In a build.rs

```rust
use neutrino_schema::{
    PostgresIntrospector, SchemaIR, TableIR, FieldIR,
    RelationStrategy, GeneratorConfig, generate_files,
};

#[tokio::main]
async fn main() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect("postgres://localhost/mydb")
        .await
        .unwrap();

    let introspector = PostgresIntrospector::new(pool);
    let table_names = introspector.list_tables().await.unwrap();

    let mut tables = Vec::new();
    for name in &table_names {
        let columns = introspector.list_columns(name).await.unwrap();
        let fields: Vec<FieldIR> = columns
            .iter()
            .map(PostgresIntrospector::column_to_field)
            .collect();
        tables.push(TableIR { name: name.clone(), fields });
    }

    let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

    let config = GeneratorConfig {
        output_dir: "src/models".into(),
        module_name: "models".into(),
        render_mode: neutrino_schema::RenderMode::Clean,
    };

    generate_files(&schema, &config).unwrap();
}
```

### Types and IR only (no database)

```rust
use neutrino_schema::{
    PgType, DbType, to_db_type, dbtype_to_rust,
    FieldIR, TableIR, SchemaIR, RelationStrategy,
    RenderMode, generate_struct,
};

let fields = vec![
    FieldIR {
        name: "id".into(),
        ty: DbType::Int,
        nullable: false,
        raw_type: "integer".into(),
    },
    FieldIR {
        name: "email".into(),
        ty: DbType::String,
        nullable: false,
        raw_type: "varchar".into(),
    },
];
let tables = vec![TableIR { name: "users".into(), fields }];
let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

let output = generate_struct(&schema.tables[0], RenderMode::Clean);
println!("{output}");
```

## Features

| Feature | Default | Description |
|---|---|---|
| `postgres` | yes | PostgreSQL introspection (sqlx) |
| `cli` | yes | CLI subcommands (clap) |

Disable both for a lightweight types-only dependency:

```toml
neutrino-schema = { version = "0.1", default-features = false }
```

## Output example

Generates one `.rs` file per table plus a `mod.rs`:

```
src/models/
├── mod.rs
├── users.rs
├── posts.rs
└── comments.rs
```

`users.rs`:

```rust
#[derive(Debug, Clone)]
pub struct Users {
    pub id: i64,
    pub email: String,
    pub name: Option<String>,
}
```

## Relation inference

The naming heuristic (`RelationStrategy::NamingHeuristic`) scans for columns ending in `_id` and attempts to match a corresponding table by exact name, then by plural (`role_id` → `roles`).

These are best-effort guesses — no foreign key constraints are verified.

Use `RelationStrategy::Disabled` to skip inference.

## License

MIT OR Apache-2.0
