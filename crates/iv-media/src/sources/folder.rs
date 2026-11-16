//! Local directory media source with symlink skip and natural ordering.

use super::{ImageSourceType, MediaEntry, MediaSource, SourceError, is_supported_image_extension};
use iv_core::natural_sort::NaturalSort;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

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
        Ok(std::fs::read(&entry.source_key)?)
    }
}
