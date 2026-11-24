#![allow(dead_code)]
mod fixtures;

#[cfg(feature = "shell-registration")]
#[test]
fn validates_exe_paths_and_generates_xdg_entries() {
    use iv_platform::integration::OsIntegration;
    assert!(OsIntegration::validate_exe_path("/usr/local/bin/imageviewer").is_ok());
    assert!(OsIntegration::validate_exe_path("/usr/local/bin/bad\"quote").is_err());
    assert!(OsIntegration::validate_exe_path("/usr/local/bin/bad\nctrl").is_err());

    let desktop = OsIntegration::render_xdg_desktop_entry("/usr/local/bin/imageviewer");
    assert!(desktop.contains("Exec=/usr/local/bin/imageviewer %F"));
    assert!(desktop.contains("MimeType="));
    assert!(desktop.contains("image/jpeg"));

    assert!(OsIntegration::commands_match_normalized(
        "  /usr/local/bin/imageviewer  ",
        "/usr/local/bin/imageviewer"
    ));
}

// ---------------------------------------------------------------------------
// specs/004 T039: the fatal dialog names the log folder without the user's real path
// ---------------------------------------------------------------------------

#[test]
fn redacted_log_folder_is_fixed_per_os_and_never_contains_a_user_name() {
    use iv_platform::fatal_dialog::redacted_log_folder;
    assert_eq!(
        redacted_log_folder("macos"),
        "~/Library/Application Support/com.Ezenitac-AiService.ImageViewer/logs"
    );
    assert_eq!(redacted_log_folder("linux"), "~/.config/imageviewer/logs");
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_default();
    for os in ["macos", "linux"] {
        if !user.is_empty() {
            assert!(!redacted_log_folder(os).contains(&user));
        }
    }
}

// ---------------------------------------------------------------------------
// specs/004 T054: OS locale tags → UI language
// ---------------------------------------------------------------------------

#[test]
fn language_from_tags_picks_korean_only_for_ko_tags() {
    use iv_core::i18n::Language;
    use iv_platform::locale::language_from_tags;
    assert_eq!(language_from_tags(&["ko-KR", "en-US"]), Language::Ko);
    assert_eq!(language_from_tags(&["ko"]), Language::Ko);
    assert_eq!(language_from_tags(&["en-US", "ko-KR"]), Language::En);
    assert_eq!(language_from_tags(&["fr-FR"]), Language::En);
    assert_eq!(language_from_tags(&[]), Language::En);
}
