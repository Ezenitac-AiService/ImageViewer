mod fixtures;

use iv_core::settings::{
    DisplayMode, MouseWheelAction, SettingsStore, SlideShowMode, ViewerBackground, ViewerSettings,
};

#[test]
fn parses_v106_camelcase_json_with_comments_and_trailing_commas_and_migrates() {
    let json = r#"{
        // v1.0.6 legacy config
        "version": 1,
        "display": {
            "mode": "Fill",
            "background": "Black",
        },
        "filter": {
            "smoothSigma": 0.8,
        },
        "unknownFutureField": 42,
    }"#;
    let settings = ViewerSettings::from_json_str(json);
    assert_eq!(settings.display.mode, DisplayMode::Fill);
    assert_eq!(settings.display.background, ViewerBackground::Black);
    // Version < 2 -> MouseWheel defaults to Navigate
    assert_eq!(settings.display.mouse_wheel, MouseWheelAction::Navigate);
    // Version < 3 with smoothSigma ~= 0.8 -> 0.5
    assert!((settings.filter.smooth_sigma - 0.5).abs() < 1e-6);
    assert!(settings.chrome.top_bar_pinned);
    assert!(settings.chrome.bottom_bar_pinned);
    assert!(settings.navigation.wrap_around);
    assert!(settings.navigation.watch_folder);
    assert!(settings.thumbnail.visible);
    assert!(settings.update.auto_check);
    assert_eq!(settings.slideshow.mode, SlideShowMode::Sequential);
    assert_eq!(settings.slideshow.interval_seconds, 5);
    assert!(settings.slideshow.repeat);
    assert_eq!(settings.limits.max_image_megapixels, 150);
    assert_eq!(settings.limits.max_zip_entry_megabytes, 512);
}

#[test]
fn recovers_from_non_finite_out_of_range_and_oversized_settings() {
    let dir = fixtures::TempTestDir::new("settings_bounds");
    let path = dir.path().join("settings.json");

    // > 64 KB rejected -> defaults
    let huge = vec![b' '; 70_000];
    std::fs::write(&path, &huge).expect("write huge");
    let loaded = SettingsStore::load(&path);
    assert_eq!(loaded.window.width, 1280.0);
    assert_eq!(loaded.window.height, 800.0);

    // NaN / Infinity / invalid skippedVersion normalized cleanly
    let mut s = ViewerSettings::default();
    s.window.width = f64::NAN;
    s.window.height = 99_999.0;
    s.window.left = Some(f64::INFINITY);
    s.filter.smooth_sigma = 0.01;
    s.limits.max_image_megapixels = 9999;
    s.update.skipped_version = Some("1.0.0\nrm -rf".to_string());
    s.normalize();
    assert_eq!(s.window.width, 1280.0);
    assert_eq!(s.window.height, 20_000.0);
    assert_eq!(s.window.left, None);
    assert!((s.filter.smooth_sigma - 0.1).abs() < 1e-6);
    assert_eq!(s.limits.max_image_megapixels, 1000);
    assert_eq!(s.update.skipped_version, None);
}

#[test]
fn selective_merge_save_prevents_multi_instance_lost_updates() {
    let dir = fixtures::TempTestDir::new("settings_merge");
    let path = dir.path().join("settings.json");

    let mut initial = ViewerSettings::default();
    initial.display.background = ViewerBackground::Checkerboard;
    SettingsStore::save_preferences(&path, &initial).expect("save initial");

    // Instance A changes preference on disk
    let mut inst_a = SettingsStore::load(&path);
    inst_a.navigation.wrap_around = false;
    SettingsStore::save_preferences(&path, &inst_a).expect("save A");

    // Instance B had stale preferences in memory, closes and saves only its window geometry
    let mut stale_b = initial.clone();
    stale_b.window.width = 1600.0;
    stale_b.window.height = 900.0;
    SettingsStore::save_window_geometry(&path, &stale_b.window).expect("save window B");

    let final_disk = SettingsStore::load(&path);
    assert!(
        !final_disk.navigation.wrap_around,
        "preference from A must survive"
    );
    assert_eq!(
        final_disk.window.width, 1600.0,
        "geometry from B must survive"
    );
    assert_eq!(final_disk.window.height, 900.0);
}

// ---------------------------------------------------------------------------
// specs/004 T035: settings load outcome and failure modes
// ---------------------------------------------------------------------------

mod load_outcome {
    use super::fixtures::TempTestDir;
    use iv_core::i18n::Language;
    use iv_core::settings::{LoadOutcome, SettingsStore, ViewerSettings};

    #[test]
    fn absent_file_creates_defaults() {
        let dir = TempTestDir::new("outcome_absent");
        let (s, outcome) = SettingsStore::load_with_outcome(&dir.path().join("settings.json"));
        assert_eq!(outcome, LoadOutcome::CreatedDefaults);
        assert_eq!(s, ViewerSettings::default());
    }

    #[test]
    fn valid_file_is_loaded() {
        let dir = TempTestDir::new("outcome_valid");
        let path = dir.path().join("settings.json");
        let prefs = ViewerSettings {
            language: Language::Ko,
            ..ViewerSettings::default()
        };
        SettingsStore::save_preferences(&path, &prefs).expect("save");
        let (s, outcome) = SettingsStore::load_with_outcome(&path);
        assert_eq!(outcome, LoadOutcome::Loaded);
        assert_eq!(s.language, Language::Ko);
    }

    #[test]
    fn oversized_file_is_recovered_from_damage() {
        let dir = TempTestDir::new("outcome_big");
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "x".repeat(70 * 1024)).expect("write");
        let (s, outcome) = SettingsStore::load_with_outcome(&path);
        assert_eq!(outcome, LoadOutcome::RecoveredFromDamage);
        assert_eq!(s, ViewerSettings::default());
    }

    #[test]
    fn malformed_json_is_recovered_from_damage() {
        let dir = TempTestDir::new("outcome_malformed");
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{ \"display\": [1, 2,").expect("write");
        let (_, outcome) = SettingsStore::load_with_outcome(&path);
        assert_eq!(outcome, LoadOutcome::RecoveredFromDamage);
    }

    #[test]
    fn out_of_range_values_are_normalized_and_reported() {
        let dir = TempTestDir::new("outcome_range");
        let path = dir.path().join("settings.json");
        std::fs::write(
            &path,
            r#"{"version": 3, "limits": {"maxImageMegapixels": 999999}}"#,
        )
        .expect("write");
        let (s, outcome) = SettingsStore::load_with_outcome(&path);
        assert_eq!(outcome, LoadOutcome::RecoveredFromDamage);
        assert_eq!(s.limits.max_image_megapixels, 1000);
    }

    #[cfg(unix)]
    #[test]
    fn read_only_directory_is_unreadable() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempTestDir::new("outcome_readonly");
        let locked = dir.path().join("cfg");
        std::fs::create_dir_all(&locked).expect("mkdir");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).expect("chmod");
        let path = locked.join("settings.json");
        let (s, outcome) = SettingsStore::load_with_outcome(&path);
        let save = SettingsStore::save_preferences(&path, &ViewerSettings::default());
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755))
            .expect("chmod back");
        assert_eq!(outcome, LoadOutcome::Unreadable);
        assert_eq!(s, ViewerSettings::default());
        assert!(save.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_settings_file_is_unreadable() {
        let dir = TempTestDir::new("outcome_symlink");
        let target = dir.path().join("elsewhere.json");
        std::fs::write(&target, "{}").expect("write");
        let link = dir.path().join("settings.json");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        let (_, outcome) = SettingsStore::load_with_outcome(&link);
        assert_eq!(outcome, LoadOutcome::Unreadable);
    }

    #[test]
    fn concurrent_saves_from_two_instances_never_corrupt_the_file() {
        let dir = TempTestDir::new("outcome_concurrent");
        let path = dir.path().join("settings.json");
        for _ in 0..200 {
            let ko = ViewerSettings {
                language: Language::Ko,
                ..ViewerSettings::default()
            };
            let en = ViewerSettings::default();
            let (p1, p2) = (path.clone(), path.clone());
            let t1 = std::thread::spawn(move || SettingsStore::save_preferences(&p1, &ko));
            let t2 = std::thread::spawn(move || SettingsStore::save_preferences(&p2, &en));
            assert!(t1.join().expect("join").is_ok());
            assert!(t2.join().expect("join").is_ok());
            let raw = std::fs::read_to_string(&path).expect("read");
            let parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
            let lang = parsed["language"].as_str().expect("language");
            assert!(lang == "ko" || lang == "en", "{lang}");
        }
    }
}
