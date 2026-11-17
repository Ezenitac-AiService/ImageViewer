//! Leading 8,192-byte multi-BOM and script/document sniffer (`FormatGuard`).

use thiserror::Error;

/// Maximum header inspection window in bytes.
pub const FORMAT_GUARD_SCAN_BYTES: usize = 8192;
/// Pure-whitespace prefix threshold beyond which inputs are rejected.
pub const MAX_LEADING_WHITESPACE_BYTES: usize = 512;

/// Rejections emitted by [`FormatGuard`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FormatGuardError {
    #[error("empty input")]
    EmptyInput,
    #[error("excessive leading whitespace padding")]
    ExcessiveWhitespacePadding,
    #[error("blocked script or document signature")]
    BlockedScriptOrDocument,
}

/// Inspects stream prefixes prior to both pure-Rust and host OS codec decoders.
pub struct FormatGuard;

impl FormatGuard {
    /// Validates up to `8,192` leading bytes, rejecting script/document polyglots across UTF-32/16/8 BOMs.
    pub fn validate_header(bytes: &[u8]) -> Result<(), FormatGuardError> {
        if bytes.is_empty() {
            return Err(FormatGuardError::EmptyInput);
        }
        let sample = &bytes[..bytes.len().min(FORMAT_GUARD_SCAN_BYTES)];
        let decoded = Self::decode_prefix_text(sample);

        let trimmed_chars = decoded.trim_start_matches(|c: char| c.is_whitespace() || c == '\0');
        let leading_ws_bytes = decoded.len().saturating_sub(trimmed_chars.len());
        if leading_ws_bytes >= MAX_LEADING_WHITESPACE_BYTES && trimmed_chars.is_empty() {
            return Err(FormatGuardError::ExcessiveWhitespacePadding);
        }
        // Also reject raw >= 512 ASCII whitespace padding before any content
        let raw_ws = sample
            .iter()
            .take_while(|b| b.is_ascii_whitespace() || **b == 0)
            .count();
        if raw_ws >= MAX_LEADING_WHITESPACE_BYTES {
            return Err(FormatGuardError::ExcessiveWhitespacePadding);
        }

        let lower = trimmed_chars.to_ascii_lowercase();
        const BLOCKED_PREFIXES: &[&str] = &[
            "<", "<?xml", "<svg", "<html", "%!", "%pdf", "push ", "viewbox",
        ];
        if BLOCKED_PREFIXES
            .iter()
            .any(|prefix| lower.starts_with(prefix))
        {
            return Err(FormatGuardError::BlockedScriptOrDocument);
        }

        Ok(())
    }

    fn decode_prefix_text(sample: &[u8]) -> String {
        // Check 4-byte UTF-32 LE/BE BOMs strictly BEFORE 2-byte UTF-16 BOMs!
        if sample.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
            return Self::decode_utf32(&sample[4..], true);
        }
        if sample.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
            return Self::decode_utf32(&sample[4..], false);
        }
        if sample.starts_with(&[0xFF, 0xFE]) {
            return Self::decode_utf16(&sample[2..], true);
        }
        if sample.starts_with(&[0xFE, 0xFF]) {
            return Self::decode_utf16(&sample[2..], false);
        }
        if sample.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return String::from_utf8_lossy(&sample[3..]).into_owned();
        }

        // BOM-less UTF-16 interleaved ASCII heuristic (`<\0`, `\0<`, ` \0`, etc.)
        if sample.len() >= 4 {
            let pairs = sample.as_chunks::<2>().0;
            let even_ascii_odd_zero = pairs
                .iter()
                .take(8)
                .all(|[lo, hi]| lo.is_ascii() && *hi == 0);
            if even_ascii_odd_zero {
                return Self::decode_utf16(sample, true);
            }
            let even_zero_odd_ascii = pairs
                .iter()
                .take(8)
                .all(|[lo, hi]| *lo == 0 && hi.is_ascii());
            if even_zero_odd_ascii {
                return Self::decode_utf16(sample, false);
            }
        }

        String::from_utf8_lossy(sample).into_owned()
    }

    fn decode_utf32(bytes: &[u8], little_endian: bool) -> String {
        let mut out = String::new();
        for chunk in bytes.as_chunks::<4>().0 {
            let code = if little_endian {
                u32::from_le_bytes(*chunk)
            } else {
                u32::from_be_bytes(*chunk)
            };
            if let Some(ch) = char::from_u32(code) {
                out.push(ch);
            }
        }
        out
    }

    fn decode_utf16(bytes: &[u8], little_endian: bool) -> String {
        let units: Vec<u16> = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|chunk| {
                if little_endian {
                    u16::from_le_bytes(*chunk)
                } else {
                    u16::from_be_bytes(*chunk)
                }
            })
            .collect();
        String::from_utf16_lossy(&units)
    }
}

/// Formats the shipped build decodes (specs/004 FR-021), identified by content only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreFormat {
    Jpeg,
    Png,
    Gif,
    WebP,
    Bmp,
    Tiff,
    Zip,
}

/// BITMAPINFOHEADER family sizes accepted after the `BM` file header.
const BMP_DIB_HEADER_SIZES: [u32; 6] = [12, 40, 52, 56, 108, 124];

impl FormatGuard {
    /// Positively identifies a Core format from its leading bytes
    /// (`contracts/input-acceptance-contract.md` §1). Returns `None` for anything else, whatever
    /// the file is called; callers must then refuse to decode it.
    pub fn identify(bytes: &[u8]) -> Option<CoreFormat> {
        const PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(CoreFormat::Jpeg);
        }
        if bytes.starts_with(&PNG) {
            return Some(CoreFormat::Png);
        }
        if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            return Some(CoreFormat::Gif);
        }
        if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
            return Some(CoreFormat::WebP);
        }
        if bytes.len() >= 26 && bytes.starts_with(b"BM") {
            let dib = u32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
            return BMP_DIB_HEADER_SIZES
                .contains(&dib)
                .then_some(CoreFormat::Bmp);
        }
        if bytes.starts_with(&[0x49, 0x49, 0x2A, 0x00])
            || bytes.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
        {
            return Some(CoreFormat::Tiff);
        }
        if bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04])
            || bytes.starts_with(&[0x50, 0x4B, 0x05, 0x06])
        {
            return Some(CoreFormat::Zip);
        }
        None
    }
}
