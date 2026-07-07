//! Configuration for the code generation pipeline.
//!
//! [`GeneratorConfig`] controls output directory, module naming, and
//! render mode.  It is consumed by [`generate_files`](crate::generate_files).

mod generator;

pub use generator::*;
