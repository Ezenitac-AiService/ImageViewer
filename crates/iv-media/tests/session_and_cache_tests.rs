#![allow(dead_code)]
mod fixtures;

use iv_media::cache::{DecodedFrameCache, DecodedFrame};
use iv_media::session::SessionCoordinator;
use std::sync::Arc;

#[test]
fn frame_lru_cache_bounds_at_eight_and_session_cancels_stale_generations() {
    let mut cache = DecodedFrameCache::new(8);
    for i in 0..12 {
        let frame = Arc::new(DecodedFrame::empty_for_test(10 + i, 10));
        cache.insert(format!("k{i}"), frame);
    }
    assert_eq!(cache.len(), 8);
    assert!(cache.get("k0").is_none());
    assert!(cache.get("k11").is_some());

    let mut coord = SessionCoordinator::new(2);
    let g1 = coord.begin_load(0);
    let g2 = coord.begin_load(1);
    assert!(!coord.is_load_current(g1));
    assert!(coord.is_load_current(g2));

    let targets = coord.prefetch_targets(5, 20, true);
    assert_eq!(targets, vec![6, 7, 4, 8, 9]);
}
