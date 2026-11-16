//! OS app-data directories and non-blocking `rfd::AsyncFileDialog` file/folder pickers.

use directories::ProjectDirs;
use std::path::PathBuf;
use winit::event_loop::EventLoopProxy;

/// User-facing dialog completion events sent asynchronously to `winit::ApplicationHandler`.
#[derive(Debug, Clone)]
pub enum AppUserEvent {
    PickedOpenFile(Option<PathBuf>),
    PickedOpenFolder(Option<PathBuf>),
    OpenUrlPaths(Vec<PathBuf>),
    DecodeWakeup,
    UpdateChecked,
}

pub struct OsPaths;

impl OsPaths {
    /// Returns the OS-idiomatic configuration directory (`%LOCALAPPDATA%\ImageViewer` on Windows).
    pub fn config_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local).join("ImageViewer");
        }
        ProjectDirs::from("com", "ImageViewer", "ImageViewer")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| std::env::temp_dir().join("ImageViewer"))
    }

    pub fn settings_file() -> PathBuf {
        Self::config_dir().join("settings.json")
    }

    pub fn logs_dir() -> PathBuf {
        Self::config_dir().join("logs")
    }
}

pub struct OsDialogs;

impl OsDialogs {
    /// Launches a non-blocking native file/archive picker and forwards the result via `EventLoopProxy`.
    pub fn pick_image_or_archive_async(proxy: EventLoopProxy<AppUserEvent>) {
        let dialog = rfd::AsyncFileDialog::new().add_filter(
            "Images & Archives",
            &[
                "jpg", "jpeg", "png", "apng", "gif", "webp", "bmp", "tif", "tiff", "ico", "cur",
                "jxl", "psd", "dds", "pcx", "tga", "qoi", "exr", "hdr", "heic", "heif", "avif",
                "dng", "cr2", "cr3", "nef", "arw", "zip", "cbz",
            ],
        );
        std::thread::spawn(move || {
            let picked = pollster_block(dialog.pick_file()).map(|f| f.path().to_path_buf());
            let _ = proxy.send_event(AppUserEvent::PickedOpenFile(picked));
        });
    }

    /// Launches a non-blocking native folder picker and forwards the result via `EventLoopProxy`.
    pub fn pick_folder_async(proxy: EventLoopProxy<AppUserEvent>) {
        let dialog = rfd::AsyncFileDialog::new();
        std::thread::spawn(move || {
            let picked = pollster_block(dialog.pick_folder()).map(|f| f.path().to_path_buf());
            let _ = proxy.send_event(AppUserEvent::PickedOpenFolder(picked));
        });
    }
}

fn pollster_block<F: std::future::Future>(fut: F) -> F::Output {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    match rt {
        Ok(r) => r.block_on(fut),
        Err(_) => futures_noop_block(fut),
    }
}

fn futures_noop_block<F: std::future::Future>(fut: F) -> F::Output {
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    fn noop(_: *const ()) {}
    fn clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    let mut pinned = std::pin::pin!(fut);
    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
