mod generate;
mod inspect;

use clap::{Parser, Subcommand};
use generate::GenerateCommand;
use inspect::InspectCommand;

#[derive(Parser)]
#[command(name = "neutrino-schema", about = "Schema-to-Rust compiler pipeline")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Inspect a database and print structs to stdout
    Inspect(InspectCommand),
    /// Generate Rust model files from a database schema
    Generate(GenerateCommand),
}
