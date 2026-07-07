use std::path::PathBuf;

use clap::Args;

use crate::codegen::RenderMode;
use crate::config::GeneratorConfig;
use crate::introspect::DatabaseIntrospector;

/// Generate Rust model files from a database schema.
///
/// Connects to the database, introspects the public schema, and writes
/// one `.rs` file per table (plus a `mod.rs`) to the output directory.
#[derive(Args)]
pub struct GenerateCommand {
    /// Database connection string (also read from `DATABASE_URL` env).
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    /// Directory to write generated files into (default: `./src/models`).
    #[arg(short, long, default_value = "./src/models")]
    pub output: PathBuf,

    /// Only generate structs for these tables (repeatable: --table users --table posts).
    #[arg(long)]
    pub table: Vec<String>,

    /// Include raw type and nullability comments in generated structs.
    #[arg(long)]
    pub debug: bool,
}

impl GenerateCommand {
    /// Execute the generate subcommand.
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection fails, introspection fails,
    /// or files cannot be written to the output directory.
    pub async fn run(&self) -> anyhow::Result<()> {
        use crate::ir::RelationStrategy;

        let introspector = self.connect().await?;

        let table_names = if self.table.is_empty() {
            introspector.list_tables().await?
        } else {
            self.table.clone()
        };

        let tables = crate::cli::introspect_tables(introspector.as_ref(), &table_names).await?;

        let schema = crate::ir::SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

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

    /// Create the appropriate introspector for the database URL.
    #[cfg(feature = "postgres")]
    async fn connect(&self) -> anyhow::Result<Box<dyn DatabaseIntrospector>> {
        if self.database_url.starts_with("sqlite:") {
            return self.connect_sqlite().await;
        }
        if self.database_url.starts_with("mysql:") {
            return self.connect_mysql().await;
        }
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&self.database_url)
            .await?;
        Ok(Box::new(crate::introspect::PostgresIntrospector::new(pool)))
    }

    /// Create the appropriate introspector for the database URL (no Postgres).
    #[cfg(all(not(feature = "postgres"), any(feature = "sqlite", feature = "mysql")))]
    async fn connect(&self) -> anyhow::Result<Box<dyn DatabaseIntrospector>> {
        if self.database_url.starts_with("mysql:") {
            return self.connect_mysql().await;
        }
        self.connect_sqlite().await
    }

    /// Connect to SQLite — returns an error if the `sqlite` feature is disabled.
    #[cfg(feature = "sqlite")]
    async fn connect_sqlite(&self) -> anyhow::Result<Box<dyn DatabaseIntrospector>> {
        let pool = sqlx::sqlite::SqlitePool::connect(&self.database_url).await?;
        Ok(Box::new(crate::introspect::SqliteIntrospector::new(pool)))
    }

    /// Stub — SQLite support not compiled in.
    #[cfg(all(feature = "postgres", not(feature = "sqlite")))]
    async fn connect_sqlite(&self) -> anyhow::Result<Box<dyn DatabaseIntrospector>> {
        anyhow::bail!("SQLite support not enabled (enable the `sqlite` feature)")
    }

    /// Connect to MySQL — returns an error if the `mysql` feature is disabled.
    #[cfg(feature = "mysql")]
    async fn connect_mysql(&self) -> anyhow::Result<Box<dyn DatabaseIntrospector>> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(1)
            .connect(&self.database_url)
            .await?;
        Ok(Box::new(crate::introspect::MysqlIntrospector::new(pool)))
    }

    /// Stub — MySQL support not compiled in.
    #[cfg(all(feature = "postgres", not(feature = "mysql")))]
    async fn connect_mysql(&self) -> anyhow::Result<Box<dyn DatabaseIntrospector>> {
        anyhow::bail!("MySQL support not enabled (enable the `mysql` feature)")
    }
}
