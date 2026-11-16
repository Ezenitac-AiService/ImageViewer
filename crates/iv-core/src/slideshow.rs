//! `Clock`-abstracted `SlideShowController` (`5–90 s`, Sequential / non-repeating Random, Repeat).

use crate::settings::SlideShowMode;

pub trait Clock: Send + Sync {
    fn now_millis(&self) -> u64;
}

pub struct SystemClock;
impl Clock for SystemClock {
    fn now_millis(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

pub struct SlideShowController {
    clock: Box<dyn Clock>,
    pub interval_seconds: u32,
    pub mode: SlideShowMode,
    pub repeat: bool,
    pub playing: bool,
    last_tick_ms: u64,
    visited: Vec<usize>,
}

impl SlideShowController {
    pub fn new(clock: Box<dyn Clock>, interval_seconds: u32, mode: SlideShowMode, repeat: bool) -> Self {
        let now = clock.now_millis();
        Self {
            clock,
            interval_seconds: interval_seconds.clamp(5, 90),
            mode,
            repeat,
            playing: false,
            last_tick_ms: now,
            visited: Vec::new(),
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.last_tick_ms = self.clock.now_millis();
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn notify_manual_navigation(&mut self) {
        self.last_tick_ms = self.clock.now_millis();
    }

    pub fn poll_advance(&mut self, current: usize, total: usize) -> Option<usize> {
        if !self.playing || total <= 1 {
            return None;
        }
        let now = self.clock.now_millis();
        let interval_ms = u64::from(self.interval_seconds) * 1000;
        if now.saturating_sub(self.last_tick_ms) < interval_ms {
            return None;
        }
        self.last_tick_ms = now;

        match self.mode {
            SlideShowMode::Sequential => {
                if current + 1 < total {
                    Some(current + 1)
                } else if self.repeat {
                    Some(0)
                } else {
                    self.playing = false;
                    None
                }
            }
            SlideShowMode::Random => {
                if !self.visited.contains(&current) {
                    self.visited.push(current);
                }
                if self.visited.len() >= total {
                    if !self.repeat {
                        self.playing = false;
                        return None;
                    }
                    self.visited.clear();
                    self.visited.push(current);
                }
                for candidate in 0..total {
                    if !self.visited.contains(&candidate) {
                        self.visited.push(candidate);
                        return Some(candidate);
                    }
                }
                None
            }
        }
    }
}
