mod fixtures;

use iv_core::cli::{CliCommand, CliOptions};
use iv_core::i18n::{I18n, Language};

#[test]
fn empty_args_launches_empty_viewer() {
    let opts = CliOptions::parse(&["ImageViewer".to_string()], Language::En);
    assert!(opts.error.is_none());
    assert!(!opts.is_path_not_found_error);
    assert_eq!(opts.command, CliCommand::OpenEmpty);
}

#[test]
fn ignores_benign_macos_psn_session_flag() {
    let opts = CliOptions::parse(
        &["ImageViewer".to_string(), "-psn_0_123456".to_string()],
        Language::En,
    );
    assert!(opts.error.is_none());
    assert_eq!(opts.command, CliCommand::OpenEmpty);
}

#[test]
fn opens_existing_single_path_with_psn_flag() {
    let dir = fixtures::TempTestDir::new("cli_existing");
    let file_path = dir.path().join("photo.png");
    std::fs::write(&file_path, b"stub").expect("write stub");

    let opts = CliOptions::parse(
        &[
            "ImageViewer".to_string(),
            "-psn_0_999".to_string(),
            file_path.to_string_lossy().to_string(),
        ],
        Language::En,
    );
    assert!(opts.error.is_none());
    assert!(!opts.is_path_not_found_error);
    match opts.command {
        CliCommand::OpenPath(p) => assert_eq!(p, std::fs::canonicalize(&file_path).expect("canon")),
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn missing_single_path_sets_is_path_not_found_error() {
    let opts = CliOptions::parse(
        &[
            "ImageViewer".to_string(),
            "/nonexistent/path/for_iv_test_999.jpg".to_string(),
        ],
        Language::En,
    );
    assert!(opts.error.is_some());
    assert!(opts.is_path_not_found_error);
}

#[test]
fn rejects_multiple_positional_paths() {
    let opts = CliOptions::parse(
        &[
            "ImageViewer".to_string(),
            "a.jpg".to_string(),
            "b.jpg".to_string(),
        ],
        Language::En,
    );
    assert!(opts.error.is_some());
    assert!(!opts.is_path_not_found_error);
}

#[test]
fn rejects_unknown_flags() {
    let opts = CliOptions::parse(
        &["ImageViewer".to_string(), "--unknown-flag".to_string()],
        Language::En,
    );
    assert!(opts.error.is_some());
    assert!(!opts.is_path_not_found_error);
}

#[test]
fn shell_registration_switches_are_unavailable_on_every_os() {
    // specs/004 FR-024: shell and file-association registration is Unavailable in this release.
    let expected = I18n::tr(Language::En, "error.unsupported_switch");
    for switch in ["--register-shell", "--unregister-shell"] {
        let parsed = CliOptions::parse(
            &["ImageViewer".to_string(), switch.to_string()],
            Language::En,
        );
        assert_eq!(parsed.command, CliCommand::OpenEmpty, "{switch}");
        assert_eq!(parsed.error.as_deref(), Some(expected), "{switch}");
        assert!(!parsed.is_path_not_found_error);
    }
    let ko = CliOptions::parse(
        &["ImageViewer".to_string(), "--register-shell".to_string()],
        Language::Ko,
    );
    assert_eq!(
        ko.error.as_deref(),
        Some(I18n::tr(Language::Ko, "error.unsupported_switch"))
    );
}
