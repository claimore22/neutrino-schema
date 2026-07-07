use std::path::Path;

use clap::Args;

use crate::codegen::{generate_files, generate_struct, RenderMode};
use crate::ir::{RelationStrategy, SchemaIR, TableIR};
use crate::introspect::{DatabaseIntrospector, PostgresIntrospector};

#[derive(Args)]
pub struct InspectCommand {
    pub database_url: String,
    pub table: Option<String>,

    #[arg(long)]
    pub all: bool,

    #[arg(short, long)]
    pub comments: bool,
}

impl InspectCommand {
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
            let output_dir = Path::new("generated");
            generate_files(&schema, output_dir, mode)?;

            println!("Generated {} tables to {:?}", schema.tables.len(), output_dir);

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
