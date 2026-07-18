//! Database introspection — reads live schema metadata.
//!
//! The [`DatabaseIntrospector`] trait abstracts the introspection API.
//! [`PostgresIntrospector`], [`SqliteIntrospector`], and [`MysqlIntrospector`]
//! are the built-in implementations using `sqlx`.

mod column;
mod helpers;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "mysql")]
mod mysql_enum;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;
mod table;
mod traits;

pub use column::*;
pub(crate) use helpers::*;
#[cfg(feature = "mysql")]
pub use mysql::*;
#[cfg(feature = "mysql")]
pub use mysql_enum::*;
#[cfg(feature = "postgres")]
pub use postgres::*;
#[cfg(feature = "sqlite")]
pub use sqlite::*;
pub use table::*;
pub use traits::*;

use crate::ir::{RelationStrategy, SchemaIR, TableIR};

/// Collect all columns from the given [`TableInfo`] entries and convert to [`FieldIR`](crate::ir::FieldIR).
///
/// The table comment from [`TableInfo::comment`] is propagated into [`TableIR::comment`]
/// so that codegen can emit it as a `///` doc comment on the generated struct.
pub async fn introspect_tables(
    introspector: &dyn DatabaseIntrospector,
    table_infos: &[TableInfo],
) -> anyhow::Result<Vec<TableIR>> {
    let mut tables = Vec::new();
    for info in table_infos {
        let columns = introspector.list_columns(&info.name).await?;
        let fields: Vec<_> = columns
            .iter()
            .map(|c| introspector.column_to_field(c))
            .collect();
        let constraints = introspector.list_constraints(&info.name).await?;
        let indexes = introspector.list_indexes(&info.name).await?;
        tables.push(TableIR {
            name: info.name.clone(),
            fields,
            constraints,
            comment: info.comment.clone(),
            indexes,
        });
    }
    Ok(tables)
}

/// Introspect tables and enums, returning a fully resolved [`SchemaIR`].
///
/// After collecting tables and enums, this function post-processes field
/// types to promote database-level enum columns (MySQL `enum(...)`, or
/// any [`DbType::Unknown`](crate::DbType::Unknown) whose raw type name matches a known enum name)
/// to [`DbType::Enum`](crate::DbType::Enum).
pub async fn introspect_schema(
    introspector: &dyn DatabaseIntrospector,
    table_infos: &[TableInfo],
    strategy: RelationStrategy,
) -> anyhow::Result<SchemaIR> {
    use std::collections::HashMap;

    let tables = introspect_tables(introspector, table_infos).await?;
    let enums = introspector.introspect_enums().await?;

    let mut enum_by_raw_name: HashMap<&str, &crate::ir::EnumIR> = HashMap::new();
    for enm in &enums {
        enum_by_raw_name.insert(&enm.database_name, enm);
    }
    let enum_by_rust_name: HashMap<&str, &crate::ir::EnumIR> =
        enums.iter().map(|e| (e.rust_name.as_str(), e)).collect();

    let tables: Vec<TableIR> = tables
        .into_iter()
        .map(|mut table| {
            for field in &mut table.fields {
                if field.raw_type == "enum" {
                    let db_name = format!("{}.{}", table.name, field.name);
                    if let Some(enm) = enum_by_raw_name.get(db_name.as_str()) {
                        field.ty = crate::types::DbType::Enum(crate::types::EnumRef {
                            rust_name: enm.rust_name.clone(),
                        });
                        continue;
                    }
                }
                if let crate::types::DbType::Unknown(name) = &field.ty {
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

    Ok(SchemaIR::with_enums(tables, enums, strategy))
}
