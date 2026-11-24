//! OS app-data directories and non-blocking native file/folder pickers.
//!
//! Pickers run on a worker thread and report through a caller-supplied callback, so this crate
//! has no dependency on the GUI framework (specs/004 finding C9).

use directories::ProjectDirs;
use std::path::PathBuf;

/// Callback invoked once with the picked path, or `None` when the user cancelled.
pub type PickCallback = Box<dyn FnOnce(Option<PathBuf>) + Send + 'static>;

/// Extensions offered by the file picker: Core formats plus read-only archives.
pub const CORE_PICKER_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "jpe", "jfif", "png", "apng", "gif", "webp", "bmp", "dib", "tif", "tiff", "zip",
    "cbz",
];

pub struct OsPaths;

impl OsPaths {
    /// Returns the OS-idiomatic configuration directory.
    pub fn config_dir() -> PathBuf {
        ProjectDirs::from("com", "Ezenitac-AiService", "ImageViewer")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| std::env::temp_dir().join("ImageViewer"))
    }

    pub fn settings_file() -> PathBuf {
        Self::config_dir().join("settings.json")
    }

    pub fn logs_dir() -> PathBuf {
        Self::config_dir().join("logs")
    }
}

pub struct OsDialogs;

impl OsDialogs {
    /// Shows a native file/archive picker without blocking the caller.
    pub fn pick_image_or_archive_async(title: &str, filter_label: &str, on_done: PickCallback) {
        let dialog = rfd::AsyncFileDialog::new()
            .set_title(title)
            .add_filter(filter_label, CORE_PICKER_EXTENSIONS);
        std::thread::spawn(move || {
            let picked = pollster::block_on(dialog.pick_file()).map(|f| f.path().to_path_buf());
            on_done(picked);
        });
    }

    /// Shows a native folder picker without blocking the caller.
    pub fn pick_folder_async(title: &str, on_done: PickCallback) {
        let dialog = rfd::AsyncFileDialog::new().set_title(title);
        std::thread::spawn(move || {
            let picked = pollster::block_on(dialog.pick_folder()).map(|f| f.path().to_path_buf());
            on_done(picked);
        });
    }
}
