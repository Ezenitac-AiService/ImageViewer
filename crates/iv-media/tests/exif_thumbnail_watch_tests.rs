#![allow(dead_code)]
mod fixtures;

use iv_media::cache::{ThumbnailCache, ThumbnailEntry};
use iv_media::exif::ExifExtractor;
use iv_media::sources::watcher::FolderWatcher;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn exif_extractor_handles_corrupt_bytes_gracefully_and_thumbnail_cache_caps_at_300() {
    let items = ExifExtractor::extract(b"not valid exif");
    assert!(items.is_empty());

    let mut thumbs = ThumbnailCache::default();
    for i in 0..320 {
        thumbs.insert(
            format!("t{i}"),
            ThumbnailEntry {
                width: 96,
                height: 96,
                rgba8: Arc::new(vec![0; 4]),
            },
        );
    }
    assert_eq!(thumbs.len(), 300);
}

#[test]
fn folder_watcher_initializes_with_700ms_debounce() {
    let dir = fixtures::TempTestDir::new("watcher");
    let watcher = FolderWatcher::watch(dir.path(), Duration::from_millis(700));
    assert!(watcher.is_ok());
}
