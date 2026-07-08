use std::collections::HashMap;

use clap::Args;

use crate::config::{DatabaseConfig, ProjectConfig};

/// Create a `neutrino-schema.toml` configuration file.
#[derive(Args)]
pub struct InitCommand {
    /// Database connection URL to pre-fill in the config.
    #[arg(long)]
    pub database_url: Option<String>,

    /// Overwrite an existing `neutrino-schema.toml` without warning.
    #[arg(long)]
    pub force: bool,
}

impl InitCommand {
    /// Execute the init subcommand.
    pub fn run(&self) -> anyhow::Result<()> {
        let path = std::env::current_dir()?.join("neutrino-schema.toml");

        if path.exists() && !self.force {
            anyhow::bail!(
                "neutrino-schema.toml already exists.\n\
                 Use --force to overwrite."
            );
        }

        let mut config = ProjectConfig::default();

        if let Some(url) = &self.database_url {
            let provider = crate::config::detect_provider(url);
            let db = DatabaseConfig {
                url: Some(url.clone()),
                provider,
                output: None,
            };
            config.databases = HashMap::from([("default".into(), db)]);
        }

        config.save_to_cwd()?;

        println!("✓ Created neutrino-schema.toml");

        if self.database_url.is_none() {
            println!(
                "  Edit the [databases.default] section to set your database URL,\n\
                 then run: neutrino-schema generate"
            );
        }

        Ok(())
    }
}
