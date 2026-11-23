//! Pure-Rust ICC profile conversion (`moxcms`) and lightweight HDR tone mapping.

use moxcms::{ColorProfile, Layout, TransformOptions};

pub struct ColorHdrNormalizer;

impl ColorHdrNormalizer {
    /// Converts RGBA8 pixels embedded with `icc_bytes` into sRGB RGBA8, falling back cleanly if corrupt.
    pub fn convert_icc_rgba8_to_srgb(rgba8: &mut [u8], icc_bytes: &[u8]) -> bool {
        let Ok(src_profile) = ColorProfile::new_from_slice(icc_bytes) else {
            return false;
        };
        let dst_profile = ColorProfile::new_srgb();
        let Ok(transform) = src_profile.create_transform_8bit(
            Layout::Rgba,
            &dst_profile,
            Layout::Rgba,
            TransformOptions::default(),
        ) else {
            return false;
        };
        let mut dst = vec![0u8; rgba8.len()];
        if transform.transform(rgba8, &mut dst).is_ok() {
            rgba8.copy_from_slice(&dst);
            true
        } else {
            false
        }
    }

    /// Converts CMYK 8-bit (`cmyk` 4 bytes/pixel) with optional ICC profile into sRGB RGBA8.
    pub fn convert_cmyk8_to_srgb_rgba8(cmyk: &[u8], icc_bytes: Option<&[u8]>) -> Vec<u8> {
        let pixel_count = cmyk.len() / 4;
        let mut out = vec![255u8; pixel_count * 4];
        if let Some(icc) = icc_bytes
            && let Ok(src) = ColorProfile::new_from_slice(icc)
        {
            let dst = ColorProfile::new_srgb();
            if let Ok(xform) = src.create_transform_8bit(
                Layout::Rgba,
                &dst,
                Layout::Rgba,
                TransformOptions::default(),
            ) && xform.transform(cmyk, &mut out).is_ok()
            {
                for px in out.as_chunks_mut::<4>().0 {
                    px[3] = 255;
                }
                return out;
            }
        }
        // Fallback standard subtractive CMYK -> sRGB conversion
        for (src_px, dst_px) in cmyk
            .as_chunks::<4>()
            .0
            .iter()
            .zip(out.as_chunks_mut::<4>().0.iter_mut())
        {
            let c = f32::from(src_px[0]) / 255.0;
            let m = f32::from(src_px[1]) / 255.0;
            let y = f32::from(src_px[2]) / 255.0;
            let k = f32::from(src_px[3]) / 255.0;
            dst_px[0] = ((1.0 - c) * (1.0 - k) * 255.0).round().clamp(0.0, 255.0) as u8;
            dst_px[1] = ((1.0 - m) * (1.0 - k) * 255.0).round().clamp(0.0, 255.0) as u8;
            dst_px[2] = ((1.0 - y) * (1.0 - k) * 255.0).round().clamp(0.0, 255.0) as u8;
            dst_px[3] = 255;
        }
        out
    }

    /// Applies soft Reinhard tone mapping and sRGB gamma encoding to linear `f32` RGBA pixels.
    pub fn tonemap_rgba_f32_to_srgb(rgba_f32: &[f32]) -> Vec<u8> {
        let mut out = Vec::with_capacity(rgba_f32.len());
        for px in rgba_f32.as_chunks::<4>().0 {
            for &c in &px[0..3] {
                let linear = c.max(0.0);
                let mapped = linear / (1.0 + linear);
                let srgb = linear_to_srgb_channel(mapped);
                out.push((srgb * 255.0).round().clamp(0.0, 255.0) as u8);
            }
            out.push((px[3].clamp(0.0, 1.0) * 255.0).round() as u8);
        }
        out
    }

    /// Normalizes 16-bit integer RGBA pixels (`rgba16`) down to 8-bit sRGB RGBA.
    pub fn normalize_rgba16_to_rgba8(rgba16: &[u16]) -> Vec<u8> {
        rgba16.iter().map(|&v| (v >> 8) as u8).collect()
    }
}

fn linear_to_srgb_channel(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
