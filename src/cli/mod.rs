mod inspect;

use clap::{Parser, Subcommand};
use inspect::InspectCommand;

#[derive(Parser)]
#[command(name = "neutrino-schema", about = "Schema-to-Rust compiler pipeline")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Inspect a database and generate Rust structs
    Inspect(InspectCommand),
}
