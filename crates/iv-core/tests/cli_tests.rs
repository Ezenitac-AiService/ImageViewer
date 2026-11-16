mod fixtures;

use iv_core::cli::{CliCommand, CliOptions};
use iv_core::i18n::Language;

#[test]
fn empty_args_launches_empty_viewer() {
    let opts = CliOptions::parse(&["ImageViewer".to_string()], Language::En, false);
    assert!(opts.error.is_none());
    assert!(!opts.is_path_not_found_error);
    assert_eq!(opts.command, CliCommand::OpenEmpty);
}

#[test]
fn ignores_benign_macos_psn_session_flag() {
    let opts = CliOptions::parse(
        &[
            "ImageViewer".to_string(),
            "-psn_0_123456".to_string(),
        ],
        Language::En,
        false,
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
        false,
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
        false,
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
        false,
    );
    assert!(opts.error.is_some());
    assert!(!opts.is_path_not_found_error);
}

#[test]
fn rejects_unknown_flags() {
    let opts = CliOptions::parse(
        &["ImageViewer".to_string(), "--unknown-flag".to_string()],
        Language::En,
        false,
    );
    assert!(opts.error.is_some());
    assert!(!opts.is_path_not_found_error);
}

#[test]
fn repairs_explorer_trailing_quote_on_windows_drive() {
    let repaired = CliOptions::repair_explorer_quoted_drive(r#"C:\""#);
    assert_eq!(repaired, r#"C:\"#);
}

#[test]
fn shell_register_switches_work_on_windows_and_fail_closed_on_non_windows() {
    let win_reg = CliOptions::parse(
        &["ImageViewer".to_string(), "--register-shell".to_string()],
        Language::En,
        true,
    );
    assert!(win_reg.error.is_none());
    assert_eq!(win_reg.command, CliCommand::RegisterShell);

    let win_unreg = CliOptions::parse(
        &["ImageViewer".to_string(), "/unregister-shell".to_string()],
        Language::En,
        true,
    );
    assert!(win_unreg.error.is_none());
    assert_eq!(win_unreg.command, CliCommand::UnregisterShell);

    let non_win = CliOptions::parse(
        &["ImageViewer".to_string(), "--register-shell".to_string()],
        Language::En,
        false,
    );
    assert!(non_win.error.is_some());
    assert!(!non_win.is_path_not_found_error);

    let combined = CliOptions::parse(
        &[
            "ImageViewer".to_string(),
            "--register-shell".to_string(),
            "--unregister-shell".to_string(),
        ],
        Language::En,
        true,
    );
    assert!(combined.error.is_some());
}
