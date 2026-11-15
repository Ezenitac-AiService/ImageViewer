# Contract: Modular Architecture, Concurrency Coordination & Headless Testability

**Feature**: `001-architecture-security-refactoring`
**Covers**: `FR-007`, `FR-008`, `FR-009`, `FR-010`

---

## 1. Extracted Domain & Orchestration Boundaries

| Module | Location | Responsibility | UI-Thread Dependency |
|---|---|---|---|
| `IUiDispatcher` | `Services/IUiDispatcher.cs` | Marshals property/collection updates to UI thread (`WpfUiDispatcher` in `Views/`, inline `TestUiDispatcher` in tests) | Abstracted |
| `ImageFormatHelper` | `Helpers/ImageFormatHelper.cs` | Binary streaming header/BOM inspection & blocked-format enforcement | None |
| `PathSecurity` / `PathRedactor` | `Helpers/PathSecurity.cs` | Canonical root checks, `ReparsePoint`/ZIP `Attrib` guards, space/CRLF-safe log redaction | None |
| `ViewportLayoutCalculator` | `Services/ViewportLayoutCalculator.cs` | Display size, zoom anchoring, pan clamping, placeholder display dimensions | None |
| `WindowChromeState` | `ViewModels/WindowChromeState.cs` | Pin/hover/fullscreen restore state transitions | None |
| `ImageSessionCoordinator` | `Services/ImageSessionCoordinator.cs` | Versioned open/load/render cancellation, placeholder timer (`TimeProvider`), `DecodedImage` ownership | `IUiDispatcher` + `TimeProvider` |
| `PrefetchCoordinator` | `Services/PrefetchCoordinator.cs` | Directional lookahead & slideshow next-index prefetching | None |
| `UpdateViewModel` | `ViewModels/UpdateViewModel.cs` | Auto-update state machine (`Checking`/`Available`/`Downloading`/`Installing`/`Failed`), safe `AutoCheckAsync` (`TimeProvider` delay), `IAppLifetime` handoff | `TimeProvider` (optional) |
| `MainViewModel` | `ViewModels/MainViewModel.cs` | Thin MVVM facade preserving all XAML bindings (including `Update` and `RelaunchPath`), `MainWindow.xaml.cs` calls, and `TimeProvider` timers | `IUiDispatcher` + `TimeProvider` |

---

## 2. XAML & Code-Behind Compatibility Invariant

All existing bindings in `src/ImageViewer/MainWindow.xaml` (including `UpdateNotice` `DataContext="{Binding Update}"` and `AboutWindow(AboutInfo.Create(), _vm.Update)`) and code-behind calls in `src/ImageViewer/MainWindow.xaml.cs` / `PlaceholderTests.cs` MUST remain signature-compatible:
- **Constructor & Commands (`MainViewModel`)**: Optional trailing `UpdateViewModel? update = null` constructor parameter, `OpenFileCommand`, `OpenFolderCommand`, `ExitCommand`, `PreviousCommand`, `NextCommand`, `FirstCommand`, `LastCommand`, `PreviousPageCommand`, `NextPageCommand`, `ZoomInCommand`, `ZoomOutCommand`, `ResetZoomCommand`, `RotateClockwiseCommand`, `RotateCounterClockwiseCommand`, `FlipHorizontalCommand`, `FlipVerticalCommand`, `ResetTransformCommand`, `ToggleInfoPanelCommand`, `ToggleMenusCommand`, `ToggleTopBarPinCommand`, `ToggleBottomBarPinCommand`, `ToggleThumbnailsCommand`, `TogglePlayAnimationsCommand`, `ToggleSlideShowCommand`, `ToggleFullScreenCommand`, `ShowImageListCommand`, `ShowSettingsCommand`, `SetDisplayModeCommand`, `SetFilterCommand`, `SetSlideShowModeCommand`.
- **Properties (`MainViewModel` & Code-Behind Members)**: `WindowTitle`, `CurrentBitmap`, `CurrentIndex`, `DisplayWidth`, `DisplayHeight`, `PositionText`, `ScaleText`, `ErrorMessage`, `OsdMessage`, `IsLoading`, `IsShowingPlaceholder`, `PlaceholderMessage`, `FrameInfoText`, `InfoItems`, `IsInfoPanelVisible`, `IsFullScreen`, `TopBarPinned`, `BottomBarPinned`, `BothBarsPinned`, `ThumbnailsVisible`, `ThumbnailItems`, `ThumbnailSelectedIndex`, `ThumbnailSize`, `ThumbnailScrollBarVisibility`, `DisplayMode`, `Filter`, `Background`, `MouseWheelAction`, `PlayAnimations`, `IsSlideShowPlaying`, `SlideShowButtonText`, `SlideShowInterval`, `SlideShowIntervalText`, `SlideShowMode`, `SlideShowRepeat`, `Settings`, `Update`, `RelaunchPath`, `LayoutChanged`, `PropertyChanged`, `OpenPathAsync`, `GoToAsync`, `UpdateViewport`, `SaveSettings`, `Dispose`, and forwarding members `PlaceholderDisplaySize` & `PlaceholderDelay` for existing unit tests.
- **Update Overlay / About Bindings (`UpdateViewModel`)**: `CurrentVersionText`, `KindText`, `AvailableVersionText`, `StatusText`, `IsBusy`, `IsDownloading`, `Progress`, `IsNoticeVisible`, `CanInstall`, `InstallButtonText`, `RelaunchPathProvider`, `AutoCheckAsync`, `CheckNowCommand`, `InstallCommand`, `SkipVersionCommand`, `DismissCommand`, `OpenReleasePageCommand`.
- **Item ViewModel / Model Bindings (`MainWindow.xaml` Templates)**: `MetadataItem` (`Label`, `Value`, `IsHeader`) and `ThumbnailItemViewModel` (`Name`, `Thumbnail`, `ToolTip`).

---

## 3. Concurrency & Deterministic Disposal Contract

- **OpenMP & Native Limits**: `App.OnStartup` sets `OMP_NUM_THREADS` and `MAGICK_THREAD_LIMIT` **before `ConfigureServices()` or any `ImageMagick` type initialization**, then invokes `MagickNET.Initialize()` followed by `ResourceLimits.Thread` (`Math.Clamp(Environment.ProcessorCount / 2, 1, 4)`), `ResourceLimits.Memory` (`1024UL * 1024 * 1024`), `Width`, `Height`, and `ListLength`.
- **Cancellation & No Sync-over-Async Blocking**:
  - `ThumbnailService` worker loops execute asynchronously (`await OpenSeekableAsync(...)`) without `.GetAwaiter().GetResult()`, passing valid `maxStreamBytes` and `CancellationToken`.
  - `ZipImageSource` implements both `IDisposable` and `IAsyncDisposable` (synchronously/safely signaling disposal so active `OpenImageAsync` / `GetImagesAsync` callers finish `finally { _gate.Release(); }` without throwing `ObjectDisposedException`, disposing `_gate` only once no waiters hold it or allowing safe GC cleanup without touching `AvailableWaitHandle`).
- **Complete `IDisposable` & Stream Ownership**:
  - `ImageRenderService` implements `IDisposable` and disposes `_backgroundGate`.
  - `ImageLoaderService.Dispose()` disposes `_backgroundGate` as well as clearing `_cache`.
  - `App.OnStartup` guards background startup calls (`UpdateInstaller.CleanupLeftoversAsync`, `UpdateViewModel.AutoCheckAsync`) against unobserved exceptions; `App.OnExit` invokes `_lifetime.RunPending(_logger)` **before** `_services?.Dispose()` so update execution logs flush safely before `FileLoggerProvider` shuts down.
  - `MainViewModel.Dispose()` stops/disposes all 6 `TimeProvider` timers (`_renderDebounce`, `_osdTimer`, `_loadingIndicatorTimer`, `_placeholderTimer`, `_animationTimer`, `_watchDebounce`), detaches watcher/slideshow events, cancels CTS tokens, and disposes owned services cleanly (note `MainWindow` detaches `_vm.LayoutChanged`/`_vm.PropertyChanged` and closes cleanly after `OnClosing` settings save, guarding `OnDrop` with `try/catch`).
  - `AnimationSource` owns the caller `using var memory = new MemoryStream(data, writable: false);` lifetime and transfers a bare `new SKManagedStream(memory, disposeManagedStream: false)` into `SKCodec.Create(...)` **without** wrapping `SKManagedStream` in its own `using` block (preventing Skia double-free).
