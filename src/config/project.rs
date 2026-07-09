use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::GeneratorConfig;

/// Top-level configuration that maps directly to `neutrino-schema.toml`.
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectConfig {
    /// Config file version (currently `1`).
    #[cfg_attr(feature = "cli", serde(default = "default_version"))]
    pub version: u32,

    /// Named database connections.
    #[cfg_attr(feature = "cli", serde(default))]
    pub databases: HashMap<String, DatabaseConfig>,

    /// Global generator settings (can be overridden per database).
    #[cfg_attr(feature = "cli", serde(default))]
    pub generator: GeneratorConfig,

    /// User-defined type overrides.
    ///
    /// Keyed by database type name (e.g. `"citext"`) or DbType variant name
    /// (e.g. `"Decimal"`).  Values are Rust type paths (e.g. `"bigdecimal::BigDecimal"`).
    ///
    /// ```toml
    /// [types]
    /// money = "rust_decimal::Decimal"
    /// citext = "String"
    /// ```
    #[cfg_attr(feature = "cli", serde(default))]
    pub types: HashMap<String, String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            version: 1,
            databases: HashMap::new(),
            generator: GeneratorConfig::default(),
            types: HashMap::new(),
        }
    }
}

/// Configuration for a single named database.
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
pub struct DatabaseConfig {
    /// Connection URL (e.g. `postgres://localhost/mydb`, `mysql://...`, `sqlite:./db`).
    #[cfg_attr(feature = "cli", serde(default))]
    pub url: Option<String>,

    /// Explicit provider hint (inferred from URL scheme if absent).
    #[cfg_attr(feature = "cli", serde(default))]
    pub provider: Option<DatabaseProvider>,

    /// Per-database output directory override.
    #[cfg_attr(feature = "cli", serde(default))]
    pub output: Option<PathBuf>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: None,
            provider: None,
            output: None,
        }
    }
}

/// Supported database providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "cli", serde(rename_all = "lowercase"))]
pub enum DatabaseProvider {
    /// PostgreSQL
    Postgres,
    /// MySQL or MariaDB
    MySql,
    /// SQLite
    Sqlite,
}

impl DatabaseProvider {
    /// Return a human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Postgres => "PostgreSQL",
            Self::MySql => "MySQL",
            Self::Sqlite => "SQLite",
        }
    }
}

/// Infer the database provider from a URL's scheme.
pub fn detect_provider(url: &str) -> Option<DatabaseProvider> {
    let scheme = url.split(':').next()?.to_lowercase();
    match scheme.as_str() {
        "postgres" | "postgresql" => Some(DatabaseProvider::Postgres),
        "mysql" | "mariadb" => Some(DatabaseProvider::MySql),
        "sqlite" => Some(DatabaseProvider::Sqlite),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// File I/O — only compiled when the `cli` feature is active
// ---------------------------------------------------------------------------

#[cfg(feature = "cli")]
impl ProjectConfig {
    /// Load `ProjectConfig` from `./neutrino-schema.toml` in the current
    /// working directory.
    ///
    /// Returns `Ok(None)` if the file does not exist.
    pub fn load_from_cwd() -> anyhow::Result<Option<Self>> {
        let path = std::env::current_dir()?.join("neutrino-schema.toml");
        if !path.exists() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&text)?;
        Ok(Some(config))
    }

    /// Write this config to `./neutrino-schema.toml`.
    pub fn save_to_cwd(&self) -> anyhow::Result<()> {
        let path = std::env::current_dir()?.join("neutrino-schema.toml");
        let text = toml::to_string_pretty(self)?;
        std::fs::write(&path, text)?;
        Ok(())
    }
}

#[cfg(feature = "cli")]
fn default_version() -> u32 {
    1
}
