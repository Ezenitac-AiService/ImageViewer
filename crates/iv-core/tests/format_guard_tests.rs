mod fixtures;

use iv_core::format_guard::FormatGuard;

#[test]
fn accepts_valid_raster_magic_headers() {
    let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13];
    let jpg = [0xFF, 0xD8, 0xFF, 0xE0, 0, 16, b'J', b'F', b'I', b'F'];
    assert!(FormatGuard::validate_header(&png).is_ok());
    assert!(FormatGuard::validate_header(&jpg).is_ok());
}

#[test]
fn blocks_utf32_and_utf16_and_utf8_bom_script_polyglots() {
    let u32_le = fixtures::utf32_le_bom_payload("  <svg onload=alert(1)>");
    let u32_be = fixtures::utf32_be_bom_payload("<?xml version=\"1.0\"?>");
    let u16_le = fixtures::utf16_le_bom_payload("<html/>");
    let u16_be = fixtures::utf16_be_bom_payload("%PDF-1.7");
    let u16_bomless = fixtures::utf16_bomless_le_payload("<svg>");
    let mut u8_bom = vec![0xEF, 0xBB, 0xBF];
    u8_bom.extend_from_slice(b"  viewbox 0 0 100 100");

    assert!(FormatGuard::validate_header(&u32_le).is_err());
    assert!(FormatGuard::validate_header(&u32_be).is_err());
    assert!(FormatGuard::validate_header(&u16_le).is_err());
    assert!(FormatGuard::validate_header(&u16_be).is_err());
    assert!(FormatGuard::validate_header(&u16_bomless).is_err());
    assert!(FormatGuard::validate_header(&u8_bom).is_err());
}

#[test]
fn blocks_excessive_whitespace_padding_and_mvg_postscript() {
    let padded = fixtures::whitespace_padded_payload(600, "not a script");
    assert!(FormatGuard::validate_header(&padded).is_err());
    assert!(FormatGuard::validate_header(b"push graphic-context").is_err());
    assert!(FormatGuard::validate_header(b"%!PS-Adobe-3.0").is_err());
}
