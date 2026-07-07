use std::path::PathBuf;

use crate::codegen::RenderMode;

/// Configuration for the Rust model generator.
///
/// Controls output directory, render mode, and module naming.
/// Can be loaded from a config file or constructed programmatically
/// (e.g., from CLI args or build.rs).
pub struct GeneratorConfig {
    /// Directory to write generated `.rs` files and `mod.rs` into.
    pub output_dir: PathBuf,

    /// Module name used in the generated `mod.rs` header comment.
    pub module_name: String,

    /// Clean (no comments) or Debug (include raw type and nullability comments).
    pub render_mode: RenderMode,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./src/models"),
            module_name: "models".into(),
            render_mode: RenderMode::Clean,
        }
    }
}
