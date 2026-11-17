//! Narrow OS-idiomatic platform adapters: app paths, file dialogs, locale, fatal dialog, and
//! macOS open-with. Host codecs (`os-codecs`), shell registration (`shell-registration`) and the
//! updater (`updater`) are default-off features outside the shipped build (specs/004 FR-024).

#[cfg(feature = "os-codecs")]
pub mod codecs;
pub mod fatal_dialog;
pub mod integration;
pub mod locale;
#[cfg(target_os = "macos")]
pub mod macos_open_files;
pub mod paths_dialogs;
#[cfg(feature = "updater")]
pub mod update;
