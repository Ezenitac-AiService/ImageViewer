//! Media source abstractions (`FolderSource`, `ZipSource`, `FolderWatcher`).

pub mod folder;
pub mod watcher;
pub mod zip;

use iv_core::i18n::{I18n, Language};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSourceType {
    File,
    Zip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaEntry {
    pub id: u64,
    pub name: String,
    pub source_key: String,
    pub source_type: ImageSourceType,
    pub byte_size: u64,
    pub modified_utc: Option<i64>,
    pub is_supported: bool,
}

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("archive error: {0}")]
    Archive(String),
    #[error("{}", I18n::tr(Language::En, "error.encrypted_archive"))]
    EncryptedArchive,
    #[error("archive exceeded safety limits: {0}")]
    ArchiveLimitExceeded(String),
    #[error("path security violation")]
    SecurityViolation,
}

impl SourceError {
    /// Localized message for the user (FR-018), free of paths and internal detail.
    pub fn user_message(&self, lang: Language) -> String {
        let key = match self {
            Self::Io(_) => "error.read_failed",
            Self::Archive(_) => "error.archive_failed",
            Self::EncryptedArchive => "error.encrypted_archive",
            Self::ArchiveLimitExceeded(_) => "error.archive_limit",
            Self::SecurityViolation => "error.unsafe_image_or_archive",
        };
        I18n::tr(lang, key).to_string()
    }
}

pub trait MediaSource: Send + Sync {
    fn entries(&self) -> &[MediaEntry];
    fn initial_index(&self) -> usize;
    fn read_entry_bytes(&self, index: usize) -> Result<Vec<u8>, SourceError>;
}

/// Extensions listed and offered in the shipped build: the Core formats (specs/004 FR-021).
const CORE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "jpe", "jfif", "png", "apng", "gif", "bmp", "dib", "tif", "tiff", "webp",
];

/// Extended-tier extensions, listed only when that tier is compiled in.
#[cfg(feature = "extended-formats")]
const EXTENDED_EXTENSIONS: &[&str] = &[
    "ico", "cur", "jxl", "tga", "psd", "dds", "pcx", "qoi", "ppm", "pgm", "pbm", "pnm", "hdr",
    "exr",
];

/// Whether `ext_lower` belongs to the extended tier (only meaningful when it is compiled in).
#[cfg(feature = "extended-formats")]
pub fn is_extended_extension(ext_lower: &str) -> bool {
    EXTENDED_EXTENSIONS.contains(&ext_lower)
}

/// Whether files with this (lower-case) extension are listed as images.
pub fn is_supported_image_extension(ext_lower: &str) -> bool {
    #[cfg(feature = "extended-formats")]
    let extended = EXTENDED_EXTENSIONS.contains(&ext_lower);
    #[cfg(not(feature = "extended-formats"))]
    let extended = false;
    CORE_EXTENSIONS.contains(&ext_lower) || extended
}
