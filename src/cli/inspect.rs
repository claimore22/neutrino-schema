use clap::Args;

use crate::codegen::{generate_struct, RenderMode};
use crate::config::GeneratorConfig;
use crate::ir::{RelationStrategy, SchemaIR, TableIR};
use crate::introspect::{DatabaseIntrospector, PostgresIntrospector};

/// Inspect a database and print generated structs.
///
/// With a table name, prints a single struct.  With `--all`, writes all
/// tables to a `generated/` directory and prints relation info.
#[derive(Args)]
pub struct InspectCommand {
    /// PostgreSQL connection string (e.g. `postgres://user:pass@localhost/db`).
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
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&self.database_url)
            .await?;

        let introspector = PostgresIntrospector::new(pool);

        let mode = if self.comments {
            RenderMode::Debug
        } else {
            RenderMode::Clean
        };

        if self.all {
            let table_names = introspector.list_tables().await?;
            let mut tables = Vec::new();

            for name in &table_names {
                let columns = introspector.list_columns(name).await?;
                let fields = columns.iter().map(PostgresIntrospector::column_to_field).collect();
                tables.push(TableIR {
                    name: name.clone(),
                    fields,
                });
            }

            let schema = SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);
            let cfg = GeneratorConfig {
                output_dir: "generated".into(),
                module_name: "models".into(),
                render_mode: mode,
            };
            crate::codegen::generate_files(&schema, &cfg)?;

            println!("Generated {} tables to {:?}", schema.tables.len(), cfg.output_dir);

            println!("Potential relations: {}", schema.relations.len());
            println!("Strategy: Naming heuristic");
            println!("Verification: None (database foreign keys were not consulted)");
            for r in &schema.relations {
                println!("  {}.{} -> {}.{}", r.from_table, r.from_field, r.to_table, r.to_field);
            }
        } else if let Some(table_name) = &self.table {
            let columns = introspector.list_columns(table_name).await?;
            let fields = columns.iter().map(PostgresIntrospector::column_to_field).collect();
            let table_ir = TableIR {
                name: table_name.clone(),
                fields,
            };
            print!("{}", generate_struct(&table_ir, mode));
        } else {
            let tables = introspector.list_tables().await?;
            println!("Tables:");
            for t in tables {
                println!("  - {t}");
            }
            println!();
            println!("Usage:");
            println!("  neutrino-schema inspect <database_url> <table>      Generate struct for one table");
            println!("  neutrino-schema inspect <database_url> <table> -c   Include type/nullable comments");
            println!("  neutrino-schema inspect <database_url> --all         Generate all tables to generated/");
            println!("  neutrino-schema inspect <database_url> --all -c      All tables with debug comments");
        }

        Ok(())
    }
}
