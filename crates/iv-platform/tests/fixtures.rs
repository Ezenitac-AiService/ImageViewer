//! Deterministic real-fixture helpers for `iv-platform` tests (ephemeral `rcgen` HTTPS cert, temp dirs).

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TempTestDir {
    pub path: PathBuf,
}

impl TempTestDir {
    pub fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("iv_platform_test_{prefix}_{}_{nanos}", std::process::id()));
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

/// Generates an ephemeral `rcgen` loopback self-signed certificate & private key DER pair.
pub fn generate_loopback_cert() -> (Vec<u8>, Vec<u8>) {
    let certified = rcgen::generate_simple_self_signed(vec!["localhost".to_string(), "127.0.0.1".to_string()])
        .expect("generate loopback cert");
    let cert_der = certified.cert.der().to_vec();
    let key_der = certified.key_pair.serialize_der();
    (cert_der, key_der)
}
