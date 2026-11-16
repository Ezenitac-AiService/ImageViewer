#![allow(dead_code)]
mod fixtures;

use iv_media::sources::MediaSource;
use iv_media::sources::folder::FolderSource;
use iv_media::sources::zip::ZipSource;

#[test]
fn folder_source_natural_sorts_and_skips_symlinks() {
    let dir = fixtures::TempTestDir::new("folder_src");
    let png = fixtures::build_rgba_png(2, 2, &[255; 16]);
    std::fs::write(dir.path().join("page10.png"), &png).expect("write 10");
    std::fs::write(dir.path().join("page2.png"), &png).expect("write 2");

    #[cfg(unix)]
    {
        let _ =
            std::os::unix::fs::symlink(dir.path().join("page2.png"), dir.path().join("link.png"));
    }

    let src = FolderSource::open(&dir.path().join("page2.png")).expect("open folder");
    let names: Vec<_> = src.entries().iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["page2.png", "page10.png"]);
}

#[test]
fn zip_source_enforces_preflight_eocd_caps_traversal_and_symlinks() {
    // 1. Pre-flight EOCD rejects > 50,000 entries or > 64 MB CD BEFORE ZipArchive::new
    let forged_entries = fixtures::build_forged_eocd_zip(55_000, 1024);
    assert!(ZipSource::preflight_eocd_check(&forged_entries).is_err());

    let forged_cd = fixtures::build_forged_eocd_zip(10, 70 * 1024 * 1024);
    assert!(ZipSource::preflight_eocd_check(&forged_cd).is_err());

    // 2. Rejects traversal keys and POSIX S_IFLNK symlinks inside zip
    let dir = fixtures::TempTestDir::new("zip_src");
    let zip_path = dir.path().join("test.cbz");
    let png = fixtures::build_rgba_png(2, 2, &[255; 16]);
    fixtures::write_zip_with_entries(
        &zip_path,
        &[
            ("01.png", &png, 0o100644 << 16),
            ("../escape.png", &png, 0o100644 << 16),
            ("sym.png", &png, 0xA000_0000),
        ],
    );

    let src = ZipSource::open(&zip_path, 512).expect("open valid zip");
    let names: Vec<_> = src.entries().iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["01.png"]);
    assert!(
        !src.read_entry_bytes(0)
            .expect("read reused handle")
            .is_empty()
    );

    // 3. Rejects ZIP64 EOCD claiming > 50,000 entries
    let mut zip64 = Vec::new();
    // ZIP64 EOCD record (56 bytes) at offset 0
    zip64.extend_from_slice(&0x0606_4b50u32.to_le_bytes());
    zip64.extend_from_slice(&44u64.to_le_bytes());
    zip64.extend_from_slice(&45u16.to_le_bytes());
    zip64.extend_from_slice(&45u16.to_le_bytes());
    zip64.extend_from_slice(&0u32.to_le_bytes());
    zip64.extend_from_slice(&0u32.to_le_bytes());
    zip64.extend_from_slice(&60_000u64.to_le_bytes());
    zip64.extend_from_slice(&60_000u64.to_le_bytes());
    zip64.extend_from_slice(&1024u64.to_le_bytes());
    zip64.extend_from_slice(&0u64.to_le_bytes());
    // ZIP64 EOCD locator (20 bytes)
    zip64.extend_from_slice(&0x0706_4b50u32.to_le_bytes());
    zip64.extend_from_slice(&0u32.to_le_bytes());
    zip64.extend_from_slice(&0u64.to_le_bytes());
    zip64.extend_from_slice(&1u32.to_le_bytes());
    // Standard EOCD with 0xFFFF sentinel
    zip64.extend_from_slice(&fixtures::build_forged_eocd_zip(0xFFFF, 1024));
    assert!(ZipSource::preflight_eocd_check(&zip64).is_err());
}

// ---------------------------------------------------------------------------
// specs/004 T038 (FR-018): every user-visible error has a Korean and an English message
// ---------------------------------------------------------------------------

#[test]
fn every_decode_source_and_animation_error_has_localized_user_messages() {
    use iv_core::codecs::OsCodecError;
    use iv_core::i18n::Language;
    use iv_media::animation::AnimationError;
    use iv_media::decode::DecodeError;
    use iv_media::sources::SourceError;

    let host_path = "/Users/bob/Private/secret-folder/photo.jpg";
    let messages: Vec<(String, String)> = vec![
        pair(&DecodeError::SecurityRejected(host_path.into())),
        pair(&DecodeError::ExceedsPixelLimit),
        pair(&DecodeError::DimensionOverflow),
        pair(&DecodeError::UnsupportedOrCorrupt(host_path.into())),
        pair(&DecodeError::OsCodec(OsCodecError::MissingCodec)),
        pair_s(&SourceError::Io(std::io::Error::other(host_path))),
        pair_s(&SourceError::Archive(host_path.into())),
        pair_s(&SourceError::EncryptedArchive),
        pair_s(&SourceError::ArchiveLimitExceeded(host_path.into())),
        pair_s(&SourceError::SecurityViolation),
        (
            AnimationError::Decode(host_path.into()).user_message(Language::Ko),
            AnimationError::Decode(host_path.into()).user_message(Language::En),
        ),
    ];
    for (ko, en) in messages {
        assert!(!ko.is_empty() && !en.is_empty());
        assert_ne!(ko, en);
        assert!(
            !ko.contains("/Users/bob") && !en.contains("/Users/bob"),
            "{en}"
        );
    }

    fn pair(e: &DecodeError) -> (String, String) {
        (e.user_message(Language::Ko), e.user_message(Language::En))
    }
    fn pair_s(e: &SourceError) -> (String, String) {
        (e.user_message(Language::Ko), e.user_message(Language::En))
    }
}
