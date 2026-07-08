use std::io::IsTerminal;
use std::path::PathBuf;

use clap::Args;

use crate::codegen::RenderMode;
use crate::config::{DatabaseConfig, GeneratorConfig, ProjectConfig};
use crate::cli::url_to_introspector;

/// Generate Rust model files from a database schema.
///
/// Connects to the database, introspects the public schema, and writes
/// one `.rs` file per table (plus a `mod.rs`) to the output directory.
///
/// When run without arguments, `neutrino-schema generate` looks for a
/// `neutrino-schema.toml` file in the current directory, or prompts
/// interactively for a database URL.
#[derive(Args)]
pub struct GenerateCommand {
    /// Database connection string (also read from `DATABASE_URL` env).
    #[arg(long)]
    pub database_url: Option<String>,

    /// Directory to write generated files into.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Named database connection from `neutrino-schema.toml` (default: `default`).
    #[arg(long, default_value = "default")]
    pub database: String,

    /// Only generate structs for these tables.
    ///
    /// Repeatable (`--table users --table posts`) and supports comma and
    /// semicolon delimiters (`--table users,posts`, `--table "users;posts"`).
    #[arg(long)]
    pub table: Vec<String>,

    /// Include raw type and nullability comments in generated structs.
    #[arg(long)]
    pub debug: bool,

    /// Save the resolved database URL to `neutrino-schema.toml`.
    #[arg(long)]
    pub save: bool,

    /// Skip all interactive prompts; fail if a database URL cannot be resolved.
    #[arg(long)]
    pub non_interactive: bool,

    /// Generate models from all configured databases (not yet implemented).
    #[arg(long)]
    pub all: bool,
}

impl GenerateCommand {
    /// Execute the generate subcommand.
    pub async fn run(&self) -> anyhow::Result<()> {
        use crate::ir::RelationStrategy;

        if self.all {
            anyhow::bail!("--all is not yet implemented");
        }

        let url = self.resolve_database_url()?;
        let introspector = url_to_introspector(&url).await?;

        let provider = crate::config::detect_provider(&url)
            .map(|p| p.display_name().to_string())
            .unwrap_or_else(|| "Database".into());

        eprintln!("Using database \"{}\"", self.database);
        eprintln!("Inspecting {provider}...");

        let table_names = if self.table.is_empty() {
            introspector.list_tables().await?
        } else {
            normalize_table_names(&self.table)
        };

        let tables = crate::cli::introspect_tables(introspector.as_ref(), &table_names).await?;

        let schema = crate::ir::SchemaIR::from_tables(tables, RelationStrategy::NamingHeuristic);

        let output_dir = self
            .output
            .clone()
            .unwrap_or_else(|| PathBuf::from("./src/entities"));

        let config = GeneratorConfig {
            output_dir,
            module_name: "models".into(),
            render_mode: if self.debug { RenderMode::Debug } else { RenderMode::Clean },
        };

        crate::codegen::generate_files(&schema, &config)?;

        eprintln!(
            "✓ Generated {} tables to {:?}",
            schema.tables.len(),
            config.output_dir,
        );

        if !schema.relations.is_empty() {
            eprintln!("  Relations: {} (naming heuristic)", schema.relations.len());
            for r in &schema.relations {
                eprintln!(
                    "    {}.{} → {}.{}",
                    r.from_table, r.from_field, r.to_table, r.to_field
                );
            }
        }

        Ok(())
    }

    /// Resolve the database URL from CLI flags, environment, config file,
    /// or interactive prompt — in that order.
    fn resolve_database_url(&self) -> anyhow::Result<String> {
        // 1. CLI flag
        if let Some(url) = &self.database_url {
            if self.save {
                self.save_url_to_config(url)?;
            }
            return Ok(url.clone());
        }

        // 2. Environment variable
        if let Ok(url) = std::env::var("DATABASE_URL") {
            self.maybe_save_url_to_config(&url)?;
            return Ok(url);
        }

        // 3. Config file
        if let Some(config) = ProjectConfig::load_from_cwd()? {
            if let Some(db) = config.databases.get(&self.database) {
                if let Some(url) = &db.url {
                    return Ok(url.clone());
                }
            }

            // Named database exists but has no URL
            if config.databases.contains_key(&self.database) {
                // Fall through to prompt or error
            } else {
                // Named database doesn't exist — list available
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
                msg.push_str(&format!(
                    "\nCreate it with:\n  neutrino-schema generate \
                     --database {} --database-url <url> --save\n",
                    self.database,
                ));
                anyhow::bail!("{msg}");
            }
        }

        // 4. Interactive prompt (if terminal)
        if !self.non_interactive && std::io::stdin().is_terminal() {
            let url = self.prompt_database_url()?;
            self.save_url_to_config(&url)?;
            eprintln!("✓ Saved to neutrino-schema.toml");
            return Ok(url);
        }

        // 5. Nothing worked
        anyhow::bail!(
            "No database URL found.\n\n\
             Pass --database-url, or set the DATABASE_URL environment variable,\n\
             or create a neutrino-schema.toml file.\n\n\
             Quick start:\n  neutrino-schema init --database-url \"postgres://localhost/mydb\"\n  neutrino-schema generate"
        )
    }

    /// Prompt the user for a database URL via stdin.
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

    /// Save the given URL to `neutrino-schema.toml`, merging with any
    /// existing configuration.
    fn save_url_to_config(&self, url: &str) -> anyhow::Result<()> {
        let mut config = match ProjectConfig::load_from_cwd()? {
            Some(c) => c,
            None => ProjectConfig::default(),
        };

        let provider = crate::config::detect_provider(url);

        let db_entry = config
            .databases
            .entry(self.database.clone())
            .or_insert_with(DatabaseConfig::default);

        db_entry.url = Some(url.to_string());

        // Only set provider if it wasn't already explicitly configured
        if db_entry.provider.is_none() {
            db_entry.provider = provider;
        }

        // Validate provider matches URL
        if let Some(ref configured_provider) = db_entry.provider {
            if let Some(detected_provider) = &provider {
                if configured_provider != detected_provider {
                    anyhow::bail!(
                        "Provider mismatch: configured \"{cp}\" but URL uses \"{dp}\"\n\
                         Remove `provider` from neutrino-schema.toml or correct it.",
                        cp = configured_provider.display_name(),
                        dp = detected_provider.display_name(),
                    );
                }
            }
        }

        config.save_to_cwd()
    }

    /// Ask the user whether to save the environment variable URL to config.
    fn maybe_save_url_to_config(&self, url: &str) -> anyhow::Result<()> {
        if self.non_interactive || !std::io::stdin().is_terminal() {
            return Ok(());
        }

        // Only prompt if no config file exists
        let path = std::env::current_dir()?.join("neutrino-schema.toml");
        if path.exists() {
            return Ok(());
        }

        eprintln!("Using DATABASE_URL from environment.");
        eprintln!("No neutrino-schema.toml found.");

        let answer = self.prompt_yes_no("Save this configuration for future runs? [y/N]")?;
        if answer {
            self.save_url_to_config(url)?;
            eprintln!("✓ Saved to neutrino-schema.toml");
        }

        Ok(())
    }

    /// Prompt for a yes/no answer.
    fn prompt_yes_no(&self, question: &str) -> anyhow::Result<bool> {
        use std::io::{self, Write};
        print!("{question} ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_lowercase();
        Ok(trimmed == "y" || trimmed == "yes")
    }
}

pub(crate) fn normalize_table_names(values: &[String]) -> Vec<String> {
    values
        .iter()
        .flat_map(|s| s.split([',', ';']))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_table_formats() {
        let input = vec![
            "users".to_string(),
            "notifications,orders".to_string(),
            "logs;events".to_string(),
        ];

        let result = normalize_table_names(&input);

        assert_eq!(
            result,
            vec![
                "users".to_string(),
                "notifications".to_string(),
                "orders".to_string(),
                "logs".to_string(),
                "events".to_string(),
            ]
        );
    }

    #[test]
    fn deduplicates_empty_segments() {
        let input = vec!["users,,posts".to_string(), ";".to_string()];
        let result = normalize_table_names(&input);
        assert_eq!(result, vec!["users".to_string(), "posts".to_string()]);
    }

    #[test]
    fn trims_whitespace() {
        let input = vec![" users , posts ".to_string()];
        let result = normalize_table_names(&input);
        assert_eq!(result, vec!["users".to_string(), "posts".to_string()]);
    }
}
