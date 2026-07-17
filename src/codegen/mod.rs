//! Rust source code generation from [`SchemaIR`](crate::SchemaIR).
//!
//! The main entry point is [`generate()`](crate::generate) which produces a
//! [`GeneratedOutput`](crate::GeneratedOutput) that the CLI writes via
//! [`OutputWriter`](crate::OutputWriter).

mod options;
mod output;
pub mod rust;
mod writer;

pub use options::*;
pub use output::*;
pub use writer::*;
pub use rust::{
    generate, generate_enum_defs, generate_files, generate_files_with_registry,
    generate_imports, generate_struct,
};
