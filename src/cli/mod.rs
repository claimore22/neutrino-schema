//! CLI entry points for the `neutrino-schema` binary.
//!
//! Three subcommands:
//! - `inspect` — print generated structs to stdout or dump all tables.
//! - `generate` — write `.rs` model files to a directory.
//! - `init` — create a `neutrino-schema.toml` config file.

mod generate;
mod init;
mod inspect;

use clap::{Parser, Subcommand};
use generate::GenerateCommand;
use init::InitCommand;
use inspect::InspectCommand;

/// Collect all columns from the given table names and convert to [`FieldIR`]
/// using the provided introspector.
pub(crate) async fn introspect_tables(
    introspector: &dyn crate::introspect::DatabaseIntrospector,
    table_names: &[String],
) -> anyhow::Result<Vec<crate::ir::TableIR>> {
    let mut tables = Vec::new();
    for name in table_names {
        let columns = introspector.list_columns(name).await?;
        let fields: Vec<_> = columns.iter().map(|c| introspector.column_to_field(c)).collect();
        let constraints = introspector.list_constraints(name).await?;
        tables.push(crate::ir::TableIR {
            name: name.clone(),
            fields,
            constraints,
        });
    }
    Ok(tables)
}

/// Introspect tables and enums, returning a fully resolved [`SchemaIR`].
///
/// After collecting tables and enums, this function post-processes field
/// types to promote database-level enum columns (MySQL `enum(...)`, or
/// any [`DbType::Unknown`] whose raw type name matches a known enum name)
/// to [`DbType::Enum`].
pub(crate) async fn introspect_schema(
    introspector: &dyn crate::introspect::DatabaseIntrospector,
    table_names: &[String],
    strategy: crate::ir::RelationStrategy,
) -> anyhow::Result<crate::ir::SchemaIR> {
    let tables = introspect_tables(introspector, table_names).await?;
    let enums = introspector.introspect_enums().await?;

    // Build a lookup from raw-type -> EnumIR for enum-name resolution.
    let mut enum_by_raw_name: std::collections::HashMap<&str, &crate::ir::EnumIR> =
        std::collections::HashMap::new();
    for enm in &enums {
        // For PostgreSQL: the database_name is the raw type name (e.g. "mood").
        enum_by_raw_name.insert(&enm.database_name, enm);
    }
    // Also index by rust_name for Unknown(string) resolution
    let enum_by_rust_name: std::collections::HashMap<&str, &crate::ir::EnumIR> =
        enums.iter().map(|e| (e.rust_name.as_str(), e)).collect();

    // Post-process tables: promote matching types to DbType::Enum
    let tables: Vec<crate::ir::TableIR> = tables
        .into_iter()
        .map(|mut table| {
            for field in &mut table.fields {
                // MySQL: raw_type is "enum" — look up by table.column database_name
                if field.raw_type == "enum" {
                    let db_name = format!("{}.{}", table.name, field.name);
                    if let Some(enm) = enum_by_raw_name.get(db_name.as_str()) {
                        field.ty = crate::types::DbType::Enum(crate::types::EnumRef {
                            rust_name: enm.rust_name.clone(),
                        });
                        continue;
                    }
                }
                // PostgreSQL: DbType::Unknown(name) — match by raw type name
                if let crate::types::DbType::Unknown(name) = &field.ty {
                    // Try matching by database_name first, then rust_name
                    let matched = enum_by_raw_name
                        .get(name.as_str())
                        .or_else(|| enum_by_rust_name.get(name.as_str()));
                    if let Some(enm) = matched {
                        field.ty = crate::types::DbType::Enum(crate::types::EnumRef {
                            rust_name: enm.rust_name.clone(),
                        });
                    }
                }
            }
            table
        })
        .collect();

    Ok(crate::ir::SchemaIR::with_enums(tables, enums, strategy))
}

// ---------------------------------------------------------------------------
// Shared connect helpers
// ---------------------------------------------------------------------------

/// Create an appropriate introspector by detecting the database provider
/// from the URL scheme.
pub async fn url_to_introspector(url: &str) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
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
pub async fn connect_postgres(url: &str) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await?;
    Ok(Box::new(crate::introspect::PostgresIntrospector::new(pool)))
}

/// Stub — PostgreSQL support not compiled in.
#[cfg(not(feature = "postgres"))]
pub async fn connect_postgres(_url: &str) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    anyhow::bail!("PostgreSQL support not enabled (enable the `postgres` feature)")
}

/// Connect to a SQLite database.
#[cfg(feature = "sqlite")]
pub async fn connect_sqlite(url: &str) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    let pool = sqlx::sqlite::SqlitePool::connect(url).await?;
    Ok(Box::new(crate::introspect::SqliteIntrospector::new(pool)))
}

/// Stub — SQLite support not compiled in.
#[cfg(not(feature = "sqlite"))]
pub async fn connect_sqlite(_url: &str) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    anyhow::bail!("SQLite support not enabled (enable the `sqlite` feature)")
}

/// Connect to a MySQL database.
#[cfg(feature = "mysql")]
pub async fn connect_mysql(url: &str) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await?;
    Ok(Box::new(crate::introspect::MysqlIntrospector::new(pool)))
}

/// Stub — MySQL support not compiled in.
#[cfg(not(feature = "mysql"))]
pub async fn connect_mysql(_url: &str) -> anyhow::Result<Box<dyn crate::introspect::DatabaseIntrospector>> {
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
    /// Generate Rust model files from a database schema
    Generate(GenerateCommand),
    /// Create a neutrino-schema.toml configuration file
    Init(InitCommand),
}
