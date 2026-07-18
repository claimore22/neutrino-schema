use std::io::IsTerminal;
use std::path::PathBuf;

use clap::Args;

use crate::cli::url_to_introspector;
use crate::config::{DatabaseConfig, GeneratorConfig, ProjectConfig};
use crate::{GenerateOptions, OutputWriter, RenderMode, RustGeneratorConfig};

/// Generate Rust model files from a database schema or SchemaIR JSON.
///
/// Connects to a live database (default) or loads a previously exported
/// SchemaIR JSON file via `--from-ir`.
///
/// When run without arguments, `neutrino-schema generate` looks for a
/// `neutrino-schema.toml` file in the current directory, or prompts
/// interactively for a database URL.
#[derive(Args)]
pub struct GenerateCommand {
    /// Path to a SchemaIR JSON file (exported via `neutrino-schema export`).
    ///
    /// When set, generation happens without a live database connection.
    /// Mutually exclusive with --database-url.
    #[arg(long, alias = "from-json", value_name = "FILE")]
    pub from_ir: Option<PathBuf>,

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

    /// Generate types from all configured databases (not yet implemented).
    #[arg(long)]
    pub all: bool,
}

impl GenerateCommand {
    /// Execute the generate subcommand.
    pub async fn run(&self) -> anyhow::Result<()> {
        if self.all {
            anyhow::bail!("--all is not yet implemented");
        }

        let schema = if let Some(path) = &self.from_ir {
            self.run_from_ir(path)?
        } else {
            self.run_from_database().await?
        };

        // Build type registry and generator config from config file,
        // then apply CLI overrides (CLI > config > default).
        let (mut config, registry) = match crate::config::ProjectConfig::load_from_cwd()? {
            Some(cfg) => (
                cfg.generator,
                crate::types::TypeRegistry::with_overrides(cfg.types),
            ),
            None => (
                GeneratorConfig::default(),
                crate::types::TypeRegistry::default(),
            ),
        };

        if let Some(ref output) = self.output {
            config.output_dir = output.clone();
        }
        if self.debug {
            config.render_mode = RenderMode::Debug;
        }

        let options = GenerateOptions {
            render_mode: config.render_mode,
            rust: RustGeneratorConfig {
                module_name: config.module_name.clone(),
                derive_from_row: false,
                type_registry: registry,
            },
        };
        let output = crate::codegen::generate(&schema, &options);
        OutputWriter::write(&output, &config.output_dir)?;

        eprintln!(
            "✓ Generated {} tables to {:?}",
            schema.tables.len(),
            config.output_dir,
        );

        if !schema.relations.is_empty() {
            eprintln!("  Relations: {} (naming heuristic)", schema.relations.len());
            for r in &schema.relations {
                let from_cols = r.from_columns.join(", ");
                let to_cols = r.to_columns.join(", ");
                eprintln!(
                    "    {}.({}) → {}.({})",
                    r.from_table, from_cols, r.to_table, to_cols
                );
            }
        }

        Ok(())
    }

    /// Load and validate a SchemaIR from a JSON file.
    fn run_from_ir(&self, path: &PathBuf) -> anyhow::Result<crate::ir::SchemaIR> {
        let text = std::fs::read_to_string(path)?;
        let schema = crate::ir::SchemaIR::from_json_str(&text)
            .map_err(|e| anyhow::anyhow!("Failed to parse SchemaIR JSON: {e}"))?;

        let report = crate::validation::validate(&schema);
        if report.has_errors() {
            eprintln!("Warning: loaded SchemaIR has validation errors:");
            for entry in &report.entries {
                let level = match entry.level {
                    crate::validation::ValidationLevel::Error => "error",
                    crate::validation::ValidationLevel::Warning => "warning",
                };
                let loc = entry.location.as_deref().unwrap_or("(global)");
                eprintln!("  [{level}] {loc}: {}", entry.message);
            }
        }

        eprintln!(
            "Loaded IR: {} tables, {} relations, {} enums",
            schema.tables.len(),
            schema.relations.len(),
            schema.enums.len(),
        );

        Ok(schema)
    }

    /// Introspect a live database and build a SchemaIR.
    async fn run_from_database(&self) -> anyhow::Result<crate::ir::SchemaIR> {
        use crate::ir::RelationStrategy;

        let url = self.resolve_database_url()?;
        let introspector = url_to_introspector(&url).await?;

        let provider = crate::config::detect_provider(&url)
            .map(|p| p.display_name().to_string())
            .unwrap_or_else(|| "Database".into());

        eprintln!("Using database \"{}\"", self.database);
        eprintln!("Inspecting {provider}...");

        let table_infos = if self.table.is_empty() {
            introspector.list_tables_with_info().await?
        } else {
            let all_tables = introspector.list_tables_with_info().await?;
            let existing: std::collections::HashSet<&str> =
                all_tables.iter().map(|t| t.name.as_str()).collect();
            let names = normalize_table_names(&self.table);
            for name in &names {
                if !existing.contains(name.as_str()) {
                    anyhow::bail!("Table \"{name}\" not found in database");
                }
            }
            names
                .into_iter()
                .map(|name| crate::introspect::TableInfo {
                    name,
                    comment: None,
                })
                .collect::<Vec<_>>()
        };

        crate::introspect::introspect_schema(
            introspector.as_ref(),
            &table_infos,
            RelationStrategy::NamingHeuristic,
        )
        .await
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
