use std::path::PathBuf;

use crate::codegen::RenderMode;

/// Configuration for the Rust model generator.
///
/// Controls output directory, render mode, and module naming.
/// Can be loaded from a config file or constructed programmatically
/// (e.g., from CLI args or build.rs).
///
/// ## TOML section
///
/// ```toml
/// [generator]
/// output = "src/entities"
/// module_name = "types"
/// render_mode = "clean"
/// ```
#[cfg_attr(feature = "cli", derive(serde::Serialize, serde::Deserialize))]
pub struct GeneratorConfig {
    /// Directory to write generated `.rs` files and `mod.rs` into.
    #[cfg_attr(
        feature = "cli",
        serde(default = "default_output_dir", rename = "output")
    )]
    pub output_dir: PathBuf,

    /// Module name used in the generated `mod.rs` header comment.
    #[cfg_attr(feature = "cli", serde(default = "default_module_name"))]
    pub module_name: String,

    /// Clean (no comments) or Debug (include raw type and nullability comments).
    #[cfg_attr(feature = "cli", serde(default = "default_render_mode"))]
    pub render_mode: RenderMode,
}

/// Default config: output to `./src/entities`, module name `"types"`,
/// [`RenderMode::Clean`](crate::RenderMode).
impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./src/entities"),
            module_name: "types".into(),
            render_mode: RenderMode::Clean,
        }
    }
}

#[doc(hidden)]
pub fn default_output_dir() -> PathBuf {
    PathBuf::from("./src/entities")
}

#[doc(hidden)]
pub fn default_module_name() -> String {
    "types".into()
}

#[doc(hidden)]
pub fn default_render_mode() -> RenderMode {
    RenderMode::Clean
}
