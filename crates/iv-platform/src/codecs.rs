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
    use objc2_core_foundation::{kCFBooleanFalse, CFData, CFDictionary, CFNumber, CFString};
    use objc2_core_graphics::{
        kCGColorSpaceSRGB, CGBitmapContextCreate, CGColorSpace, CGContext, CGImage,
        CGImageAlphaInfo,
    };
    use objc2_image_io::{
        kCGImagePropertyPixelHeight, kCGImagePropertyPixelWidth, kCGImageSourceShouldCache,
        CGImageSource,
    };
    use std::ffi::c_void;

    // SAFETY: CFData / CGImageSource wrap valid immutable Rust slice `bytes` cleanly.
    let cf_data = CFData::from_bytes(bytes);
    let source = unsafe { CGImageSource::with_data(&cf_data, None) }
        .ok_or(OsCodecError::UnverifiedDimensions)?;

    let count = unsafe { source.count() };
    if count == 0 {
        return Err(OsCodecError::UnverifiedDimensions);
    }

    // Pre-decode dimension query BEFORE pixel allocation (`kCGImageSourceShouldCache = false`)
    let cache_key: &CFString = unsafe { kCGImageSourceShouldCache };
    let cache_val = unsafe { kCFBooleanFalse }.ok_or(OsCodecError::UnverifiedDimensions)?;
    let keys = [cache_key as *const CFString as *const c_void];
    let vals = [cache_val as *const _ as *const c_void];
    let opts = unsafe {
        CFDictionary::new(
            None,
            keys.as_ptr().cast_mut(),
            vals.as_ptr().cast_mut(),
            1,
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    let props = unsafe { source.properties_at_index(0, opts.as_deref()) }
        .ok_or(OsCodecError::UnverifiedDimensions)?;

    let pw_key: &CFString = unsafe { kCGImagePropertyPixelWidth };
    let ph_key: &CFString = unsafe { kCGImagePropertyPixelHeight };
    let w_ptr = unsafe { props.value(pw_key as *const CFString as *const c_void) };
    let h_ptr = unsafe { props.value(ph_key as *const CFString as *const c_void) };
    if w_ptr.is_null() || h_ptr.is_null() {
        return Err(OsCodecError::UnverifiedDimensions);
    }
    let w_num: &CFNumber = unsafe { &*w_ptr.cast::<CFNumber>() };
    let h_num: &CFNumber = unsafe { &*h_ptr.cast::<CFNumber>() };
    let pre_w = w_num.as_i64().unwrap_or(0).max(0) as u32;
    let pre_h = h_num.as_i64().unwrap_or(0).max(0) as u32;
    if pre_w == 0 || pre_h == 0 {
        return Err(OsCodecError::UnverifiedDimensions);
    }
    let pre_pixels = u64::from(pre_w)
        .checked_mul(u64::from(pre_h))
        .ok_or(OsCodecError::ExceedsPixelLimit)?;
    let cap = u64::from(max_megapixels.clamp(10, 1000)) * 1_000_000;
    if pre_pixels > cap {
        return Err(OsCodecError::ExceedsPixelLimit);
    }

    // Only allocate and decode pixels AFTER verifying pre-decode dimensions
    let cg_image = unsafe { source.image_at_index(0, opts.as_deref()) }
        .ok_or_else(|| OsCodecError::DecodeFailed("ImageIO failed to decode".to_string()))?;

    let w = CGImage::width(Some(&cg_image)) as u32;
    let h = CGImage::height(Some(&cg_image)) as u32;
    if w == 0 || h == 0 {
        return Err(OsCodecError::UnverifiedDimensions);
    }
    let pixels = u64::from(w)
        .checked_mul(u64::from(h))
        .ok_or(OsCodecError::ExceedsPixelLimit)?;
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
fn decode_platform(bytes: &[u8], max_megapixels: u32) -> Result<OsDecodedBuffer, OsCodecError> {
    use windows::Win32::Graphics::Imaging::{
        CLSID_WICImagingFactory, GUID_WICPixelFormat32bppRGBA, IWICImagingFactory,
        WICBitmapDitherTypeNone, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
    };

    // SAFETY: Initialize COM on current thread and query WIC decoder dimensions strictly BEFORE CopyPixels.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
                .map_err(|_| OsCodecError::MissingCodec)?;
        let stream = factory
            .CreateStream()
            .map_err(|e| OsCodecError::DecodeFailed(e.to_string()))?;
        stream
            .InitializeFromMemory(bytes)
            .map_err(|e| OsCodecError::DecodeFailed(e.to_string()))?;
        let decoder = factory
            .CreateDecoderFromStream(&stream, std::ptr::null(), WICDecodeMetadataCacheOnDemand)
            .map_err(|_| OsCodecError::MissingCodec)?;
        let frame = decoder
            .GetFrame(0)
            .map_err(|e| OsCodecError::DecodeFailed(e.to_string()))?;

        let mut w = 0u32;
        let mut h = 0u32;
        frame
            .GetSize(&mut w, &mut h)
            .map_err(|_| OsCodecError::UnverifiedDimensions)?;
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

        let converter = factory
            .CreateFormatConverter()
            .map_err(|e| OsCodecError::DecodeFailed(e.to_string()))?;
        converter
            .Initialize(
                &frame,
                &GUID_WICPixelFormat32bppRGBA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeCustom,
            )
            .map_err(|e| OsCodecError::DecodeFailed(e.to_string()))?;

        let stride = w.checked_mul(4).ok_or(OsCodecError::ExceedsPixelLimit)?;
        let mut rgba = vec![0u8; (pixels as usize) * 4];
        converter
            .CopyPixels(std::ptr::null(), stride, &mut rgba)
            .map_err(|e| OsCodecError::DecodeFailed(e.to_string()))?;

        Ok(OsDecodedBuffer {
            width: w,
            height: h,
            rgba8: rgba,
            orientation_applied: false,
            srgb_converted: false,
            exif_bytes: None,
        })
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn decode_platform(_bytes: &[u8], _max_megapixels: u32) -> Result<OsDecodedBuffer, OsCodecError> {
    // Linux fails closed with localized missing-codec message when optional system codecs are absent.
    Err(OsCodecError::MissingCodec)
}
