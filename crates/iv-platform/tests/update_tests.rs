#![allow(dead_code)]
mod fixtures;

use iv_platform::update::{
    HttpFeedTransport, UpdateClient, UpdateError, MAX_BINARY_BYTES, MAX_JSON_BYTES, MAX_SUMS_BYTES,
};
use sha2::{Digest, Sha256};

struct MemoryFeedTransport {
    json_body: Vec<u8>,
    sums_body: Vec<u8>,
    bin_body: Vec<u8>,
}

impl HttpFeedTransport for MemoryFeedTransport {
    fn get_bounded(&self, url: &str, max_bytes: usize) -> Result<Vec<u8>, UpdateError> {
        if !url.starts_with("https://") {
            return Err(UpdateError::InsecureScheme);
        }
        let body = if url.ends_with("latest") || url.ends_with(".json") {
            &self.json_body
        } else if url.ends_with("SHA256SUMS.txt") {
            &self.sums_body
        } else {
            &self.bin_body
        };
        if body.len() > max_bytes {
            return Err(UpdateError::BodyTooLarge);
        }
        Ok(body.clone())
    }
}

#[test]
fn update_client_enforces_https_stream_caps_and_sha256_verification() {
    let dir = fixtures::TempTestDir::new("updater");
    let payload = b"ImageViewer-1.2.0-installer-bytes";
    let digest = format!("{:x}", Sha256::digest(payload));

    let transport = MemoryFeedTransport {
        json_body: serde_json::json!({
            "tag_name": "v1.2.0",
            "html_url": "https://github.com/ImageViewer/ImageViewer/releases/tag/v1.2.0",
            "body": "Release notes",
            "prerelease": false,
            "draft": false,
            "assets": [{
                "name": "ImageViewer-1.2.0.dmg",
                "browser_download_url": "https://github.com/ImageViewer/ImageViewer/releases/download/v1.2.0/ImageViewer-1.2.0.dmg"
            }]
        })
        .to_string()
        .into_bytes(),
        sums_body: format!("{digest}  ImageViewer-1.2.0.dmg\n").into_bytes(),
        bin_body: payload.to_vec(),
    };

    // Rejects http:// unconditionally
    let err = UpdateClient::check_latest_with_transport(
        &transport,
        "http://example.com/releases/latest",
        "1.1.0",
    )
    .expect_err("must reject http://");
    assert!(matches!(err, UpdateError::InsecureScheme));

    let rel = UpdateClient::check_latest_with_transport(
        &transport,
        "https://api.github.com/repos/ImageViewer/ImageViewer/releases/latest",
        "1.1.0",
    )
    .expect("valid release");
    assert_eq!(rel.version, "1.2.0");

    // Verify valid SHA-256 download succeeds
    let out_file = UpdateClient::download_and_verify(
        &transport,
        &rel.asset_download_url,
        &digest,
        dir.path(),
        "ImageViewer-1.2.0.dmg",
    )
    .expect("verified download");
    assert!(out_file.exists());

    // Verify mismatch deletes .part and fails
    let bad_digest = "0".repeat(64);
    let mismatch = UpdateClient::download_and_verify(
        &transport,
        &rel.asset_download_url,
        &bad_digest,
        dir.path(),
        "ImageViewer-bad.dmg",
    );
    assert!(matches!(mismatch, Err(UpdateError::ChecksumMismatch)));
    assert!(!dir.path().join("ImageViewer-bad.dmg.part").exists());

    // Sanitize Windows /RELAUNCH=
    assert!(UpdateClient::validate_relaunch_path(r"C:\Program Files\ImageViewer\ImageViewer.exe").is_ok());
    assert!(UpdateClient::validate_relaunch_path("C:\\bad\" /calc.exe").is_err());
    assert!(UpdateClient::validate_relaunch_path("C:\\bad\r\n.exe").is_err());

    assert_eq!(MAX_JSON_BYTES, 1024 * 1024);
    assert_eq!(MAX_SUMS_BYTES, 64 * 1024);
    assert_eq!(MAX_BINARY_BYTES, 256 * 1024 * 1024);
}
