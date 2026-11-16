//! Unconditional `https://` + `sha2` SHA-256 auto-updater with bounded streams and `HttpFeedTransport`.

use iv_core::path_security::PathSecurity;
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const MAX_JSON_BYTES: usize = 1024 * 1024;
pub const MAX_SUMS_BYTES: usize = 64 * 1024;
pub const MAX_BINARY_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("only https:// URLs are permitted")]
    InsecureScheme,
    #[error("HTTP response exceeded maximum allowed byte cap")]
    BodyTooLarge,
    #[error("SHA-256 checksum mismatch")]
    ChecksumMismatch,
    #[error("unsafe target filename or path")]
    UnsafePath,
    #[error("unsafe Windows /RELAUNCH= path")]
    UnsafeRelaunchPath,
    #[error("network or parse error: {0}")]
    Transport(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseInfo {
    pub version: String,
    pub tag: String,
    pub page_url: String,
    pub notes: String,
    pub asset_name: String,
    pub asset_download_url: String,
}

pub trait HttpFeedTransport: Send + Sync {
    fn get_bounded(&self, url: &str, max_bytes: usize) -> Result<Vec<u8>, UpdateError>;
}

pub struct ReqwestTlsTransport {
    client: Client,
}

impl ReqwestTlsTransport {
    pub fn new() -> Result<Self, UpdateError> {
        let client = Client::builder()
            .use_rustls_tls()
            .build()
            .map_err(|e| UpdateError::Transport(e.to_string()))?;
        Ok(Self { client })
    }
}

impl HttpFeedTransport for ReqwestTlsTransport {
    fn get_bounded(&self, url: &str, max_bytes: usize) -> Result<Vec<u8>, UpdateError> {
        if !url.starts_with("https://") {
            return Err(UpdateError::InsecureScheme);
        }
        let resp = self
            .client
            .get(url)
            .send()
            .map_err(|e| UpdateError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(UpdateError::Transport(format!("HTTP {}", resp.status())));
        }
        let mut buf = Vec::new();
        let mut limited = resp.take((max_bytes as u64).saturating_add(1));
        limited
            .read_to_end(&mut buf)
            .map_err(|e| UpdateError::Transport(e.to_string()))?;
        if buf.len() > max_bytes {
            return Err(UpdateError::BodyTooLarge);
        }
        Ok(buf)
    }
}

#[derive(Deserialize)]
struct GitHubReleaseDto {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    prerelease: bool,
    draft: bool,
    assets: Vec<GitHubAssetDto>,
}

#[derive(Deserialize)]
struct GitHubAssetDto {
    name: String,
    browser_download_url: String,
}

pub struct UpdateClient;

impl UpdateClient {
    pub fn check_latest_with_transport(
        transport: &dyn HttpFeedTransport,
        feed_url: &str,
        _current_version: &str,
    ) -> Result<ReleaseInfo, UpdateError> {
        if !feed_url.starts_with("https://") {
            return Err(UpdateError::InsecureScheme);
        }
        let bytes = transport.get_bounded(feed_url, MAX_JSON_BYTES)?;
        let dto: GitHubReleaseDto =
            serde_json::from_slice(&bytes).map_err(|e| UpdateError::Transport(e.to_string()))?;
        if dto.prerelease || dto.draft || !dto.html_url.starts_with("https://") {
            return Err(UpdateError::InsecureScheme);
        }
        let asset = dto
            .assets
            .into_iter()
            .find(|a| a.browser_download_url.starts_with("https://"))
            .ok_or_else(|| UpdateError::Transport("no https asset".to_string()))?;

        let version = dto.tag_name.trim_start_matches('v').to_string();
        Ok(ReleaseInfo {
            version,
            tag: dto.tag_name,
            page_url: dto.html_url,
            notes: dto.body.unwrap_or_default(),
            asset_name: asset.name,
            asset_download_url: asset.browser_download_url,
        })
    }

    pub fn download_and_verify(
        transport: &dyn HttpFeedTransport,
        download_url: &str,
        expected_sha256_hex: &str,
        dest_dir: &Path,
        filename: &str,
    ) -> Result<PathBuf, UpdateError> {
        if !download_url.starts_with("https://") {
            return Err(UpdateError::InsecureScheme);
        }
        if expected_sha256_hex.len() != 64
            || !expected_sha256_hex
                .chars()
                .all(|c| c.is_ascii_hexdigit())
        {
            return Err(UpdateError::ChecksumMismatch);
        }
        if PathSecurity::ensure_not_symlink(dest_dir).is_err() {
            return Err(UpdateError::UnsafePath);
        }
        let final_path =
            PathSecurity::safe_join_filename(dest_dir, filename).map_err(|_| UpdateError::UnsafePath)?;
        let part_name = format!("{filename}.part");
        let part_path = PathSecurity::safe_join_filename(dest_dir, &part_name)
            .map_err(|_| UpdateError::UnsafePath)?;

        let bytes = transport.get_bounded(download_url, MAX_BINARY_BYTES)?;
        {
            let mut f = File::create(&part_path).map_err(|e| UpdateError::Transport(e.to_string()))?;
            f.write_all(&bytes)
                .map_err(|e| UpdateError::Transport(e.to_string()))?;
            f.sync_all().map_err(|e| UpdateError::Transport(e.to_string()))?;
        }

        let actual_hex = format!("{:x}", Sha256::digest(&bytes));
        if !actual_hex.eq_ignore_ascii_case(expected_sha256_hex) {
            let _ = std::fs::remove_file(&part_path);
            return Err(UpdateError::ChecksumMismatch);
        }

        std::fs::rename(&part_path, &final_path)
            .map_err(|e| UpdateError::Transport(e.to_string()))?;
        Ok(final_path)
    }

    pub fn validate_relaunch_path(path: &str) -> Result<(), UpdateError> {
        if path.is_empty()
            || path.contains('"')
            || path.contains(" /")
            || path.chars().any(|c| c.is_control())
        {
            return Err(UpdateError::UnsafeRelaunchPath);
        }
        Ok(())
    }
}
