//! Rust source code generation from [`SchemaIR`](crate::SchemaIR).
//!
//! [`generate_struct`] renders a single table as a `pub struct`.
//! [`generate_files`] writes one `.rs` file per table plus a `mod.rs`.

mod rust;

pub use rust::*;
