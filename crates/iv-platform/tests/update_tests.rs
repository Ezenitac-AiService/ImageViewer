#![cfg(feature = "updater")]
#![allow(dead_code)]
mod fixtures;

use iv_platform::update::{
    HttpFeedTransport, MAX_BINARY_BYTES, MAX_JSON_BYTES, MAX_SUMS_BYTES, UpdateClient, UpdateError,
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
            "html_url": "https://github.com/Ezenitac-AiService/ImageViewer/releases/tag/v1.2.0",
            "body": "Release notes",
            "prerelease": false,
            "draft": false,
            "assets": [{
                "name": "ImageViewer-1.2.0.dmg",
                "browser_download_url": "https://github.com/Ezenitac-AiService/ImageViewer/releases/download/v1.2.0/ImageViewer-1.2.0.dmg"
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
        "https://api.github.com/repos/Ezenitac-AiService/ImageViewer/releases/latest",
        "1.1.0",
    )
    .expect("valid release");
    assert_eq!(rel.version, "1.2.0");
    assert_eq!(rel.sha256.as_deref(), Some(digest.as_str()));

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
    assert!(
        UpdateClient::validate_relaunch_path(r"C:\Program Files\ImageViewer\ImageViewer.exe")
            .is_ok()
    );
    assert!(UpdateClient::validate_relaunch_path("C:\\bad\" /calc.exe").is_err());
    assert!(UpdateClient::validate_relaunch_path("C:\\bad\r\n.exe").is_err());

    assert_eq!(MAX_JSON_BYTES, 1024 * 1024);
    assert_eq!(MAX_SUMS_BYTES, 64 * 1024);
    assert_eq!(MAX_BINARY_BYTES, 256 * 1024 * 1024);
}

#[test]
fn update_client_selects_os_matching_release_asset() {
    let mac_digest = "b".repeat(64);
    let linux_digest = "c".repeat(64);

    let transport = MemoryFeedTransport {
        json_body: serde_json::json!({
            "tag_name": "v1.2.0",
            "html_url": "https://github.com/Ezenitac-AiService/ImageViewer/releases/tag/v1.2.0",
            "body": "Cross-platform release",
            "prerelease": false,
            "draft": false,
            "assets": [
                {
                    "name": "SHA256SUMS.txt",
                    "browser_download_url": "https://github.com/Ezenitac-AiService/ImageViewer/releases/download/v1.2.0/SHA256SUMS.txt"
                },
                {
                    "name": "ImageViewer-linux-x64.tar.gz",
                    "browser_download_url": "https://github.com/Ezenitac-AiService/ImageViewer/releases/download/v1.2.0/ImageViewer-linux-x64.tar.gz"
                },
                {
                    "name": "ImageViewer-macos.dmg",
                    "browser_download_url": "https://github.com/Ezenitac-AiService/ImageViewer/releases/download/v1.2.0/ImageViewer-macos.dmg"
                }
            ]
        })
        .to_string()
        .into_bytes(),
        sums_body: format!(
            "{mac_digest}  ImageViewer-macos.dmg\n\
             {linux_digest}  ImageViewer-linux-x64.tar.gz\n"
        )
        .into_bytes(),
        bin_body: Vec::new(),
    };

    let feed = "https://api.github.com/repos/Ezenitac-AiService/ImageViewer/releases/latest";

    let mac = UpdateClient::check_latest_with_transport_for_os(&transport, feed, "1.1.0", "macos")
        .expect("macos asset");
    assert_eq!(mac.asset_name, "ImageViewer-macos.dmg");
    assert_eq!(mac.sha256.as_deref(), Some(mac_digest.as_str()));

    let linux =
        UpdateClient::check_latest_with_transport_for_os(&transport, feed, "1.1.0", "linux")
            .expect("linux asset");
    assert_eq!(linux.asset_name, "ImageViewer-linux-x64.tar.gz");
    assert_eq!(linux.sha256.as_deref(), Some(linux_digest.as_str()));
}

#[test]
fn reqwest_tls_transport_accepts_custom_root_store_over_rcgen_loopback_https() {
    use iv_platform::update::ReqwestTlsTransport;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Arc;

    let (cert_der, key_der) = fixtures::generate_loopback_cert();
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from(cert_der.clone()))
        .expect("add loopback cert");

    let server_cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(cert_der)],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der)),
        )
        .expect("server config");
    let server_cfg = Arc::new(server_cfg);

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().expect("addr").port();

    let handle = std::thread::spawn(move || {
        if let Ok((mut tcp, _)) = listener.accept() {
            let conn = rustls::ServerConnection::new(server_cfg).expect("server conn");
            let mut tls = rustls::StreamOwned::new(conn, &mut tcp);
            let mut req_buf = [0u8; 1024];
            let _ = tls.read(&mut req_buf);
            let body = br#"{"ok":true}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = tls.write_all(resp.as_bytes());
            let _ = tls.write_all(body);
            let _ = tls.flush();
        }
    });

    let transport = ReqwestTlsTransport::with_root_certs(roots).expect("tls transport");
    let url = format!("https://localhost:{port}/latest.json");
    let body = transport
        .get_bounded(&url, 1024)
        .expect("loopback https get");
    assert_eq!(body, br#"{"ok":true}"#);
    let _ = handle.join();
}
