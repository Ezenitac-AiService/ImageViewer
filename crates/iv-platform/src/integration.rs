//! OS-idiomatic shell & file association integration (`macOS` Finder, `Linux` XDG).

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IntegrationError {
    #[error("executable path contains unsafe characters")]
    UnsafeExePath,
    #[error("desktop integration failed: {0}")]
    Failed(String),
}

pub struct OsIntegration;

impl OsIntegration {
    pub fn validate_exe_path(exe_path: &str) -> Result<(), IntegrationError> {
        if exe_path.is_empty() || exe_path.contains('"') || exe_path.chars().any(|c| c.is_control())
        {
            return Err(IntegrationError::UnsafeExePath);
        }
        Ok(())
    }

    pub fn commands_match_normalized(actual: &str, expected: &str) -> bool {
        actual.trim() == expected.trim()
    }

    pub fn render_xdg_desktop_entry(exec_path: &str) -> String {
        format!(
            "[Desktop Entry]\nType=Application\nName=ImageViewer\nExec={exec_path} %F\nTerminal=false\nCategories=Graphics;Viewer;\nMimeType=image/jpeg;image/png;image/gif;image/webp;image/bmp;image/tiff;image/jxl;image/avif;image/heif;application/zip;application/vnd.comicbook+zip;\n"
        )
    }

    /// Installs user-scope XDG `.desktop` entry on Linux.
    pub fn register_linux_desktop(exec_path: &str) -> Result<PathBuf, IntegrationError> {
        Self::validate_exe_path(exec_path)?;
        let content = Self::render_xdg_desktop_entry(exec_path);
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .ok_or_else(|| IntegrationError::Failed("missing HOME".to_string()))?;
        let app_dir = base.join("applications");
        std::fs::create_dir_all(&app_dir).map_err(|e| IntegrationError::Failed(e.to_string()))?;
        let desktop_path = app_dir.join("imageviewer.desktop");
        std::fs::write(&desktop_path, content)
            .map_err(|e| IntegrationError::Failed(e.to_string()))?;
        Ok(desktop_path)
    }
}
