#![allow(dead_code)]
//! Deterministic real-fixture helpers for `iv-core` tests.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Temporary directory that cleans itself up on `Drop`.
pub struct TempTestDir {
    pub path: PathBuf,
}

impl TempTestDir {
    pub fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("iv_test_{prefix}_{pid}_{nanos}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        Self { path: dir }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Controllable deterministic clock for slideshow & timer tests.
#[derive(Clone, Debug)]
pub struct ManualClock {
    now_ms: Arc<Mutex<u64>>,
}

impl ManualClock {
    pub fn new(initial_ms: u64) -> Self {
        Self {
            now_ms: Arc::new(Mutex::new(initial_ms)),
        }
    }

    pub fn advance(&self, delta: Duration) {
        let mut guard = self.now_ms.lock().expect("clock lock");
        *guard = guard.saturating_add(delta.as_millis() as u64);
    }

    pub fn now_millis(&self) -> u64 {
        *self.now_ms.lock().expect("clock lock")
    }
}

/// Generates BOM + whitespace + script polyglot payloads for `FormatGuard` tests.
pub fn utf32_le_bom_payload(body: &str) -> Vec<u8> {
    let mut out = vec![0xFF, 0xFE, 0x00, 0x00];
    for ch in body.chars() {
        out.extend_from_slice(&(ch as u32).to_le_bytes());
    }
    out
}

pub fn utf32_be_bom_payload(body: &str) -> Vec<u8> {
    let mut out = vec![0x00, 0x00, 0xFE, 0xFF];
    for ch in body.chars() {
        out.extend_from_slice(&(ch as u32).to_be_bytes());
    }
    out
}

pub fn utf16_le_bom_payload(body: &str) -> Vec<u8> {
    let mut out = vec![0xFF, 0xFE];
    for unit in body.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

pub fn utf16_be_bom_payload(body: &str) -> Vec<u8> {
    let mut out = vec![0xFE, 0xFF];
    for unit in body.encode_utf16() {
        out.extend_from_slice(&unit.to_be_bytes());
    }
    out
}

pub fn utf16_bomless_le_payload(body: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() * 2);
    for unit in body.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

pub fn whitespace_padded_payload(spaces: usize, body: &str) -> Vec<u8> {
    let mut out = vec![b' '; spaces];
    out.extend_from_slice(body.as_bytes());
    out
}
