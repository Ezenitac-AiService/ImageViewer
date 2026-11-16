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
    assert!(!final_disk.navigation.wrap_around, "preference from A must survive");
    assert_eq!(final_disk.window.width, 1600.0, "geometry from B must survive");
    assert_eq!(final_disk.window.height, 900.0);
}
