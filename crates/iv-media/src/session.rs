//! Load-generation cancellation, bounded `rayon` pool, placeholder timers, and prefetching.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

pub const PLACEHOLDER_TIMER_MS: u64 = 250;
pub const SPINNER_TIMER_MS: u64 = 500;

/// Coordinates the monotonic load generation (stale-result cancellation) and directional prefetch.
pub struct SessionCoordinator {
    pub load_gen: Arc<AtomicU64>,
    pub current_index: usize,
    pub pool: Option<rayon::ThreadPool>,
}

impl SessionCoordinator {
    pub fn new(num_cpus: usize) -> Self {
        let threads = (num_cpus / 2).clamp(1, 4);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .ok();
        Self {
            load_gen: Arc::new(AtomicU64::new(0)),
            current_index: 0,
            pool,
        }
    }

    pub fn begin_load(&mut self, index: usize) -> u64 {
        self.current_index = index;
        self.load_gen.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn is_load_current(&self, token: u64) -> bool {
        self.load_gen.load(Ordering::SeqCst) == token
    }

    /// Computes `+1, +2, -1` directional prefetch indices (expanding forward up to `+4` on burst).
    pub fn prefetch_targets(&self, current: usize, total: usize, is_burst: bool) -> Vec<usize> {
        if total <= 1 {
            return Vec::new();
        }
        let mut out = Vec::new();
        let mut push_unique = |idx: usize| {
            if idx != current && idx < total && !out.contains(&idx) {
                out.push(idx);
            }
        };
        if current + 1 < total {
            push_unique(current + 1);
        }
        if current + 2 < total {
            push_unique(current + 2);
        }
        if current > 0 {
            push_unique(current - 1);
        }
        if is_burst {
            if current + 3 < total {
                push_unique(current + 3);
            }
            if current + 4 < total {
                push_unique(current + 4);
            }
        }
        out
    }

    pub fn should_show_placeholder(elapsed: Duration, is_original_mode: bool) -> bool {
        !is_original_mode && elapsed >= Duration::from_millis(PLACEHOLDER_TIMER_MS)
    }

    pub fn should_show_spinner(elapsed: Duration) -> bool {
        elapsed >= Duration::from_millis(SPINNER_TIMER_MS)
    }
}
