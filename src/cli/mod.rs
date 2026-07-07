//! CLI entry points for the `neutrino-schema` binary.
//!
//! Two subcommands:
//! - `inspect` — print generated structs to stdout or dump all tables.
//! - `generate` — write `.rs` model files to a directory.

mod generate;
mod inspect;

use clap::{Parser, Subcommand};
use generate::GenerateCommand;
use inspect::InspectCommand;

/// `neutrino-schema` CLI — schema-to-Rust compiler.
#[derive(Parser)]
#[command(name = "neutrino-schema", about = "Schema-to-Rust compiler pipeline")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Available CLI subcommands.
#[derive(Subcommand)]
pub enum Command {
    /// Inspect a database and print structs to stdout
    Inspect(InspectCommand),
    /// Generate Rust model files from a database schema
    Generate(GenerateCommand),
}
