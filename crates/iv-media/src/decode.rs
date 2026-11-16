//! `FormatGuard`-gated pure-Rust decoders, `.cur` parser, multi-page TIFF parity, and OS codec bridge.

use crate::cache::DecodedFrame;
use crate::color_hdr::ColorHdrNormalizer;
use crate::exif::ExifExtractor;
use crate::resize_filter::SimdResizer;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};
use iv_core::codecs::{OsCodecError, OsCodecProvider};
use iv_core::format_guard::FormatGuard;
use std::io::Cursor;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("blocked by format security guard: {0}")]
    SecurityRejected(String),
    #[error("image dimensions exceed configured safety limit")]
    ExceedsPixelLimit,
    #[error("arithmetic overflow validating pixel buffer dimensions")]
    DimensionOverflow,
    #[error("unsupported or corrupt image data: {0}")]
    UnsupportedOrCorrupt(String),
    #[error("host OS codec error: {0}")]
    OsCodec(#[from] OsCodecError),
}

impl DecodeError {
    /// Localized message for the user (FR-018). Never includes paths or internal detail; the
    /// English `Display` text is for the diagnostic record only.
    pub fn user_message(&self, lang: iv_core::i18n::Language) -> String {
        let key = match self {
            Self::SecurityRejected(_) => "error.security_rejected",
            Self::ExceedsPixelLimit | Self::DimensionOverflow => "error.too_large",
            Self::UnsupportedOrCorrupt(_) => "error.corrupt",
            Self::OsCodec(OsCodecError::MissingCodec) => "error.missing_os_codec",
            Self::OsCodec(OsCodecError::ExceedsPixelLimit) => "error.too_large",
            Self::OsCodec(OsCodecError::SecurityRejected) => "error.security_rejected",
            Self::OsCodec(_) => "error.corrupt",
        };
        iv_core::i18n::I18n::tr(lang, key).to_string()
    }
}

#[derive(Debug, Clone)]
pub struct DecodeOptions {
    pub max_megapixels: u32,
    pub target_viewport_physical: Option<(u32, u32)>,
    pub page_index: u32,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {
            max_megapixels: 150,
            target_viewport_physical: None,
            page_index: 0,
        }
    }
}

pub struct MediaDecoder;

impl MediaDecoder {
    /// Checks `checked_mul(width, 4)` and `width * height <= max_megapixels * 1_000_000` before allocation.
    pub fn check_pixel_cap(
        width: u32,
        height: u32,
        max_megapixels: u32,
    ) -> Result<(), DecodeError> {
        if width == 0 || height == 0 {
            return Err(DecodeError::UnsupportedOrCorrupt(
                "zero dimensions".to_string(),
            ));
        }
        width.checked_mul(4).ok_or(DecodeError::DimensionOverflow)?;
        let pixels = u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or(DecodeError::DimensionOverflow)?;
        let cap = u64::from(max_megapixels.clamp(10, 1000)) * 1_000_000;
        if pixels > cap {
            return Err(DecodeError::ExceedsPixelLimit);
        }
        Ok(())
    }

    /// Decodes `bytes` using pure-Rust core decoders (or optional `OsCodecProvider` if given).
    pub fn decode_bytes(
        bytes: &[u8],
        filename: &str,
        opts: &DecodeOptions,
    ) -> Result<DecodedFrame, DecodeError> {
        Self::decode_bytes_with_os(bytes, filename, opts, None)
    }

    pub fn decode_bytes_with_os(
        bytes: &[u8],
        filename: &str,
        opts: &DecodeOptions,
        os_provider: Option<&dyn OsCodecProvider>,
    ) -> Result<DecodedFrame, DecodeError> {
        // Mandatory Zero Trust 8 KB sniffer before ANY decoder or OS codec
        FormatGuard::validate_header(bytes)
            .map_err(|e| DecodeError::SecurityRejected(e.to_string()))?;

        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        if let Some(provider) = os_provider
            && provider.supports_extension(&ext)
        {
            if bytes.len() > 512 * 1024 * 1024 {
                return Err(DecodeError::ExceedsPixelLimit);
            }
            let os_buf = provider.decode_guarded(bytes, &ext, opts.max_megapixels)?;
            Self::check_pixel_cap(os_buf.width, os_buf.height, opts.max_megapixels)?;

            let mut img = image::RgbaImage::from_raw(os_buf.width, os_buf.height, os_buf.rgba8)
                .map(DynamicImage::ImageRgba8)
                .ok_or_else(|| {
                    DecodeError::UnsupportedOrCorrupt("invalid OS codec buffer".to_string())
                })?;

            // Skip duplicate EXIF rotation / ICC conversion if OS codec already performed them
            let mut orientation_applied = os_buf.orientation_applied;
            if !orientation_applied {
                let tag = os_buf
                    .exif_bytes
                    .as_deref()
                    .map(ExifExtractor::read_orientation_tag)
                    .unwrap_or(1);
                if tag != 1 {
                    apply_exif_orientation_tag(&mut img, tag);
                    orientation_applied = true;
                }
            }

            let source_width = img.width();
            let source_height = img.height();
            let exif_items = os_buf
                .exif_bytes
                .as_deref()
                .map(ExifExtractor::extract)
                .unwrap_or_default();

            let (rgba8, buffer_width, buffer_height, is_downscaled) = maybe_downscale_for_viewport(
                img.into_rgba8().into_raw(),
                source_width,
                source_height,
                opts,
            );

            return Ok(DecodedFrame {
                source_width,
                source_height,
                buffer_width,
                buffer_height,
                rgba8,
                is_downscaled,
                page_count: 1,
                page_index: 0,
                exif_orientation_applied: orientation_applied,
                color_profile_converted: os_buf.srgb_converted,
                format_label: ext.to_ascii_uppercase(),
                exif_items,
            });
        }

        let exif_items = ExifExtractor::extract(bytes);
        let (mut img, icc_opt, orientation_opt, page_count, page_index, format_label) =
            match ext.as_str() {
                "cur" => decode_cur_largest(bytes, opts.max_megapixels)?,
                "pcx" => decode_pcx(bytes, opts.max_megapixels)?,
                "dds" => decode_dds(bytes, opts.max_megapixels)?,
                "psd" => decode_psd(bytes, opts.max_megapixels)?,
                "jxl" => decode_jxl(bytes, opts.max_megapixels)?,
                "exr" => decode_exr(bytes, opts.max_megapixels)?,
                "tif" | "tiff" => decode_tiff_page(bytes, opts.page_index, opts.max_megapixels)?,
                "jpg" | "jpeg" | "jpe" | "jfif" => decode_jpeg_fast(bytes, opts.max_megapixels)?,
                _ => decode_standard_image(bytes, &ext, opts.max_megapixels)?,
            };

        let mut orientation_applied = false;
        if let Some(orient) = orientation_opt
            && orient != Orientation::NoTransforms
        {
            // Check swapped dimensions on Orientation 5-8 before allocation/transform
            let (w, h) = (img.width(), img.height());
            if matches!(
                orient,
                Orientation::Rotate90
                    | Orientation::Rotate270
                    | Orientation::Rotate90FlipH
                    | Orientation::Rotate270FlipH
            ) {
                Self::check_pixel_cap(h, w, opts.max_megapixels)?;
            }
            img.apply_orientation(orient);
            orientation_applied = true;
        }

        let source_width = img.width();
        let source_height = img.height();
        let mut rgba_raw = img.into_rgba8().into_raw();

        let mut color_converted = false;
        if let Some(icc) = icc_opt.as_deref() {
            color_converted = ColorHdrNormalizer::convert_icc_rgba8_to_srgb(&mut rgba_raw, icc);
        }

        let (rgba8, buffer_width, buffer_height, is_downscaled) =
            maybe_downscale_for_viewport(rgba_raw, source_width, source_height, opts);

        Ok(DecodedFrame {
            source_width,
            source_height,
            buffer_width,
            buffer_height,
            rgba8,
            is_downscaled,
            page_count,
            page_index,
            exif_orientation_applied: orientation_applied,
            color_profile_converted: color_converted,
            format_label,
            exif_items,
        })
    }
}

type RawDecodeResult = (
    DynamicImage,
    Option<Vec<u8>>,
    Option<Orientation>,
    u32,
    u32,
    String,
);

fn decode_jpeg_fast(bytes: &[u8], max_mp: u32) -> Result<RawDecodeResult, DecodeError> {
    let mut decoder = zune_jpeg::JpegDecoder::new(bytes);
    decoder
        .decode_headers()
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(format!("jpeg headers: {e}")))?;
    let (w, h) = decoder
        .dimensions()
        .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("missing jpeg dimensions".to_string()))?;
    MediaDecoder::check_pixel_cap(w as u32, h as u32, max_mp)?;

    let icc = decoder.icc_profile();
    let tag = ExifExtractor::read_orientation_tag(bytes);
    let orient = orientation_from_tag(tag);

    let pixels = decoder
        .decode()
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(format!("jpeg decode: {e}")))?;
    let expected_rgb = w * h * 3;
    let expected_rgba = w * h * 4;
    let expected_luma = w * h;
    let dyn_img = if pixels.len() == expected_rgb {
        image::RgbImage::from_raw(w as u32, h as u32, pixels)
            .map(DynamicImage::ImageRgb8)
            .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("jpeg rgb buffer".to_string()))?
    } else if pixels.len() == expected_luma {
        image::GrayImage::from_raw(w as u32, h as u32, pixels)
            .map(DynamicImage::ImageLuma8)
            .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("jpeg luma buffer".to_string()))?
    } else if pixels.len() == expected_rgba {
        // Fallback to `image` crate for CMYK/YCCK JPEGs
        let mut reader = ImageReader::new(Cursor::new(bytes));
        reader.set_format(ImageFormat::Jpeg);
        reader
            .decode()
            .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?
    } else {
        return Err(DecodeError::UnsupportedOrCorrupt(
            "unexpected jpeg buffer length".to_string(),
        ));
    };

    Ok((dyn_img, icc, orient, 1, 0, "JPEG".to_string()))
}

fn decode_standard_image(
    bytes: &[u8],
    ext: &str,
    max_mp: u32,
) -> Result<RawDecodeResult, DecodeError> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    let fmt = reader.format();
    let mut decoder = reader
        .into_decoder()
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    let (w, h) = decoder.dimensions();
    MediaDecoder::check_pixel_cap(w, h, max_mp)?;
    let orient = decoder.orientation().ok();
    let icc = decoder.icc_profile().ok().flatten();
    let img = DynamicImage::from_decoder(decoder)
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    let label = fmt
        .map(|f| format!("{f:?}").to_ascii_uppercase())
        .unwrap_or_else(|| ext.to_ascii_uppercase());
    Ok((img, icc, orient, 1, 0, label))
}

fn decode_tiff_page(
    bytes: &[u8],
    target_page: u32,
    max_mp: u32,
) -> Result<RawDecodeResult, DecodeError> {
    let mut dec = tiff::decoder::Decoder::new(Cursor::new(bytes))
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    let mut total_pages = 1u32;
    while dec.more_images() && dec.next_image().is_ok() {
        total_pages = total_pages.saturating_add(1);
    }

    let clamped_page = target_page.min(total_pages.saturating_sub(1));
    let mut page_dec = tiff::decoder::Decoder::new(Cursor::new(bytes))
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    for _ in 0..clamped_page {
        page_dec
            .next_image()
            .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    }

    let (w, h) = page_dec
        .dimensions()
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    MediaDecoder::check_pixel_cap(w, h, max_mp)?;

    let orient_val = page_dec
        .get_tag_u32(tiff::tags::Tag::Orientation)
        .ok()
        .map(|v| v as u16)
        .unwrap_or(1);
    let orient = orientation_from_tag(orient_val);
    let icc = page_dec
        .get_tag_u8_vec(tiff::tags::Tag::Unknown(34675))
        .ok();

    let res = page_dec
        .read_image()
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;

    let dyn_img = match res {
        tiff::decoder::DecodingResult::U8(buf) => {
            let ct = page_dec.colortype().unwrap_or(tiff::ColorType::RGBA(8));
            match ct {
                tiff::ColorType::RGB(8) => image::RgbImage::from_raw(w, h, buf)
                    .map(DynamicImage::ImageRgb8)
                    .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("tiff rgb8".to_string()))?,
                tiff::ColorType::RGBA(8) => image::RgbaImage::from_raw(w, h, buf)
                    .map(DynamicImage::ImageRgba8)
                    .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("tiff rgba8".to_string()))?,
                tiff::ColorType::Gray(8) => image::GrayImage::from_raw(w, h, buf)
                    .map(DynamicImage::ImageLuma8)
                    .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("tiff luma8".to_string()))?,
                tiff::ColorType::CMYK(8) => {
                    let srgb =
                        ColorHdrNormalizer::convert_cmyk8_to_srgb_rgba8(&buf, icc.as_deref());
                    image::RgbaImage::from_raw(w, h, srgb)
                        .map(DynamicImage::ImageRgba8)
                        .ok_or_else(|| {
                            DecodeError::UnsupportedOrCorrupt("tiff cmyk8".to_string())
                        })?
                }
                _ => {
                    return Err(DecodeError::UnsupportedOrCorrupt(
                        "unsupported tiff u8 color layout".to_string(),
                    ));
                }
            }
        }
        tiff::decoder::DecodingResult::U16(buf) => {
            let u8_buf = ColorHdrNormalizer::normalize_rgba16_to_rgba8(&buf);
            let ct = page_dec.colortype().unwrap_or(tiff::ColorType::RGBA(16));
            match ct {
                tiff::ColorType::CMYK(16) => {
                    let srgb =
                        ColorHdrNormalizer::convert_cmyk8_to_srgb_rgba8(&u8_buf, icc.as_deref());
                    image::RgbaImage::from_raw(w, h, srgb)
                        .map(DynamicImage::ImageRgba8)
                        .ok_or_else(|| {
                            DecodeError::UnsupportedOrCorrupt("tiff cmyk16".to_string())
                        })?
                }
                tiff::ColorType::RGBA(16) => image::RgbaImage::from_raw(w, h, u8_buf)
                    .map(DynamicImage::ImageRgba8)
                    .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("tiff rgba16".to_string()))?,
                tiff::ColorType::RGB(16) => image::RgbImage::from_raw(w, h, u8_buf)
                    .map(DynamicImage::ImageRgb8)
                    .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("tiff rgb16".to_string()))?,
                _ => {
                    return Err(DecodeError::UnsupportedOrCorrupt(
                        "unsupported tiff u16 color layout".to_string(),
                    ));
                }
            }
        }
        _ => {
            return Err(DecodeError::UnsupportedOrCorrupt(
                "unsupported tiff sample format".to_string(),
            ));
        }
    };

    Ok((
        dyn_img,
        icc,
        orient,
        total_pages,
        clamped_page,
        "TIFF".to_string(),
    ))
}

fn decode_cur_largest(bytes: &[u8], max_mp: u32) -> Result<RawDecodeResult, DecodeError> {
    if bytes.len() < 6 {
        return Err(DecodeError::UnsupportedOrCorrupt(
            "truncated .cur".to_string(),
        ));
    }
    let reserved = u16::from_le_bytes([bytes[0], bytes[1]]);
    let img_type = u16::from_le_bytes([bytes[2], bytes[3]]);
    let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    if reserved != 0 || img_type != 2 || count == 0 {
        return Err(DecodeError::UnsupportedOrCorrupt(
            "invalid .cur header".to_string(),
        ));
    }
    let dir_end = 6usize
        .checked_add(count * 16)
        .ok_or(DecodeError::DimensionOverflow)?;
    if bytes.len() < dir_end {
        return Err(DecodeError::UnsupportedOrCorrupt(
            "truncated .cur directory".to_string(),
        ));
    }

    let mut best: Option<(u32, usize, usize)> = None;
    for i in 0..count {
        let off = 6 + i * 16;
        let w = if bytes[off] == 0 {
            256u32
        } else {
            u32::from(bytes[off])
        };
        let h = if bytes[off + 1] == 0 {
            256u32
        } else {
            u32::from(bytes[off + 1])
        };
        let size = u32::from_le_bytes([
            bytes[off + 8],
            bytes[off + 9],
            bytes[off + 10],
            bytes[off + 11],
        ]) as usize;
        let img_off = u32::from_le_bytes([
            bytes[off + 12],
            bytes[off + 13],
            bytes[off + 14],
            bytes[off + 15],
        ]) as usize;
        let Some(end) = img_off.checked_add(size) else {
            continue;
        };
        if end <= bytes.len() {
            let area = w.saturating_mul(h);
            if best.is_none_or(|(b_area, _, _)| area > b_area) {
                best = Some((area, img_off, end));
            }
        }
    }

    let (_, start, end) =
        best.ok_or_else(|| DecodeError::UnsupportedOrCorrupt("no valid .cur frame".to_string()))?;
    let payload = &bytes[start..end];
    if payload.starts_with(&[0x89, b'P', b'N', b'G']) {
        let mut res = decode_standard_image(payload, "png", max_mp)?;
        res.5 = "CUR".to_string();
        Ok(res)
    } else {
        // Re-tag ICO type=1 so `image` ICO decoder parses DIB/XOR/AND masks cleanly
        let mut ico_buf = Vec::with_capacity(22 + payload.len());
        ico_buf.extend_from_slice(&0u16.to_le_bytes());
        ico_buf.extend_from_slice(&1u16.to_le_bytes());
        ico_buf.extend_from_slice(&1u16.to_le_bytes());
        ico_buf.extend_from_slice(&bytes[6..22]);
        ico_buf[18..22].copy_from_slice(&22u32.to_le_bytes());
        ico_buf.extend_from_slice(payload);
        let mut res = decode_standard_image(&ico_buf, "ico", max_mp)?;
        res.5 = "CUR".to_string();
        Ok(res)
    }
}

fn decode_pcx(bytes: &[u8], max_mp: u32) -> Result<RawDecodeResult, DecodeError> {
    let dec = image_extras::pcx::PCXDecoder::new(Cursor::new(bytes))
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    let (w, h) = dec.dimensions();
    MediaDecoder::check_pixel_cap(w, h, max_mp)?;
    let img = DynamicImage::from_decoder(dec)
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    Ok((img, None, None, 1, 0, "PCX".to_string()))
}

fn decode_dds(bytes: &[u8], max_mp: u32) -> Result<RawDecodeResult, DecodeError> {
    let dds = image_dds::ddsfile::Dds::read(&mut Cursor::new(bytes))
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    MediaDecoder::check_pixel_cap(dds.get_width(), dds.get_height(), max_mp)?;
    let rgba = image_dds::image_from_dds(&dds, 0)
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    Ok((
        DynamicImage::ImageRgba8(rgba),
        None,
        None,
        1,
        0,
        "DDS".to_string(),
    ))
}

fn decode_psd(bytes: &[u8], max_mp: u32) -> Result<RawDecodeResult, DecodeError> {
    let meta = rawpsd::parse_psd_metadata(bytes).map_err(DecodeError::UnsupportedOrCorrupt)?;
    let w = meta.width;
    let h = meta.height;
    MediaDecoder::check_pixel_cap(w, h, max_mp)?;
    let layers = rawpsd::parse_layer_records(bytes).unwrap_or_default();
    let mut canvas = vec![0u8; (w as usize) * (h as usize) * 4];
    let mut drew_layer = false;
    for layer in layers {
        if !layer.image_data_rgba.is_empty() {
            let lw = layer.w;
            let lh = layer.h;
            let copy_w = lw.min(w);
            let copy_h = lh.min(h);
            for y in 0..copy_h as usize {
                let src_start = y * (lw as usize) * 4;
                let src_end = src_start + (copy_w as usize) * 4;
                if src_end <= layer.image_data_rgba.len() {
                    let dst_start = y * (w as usize) * 4;
                    canvas[dst_start..dst_start + (copy_w as usize) * 4]
                        .copy_from_slice(&layer.image_data_rgba[src_start..src_end]);
                }
            }
            drew_layer = true;
            break;
        }
    }
    if !drew_layer {
        decode_psd_composite_fallback(bytes, w, h, &mut canvas)?;
    }
    let img = image::RgbaImage::from_raw(w, h, canvas)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("invalid psd rgba".to_string()))?;
    Ok((img, None, None, 1, 0, "PSD".to_string()))
}

fn decode_psd_composite_fallback(
    bytes: &[u8],
    w: u32,
    h: u32,
    canvas: &mut [u8],
) -> Result<(), DecodeError> {
    if bytes.len() < 34 || &bytes[0..4] != b"8BPS" {
        return Err(DecodeError::UnsupportedOrCorrupt(
            "invalid psd header".to_string(),
        ));
    }
    let channels = u16::from_be_bytes([bytes[12], bytes[13]]) as usize;
    let mut pos = 26usize;
    for _ in 0..3 {
        if pos + 4 > bytes.len() {
            return Err(DecodeError::UnsupportedOrCorrupt(
                "truncated psd section".to_string(),
            ));
        }
        let sec_len =
            u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]])
                as usize;
        pos = pos
            .checked_add(4 + sec_len)
            .ok_or(DecodeError::DimensionOverflow)?;
    }
    if pos + 2 > bytes.len() {
        return Err(DecodeError::UnsupportedOrCorrupt(
            "missing psd image data".to_string(),
        ));
    }
    let compression = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]);
    pos += 2;
    let plane_len = (w as usize) * (h as usize);
    let mut planes = Vec::with_capacity(channels);
    if compression == 0 {
        for _ in 0..channels.min(4) {
            let end = pos + plane_len;
            if end > bytes.len() {
                break;
            }
            planes.push(bytes[pos..end].to_vec());
            pos = end;
        }
    } else if compression == 1 {
        let byte_count_table = channels * (h as usize) * 2;
        pos = pos.saturating_add(byte_count_table);
        for _ in 0..channels.min(4) {
            let mut plane = Vec::with_capacity(plane_len);
            while plane.len() < plane_len && pos < bytes.len() {
                let n = bytes[pos] as i8;
                pos += 1;
                if n >= 0 {
                    let count = (n as usize) + 1;
                    let end = (pos + count).min(bytes.len());
                    plane.extend_from_slice(&bytes[pos..end]);
                    pos = end;
                } else if n != -128 && pos < bytes.len() {
                    let count = (1 - i16::from(n)) as usize;
                    let val = bytes[pos];
                    pos += 1;
                    plane.resize((plane.len() + count).min(plane_len), val);
                }
            }
            planes.push(plane);
        }
    }
    if planes.is_empty() {
        return Err(DecodeError::UnsupportedOrCorrupt(
            "empty psd composite".to_string(),
        ));
    }
    for i in 0..plane_len {
        let r = planes[0].get(i).copied().unwrap_or(0);
        let g = planes.get(1).and_then(|p| p.get(i).copied()).unwrap_or(r);
        let b = planes.get(2).and_then(|p| p.get(i).copied()).unwrap_or(r);
        let a = planes.get(3).and_then(|p| p.get(i).copied()).unwrap_or(255);
        canvas[i * 4..i * 4 + 4].copy_from_slice(&[r, g, b, a]);
    }
    Ok(())
}

fn decode_jxl(bytes: &[u8], max_mp: u32) -> Result<RawDecodeResult, DecodeError> {
    let jxl = jxl_oxide::JxlImage::builder()
        .read(Cursor::new(bytes))
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    let w = jxl.width();
    let h = jxl.height();
    MediaDecoder::check_pixel_cap(w, h, max_mp)?;
    let render = jxl
        .render_frame(0)
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    let fb = render.image_all_channels();
    let float_buf = fb.buf();
    let channels = fb.channels();
    let mut rgba = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for px in float_buf.chunks_exact(channels) {
        let r = (px[0].clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (px.get(1).copied().unwrap_or(px[0]).clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (px.get(2).copied().unwrap_or(px[0]).clamp(0.0, 1.0) * 255.0).round() as u8;
        let a = (px.get(3).copied().unwrap_or(1.0).clamp(0.0, 1.0) * 255.0).round() as u8;
        rgba.extend_from_slice(&[r, g, b, a]);
    }
    let img = image::RgbaImage::from_raw(w, h, rgba)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("invalid jxl buffer".to_string()))?;
    Ok((img, None, None, 1, 0, "JXL".to_string()))
}

fn decode_exr(bytes: &[u8], max_mp: u32) -> Result<RawDecodeResult, DecodeError> {
    let mut reader = ImageReader::new(Cursor::new(bytes));
    reader.set_format(ImageFormat::OpenExr);
    let dec = reader
        .into_decoder()
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    let (w, h) = dec.dimensions();
    MediaDecoder::check_pixel_cap(w, h, max_mp)?;
    let img = DynamicImage::from_decoder(dec)
        .map_err(|e| DecodeError::UnsupportedOrCorrupt(e.to_string()))?;
    let f32_buf = img.into_rgba32f().into_raw();
    let tonemapped = ColorHdrNormalizer::tonemap_rgba_f32_to_srgb(&f32_buf);
    let ldr = image::RgbaImage::from_raw(w, h, tonemapped)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| DecodeError::UnsupportedOrCorrupt("invalid exr buffer".to_string()))?;
    Ok((ldr, None, None, 1, 0, "EXR".to_string()))
}

fn maybe_downscale_for_viewport(
    rgba_raw: Vec<u8>,
    source_w: u32,
    source_h: u32,
    opts: &DecodeOptions,
) -> (Vec<u8>, u32, u32, bool) {
    if let Some((vp_w, vp_h)) = opts.target_viewport_physical
        && vp_w > 0
        && vp_h > 0
    {
        let ratio_w = f64::from(vp_w) / f64::from(source_w.max(1));
        let ratio_h = f64::from(vp_h) / f64::from(source_h.max(1));
        let scale = ratio_w.min(ratio_h);
        if scale < 0.60 {
            let dst_w = ((f64::from(source_w) * scale).round() as u32).max(1);
            let dst_h = ((f64::from(source_h) * scale).round() as u32).max(1);
            if let Ok(down) =
                SimdResizer::downscale_rgba8(&rgba_raw, source_w, source_h, dst_w, dst_h)
            {
                return (down, dst_w, dst_h, true);
            }
        }
    }
    (rgba_raw, source_w, source_h, false)
}

fn orientation_from_tag(tag: u16) -> Option<Orientation> {
    match tag {
        2 => Some(Orientation::FlipHorizontal),
        3 => Some(Orientation::Rotate180),
        4 => Some(Orientation::FlipVertical),
        5 => Some(Orientation::Rotate90FlipH),
        6 => Some(Orientation::Rotate90),
        7 => Some(Orientation::Rotate270FlipH),
        8 => Some(Orientation::Rotate270),
        _ => None,
    }
}

fn apply_exif_orientation_tag(img: &mut DynamicImage, tag: u16) {
    if let Some(orient) = orientation_from_tag(tag) {
        img.apply_orientation(orient);
    }
}
