use std::path::PathBuf;

/// A single file produced by a code generator.
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// Relative path (e.g. `"users.rs"`, `"enums.rs"`, `"mod.rs"`).
    pub path: PathBuf,
    /// File content.
    pub content: String,
    /// If `false`, skip writing when target file already exists.
    pub overwrite: bool,
}

/// The complete output of a code generator.
///
/// Generators return this struct.
/// The CLI is responsible for writing the files via [`OutputWriter`](crate::OutputWriter).
#[derive(Debug, Clone)]
pub struct GeneratedOutput {
    pub files: Vec<GeneratedFile>,
}

impl GeneratedOutput {
    /// Look up a generated file by its relative path.
    pub fn file(&self, path: &str) -> Option<&GeneratedFile> {
        self.files.iter().find(|f| f.path == std::path::Path::new(path))
    }
}
