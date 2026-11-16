use iv_core::chrome_state::ChromeState;
use iv_core::layout::{ViewportLayoutCalculator, ViewportState};
use iv_core::natural_sort::NaturalSort;
use iv_core::settings::DisplayMode;
use iv_core::trackpad::TrackpadAccumulator;
use std::time::Duration;

#[test]
fn natural_sort_orders_numeric_chunks_before_lexicographic() {
    let mut items = vec![
        "image10.png",
        "image2.png",
        "image1.png",
        "IMG_020.jpg",
        "img_3.jpg",
    ];
    items.sort_by(|a, b| NaturalSort::cmp(a, b));
    assert_eq!(
        items,
        vec![
            "image1.png",
            "image2.png",
            "image10.png",
            "img_3.jpg",
            "IMG_020.jpg"
        ]
    );
}

#[test]
fn viewport_layout_calculates_modes_dpi_and_transforms() {
    let mut vp = ViewportState::new(1280.0, 800.0, 2.0); // physical 2560x1600
    assert_eq!(vp.physical_width(), 2560);
    assert_eq!(vp.physical_height(), 1600);

    // Small image 400x200 in Fill mode enlarges cleanly to fill 2560x1600 (min(2560/400=6.4, 1600/200=8.0) = 6.4)
    let fit = ViewportLayoutCalculator::compute_fit_scale(400, 200, &vp, DisplayMode::Fill);
    assert!((fit - 6.4).abs() < 1e-4, "expected 6.4 got {fit}");

    // Rotate 90 deg swaps effective source dims
    vp.rotate_cw();
    let fit_rot = ViewportLayoutCalculator::compute_fit_scale(400, 200, &vp, DisplayMode::Fill);
    // Swapped 200x400 into 2560x1600 -> 1600/400 = 4.0
    assert!((fit_rot - 4.0).abs() < 1e-4);

    vp.zoom_at_cursor(1.5, 640.0, 400.0);
    assert!(vp.zoom_scale > 1.0);
}

#[test]
fn chrome_state_handles_global_pin_toggle_and_wayland_geometry() {
    let mut chrome = ChromeState::default();
    assert!(chrome.top_bar_pinned && chrome.bottom_bar_pinned);
    chrome.toggle_all_pins();
    assert!(!chrome.top_bar_pinned && !chrome.bottom_bar_pinned);
    chrome.top_bar_pinned = true;
    // Either pinned -> unpin both
    chrome.toggle_all_pins();
    assert!(!chrome.top_bar_pinned && !chrome.bottom_bar_pinned);
    // Both unpinned -> pin both
    chrome.toggle_all_pins();
    assert!(chrome.top_bar_pinned && chrome.bottom_bar_pinned);
}

#[test]
fn trackpad_accumulator_prevents_overshoot_and_supports_pinch() {
    let mut acc = TrackpadAccumulator::default();
    assert_eq!(acc.accumulate_scroll_step(15.0, 0), 0);
    assert_eq!(acc.accumulate_scroll_step(20.0, 10), 0);
    // Crosses >= 48.0 threshold
    assert_eq!(acc.accumulate_scroll_step(20.0, 20), 1);
    // Short cooldown suppresses immediate second step
    assert_eq!(acc.accumulate_scroll_step(60.0, 30), 0);
    // After cooldown (> 120 ms)
    assert_eq!(acc.accumulate_scroll_step(60.0, 200), 1);
    let _ = Duration::from_millis(1);
}
