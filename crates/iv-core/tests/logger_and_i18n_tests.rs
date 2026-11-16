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
