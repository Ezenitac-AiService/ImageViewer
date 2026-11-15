# Phase 0 Research: Project-Wide Architecture, Security, and Reliability Refactoring

**Feature**: `001-architecture-security-refactoring`
**Date**: 2026-11-15
**Spec**: [spec.md](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/spec.md)

---

## R1. Zero Trust Binary Content & Multi-Encoding Format Validation

- **Decision**: Harden `ImageFormatHelper.LooksLikeScriptOrDocument` to inspect a leading buffer up to 8,192 bytes (streaming past leading whitespace without holding large allocations), detect 4-byte UTF-32 LE (`FF FE 00 00`) and UTF-32 BE (`00 00 FE FF`) BOMs **before** 2-byte UTF-16 LE (`FF FE`) / UTF-16 BE (`FE FF`) and UTF-8 (`EF BB BF`) BOMs (plus BOM-less UTF-16 LE/BE heuristics `<\0`/`\0<`), reject all-whitespace leading buffers >= 512 bytes, match script/document prefixes (`<`, `<?xml`, `<svg`, `<html`, `%!`, `%PDF`, `push `, `viewbox`) case-insensitively, and apply the same signature check even when `GetFormatHint(fileName)` is consulted on missing-delegate fallback.
- **Rationale**: Code inspection of `src/ImageViewer/Helpers/ImageFormatHelper.cs` (lines 65–93) and `src/ImageViewer/Services/ImageLoaderService.cs` (lines 428–467) showed that header inspection currently checks only a 64-byte slice with UTF-8 BOM skipping, leaving UTF-16/UTF-32 XML/SVG payloads, `>= 64` bytes of leading whitespace (ImageTragick / MVG `push graphic-context` polyglot padding), and uppercase MVG tokens undetected before `MagickImageInfo` / `GetFormatHint` runs. Inspecting UTF-32 BOMs before UTF-16 LE prevents `FF FE 00 00` from being misparsed as UTF-16 LE.
- **Alternatives considered**:
  - *Extension-only allowlisting or fixed 512-byte cap*: Rejected because renamed active documents bypass extension filters and `>= 512` space-padded MVG/SVG headers bypass fixed small slices.
  - *Removing `GetFormatHint` entirely*: Rejected because legitimate binary formats lacking magic headers (such as `.tga` or certain `.ico`/RAW inputs) require format hints after active-document signatures are ruled out.
- **Sources**: OWASP Input Validation & File Upload Cheat Sheets; ImageTragick / MVG polyglot magic-header bypass guidance (CVE-2016-3714 class); direct audit of `ImageFormatHelper.cs` and `ImageLoaderService.cs`.

---

## R2. Arithmetic Overflow (CWE-190) and Stream / Animation Memory Caps (CWE-409)

- **Decision**:
  1. Enforce `checked` 64-bit arithmetic and configurable `maxPixels` validation (`Limits.MaxImageMegapixels`, `[10, 1000]`) across **all** bitmap/pixel-buffer allocation paths: `ImageRenderService.ToBitmapSource` (pass explicit `maxPixels` / settings-aware helper alongside `checked((int)(width * 4))` stride), multi-frame `.ico`/`.cur` (`ReadLargestFrame` checking per-frame dimensions before decoding), multi-page TIFF secondary pages (`pageIndex > 0`), and `AnimationSource.DecodeFirstFrame()` / `AnimationPlayback` (`SKImageInfo` + `Freeze()` `checked((int)(width * 4))` and `maxPixels`).
  2. Add `long maxStreamBytes` and real `CancellationToken` propagation (with cooperative `SharedWork` linked CTS shutdown support instead of `CancellationToken.None`) to `ImageLoaderService.OpenSeekableAsync` and `ImageLoaderService.TryReadAnimation`, enforcing the configured `MaxZipEntryMegabytes` cap (`default 512 MB`) during chunked streaming reads and before `new byte[...]` allocations across both `ImageLoaderService` and `ThumbnailService`.
- **Rationale**: Inspection of `ImageRenderService.cs:130-136`, `ImageLoaderService.cs:319,405-408,535,616-619,662`, `ThumbnailService.cs:122`, and `AnimationPlayback.cs:87,150,399-406` revealed unchecked `(int)width`/`width * 4` math in both `ImageRenderService` and `AnimationPlayback`, secondary-frame/animation gaps in `maxPixels`, uncancelable `CancellationToken.None` reads, and unbounded `MemoryStream` / `new byte[stream.Length]` LOH allocations.
- **Alternatives considered**:
  - *Validating only `ImageRenderService.ToBitmapSource` with a hardcoded 150 MP cap*: Rejected because `AnimationPlayback` / `ReadLargestFrame` would remain vulnerable to CWE-190/oversized frames while hardcoding 150 MP would break user-configured `MaxImageMegapixels` up to 1000 MP.
- **Sources**: OWASP Input Validation Cheat Sheet (CWE-409), SharpCompress Decompression Bomb Mitigation patterns, MITRE CWE-190, direct audit of `ImageRenderService.cs`, `ImageLoaderService.cs`, `ThumbnailService.cs`, and `AnimationPlayback.cs`.

---

## R3. Path Canonicalization, Trailing Separator Boundary, and Symbolic-Link / Junction Policy (CWE-22)

- **Decision**:
  1. Centralize path-containment verification in `PathSecurity`: canonicalize both base and candidate paths via `Path.GetFullPath`, append `Path.DirectorySeparatorChar` to the base directory before OS-appropriate `StartsWith` comparison, normalize archive entry keys (`\` -> `/`, rejecting root-anchored keys, drive letters, `..` traversal segments, and null bytes), and clarify `ZipImageSource` reads entries purely into `MemoryStream` without disk extraction.
  2. Reject symbolic links / junctions on archive entries by inspecting both `IArchiveEntry.Attrib` (POSIX `S_IFLNK` mode `(attrib >> 16) & 0xF000 == 0xA000` and DOS `ReparsePoint` `0x0400`) and `LinkTarget != null` (because SharpCompress `ZipArchive` entries always return `LinkTarget == null` and only expose POSIX/DOS flags in `Attrib`).
  3. On local configuration/log paths (`SettingsService`, `FileLoggerProvider`), check `FileAttributes.ReparsePoint` / `LinkTarget != null` **strictly on the immediate app-managed directory (`DataDirectory`/`LogDirectory`) and target file** (not arbitrary ancestor profile directories such as corporate OneDrive junctions), skip reparse-point file entries during `FolderImageSource` enumeration (`AttributesToSkip` / attribute inspection), and preserve explicitly user-opened top-level paths (`CommandLineHelper`, `OpenFile`/`OpenFolder`/`OnDrop`) per Clarification Q3.
- **Rationale**: SharpCompress `ZipArchive` source/issue inspection confirms `ZipArchiveEntry.LinkTarget` is always `null` while POSIX symlink indicators live in external attributes (`Attrib`). Checking immediate managed paths rather than ancestor folders prevents false-positive lockouts on Windows enterprise profiles while still blocking local config/log symlink redirection.
- **Alternatives considered**:
  - *Checking only `IArchiveEntry.LinkTarget != null`*: Rejected because it silently fails to detect POSIX ZIP symlinks in `ZipArchive`.
  - *Checking ancestor directories above `AppPaths.DataDirectory`*: Rejected to avoid breaking legitimate Windows enterprise folder redirection.
- **Sources**: SharpCompress `Entry.Attrib` & `ZipEntry.LinkTarget` behavior, .NET `EnumerationOptions.AttributesToSkip` & NTFS Reparse Point docs, SonarSource / OWASP .NET CWE-22 `Path.GetFullPath` trailing-separator guidance.

---

## R4. Fail-Closed CLI Parsing and Windows Shell Context-Menu Hardening (CWE-88)

- **Decision**:
  1. Update `CommandLineHelper.Parse` to preserve positional backward compatibility on `CommandLineOptions(..., bool IsPathNotFoundError = false)`, accept at most one positional target path OR a single valid shell switch (`--register-shell`, `/register-shell`, `--unregister-shell`, `/unregister-shell`), fail closed on duplicate or conflicting flags, unknown `-`/`/` flags (while accepting Windows `<letter>:\...` drive paths, `\\...` UNC paths, and `.\...` relative paths), extra positional paths, invalid characters, or path+switch combinations (Clarification Q2).
  2. Harden `ShellIntegrationService.IsRegistered()` to validate exact normalized quoted command equality (`string.Equals(value?.Trim(), expectedCommand, StringComparison.OrdinalIgnoreCase)`) rather than `value.Contains(ExecutablePath, ...)`, and reject `ExecutablePath` containing embedded `"` or control characters before registry writes.
  3. Because `App.RunShellCommand` executes and exits before `ConfigureServices()` builds DI logging, instantiate and cleanly flush/dispose a scoped `PathRedactor`-enabled `FileLoggerProvider` inside `RunShellCommand` to log shell registration failures cleanly before returning exit code `1`.
- **Rationale**: Direct audit of `CommandLineHelper.cs` (lines 17–38), `ShellIntegrationService.cs` (lines 71–85), and `App.xaml.cs` (lines 26–34) revealed silent argument dropping, substring spoofing in `IsRegistered()`, and `_logger == null` during early shell command exit.
- **Alternatives considered**:
  - *Initializing the entire WPF DI container just for CLI `--register-shell`*: Rejected to keep silent installer shell registration lightweight without touching WPF services/Magick.NET.
- **Sources**: SpecterOps / MITRE CWE-88 Windows Explorer shell verb quoting research; direct audit of `CommandLineHelper.cs`, `ShellIntegrationService.cs`, and `App.xaml.cs`.

---

## R5. Configuration Validation (`64 KB` Cap, Finite Bounds) and Sanitized Non-Blocking Logging

- **Decision**:
  1. In `SettingsService.Load()`, verify immediate path safety (`PathSecurity`) and file size `<= 65,536` bytes (`64 KB`, Clarification Q4) before reading; in `ViewerSettings.Normalize()`, guard all `double` fields (`Window.Width`, `Window.Height`, nullable `Window.Left`/`Top`, `Filter.SmoothSigma`, `Filter.SharpenSigma`, `Filter.SharpenAmount`) with `double.IsFinite(...)` before clamping/nulling. In `SettingsService.Save()`, write to a randomized temp file (`Path.GetRandomFileName()`), catch `IOException`/`UnauthorizedAccessException`/`NotSupportedException`/`ArgumentException`, and delete orphaned temp files in `finally`.
  2. Extract shared `PathRedactor` + `FormatSanitizedEntry` log-sanitization helpers used by `FileLoggerProvider`: strip **quoted (`'...'`/`"..."`) and space-containing Windows (`[A-Za-z]:\...`), UNC (`\\...`), and Unix paths** down to `<redacted>\<filename>` or filename-only, strip `\r`/`\n`/control characters against log injection (CWE-117), and sanitize `App.OnDispatcherUnhandledException` UI dialogs (echoing user paths only when CLI `IsPathNotFoundError` is true per Clarification Q5).
  3. Refactor `FileLoggerProvider` to use a bounded `Channel<SanitizedLogRecord>` (`BoundedChannelFullMode.DropOldest`, non-blocking `TryWrite`), implement **both** `IDisposable` and `IAsyncDisposable` (so `ServiceProvider.Dispose()` on WPF exit never throws `InvalidOperationException`), run `ProcessQueueAsync` exclusively with `.ConfigureAwait(false)`, and drain in `Dispose()` via `writer.TryComplete()` + a bounded timeout (`TimeSpan.FromSeconds(2)`) so stalled disks cannot hang UI shutdown.
- **Rationale**: Microsoft Learn DI & `Channel<T>` shutdown research confirms `ServiceProvider.Dispose()` throws if an `ILoggerProvider` implements only `IAsyncDisposable`, and blocking `.Wait()` without `.ConfigureAwait(false)` and timeout deadlocks WPF `DispatcherSynchronizationContext`. Space-aware redaction fixes partial leakage in standard Windows messages (`C:\Users\My User\...`).
- **Alternatives considered**:
  - *Regex stopping at first whitespace (`[^\s]+`)*: Rejected because standard Windows paths contain spaces (`Users\First Last\...`, `Program Files\...`).
- **Sources**: Microsoft Learn custom `ILoggerProvider` / `Channel<T>` & `ServiceProvider.Dispose()` async disposal guidance; CWE-117 log injection mitigation; direct audit of `ViewerSettings.cs`, `SettingsService.cs`, `FileLoggerProvider.cs`, and `App.xaml.cs`.

---

## R6. Concurrency Coordination, Native Resource Limits, and Deterministic Disposal

- **Decision**:
  1. In `App.xaml.cs`, set `OMP_NUM_THREADS` (`Math.Clamp(Environment.ProcessorCount / 2, 1, 4)`) and `MAGICK_THREAD_LIMIT` **at the very start of `OnStartup` before `ConfigureServices()` or any `ImageMagick` type initializes**, then call `MagickNET.Initialize()`, configure `ResourceLimits.Thread` (`Math.Clamp(Environment.ProcessorCount / 2, 1, 4)` bounded for multi-worker concurrency), and enforce explicit `ResourceLimits.Memory` (`1024UL * 1024 * 1024` / `1024 MB`) alongside `Width`/`Height`/`ListLength`.
  2. Replace `.GetAwaiter().GetResult()` in `ThumbnailService.Create` with async worker execution (`await OpenSeekableAsync(...)`). On `ZipImageSource`, make `Dispose()` thread-safe and non-throwing against concurrent `OpenImageAsync` `finally { _gate.Release(); }` calls by canceling/marking `_disposed` cleanly, implementing both `IDisposable` and `IAsyncDisposable` (disposing `_gate` only once active readers finish or skipping unmanaged WaitHandle allocation safely), and propagating cancellation tokens across `ImageSessionCoordinator` -> `IImageSource`.
  3. Fix `AnimationSource` stream lifetime in `AnimationPlayback.cs`: allocate `using var stream = new MemoryStream(data, writable: false);` under caller `using` control and pass `new SKManagedStream(stream, disposeManagedStream: false)` into `SKCodec.Create(...)` **without** wrapping `SKManagedStream` in its own `using` block, because `SKCodec.Create` takes C++ native ownership of `SKManagedStream` and deletes it on codec disposal or `null` return.
  4. Implement `IDisposable` on `ImageRenderService`, dispose `_backgroundGate` in `ImageLoaderService.Dispose()`, stop/dispose all timers in `MainViewModel.Dispose()`, detach `MainWindow` handlers cleanly in `OnClosed` (while preserving non-canceled close behavior), and wrap `MainWindow.OnDrop` (`async void`) in `try/catch` with sanitized error reporting.
- **Rationale**: Magick.NET docs confirm OpenMP environment variables must precede native/JIT initialization of `ImageMagick` types and `ResourceLimits.Thread` controls runtime thread fan-out. SkiaSharp docs warn that `SKCodec.Create` owns and frees `SKManagedStream`; wrapping `SKManagedStream` in a separate `using` causes double-free corruption.
- **Alternatives considered**:
  - *Wrapping `SKManagedStream` in `using` inside `AnimationSource`*: Explicitly rejected due to double-disposal bug verified in SkiaSharp docs.
- **Sources**: Microsoft Learn WPF Memory Leaks & `BitmapSource.Freeze()`; Magick.NET `ResourceLimits.Thread` & OpenMP initialization docs; SkiaSharp `SKCodec.Create`/`SKManagedStream` ownership docs; direct audit of `App.xaml.cs`, `ImageRenderService.cs`, `ImageLoaderService.cs`, `ZipImageSource.cs`, `ThumbnailService.cs`, `AnimationPlayback.cs`, `MainWindow.xaml.cs`, and `MainViewModel.cs`.

---

## R7. Modular Decomposition and Headless Classical TDD Strategy

- **Decision**:
  1. Decompose monolithic logic coherently using one unified module set across `plan.md`, `contracts/`, and `tasks.md` while keeping every `MainWindow.xaml` binding AND `MainWindow.xaml.cs` `_vm.*` surface (`FirstCommand`, `LastCommand`, `UpdateViewport`, `MouseWheelAction`, `GoToAsync`, `OpenPathAsync`, `SaveSettings`, plus `PlaceholderTests`-referenced `MainViewModel.PlaceholderDisplaySize` / `PlaceholderDelay` compatibility members) intact:
     - `PathSecurity` & `PathRedactor` (`Helpers/PathSecurity.cs`)
     - `ViewportLayoutCalculator` (`Services/ViewportLayoutCalculator.cs`: pan/zoom/anchor/placeholder display geometry)
     - `WindowChromeState` (`ViewModels/WindowChromeState.cs`: pin/hover/fullscreen restore state)
     - `ImageSessionCoordinator` (`Services/ImageSessionCoordinator.cs`: versioned open/load/render lifecycle, `TimeProvider` placeholder delay timer, `DecodedImage` ownership, `IUiDispatcher`)
     - `PrefetchCoordinator` (`Services/PrefetchCoordinator.cs`: directional `+1`/`+2`/`-1`/burst lookahead and slideshow peek)
     - Cohesive helper extraction inside/from `ImageLoaderService.cs` (`ImageFormatHelper` binary validator + private/internal `ImageAnimationHelper` / metadata cleanly factored without conflicting symbol names)
  2. Introduce `IUiDispatcher` (`WpfUiDispatcher` in `Views/`, inline `TestUiDispatcher` in `ImageViewer.Tests`) and inject `System.TimeProvider` into `SlideShowService`, `ImageSessionCoordinator`, and all `MainViewModel` timers (`_renderDebounce`, `_osdTimer`, `_loadingIndicatorTimer`, `_placeholderTimer`, `_animationTimer`, `_watchDebounce`). Use `Microsoft.Extensions.TimeProvider.Testing` (`FakeTimeProvider`) if available or a contract-faithful `ManualTimeProvider` implementing full `ITimer.Change`/periodic semantics cleanly for offline testability.
- **Rationale**: Reconciling module names eliminates `ViewportPanZoomCalculator` / `ThumbnailCoordinator` drift, ensures `PlaceholderTests.cs` compiles uninterrupted, and ensures zero `DispatcherTimer` UI-thread lock-in remains inside ViewModels/services.
- **Alternatives considered**:
  - *Keeping 6 `DispatcherTimer` instances inside `MainViewModel`*: Rejected because headless `xUnit` suites cannot deterministically test placeholder timing, OSD expiry, folder-watch debounce, or slideshow ticks without live WPF dispatchers.
- **Sources**: Microsoft Learn `TimeProvider` / `FakeTimeProvider` WPF MVVM guidance; Martin Fowler *Mocks Aren't Stubs*; direct audit of `MainViewModel.cs`, `MainViewModel.Features.cs`, `MainWindow.xaml`, `MainWindow.xaml.cs`, `PlaceholderTests.cs`, and `SlideShowService.cs`.
