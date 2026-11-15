---

description: "Actionable, dependency-ordered TDD task list for Project-Wide Architecture, Security, and Reliability Refactoring"
---

# Tasks: Project-Wide Architecture, Security, and Reliability Refactoring

**Input**: Design documents from `/specs/001-architecture-security-refactoring/`
**Prerequisites**: [plan.md](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/plan.md) (required), [spec.md](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/spec.md) (required), [research.md](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/research.md), [data-model.md](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/data-model.md), [contracts/](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/contracts), [quickstart.md](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/quickstart.md)

**Tests**: REQUIRED by [constitution.md](file:///Users/dev/DEV/ImageViewer/.specify/memory/constitution.md) Principle III (Classical Red-Green-Refactor TDD in `xUnit`, zero mocking frameworks, zero `[Fact(Skip = ...)]`, zero test-only production switches). Failing tests MUST be written and observed failing before implementing the corresponding production code in every phase.

**Organization**: Tasks are grouped by priority-ordered user stories (`US1 P1 -> US2 P2 -> US3 P3`) so each increment can be implemented, tested, and verified independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: User story label (`[US1]`, `[US2]`, `[US3]`) required for user story phases only
- Every task includes exact repository file paths and verbatim constraints from `data-model.md` and `contracts/`

---

## Phase 1: Setup (Shared Infrastructure & Test Fixtures)

**Purpose**: Establish shared Classical-TDD deterministic fixtures and baseline verification structure without introducing mocking libraries.

- [ ] T001 Audit baseline solution structure and build/test configuration across `ImageViewer.sln`, `src/ImageViewer/ImageViewer.csproj`, `tests/ImageViewer.Tests/ImageViewer.Tests.csproj`, and `tests/ImageViewer.IntegrationTests/ImageViewer.IntegrationTests.csproj` (preserving `net10.0-windows`, `PlatformTarget=x64`, `InternalsVisibleTo`, and `SatelliteResourceLanguages=ko;en`; record non-Windows host limitation verbatim if running on macOS/Linux without `dotnet`)
- [ ] T002 [P] Configure official `Microsoft.Extensions.TimeProvider.Testing` (`FakeTimeProvider`) in `tests/ImageViewer.Tests/ImageViewer.Tests.csproj` (or contract-faithful `ManualTimeProvider` with periodic/one-shot `ITimer.Change`/`Dispose` semantics if offline restore unavailable) and create inline synchronous `TestUiDispatcher : IUiDispatcher` in `tests/ImageViewer.Tests/TestFixtures/DeterministicTestFixtures.cs`
- [ ] T003 [P] Create real-filesystem security test fixture helpers (generators for UTF-32/UTF-16/UTF-8 BOM-prefixed and >512-byte space-padded polyglot payloads, oversized non-seekable streams, traversal + POSIX-`0xA000`/DOS-`0x0400` ZIP archives, and temporary directory cleanup) in `tests/ImageViewer.IntegrationTests/TestFixtures/SecurityTestFixtures.cs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core trust-boundary and UI-abstraction primitives shared across US1, US2, and US3.

**⚠️ CRITICAL**: Complete this phase before beginning user story phases.

- [ ] T004 Write failing unit tests for `PathSecurity` (canonical `Path.GetFullPath` root containment enforcing trailing `Path.DirectorySeparatorChar` to reject sibling-prefix bypasses such as `logs-evil` vs `logs`, archive entry key normalization `\`->`/` rejecting `..`/drive-root/leading-`/`/null-byte segments, `FileAttributes.ReparsePoint` / `LinkTarget != null` detection limited to immediate target dir/file, and ZIP `Attrib` POSIX `0xA000` / DOS `0x0400` symlink flag detection) and `PathRedactor` (stripping quoted and **space-containing** Windows `[A-Za-z]:\...`, UNC `\\...`, and Unix directory prefixes down to `<redacted>\<filename>` or filename-only, plus escaping `\r`/`\n`/control chars against CWE-117 log injection) in `tests/ImageViewer.Tests/PathSecurityTests.cs`
- [ ] T005 Implement `PathSecurity` and `PathRedactor` in `src/ImageViewer/Helpers/PathSecurity.cs` to satisfy all boundary, ZIP-attribute, and space/CRLF-safe redaction invariants tested in `T004`
- [ ] T006 [P] Define narrow UI-thread marshaling abstraction `IUiDispatcher` in `src/ImageViewer/Services/IUiDispatcher.cs` and implement WPF dispatcher adapter `WpfUiDispatcher` in `src/ImageViewer/Views/WpfUiDispatcher.cs`

**Checkpoint**: Shared path-containment, path-redaction, and UI-dispatcher primitives are verified and ready for P1–P3 implementation.

---

## Phase 3: User Story 1 - Zero Trust Input, Archive, and Privacy Hardening (Priority: P1) 🎯 MVP

**Goal**: Fail closed on extension-spoofed scripts/documents across encodings, enforce CWE-190 checked dimension arithmetic and CWE-409 bounded stream/animation caps, block CWE-22 archive/app-data traversal and `ReparsePoint` escapes, fail closed on malformed CLI/shell inputs (CWE-88), enforce `64 KB` + finite settings normalization, and strip full host paths from diagnostic logs and runtime UI error dialogs.

**Independent Test**: Run `FormatValidationTests`, `HelperTests`, `ShellIntegrationTests`, `ServiceTests`, `SecurityHardeningTests`, and `SecurityIntegrationTests` with crafted hostile inputs (`SC-001`, `SC-005`) and confirm 100% fail-closed rejection, default fallback, and redacted logging with zero crashes.

### Tests for User Story 1 (Red First — MUST FAIL before implementation)

- [ ] T007 [P] [US1] Write failing unit tests for `ImageFormatHelper.LooksLikeScriptOrDocument` covering 4-byte UTF-32 LE (`FF FE 00 00`) / UTF-32 BE (`00 00 FE FF`) BOMs checked before 2-byte UTF-16 LE (`FF FE`) / UTF-16 BE (`FE FF`) and UTF-8 (`EF BB BF`) BOMs, BOM-less UTF-16 LE/BE heuristics (`<\0`, `\0<`), leading whitespace/padding up to 8,192 bytes (rejecting >=512-byte pure-whitespace headers), and case-insensitive blocked prefixes (`<`, `<?xml`, `<svg`, `<html`, `%!`, `%PDF`, `push `, `viewbox`) in `tests/ImageViewer.Tests/FormatValidationTests.cs`
- [ ] T008 [P] [US1] Write failing unit tests for fail-closed `CommandLineHelper.Parse` verifying acceptance of `0` args OR `1` existing positional path (drive `<letter>:\...`, UNC `\\...`, relative `.\...`) OR `1` valid shell flag (`--register-shell`, `/register-shell`, `--unregister-shell`, `/unregister-shell`), while failing closed (`Error != null`, `Path == null`, `RegisterShell == false`, `UnregisterShell == false`) on `>=2` positional paths, duplicate or unknown `-`/`--`/`/` switches, simultaneous register+unregister flags, path+switch combinations, invalid path characters, and setting `IsPathNotFoundError = true` (preserving `C:\"` repair and 4-arg constructor compatibility) in `tests/ImageViewer.Tests/HelperTests.cs`
- [ ] T009 [P] [US1] Write failing unit tests for `ShellIntegrationService` verifying exact trimmed command matching (`"<exePath>" "%1"` / `"<exePath>" "%V"`) and menu label (`ImageViewer에서 보기(&3)`) in `IsRegistered()`, rejecting partial/wrapper substring matches, and throwing `ArgumentException` when `ExecutablePath` contains `"` or control characters in `tests/ImageViewer.Tests/ShellIntegrationTests.cs`
- [ ] T010 [P] [US1] Write failing unit tests for `SettingsService.Load()` (`> 65,536` byte `64 KB` file cap fallback to defaults, immediate directory/file `FileAttributes.ReparsePoint` rejection, corrupted JSON recovery, atomic randomized temp file save with orphan temp cleanup on failure) and `ViewerSettings.Normalize()` (`double.IsFinite` checks resetting `NaN`/`Infinity`/`-Infinity` to defaults/safe bounds across `Window.Width` `[320, 20000]` default `1280`, `Window.Height` `[240, 20000]` default `800`, nullable `Window.Left`/`Top` -> `null`, `Filter.SmoothSigma` `[0.1, 3.0]` default `0.5`, `Filter.SharpenSigma` `[0.1, 5.0]`, `Filter.SharpenAmount` `[0.1, 5.0]`, `Limits.MaxImageMegapixels` `[10, 1000]` default `150`, `Limits.MaxZipEntryMegabytes` `[10, 4096]` default `512`) in `tests/ImageViewer.Tests/ServiceTests.cs`
- [ ] T011 [P] [US1] Write failing unit tests for `ImageRenderService.ToBitmapSource` (checked 64-bit/`checked((int)(width * 4))` stride + configurable `maxPixels` limit), `AnimationSource` extreme-canvas CWE-190/megapixel overflow rejection (`DecodeFirstFrame`/`Freeze`), and `FileLoggerProvider` path-redaction stripping quoted & space-containing Windows/UNC/Unix full host paths plus CRLF log-injection characters from log entries and `Exception.Message` in `tests/ImageViewer.Tests/SecurityHardeningTests.cs`
- [ ] T012 [P] [US1] Write failing integration tests for UTF-32/UTF-16 `<svg>` & >512-byte space-padded MVG payloads disguised as `.png`/`.tga` triggering `GetFormatHint`, multi-frame `.ico`/TIFF secondary-frame pixel cap rejection, non-seekable streams exceeding configured `MaxZipEntryMegabytes`, deceptive-`entry.Size` ZIP bombs, ZIP traversal/`Attrib` POSIX-`0xA000` symlink/null-byte entry rejection, and `FolderImageSource` skipping `ReparsePoint` file entries while permitting explicitly user-opened linked top-level paths in `tests/ImageViewer.IntegrationTests/SecurityIntegrationTests.cs`

### Implementation for User Story 1 (Green & Refactor)

- [ ] T013 [P] [US1] Update `ImageFormatHelper.LooksLikeScriptOrDocument` (plus streaming overload up to 8,192 bytes) in `src/ImageViewer/Helpers/ImageFormatHelper.cs` to check 4-byte UTF-32 LE/BE BOMs before 2-byte UTF-16 LE/BE and UTF-8 BOMs, handle BOM-less UTF-16 interleaved ASCII markup (`<`, `<?xml`, `<svg`, `<html`, `%!`, `%PDF`, `push `, `viewbox`) case-insensitively after skipping BOM and whitespace, and reject `>=512`-byte pure-whitespace leading headers
- [ ] T014 [P] [US1] Refactor `CommandLineOptions(..., bool IsPathNotFoundError = false)` and `CommandLineHelper.Parse` in `src/ImageViewer/Helpers/CommandLineHelper.cs` to repair Explorer `C:\"` quoting, accept only `0` args OR `1` positional path (`<letter>:\`, `\\`, `.\`) OR `1` valid shell flag (`--register-shell`/`/register-shell`/`--unregister-shell`/`/unregister-shell`), and fail closed with localized Korean errors on extra paths, unknown or duplicate switches, conflicting flags, or invalid path syntax
- [ ] T015 [P] [US1] Harden `ShellIntegrationService` in `src/ImageViewer/Services/ShellIntegrationService.cs` to validate `ExecutablePath` contains no embedded `"` or control characters prior to HKCU `Software\Classes\...` writes and enforce exact normalized command-string + label equality in `IsRegistered()`
- [ ] T016 [P] [US1] Guard all `double` fields (`Window.Width`, `Window.Height`, nullable `Window.Left`/`Top`, `Filter.SmoothSigma`, `Filter.SharpenSigma`, `Filter.SharpenAmount`) with `double.IsFinite` before clamping in `src/ImageViewer/Models/ViewerSettings.cs`, and enforce `<= 65,536` byte (`64 KB`) size limit, immediate-path `PathSecurity` reparse-point check, `Path.GetRandomFileName()` atomic write, exception catching (`IOException`, `UnauthorizedAccessException`, `NotSupportedException`, `ArgumentException`), and `finally` orphaned temp file cleanup in `src/ImageViewer/Services/SettingsService.cs`
- [ ] T017 [P] [US1] Enforce `checked((int)(width * 4))` stride arithmetic and configurable `(long)width * height <= maxPixels` bounds (preserving backward-compatible default overload + `SettingsService` limit wiring) in `ImageRenderService.ToBitmapSource` in `src/ImageViewer/Services/ImageRenderService.cs`, and apply identical checked stride + `maxPixels` validation in `AnimationSource.DecodeFirstFrame()` and `AnimationPlayback.Freeze()` in `src/ImageViewer/Services/AnimationPlayback.cs`
- [ ] T018 [US1] Enforce streaming `ImageFormatHelper.LooksLikeScriptOrDocument` prior to both `MagickImageInfo` ping and `GetFormatHint(fileName)` fallback, add `long maxStreamBytes` + valid `CancellationToken` propagation to `OpenSeekableAsync` and `TryReadAnimation` (`MaxZipEntryMegabytes` cap before chunked copy and `new byte[...]`), enforce `maxPixels` on multi-frame `.ico`/`.cur` (`ReadLargestFrame`) and multi-page TIFF `pageIndex > 0` before decoding, and redact logged failure details through `PathRedactor` in `src/ImageViewer/Services/ImageLoaderService.cs` (and update `OpenSeekableAsync` call in `src/ImageViewer/Services/ThumbnailService.cs` so the solution compiles cleanly at MVP checkpoint)
- [ ] T019 [P] [US1] Enforce `PathSecurity` `ReparsePoint` skipping during `FolderImageSource` folder enumeration (preserving explicitly user-opened top-level paths) in `src/ImageViewer/Services/ImageSources/FolderImageSource.cs` and enforce `PathSecurity` in-memory archive key validation (`..`, drive root, leading `/`, null bytes), `LinkTarget != null` + ZIP `Attrib` POSIX `0xA000` / DOS `0x0400` symlink rejection, and bounded streaming reads in `src/ImageViewer/Services/ImageSources/ZipImageSource.cs`
- [ ] T020 [US1] Extract `FormatSanitizedEntry` using `PathRedactor` (space/quote-aware + CRLF safe) and immediate-path `PathSecurity` reparse-point check in `src/ImageViewer/Services/FileLoggerProvider.cs`, initialize/flush a scoped `FileLoggerProvider` on failure inside `App.RunShellCommand` (which runs prior to `ConfigureServices()`), and omit raw `e.Exception.Message` host paths from `OnDispatcherUnhandledException` UI dialogs (echoing user paths only when `IsPathNotFoundError` is true) in `src/ImageViewer/App.xaml.cs`

**Checkpoint**: User Story 1 (MVP — Zero Trust Hardening) is complete and independently verifiable against all `SC-001` and `SC-005` negative security tests.

---

## Phase 4: User Story 2 - Deterministic Lifecycle, Concurrency, and Responsiveness Stability (Priority: P2)

**Goal**: Eliminate OpenMP CPU oversubscription via startup native resource configuration, remove sync-over-async blocking, convert file logging to a bounded non-blocking `Channel<SanitizedLogRecord>`, enforce deterministic `IDisposable`/`IAsyncDisposable` and explicit `SKManagedStream` ownership, and isolate drag-and-drop and shutdown transitions from unhandled async errors or handler leaks.

**Independent Test**: Run `LifecycleAndLoggingTests` and `ConcurrencyLifecycleTests` (`SC-002`) executing 50+ consecutive mixed static/animated switches with mid-load cancellation/disposal and verify zero unhandled exceptions, zero leaked gates/timers, and clean bounded logging drain.

### Tests for User Story 2 (Red First — MUST FAIL before implementation)

- [ ] T021 [P] [US2] Write failing unit tests for `FileLoggerProvider` bounded non-blocking `Channel<SanitizedLogRecord>` buffering (`DropOldest`, safe recovery when log path is read-only/unavailable, synchronous `Dispose()` bounded timeout drain without WPF-context deadlock, and `DisposeAsync()`) and `ImageRenderService`/`ImageLoaderService` deterministic `IDisposable` gate/cache disposal in `tests/ImageViewer.Tests/LifecycleAndLoggingTests.cs`
- [ ] T022 [P] [US2] Write failing integration tests for deadlock-free async `ThumbnailService` worker execution with cancellation propagation, concurrent `ZipImageSource.OpenImageAsync` + `Dispose`/`DisposeAsync` race safety (zero `ObjectDisposedException` on `_gate.Release()`), `AnimationSource` repeated `OpenCodec`/dispose keeping native stream ownership clean without double-free, and 50+ rapid image switches + mid-flight disposal (`SC-002`) in `tests/ImageViewer.IntegrationTests/ConcurrencyLifecycleTests.cs`

### Implementation for User Story 2 (Green & Refactor)

- [ ] T023 [P] [US2] Refactor `FileLoggerProvider` in `src/ImageViewer/Services/FileLoggerProvider.cs` reusing `FormatSanitizedEntry` to enqueue materialized `SanitizedLogRecord` entries into a bounded `Channel<SanitizedLogRecord>` (`BoundedChannelFullMode.DropOldest`, non-blocking `TryWrite`), consume asynchronously via `.ConfigureAwait(false)` with fail-silent I/O handling, and implement both `IDisposable` (bounded 2-second timeout drain) and `IAsyncDisposable`
- [ ] T024 [P] [US2] Eliminate `.GetAwaiter().GetResult()` in `ThumbnailService` via async worker execution (`await OpenSeekableAsync(...)` with configured `maxStreamBytes` and worker cancellation token) in `src/ImageViewer/Services/ThumbnailService.cs`, and implement race-free `IDisposable` + `IAsyncDisposable` in `src/ImageViewer/Services/ImageSources/ZipImageSource.cs` (ensuring active `OpenImageAsync`/`GetImagesAsync` `finally { _gate.Release(); }` callers never hit `ObjectDisposedException` during concurrent close)
- [ ] T025 [P] [US2] Implement `IDisposable` disposing `_backgroundGate` and owned state in `src/ImageViewer/Services/ImageRenderService.cs`, and dispose `_backgroundGate` alongside `_cache` inside `Dispose()` in `src/ImageViewer/Services/ImageLoaderService.cs`
- [ ] T026 [P] [US2] Update `AnimationSource` in `src/ImageViewer/Services/AnimationPlayback.cs` to own caller `using var stream = new MemoryStream(data, writable: false);` and transfer an unwrapped `new SKManagedStream(stream, disposeManagedStream: false)` to `SKCodec.Create(...)` (never double-wrapping `SKManagedStream` in its own `using` block)
- [ ] T027 [US2] Set `OMP_NUM_THREADS` (`Math.Clamp(Environment.ProcessorCount / 2, 1, 4)`) and `MAGICK_THREAD_LIMIT` at the start of `OnStartup` **before `ConfigureServices()` or any `ImageMagick` type initializes**, call `MagickNET.Initialize()`, configure `ResourceLimits.Thread` (`Math.Clamp(Environment.ProcessorCount / 2, 1, 4)`) + `ResourceLimits.Memory` (`1024UL * 1024 * 1024`) in `App.ConfigureMagick()`, and verify safe `ServiceProvider.Dispose()` teardown of `FileLoggerProvider` in `src/ImageViewer/App.xaml.cs`
- [ ] T028 [US2] Guard `MainWindow.OnDrop` (`async void`) with `try/catch` + sanitized error reporting and detach `_vm.LayoutChanged` and `_vm.PropertyChanged` subscriptions cleanly inside `OnClosed` in `src/ImageViewer/MainWindow.xaml.cs`, while ensuring active timer, watcher, CTS cancellation, and owned-service disposal (invoking `IAsyncDisposable` if supported) in `MainViewModel.Dispose()` in `src/ImageViewer/ViewModels/MainViewModel.cs`

**Checkpoint**: User Stories 1 and 2 work cleanly together with bounded concurrency, deadlock-free thumbnails/archives, non-blocking logging, and deterministic teardown.

---

## Phase 5: User Story 3 - Modular Architecture and Headless Verification Coverage (Priority: P3)

**Goal**: Decompose `MainViewModel` (2,106 lines) and `MainWindow.xaml.cs` (849 lines) into pure, headless-testable domain/orchestration components (`ViewportLayoutCalculator`, `WindowChromeState`, `ImageSessionCoordinator`, `PrefetchCoordinator`, `SlideShowService` with `TimeProvider`) behind a thin `MainViewModel` facade retaining 100% compatibility with all `MainWindow.xaml` bindings, keyboard shortcuts, localization, and `settings.json` v1–v3 behavior.

**Independent Test**: Run `ViewportLayoutCalculatorTests`, `WindowChromeStateTests`, `SlideShowServiceTests`, `ImageSessionCoordinatorTests`, and `MainViewModelHeadlessTests` headlessly (`SC-003`, `SC-004`, `SC-006`) without instantiating any WPF `Window`.

### Tests for User Story 3 (Red First — MUST FAIL before implementation)

- [ ] T029 [P] [US3] Write failing headless unit tests for `ViewportLayoutCalculator` covering display sizing across `DisplayMode` variants, cursor zoom anchoring, pan offset clamping, placeholder display sizing, and zero/negative viewport guardrails in `tests/ImageViewer.Tests/ViewportLayoutCalculatorTests.cs`
- [ ] T030 [P] [US3] Write failing headless unit tests for `WindowChromeState` covering top/bottom bar pinning, edge hover visibility, fullscreen toggle entry, and fullscreen exit state restoration in `tests/ImageViewer.Tests/WindowChromeStateTests.cs`
- [ ] T031 [P] [US3] Write failing headless unit tests for `SlideShowService` using `FakeTimeProvider`/`ManualTimeProvider` covering `[5, 90]` second interval advancement, sequential/random navigation modes, repeat wrap behavior, pause/resume, and clean timer disposal in `tests/ImageViewer.Tests/SlideShowServiceTests.cs`
- [ ] T032 [P] [US3] Write failing headless unit tests for `ImageSessionCoordinator` and `PrefetchCoordinator` using `FakeTimeProvider`/`ManualTimeProvider`, `TestUiDispatcher`, and real in-memory/temp-folder `IImageSource` instances covering monotonic `_openVersion`/`_loadVersion`/`_renderVersion` cancellation races, delayed placeholder display timer transitions, `DecodedImage` lifetime ownership, directional `+1`/`+2`/`-1`/burst prefetch ordering, and full `Dispose()` cleanup in `tests/ImageViewer.Tests/ImageSessionCoordinatorTests.cs`
- [ ] T033 [P] [US3] Write failing headless integration-facade tests for `MainViewModel` verifying signature/execution preservation of all `MainWindow.xaml` commands + `MainWindow.xaml.cs` `_vm.*` calls (`OpenFileCommand`, `PreviousCommand`, `NextCommand`, `FirstCommand`, `LastCommand`, `ZoomInCommand`, `RotateClockwiseCommand`, `ToggleInfoPanelCommand`, `ToggleSlideShowCommand`, `ToggleFullScreenCommand`, `UpdateViewport`, `GoToAsync`, `OpenPathAsync`, `SaveSettings`, etc.), template item properties (`MetadataItem`, `ThumbnailItemViewModel`), `TimeProvider`-driven debounce/OSD/loading/animation/folder-watch timers, and `PlaceholderTests` compatibility (`PlaceholderDisplaySize`, `PlaceholderDelay`) without instantiating `Window` in `tests/ImageViewer.Tests/MainViewModelHeadlessTests.cs`

### Implementation for User Story 3 (Green & Refactor)

- [ ] T034 [P] [US3] Extract pure display geometry, zoom anchoring, pan clamping, and placeholder dimension logic from `MainWindow.xaml.cs` and `MainViewModel` into `src/ImageViewer/Services/ViewportLayoutCalculator.cs` (retaining forwarding wrappers on `MainViewModel` for `PlaceholderTests.cs`)
- [ ] T035 [P] [US3] Extract pin/hover/fullscreen restore state transitions from `MainWindow.xaml.cs` and `MainViewModel` into `src/ImageViewer/ViewModels/WindowChromeState.cs`
- [ ] T036 [P] [US3] Inject `System.TimeProvider` into `SlideShowService` (replacing WPF-coupled `DispatcherTimer` while preserving `[5, 90]` interval, sequential/random mode, repeat, and event semantics) in `src/ImageViewer/Services/SlideShowService.cs`
- [ ] T037 [P] [US3] Extract directional lookahead (`+1`, `+2`, `-1`, burst up to `+4`) and slideshow peek scheduling into `src/ImageViewer/Services/PrefetchCoordinator.cs`
- [ ] T038 [US3] Implement `ImageSessionCoordinator` owning active `IImageSource?`, `_images`, monotonic `_openVersion`/`_loadVersion`/`_renderVersion` CTS cancellation, `DecodedImage?` lifecycle, `TimeProvider` placeholder delay timer, and `IUiDispatcher` marshaling in `src/ImageViewer/Services/ImageSessionCoordinator.cs`
- [ ] T039 [US3] Refactor `src/ImageViewer/ViewModels/MainViewModel.cs` and `src/ImageViewer/ViewModels/MainViewModel.Features.cs` into a cohesive MVVM facade moving all 6 `DispatcherTimer` fields (`_renderDebounce`, `_osdTimer`, `_loadingIndicatorTimer`, `_placeholderTimer`, `_animationTimer`, `_watchDebounce`) onto `System.TimeProvider` + `IUiDispatcher`, delegating to `ViewportLayoutCalculator`, `WindowChromeState`, `ImageSessionCoordinator`, and `PrefetchCoordinator`, and preserving 100% XAML/`MainWindow.xaml.cs`/`PlaceholderTests.cs` compatibility (`contracts/modular-architecture-contract.md`)
- [ ] T040 [US3] Thin `src/ImageViewer/MainWindow.xaml.cs` to forward zoom/pan and chrome-state updates through `ViewportLayoutCalculator` and `WindowChromeState`, and register `IUiDispatcher` (`WpfUiDispatcher`), `TimeProvider.System`, `ViewportLayoutCalculator`, `WindowChromeState`, `PrefetchCoordinator`, and `ImageSessionCoordinator` inside `src/ImageViewer/App.xaml.cs`

**Checkpoint**: All core orchestration, math, chrome state, security checks, and navigation workflows are modularized and verified headlessly alongside 100% XAML binding compatibility.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Constitutional compliance audits, regression sweeps, and end-to-end validation across the entire solution.

- [ ] T041 [P] Audit all `.csproj`, `src/ImageViewer/`, and `tests/` files to verify zero mocking frameworks, zero `[Fact(Skip = ...)]`, zero loosened assertions, and zero `#if`/test-only production switches (Constitution Principle IV)
- [ ] T042 [P] Audit all new and modified C# identifiers, docstrings, and comments across `src/ImageViewer/` and `tests/` for English-first compliance (Principle II), verify `ko;en` resource and Korean CLI error preservation (Principle I), and confirm every contract in `specs/001-architecture-security-refactoring/contracts/` is satisfied
- [ ] T043 Run full solution build and test validation per `specs/001-architecture-security-refactoring/quickstart.md` (`dotnet build ImageViewer.sln -c Debug`, `dotnet test tests/ImageViewer.Tests/ImageViewer.Tests.csproj --no-build`, `dotnet test tests/ImageViewer.IntegrationTests/ImageViewer.IntegrationTests.csproj --no-build`, `dotnet test ImageViewer.sln`) on Windows 10/11 x64 and report exact observed pass/fail counts (explicitly disclosing non-Windows host limitation if `dotnet` / `net10.0-windows` cannot execute locally)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1 completion — BLOCKS Phases 3, 4, and 5 (`PathSecurity`, `PathRedactor`, and `IUiDispatcher` are shared across stories).
- **Phase 3 (User Story 1 — P1 MVP)**: Depends on Phase 2 completion. Executes first per Priority P1 (`Zero Trust Security & Privacy`).
- **Phase 4 (User Story 2 — P2)**: Depends on Phase 2 (and integrates cleanly onto US1's hardened `FileLoggerProvider`, `ImageLoaderService`, `ImageRenderService`, `ZipImageSource`, and `App.xaml.cs`).
- **Phase 5 (User Story 3 — P3)**: Depends on Phase 2 and P1/P2 stabilization of core services (`ImageLoaderService`, `ImageRenderService`, `MainViewModel`, `MainWindow.xaml.cs`).
- **Phase 6 (Polish)**: Depends on completion of Phases 3, 4, and 5.

### Within Each User Story (Classical Red-Green-Refactor Order)

1. Write failing `xUnit` unit/integration tests FIRST and observe expected failure reasons.
2. Implement pure helpers/models/validators before orchestrating services.
3. Wire service changes into `App.xaml.cs`, `MainWindow.xaml.cs`, and `MainViewModel` without breaking XAML bindings.
4. Confirm tests transition Red -> Green, refactor without altering green status, and verify checkpoint before proceeding to next priority story.

---

## Parallel Execution Examples

### User Story 1 (`US1` — Security Hardening)

```text
# Parallel Red-phase test authoring (distinct files):
- T007: FormatValidationTests.cs (UTF-32/16/8 BOM + 8 KB space padding + SVG/MVG/PDF signatures)
- T008: HelperTests.cs (Fail-closed CLI parsing)
- T009: ShellIntegrationTests.cs (Exact quoted shell-verb match & quote sanitization)
- T010: ServiceTests.cs (64 KB settings cap & finite normalization)
- T011: SecurityHardeningTests.cs (CWE-190 checked stride, AnimationSource overflow & space/CRLF-safe PathRedactor tests)
- T012: SecurityIntegrationTests.cs (Spoofed archives, multi-frame cap, non-seekable cap, POSIX 0xA000 ZIP attrib)

# Parallel Green-phase module hardening (distinct files):
- T013: ImageFormatHelper.cs
- T014: CommandLineHelper.cs
- T015: ShellIntegrationService.cs
- T016: ViewerSettings.cs + SettingsService.cs
- T017: ImageRenderService.cs
- T019: FolderImageSource.cs + ZipImageSource.cs
```

### User Story 2 (`US2` — Lifecycle & Concurrency)

```text
# Parallel Red-phase tests:
- T021: LifecycleAndLoggingTests.cs
- T022: ConcurrencyLifecycleTests.cs

# Parallel Green-phase service fixes:
- T023: FileLoggerProvider.cs (Bounded Channel<T> + DisposeAsync)
- T024: ThumbnailService.cs + ZipImageSource.cs (Async workers + IAsyncDisposable)
- T025: ImageRenderService.cs + ImageLoaderService.cs (IDisposable gate cleanup)
- T026: AnimationPlayback.cs (Explicit SKManagedStream lifetime)
```

### User Story 3 (`US3` — Modular Decomposition & Headless TDD)

```text
# Parallel Red-phase headless tests:
- T029: ViewportLayoutCalculatorTests.cs
- T030: WindowChromeStateTests.cs
- T031: SlideShowServiceTests.cs
- T032: ImageSessionCoordinatorTests.cs
- T033: MainViewModelHeadlessTests.cs

# Parallel Green-phase pure extractions:
- T034: ViewportLayoutCalculator.cs
- T035: WindowChromeState.cs
- T036: SlideShowService.cs (TimeProvider)
- T037: PrefetchCoordinator.cs
```

---

## Implementation Strategy

### MVP First (User Story 1 — Zero Trust Hardening)

1. Complete **Phase 1 (Setup)** and **Phase 2 (Foundational)** (`T001`–`T006`).
2. Complete **Phase 3 (User Story 1 / P1)** (`T007`–`T020`).
3. **STOP and VALIDATE**: Confirm all P1 security unit/integration tests pass without relaxing assertions; at this milestone `ImageViewer` enforces all CWE-22/88/190/409/settings/redaction defenses.

### Incremental Delivery (`P1 -> P2 -> P3`)

1. **Increment 1 (`US1`)**: Zero Trust trust-boundary hardening (`T007`–`T020`).
2. **Increment 2 (`US2`)**: Concurrency bounds, deadlock removal, non-blocking logging, deterministic disposal (`T021`–`T028`).
3. **Increment 3 (`US3`)**: Modular extraction & headless TDD suite preserving 100% XAML bindings (`T029`–`T040`).
4. **Final Verification (`Phase 6`)**: Constitutional audit + full Windows x64 `dotnet test ImageViewer.sln` pass (`T041`–`T043`).
