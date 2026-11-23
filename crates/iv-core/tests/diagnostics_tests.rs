//! Launch-record grammar tests (`specs/004-launch-reliability-refactor/contracts/launch-and-diagnostics-contract.md` §3).

mod fixtures;

use iv_core::diagnostics::{ExitCode, FatalCategory, LaunchId, LaunchRecord, Phase, source_name};
use iv_core::logger::AsyncFileLogger;

fn record() -> LaunchRecord {
    LaunchRecord::new(LaunchId::from_u32(0x00ab_cdef))
}

#[test]
fn launch_id_is_exactly_eight_lowercase_hex_chars() {
    assert_eq!(LaunchId::from_u32(0x00ab_cdef).as_str(), "00abcdef");
    for _ in 0..32 {
        let id = LaunchId::random();
        assert_eq!(id.as_str().len(), 8);
        assert!(
            id.as_str()
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }
}

#[test]
fn start_line_carries_version_os_os_version_and_arch() {
    let line = record().start("1.1.0-alpha.2", "macos", "27.2.1", "aarch64");
    assert_eq!(
        line,
        "launch=00abcdef phase=start version=1.1.0-alpha.2 os=macos os_version=27.2.1 arch=aarch64"
    );
}

#[test]
fn phase_names_match_contract_grammar() {
    let cases = [
        (Phase::Start, "start"),
        (Phase::Settings, "settings"),
        (Phase::Probe, "probe"),
        (Phase::Window, "window"),
        (Phase::FirstFrame, "first_frame"),
        (Phase::Navigate, "navigate"),
    ];
    for (phase, name) in cases {
        assert_eq!(phase.as_str(), name);
        assert_eq!(
            record().phase(phase, &[]),
            format!("launch=00abcdef phase={name}")
        );
    }
}

#[test]
fn values_with_spaces_are_quoted_and_inner_quotes_removed() {
    let line = record().phase(
        Phase::Probe,
        &[
            ("renderer", "wgpu"),
            ("adapter", "Apple \"M3\" Pro"),
            ("type", "integrated"),
        ],
    );
    assert_eq!(
        line,
        "launch=00abcdef phase=probe renderer=wgpu adapter=\"Apple M3 Pro\" type=integrated"
    );
}

#[test]
fn source_keeps_file_name_only_for_unix_and_windows_paths() {
    assert_eq!(
        source_name("/Users/bob/Private/vacation.jpg"),
        "vacation.jpg"
    );
    assert_eq!(source_name(r"C:\Users\bob\Pictures\cat.png"), "cat.png");
    assert_eq!(source_name("reference.png"), "reference.png");
    assert_eq!(source_name(""), "none");
    let line = record().phase(
        Phase::FirstFrame,
        &[
            ("source", &source_name("/Users/bob/x/reference.png")),
            ("w", "96"),
            ("h", "96"),
        ],
    );
    assert_eq!(
        line,
        "launch=00abcdef phase=first_frame source=reference.png w=96 h=96"
    );
}

#[test]
fn values_never_carry_control_characters_or_host_paths() {
    let line = record().phase(
        Phase::Navigate,
        &[("note", "a\r\nINJECT /Users/bob/secret/dir/file.jpg")],
    );
    assert!(!line.contains('\r') && !line.contains('\n'));
    assert!(!line.contains("/Users/bob"));
    assert!(line.contains("file.jpg"));
}

#[test]
fn fatal_categories_and_clean_exit_lines() {
    let cases = [
        (FatalCategory::NoRenderer, "no_renderer"),
        (FatalCategory::WindowFailed, "window_failed"),
        (FatalCategory::SettingsUnwritable, "settings_unwritable"),
        (FatalCategory::Panic, "panic"),
    ];
    for (cat, name) in cases {
        assert_eq!(cat.as_str(), name);
        assert_eq!(
            record().fatal(cat, &[("at", "main.rs:10")]),
            format!("launch=00abcdef fatal category={name} at=main.rs:10")
        );
    }
    assert_eq!(record().exit_clean(), "launch=00abcdef exit=clean");
}

#[test]
fn exit_codes_match_contract() {
    assert_eq!(ExitCode::Clean.code(), 0);
    assert_eq!(ExitCode::FatalStartup.code(), 2);
    assert_eq!(ExitCode::Panic.code(), 3);
}

#[test]
fn write_sync_reaches_disk_before_returning() {
    let dir = fixtures::TempTestDir::new("diag_sync");
    let logger = AsyncFileLogger::new(dir.path(), 64).expect("init logger");
    logger.write_sync("INFO", &record().exit_clean());
    // Read while the logger is still alive: the line must already be on disk.
    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read log dir")
        .filter_map(|e| e.ok())
        .collect();
    let content = std::fs::read_to_string(entries[0].path()).expect("read log");
    assert!(
        content.contains("[INFO] launch=00abcdef exit=clean"),
        "{content}"
    );
    drop(logger);
}
