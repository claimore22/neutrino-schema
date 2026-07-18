//! Relation inference engine.
//!
//! Converts raw [`RelationIR`](crate::ir::RelationIR) entries
//! (from database FK constraints or naming heuristics) into
//! application-facing [`SemanticRelationIR`](crate::ir::SemanticRelationIR)
//! with cardinality, relation names, and inverse information.
//!
//! # Pipeline
//!
//! 1. **Many-to-many detection** — finds join tables (composite PK of 2 FKs)
//! 2. **Cardinality inference** — OneToOne (unique FK), ManyToOne (default),
//!    ManyToMany (join table)
//! 3. **Naming** — singularize/pluralize table names for relation names
//!
//! # Example
//!
//! ```rust
//! use neutrino_schema::inference::RelationInferenceEngine;
//!
//! // Given a SchemaIR with relations:
//! // let engine = RelationInferenceEngine::new(&schema);
//! // let semantic_relations = engine.infer();
//! ```

pub mod cardinality;
pub mod engine;
pub mod many_to_many;
pub mod naming;

pub use engine::RelationInferenceEngine;
