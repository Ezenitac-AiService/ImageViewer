#![allow(dead_code)]
mod fixtures;

use iv_media::color_hdr::ColorHdrNormalizer;
use iv_media::decode::{DecodeOptions, MediaDecoder};
use iv_media::resize_filter::SimdResizer;

#[test]
fn decodes_png_and_synthetic_cur_and_blocks_script_headers() {
    let png = fixtures::build_rgba_png(8, 4, &[200u8; 8 * 4 * 4]);
    let frame = MediaDecoder::decode_bytes(&png, "test.png", &DecodeOptions::default()).expect("png");
    assert_eq!(frame.source_width, 8);
    assert_eq!(frame.source_height, 4);

    let cur = fixtures::build_synthetic_cur(8, 4, &png);
    let cur_frame = MediaDecoder::decode_bytes(&cur, "cursor.cur", &DecodeOptions::default()).expect("cur");
    assert_eq!(cur_frame.source_width, 8);
    assert_eq!(cur_frame.source_height, 4);

    let polyglot = b"   <svg onload=alert(1)>";
    assert!(MediaDecoder::decode_bytes(polyglot, "evil.png", &DecodeOptions::default()).is_err());
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
