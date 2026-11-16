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

pub trait MediaSource: Send + Sync {
    fn entries(&self) -> &[MediaEntry];
    fn initial_index(&self) -> usize;
    fn read_entry_bytes(&self, index: usize) -> Result<Vec<u8>, SourceError>;
}

/// Supported extensions across pure-Rust core decoders and host OS codecs.
pub fn is_supported_image_extension(ext_lower: &str) -> bool {
    matches!(
        ext_lower,
        "jpg"
            | "jpeg"
            | "jpe"
            | "jfif"
            | "png"
            | "apng"
            | "gif"
            | "bmp"
            | "dib"
            | "tif"
            | "tiff"
            | "webp"
            | "ico"
            | "cur"
            | "jxl"
            | "tga"
            | "psd"
            | "dds"
            | "pcx"
            | "qoi"
            | "ppm"
            | "pgm"
            | "pbm"
            | "pnm"
            | "hdr"
            | "exr"
            | "dng"
            | "cr2"
            | "cr3"
            | "nef"
            | "arw"
            | "orf"
            | "rw2"
            | "raf"
            | "heic"
            | "heif"
            | "avif"
            | "jp2"
    )
}
