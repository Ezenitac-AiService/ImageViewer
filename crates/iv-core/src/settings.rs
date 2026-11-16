//! Backward-compatible `v1.0.6` `settings.json` schema, normalization, and `fs4`-locked selective-merge storage.

use crate::i18n::Language;
use crate::path_security::PathSecurity;
use fs4::fs_std::FileExt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use thiserror::Error;

/// Maximum permitted `settings.json` file size (`64 KB`).
pub const MAX_SETTINGS_BYTES: u64 = 65_536;
/// Current schema version.
pub const CURRENT_SETTINGS_VERSION: u32 = 3;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("unsafe symlink on settings path")]
    SymlinkRejected,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DisplayMode {
    Original,
    FitWidth,
    FitHeight,
    #[default]
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ViewerBackground {
    #[default]
    Black,
    DarkGray,
    White,
    Checkerboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MouseWheelAction {
    #[default]
    Navigate,
    Zoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FilterMode {
    #[default]
    None,
    Smooth,
    SmoothSharpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SlideShowMode {
    #[default]
    Sequential,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortField {
    #[default]
    Name,
    ModifiedDate,
    FileSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WindowGeometry {
    pub left: Option<f64>,
    pub top: Option<f64>,
    pub width: f64,
    pub height: f64,
    pub is_maximized: bool,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            left: None,
            top: None,
            width: 1280.0,
            height: 800.0,
            is_maximized: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DisplaySettings {
    pub mode: DisplayMode,
    pub background: ViewerBackground,
    pub mouse_wheel: MouseWheelAction,
    pub play_animations: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            mode: DisplayMode::Fill,
            background: ViewerBackground::Black,
            mouse_wheel: MouseWheelAction::Navigate,
            play_animations: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChromeSettings {
    pub top_bar_pinned: bool,
    pub bottom_bar_pinned: bool,
}

impl Default for ChromeSettings {
    fn default() -> Self {
        Self {
            top_bar_pinned: true,
            bottom_bar_pinned: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NavigationSettings {
    pub wrap_around: bool,
    pub watch_folder: bool,
    pub sort_field: SortField,
    pub sort_direction: SortDirection,
}

impl Default for NavigationSettings {
    fn default() -> Self {
        Self {
            wrap_around: true,
            watch_folder: true,
            sort_field: SortField::Name,
            sort_direction: SortDirection::Ascending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SlideShowSettings {
    pub mode: SlideShowMode,
    pub interval_seconds: u32,
    pub repeat: bool,
}

impl Default for SlideShowSettings {
    fn default() -> Self {
        Self {
            mode: SlideShowMode::Sequential,
            interval_seconds: 5,
            repeat: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ThumbnailSettings {
    pub visible: bool,
    pub show_scroll_bar: bool,
    pub size: u32,
}

impl Default for ThumbnailSettings {
    fn default() -> Self {
        Self {
            visible: true,
            show_scroll_bar: true,
            size: 96,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FilterSettings {
    pub mode: FilterMode,
    pub smooth_sigma: f64,
    pub sharpen_sigma: f64,
    pub sharpen_amount: f64,
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self {
            mode: FilterMode::None,
            smooth_sigma: 0.5,
            sharpen_sigma: 1.0,
            sharpen_amount: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LimitSettings {
    pub max_image_megapixels: u32,
    pub max_zip_entry_megabytes: u32,
}

impl Default for LimitSettings {
    fn default() -> Self {
        Self {
            max_image_megapixels: 150,
            max_zip_entry_megabytes: 512,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateSettings {
    pub auto_check: bool,
    pub skipped_version: Option<String>,
    pub last_check_utc: Option<i64>,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            auto_check: true,
            skipped_version: None,
            last_check_utc: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ViewerSettings {
    pub version: u32,
    pub language: Language,
    pub window: WindowGeometry,
    pub display: DisplaySettings,
    pub chrome: ChromeSettings,
    pub navigation: NavigationSettings,
    pub slideshow: SlideShowSettings,
    pub thumbnail: ThumbnailSettings,
    pub filter: FilterSettings,
    pub limits: LimitSettings,
    pub update: UpdateSettings,
}

impl Default for ViewerSettings {
    fn default() -> Self {
        Self {
            version: CURRENT_SETTINGS_VERSION,
            language: Language::En,
            window: WindowGeometry::default(),
            display: DisplaySettings::default(),
            chrome: ChromeSettings::default(),
            navigation: NavigationSettings::default(),
            slideshow: SlideShowSettings::default(),
            thumbnail: ThumbnailSettings::default(),
            filter: FilterSettings::default(),
            limits: LimitSettings::default(),
            update: UpdateSettings::default(),
        }
    }
}

impl ViewerSettings {
    /// Parses `v1.0.6`-compatible JSON (with comments and trailing commas), applies migrations, and normalizes bounds.
    pub fn from_json_str(raw: &str) -> Self {
        let stripped = strip_json_comments_and_trailing_commas(raw);
        let mut parsed: Self = serde_json::from_str(&stripped).unwrap_or_default();
        if parsed.version < 2 {
            parsed.display.mouse_wheel = MouseWheelAction::Navigate;
        }
        if parsed.version < 3 && (parsed.filter.smooth_sigma - 0.8).abs() < 1e-6 {
            parsed.filter.smooth_sigma = 0.5;
        }
        parsed.version = CURRENT_SETTINGS_VERSION;
        parsed.normalize();
        parsed
    }

    /// Clamps numeric bounds, clears non-finite floats, and sanitizes `skipped_version`.
    pub fn normalize(&mut self) {
        self.window.width = finite_clamp(self.window.width, 320.0, 20_000.0, 1280.0);
        self.window.height = finite_clamp(self.window.height, 240.0, 20_000.0, 800.0);
        self.window.left = self
            .window
            .left
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(-20_000.0, 20_000.0));
        self.window.top = self
            .window
            .top
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(-20_000.0, 20_000.0));

        self.slideshow.interval_seconds = self.slideshow.interval_seconds.clamp(5, 90);
        self.thumbnail.size = self.thumbnail.size.clamp(48, 256);

        self.filter.smooth_sigma = finite_clamp(self.filter.smooth_sigma, 0.1, 3.0, 0.5);
        self.filter.sharpen_sigma = finite_clamp(self.filter.sharpen_sigma, 0.1, 5.0, 1.0);
        self.filter.sharpen_amount = finite_clamp(self.filter.sharpen_amount, 0.1, 5.0, 1.0);

        self.limits.max_image_megapixels = self.limits.max_image_megapixels.clamp(10, 1000);
        self.limits.max_zip_entry_megabytes = self.limits.max_zip_entry_megabytes.clamp(10, 4096);

        self.update.skipped_version = self
            .update
            .skipped_version
            .as_deref()
            .and_then(sanitize_version_token);
    }
}

/// Multi-instance safe `fs4`-locked settings persistence.
pub struct SettingsStore;

impl SettingsStore {
    /// Loads settings safely (rejecting symlinks and files `> 64 KB`).
    pub fn load(path: &Path) -> ViewerSettings {
        if let Some(parent) = path.parent()
            && PathSecurity::ensure_not_symlink(parent).is_err()
        {
            return ViewerSettings::default();
        }
        if PathSecurity::ensure_not_symlink(path).is_err() {
            return ViewerSettings::default();
        }
        let Ok(meta) = std::fs::metadata(path) else {
            return ViewerSettings::default();
        };
        if meta.len() > MAX_SETTINGS_BYTES {
            return ViewerSettings::default();
        }
        match std::fs::read_to_string(path) {
            Ok(raw) => ViewerSettings::from_json_str(&raw),
            Err(_) => ViewerSettings::default(),
        }
    }

    /// Saves preference sections while preserving latest disk `window` geometry under `fs4` lock.
    pub fn save_preferences(path: &Path, prefs: &ViewerSettings) -> Result<(), SettingsError> {
        Self::locked_merge_write(path, |disk| {
            let saved_window = disk.window.clone();
            *disk = prefs.clone();
            disk.window = saved_window;
            disk.normalize();
        })
    }

    /// Saves only `window` geometry while preserving latest disk preferences under `fs4` lock.
    pub fn save_window_geometry(path: &Path, window: &WindowGeometry) -> Result<(), SettingsError> {
        Self::locked_merge_write(path, |disk| {
            disk.window = window.clone();
            disk.normalize();
        })
    }

    fn locked_merge_write<F>(path: &Path, mutate: F) -> Result<(), SettingsError>
    where
        F: FnOnce(&mut ViewerSettings),
    {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
            PathSecurity::ensure_not_symlink(parent).map_err(|_| SettingsError::SymlinkRejected)?;
            cleanup_orphan_tmps(parent);
        }
        PathSecurity::ensure_not_symlink(path).map_err(|_| SettingsError::SymlinkRejected)?;

        let lock_path = path.with_extension("lock");
        PathSecurity::ensure_not_symlink(&lock_path).map_err(|_| SettingsError::SymlinkRejected)?;
        let lock_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)?;
        lock_file.lock_exclusive()?;

        let mut latest = Self::load(path);
        mutate(&mut latest);
        let serialized = serde_json::to_string_pretty(&latest)?;

        let rand_suffix: u64 = rand::rng().random();
        let tmp_name = format!("settings.{rand_suffix:016x}.tmp");
        let tmp_path = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(tmp_name);
        {
            let mut tmp = File::create(&tmp_path)?;
            tmp.write_all(serialized.as_bytes())?;
            tmp.sync_all()?;
        }
        std::fs::rename(&tmp_path, path)?;
        let _ = FileExt::unlock(&lock_file);
        Ok(())
    }
}

fn finite_clamp(val: f64, min: f64, max: f64, fallback: f64) -> f64 {
    if val.is_finite() {
        val.clamp(min, max)
    } else {
        fallback
    }
}

fn sanitize_version_token(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 32 {
        return None;
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn cleanup_orphan_tmps(dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if s.starts_with("settings.") && s.ends_with(".tmp") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

fn strip_json_comments_and_trailing_commas(input: &str) -> String {
    let mut no_comments = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut in_str = false;
    let mut escape = false;

    while i < chars.len() {
        let c = chars[i];
        if in_str {
            no_comments.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            no_comments.push(c);
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            i += 2;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i = (i + 2).min(chars.len());
            continue;
        }
        no_comments.push(c);
        i += 1;
    }

    // Remove trailing commas before `}` or `]`
    let out_chars: Vec<char> = no_comments.chars().collect();
    let mut clean = String::with_capacity(out_chars.len());
    let mut j = 0;
    in_str = false;
    escape = false;
    while j < out_chars.len() {
        let c = out_chars[j];
        if in_str {
            clean.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }
            j += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            clean.push(c);
            j += 1;
            continue;
        }
        if c == ',' {
            let mut k = j + 1;
            while k < out_chars.len() && out_chars[k].is_whitespace() {
                k += 1;
            }
            if k < out_chars.len() && (out_chars[k] == '}' || out_chars[k] == ']') {
                j += 1;
                continue;
            }
        }
        clean.push(c);
        j += 1;
    }
    clean
}
