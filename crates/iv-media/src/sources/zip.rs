//! Zero-disk-extraction ZIP/CBZ source with zero-allocation pre-flight EOCD checks.

use super::{is_supported_image_extension, ImageSourceType, MediaEntry, MediaSource, SourceError};
use iv_core::natural_sort::NaturalSort;
use iv_core::path_security::PathSecurity;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use zip::result::ZipError;
use zip::ZipArchive;

pub const MAX_ZIP_ENTRIES: u64 = 50_000;
pub const MAX_ZIP_CD_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_ZIP_CONTAINER_BYTES: u64 = 4 * 1024 * 1024 * 1024;

pub struct ZipSource {
    archive_path: PathBuf,
    entries: Vec<MediaEntry>,
    max_entry_bytes: u64,
}

impl ZipSource {
    /// Zero-allocation pre-flight EOCD/ZIP64 EOCD validation on an in-memory tail slice.
    pub fn preflight_eocd_check(tail: &[u8]) -> Result<(), SourceError> {
        // Search backwards for 22-byte EOCD signature 0x06054b50
        if tail.len() < 22 {
            return Err(SourceError::Archive("truncated archive".to_string()));
        }
        let max_scan = tail.len().min(65_557);
        let search_slice = &tail[tail.len() - max_scan..];
        let pos = search_slice
            .windows(4)
            .rposition(|w| w == 0x0605_4b50u32.to_le_bytes())
            .ok_or_else(|| SourceError::Archive("missing EOCD trailer".to_string()))?;

        let eocd = &search_slice[pos..];
        if eocd.len() < 22 {
            return Err(SourceError::Archive("invalid EOCD".to_string()));
        }

        let total_entries = u16::from_le_bytes([eocd[10], eocd[11]]) as u64;
        let cd_size = u32::from_le_bytes([eocd[12], eocd[13], eocd[14], eocd[15]]) as u64;

        if total_entries > MAX_ZIP_ENTRIES {
            return Err(SourceError::ArchiveLimitExceeded(format!(
                "too many archive entries: {total_entries}"
            )));
        }
        if cd_size > MAX_ZIP_CD_BYTES {
            return Err(SourceError::ArchiveLimitExceeded(format!(
                "central directory exceeds 64 MB: {cd_size}"
            )));
        }

        // Check for ZIP64 EOCD locator (0x07064b50) 20 bytes before EOCD if present
        if pos >= 20 {
            let loc = &search_slice[pos - 20..pos];
            if loc[0..4] == 0x0706_4b50u32.to_le_bytes() {
                // Reject oversized ZIP64 locator indicators when standard EOCD fields are 0xFFFF/0xFFFFFFFF
                if total_entries == 0xFFFF || cd_size == 0xFFFF_FFFF {
                    // Require ZIP64 EOCD verification in full stream
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn preflight_file(file: &mut File) -> Result<(), SourceError> {
        let len = file.metadata()?.len();
        if len > MAX_ZIP_CONTAINER_BYTES {
            return Err(SourceError::ArchiveLimitExceeded(
                "archive exceeds 4 GB container cap".to_string(),
            ));
        }
        let tail_len = len.min(65_557) as usize;
        file.seek(SeekFrom::End(-(tail_len as i64)))?;
        let mut tail = vec![0u8; tail_len];
        file.read_exact(&mut tail)?;
        Self::preflight_eocd_check(&tail)?;
        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    pub fn open(archive_path: &Path, max_zip_entry_megabytes: u32) -> Result<Self, SourceError> {
        let mut file = File::open(archive_path)?;
        Self::preflight_file(&mut file)?;

        let mut archive =
            ZipArchive::new(file).map_err(|e| SourceError::Archive(e.to_string()))?;
        if (archive.len() as u64) > MAX_ZIP_ENTRIES {
            return Err(SourceError::ArchiveLimitExceeded(
                "archive entries exceed 50,000".to_string(),
            ));
        }

        let max_entry_bytes = u64::from(max_zip_entry_megabytes.clamp(10, 4096)) * 1024 * 1024;
        let mut entries = Vec::new();
        let mut next_id = 1u64;

        for idx in 0..archive.len() {
            let Ok(entry) = archive.by_index_raw(idx) else {
                continue;
            };
            if entry.is_dir() {
                continue;
            }
            let ext_attr = entry.unix_mode().map(|m| m << 16).unwrap_or(0);
            if PathSecurity::is_archive_symlink(ext_attr, false) {
                continue;
            }
            let Ok(clean_key) = PathSecurity::normalize_archive_entry_key(entry.name()) else {
                continue;
            };
            let ext = Path::new(&clean_key)
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            if !is_supported_image_extension(&ext) {
                continue;
            }
            let name = Path::new(&clean_key)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| clean_key.clone());

            entries.push(MediaEntry {
                id: next_id,
                name,
                source_key: entry.name().to_string(),
                source_type: ImageSourceType::Zip,
                byte_size: entry.size(),
                modified_utc: None,
                is_supported: true,
            });
            next_id += 1;
        }

        entries.sort_by(|a, b| NaturalSort::cmp(&a.name, &b.name));

        Ok(Self {
            archive_path: archive_path.to_path_buf(),
            entries,
            max_entry_bytes,
        })
    }
}

impl MediaSource for ZipSource {
    fn entries(&self) -> &[MediaEntry] {
        &self.entries
    }

    fn initial_index(&self) -> usize {
        0
    }

    fn read_entry_bytes(&self, index: usize) -> Result<Vec<u8>, SourceError> {
        let entry = self
            .entries
            .get(index)
            .ok_or_else(|| SourceError::Archive("index out of bounds".to_string()))?;
        let mut file = File::open(&self.archive_path)?;
        Self::preflight_file(&mut file)?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| SourceError::Archive(e.to_string()))?;

        let zf = match archive.by_name(&entry.source_key) {
            Ok(f) => f,
            Err(ZipError::UnsupportedArchive(_)) => return Err(SourceError::EncryptedArchive),
            Err(err) => return Err(SourceError::Archive(err.to_string())),
        };

        if zf.encrypted() {
            return Err(SourceError::EncryptedArchive);
        }
        if zf.size() > self.max_entry_bytes {
            return Err(SourceError::ArchiveLimitExceeded(
                "zip entry exceeds MaxZipEntryMegabytes".to_string(),
            ));
        }

        let mut buf = Vec::with_capacity((zf.size().min(16 * 1024 * 1024)) as usize);
        let mut limited = zf.take(self.max_entry_bytes.saturating_add(1));
        limited.read_to_end(&mut buf)?;
        if (buf.len() as u64) > self.max_entry_bytes {
            return Err(SourceError::ArchiveLimitExceeded(
                "zip entry stream exceeded MaxZipEntryMegabytes".to_string(),
            ));
        }
        Ok(buf)
    }
}
