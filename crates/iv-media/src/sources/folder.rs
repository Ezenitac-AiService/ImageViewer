//! Local directory media source with symlink skip and natural ordering.

use super::{ImageSourceType, MediaEntry, MediaSource, SourceError, is_supported_image_extension};
use iv_core::natural_sort::NaturalSort;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::UNIX_EPOCH;

/// Largest file read into memory (constitution v2.4.0 safety caps: 512 MB stream cap).
pub const MAX_FILE_READ_BYTES: u64 = 512 * 1024 * 1024;

pub struct FolderSource {
    entries: Vec<MediaEntry>,
    initial_index: usize,
}

impl FolderSource {
    /// Opens a folder (or parent folder of `target_path`), skipping symlink/reparse entries.
    pub fn open(target_path: &Path) -> Result<Self, SourceError> {
        let meta = std::fs::metadata(target_path)?;
        let (dir_path, target_file_name): (PathBuf, Option<String>) = if meta.is_dir() {
            (target_path.to_path_buf(), None)
        } else {
            let parent = target_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            let fname = target_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string());
            (parent, fname)
        };

        let mut entries = Vec::new();
        let mut next_id = 1u64;
        for item in std::fs::read_dir(&dir_path)? {
            let Ok(entry) = item else { continue };
            let path = entry.path();
            let Ok(sym_meta) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            // Skip symlinks and directories
            if sym_meta.file_type().is_symlink() || sym_meta.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            let is_supported = is_supported_image_extension(&ext);
            if !is_supported && target_file_name.as_deref() != Some(name.as_str()) {
                continue;
            }
            let modified_utc = sym_meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);
            entries.push(MediaEntry {
                id: next_id,
                name,
                source_key: path.to_string_lossy().to_string(),
                source_type: ImageSourceType::File,
                byte_size: sym_meta.len(),
                modified_utc,
                is_supported,
            });
            next_id += 1;
        }

        entries.sort_by(|a, b| NaturalSort::cmp(&a.name, &b.name));

        let initial_index = target_file_name
            .as_deref()
            .and_then(|target| entries.iter().position(|e| e.name == target))
            .unwrap_or(0);

        Ok(Self {
            entries,
            initial_index,
        })
    }
}

impl FolderSource {
    /// A one-entry source for `file`, used to show a targeted image before its folder is scanned.
    pub fn from_single_file(file: &Path) -> Result<Self, SourceError> {
        let meta = std::fs::symlink_metadata(file)?;
        if meta.file_type().is_symlink() || meta.is_dir() {
            return Err(SourceError::SecurityViolation);
        }
        let name = file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or(SourceError::SecurityViolation)?;
        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        let modified_utc = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);
        Ok(Self {
            entries: vec![MediaEntry {
                id: 1,
                name,
                source_key: file.to_string_lossy().to_string(),
                source_type: ImageSourceType::File,
                byte_size: meta.len(),
                modified_utc,
                is_supported: is_supported_image_extension(&ext),
            }],
            initial_index: 0,
        })
    }
}

/// Progress of [`scan_async`].
pub enum ScanEvent {
    /// The targeted file alone, available before the folder scan completes.
    Partial(FolderSource),
    /// The fully scanned, naturally sorted folder.
    Complete(Result<FolderSource, SourceError>),
}

/// Scans `target` (a folder, or a file whose folder is scanned) on a worker thread so the UI
/// never blocks on directory enumeration (FR-017). When `target` is a file, a
/// [`ScanEvent::Partial`] with just that file is sent first.
pub fn scan_async(target: PathBuf) -> mpsc::Receiver<ScanEvent> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let is_file = std::fs::metadata(&target).is_ok_and(|m| m.is_file());
        if is_file
            && let Ok(single) = FolderSource::from_single_file(&target)
            && tx.send(ScanEvent::Partial(single)).is_err()
        {
            return;
        }
        // The receiver may already be gone (a newer open superseded this scan); nothing to do.
        let _ = tx.send(ScanEvent::Complete(FolderSource::open(&target)));
    });
    rx
}

/// Rescans after a change, keeping the current entry when it still exists
/// (`data-model.md` §8). Returns the new source and the index to show: the entry whose
/// `source_key` equals `current_key`, else `old_index` clamped into range, else `None` when empty.
pub fn rescan_keeping(
    target: &Path,
    current_key: Option<&str>,
    old_index: Option<usize>,
) -> Result<(FolderSource, Option<usize>), SourceError> {
    let dir = if target.is_dir() {
        target.to_path_buf()
    } else {
        target
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };
    let source = FolderSource::open(&dir)?;
    let len = source.entries.len();
    let index = if len == 0 {
        None
    } else {
        current_key
            .and_then(|key| source.entries.iter().position(|e| e.source_key == key))
            .or_else(|| old_index.map(|i| i.min(len - 1)))
            .or(Some(0))
    };
    Ok((source, index))
}

impl MediaSource for FolderSource {
    fn entries(&self) -> &[MediaEntry] {
        &self.entries
    }

    fn initial_index(&self) -> usize {
        self.initial_index
    }

    fn read_entry_bytes(&self, index: usize) -> Result<Vec<u8>, SourceError> {
        let entry = self
            .entries
            .get(index)
            .ok_or_else(|| SourceError::Archive("index out of bounds".to_string()))?;
        read_capped(Path::new(&entry.source_key), MAX_FILE_READ_BYTES)
    }
}

/// Reads `path` only if it is at most `cap` bytes, checking the size before allocating and again
/// while reading (the file may grow in between).
fn read_capped(path: &Path, cap: u64) -> Result<Vec<u8>, SourceError> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    if len > cap {
        return Err(SourceError::ArchiveLimitExceeded(format!(
            "file is {len} bytes; limit is {cap}"
        )));
    }
    let mut buf = Vec::with_capacity(usize::try_from(len).unwrap_or(0));
    file.take(cap + 1).read_to_end(&mut buf)?;
    if buf.len() as u64 > cap {
        return Err(SourceError::ArchiveLimitExceeded(format!(
            "file grew past the {cap}-byte limit while reading"
        )));
    }
    Ok(buf)
}
