# Contract: Zero Trust Security, Path Containment & Redacted Logging

**Feature**: `001-architecture-security-refactoring`
**Covers**: `FR-001`, `FR-002`, `FR-003`, `FR-004`, `FR-006`

---

## 1. Binary Format & Script Rejection (`ImageFormatHelper`)

- MUST stream/inspect up to **8,192 leading bytes** (skipping leading whitespace cleanly, and rejecting files whose first 512+ inspected bytes are exclusively whitespace/control padding).
- MUST check BOM signatures in strict byte-length order (**4-byte BOMs before 2-byte BOMs**):
  - UTF-32 LE BOM (`FF FE 00 00`) and UTF-32 BE BOM (`00 00 FE FF`)
  - UTF-8 BOM (`EF BB BF`) and ASCII/UTF-8
  - UTF-16 LE BOM (`FF FE`) and UTF-16 BE BOM (`FE FF`)
  - BOM-less UTF-16 LE/BE heuristics for ASCII markup prefixes (`<\0`, `\0<`)
- MUST reject payloads matching `<`, `<?xml`, `<svg`, `<html`, `%!`, `%PDF`, `push `, or `viewbox` (case-insensitive after skipping BOM + whitespace) **both** before initial `MagickImageInfo` ping and before any `GetFormatHint` extension fallback.

---

## 2. Arithmetic & Stream Bounds (`ImageRenderService`, `ImageLoaderService`, `AnimationPlayback`, `ZipImageSource`)

- **Pixel/Stride Overflow (CWE-190)**:
  - `ImageRenderService.ToBitmapSource` (preserving default/explicit `maxPixels` parameterization up to configured `[10, 1000] MP`), multi-frame `.ico`/`.cur` (`ReadLargestFrame`), multi-page `TIFF` secondary pages (`pageIndex > 0`), and `AnimationSource.DecodeFirstFrame()` / `AnimationPlayback` (`SKImageInfo`, `Freeze()`) MUST verify `checked((int)(width * 4))` and `(long)width * height <= maxPixels` before allocating pixel arrays, `SKBitmap`, or `BitmapSource`.
- **Stream Size Cap & Cancellation (CWE-409)**:
  - `OpenSeekableAsync(IImageSource, ImageInfo, long maxStreamBytes, CancellationToken)` MUST enforce `maxStreamBytes` (matching configured `MaxZipEntryMegabytes`, default `512 MB`) during chunked copy and propagate a valid `CancellationToken` (including caller cancellation in `ThumbnailService` and cooperative cancellation in `ImageLoaderService`).
  - `TryReadAnimation(Stream, string, bool keepAnimation, long maxAnimationSourceBytes)` MUST reject streams exceeding `maxAnimationSourceBytes` prior to `new byte[...]`.

---

## 3. Path Canonicalization & Symlink/Junction Boundaries

- **Root-Boundary Canonicalization**:
  - `baseFull = Path.GetFullPath(baseDir)` normalized to end with `Path.DirectorySeparatorChar`.
  - `candidateFull = Path.GetFullPath(Path.Combine(baseFull, relativePath))`.
  - MUST reject if `!candidateFull.StartsWith(baseFull, StringComparison.OrdinalIgnoreCase)` (on Windows) or ordinal on case-sensitive hosts.
- **Reparse-Point & Archive Traversal Enforcement**:
  - `FolderImageSource` enumeration skips any entry with `FileAttributes.ReparsePoint` (`LinkTarget is not null`) while permitting explicitly user-opened top-level folders/files.
  - `ZipImageSource` (which reads strictly into memory without disk extraction) rejects any entry key containing `..`, drive roots, leading `/`, null bytes, `LinkTarget != null`, OR POSIX `S_IFLNK` (`((entry.Attrib >> 16) & 0xF000) == 0xA000`) / DOS `ReparsePoint` (`0x0400`) flags.
  - `SettingsService` and `FileLoggerProvider` check `FileAttributes.ReparsePoint` / `LinkTarget != null` **strictly on the immediate app-managed directory (`DataDirectory`/`LogDirectory`) and target file** (without failing on ancestor user-profile junctions).

---

## 4. Configuration Validation (`SettingsService` & `ViewerSettings`)

- `SettingsService.Load()` checks immediate path safety and `new FileInfo(SettingsPath).Length <= 65_536` (`64 KB`) before reading; oversized, reparse-point, or malformed files fall back cleanly to `new ViewerSettings().Normalize()`.
- `ViewerSettings.Normalize()` validates every `double` via `double.IsFinite(...)` before range clamping (replacing `NaN`/`Infinity` with safe defaults, and resetting non-finite `Window.Left`/`Window.Top` to `null`).
- `SettingsService.Save()` writes to a randomized temp file (`Path.GetRandomFileName()`) before replacing `settings.json`, catching storage exceptions and deleting orphaned temp files in `finally`.

---

## 5. Path Redaction & Non-Blocking Logging (`FileLoggerProvider` & UI Error Dialogs)

- `PathRedactor` strips **quoted and space-containing** Windows drive paths (`[A-Za-z]:\\...`), UNC paths (`\\\\...`), and Unix directory paths (`/(?:[^/\s]+/)+` and quoted spans) down to filename-only (`<redacted>\filename` or `filename`) in both log messages and `Exception.Message`, and strips `\r` / `\n` / control characters against CWE-117 log injection.
- `FileLoggerProvider` formats entries via `FormatSanitizedEntry`, queues materialized `SanitizedLogRecord` items into a bounded `Channel` (`BoundedChannelFullMode.DropOldest`, non-blocking `TryWrite`), runs consumer I/O with `.ConfigureAwait(false)`, and implements **both** `IDisposable` and `IAsyncDisposable` (draining synchronously in `Dispose()` via `writer.TryComplete()` + a bounded `2s` timeout wait so WPF shutdown never deadlocks or throws).
- `App.OnDispatcherUnhandledException` displays a localized generic error dialog without raw `e.Exception.Message` host paths.
