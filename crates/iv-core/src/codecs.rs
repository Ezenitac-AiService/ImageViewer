//! Acyclic `OsCodecProvider` trait and `OsDecodedBuffer` DTO (`iv-media -> iv-core`, `iv-platform -> iv-core`).

use thiserror::Error;

/// Decoded RGBA8 buffer returned by a host OS codec (macOS `ImageIO` or Linux probe).
#[derive(Debug, Clone)]
pub struct OsDecodedBuffer {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
    /// True if the OS codec already rotated pixels according to EXIF Orientation.
    pub orientation_applied: bool,
    /// True if the OS codec already color-managed pixels into display sRGB.
    pub srgb_converted: bool,
    pub exif_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OsCodecError {
    #[error("no compatible host OS codec available for format")]
    MissingCodec,
    #[error("image dimensions exceed configured safety limit")]
    ExceedsPixelLimit,
    #[error("failed to query image dimensions prior to pixel allocation")]
    UnverifiedDimensions,
    #[error("blocked by binary format security guard")]
    SecurityRejected,
    #[error("host OS codec decode failed: {0}")]
    DecodeFailed(String),
}

/// Narrow zero-`unsafe` trait implemented by `iv-platform` and invoked by `iv-media`.
pub trait OsCodecProvider: Send + Sync {
    /// Returns true if `ext_lower` (without leading dot, e.g. `"heic"`, `"dng"`) is handled by OS codecs.
    fn supports_extension(&self, ext_lower: &str) -> bool;

    /// Decodes `bytes` after `FormatGuard`, stream cap (`<= 512 MB`), and pre-allocation dimension check (`<= max_megapixels`).
    fn decode_guarded(
        &self,
        bytes: &[u8],
        ext_lower: &str,
        max_megapixels: u32,
    ) -> Result<OsDecodedBuffer, OsCodecError>;
}
