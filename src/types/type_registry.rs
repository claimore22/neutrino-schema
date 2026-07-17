use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::types::DbType;

/// The resolved Rust name and required imports for a [`DbType`].
///
/// Returned by [`TypeRegistry::resolve`] and consumed by the code generator
/// to emit correct type paths and `use` statements.
#[derive(Debug, Clone)]
pub struct RustType {
    /// The Rust type expression (e.g. `"Decimal"`, `"Vec<String>"`).
    pub name: String,
    /// Import statements needed to use the type (e.g. `["use rust_decimal::Decimal;"]`).
    pub imports: Vec<String>,
}

/// Resolves [`DbType`] variants to [`RustType`] values, with feature-gated
/// defaults and user-configurable overrides.
///
/// Each `DbType` variant maps to a Rust type path and optional imports.
/// Unknown types are mapped to `String` with a one-time warning per type name.
///
/// # Example
///
/// ```rust
/// use neutrino_schema::{DbType, TypeRegistry};
///
/// let registry = TypeRegistry::default();
/// let rt = registry.resolve(&DbType::Decimal);
/// assert_eq!(rt.name, "rust_decimal::Decimal");
/// ```
#[derive(Debug)]
pub struct TypeRegistry {
    overrides: HashMap<String, RustType>,
    warned_unknown: Mutex<HashSet<String>>,
}

impl Clone for TypeRegistry {
    fn clone(&self) -> Self {
        Self {
            overrides: self.overrides.clone(),
            warned_unknown: Mutex::new(
                self.warned_unknown
                    .lock()
                    .expect("poisoned lock")
                    .clone(),
            ),
        }
    }
}

impl TypeRegistry {
    /// Create a new registry with default feature-gated mappings.
    pub fn new() -> Self {
        Self {
            overrides: HashMap::new(),
            warned_unknown: Mutex::new(HashSet::new()),
        }
    }

    /// Create a registry with user-specified type overrides.
    ///
    /// The `overrides` map is keyed by the database type name (e.g. `"citext"`)
    /// or the DbType variant name (e.g. `"Decimal"`).  Values are any valid
    /// Rust type path (e.g. `"bigdecimal::BigDecimal"`).
    ///
    /// Overrides take precedence over default mappings.
    pub fn with_overrides(overrides: HashMap<String, String>) -> Self {
        let mut registry = Self::new();
        for (key, type_path) in overrides {
            registry.overrides.insert(
                key,
                RustType {
                    name: type_path,
                    imports: Vec::new(),
                },
            );
        }
        registry
    }

    /// Resolve a [`DbType`] to its [`RustType`].
    ///
    /// Checks user overrides first, then falls back to the default
    /// feature-gated mapping.  Unknown types produce a one-time `eprintln!`
    /// warning and map to `String`.
    pub fn resolve(&self, ty: &DbType) -> RustType {
        // Check user overrides by DbType debug name
        let key = format!("{ty:?}");
        if let Some(rt) = self.overrides.get(&key) {
            return rt.clone();
        }
        // Check user overrides for Unknown type database_name
        if let DbType::Unknown(db_type) = ty {
            if let Some(rt) = self.overrides.get(db_type.as_str()) {
                return rt.clone();
            }
        }

        let default = self.default_for(ty);
        // Warn once per unknown type
        if let DbType::Unknown(db_type) = ty {
            let mut warned = self.warned_unknown.lock().expect("poisoned lock");
            if warned.insert(db_type.clone()) {
                eprintln!("warning: unsupported database type '{db_type}', mapped to String");
            }
        }
        default
    }

    /// The default feature-gated mapping for a given DbType.
    fn default_for(&self, ty: &DbType) -> RustType {
        match ty {
            // Numeric — no deps needed
            DbType::SmallInt => RustType::bare("i16"),
            DbType::Integer => RustType::bare("i32"),
            DbType::BigInt => RustType::bare("i64"),
            DbType::SmallSerial => RustType::bare("i16"),
            DbType::Serial => RustType::bare("i32"),
            DbType::BigSerial => RustType::bare("i64"),
            DbType::Float32 => RustType::bare("f32"),
            DbType::Float64 => RustType::bare("f64"),

            // Decimal — requires rust_decimal crate
            DbType::Decimal => {
                if cfg!(feature = "decimal") {
                    RustType::with_import("rust_decimal::Decimal", "use rust_decimal::Decimal;")
                } else {
                    RustType::bare("String")
                }
            }

            // Text
            DbType::String => RustType::bare("String"),
            DbType::Text => RustType::bare("String"),

            // Boolean
            DbType::Boolean => RustType::bare("bool"),

            // Binary
            DbType::Binary => RustType::bare("Vec<u8>"),

            // Date/time — requires chrono crate
            DbType::Date => {
                if cfg!(feature = "chrono") {
                    RustType::with_import("chrono::NaiveDate", "use chrono::NaiveDate;")
                } else {
                    RustType::bare("String")
                }
            }
            DbType::Time => {
                if cfg!(feature = "chrono") {
                    RustType::with_import("chrono::NaiveTime", "use chrono::NaiveTime;")
                } else {
                    RustType::bare("String")
                }
            }
            DbType::Timestamp => {
                if cfg!(feature = "chrono") {
                    RustType::with_import("chrono::NaiveDateTime", "use chrono::NaiveDateTime;")
                } else {
                    RustType::bare("String")
                }
            }
            DbType::TimestampTz => {
                if cfg!(feature = "chrono") {
                    RustType::with_imports(
                        "chrono::DateTime<chrono::Utc>",
                        vec!["use chrono::{DateTime, Utc};".into()],
                    )
                } else {
                    RustType::bare("String")
                }
            }

            // Structured
            DbType::Json => RustType::with_import("serde_json::Value", "use serde_json::Value;"),
            DbType::Jsonb => RustType::with_import("serde_json::Value", "use serde_json::Value;"),

            // UUID — requires uuid crate
            DbType::Uuid => {
                if cfg!(feature = "uuid") {
                    RustType::with_import("uuid::Uuid", "use uuid::Uuid;")
                } else {
                    RustType::bare("String")
                }
            }

            DbType::Inet => RustType::bare("std::net::IpAddr"),

            // Collections
            DbType::Array(inner) => {
                let inner_rt = self.resolve(inner.as_ref());
                let name = format!("Vec<{}>", inner_rt.name);
                let mut imports = inner_rt.imports;
                imports.sort();
                imports.dedup();
                RustType { name, imports }
            }

            // Schema references
            DbType::Enum(enm) => RustType::bare(enm.rust_name.clone()),

            // Escape hatch
            DbType::Unknown(_) => RustType::bare("String"),
        }
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// RustType construction helpers
// ---------------------------------------------------------------------------

impl RustType {
    fn bare(name: impl Into<String>) -> Self {
        RustType {
            name: name.into(),
            imports: Vec::new(),
        }
    }

    fn with_import(name: impl Into<String>, import: impl Into<String>) -> Self {
        RustType {
            name: name.into(),
            imports: vec![import.into()],
        }
    }

    fn with_imports(name: impl Into<String>, imports: Vec<String>) -> Self {
        RustType {
            name: name.into(),
            imports,
        }
    }
}
