mod fixtures;

use iv_core::i18n::{I18n, Language};
use iv_core::logger::AsyncFileLogger;
use std::time::Duration;

#[test]
fn ko_and_en_catalogs_have_parity_for_core_keys() {
    for key in [
        "app.title",
        "error.path_not_found",
        "error.unsupported_os_switch",
        "error.encrypted_archive",
        "error.missing_os_codec",
        "dialog.settings",
        "dialog.about",
        "dialog.image_list",
    ] {
        assert!(!I18n::tr(Language::En, key).is_empty(), "missing en {key}");
        assert!(!I18n::tr(Language::Ko, key).is_empty(), "missing ko {key}");
        assert_ne!(I18n::tr(Language::En, key), key, "untranslated en {key}");
        assert_ne!(I18n::tr(Language::Ko, key), key, "untranslated ko {key}");
    }
    assert!(!I18n::tr(Language::En, "error.unsupported_os_switch").contains("Windows"));
    assert!(!I18n::tr(Language::Ko, "error.unsupported_os_switch").contains("Windows"));
}

#[test]
fn async_logger_redacts_paths_rotates_and_flushes_on_drop() {
    let dir = fixtures::TempTestDir::new("logger");
    {
        let logger = AsyncFileLogger::new(dir.path(), 64).expect("init logger");
        logger.log_info(r#"Opened /Users/bob/Private/vacation.jpg\r\nINJECT"#);
        logger.log_slow_render("vacation.jpg", Duration::from_millis(342));
        logger.flush_bounded(Duration::from_secs(2));
    }

    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read log dir")
        .filter_map(|e| e.ok())
        .collect();
    assert!(!entries.is_empty());
    let content = std::fs::read_to_string(entries[0].path()).expect("read log file");
    assert!(!content.contains("/Users/bob"));
    assert!(content.contains("vacation.jpg"));
    assert!(!content.contains("\r\nINJECT"));
    assert!(content.contains("342ms"));
}

#[test]
fn startup_failure_and_status_keys_exist_in_both_languages_and_differ() {
    for key in [
        "fatal.no_renderer",
        "fatal.window_failed",
        "fatal.panic",
        "fatal.log_location",
        "settings.reset_notice",
        "settings.unreadable_notice",
        "settings.save_failed",
        "error.unsupported_format",
    ] {
        let en = I18n::tr(Language::En, key);
        let ko = I18n::tr(Language::Ko, key);
        assert!(!en.is_empty(), "missing en {key}");
        assert!(!ko.is_empty(), "missing ko {key}");
        assert_ne!(en, ko, "untranslated {key}");
    }
}

// ---------------------------------------------------------------------------
// specs/004 T098: log timestamps without a date-time dependency (UTC, RFC 3339)
// ---------------------------------------------------------------------------

#[test]
fn utc_timestamps_and_log_dates_are_formatted_from_unix_time() {
    use iv_core::logger::{utc_date, utc_timestamp};
    use std::time::{Duration, UNIX_EPOCH};
    let at = |secs: u64, millis: u64| UNIX_EPOCH + Duration::from_millis(secs * 1000 + millis);
    assert_eq!(utc_timestamp(at(0, 0)), "1970-01-01T00:00:00.000Z");
    assert_eq!(
        utc_timestamp(at(1_794_918_896, 789)),
        "2026-11-17T12:34:56.789Z"
    );
    assert_eq!(
        utc_timestamp(at(1_709_251_199, 999)),
        "2024-02-29T23:59:59.999Z"
    );
    assert_eq!(
        utc_timestamp(at(946_684_800, 0)),
        "2000-01-01T00:00:00.000Z"
    );
    assert_eq!(
        utc_timestamp(at(4_107_542_400, 1)),
        "2100-03-01T00:00:00.001Z"
    );
    assert_eq!(utc_date(at(1_794_918_896, 0)), "2026-11-17");
    // Times before the epoch clamp to the epoch rather than panicking.
    assert_eq!(utc_date(UNIX_EPOCH - Duration::from_secs(10)), "1970-01-01");
}
