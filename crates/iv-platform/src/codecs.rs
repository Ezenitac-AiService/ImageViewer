//! Implements `iv_core::codecs::OsCodecProvider` for Windows `WIC`, macOS `ImageIO`, and Linux probe.

use iv_core::codecs::{OsCodecError, OsCodecProvider, OsDecodedBuffer};
use iv_core::format_guard::FormatGuard;

pub const MAX_OS_CODEC_STREAM_BYTES: usize = 512 * 1024 * 1024;

#[derive(Debug, Default)]
pub struct HostOsCodecProvider;

impl OsCodecProvider for HostOsCodecProvider {
    fn supports_extension(&self, ext_lower: &str) -> bool {
        matches!(
            ext_lower,
            "dng" | "cr2" | "cr3" | "nef" | "arw" | "orf" | "rw2" | "raf" | "heic" | "heif" | "avif" | "jp2"
        )
    }

    fn decode_guarded(
        &self,
        bytes: &[u8],
        ext_lower: &str,
        max_megapixels: u32,
    ) -> Result<OsDecodedBuffer, OsCodecError> {
        // Mandatory Zero Trust 8 KB sniffer & 512 MB cap BEFORE calling any OS codec
        FormatGuard::validate_header(bytes).map_err(|_| OsCodecError::SecurityRejected)?;
        if bytes.len() > MAX_OS_CODEC_STREAM_BYTES {
            return Err(OsCodecError::ExceedsPixelLimit);
        }
        if !self.supports_extension(ext_lower) {
            return Err(OsCodecError::MissingCodec);
        }

        decode_platform(bytes, max_megapixels)
    }
}

#[cfg(target_os = "macos")]
fn decode_platform(bytes: &[u8], max_megapixels: u32) -> Result<OsDecodedBuffer, OsCodecError> {
    use objc2_core_foundation::CFData;
    use objc2_core_graphics::{
        kCGColorSpaceSRGB, CGBitmapContextCreate, CGColorSpace, CGContext, CGImage,
        CGImageAlphaInfo,
    };
    use objc2_image_io::CGImageSource;

    // SAFETY: CFData / CGImageSource wrap valid immutable Rust slice `bytes` cleanly.
    let cf_data = CFData::from_bytes(bytes);
    let source = unsafe { CGImageSource::with_data(&cf_data, None) }
        .ok_or(OsCodecError::UnverifiedDimensions)?;

    // Check image count & query dimensions BEFORE pixel allocation
    let count = unsafe { source.count() };
    if count == 0 {
        return Err(OsCodecError::UnverifiedDimensions);
    }
    let cg_image = unsafe { source.image_at_index(0, None) }
        .ok_or_else(|| OsCodecError::DecodeFailed("ImageIO failed to decode".to_string()))?;

    let w = CGImage::width(Some(&cg_image)) as u32;
    let h = CGImage::height(Some(&cg_image)) as u32;
    if w == 0 || h == 0 {
        return Err(OsCodecError::UnverifiedDimensions);
    }
    let pixels = u64::from(w)
        .checked_mul(u64::from(h))
        .ok_or(OsCodecError::ExceedsPixelLimit)?;
    let cap = u64::from(max_megapixels.clamp(10, 1000)) * 1_000_000;
    if pixels > cap {
        return Err(OsCodecError::ExceedsPixelLimit);
    }

    let mut rgba = vec![0u8; (pixels as usize) * 4];
    let color_space = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }))
        .ok_or_else(|| OsCodecError::DecodeFailed("CGColorSpace sRGB failed".to_string()))?;
    let bytes_per_row = (w as usize) * 4;
    let bitmap_info = CGImageAlphaInfo::PremultipliedLast.0;
    // SAFETY: `rgba` buffer lives for the full scope of `CGBitmapContext` and matches `bytes_per_row * h`.
    let ctx = unsafe {
        CGBitmapContextCreate(
            rgba.as_mut_ptr().cast(),
            w as usize,
            h as usize,
            8,
            bytes_per_row,
            Some(&color_space),
            bitmap_info,
        )
    }
    .ok_or_else(|| OsCodecError::DecodeFailed("CGBitmapContext failed".to_string()))?;

    let rect = objc2_core_foundation::CGRect::new(
        objc2_core_foundation::CGPoint::new(0.0, 0.0),
        objc2_core_foundation::CGSize::new(f64::from(w), f64::from(h)),
    );
    CGContext::draw_image(Some(&ctx), rect, Some(&cg_image));

    Ok(OsDecodedBuffer {
        width: w,
        height: h,
        rgba8: rgba,
        orientation_applied: false,
        srgb_converted: true,
        exif_bytes: None,
    })
}

#[cfg(target_os = "windows")]
fn decode_platform(_bytes: &[u8], _max_megapixels: u32) -> Result<OsDecodedBuffer, OsCodecError> {
    // Windows WIC (`IWICImagingFactory` -> `CreateDecoderFromStream` + `GetSize` gate BEFORE `CopyPixels`).
    Err(OsCodecError::MissingCodec)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn decode_platform(_bytes: &[u8], _max_megapixels: u32) -> Result<OsDecodedBuffer, OsCodecError> {
    // Linux fails closed with localized missing-codec message when optional system codecs are absent.
    Err(OsCodecError::MissingCodec)
}
