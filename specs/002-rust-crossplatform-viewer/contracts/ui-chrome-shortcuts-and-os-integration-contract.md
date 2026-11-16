# Contract: Cross-Platform UI Chrome, Shortcuts, Gestures & OS Integration

**Feature**: `002-rust-crossplatform-viewer`
**Covers**: `FR-002`, `FR-003`, `FR-007`, `FR-008`, `SC-004`

---

## 1. Frameless Window, Chrome & Platform Modifiers (`iv-app` + `iv-core`)

- **Event-Loop & UI Architecture**: `iv-app` directly drives `winit 0.30` (`ApplicationHandler`) paired with a custom **`wgpu` image/shader render pipeline** (explicitly requesting `max_texture_dimension_2d: min(adapter.limits().max_texture_dimension_2d, 16384)` and tiling oversized images) and **`egui-winit` / `egui-wgpu` overlay UI** across multiple independent native OS windows (`WindowId -> WindowSurfaceState`). Opaque frameless windows (`transparent: false`, `decorations: false`) avoid Windows DX12 transparency resize stalls; `WindowEvent::Resized` defers swapchain reconfiguration safely to the pre-render step (skipping zero-size minimizes); `WindowEvent::ScaleFactorChanged` updates surface texel resolution and `ViewportState::dpi_scale_factor` immediately across Retina, Windows DPI, and Wayland fractional scaling (`wp-fractional-scale-v1`); and native file/folder pickers use non-blocking **`rfd::AsyncFileDialog`** + `winit::event_loop::EventLoopProxy` (with macOS AppKit / Linux `xdg-desktop-portal` + `zenity` fallback) so event loops never deadlock. System Korean/English font loading ensures CJK UI & EXIF typography parity.
- **Native OS Hooks Exposed**: Direct `winit::ApplicationHandler` ownership captures macOS `objc2` `NSApplicationDelegate` `openURLs:`, native `WindowEvent::PinchGesture`, Wayland `drag_window()`, and multi-window drag-and-drop without framework lock-in.

- **Chrome Parity**:
  - Custom frameless titlebar (current filename, app title, minimize / maximize-restore / close buttons).
  - Auto-hiding translucent floating top icon menu bar and bottom action bubble (`zoom - / reset / + | prev · pos · next | slideshow · pin · thumbnails`).
  - Independent Top/Bottom Pin controls + `` ` `` global toggle (if either pinned -> unpin both; if both unpinned -> pin both); pinned bars stay visible even in fullscreen; unpinned bars reveal on top/bottom client-area edge hover.
  - Wayland-safe geometry: Window position `(left, top)` restored on Windows, macOS, and Linux X11; Wayland restores `width`, `height`, and `is_maximized` only, defers coordinates to the compositor, uses native `window::drag` on titlebar/canvas drag (outside zoom/fullscreen pan mode), and detects edge hover strictly inside client surface bounds.
- **Cross-Platform Shortcut Matrix**:
  - Unmodified single keys identical across Windows/macOS/Linux: `` ` `` (pin toggle), `←`/`↑`/`→`/`↓` (prev/next), `Home`/`End` (first/last), `PgUp`/`PgDn` (multi-page TIFF prev/next page or image prev/next), `R`/`L` (CW/CCW 90° rotate), `H`/`V` (flip H/V), `Tab` (EXIF panel), `Enter` (Image List dialog), `0`/`7`/`8`/`9` (Original/FitWidth/FitHeight/Fill), `U`/`I`/`S` (None/Smooth/SmoothSharpen), `+`/`=`/`-` (zoom), `Space` (slideshow play/pause), `Esc` (close active modal dialog, or close active viewer window; exits app when last window closes).
  - Primary modifier keys (`Cmd` on macOS, `Ctrl` on Windows/Linux): `<Mod>+O` (open file/archive), `<Mod>+Shift+O` (open folder), `<Mod>+0` (reset zoom), `<Mod>+Wheel` (invert configured wheel action).
  - Fullscreen toggle: `Alt+Enter` across all OSes, plus `Ctrl+Cmd+F` alias on macOS.
- **Mouse & Precision Trackpad Controls**:
  - Physical mouse wheel: default `Navigate` (prev/next) toggleable in Settings to `Zoom`; `Shift+Wheel` vertical pan; double-click maximize/restore; canvas drag window move (pans image when zoomed or fullscreen); middle-drag pans image; mouse Back/Forward buttons navigate prev/next.
  - Precision trackpad: Continuous smooth scroll accumulates pixel delta (`>= 48.0` px equivalent + short cooldown) before stepping in `Navigate` mode to prevent rapid multi-frame overshooting; native `Pinch` gesture drives smooth cursor-anchored zoom.

---

## 2. Secondary Dialogs, Localization & Visual Overlays

- **EXIF Info Panel (`Tab`)**: Displays non-empty `MetadataItem` fields (`Camera`, `Lens`, `Date Taken`, `Exposure Time`, `Aperture`, `ISO`, `Focal Length` + `35mm` equiv, `Exposure Bias`, `Flash`, `GPS`, `Software`, `Artist`, `Copyright`).
- **Animation / Multi-Page Overlay**: Bottom-left translucent badge (`frame n/m · status, elapsed/total · delay, WxH · format` or `page n/m, WxH · format` for multi-page TIFF) always reporting true post-orientation `source_width × source_height` even during reduced-resolution preview (`is_downscaled`).
- **Native OS Child Dialogs (`ImageList`, `Settings`, `About`)**: Rendered as **separate native OS child windows** (`winit` multi-window `WindowId` + own `wgpu::Surface` + `egui_winit::State`, matching WPF `ShowDialog` behavior without clipping when the main viewer window is small):
  - **Image List Dialog (`Enter`)**: Virtualized/scrollable top thumbnail strip + sortable list (`Name` natural, `Modified Date`, `File Size`; unsupported entries grayed out; active item highlighted).
  - **Settings Dialog**: Display (`Background`, `MouseWheel`), Navigation & Slideshow (`WrapAround`, `WatchFolder`, `SlideShowRepeat`), Thumbnail (`Visible`, `ShowScrollBar`, `Size 48-256`), Filter (`SmoothSigma 0.1-2.0`, `SharpenSigma 0.1-3.0`, `SharpenAmount 0.1-3.0`), Safety Limit (`MaxImageMegapixels 10-500 MP`), Updates (`AutoCheck`), OS Integration section (Windows Explorer context menu + Default Apps button; macOS/Linux OS-appropriate file association guide/action).
  - **About Dialog & Copy**: Displays app version, runtime OS/arch (`x64`/`arm64`), dynamically populated third-party Rust/OS library versions, licenses, project URL, and clipboard Copy button + manual Check Now update trigger.
- **Update Notice Overlay**: Top-right floating overlay with Release Notes link, Skip Version, Dismiss, Download ProgressBar, and Install / Open Release Page action.
- **Localization**: Bilingual `ko` and `en` UI message tables complete across all 3 OSes.

---

## 3. OS-Idiomatic Integration, Packaging & Updates (`iv-platform`)

- **Windows 10/11 (x64; ARM64 target-clean)**:
  - HKCU Explorer verbs (`Directory\shell\ImageViewer`, `Directory\Background\shell\ImageViewer`, `SystemFileAssociations\<ext>\shell\ImageViewer`, `MultiSelectModel=Single`, exact trimmed command equality in `IsRegistered()`, quote/control-char rejection on `exePath`, `--register-shell` / `--unregister-shell`; preserves `v1.0.6` Windows 11 classic "Show more options" placement without requiring MSIX Sparse Package identity in v1).
  - HKCU Default Apps (`ImageViewer.Image`, `ImageViewer.Archive`, `OpenWithProgids`, `Capabilities`, `RegisteredApplications`, `ms-settings:defaultapps?registeredAppUser=ImageViewer`).
  - Updates: NSIS silent installer (`setup.exe /S /RELAUNCH=<sanitized_path>`) and Portable self-swap (`.exe -> .old`, `.new -> .exe`, cleanup `.old` on startup).
- **macOS (Apple Silicon `aarch64` & Intel `x86_64`)**:
  - `.app` bundle `Info.plist` (`CFBundleDocumentTypes` covering supported extensions + ZIP/CBZ), Finder Open With / default viewer compatibility, custom `objc2` `NSApplicationDelegate` catching `openURLs:` events to launch dedicated viewer windows cleanly.
  - Updates: SHA-256-verified `.dmg`/archive download -> open installer/folder cleanly (or fallback to verified HTTPS release page).
- **Linux (`x86_64`, X11 & Wayland)**:
  - User-scope XDG `.desktop` entry (`~/.local/share/applications/`) + `MimeType=` declarations + `xdg-mime` association registration in user scope (no root elevation).
  - Updates: SHA-256-verified AppImage/package archive download -> open downloaded artifact/folder (or fallback to verified HTTPS release page).
