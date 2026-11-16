//! Bounded `Arc<DecodedFrame>` LRU (`<= 8`) and thumbnail LRU (`<= 300`).

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataItem {
    pub label_key: &'static str,
    pub value: String,
}

/// Immutable decoded frame retaining true post-orientation `source_width`/`source_height`.
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    pub source_width: u32,
    pub source_height: u32,
    pub buffer_width: u32,
    pub buffer_height: u32,
    pub rgba8: Vec<u8>,
    pub is_downscaled: bool,
    pub page_count: u32,
    pub page_index: u32,
    pub exif_orientation_applied: bool,
    pub color_profile_converted: bool,
    pub format_label: String,
    pub exif_items: Vec<MetadataItem>,
}

impl DecodedFrame {
    pub fn empty_for_test(width: u32, height: u32) -> Self {
        Self {
            source_width: width,
            source_height: height,
            buffer_width: width,
            buffer_height: height,
            rgba8: vec![0u8; (width as usize) * (height as usize) * 4],
            is_downscaled: false,
            page_count: 1,
            page_index: 0,
            exif_orientation_applied: false,
            color_profile_converted: false,
            format_label: "TEST".to_string(),
            exif_items: Vec::new(),
        }
    }
}

pub struct DecodedFrameCache {
    inner: LruCache<String, Arc<DecodedFrame>>,
}

impl DecodedFrameCache {
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity.max(1)).unwrap_or(NonZeroUsize::MIN);
        Self {
            inner: LruCache::new(cap),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<Arc<DecodedFrame>> {
        self.inner.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, frame: Arc<DecodedFrame>) {
        self.inner.put(key, frame);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

#[derive(Debug, Clone)]
pub struct ThumbnailEntry {
    pub width: u32,
    pub height: u32,
    pub rgba8: Arc<Vec<u8>>,
}

pub struct ThumbnailCache {
    inner: LruCache<String, ThumbnailEntry>,
}

impl Default for ThumbnailCache {
    fn default() -> Self {
        Self::new(300)
    }
}

impl ThumbnailCache {
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity.max(1)).unwrap_or(NonZeroUsize::MIN);
        Self {
            inner: LruCache::new(cap),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<ThumbnailEntry> {
        self.inner.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, thumb: ThumbnailEntry) {
        self.inner.put(key, thumb);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
