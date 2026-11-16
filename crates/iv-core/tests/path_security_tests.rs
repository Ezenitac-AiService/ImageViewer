mod fixtures;

use iv_core::path_security::{PathRedactor, PathSecurity};

#[test]
fn canonical_root_containment_rejects_sibling_prefix_bypass() {
    let dir = fixtures::TempTestDir::new("containment");
    let logs = dir.path().join("logs");
    let logs_evil = dir.path().join("logs-evil");
    std::fs::create_dir_all(&logs).expect("logs");
    std::fs::create_dir_all(&logs_evil).expect("logs-evil");

    let target_ok = logs.join("app.log");
    let target_evil = logs_evil.join("app.log");
    std::fs::write(&target_ok, b"ok").expect("ok");
    std::fs::write(&target_evil, b"evil").expect("evil");

    assert!(PathSecurity::is_within_root(&logs, &target_ok).expect("check ok"));
    assert!(!PathSecurity::is_within_root(&logs, &target_evil).expect("check evil"));
}

#[test]
fn archive_entry_key_normalization_rejects_traversal_and_symlinks() {
    assert_eq!(
        PathSecurity::normalize_archive_entry_key(r"folder\sub\01.png").expect("valid"),
        "folder/sub/01.png"
    );
    assert!(PathSecurity::normalize_archive_entry_key("../secret.png").is_err());
    assert!(PathSecurity::normalize_archive_entry_key("a/../../b.png").is_err());
    assert!(PathSecurity::normalize_archive_entry_key("/etc/passwd").is_err());
    assert!(PathSecurity::normalize_archive_entry_key("C:\\Windows\\win.ini").is_err());
    assert!(PathSecurity::normalize_archive_entry_key("bad\0key.png").is_err());

    // POSIX S_IFLNK (0xA000 << 16)
    assert!(PathSecurity::is_archive_symlink(0xA000_0000, false));
    // DOS reparse point (0x0400)
    assert!(PathSecurity::is_archive_symlink(0x0400, true));
    assert!(!PathSecurity::is_archive_symlink(0x81A4_0000, false));
}

#[test]
fn path_redactor_strips_paths_and_sanitizes_control_chars() {
    let raw = "Failed reading \"C:\\Users\\Alice Smith\\Pictures\\holiday\\photo 1.jpg\" and \\\\server\\share\\secret\\pic.png and /Users/alice/my dir/img.webp\r\nFAKE LOG";
    let redacted = PathRedactor::redact(raw);
    assert!(!redacted.contains("Alice Smith"));
    assert!(!redacted.contains("server"));
    assert!(!redacted.contains("/Users/alice"));
    assert!(!redacted.contains('\r'));
    assert!(!redacted.contains('\n'));
    assert!(redacted.contains("photo 1.jpg"));
    assert!(redacted.contains("pic.png"));
    assert!(redacted.contains("img.webp"));
}
