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

// ---------------------------------------------------------------------------
// specs/004 T090 (US6): Core content allowlist (contracts/input-acceptance-contract.md §1)
// ---------------------------------------------------------------------------

mod identify {
    use iv_core::format_guard::{CoreFormat, FormatGuard};

    fn with_tail(head: &[u8]) -> Vec<u8> {
        let mut v = head.to_vec();
        v.resize(64, 0x11);
        v
    }

    #[test]
    fn recognizes_each_core_signature() {
        let mut bmp = b"BM".to_vec();
        bmp.resize(14, 0);
        bmp.extend_from_slice(&40u32.to_le_bytes());
        bmp.resize(64, 0);
        let mut webp = b"RIFF".to_vec();
        webp.extend_from_slice(&[0x10, 0, 0, 0]);
        webp.extend_from_slice(b"WEBPVP8 ");
        let cases: Vec<(Vec<u8>, CoreFormat)> = vec![
            (with_tail(&[0xFF, 0xD8, 0xFF, 0xE0]), CoreFormat::Jpeg),
            (
                with_tail(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
                CoreFormat::Png,
            ),
            (with_tail(b"GIF87a"), CoreFormat::Gif),
            (with_tail(b"GIF89a"), CoreFormat::Gif),
            (with_tail(&webp), CoreFormat::WebP),
            (bmp, CoreFormat::Bmp),
            (with_tail(&[0x49, 0x49, 0x2A, 0x00]), CoreFormat::Tiff),
            (with_tail(&[0x4D, 0x4D, 0x00, 0x2A]), CoreFormat::Tiff),
            (with_tail(&[0x50, 0x4B, 0x03, 0x04]), CoreFormat::Zip),
            (with_tail(&[0x50, 0x4B, 0x05, 0x06]), CoreFormat::Zip),
        ];
        for (bytes, expected) in cases {
            assert_eq!(
                FormatGuard::identify(&bytes),
                Some(expected),
                "{:02x?}",
                &bytes[..8]
            );
        }
    }

    #[test]
    fn accepts_every_documented_bmp_dib_header_size() {
        for dib in [12u32, 40, 52, 56, 108, 124] {
            let mut bmp = b"BM".to_vec();
            bmp.resize(14, 0);
            bmp.extend_from_slice(&dib.to_le_bytes());
            bmp.resize(64, 0);
            assert_eq!(
                FormatGuard::identify(&bmp),
                Some(CoreFormat::Bmp),
                "dib={dib}"
            );
        }
    }

    #[test]
    fn rejects_unknown_truncated_and_lookalike_inputs() {
        let random: Vec<u8> = (0..1024u32)
            .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8 | 1)
            .collect();
        assert_eq!(FormatGuard::identify(&random), None);
        assert_eq!(FormatGuard::identify(&[]), None);
        assert_eq!(FormatGuard::identify(b"BM\x00\x01"), None, "truncated BMP");
        let mut bad_dib = b"BM".to_vec();
        bad_dib.resize(14, 0);
        bad_dib.extend_from_slice(&41u32.to_le_bytes());
        bad_dib.resize(64, 0);
        assert_eq!(FormatGuard::identify(&bad_dib), None, "unknown DIB size");
        assert_eq!(
            FormatGuard::identify(&with_tail(b"RIFF\x10\x00\x00\x00WAVEfmt ")),
            None
        );
        assert_eq!(FormatGuard::identify(&[0xFF, 0xD8]), None, "truncated JPEG");
        assert_eq!(FormatGuard::identify(&with_tail(b"GIF90a")), None);
    }
}
