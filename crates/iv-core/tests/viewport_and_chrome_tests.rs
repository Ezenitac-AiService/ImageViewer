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

// ---------------------------------------------------------------------------
// specs/004 T014: image placement on screen (ImageQuad)
// ---------------------------------------------------------------------------

use iv_core::layout::{TileRect, tile_grid};

fn approx(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6
}

fn vp(w: f64, h: f64, dpi: f64, mode: DisplayMode) -> ViewportState {
    let mut v = ViewportState::new(w, h, dpi);
    v.display_mode = mode;
    v
}

#[test]
fn image_quad_fill_centers_and_letterboxes_in_logical_points() {
    // 400x200 image in an 800x800 logical viewport at 2x DPI: Fill scales by width.
    let v = vp(800.0, 800.0, 2.0, DisplayMode::Fill);
    let q = ViewportLayoutCalculator::image_quad(400, 200, &v);
    assert!(approx((q.width, q.height), (800.0, 400.0)));
    assert!(approx((q.min_x, q.min_y), (0.0, 200.0)));
}

#[test]
fn image_quad_sizes_for_each_display_mode() {
    let v = vp(1000.0, 500.0, 1.0, DisplayMode::FitWidth);
    let q = ViewportLayoutCalculator::image_quad(200, 400, &v);
    assert!(approx((q.width, q.height), (1000.0, 2000.0)));

    let v = vp(1000.0, 500.0, 1.0, DisplayMode::FitHeight);
    let q = ViewportLayoutCalculator::image_quad(200, 400, &v);
    assert!(approx((q.width, q.height), (250.0, 500.0)));

    // Original: one image pixel per physical pixel, so 2x DPI halves the logical size.
    let v = vp(1000.0, 500.0, 2.0, DisplayMode::Original);
    let q = ViewportLayoutCalculator::image_quad(200, 400, &v);
    assert!(approx((q.width, q.height), (100.0, 200.0)));
}

#[test]
fn image_quad_maps_image_corners_for_each_rotation() {
    let mut v = vp(100.0, 100.0, 1.0, DisplayMode::Original);
    // Square image so the rect is identical for every rotation: (0,0)..(100,100).
    let tl = (0.0, 0.0);
    let tr = (100.0, 0.0);
    let br = (100.0, 100.0);
    let bl = (0.0, 100.0);
    // Expected screen position of the image's top-left corner after k clockwise turns.
    let expected_image_tl = [tl, tr, br, bl];
    for (turns, expected) in expected_image_tl.iter().enumerate() {
        v.rotation_quarter_turns = turns as u8;
        let q = ViewportLayoutCalculator::image_quad(100, 100, &v);
        assert!(
            approx(q.map(0.0, 0.0), *expected),
            "turns={turns} got {:?}",
            q.map(0.0, 0.0)
        );
    }
}

#[test]
fn image_quad_rotation_swaps_rect_dimensions() {
    let mut v = vp(1000.0, 1000.0, 1.0, DisplayMode::Original);
    v.rotation_quarter_turns = 1;
    let q = ViewportLayoutCalculator::image_quad(300, 100, &v);
    assert!(approx((q.width, q.height), (100.0, 300.0)));
    // Image top-right corner (1,0) lands at the screen bottom-right after a clockwise turn.
    assert!(approx(q.map(1.0, 0.0), (q.min_x + 100.0, q.min_y + 300.0)));
}

#[test]
fn image_quad_flips_mirror_the_displayed_result() {
    let mut v = vp(100.0, 100.0, 1.0, DisplayMode::Original);
    v.flip_h = true;
    let q = ViewportLayoutCalculator::image_quad(100, 100, &v);
    assert!(approx(q.map(0.0, 0.0), (100.0, 0.0)));
    v.flip_h = false;
    v.flip_v = true;
    let q = ViewportLayoutCalculator::image_quad(100, 100, &v);
    assert!(approx(q.map(0.0, 0.0), (0.0, 100.0)));
    // Flip applies on screen after rotation: rotate CW then mirror horizontally.
    v.flip_v = false;
    v.flip_h = true;
    v.rotation_quarter_turns = 1;
    let q = ViewportLayoutCalculator::image_quad(100, 100, &v);
    assert!(approx(q.map(0.0, 0.0), (0.0, 0.0)));
}

#[test]
fn image_quad_applies_pan_offsets() {
    let mut v = vp(100.0, 100.0, 1.0, DisplayMode::Original);
    v.pan_x = 15.0;
    v.pan_y = -5.0;
    let q = ViewportLayoutCalculator::image_quad(20, 20, &v);
    assert!(approx((q.min_x, q.min_y), (55.0, 35.0)));
}

#[test]
fn zoom_at_cursor_keeps_the_image_point_under_the_cursor_fixed() {
    let mut v = vp(800.0, 600.0, 1.0, DisplayMode::Fill);
    let cursor = (600.0, 150.0);
    let before = ViewportLayoutCalculator::image_quad(1600, 1200, &v);
    let u = (
        (cursor.0 - before.min_x) / before.width,
        (cursor.1 - before.min_y) / before.height,
    );
    v.zoom_at_cursor(1.75, cursor.0, cursor.1);
    let after = ViewportLayoutCalculator::image_quad(1600, 1200, &v);
    assert!(approx(after.map(u.0, u.1), cursor));
}

// ---------------------------------------------------------------------------
// specs/004 T016: texture tiling
// ---------------------------------------------------------------------------

#[test]
fn tile_grid_single_tile_when_within_limit() {
    assert_eq!(
        tile_grid(4000, 3000, 8192),
        vec![TileRect {
            x: 0,
            y: 0,
            w: 4000,
            h: 3000
        }]
    );
}

#[test]
fn tile_grid_covers_large_images_without_overlap() {
    let tiles = tile_grid(20_000, 9_000, 8192);
    assert_eq!(tiles.len(), 3 * 2);
    let mut covered: u64 = 0;
    for t in &tiles {
        assert!(t.w <= 8192 && t.h <= 8192 && t.w > 0 && t.h > 0);
        covered += u64::from(t.w) * u64::from(t.h);
        for o in &tiles {
            if o == t {
                continue;
            }
            let overlap_x = t.x < o.x + o.w && o.x < t.x + t.w;
            let overlap_y = t.y < o.y + o.h && o.y < t.y + t.h;
            assert!(!(overlap_x && overlap_y), "{t:?} overlaps {o:?}");
        }
    }
    assert_eq!(covered, 20_000 * 9_000);
}

#[test]
fn tile_grid_empty_for_zero_sizes() {
    assert!(tile_grid(0, 100, 8192).is_empty());
    assert!(tile_grid(100, 0, 8192).is_empty());
    assert!(tile_grid(100, 100, 0).is_empty());
}
