//! Launch diagnostic records (`specs/004-launch-reliability-refactor/contracts/launch-and-diagnostics-contract.md`).
//!
//! Produces the body of each diagnostic line (`launch=<id> phase=<name> key=value ...`). The
//! timestamp and level prefix are added by [`crate::logger::AsyncFileLogger`]. Every value is
//! passed through [`PathRedactor`] so records never carry full host paths or control characters.

use crate::path_security::PathRedactor;
use rand::Rng;

/// Per-launch identifier: exactly 8 lowercase hex characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchId(String);

impl LaunchId {
    pub fn from_u32(value: u32) -> Self {
        Self(format!("{value:08x}"))
    }

    pub fn random() -> Self {
        Self::from_u32(rand::rng().random())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Startup and runtime phases recorded for every launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Start,
    Settings,
    Probe,
    Window,
    FirstFrame,
    Navigate,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Settings => "settings",
            Self::Probe => "probe",
            Self::Window => "window",
            Self::FirstFrame => "first_frame",
            Self::Navigate => "navigate",
        }
    }
}

/// Cause categories for failures that end a launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FatalCategory {
    NoRenderer,
    WindowFailed,
    SettingsUnwritable,
    Panic,
}

impl FatalCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoRenderer => "no_renderer",
            Self::WindowFailed => "window_failed",
            Self::SettingsUnwritable => "settings_unwritable",
            Self::Panic => "panic",
        }
    }
}

/// Process exit codes reported to the operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Clean,
    FatalStartup,
    Panic,
}

impl ExitCode {
    pub fn code(self) -> i32 {
        match self {
            Self::Clean => 0,
            Self::FatalStartup => 2,
            Self::Panic => 3,
        }
    }
}

/// Formats diagnostic line bodies for one launch.
#[derive(Debug, Clone)]
pub struct LaunchRecord {
    id: LaunchId,
}

impl LaunchRecord {
    pub fn new(id: LaunchId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &LaunchId {
        &self.id
    }

    /// The mandatory first line of every launch.
    pub fn start(&self, version: &str, os: &str, os_version: &str, arch: &str) -> String {
        self.phase(
            Phase::Start,
            &[
                ("version", version),
                ("os", os),
                ("os_version", os_version),
                ("arch", arch),
            ],
        )
    }

    pub fn phase(&self, phase: Phase, fields: &[(&str, &str)]) -> String {
        let mut line = format!("launch={} phase={}", self.id.as_str(), phase.as_str());
        append_fields(&mut line, fields);
        line
    }

    pub fn fatal(&self, category: FatalCategory, fields: &[(&str, &str)]) -> String {
        let mut line = format!(
            "launch={} fatal category={}",
            self.id.as_str(),
            category.as_str()
        );
        append_fields(&mut line, fields);
        line
    }

    pub fn exit_clean(&self) -> String {
        format!("launch={} exit=clean", self.id.as_str())
    }
}

/// Returns the file name of `path` (either separator style), or `none` when empty.
pub fn source_name(path: &str) -> String {
    let name = path.rsplit(['/', '\\']).find(|seg| !seg.is_empty());
    match name {
        Some(n) => n.to_string(),
        None => "none".to_string(),
    }
}

fn append_fields(line: &mut String, fields: &[(&str, &str)]) {
    for (key, value) in fields {
        line.push(' ');
        line.push_str(key);
        line.push('=');
        line.push_str(&format_value(value));
    }
}

fn format_value(raw: &str) -> String {
    let redacted = PathRedactor::redact(raw);
    let cleaned: String = redacted
        .chars()
        .filter(|c| *c != '"' && !c.is_control())
        .collect();
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() {
        "\"\"".to_string()
    } else if cleaned.contains(' ') {
        format!("\"{cleaned}\"")
    } else {
        cleaned
    }
}

/// Installs a process-wide panic hook that writes one `fatal category=panic at=<file>:<line>`
/// record through `sink`, then calls `on_fatal` (which typically shows the fatal dialog and
/// exits with [`ExitCode::Panic`]). The previous hook still runs afterwards so the message also
/// reaches stderr when a console exists.
pub fn install_panic_hook(
    record: LaunchRecord,
    sink: impl Fn(&str) + Send + Sync + 'static,
    on_fatal: impl Fn() + Send + Sync + 'static,
) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let at = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown".to_string());
        sink(&record.fatal(FatalCategory::Panic, &[("at", &at)]));
        previous(info);
        on_fatal();
    }));
}
