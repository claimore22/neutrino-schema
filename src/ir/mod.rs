//! Intermediate Representation (IR) — schema-agnostic model for code generation.
//!
//! Types are collected from [`Column`](crate::Column) into [`FieldIR`] → [`TableIR`] → [`SchemaIR`].
//! Optional relation inference produces [`RelationIR`] entries from naming heuristics
//! or from foreign key constraints ([`ConstraintKind::ForeignKey`]).

mod constraint;
mod enum_def;
mod field;
mod index;
mod metadata;
mod relation;
mod schema;
mod table;

pub use constraint::*;
pub use enum_def::*;
pub use field::*;
pub use index::*;
pub use metadata::*;
pub use relation::*;
pub use schema::SchemaIR;
pub use schema::IR_VERSION;
pub use table::*;
