//! Intermediate Representation (IR) — schema-agnostic model for code generation.
//!
//! Types are collected from [`Column`](crate::Column) into [`FieldIR`] → [`TableIR`] → [`SchemaIR`].
//! Optional relation inference produces [`RelationIR`] entries from naming heuristics.

mod field;
mod table;
mod relation;
mod schema;

pub use field::*;
pub use table::*;
pub use relation::*;
pub use schema::*;
