# neutrino-schema

Schema-to-Rust compiler pipeline.

Features:
- PostgreSQL introspection
- Database type normalization
- SchemaIR generation
- Rust model generation support

Pipeline:

PostgreSQL
 ↓
Introspection
 ↓
SchemaIR
 ↓
Rust code