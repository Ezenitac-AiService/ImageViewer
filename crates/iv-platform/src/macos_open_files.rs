//! macOS "Open With" / Finder double-click support (specs/004 research R7, FR-012).
//!
//! winit 0.30.13 registers its own `WinitApplicationDelegate` when the event loop is created and
//! panics if the delegate is later replaced (`ApplicationDelegate::get`), so a separate delegate
//! cannot be installed (spike S2, research.md). Instead, once the event loop exists, this module
//! adds the optional `application:openURLs:` method to the class of the delegate winit installed.
//! AppKit checks `respondsToSelector:` when it delivers documents, so the method takes effect for
//! files opened before and after launch finishes.

use objc2::ffi;
use objc2::runtime::{AnyClass, AnyObject, MethodImplementation, Sel};
use objc2::{MainThreadMarker, sel};
use objc2_app_kit::NSApplication;
use objc2_foundation::{NSArray, NSURL};
use std::ffi::CStr;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Receives the file paths the OS asks the app to open.
pub type OpenFilesCallback = Box<dyn Fn(Vec<PathBuf>) + Send + Sync + 'static>;

static CALLBACK: OnceLock<OpenFilesCallback> = OnceLock::new();

/// Objective-C type encoding of `-(void)application:(NSApplication*)app openURLs:(NSArray*)urls`.
const OPEN_URLS_TYPES: &CStr = c"v@:@@";

extern "C-unwind" fn application_open_urls(
    _this: *mut AnyObject,
    _cmd: Sel,
    _application: *mut NSApplication,
    urls: *mut NSArray<NSURL>,
) {
    // SAFETY: AppKit passes a valid, non-null `NSArray<NSURL *> *` that stays alive for the
    // duration of this call; a null pointer is treated as "no files".
    let Some(urls) = (unsafe { urls.as_ref() }) else {
        return;
    };
    let paths: Vec<PathBuf> = urls
        .iter()
        .filter(|url| url.isFileURL())
        .filter_map(|url| url.path())
        .map(|p| PathBuf::from(p.to_string()))
        .collect();
    if !paths.is_empty()
        && let Some(callback) = CALLBACK.get()
    {
        callback(paths);
    }
}

/// Why the handler could not be attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachError {
    NotMainThread,
    NoDelegate,
    AlreadyAttached,
    MethodExists,
}

/// Attaches `on_open` to the application delegate. Call on the main thread **after** the winit
/// event loop has been created and **before** it runs.
pub fn attach(on_open: OpenFilesCallback) -> Result<(), AttachError> {
    let mtm = MainThreadMarker::new().ok_or(AttachError::NotMainThread)?;
    let app = NSApplication::sharedApplication(mtm);
    let delegate = app.delegate().ok_or(AttachError::NoDelegate)?;
    let object: &AnyObject = delegate.as_ref();
    let class: &AnyClass = object.class();
    CALLBACK
        .set(on_open)
        .map_err(|_| AttachError::AlreadyAttached)?;
    let imp = (application_open_urls
        as extern "C-unwind" fn(*mut AnyObject, Sel, *mut NSApplication, *mut NSArray<NSURL>))
        .__imp();
    // SAFETY: `class` is the live, registered class of the application delegate; the IMP has the
    // exact C ABI `void (id, SEL, NSApplication *, NSArray<NSURL *> *)` described by
    // `OPEN_URLS_TYPES`; and `class_addMethod` only adds a method when the class does not already
    // implement the selector, so no existing behaviour of winit's delegate is replaced.
    let added = unsafe {
        ffi::class_addMethod(
            (class as *const AnyClass).cast_mut(),
            sel!(application:openURLs:),
            imp,
            OPEN_URLS_TYPES.as_ptr(),
        )
    };
    if added.as_bool() {
        Ok(())
    } else {
        Err(AttachError::MethodExists)
    }
}
