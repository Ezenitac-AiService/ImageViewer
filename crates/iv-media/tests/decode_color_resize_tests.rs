#![allow(dead_code)]
mod fixtures;

use iv_media::color_hdr::ColorHdrNormalizer;
use iv_media::decode::{DecodeOptions, MediaDecoder};
use iv_media::resize_filter::SimdResizer;

#[test]
fn decodes_png_and_blocks_script_headers() {
    let png = fixtures::build_rgba_png(8, 4, &[200u8; 8 * 4 * 4]);
    let frame =
        MediaDecoder::decode_bytes(&png, "test.png", &DecodeOptions::default()).expect("png");
    assert_eq!(frame.source_width, 8);
    assert_eq!(frame.source_height, 4);

    let polyglot = b"   <svg onload=alert(1)>";
    assert!(MediaDecoder::decode_bytes(polyglot, "evil.png", &DecodeOptions::default()).is_err());
}

#[cfg(feature = "extended-formats")]
#[test]
fn decodes_synthetic_cur() {
    let png = fixtures::build_rgba_png(8, 4, &[200u8; 8 * 4 * 4]);
    let cur = fixtures::build_synthetic_cur(8, 4, &png);
    let cur_frame =
        MediaDecoder::decode_bytes(&cur, "cursor.cur", &DecodeOptions::default()).expect("cur");
    assert_eq!(cur_frame.source_width, 8);
    assert_eq!(cur_frame.source_height, 4);
}

#[test]
fn enforces_pixel_cap_and_performs_simd_downscale_and_hdr_tonemap() {
    assert!(MediaDecoder::check_pixel_cap(50_000, 50_000, 150).is_err());
    assert!(MediaDecoder::check_pixel_cap(1920, 1080, 150).is_ok());

    let src = vec![128u8; 64 * 64 * 4];
    let down = SimdResizer::downscale_rgba8(&src, 64, 64, 16, 16).expect("simd resize");
    assert_eq!(down.len(), 16 * 16 * 4);

    let hdr = vec![2.0f32, 0.5, 0.0, 1.0];
    let ldr = ColorHdrNormalizer::tonemap_rgba_f32_to_srgb(&hdr);
    assert_eq!(ldr.len(), 4);
    assert!(ldr[0] > ldr[1]);
}

#[cfg(feature = "extended-formats")]
#[test]
fn decodes_flattened_composite_psd_without_layers() {
    // Minimal 2x2 RGB 8-bit flattened PSD (8BPS v1, no color mode/resources/layers, raw compression=0)
    let mut psd = Vec::new();
    psd.extend_from_slice(b"8BPS");
    psd.extend_from_slice(&1u16.to_be_bytes()); // version 1
    psd.extend_from_slice(&[0u8; 6]); // reserved
    psd.extend_from_slice(&3u16.to_be_bytes()); // 3 channels (RGB)
    psd.extend_from_slice(&2u32.to_be_bytes()); // height = 2
    psd.extend_from_slice(&2u32.to_be_bytes()); // width = 2
    psd.extend_from_slice(&8u16.to_be_bytes()); // depth = 8
    psd.extend_from_slice(&3u16.to_be_bytes()); // color mode = RGB (3)
    psd.extend_from_slice(&0u32.to_be_bytes()); // color mode data len
    psd.extend_from_slice(&0u32.to_be_bytes()); // image resources len
    psd.extend_from_slice(&0u32.to_be_bytes()); // layer and mask info len (0 = flattened)
    psd.extend_from_slice(&0u16.to_be_bytes()); // compression = 0 (Raw)
    // Planar R(4), G(4), B(4)
    psd.extend_from_slice(&[200, 200, 200, 200]);
    psd.extend_from_slice(&[100, 100, 100, 100]);
    psd.extend_from_slice(&[50, 50, 50, 50]);

    let frame =
        MediaDecoder::decode_bytes(&psd, "flat.psd", &DecodeOptions::default()).expect("flat psd");
    assert_eq!(frame.source_width, 2);
    assert_eq!(frame.source_height, 2);
    assert_eq!(&frame.rgba8[0..4], &[200, 100, 50, 255]);
}

// ---------------------------------------------------------------------------
// specs/004 T080 (US5): extended formats are refused in the shipped build
// ---------------------------------------------------------------------------

#[cfg(not(feature = "extended-formats"))]
#[test]
fn extended_format_signatures_are_unsupported_in_the_default_build() {
    use iv_media::decode::DecodeError;
    let mut qoi = b"qoif".to_vec();
    qoi.extend_from_slice(&1u32.to_be_bytes());
    qoi.extend_from_slice(&1u32.to_be_bytes());
    qoi.extend_from_slice(&[4, 0, 0xFF, 1, 2, 3, 255, 0, 0, 0, 0, 0, 0, 0, 1]);
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("a.qoi", qoi),
        (
            "a.jxl",
            vec![0xFF, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        ),
        ("a.psd", b"8BPS\x00\x01\x00\x00\x00\x00\x00\x00".to_vec()),
        ("a.dds", b"DDS |\x00\x00\x00".to_vec()),
        ("a.pcx", vec![0x0A, 0x05, 0x01, 0x08, 0, 0, 0, 0]),
        (
            "a.tga",
            vec![
                0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 32, 8, 1, 2, 3, 4,
            ],
        ),
    ];
    for (name, bytes) in cases {
        let err =
            MediaDecoder::decode_bytes(&bytes, name, &DecodeOptions::default()).expect_err(name);
        assert!(
            matches!(err, DecodeError::UnsupportedFormat),
            "{name}: {err:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// specs/004 T091–T092 (US6): content allowlist and pre-allocation limits
// ---------------------------------------------------------------------------

fn encode(format: image::ImageFormat) -> Vec<u8> {
    let rgba = image::RgbaImage::from_pixel(4, 3, image::Rgba([10, 20, 30, 255]));
    let img = match format {
        image::ImageFormat::Jpeg => {
            image::DynamicImage::ImageRgb8(image::DynamicImage::ImageRgba8(rgba).to_rgb8())
        }
        _ => image::DynamicImage::ImageRgba8(rgba),
    };
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), format)
        .expect("encode fixture");
    buf
}

const CORE: [image::ImageFormat; 6] = [
    image::ImageFormat::Jpeg,
    image::ImageFormat::Png,
    image::ImageFormat::Gif,
    image::ImageFormat::WebP,
    image::ImageFormat::Bmp,
    image::ImageFormat::Tiff,
];

#[test]
fn identify_agrees_with_image_guess_format_for_every_core_fixture() {
    use iv_core::format_guard::{CoreFormat, FormatGuard};
    for format in CORE {
        let bytes = encode(format);
        let ours = FormatGuard::identify(&bytes).expect("identified");
        let expected = match format {
            image::ImageFormat::Jpeg => CoreFormat::Jpeg,
            image::ImageFormat::Png => CoreFormat::Png,
            image::ImageFormat::Gif => CoreFormat::Gif,
            image::ImageFormat::WebP => CoreFormat::WebP,
            image::ImageFormat::Bmp => CoreFormat::Bmp,
            _ => CoreFormat::Tiff,
        };
        assert_eq!(ours, expected, "{format:?}");
        assert_eq!(image::guess_format(&bytes).expect("guess"), format);
        let frame = MediaDecoder::decode_bytes(&bytes, "fixture.bin", &DecodeOptions::default())
            .unwrap_or_else(|e| panic!("{format:?}: {e}"));
        assert_eq!(
            (frame.source_width, frame.source_height),
            (4, 3),
            "{format:?}"
        );
    }
}

#[test]
fn script_and_document_payloads_named_png_are_refused() {
    use iv_media::decode::DecodeError;
    for payload in [
        &b"<svg xmlns='http://www.w3.org/2000/svg'/>"[..],
        b"<!DOCTYPE html><html></html>",
        b"%PDF-1.7\n",
        b"%!PS-Adobe-3.0\n",
    ] {
        let err = MediaDecoder::decode_bytes(payload, "x.png", &DecodeOptions::default())
            .expect_err("refused");
        assert!(matches!(err, DecodeError::SecurityRejected(_)), "{err:?}");
    }
}

#[test]
fn random_bytes_named_jpg_are_unsupported_without_decoding() {
    use iv_media::decode::DecodeError;
    let bytes: Vec<u8> = (0..1024u32)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8 | 1)
        .collect();
    let err = MediaDecoder::decode_bytes(&bytes, "photo.jpg", &DecodeOptions::default())
        .expect_err("refused");
    assert!(matches!(err, DecodeError::UnsupportedFormat), "{err:?}");
}

#[test]
fn a_png_named_jpg_is_decoded_as_png() {
    let png = fixtures::build_rgba_png(3, 2, &[1u8; 3 * 2 * 4]);
    let frame = MediaDecoder::decode_bytes(&png, "really-a-png.jpg", &DecodeOptions::default())
        .expect("decoded by content");
    assert_eq!((frame.source_width, frame.source_height), (3, 2));
    assert_eq!(frame.format_label, "PNG");
}

#[test]
fn huge_declared_png_dimensions_fail_before_allocation() {
    use iv_media::decode::DecodeError;
    // Valid PNG signature + IHDR declaring 100000 x 100000 (CRC not needed: rejected at header).
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    png.extend_from_slice(&13u32.to_be_bytes());
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&100_000u32.to_be_bytes());
    png.extend_from_slice(&100_000u32.to_be_bytes());
    png.extend_from_slice(&[8, 6, 0, 0, 0]);
    let crc = crc32(&png[12..]);
    png.extend_from_slice(&crc.to_be_bytes());
    // A tiny IDAT so the decoder reaches the point where it reports dimensions.
    let idat_start = png.len();
    png.extend_from_slice(&2u32.to_be_bytes());
    png.extend_from_slice(b"IDAT");
    png.extend_from_slice(&[0x78, 0x9C]);
    let crc = crc32(&png[idat_start + 4..]);
    png.extend_from_slice(&crc.to_be_bytes());
    png.extend_from_slice(&0u32.to_be_bytes());
    png.extend_from_slice(b"IEND");
    png.extend_from_slice(&0xAE42_6082u32.to_be_bytes());
    let err = MediaDecoder::decode_bytes(&png, "big.png", &DecodeOptions::default())
        .expect_err("refused");
    assert!(
        matches!(
            err,
            DecodeError::ExceedsPixelLimit | DecodeError::DimensionOverflow
        ),
        "{err:?}"
    );
}

#[test]
fn bmp_magic_with_a_truncated_header_is_unsupported() {
    use iv_media::decode::DecodeError;
    let err = MediaDecoder::decode_bytes(b"BM\x00\x01", "x.bmp", &DecodeOptions::default())
        .expect_err("refused");
    assert!(matches!(err, DecodeError::UnsupportedFormat), "{err:?}");
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
