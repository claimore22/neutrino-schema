//! Intermediate Representation (IR) — schema-agnostic model for code generation.
//!
//! Types are collected from [`Column`](crate::Column) into [`FieldIR`] → [`TableIR`] → [`SchemaIR`].
//! Optional relation inference produces [`RelationIR`] entries from naming heuristics
//! or from foreign key constraints ([`ConstraintKind::ForeignKey`]).

mod constraint;
mod field;
mod relation;
mod schema;
mod table;
mod enum_def;
mod index;

pub use constraint::*;
pub use field::*;
pub use relation::*;
pub use schema::*;
pub use table::*;
pub use enum_def::*;
pub use index::*;