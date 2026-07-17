use std::io::IsTerminal;
use std::path::PathBuf;

use clap::Args;

use crate::ir::RelationStrategy;

/// Export a database schema to SchemaIR JSON format.
///
/// Connects to the database, introspects the public schema, and writes
/// a `.json` file containing the normalized IR.  This file can later be
/// consumed by `generate --from-ir` without a live database connection.
#[derive(Args)]
pub struct ExportCommand {
    /// Database connection string (also read from `DATABASE_URL` env).
    #[arg(long)]
    pub database_url: Option<String>,

    /// Output file path (default: `<database_name>.schema.json`).
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Named database connection from `neutrino-schema.toml` (default: `default`).
    #[arg(long, default_value = "default")]
    pub database: String,

    /// Pretty-print the JSON output.
    #[arg(short, long)]
    pub pretty: bool,

    /// Skip all interactive prompts; fail if a database URL cannot be resolved.
    #[arg(long)]
    pub non_interactive: bool,
}

impl ExportCommand {
    /// Execute the export subcommand.
    pub async fn run(&self) -> anyhow::Result<()> {
        let url = self.resolve_database_url()?;
        let introspector = crate::cli::url_to_introspector(&url).await?;

        eprintln!("Inspecting database...");

        let table_infos = introspector.list_tables_with_info().await?;
        let schema = crate::ir::SchemaIR::from_database(
            introspector.as_ref(),
            &table_infos,
            RelationStrategy::NamingHeuristic,
            crate::config::detect_provider(&url),
            None,
        )
        .await?;

        // Run validation — warn but don't block export
        let report = crate::validation::validate(&schema);
        if report.has_errors() {
            eprintln!("Warning: schema has validation errors:");
            for entry in &report.entries {
                let level = match entry.level {
                    crate::validation::ValidationLevel::Error => "error",
                    crate::validation::ValidationLevel::Warning => "warning",
                };
                let loc = entry
                    .location
                    .as_deref()
                    .unwrap_or("(global)");
                eprintln!("  [{level}] {loc}: {}", entry.message);
            }
        }

        // Determine output path
        let path = match &self.output {
            Some(p) => p.clone(),
            None => {
                let db_name = schema
                    .metadata
                    .database_name
                    .as_deref()
                    .unwrap_or("schema");
                PathBuf::from(format!("{db_name}.schema.json"))
            }
        };

        let file = std::fs::File::create(&path)?;
        schema.write_json_to(file, self.pretty)?;

        eprintln!(
            "✓ Exported {} tables, {} relations, {} enums to {}",
            schema.tables.len(),
            schema.relations.len(),
            schema.enums.len(),
            path.display(),
        );

        Ok(())
    }

    /// Resolve the database URL from CLI flags, environment, or config file.
    fn resolve_database_url(&self) -> anyhow::Result<String> {
        use crate::config::ProjectConfig;

        // 1. CLI flag
        if let Some(url) = &self.database_url {
            return Ok(url.clone());
        }

        // 2. Environment variable
        if let Ok(url) = std::env::var("DATABASE_URL") {
            return Ok(url);
        }

        // 3. Config file
        if let Some(config) = ProjectConfig::load_from_cwd()? {
            if let Some(db) = config.databases.get(&self.database) {
                if let Some(url) = &db.url {
                    return Ok(url.clone());
                }
            }

            let mut msg = format!(
                "Database \"{}\" not found in neutrino-schema.toml.\n",
                self.database,
            );
            let mut keys: Vec<&String> = config.databases.keys().collect();
            keys.sort();
            if keys.is_empty() {
                msg.push_str("\nNo databases configured.\n");
            } else {
                msg.push_str("\nAvailable databases:\n");
                for key in keys {
                    msg.push_str(&format!("  {key}\n"));
                }
            }
            anyhow::bail!("{msg}");
        }

        // 4. Interactive prompt
        if !self.non_interactive && std::io::stdin().is_terminal() {
            return self.prompt_database_url();
        }

        anyhow::bail!(
            "No database URL found.\n\n\
             Pass --database-url, or set the DATABASE_URL environment variable,\n\
             or create a neutrino-schema.toml file."
        )
    }

    fn prompt_database_url(&self) -> anyhow::Result<String> {
        use std::io::{self, Write};
        print!("Database URL: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let url = input.trim().to_string();
        if url.is_empty() {
            anyhow::bail!("Database URL cannot be empty");
        }
        Ok(url)
    }
}
