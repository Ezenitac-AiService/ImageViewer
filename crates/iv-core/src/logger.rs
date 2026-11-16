//! Non-blocking bounded channel logger with path redaction, 7-day rotation, and bounded shutdown flush.

use crate::path_security::{PathRedactor, PathSecurity};
use chrono::Local;
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime};

/// Threshold above which slow render diagnostics are emitted.
pub const SLOW_RENDER_THRESHOLD_MS: u128 = 300;
/// Maximum retained daily log files.
pub const MAX_LOG_RETENTION_DAYS: u64 = 7;

#[derive(Debug)]
enum LogCmd {
    Record(String),
    Flush(Arc<(Mutex<bool>, Condvar)>),
    Shutdown,
}

/// Bounded `DropOldest` non-blocking diagnostic logger.
pub struct AsyncFileLogger {
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    worker: Option<JoinHandle<()>>,
}

struct QueueState {
    items: VecDeque<LogCmd>,
    capacity: usize,
    closed: bool,
}

impl AsyncFileLogger {
    /// Starts a background writer thread into `log_dir` (skipping symlinks and rotating >7-day logs).
    pub fn new(log_dir: &Path, capacity: usize) -> std::io::Result<Self> {
        std::fs::create_dir_all(log_dir)?;
        if PathSecurity::ensure_not_symlink(log_dir).is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "log directory is a symlink",
            ));
        }
        cleanup_old_logs(log_dir);
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let file_path = log_dir.join(format!("imageviewer-{date_str}.log"));

        let queue = Arc::new((
            Mutex::new(QueueState {
                items: VecDeque::with_capacity(capacity.max(8)),
                capacity: capacity.max(8),
                closed: false,
            }),
            Condvar::new(),
        ));
        let worker_queue = Arc::clone(&queue);
        let worker = thread::Builder::new()
            .name("iv-logger".to_string())
            .spawn(move || writer_loop(file_path, worker_queue))?;

        Ok(Self {
            queue,
            worker: Some(worker),
        })
    }

    pub fn log_info(&self, raw_message: &str) {
        self.push_line("INFO", raw_message);
    }

    pub fn log_error(&self, raw_message: &str) {
        self.push_line("ERROR", raw_message);
    }

    pub fn log_slow_render(&self, label: &str, elapsed: Duration) {
        let ms = elapsed.as_millis();
        if ms >= SLOW_RENDER_THRESHOLD_MS {
            self.push_line(
                "PERF",
                &format!("slow_render item={label} elapsed={}ms", ms),
            );
        }
    }

    /// Writes one line and waits (up to 2 s) until it has reached disk.
    ///
    /// Used for launch records that must survive an immediate process exit
    /// (`contracts/launch-and-diagnostics-contract.md` §1).
    pub fn write_sync(&self, level: &str, raw_message: &str) {
        self.push_line(level, raw_message);
        self.flush_bounded(Duration::from_secs(2));
    }

    fn push_line(&self, level: &str, raw_message: &str) {
        let ts = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f");
        let redacted = PathRedactor::redact(raw_message);
        let line = format!("[{ts}] [{level}] {redacted}\n");
        let (lock, cv) = &*self.queue;
        if let Ok(mut state) = lock.lock() {
            if state.closed {
                return;
            }
            if state.items.len() >= state.capacity {
                // Drop oldest record message (never drop Flush/Shutdown)
                if let Some(pos) = state
                    .items
                    .iter()
                    .position(|cmd| matches!(cmd, LogCmd::Record(_)))
                {
                    state.items.remove(pos);
                }
            }
            state.items.push_back(LogCmd::Record(line));
            cv.notify_one();
        }
    }

    /// Waits up to `timeout` for pending log records to hit disk.
    pub fn flush_bounded(&self, timeout: Duration) {
        let done = Arc::new((Mutex::new(false), Condvar::new()));
        {
            let (lock, cv) = &*self.queue;
            if let Ok(mut state) = lock.lock() {
                if state.closed {
                    return;
                }
                state.items.push_back(LogCmd::Flush(Arc::clone(&done)));
                cv.notify_one();
            }
        }
        let (done_lock, done_cv) = &*done;
        let deadline = Instant::now() + timeout;
        if let Ok(mut finished) = done_lock.lock() {
            while !*finished {
                let Some(rem) = deadline.checked_duration_since(Instant::now()) else {
                    break;
                };
                let Ok((guard, wait_res)) = done_cv.wait_timeout(finished, rem) else {
                    break;
                };
                finished = guard;
                if wait_res.timed_out() {
                    break;
                }
            }
        }
    }
}

impl Drop for AsyncFileLogger {
    fn drop(&mut self) {
        self.flush_bounded(Duration::from_secs(2));
        {
            let (lock, cv) = &*self.queue;
            if let Ok(mut state) = lock.lock() {
                state.closed = true;
                state.items.push_back(LogCmd::Shutdown);
                cv.notify_all();
            }
        }
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

fn writer_loop(file_path: PathBuf, queue: Arc<(Mutex<QueueState>, Condvar)>) {
    let mut file = if PathSecurity::ensure_not_symlink(&file_path).is_ok() {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .ok()
    } else {
        None
    };

    loop {
        let cmd = {
            let (lock, cv) = &*queue;
            let Ok(mut guard) = lock.lock() else {
                break;
            };
            while guard.items.is_empty() {
                let Ok(next) = cv.wait(guard) else {
                    return;
                };
                guard = next;
            }
            guard.items.pop_front()
        };

        match cmd {
            Some(LogCmd::Record(line)) => {
                if let Some(f) = file.as_mut() {
                    let _ = f.write_all(line.as_bytes());
                }
            }
            Some(LogCmd::Flush(done)) => {
                if let Some(f) = file.as_mut() {
                    let _ = f.flush();
                }
                let (dlock, dcv) = &*done;
                if let Ok(mut flag) = dlock.lock() {
                    *flag = true;
                    dcv.notify_all();
                }
            }
            Some(LogCmd::Shutdown) | None => {
                if let Some(f) = file.as_mut() {
                    let _ = f.flush();
                }
                break;
            }
        }
    }
}

fn cleanup_old_logs(dir: &Path) {
    let max_age = Duration::from_secs(MAX_LOG_RETENTION_DAYS * 86_400);
    let now = SystemTime::now();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("log") {
                continue;
            }
            if let Ok(meta) = entry.metadata()
                && let Ok(modified) = meta.modified()
                && let Ok(age) = now.duration_since(modified)
                && age > max_age
            {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}
