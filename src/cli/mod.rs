//! CLI entry points for the `neutrino-schema` binary.
//!
//! Subcommands:
//! - `inspect` — print generated structs to stdout or dump all tables.
//! - `generate` — write `.rs` model files to a directory.
//! - `export` — write SchemaIR JSON to a file.
//! - `init` — create a `neutrino-schema.toml` config file.

mod export;
mod generate;
mod init;
mod inspect;

use clap::{Parser, Subcommand};
use export::ExportCommand;
use generate::GenerateCommand;
use init::InitCommand;
use inspect::InspectCommand;

// ---------------------------------------------------------------------------
// Shared connect helpers
// ---------------------------------------------------------------------------

/// Create an appropriate introspector by detecting the database provider
/// from the URL scheme.
pub async fn url_to_introspector(
    url: &str,
) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    use crate::config::detect_provider;

    match detect_provider(url) {
        Some(crate::config::DatabaseProvider::Postgres) => connect_postgres(url).await,
        Some(crate::config::DatabaseProvider::MySql) => connect_mysql(url).await,
        Some(crate::config::DatabaseProvider::Sqlite) => connect_sqlite(url).await,
        None => {
            let scheme = url.split(':').next().unwrap_or(url);
            anyhow::bail!(
                "Unsupported database URL scheme: {scheme}\n\n\
                 Supported schemes:\n  postgres://\n  mysql://\n  sqlite:"
            )
        }
    }
}

/// Connect to a PostgreSQL database.
#[cfg(feature = "postgres")]
pub async fn connect_postgres(
    url: &str,
) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await?;
    Ok(Box::new(crate::introspect::PostgresIntrospector::new(pool)))
}

/// Stub — PostgreSQL support not compiled in.
#[cfg(not(feature = "postgres"))]
pub async fn connect_postgres(
    _url: &str,
) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    anyhow::bail!("PostgreSQL support not enabled (enable the `postgres` feature)")
}

/// Connect to a SQLite database.
#[cfg(feature = "sqlite")]
pub async fn connect_sqlite(
    url: &str,
) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    let pool = sqlx::sqlite::SqlitePool::connect(url).await?;
    Ok(Box::new(crate::introspect::SqliteIntrospector::new(pool)))
}

/// Stub — SQLite support not compiled in.
#[cfg(not(feature = "sqlite"))]
pub async fn connect_sqlite(
    _url: &str,
) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    anyhow::bail!("SQLite support not enabled (enable the `sqlite` feature)")
}

/// Connect to a MySQL database.
#[cfg(feature = "mysql")]
pub async fn connect_mysql(
    url: &str,
) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await?;
    Ok(Box::new(crate::introspect::MysqlIntrospector::new(pool)))
}

/// Stub — MySQL support not compiled in.
#[cfg(not(feature = "mysql"))]
pub async fn connect_mysql(
    _url: &str,
) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    anyhow::bail!("MySQL support not enabled (enable the `mysql` feature)")
}

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

/// `neutrino-schema` CLI — schema-to-Rust compiler.
#[derive(Parser)]
#[command(name = "neutrino-schema", about = "Schema-to-Rust compiler pipeline")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Available CLI subcommands.
#[derive(Subcommand)]
pub enum Command {
    /// Inspect a database and print structs to stdout
    Inspect(InspectCommand),
    /// Export SchemaIR JSON to a file
    Export(ExportCommand),
    /// Generate Rust model files from a database schema
    Generate(GenerateCommand),
    /// Create a neutrino-schema.toml configuration file
    Init(InitCommand),
}
