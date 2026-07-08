//! Configuration for the code generation pipeline.
//!
//! [`GeneratorConfig`] controls output directory, module naming, and
//! render mode.  It is consumed by [`generate_files`](crate::generate_files).
//!
//! When the `cli` feature is enabled, [`ProjectConfig`] provides the
//! `neutrino-schema.toml` file format with named database connections.

mod generator;
mod project;

pub use generator::*;
pub use project::*;
