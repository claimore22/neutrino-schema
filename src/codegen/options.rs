use crate::types::TypeRegistry;

/// Controls whether rendered output includes debug annotations.
///
/// Every generator interprets these modes in its own way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "cli", serde(rename_all = "lowercase"))]
pub enum RenderMode {
    /// Clean output — no extra annotations.
    Clean,
    /// Include type annotations, nullability, or other debug information.
    Debug,
}

/// Generic generation options shared across all generators.
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub render_mode: RenderMode,
    pub rust: RustGeneratorConfig,
}

/// Rust-specific generator configuration.
#[derive(Debug, Clone)]
pub struct RustGeneratorConfig {
    pub module_name: String,
    pub derive_from_row: bool,
    pub type_registry: TypeRegistry,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            render_mode: RenderMode::Clean,
            rust: RustGeneratorConfig::default(),
        }
    }
}

impl Default for RustGeneratorConfig {
    fn default() -> Self {
        Self {
            module_name: "types".into(),
            derive_from_row: false,
            type_registry: TypeRegistry::default(),
        }
    }
}
