//! OS-idiomatic shell & file association integration (`Windows` HKCU, `macOS` Finder, `Linux` XDG).

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IntegrationError {
    #[error("executable path contains unsafe characters")]
    UnsafeExePath,
    #[error("registry or desktop integration failed: {0}")]
    Failed(String),
}

pub struct OsIntegration;

impl OsIntegration {
    pub fn validate_exe_path(exe_path: &str) -> Result<(), IntegrationError> {
        if exe_path.is_empty()
            || exe_path.contains('"')
            || exe_path.chars().any(|c| c.is_control())
        {
            return Err(IntegrationError::UnsafeExePath);
        }
        Ok(())
    }

    pub fn format_windows_open_command(exe_path: &str) -> String {
        format!("\"{exe_path}\" \"%1\"")
    }

    pub fn commands_match_normalized(actual: &str, expected: &str) -> bool {
        actual.trim() == expected.trim()
    }

    pub fn render_xdg_desktop_entry(exec_path: &str) -> String {
        format!(
            "[Desktop Entry]\nType=Application\nName=ImageViewer\nExec={exec_path} %F\nTerminal=false\nCategories=Graphics;Viewer;\nMimeType=image/jpeg;image/png;image/gif;image/webp;image/bmp;image/tiff;image/jxl;image/avif;image/heif;application/zip;application/vnd.comicbook+zip;\n"
        )
    }
}
