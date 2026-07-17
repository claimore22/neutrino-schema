use clap::Args;

use crate::cli::url_to_introspector;
use crate::introspect::TableInfo;
use crate::ir::RelationStrategy;
use crate::{GenerateOptions, OutputWriter, RenderMode, SchemaIR};

/// Inspect a database and print generated structs.
///
/// With a table name, prints a single struct.  With `--all`, writes all
/// tables to a `generated/` directory and prints relation info.
#[derive(Args)]
pub struct InspectCommand {
    /// Database connection string (e.g. `postgres://user:pass@localhost/db`
    /// or `sqlite:./dev.db`).
    pub database_url: String,

    /// Name of a single table to inspect (omit or use `--all` for all tables).
    pub table: Option<String>,

    /// Inspect all tables in the public schema.
    #[arg(long)]
    pub all: bool,

    /// Include raw type and nullability annotations in output.
    #[arg(short, long)]
    pub comments: bool,
}

impl InspectCommand {
    /// Execute the inspect subcommand.
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection fails or introspection queries fail.
    pub async fn run(&self) -> anyhow::Result<()> {
        let introspector = url_to_introspector(&self.database_url).await?;

        let mode = if self.comments {
            RenderMode::Debug
        } else {
            RenderMode::Clean
        };

        if self.all {
            let table_infos: Vec<TableInfo> = introspector.list_tables_with_info().await?;
            let schema: SchemaIR = crate::introspect::introspect_schema(
                introspector.as_ref(),
                &table_infos,
                RelationStrategy::NamingHeuristic,
            )
            .await?;
            let options = GenerateOptions {
                render_mode: mode,
                ..Default::default()
            };
            let output = crate::codegen::generate(&schema, &options);
            OutputWriter::write(&output, std::path::Path::new("generated"))?;

            eprintln!(
                "Generated {} tables to {:?}",
                schema.tables.len(),
                "generated",
            );
            eprintln!("Relations: {}", schema.relations.len());
            for r in &schema.relations {
                let source = match &r.origin {
                    crate::ir::RelationOrigin::ForeignKey => " (FK)".to_string(),
                    crate::ir::RelationOrigin::Inferred => " (heuristic)".to_string(),
                };
                let from_cols = r.from_columns.join(", ");
                let to_cols = r.to_columns.join(", ");
                eprintln!(
                    "  {}.({}) -> {}.({}){source}",
                    r.from_table, from_cols, r.to_table, to_cols
                );
            }
            let constraint_count: usize = schema.tables.iter().map(|t| t.constraints.len()).sum();
            eprintln!("Constraints: {}", constraint_count);
        } else if let Some(table_name) = &self.table {
            let columns = introspector.list_columns(table_name).await?;
            let fields: Vec<_> = columns
                .iter()
                .map(|c| introspector.column_to_field(c))
                .collect();
            let constraints = introspector.list_constraints(table_name).await?;
            // Single table: need to look up the comment separately
            let all_infos = introspector.list_tables_with_info().await?;
            let comment = all_infos
                .iter()
                .find(|ti| ti.name == *table_name)
                .and_then(|ti| ti.comment.clone());
            let indexes = introspector.list_indexes(table_name).await?;
            let table_ir = crate::ir::TableIR {
                name: table_name.clone(),
                fields,
                constraints,
                comment,
                indexes,
            };
            let options = GenerateOptions {
                render_mode: mode,
                ..Default::default()
            };
            print!("{}", crate::codegen::generate_struct(&table_ir, &options));
        } else {
            let table_infos = introspector.list_tables_with_info().await?;
            println!("Tables:");
            for ti in table_infos {
                println!("  - {}", ti.name);
                if let Some(comment) = &ti.comment {
                    println!("    {}", comment);
                }
            }
            println!();
            println!("Usage:");
            println!(
                "  neutrino-schema inspect <database_url> <table>      Generate struct for one table"
            );
            println!(
                "  neutrino-schema inspect <database_url> <table> -c   Include type/nullable comments"
            );
            println!(
                "  neutrino-schema inspect <database_url> --all         Generate all tables to generated/"
            );
            println!(
                "  neutrino-schema inspect <database_url> --all -c      All tables with debug comments"
            );
        }

        Ok(())
    }
}
