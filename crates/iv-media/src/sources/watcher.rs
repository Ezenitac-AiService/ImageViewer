//! Debounced (`0.7 s`) folder watcher filtering symlinks.

use notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub struct FolderWatcher {
    _debouncer: Debouncer<notify::RecommendedWatcher>,
    pub dirty: Arc<AtomicBool>,
}

impl FolderWatcher {
    pub fn watch(dir: &Path, debounce: Duration) -> Result<Self, String> {
        let dirty = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&dirty);
        let mut debouncer = new_debouncer(debounce, move |res: DebounceEventResult| {
            if let Ok(events) = res {
                for ev in events {
                    if let Ok(meta) = std::fs::symlink_metadata(&ev.path)
                        && meta.file_type().is_symlink()
                    {
                        continue;
                    }
                    flag.store(true, Ordering::SeqCst);
                }
            }
        })
        .map_err(|e| e.to_string())?;

        debouncer
            .watcher()
            .watch(dir, RecursiveMode::NonRecursive)
            .map_err(|e| e.to_string())?;

        Ok(Self {
            _debouncer: debouncer,
            dirty,
        })
    }
}
