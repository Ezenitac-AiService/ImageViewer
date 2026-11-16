//! Precision trackpad smooth-scroll accumulator and native pinch math.

/// Accumulates pixel deltas so precision trackpads don't overshoot multiple images per flick.
#[derive(Debug, Clone)]
pub struct TrackpadAccumulator {
    acc_y: f64,
    last_step_ms: u64,
    pub step_threshold_px: f64,
    pub cooldown_ms: u64,
}

impl Default for TrackpadAccumulator {
    fn default() -> Self {
        Self {
            acc_y: 0.0,
            last_step_ms: 0,
            step_threshold_px: 48.0,
            cooldown_ms: 120,
        }
    }
}

impl TrackpadAccumulator {
    /// Accumulates vertical pixel delta and returns `-1` (prev), `0` (none), or `+1` (next).
    pub fn accumulate_scroll_step(&mut self, delta_y: f64, now_ms: u64) -> i32 {
        if !delta_y.is_finite() {
            return 0;
        }
        self.acc_y += delta_y;
        if self.acc_y.abs() >= self.step_threshold_px {
            let elapsed = now_ms.saturating_sub(self.last_step_ms);
            let dir = if self.acc_y > 0.0 { 1 } else { -1 };
            self.acc_y = 0.0;
            if self.last_step_ms == 0 || elapsed >= self.cooldown_ms {
                self.last_step_ms = now_ms;
                return dir;
            }
        }
        0
    }

    pub fn reset(&mut self) {
        self.acc_y = 0.0;
    }

    /// Converts a native `PinchGesture` delta into a multiplicative zoom factor.
    pub fn pinch_delta_to_zoom_factor(delta: f64) -> f64 {
        if !delta.is_finite() {
            return 1.0;
        }
        (1.0 + delta).clamp(0.5, 2.0)
    }
}
