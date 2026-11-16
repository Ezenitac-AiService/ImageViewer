//! `fast_image_resize` SIMD downscaling and screen-pixel filter uniform math.

use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use iv_core::settings::FilterMode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResizeError {
    #[error("invalid source or destination dimensions")]
    InvalidDimensions,
    #[error("resizer failure: {0}")]
    Failed(String),
}

/// GPU shader filter uniforms calibrated in screen pixels (`sigma / display_scale`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilterUniforms {
    pub blur_sigma: f32,
    pub sharpen_sigma: f32,
    pub sharpen_amount: f32,
    pub active: bool,
}

pub struct SimdResizer;

impl SimdResizer {
    /// Downscales `rgba8` (`src_w x src_h`) to `dst_w x dst_h` using AVX2/SSE4.1/NEON SIMD.
    pub fn downscale_rgba8(
        rgba8: &[u8],
        src_w: u32,
        src_h: u32,
        dst_w: u32,
        dst_h: u32,
    ) -> Result<Vec<u8>, ResizeError> {
        if src_w == 0 || src_h == 0 || dst_w == 0 || dst_h == 0 {
            return Err(ResizeError::InvalidDimensions);
        }
        let expected = (src_w as usize)
            .checked_mul(src_h as usize)
            .and_then(|p| p.checked_mul(4))
            .ok_or(ResizeError::InvalidDimensions)?;
        if rgba8.len() < expected {
            return Err(ResizeError::InvalidDimensions);
        }

        let src_img = ImageRef::new(src_w, src_h, rgba8, PixelType::U8x4)
            .map_err(|e| ResizeError::Failed(e.to_string()))?;
        let mut dst_img = Image::new(dst_w, dst_h, PixelType::U8x4);
        let mut resizer = Resizer::new();
        let opts = ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3));
        resizer
            .resize(&src_img, &mut dst_img, Some(&opts))
            .map_err(|e| ResizeError::Failed(e.to_string()))?;
        Ok(dst_img.into_vec())
    }

    /// Computes screen-pixel calibrated filter parameters (`sigma / display_scale`, skipping `< 0.05` and animations).
    pub fn compute_filter_uniforms(
        mode: FilterMode,
        smooth_sigma: f64,
        sharpen_sigma: f64,
        sharpen_amount: f64,
        display_scale: f64,
        is_animated: bool,
    ) -> FilterUniforms {
        if is_animated || mode == FilterMode::None {
            return FilterUniforms {
                blur_sigma: 0.0,
                sharpen_sigma: 0.0,
                sharpen_amount: 0.0,
                active: false,
            };
        }
        let scale = display_scale.max(0.01);
        let eff_blur = (smooth_sigma / scale) as f32;
        let eff_sharpen = (sharpen_sigma / scale) as f32;
        match mode {
            FilterMode::None => FilterUniforms {
                blur_sigma: 0.0,
                sharpen_sigma: 0.0,
                sharpen_amount: 0.0,
                active: false,
            },
            FilterMode::Smooth => FilterUniforms {
                blur_sigma: if eff_blur < 0.05 { 0.0 } else { eff_blur },
                sharpen_sigma: 0.0,
                sharpen_amount: 0.0,
                active: eff_blur >= 0.05,
            },
            FilterMode::SmoothSharpen => FilterUniforms {
                blur_sigma: if eff_blur < 0.05 { 0.0 } else { eff_blur },
                sharpen_sigma: if eff_sharpen < 0.05 { 0.0 } else { eff_sharpen },
                sharpen_amount: sharpen_amount as f32,
                active: eff_blur >= 0.05 || eff_sharpen >= 0.05,
            },
        }
    }
}
