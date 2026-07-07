use std::path::PathBuf;

use clap::Args;

use crate::codegen::RenderMode;
use crate::config::GeneratorConfig;

#[derive(Args)]
pub struct GenerateCommand {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    #[arg(short, long, default_value = "./src/models")]
    pub output: PathBuf,

    /// Only generate structs for these tables (repeatable: --table users --table posts).
    #[arg(long)]
    pub table: Vec<String>,

    #[arg(long)]
    pub debug: bool,
}

impl GenerateCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        use crate::ir::{RelationStrategy, SchemaIR, TableIR};
        use crate::introspect::{DatabaseIntrospector, PostgresIntrospector};

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&self.database_url)
            .await?;

        let introspector = PostgresIntrospector::new(pool);

        let table_names = if self.table.is_empty() {
            introspector.list_tables().await?
        } else {
            self.table.clone()
        };

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

        let config = GeneratorConfig {
            output_dir: self.output.clone(),
            module_name: "models".into(),
            render_mode: if self.debug { RenderMode::Debug } else { RenderMode::Clean },
        };

        crate::codegen::generate_files(&schema, &config)?;

        println!("Generated {} tables to {:?}", schema.tables.len(), config.output_dir);
        println!("Potential relations: {}", schema.relations.len());
        println!("Strategy: Naming heuristic");
        println!("Verification: None (database foreign keys were not consulted)");
        for r in &schema.relations {
            println!("  {}.{} -> {}.{}", r.from_table, r.from_field, r.to_table, r.to_field);
        }

        Ok(())
    }
}
