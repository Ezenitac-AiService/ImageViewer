//! Narrow OS-idiomatic platform adapters (host codecs, file dialogs, shell integration, HTTPS updates).

pub mod codecs;
pub mod fatal_dialog;
pub mod integration;
pub mod locale;
#[cfg(target_os = "macos")]
pub mod macos_open_files;
pub mod paths_dialogs;
pub mod update;
