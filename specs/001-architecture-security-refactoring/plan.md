# Implementation Plan: Project-Wide Architecture, Security, and Reliability Refactoring

**Branch**: `001-architecture-security-refactoring` | **Date**: 2026-11-15 | **Spec**: [spec.md](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/spec.md)

**Input**: Feature specification from `/specs/001-architecture-security-refactoring/spec.md`

## Summary

Incremental, TDD-driven refactoring of `ImageViewer` executed in strict priority order:
1. **P1 — Zero Trust Security & Privacy Hardening**: Multi-encoding streaming signature validation (`ImageFormatHelper`, UTF-32-before-UTF-16 BOM order, <=8,192-byte whitespace scan), CWE-190 checked dimension/stride arithmetic + dynamic `maxPixels` enforcement across static (`ImageRenderService`, multi-frame `.ico`/TIFF) and animated (`AnimationPlayback`) paths, CWE-409 bounded/cancelable `OpenSeekableAsync` and `TryReadAnimation`, CWE-22 root-boundary canonicalization + `FolderImageSource` reparse skip + `ZipImageSource` POSIX `0xA000`/DOS `0x0400` symlink flags (`PathSecurity`), CWE-88 fail-closed CLI parsing (`CommandLineOptions` backward-compatible `IsPathNotFoundError = false`) and exact quoted shell-verb verification (`ShellIntegrationService` + scoped `RunShellCommand` logging), finite + `64 KB`-capped configuration loading with temp cleanup (`ViewerSettings`, `SettingsService`), and space/quoted-path + CRLF-sanitized logging (`PathRedactor`, `FileLoggerProvider`, `App`).
2. **P2 — Deterministic Lifecycle, Concurrency & Responsiveness Stability**: Pre-DI `OMP_NUM_THREADS`/`MAGICK_THREAD_LIMIT` initialization + `MagickNET.Initialize()` + explicit `ResourceLimits.Thread` and `ResourceLimits.Memory` (`1024 MB`), elimination of sync-over-async blocking (`ThumbnailService`, `ZipImageSource`), safe non-throwing `ZipImageSource` `_gate` disposal alongside `IDisposable`/`IAsyncDisposable`, non-blocking bounded-`Channel` logging implementing both `IDisposable` (bounded timeout + `.ConfigureAwait(false)`) and `IAsyncDisposable`, non-double-disposed `SKManagedStream` ownership transfer (`AnimationPlayback`), and guarded drag-and-drop / window teardown (`MainWindow`, `MainViewModel`).
3. **P3 — Modular Decomposition & Headless Testability**: Extract pure calculation, state, and orchestration coordinators (`ViewportLayoutCalculator`, `WindowChromeState`, `ImageSessionCoordinator`, `PrefetchCoordinator`, `PathSecurity`, `PathRedactor`, cohesive format/animation helpers) paired with `IUiDispatcher` and `System.TimeProvider` across all 6 `MainViewModel` timers and `SlideShowService` so core navigation, layout, placeholder timing, slideshow, and security boundaries execute headlessly in `xUnit` while keeping `MainWindow.xaml` bindings and `MainWindow.xaml.cs` / `PlaceholderTests.cs` members 100% compatible.

## Technical Context

**Language/Version**: C# (`LangVersion` latest, `Nullable` enable, `ImplicitUsings` enable) on `.NET 10` (`net10.0-windows`, x64)

**Primary Dependencies**: `CommunityToolkit.Mvvm` (8.4.2), `Magick.NET-Q8-OpenMP-x64` (15.1.0), `SkiaSharp` (3.119.1), `SharpCompress` (1.0.0), `Microsoft.Extensions.DependencyInjection` (10.0.15), `Microsoft.Extensions.Logging` (10.0.15); optional test package `Microsoft.Extensions.TimeProvider.Testing` for deterministic `TimeProvider` advancement (or real `TimeProvider.System` in integration tests)

**Storage**: Local filesystem images, ZIP/CBZ archives (read-only), `%LOCALAPPDATA%\ImageViewer\settings.json` (`<= 65,536` bytes), `%LOCALAPPDATA%\ImageViewer\logs\imageviewer-yyyyMMdd.log`, HKCU Windows Registry (`Software\Classes\...`)

**Testing**: `xUnit` (2.9.3) with `Microsoft.NET.Test.Sdk` (18.12.0) and `xunit.runner.visualstudio` (3.1.5) across `tests/ImageViewer.Tests` and `tests/ImageViewer.IntegrationTests` (both granted `InternalsVisibleTo`)

**Target Platform**: Windows 10 / 11 (x64 only, `UseWPF=true`); note that the current macOS workspace lacks the `dotnet` CLI (`command not found: dotnet`) and Windows Desktop targeting pack, requiring final compilation/test execution on a Windows x64 runner/CI

**Project Type**: Windows WPF desktop application (`src/ImageViewer`) + NSIS installer (`src/ImageViewer.Installer`) + unit/integration test suites (`tests/`)

**Performance Goals**: Preserve documented pipeline latency targets (single-image render under ~40–50 ms on target desktop hardware, non-blocking UI input during rapid navigation, bounded foreground/prefetch concurrency without OpenMP CPU thrashing)

**Constraints**: Zero breaking changes to `MainWindow.xaml` bindings, keyboard shortcuts, `settings.json` v1–v3 migration, `ko;en` localization, or `--register-shell`/`--unregister-shell` flags; pre-decode `150 MP` pixel cap, `512 MB` stream/animation cap, `64 KB` settings cap; no mocking frameworks or production test bypasses

**Scale/Scope**: Refactoring across `src/ImageViewer/` (~40 source files) and expansion of `tests/ImageViewer.Tests/` + `tests/ImageViewer.IntegrationTests/`, decomposing monolithic presentation/orchestration concerns out of `MainViewModel` (2,106 lines across two partial files) and `MainWindow.xaml.cs` (849 lines) while hardening and factoring `ImageLoaderService.cs` (709 lines) and `AnimationPlayback.cs` (409 lines) into cohesive validators and session coordinators

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Pre-Research Gate | Post-Design Verification |
|---|---|---|
| **I. Korean Communication & Commit Policy** | All user prompts/answers in Clarify and final summaries are delivered in Korean. | **PASS** — Plan responses and commit suggestions remain Korean; technical design docs remain English. |
| **II. English-First Reasoning & Implementation** | Spec, research, models, contracts, and planned C#/xUnit additions use English identifiers/comments. | **PASS** — All new/modified symbols, tests, and design artifacts (`research.md`, `data-model.md`, `contracts/*`) are in English; `ko;en` UI resources remain untouched. |
| **III. Test-Driven Development (NON-NEGOTIABLE)** | Plan enforces Red-Green-Refactor task ordering per story in `ImageViewer.Tests` and `ImageViewer.IntegrationTests`. | **PASS** — Extracted coordinators (`ViewportLayoutCalculator`, `ImageSessionCoordinator`, `WindowChromeState`, `PathSecurity`, `PathRedactor`) enable headless xUnit Red-Green-Refactor cycles. |
| **IV. Zero Hardcoding, Mocking, or Test-Bypass Logic (NON-NEGOTIABLE)** | No mocking frameworks (`Moq`, `NSubstitute`) or `#if` test switches introduced; third-party libs (`Magick.NET`, `SkiaSharp`, `SharpCompress`) tested with real streams/files. | **PASS** — Only narrow OS/clock boundaries (`IDialogService`, `IImageSource`, `IUiDispatcher`, `System.TimeProvider`) use contract-faithful implementations; zero test-only branches in `src/`. |
| **V. Evidence-Based Research & Verification (NON-NEGOTIABLE)** | Phase 0 `research.md` cites Microsoft Learn, OWASP, NIST SP 800-207, MITRE CWEs, Magick.NET/SkiaSharp/SharpCompress docs, and exact codebase audits. | **PASS** — All 7 design decisions in `research.md` trace directly to retrieved references and observed files. |
| **VI. Skeptical Verification & Fallibility Awareness (NON-NEGOTIABLE)** | Plan addresses concurrency races, `IDisposable` leaks, `SKCodec` stream ownership, `NaN` clamping, and macOS `dotnet` absence explicitly. | **PASS** — Non-Windows host build/test limitation is flagged explicitly in `quickstart.md` and Technical Context without fabrication. |
| **VII. Zero Trust Security & Untrusted Input Stance (NON-NEGOTIABLE)** | Plan maps CWE-22, CWE-88, CWE-190, CWE-409, `64 KB` settings cap, CLI fail-closed rules, and log path redaction. | **PASS** — All trust boundaries, negative security test scenarios, and fail-closed contracts are specified in `contracts/`. |

## Project Structure

### Documentation (this feature)

```text
specs/001-architecture-security-refactoring/
├── spec.md              # Feature specification + 5 session clarifications
├── research.md          # Phase 0 evidence-based research decisions (R1–R7)
├── data-model.md        # Phase 1 entities, invariants, and state transitions
├── quickstart.md        # Phase 1 build & validation guide (Windows + host gap note)
├── checklists/
│   └── requirements.md  # Spec quality validation checklist (16/16 passing)
├── contracts/
│   ├── cli-and-shell-contract.md
│   ├── security-validation-contract.md
│   └── modular-architecture-contract.md
└── plan.md              # This file (/speckit-plan output)
```

### Source Code (repository root)

```text
src/ImageViewer/
├── App.xaml / App.xaml.cs                  # Pre-DI OMP_NUM_THREADS/MagickNET.Initialize/ResourceLimits, scoped RunShellCommand logging, DI, sanitized unhandled error dialog
├── MainWindow.xaml / MainWindow.xaml.cs    # Thin WPF view: event detach on close, guarded OnDrop, delegates math/chrome to extracted state
├── Helpers/
│   ├── CommandLineHelper.cs                # Fail-closed single-path/single-shell-switch parsing (IsPathNotFoundError = false default)
│   ├── ImageFormatHelper.cs                # <=8 KB whitespace-safe multi-BOM (UTF-32 first) & MVG/SVG script rejection + extension allowlist
│   └── PathSecurity.cs                     # (new) Canonical root-delimiter checks, immediate ReparsePoint & ZIP Attrib guards, space/CRLF-safe PathRedactor
├── Models/
│   └── ViewerSettings.cs                   # Finite-value (double.IsFinite) validation + range clamping
├── Services/
│   ├── IUiDispatcher.cs                    # (new) UI-thread marshaling abstraction (WpfUiDispatcher in Views/)
│   ├── FileLoggerProvider.cs               # Non-blocking bounded Channel<T> logger (IDisposable + IAsyncDisposable bounded drain, ConfigureAwait(false), PathRedactor)
│   ├── SettingsService.cs                  # 64 KB cap, immediate ReparsePoint guard, randomized temp file atomic save + cleanup
│   ├── ShellIntegrationService.cs          # Exact quoted-command IsRegistered check & path quote sanitization
│   ├── ImageLoaderService.cs               # Streaming byte caps + CT propagation in OpenSeekableAsync/TryReadAnimation, header-before-hint validation, secondary-frame maxPixels, IDisposable gate cleanup
│   ├── ImageRenderService.cs               # Checked CWE-190 stride/dimension arithmetic + maxPixels guard, IDisposable implementation
│   ├── AnimationPlayback.cs                # Ownership-safe SKManagedStream (caller MemoryStream without double-using) + checked SKBitmap/BitmapSource pixel bounds
│   ├── ThumbnailService.cs                 # Async worker loop (eliminates GetAwaiter().GetResult()) passing maxStreamBytes + CT
│   ├── SlideShowService.cs                 # Injected TimeProvider timer abstraction
│   ├── ViewportLayoutCalculator.cs         # (new) Pure display/zoom/pan/placeholder geometry calculations
│   ├── ImageSessionCoordinator.cs          # (new) Versioned open/load/render lifecycle, placeholder timer, cancellation
│   ├── PrefetchCoordinator.cs              # (new) Directional & slideshow lookahead scheduling
│   └── ImageSources/
│       ├── FolderImageSource.cs            # Skip ReparsePoint entries during folder enumeration (top-level links allowed)
│       └── ZipImageSource.cs               # Traversal/POSIX-Attrib symlink validation, bounded entry reads, race-free Dispose/IAsyncDisposable
├── ViewModels/
│   ├── WindowChromeState.cs                # (new) Pin/hover/fullscreen state decoupled from Window visual tree
│   ├── MainViewModel.cs                    # Thin MVVM facade (all 6 timers on TimeProvider, full XAML + PlaceholderTests compatibility)
│   └── MainViewModel.Features.cs           # Slimmed/reallocated to extracted coordinators
└── Views/
    └── WpfUiDispatcher.cs                  # (new) WPF Dispatcher implementation of IUiDispatcher

tests/ImageViewer.Tests/
├── HelperTests.cs                          # Expanded CLI fail-closed, BOM/UTF-16 signature, PathSecurity, PathRedactor tests
├── ServiceTests.cs                         # Settings 64 KB + NaN/Infinity, CWE-190 overflow, non-blocking logger, SlideShow TimeProvider tests
├── ShellIntegrationTests.cs                # Exact quoted-command match & embedded-quote rejection tests
├── ViewportLayoutCalculatorTests.cs        # (new) Pure zoom/pan/anchor/placeholder geometry unit tests
├── WindowChromeStateTests.cs               # (new) Pin/hover/fullscreen restore state tests
└── ImageSessionCoordinatorTests.cs         # (new) Headless cancellation-race, disposal, and placeholder timing tests

tests/ImageViewer.IntegrationTests/
├── ImageSourceTests.cs                     # Expanded Zip Slip, ReparsePoint skip/reject, and non-seekable cap tests
├── ImageProcessingTests.cs                 # Expanded UTF-16 spoofed header and oversized-animation payload rejection tests
└── ConcurrencyLifecycleTests.cs            # (new) Rapid 50+ image navigation & mid-load disposal stress tests
```

**Structure Decision**: Retain the existing single-solution layout (`src/ImageViewer`, `tests/ImageViewer.Tests`, `tests/ImageViewer.IntegrationTests`) while extracting cohesive pure helpers, coordinators, and interfaces inside `src/ImageViewer/` so all existing `InternalsVisibleTo` test access and XAML bindings work seamlessly without introducing unnecessary new `.csproj` assemblies.

## Complexity Tracking

> No Constitution violations exist. All design choices adhere strictly to Principles I–VII without unilateral waivers or added project-assembly sprawl.
