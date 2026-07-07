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

/// Collect all columns from the given table names and convert to [`FieldIR`]
/// using the provided introspector.
pub(crate) async fn introspect_tables(
    introspector: &dyn crate::introspect::DatabaseIntrospector,
    table_names: &[String],
) -> anyhow::Result<Vec<crate::ir::TableIR>> {
    let mut tables = Vec::new();
    for name in table_names {
        let columns = introspector.list_columns(name).await?;
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
        tables.push(crate::ir::TableIR {
            name: name.clone(),
            fields,
        });
    }
    Ok(tables)
}

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
