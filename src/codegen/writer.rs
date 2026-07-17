use std::path::Path;

use crate::GeneratedOutput;

/// Writes [`GeneratedOutput`] to the filesystem.
///
/// This is the only bridge between the generator layer and the filesystem.
/// The CLI calls this, never a generator.
pub struct OutputWriter;

impl OutputWriter {
    /// Write all files in `output` into `dir`.
    ///
    /// Creates `dir` and any intermediate directories if they do not exist.
    /// Respects [`GeneratedFile::overwrite`] — when `false`, existing files
    /// are skipped.
    pub fn write(output: &GeneratedOutput, dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        for file in &output.files {
            let full_path = dir.join(&file.path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if !file.overwrite && full_path.exists() {
                continue;
            }
            std::fs::write(&full_path, &file.content)?;
        }
        Ok(())
    }
}
