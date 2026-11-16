//! Path containment, archive key normalization, symlink guards, and log/error path redaction.

use std::path::{Component, Path, PathBuf};
use thiserror::Error;

/// Errors raised by Zero Trust path and archive key checks.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PathSecurityError {
    #[error("path escapes containment root")]
    OutsideRoot,
    #[error("unsafe archive entry key")]
    UnsafeArchiveKey,
    #[error("symlink or reparse point rejected")]
    SymlinkRejected,
    #[error("invalid filesystem path")]
    InvalidPath,
}

/// Centralized path containment and archive key validator.
pub struct PathSecurity;

impl PathSecurity {
    /// Verifies `candidate` resolves inside `root` using trailing-separator containment.
    pub fn is_within_root(root: &Path, candidate: &Path) -> Result<bool, PathSecurityError> {
        let canon_root = std::fs::canonicalize(root).map_err(|_| PathSecurityError::InvalidPath)?;
        let canon_cand =
            std::fs::canonicalize(candidate).map_err(|_| PathSecurityError::InvalidPath)?;

        let mut root_str = canon_root.to_string_lossy().to_string();
        let sep = std::path::MAIN_SEPARATOR;
        if !root_str.ends_with(sep) {
            root_str.push(sep);
        }
        let cand_str = canon_cand.to_string_lossy().to_string();

        #[cfg(target_os = "windows")]
        {
            Ok(cand_str
                .to_ascii_lowercase()
                .starts_with(&root_str.to_ascii_lowercase())
                || canon_cand == canon_root)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(cand_str.starts_with(&root_str) || canon_cand == canon_root)
        }
    }

    /// Checks `symlink_metadata` strictly on the immediate path (never rejecting ancestor enterprise junctions).
    pub fn ensure_not_symlink(path: &Path) -> Result<(), PathSecurityError> {
        match std::fs::symlink_metadata(path) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    Err(PathSecurityError::SymlinkRejected)
                } else {
                    Ok(())
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(PathSecurityError::InvalidPath),
        }
    }

    /// Normalizes `\` -> `/` and rejects `..`, leading `/`, drive prefixes, and NUL bytes.
    pub fn normalize_archive_entry_key(raw: &str) -> Result<String, PathSecurityError> {
        if raw.is_empty() || raw.contains('\0') {
            return Err(PathSecurityError::UnsafeArchiveKey);
        }
        let normalized = raw.replace('\\', "/");
        if normalized.starts_with('/') {
            return Err(PathSecurityError::UnsafeArchiveKey);
        }
        let bytes = normalized.as_bytes();
        if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
            return Err(PathSecurityError::UnsafeArchiveKey);
        }

        let mut clean_parts = Vec::new();
        for part in normalized.split('/') {
            if part.is_empty() || part == "." {
                continue;
            }
            if part == ".." {
                return Err(PathSecurityError::UnsafeArchiveKey);
            }
            clean_parts.push(part);
        }
        if clean_parts.is_empty() {
            return Err(PathSecurityError::UnsafeArchiveKey);
        }
        Ok(clean_parts.join("/"))
    }

    /// Detects POSIX `S_IFLNK` (`0xA000`) or DOS reparse point (`0x0400`) external attributes.
    pub fn is_archive_symlink(external_attributes: u32, is_dos_attributes: bool) -> bool {
        let posix_mode = (external_attributes >> 16) & 0xF000;
        if posix_mode == 0xA000 {
            return true;
        }
        if is_dos_attributes && (external_attributes & 0x0400) != 0 {
            return true;
        }
        false
    }

    /// Validates a single-segment filename (for `.part` update downloads or safe temp files).
    pub fn is_safe_single_filename(name: &str) -> bool {
        if name.is_empty() || name.contains('\0') || name.contains('/') || name.contains('\\') {
            return false;
        }
        let p = Path::new(name);
        let mut comps = p.components();
        matches!(comps.next(), Some(Component::Normal(_))) && comps.next().is_none()
    }

    /// Joins `filename` onto `dir` only after validating it is a safe single filename.
    pub fn safe_join_filename(dir: &Path, filename: &str) -> Result<PathBuf, PathSecurityError> {
        if !Self::is_safe_single_filename(filename) {
            return Err(PathSecurityError::UnsafeArchiveKey);
        }
        Ok(dir.join(filename))
    }
}

/// Redacts host directory prefixes down to filename-only and strips control characters (CWE-117).
pub struct PathRedactor;

impl PathRedactor {
    /// Strips quoted/space-containing Windows, UNC, and Unix directory paths down to filename-only,
    /// and removes `\r`, `\n`, and ASCII control characters.
    pub fn redact(input: &str) -> String {
        let mut sanitized = String::with_capacity(input.len());
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            if ch == '\r' || ch == '\n' || (ch.is_control() && ch != '\t') {
                sanitized.push(' ');
                i += 1;
                continue;
            }

            // Quoted path: "..."
            if ch == '"'
                && let Some(end_rel) = chars[i + 1..].iter().position(|&c| c == '"')
            {
                let inner: String = chars[i + 1..i + 1 + end_rel].iter().collect();
                if Self::looks_like_host_path(&inner) {
                    sanitized.push('"');
                    sanitized.push_str(Self::filename_only(&inner));
                    sanitized.push('"');
                    i += end_rel + 2;
                    continue;
                }
            }

            // UNC path: \\server\share\...
            if ch == '\\' && i + 1 < chars.len() && chars[i + 1] == '\\' {
                let end = Self::consume_path_span(&chars, i);
                let span: String = chars[i..end].iter().collect();
                sanitized.push_str(Self::filename_only(&span));
                i = end;
                continue;
            }

            // Windows drive path: C:\... or C:/...
            if ch.is_ascii_alphabetic()
                && i + 2 < chars.len()
                && chars[i + 1] == ':'
                && (chars[i + 2] == '\\' || chars[i + 2] == '/')
            {
                let end = Self::consume_path_span(&chars, i);
                let span: String = chars[i..end].iter().collect();
                sanitized.push_str(Self::filename_only(&span));
                i = end;
                continue;
            }

            // Unix absolute path: /Users/... /home/... /var/... /tmp/... /opt/... /etc/...
            if ch == '/' && i + 1 < chars.len() && chars[i + 1].is_ascii_alphanumeric() {
                let is_boundary = i == 0 || chars[i - 1].is_whitespace() || chars[i - 1] == '"';
                if is_boundary {
                    let end = Self::consume_path_span(&chars, i);
                    let span: String = chars[i..end].iter().collect();
                    if span.matches('/').count() >= 2 {
                        sanitized.push_str(Self::filename_only(&span));
                        i = end;
                        continue;
                    }
                }
            }

            sanitized.push(ch);
            i += 1;
        }

        sanitized
    }

    fn looks_like_host_path(s: &str) -> bool {
        s.starts_with("\\\\")
            || s.starts_with('/')
            || (s.len() >= 3
                && s.as_bytes()[0].is_ascii_alphabetic()
                && s.as_bytes()[1] == b':'
                && (s.as_bytes()[2] == b'\\' || s.as_bytes()[2] == b'/'))
    }

    fn consume_path_span(chars: &[char], start: usize) -> usize {
        let mut i = start;
        while i < chars.len() {
            let c = chars[i];
            if c == '"' || c == '\'' || c.is_control() {
                break;
            }
            // Allow single spaces inside unquoted path segments only when followed by another path segment
            if c == ' ' {
                let rest: String = chars[i + 1..].iter().take(32).collect();
                let next_ws = rest.find(char::is_whitespace).unwrap_or(rest.len());
                let token = &rest[..next_ws];
                let has_sep_or_ext = token.contains('/')
                    || token.contains('\\')
                    || (token.contains('.')
                        && !token.ends_with('.')
                        && token.split('.').next_back().is_some_and(|e| e.len() <= 5));
                if !has_sep_or_ext {
                    break;
                }
            }
            i += 1;
        }
        i
    }

    fn filename_only(path: &str) -> &str {
        path.rsplit(['/', '\\'])
            .find(|seg| !seg.is_empty())
            .unwrap_or("<redacted>")
    }
}
