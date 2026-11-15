# Data Model & Validation Invariants

**Feature**: `001-architecture-security-refactoring`
**Date**: 2026-11-15
**Spec**: [spec.md](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/spec.md)

---

## 1. CommandLineOptions (Untrusted CLI Input Model)

- **Fields**:
  - `Path: string?` — Canonicalized single target file or folder path; `null` when shell registration switches are used or validation fails.
  - `RegisterShell: bool` — True only when `--register-shell` or `/register-shell` is specified without conflicting switches or paths.
  - `UnregisterShell: bool` — True only when `--unregister-shell` or `/unregister-shell` is specified without conflicting switches or paths.
  - `Error: string?` — Localized Korean user-facing error message; non-null causes startup to fail closed without opening paths or mutating registry keys.
  - `IsPathNotFoundError: bool = false` — Optional defaulted flag (`false` preserves 4-argument positional constructor compatibility) indicating an explicit user-supplied target path did not exist (only case permitted to echo the user-supplied path in UI error prompt).
- **Validation Rules**:
  - Accept at most 1 positional path OR 1 shell switch (`RegisterShell` XOR `UnregisterShell`).
  - Reject unknown or duplicate flags (`-`, `--`, `/` prefixes other than a single valid shell switch `--register-shell`/`/register-shell`/`--unregister-shell`/`/unregister-shell`), extra positional arguments, embedded invalid path characters, and combined path + shell-switch invocations; accept Windows drive (`<letter>:\...`), UNC (`\\...`), and relative (`.\...`) target paths.
  - Repair broken Windows Explorer drive-root quoting (`C:\"` -> `C:\`) before canonicalization.

---

## 2. PathValidationResult & SafeArchiveEntry (Trust-Boundary Path Model)

- **Fields**:
  - `CanonicalPath: string` — Normalized full path via `Path.GetFullPath`.
  - `IsReparsePoint: bool` — Evaluated via `FileAttributes.ReparsePoint` / `LinkTarget != null` (or archive `Attrib` flags on `ZipArchive` entries: POSIX `((attrib >> 16) & 0xF000) == 0xA000` or DOS `0x0400`).
  - `NormalizedEntryKey: string` — Forward-slash normalized archive entry path (`\` -> `/`) with `..`, drive roots, leading `/`, and null-byte segments rejected (`ZipImageSource` reads entries purely into memory and never extracts to disk).
- **Validation Rules**:
  - **Root Containment**: Canonical root directory MUST end with `Path.DirectorySeparatorChar` before OS-appropriate `StartsWith(root, Comparison)` checks to prevent sibling-prefix bypasses (`logs-evil` matching `logs`).
  - **Symlink / Junction Matrix**:
    - Explicitly user-opened top-level path: `IsReparsePoint` permitted.
    - Local folder file enumeration (`FolderImageSource`): entries with `ReparsePoint` skipped.
    - Archive entries (`ZipImageSource`): POSIX `0xA000` / DOS `0x0400` / `LinkTarget != null` / traversal segments strictly rejected.
    - App-managed config/log paths (`SettingsService`, `FileLoggerProvider`): `ReparsePoint` checked strictly on the immediate managed directory (`DataDirectory`/`LogDirectory`) and target file (not ancestor profile junctions).

---

## 3. FormatValidationResult & BoundedStreamGuard (Media Payload Model)

- **Fields**:
  - `HeaderBytes: ReadOnlySpan<byte>` — Streaming inspection up to 8,192 leading bytes (skipping BOMs and leading whitespace; rejecting pure-whitespace padded payloads >= 512 bytes).
  - `IsBlockedSignature: bool` — True for UTF-32 LE (`FF FE 00 00`) / UTF-32 BE (`00 00 FE FF`) checked **before** UTF-16 LE (`FF FE`) / UTF-16 BE (`FE FF`) / UTF-8 (`EF BB BF`), plus BOM-less UTF-16 LE/BE heuristics (`<\0`, `\0<`), matching `<`, `<?xml`, `<svg`, `<html`, `%!`, `%PDF`, `push `, or `viewbox` case-insensitively.
  - `MaxUncompressedBytes: long` — Configured ceiling (`Limits.MaxZipEntryMegabytes * 1024 * 1024`, default `512 MB`, configurable `[10, 4096]`).
  - `MaxPixels: long` — Configured ceiling (`Limits.MaxImageMegapixels * 1_000_000L`, default `150 MP`, configurable `[10, 1000]`).
- **Validation Rules**:
  - `stride = checked((int)(width * 4))` and `pixelCount = (long)width * height <= MaxPixels` MUST be verified across `ImageRenderService.ToBitmapSource`, multi-frame `.ico`/`.cur` (`ReadLargestFrame`), multi-page `TIFF` secondary pages (`pageIndex > 0`), and `AnimationSource.DecodeFirstFrame()` / `AnimationPlayback` before pixel allocation.
  - Non-seekable stream buffering (`OpenSeekableAsync`) and animation byte loading (`TryReadAnimation`) MUST accept `(long maxStreamBytes, CancellationToken cancellationToken)` and enforce `MaxUncompressedBytes` while reading, never allocating unbounded `new byte[stream.Length]` or `MemoryStream` buffers above the cap.

---

## 4. ViewerSettings (Validated Configuration Model)

- **Fields**:
  - `Version: int` (`CurrentVersion = 3`)
  - `Window`: `Width` (`[320, 20000]`, default `1280`), `Height` (`[240, 20000]`, default `800`), `Left: double?`, `Top: double?`, `IsMaximized: bool`
  - `Display`: `Mode`, `TopBarPinned`, `BottomBarPinned`, `InfoPanelVisible`, `PlayAnimations`, `Background`, `Filter`, `MouseWheel`
  - `Navigation`: `WrapAround`, `WatchFolder`
  - `Slideshow`: `IntervalSeconds` (`[5, 90]`), `Mode`, `Repeat`
  - `Thumbnail`: `Visible`, `Size` (`[48, 256]`, default `96`), `ShowScrollBar`
  - `Filter`: `SmoothSigma` (`[0.1, 3.0]`, default `0.5`), `SharpenSigma` (`[0.1, 5.0]`), `SharpenAmount` (`[0.1, 5.0]`)
  - `Limits`: `MaxImageMegapixels` (`[10, 1000]`, default `150`), `MaxZipEntryMegabytes` (`[10, 4096]`, default `512`)
- **Validation & Recovery Rules**:
  - File size MUST be `<= 65,536` bytes (`64 KB`) and non-reparse-point before deserialization.
  - Every `double` property MUST pass `double.IsFinite(value)` prior to clamping; non-finite `Width`, `Height`, or filter sigmas fall back to defaults, and non-finite `Left`/`Top` reset to `null` (center-screen startup).
  - Atomic save writes to a randomized temp file in the validated directory and replaces `settings.json` only on success.

---

## 5. SanitizedLogRecord (Privacy-Safe Diagnostic Model)

- **Fields**:
  - `Timestamp: DateTimeOffset`
  - `Level: LogLevel`
  - `Category: string`
  - `Message: string` (path-redacted via `PathRedactor`, CR/LF `\r`/`\n` replaced)
  - `ExceptionType: string?`
  - `SanitizedExceptionMessage: string?` (directory/UNC/Unix prefixes including space-containing quoted paths stripped down to `<redacted>\<filename>` or filename-only; CR/LF replaced)
- **State Transitions**:
  1. `ILogger.Log` materializes formatted message + redacted exception synchronously without touching disk (`FormatSanitizedEntry`).
  2. Bounded `Channel<SanitizedLogRecord>` accepts record non-blockingly (`TryWrite`, `BoundedChannelFullMode.DropOldest`).
  3. Background consumer task executes `.ConfigureAwait(false)`, appends formatted lines asynchronously, implements both `IDisposable` and `IAsyncDisposable` (draining via `writer.TryComplete()` + bounded `2s` timeout wait inside synchronous `Dispose()` to avoid WPF shutdown deadlocks).

---

## 6. ImageSessionState & Coordinator Ownership (Lifecycle State Machine)

- **Entities & Relationships**:
  - `ImageSessionCoordinator` owns active `IImageSource?`, `_images`, current index/page, `DecodedImage?` reference-counted handle, and monotonic `_openVersion` / `_loadVersion` / `_renderVersion` tokens + linked `CancellationTokenSource` instances.
  - `PrefetchCoordinator` schedules directional lookahead (`+1`, `+2`, `-1`, burst up to `+4`, and slideshow peek) using background tokens detached from foreground `PriorityGate` starvation.
  - `ViewportLayoutCalculator` (pure math) computes image display bounds, zoom anchor offsets, clamped pan offsets, and placeholder display dimensions deterministically without `Window` dependencies.
  - `WindowChromeState` tracks pin/hover/fullscreen restore invariants decoupled from WPF visual tree traversal.
- **Lifecycle State Transitions**:
  - `Idle` -> `Opening(openVersion)` -> `Loaded(index, loadVersion)` -> `Rendering(renderVersion)` -> `Disposed`.
  - Any newer `openVersion` / `loadVersion` / `renderVersion` immediately cancels superseded `CancellationTokenSource` instances (including linked tokens passed to `OpenSeekableAsync` so in-flight stream reads abort promptly) and discards stale async continuations before mutating displayed state.
  - `Dispose()` deterministically stops all 6 `ITimer` instances (`TimeProvider`: render debounce, OSD, loading indicator, placeholder delay, animation frame, folder-watch debounce), detaches `FolderImageSource.Changed`, cancels in-flight CTS tokens, disposes `SlideShowService`, `ImageLoaderService`, `ImageRenderService`, `IImageSource` (safely guarding concurrent `_gate.Release()` callers on `ZipImageSource`), and disposes/drains `FileLoggerProvider`.
