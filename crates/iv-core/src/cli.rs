//! Fail-closed CLI argument parser matching `contracts/core-security-and-cli-contract.md`.

use crate::i18n::{I18n, Language};
use std::path::PathBuf;

/// Validated startup command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    OpenEmpty,
    OpenPath(PathBuf),
}

/// Parsed CLI options and fail-closed diagnostic state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOptions {
    pub command: CliCommand,
    pub error: Option<String>,
    pub is_path_not_found_error: bool,
    pub requested_path: Option<String>,
}

impl CliOptions {
    /// Parses raw process `argv` (`args[0]` is executable name).
    pub fn parse(args: &[String], lang: Language, is_windows: bool) -> Self {
        let mut paths: Vec<String> = Vec::new();

        for raw_arg in args.iter().skip(1) {
            // Benign macOS LaunchServices Finder session identifier
            if raw_arg.starts_with("-psn_") {
                continue;
            }

            let trimmed = raw_arg.trim();
            // Shell and file-association registration is Unavailable in this release
            // (specs/004 FR-024); the switches are recognised only to explain that.
            let switch = trimmed.to_ascii_lowercase();
            if matches!(
                switch.as_str(),
                "--register-shell" | "/register-shell" | "--unregister-shell" | "/unregister-shell"
            ) {
                return Self::fail(I18n::tr(lang, "error.unsupported_switch"));
            }

            // Unknown switches starting with `-`, `--`, or `/` (except Unix paths when not Windows switch)
            if trimmed.starts_with('-') {
                return Self::fail(I18n::tr(lang, "error.invalid_cli_args"));
            }
            if trimmed.starts_with('/') && is_windows {
                return Self::fail(I18n::tr(lang, "error.invalid_cli_args"));
            }

            paths.push(Self::repair_explorer_quoted_drive(trimmed));
        }

        match paths.len() {
            0 => Self {
                command: CliCommand::OpenEmpty,
                error: None,
                is_path_not_found_error: false,
                requested_path: None,
            },
            1 => {
                let target = &paths[0];
                match std::fs::canonicalize(target) {
                    Ok(canon) => Self {
                        command: CliCommand::OpenPath(canon),
                        error: None,
                        is_path_not_found_error: false,
                        requested_path: Some(target.clone()),
                    },
                    Err(_) => Self {
                        command: CliCommand::OpenEmpty,
                        error: Some(format!(
                            "{}: {target}",
                            I18n::tr(lang, "error.path_not_found")
                        )),
                        is_path_not_found_error: true,
                        requested_path: Some(target.clone()),
                    },
                }
            }
            _ => Self::fail(I18n::tr(lang, "error.invalid_cli_args")),
        }
    }

    /// Repairs Windows Explorer trailing `\"` quoting on root drives (e.g., `C:\"` -> `C:\`).
    pub fn repair_explorer_quoted_drive(arg: &str) -> String {
        let bytes = arg.as_bytes();
        if bytes.len() == 4
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && bytes[2] == b'\\'
            && bytes[3] == b'"'
        {
            return arg[..3].to_string();
        }
        arg.to_string()
    }

    fn fail(msg: &str) -> Self {
        Self {
            command: CliCommand::OpenEmpty,
            error: Some(msg.to_string()),
            is_path_not_found_error: false,
            requested_path: None,
        }
    }
}
