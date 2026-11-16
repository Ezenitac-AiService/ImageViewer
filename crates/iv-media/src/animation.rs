//! `AnimationPlayer` (`<= 512 MB` pre-decode vs `<= 128 MB` ring buffer, GIF/WebP/APNG compositing & delay rules).

use crate::cache::DecodedFrame;
use image::codecs::gif::GifDecoder;
use image::codecs::png::PngDecoder;
use image::AnimationDecoder;
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

pub const PREDECODE_RAM_BUDGET_BYTES: usize = 512 * 1024 * 1024;
pub const STREAM_RING_BUDGET_BYTES: usize = 128 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApngDisposeOp {
    None,
    Background,
    Previous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApngBlendOp {
    Source,
    Over,
}

#[derive(Debug, Error)]
pub enum AnimationError {
    #[error("failed to decode animation: {0}")]
    Decode(String),
}

pub struct AnimationFrame {
    pub frame: Arc<DecodedFrame>,
    pub delay: Duration,
}

pub struct AnimationPlayer {
    frames: Vec<AnimationFrame>,
    cursor: usize,
    pub is_streaming_mode: bool,
}

impl AnimationPlayer {
    /// GIF frames with delay `<= 10 ms` normalize to `100 ms`.
    pub fn normalize_gif_delay(delay: Duration) -> Duration {
        if delay <= Duration::from_millis(10) {
            Duration::from_millis(100)
        } else {
            delay
        }
    }

    /// WebP preserves per-frame delays (including `1 ms`) unless *every* frame is `<= 10 ms` (then `100 ms`).
    pub fn normalize_webp_delays(delays: &[Duration]) -> Vec<Duration> {
        if !delays.is_empty() && delays.iter().all(|d| *d <= Duration::from_millis(10)) {
            vec![Duration::from_millis(100); delays.len()]
        } else {
            delays.to_vec()
        }
    }

    /// APNG rational frame duration (`delay_num / delay_den` seconds, `delay_den == 0 -> 100`, `<= 10 ms -> 100 ms`).
    pub fn apng_frame_delay(delay_num: u16, delay_den: u16) -> Duration {
        let den = if delay_den == 0 { 100u32 } else { u32::from(delay_den) };
        let ms = (u64::from(delay_num) * 1000) / u64::from(den);
        let d = Duration::from_millis(ms);
        if d <= Duration::from_millis(10) {
            Duration::from_millis(100)
        } else {
            d
        }
    }

    /// Composites an APNG sub-rect onto the persistent RGBA8 canvas.
    #[allow(clippy::too_many_arguments)]
    pub fn composite_apng_rect(
        canvas: &mut [u8],
        canvas_w: u32,
        _canvas_h: u32,
        x_off: u32,
        y_off: u32,
        w: u32,
        h: u32,
        src_rgba: &[u8],
        blend: ApngBlendOp,
    ) {
        for row in 0..h {
            for col in 0..w {
                let cx = x_off + col;
                let cy = y_off + row;
                let dst_idx = ((cy * canvas_w + cx) * 4) as usize;
                let src_idx = ((row * w + col) * 4) as usize;
                if dst_idx + 4 > canvas.len() || src_idx + 4 > src_rgba.len() {
                    continue;
                }
                match blend {
                    ApngBlendOp::Source => {
                        canvas[dst_idx..dst_idx + 4].copy_from_slice(&src_rgba[src_idx..src_idx + 4]);
                    }
                    ApngBlendOp::Over => {
                        let sa = f32::from(src_rgba[src_idx + 3]) / 255.0;
                        let da = f32::from(canvas[dst_idx + 3]) / 255.0;
                        let out_a = sa + da * (1.0 - sa);
                        if out_a > 0.0 {
                            for c in 0..3 {
                                let sc = f32::from(src_rgba[src_idx + c]) / 255.0;
                                let dc = f32::from(canvas[dst_idx + c]) / 255.0;
                                let oc = (sc * sa + dc * da * (1.0 - sa)) / out_a;
                                canvas[dst_idx + c] = (oc * 255.0).round().clamp(0.0, 255.0) as u8;
                            }
                            canvas[dst_idx + 3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
                        }
                    }
                }
            }
        }
    }

    /// Applies APNG `DisposeOp` after presenting a frame.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_apng_dispose(
        canvas: &mut [u8],
        canvas_w: u32,
        x_off: u32,
        y_off: u32,
        w: u32,
        h: u32,
        dispose: ApngDisposeOp,
        previous_snapshot: Option<&[u8]>,
    ) {
        match dispose {
            ApngDisposeOp::None => {}
            ApngDisposeOp::Background => {
                for row in 0..h {
                    for col in 0..w {
                        let idx = (((y_off + row) * canvas_w + (x_off + col)) * 4) as usize;
                        if idx + 4 <= canvas.len() {
                            canvas[idx..idx + 4].fill(0);
                        }
                    }
                }
            }
            ApngDisposeOp::Previous => {
                if let Some(prev) = previous_snapshot
                    && prev.len() == canvas.len()
                {
                    canvas.copy_from_slice(prev);
                }
            }
        }
    }

    /// Decodes animated GIF/APNG/WebP frames cleanly.
    pub fn from_bytes(bytes: &[u8], filename: &str) -> Result<Self, AnimationError> {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "gif" => {
                let dec = GifDecoder::new(Cursor::new(bytes))
                    .map_err(|e| AnimationError::Decode(e.to_string()))?;
                let raw_frames = dec
                    .into_frames()
                    .collect_frames()
                    .map_err(|e| AnimationError::Decode(e.to_string()))?;
                let mut frames = Vec::with_capacity(raw_frames.len());
                for f in raw_frames {
                    let (num, den) = f.delay().numer_denom_ms();
                    let delay_ms = if den == 0 { 100 } else { u64::from(num) / u64::from(den) };
                    let delay = Self::normalize_gif_delay(Duration::from_millis(delay_ms));
                    let buf = f.into_buffer();
                    let (w, h) = buf.dimensions();
                    frames.push(AnimationFrame {
                        frame: Arc::new(DecodedFrame {
                            source_width: w,
                            source_height: h,
                            buffer_width: w,
                            buffer_height: h,
                            rgba8: buf.into_raw(),
                            is_downscaled: false,
                            page_count: 1,
                            page_index: 0,
                            exif_orientation_applied: true,
                            color_profile_converted: false,
                            format_label: "GIF".to_string(),
                            exif_items: Vec::new(),
                        }),
                        delay,
                    });
                }
                if frames.is_empty() {
                    return Err(AnimationError::Decode("empty gif".to_string()));
                }
                Ok(Self {
                    frames,
                    cursor: 0,
                    is_streaming_mode: false,
                })
            }
            "png" | "apng" => {
                let dec = PngDecoder::new(Cursor::new(bytes))
                    .map_err(|e| AnimationError::Decode(e.to_string()))?;
                let apng = dec
                    .apng()
                    .map_err(|e| AnimationError::Decode(e.to_string()))?;
                let raw_frames = apng
                    .into_frames()
                    .collect_frames()
                    .map_err(|e| AnimationError::Decode(e.to_string()))?;
                let mut frames = Vec::with_capacity(raw_frames.len());
                for f in raw_frames {
                    let (num, den) = f.delay().numer_denom_ms();
                    let delay_ms = if den == 0 { 100 } else { u64::from(num) / u64::from(den) };
                    let delay = Self::normalize_gif_delay(Duration::from_millis(delay_ms));
                    let buf = f.into_buffer();
                    let (w, h) = buf.dimensions();
                    frames.push(AnimationFrame {
                        frame: Arc::new(DecodedFrame {
                            source_width: w,
                            source_height: h,
                            buffer_width: w,
                            buffer_height: h,
                            rgba8: buf.into_raw(),
                            is_downscaled: false,
                            page_count: 1,
                            page_index: 0,
                            exif_orientation_applied: true,
                            color_profile_converted: false,
                            format_label: "APNG".to_string(),
                            exif_items: Vec::new(),
                        }),
                        delay,
                    });
                }
                if frames.is_empty() {
                    return Err(AnimationError::Decode("empty apng".to_string()));
                }
                Ok(Self {
                    frames,
                    cursor: 0,
                    is_streaming_mode: false,
                })
            }
            _ => {
                let mut webp = image_webp::WebPDecoder::new(Cursor::new(bytes))
                    .map_err(|e| AnimationError::Decode(e.to_string()))?;
                let (w, h) = webp.dimensions();
                let mut rgba = vec![0u8; (w as usize) * (h as usize) * 4];
                if webp.read_image(&mut rgba).is_err() {
                    let mut rgb = vec![0u8; (w as usize) * (h as usize) * 3];
                    webp.read_image(&mut rgb)
                        .map_err(|e| AnimationError::Decode(e.to_string()))?;
                    for (px, dst) in rgb.chunks_exact(3).zip(rgba.chunks_exact_mut(4)) {
                        dst[0] = px[0];
                        dst[1] = px[1];
                        dst[2] = px[2];
                        dst[3] = 255;
                    }
                }
                Ok(Self {
                    frames: vec![AnimationFrame {
                        frame: Arc::new(DecodedFrame {
                            source_width: w,
                            source_height: h,
                            buffer_width: w,
                            buffer_height: h,
                            rgba8: rgba,
                            is_downscaled: false,
                            page_count: 1,
                            page_index: 0,
                            exif_orientation_applied: true,
                            color_profile_converted: false,
                            format_label: "WEBP".to_string(),
                            exif_items: Vec::new(),
                        }),
                        delay: Duration::from_millis(100),
                    }],
                    cursor: 0,
                    is_streaming_mode: false,
                })
            }
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn current_frame(&self) -> &DecodedFrame {
        &self.frames[self.cursor].frame
    }

    pub fn advance(&mut self) {
        if !self.frames.is_empty() {
            self.cursor = (self.cursor + 1) % self.frames.len();
        }
    }
}
