# Quickstart & Validation Guide: Architecture, Security, and Reliability Refactoring

**Feature**: `001-architecture-security-refactoring`
**Spec**: [spec.md](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/spec.md)
**Data Model**: [data-model.md](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/data-model.md)
**Contracts**: [contracts/](file:///Users/dev/DEV/ImageViewer/specs/001-architecture-security-refactoring/contracts)

---

## 1. Prerequisites & Host Platform Notice

- **Target Runtime**: Windows 10 / 11 (x64), `.NET 10 SDK` (`net10.0-windows`, `UseWPF`).
- **Non-Windows Host Constraint**: WPF targets (`net10.0-windows`) and `Magick.NET-Q8-OpenMP-x64` Windows execution cannot run on macOS/Linux hosts lacking the Windows Desktop SDK (`command not found: dotnet` observed on the current macOS workspace). All build/test execution MUST be verified on a Windows 10/11 x64 runner/CI after implementation, and any non-Windows host limitation MUST be reported verbatim per Constitution Principle VI.

---

## 2. Build & Automated Verification Commands (Windows)

```powershell
# 1. Restore and build entire solution (x64)
dotnet build ImageViewer.sln -c Debug

# 2. Run all unit tests (headless domain, security, layout, ViewModel coordinators)
dotnet test tests/ImageViewer.Tests/ImageViewer.Tests.csproj --no-build

# 3. Run all integration tests (real folders, ZIP/CBZ archives, Magick.NET, SkiaSharp)
dotnet test tests/ImageViewer.IntegrationTests/ImageViewer.IntegrationTests.csproj --no-build

# 4. Full solution test verification (zero skipped or weakened assertions)
dotnet test ImageViewer.sln
```

---

## 3. End-to-End Validation Scenarios

1. **Zero Trust Security Suite (`SC-001`, `SC-005`)**:
   - Feed UTF-32 LE/BE (`FF FE 00 00` checked before `FF FE`), UTF-16 LE/BE BOM-prefixed, and >512-byte space-padded `<svg>`/`<?xml>`/`push graphic-context` files renamed to `.png`/`.tga` -> verify `ImageFormatHelper`/`ImageLoaderService` reject before decoding.
   - Feed a ZIP entry with spoofed small `entry.Size` or POSIX `0xA000`/DOS `0x0400` symlink bits and a non-seekable stream exceeding `MaxEntryBytes`, plus an extreme-width multi-frame `.ico` or `AnimationSource` -> verify `ImageLoadException`/`ImageSourceException` without OOM or integer overflow.
   - Feed `settings.json` > `65,536` bytes or containing `NaN`/`Infinity` -> verify safe fallback defaults and finite clamping.
   - Pass multiple paths, duplicate/unknown flags (`--evil`), or `--register-shell` + `--unregister-shell` to `CommandLineHelper.Parse` -> verify fail-closed errors and scoped logging on shell failure.
   - Trigger `IOException` / `UnauthorizedAccessException` containing spaces in paths (`C:\Users\My Secret User\...`) and embedded newlines -> verify `FileLoggerProvider` output contains zero full directory paths and zero injected log lines.
2. **Lifecycle & Concurrency Stress Suite (`SC-002`)**:
   - Rapidly navigate across 50+ mixed static and animated images using `ImageSessionCoordinator` + `PrefetchCoordinator` (with `FakeTimeProvider` and real `TimeProvider.System`), then dispose mid-flight -> verify zero unhandled exceptions, zero active timers, and bounded cache size.
3. **Headless Decomposition Verification (`SC-003`, `SC-004`, `SC-006`)**:
   - Execute `ViewportLayoutCalculator`, `WindowChromeState`, `SlideShowService`, `ImageSessionCoordinator`, and `MainViewModel` tests headlessly with `IUiDispatcher` and `TimeProvider` without creating any `Window` instance, verifying all existing `MainWindow.xaml` bindings and shortcuts remain intact.
