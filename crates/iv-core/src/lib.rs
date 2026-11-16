#![forbid(unsafe_code)]
//! Pure-Rust, zero-`unsafe` core domain, security, layout, and state orchestration for ImageViewer.

pub mod chrome_state;
pub mod cli;
pub mod codecs;
pub mod format_guard;
pub mod i18n;
pub mod layout;
pub mod logger;
pub mod natural_sort;
pub mod path_security;
pub mod settings;
pub mod slideshow;
pub mod trackpad;
