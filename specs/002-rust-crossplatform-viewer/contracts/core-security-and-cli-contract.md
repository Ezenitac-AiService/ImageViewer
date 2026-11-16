# Contract: Core Zero Trust Security, CLI, Settings & Redacted Logging

**Feature**: `002-rust-crossplatform-viewer`
**Covers**: `FR-005`, `FR-006`, `FR-007`

---

## 1. CLI Parsing (`CliOptions::parse` in `iv-core`)

- **Accepted Modes** (mutually exclusive):
  1. `0` positional arguments -> launches viewer empty (`path = None`).
  2. `1` positional target path (after filtering benign macOS Finder `-psn_*` session flags and repairing Explorer `C:\"` quoting): canonicalized via filesystem resolution. If missing on disk -> `error = Some(localized_not_found)`, `is_path_not_found_error = true` (sole error dialog permitted to echo user-entered target path).
  3. **Windows-only Shell Verbs**: `--register-shell` | `/register-shell` OR `--unregister-shell` | `/unregister-shell` with zero positional paths.
- **Fail-Closed Rejections**:
  - `>= 2` positional arguments.
  - Unknown or duplicated flags starting with `-`, `--`, or `/` (except Windows `<letter>:\...` / UNC `\\...` paths and macOS `-psn_*`).
  - Simultaneous register + unregister flags, or switch + path combined.
  - `--register-shell` / `--unregister-shell` invoked on non-Windows OSes -> fail closed with localized unsupported-OS error message and non-zero exit code.

---

## 2. Binary Format & Archive Security Guards (`iv-core` & `iv-media`)

- **Leading Binary Sniffer**: Inspects up to `8,192` bytes across stream start **before both core decoders and OS platform codecs (`WIC` / `ImageIO` / Linux)**:
  - Check BOM order strictly: 4-byte UTF-32 LE (`FF FE 00 00`) & BE (`00 00 FE FF`) **before** 2-byte UTF-16 LE (`FF FE`) & BE (`FE FF`) and UTF-8 (`EF BB BF`).
  - Handle BOM-less UTF-16 interleaved ASCII (`<\0`, `\0<`).
  - Reject files whose first `>= 512` inspected bytes are solely whitespace/control padding.
  - Match blocked active/document prefixes case-insensitively after BOM + whitespace: `<`, `<?xml`, `<svg`, `<html`, `%!`, `%PDF`, `push `, `viewbox`.
- **Arithmetic & Stream Caps**:
  - Verify `width.checked_mul(4)` (byte stride) and `(width as u64) * (height as u64) <= max_pixels` (`[10..1000] MP`, default `150 MP`) prior to every buffer allocation across static decodes, `.ico`/`.cur` largest-frame selection, TIFF pages, animated frames, and OS platform codecs.
  - `ZipSource` runs a **zero-allocation pre-flight EOCD / ZIP64 EOCD scan strictly BEFORE invoking `zip::ZipArchive::new`**, enforcing `<= 50,000` entries, `<= 64 MB` central directory metadata, and `<= 4 GB` container size (`CVE-2026-0429` mitigation), rejects traversal keys (`..`, leading `/`, drive root, `\0`), POSIX `S_IFLNK` (`0xA000`) / DOS `0x0400` symlinks, and bounds entry decompression streams to `max_zip_entry_megabytes * 1024 * 1024` (default `512 MB`) dynamically via bounded reader (`take(cap + 1)`). Encrypted entries return localized unsupported-encryption message without crashing.
- **Local Path Containment & Symlink Matrix**:
  - Canonical root containment appends trailing OS path separator before prefix comparison.
  - `FolderSource` & `WatchFolder` skip symlink/reparse entries via `symlink_metadata`, while allowing explicitly user-opened top-level paths.
  - Config/log/temp directories & files check `symlink_metadata` strictly on the immediate app-managed directory and target file (never failing on enterprise ancestor profile junctions).

---

## 3. Backward-Compatible `settings.json` & Multi-Instance Safety

- Deserialize/serialize `ViewerSettings` using `camelCase` keys and string enum values compatible with `v1.0.6` (`DisplayMode::Fill`, `ViewerBackground::Black`, `SlideShowMode::Sequential`, `MouseWheelAction::Navigate`, etc.), accepting JSON comments and trailing commas.
- Reject files `> 65,536` bytes (`64 KB`) or symlinked files -> return `ViewerSettings::default().normalize()`.
- Replace non-finite `f64` (`NaN`, `±Infinity`) before clamping; sanitize `update.skippedVersion` (`<= 32` chars, valid version token, no whitespace/control chars).
- Acquire advisory file lock (`fs4`), **re-read the latest disk `settings.json`**, and selectively merge dirty sections (Settings-dialog OK / `SkipVersion` persist preferences immediately; window close updates only `window` fields) via randomized temp file + atomic rename so concurrent multi-instance closes never corrupt or overwrite each other's settings (`Lost Update` prevention).

---

## 4. Path Redaction & Non-Blocking Diagnostics

- `PathRedactor` removes quoted and space-containing Windows (`[A-Za-z]:\...`), UNC (`\\...`), and Unix (`/Users/...`, `/home/...`) directory prefixes down to filename-only (`<redacted>/filename` or filename), and strips `\r`, `\n`, and control characters against log injection (CWE-117).
- `AsyncFileLogger` sends materialized `SanitizedLogRecord` structs over a bounded channel (`DropOldest` on full), maintains 7 daily log files, records slow-render stages when total frame presentation `>= 300 ms`, and implements synchronous bounded (`<= 2s`) flush on shutdown **after** any `ExitAndRun` update-launch log write completes.
- UI error dialogs omit raw host directory paths and raw system exception strings (except `cli.is_path_not_found_error == true`).

---

## 5. Secure Auto-Updater Transport & Storage

- Require `Uri` scheme `https://` unconditionally across `FeedBase`, `ReleaseInfo.page_url`, `SHA256SUMS.txt`, asset `download_url`, and external browser opens (`OpenReleasePage`), with **zero** `#[cfg(debug_assertions)]`, `danger_accept_invalid_certs`, or environment-variable security bypasses. Offline tests inject an `HttpFeedTransport` implementation or caller-provided `rustls::RootCertStore` (paired with an `rcgen` loopback `https://` server) without relaxing TLS validation in production.
- Cap HTTP stream sizes dynamically (`<= 1 MB` release JSON, `<= 64 KB` `SHA256SUMS.txt`, `<= 256 MB` binary package).
- Verify exact lowercase 64-hex SHA-256 on `.part` temp download (validated via `PathSecurity` against symlinks/invalid asset filenames) before activating.
- Windows `/RELAUNCH=` rejects `"`, `" /"`, and control characters (`\r`, `\n`, `\0`). All update logs & UI errors pass through `PathRedactor`.
