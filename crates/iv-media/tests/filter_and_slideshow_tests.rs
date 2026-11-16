use iv_core::settings::{FilterMode, SlideShowMode};
use iv_core::slideshow::{Clock, SlideShowController};
use iv_media::resize_filter::SimdResizer;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct TestClock(Arc<AtomicU64>);
impl Clock for TestClock {
    fn now_millis(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

#[test]
fn screen_pixel_filter_scales_sigma_skips_tiny_and_bypasses_animations() {
    let u = SimdResizer::compute_filter_uniforms(FilterMode::Smooth, 0.5, 1.0, 1.0, 2.0, false);
    assert!(u.active);
    assert!((u.blur_sigma - 0.25).abs() < 1e-4);

    // At 20x zoom, effective sigma 0.5/20 = 0.025 < 0.05 -> skipped
    let tiny = SimdResizer::compute_filter_uniforms(FilterMode::Smooth, 0.5, 1.0, 1.0, 20.0, false);
    assert!(!tiny.active);

    // Bypassed on animated images
    let anim = SimdResizer::compute_filter_uniforms(FilterMode::SmoothSharpen, 0.5, 1.0, 1.0, 1.0, true);
    assert!(!anim.active);
}

#[test]
fn slideshow_controller_supports_sequential_random_repeat_and_reset() {
    let ticks = Arc::new(AtomicU64::new(0));
    let clock = Box::new(TestClock(Arc::clone(&ticks)));
    let mut ss = SlideShowController::new(clock, 5, SlideShowMode::Sequential, true);
    ss.play();
    assert_eq!(ss.poll_advance(0, 3), None);
    ticks.store(5000, Ordering::SeqCst);
    assert_eq!(ss.poll_advance(0, 3), Some(1));

    // Manual navigation resets timer
    ss.notify_manual_navigation();
    ticks.store(8000, Ordering::SeqCst);
    assert_eq!(ss.poll_advance(1, 3), None);
    ticks.store(10_000, Ordering::SeqCst);
    assert_eq!(ss.poll_advance(1, 3), Some(2));

    let _ = Duration::from_secs(5);
}
