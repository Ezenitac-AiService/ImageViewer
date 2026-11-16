# Phase 0 Research: Cross-Platform Rust ImageViewer Parity & Performance Upgrade

**Feature**: `002-rust-crossplatform-viewer`
**Date**: 2026-11-16
**Spec**: [spec.md](spec.md)
**Constitution**: [constitution.md](../../.specify/memory/constitution.md) (`v2.0.0`)

---

## R1. Cross-Platform GPU-Accelerated GUI & `winit 0.30` Architecture (`wgpu` + `egui-winit`/`egui-wgpu`)

- **Decision**: Build `crates/iv-app` around an **owned `winit 0.30` `ApplicationHandler` event loop paired with custom `wgpu` canvas/shader rendering and `egui-winit` / `egui-wgpu` for frameless floating overlays and secondary dialogs** (DX12 / Metal / Vulkan) — cleanly separated from UI-independent crates (`iv-core`, `iv-media`, `iv-platform`).
- **Rationale**:
  1. **Clean `winit 0.30` Multi-Window & Event Ownership**: Unlike `iced::run` (which blocks `PinchGesture` and `NSApplicationDelegate`) and low-level `iced_winit`+`iced_wgpu` (which introduces heavy multi-window `wgpu::Surface<'static>`/`UserInterface` lifetime friction), `egui-winit` + `egui-wgpu` is architected specifically to compose cleanly inside an external `winit::ApplicationHandler` alongside a custom `wgpu` image renderer. This gives clean first-class support for macOS `objc2` `NSApplicationDelegate` (`openURLs:`), native `WindowEvent::PinchGesture`, Wayland `window.drag_window()`, and OS system CJK/Korean font rendering.
  2. **Opaque Frameless Windows, Multi-Window OS Sub-Dialogs & Large Texture Tiling**: Match C# `MainWindow.xaml`'s opaque frameless chrome (`transparent: false`, `decorations: false`) to prevent Windows DX12 transparent-window resize black/white flashes and stalls; defer `wgpu::Surface::configure` cleanly to pre-render on `WindowEvent::Resized` (skipping zero-dimension minimizes); handle `WindowEvent::ScaleFactorChanged` across Retina, Windows DPI, and Wayland `wp-fractional-scale-v1`; and render secondary dialogs (`Settings`, `ImageList`, `About`) as **separate native OS sub-windows** (independent `winit` `WindowId` + `wgpu::Surface` + `egui_winit::State`) so dialogs never clip when the main viewer window is small. On `wgpu::Device` initialization, explicitly request `wgpu::Limits { max_texture_dimension_2d: adapter.limits().max_texture_dimension_2d.min(16384), .. }` (avoiding the default `8,192` limit), and automatically split images exceeding `device.limits().max_texture_dimension_2d` (such as tall webtoons or wide panoramas) into tiled sub-textures (`<= max_dim`) with split UV quads (or SIMD viewport downscaling when zoomed out).
  3. **Zero-Readback GPU Shaders (`Smooth`/`SmoothSharpen`) & `>= 60%` Scaling**: Keeping decoded/viewport textures resident in `wgpu` lets the image renderer execute `>= 60%` bilinear/high-quality downscaling, checkerboard backgrounds, and screen-pixel separable Gaussian Blur / Unsharp Mask shaders directly on the GPU without PCIe readbacks (`R2`).
- **Alternatives Considered**:
  - *Standard closed `iced::run` or custom `iced_winit` + `iced_wgpu`*: Rejected after verifying missing `PinchGesture`/`openURLs:` support (`iced::run`) and excessive multi-window `Surface<'static>`/scheduler coupling (`iced_winit`).
  - *`Slint` / `Tauri`*: Rejected due to commercial/GPLv3 dual-license friction (`Slint`) and JS/WebView2 buffer-bridge latency (`Tauri`).
- **Sources**: `winit 0.30` `ApplicationHandler` & `iced_winit`/`iced_wgpu` multi-window friction docs; Windows DX12 transparency resize artifacts & deferred `surface.configure` patterns; Wayland `wp-fractional-scale-v1` & `ScaleFactorChanged` guidance; `wgpu` shader vs CPU readback benchmarks; host Rust toolchain (`rustc 1.100.0`).

---

## R2. Memory-Safe Core Decoders, SIMD Resize (`fast_image_resize`), and GPU Filter Shaders

- **Decision**:
  1. Standard formats decode through **pure memory-safe Rust** core decoders in `crates/iv-media` covering all 39 non-OS extensions cleanly: `zune-jpeg` (`opt-level = 3` in dev), `image` `0.25.4+` (JPEG/PNG-APNG/GIF/BMP/TIFF/WebP/ICO/TGA/QOI/PNM/HDR, plus native **`ImageDecoder::orientation()` + `DynamicImage::apply_orientation()`** matching `ImageLoaderService.cs` `AutoOrient()` and `Orientation 5–8` width/height swapping, applied **per page** on multi-page TIFFs including `16-bit`/CMYK -> `moxcms`), `image-extras` (`.pcx`), `image_dds` (BC1–BC7 `.dds`), **`rawpsd`** (8-bit RGB, CMYK, Grayscale `.psd` composite data paired with `moxcms`), `jxl-oxide` (`.jxl`), `exr` (`.exr`), `image-webp` / `zenwebp` (animated WebP), and a lightweight checked pure-Rust `.cur` parser (`ICONDIR` `type == 2`).
  2. Viewport-scaled pre-decoding uses **`fast_image_resize`** (AVX2/SSE4.1/NEON SIMD scaling, NOT general Gaussian blur) alongside bounded `rayon` concurrency while retaining original post-orientation `source_width`/`source_height` separately from `buffer_width`/`buffer_height`, swapping asynchronously to full resolution (and GPU tile grids when above `max_texture_dimension_2d`) when zooming past the cached physical display size (`Clarifications 2026-11-16`).
  3. **`Smooth` / `SmoothSharpen` Display Filters**: Compute filter kernel parameters (`sigma / display_scale`, skipping `< 0.05`) in pure-Rust `iv-core`/`iv-media` math helpers and execute separable Gaussian blur + Unsharp Mask in **`wgpu` GPU shaders** on the already-resident texture (eliminating CPU re-filtering + PCIe re-upload stalls during zoom changes), cleanly bypassed on animated playback (`R1`).
  4. Animated WebP/GIF/APNG frames stream incrementally (`<= 512 MB` pre-decode vs `<= 128 MB` forward ring buffer), preserving `v1.0.6` wall-clock delay normalization (`GIF <= 10 ms -> 100 ms`; `WebP` normalized to `100 ms` only when *all* frames are `<= 10 ms`) and implementing full **APNG** `DisposeOp` (`None`/`Background`/`Previous`), `BlendOp` (`Source`/`Over`), and `delay_num / delay_den` (`delay_den == 0 -> 100`, `<= 10 ms -> 100 ms`) frame compositing.
- **Rationale**:
  - Audits of `image 0.25` showed explicit crate additions (`image-extras`, `psd`/`rawpsd`, `image_dds`, lightweight `.cur`) are required for true 100% extension parity, and `fast_image_resize` performs strictly resampling rather than Gaussian convolution.
  - Keeping blur/unsharp filtering on GPU `wgpu` shaders directly avoids the CPU-GPU PCIe readback penalty while maintaining `60+ FPS` smooth zoom responsiveness.
- **Alternatives Considered**:
  - *CPU `image::fast_blur` / `libblur`*: Viable fallback for headless batch rendering, but recalculating CPU Gaussian blur + re-uploading textures on every interactive zoom change hurts zoom fluidity compared to `wgpu` shaders.
  - *`libjpeg-turbo` / `libwebp-sys` C-FFI*: Rejected for v1 baseline to avoid C/Assembly CVE exposure (`CVE-2023-4863`) unless benchmarks fail `SC-001`–`SC-003`.
- **Sources**: `image 0.25` supported-format docs & `image-extras` / `image_dds` (`docs.rs`); `zune-jpeg` `#434` docs; `fast_image_resize` vs `imageproc`/`fast_blur` vs `wgpu` shader benchmarks; `zenwebp` (`#![forbid(unsafe_code)]`) strip streaming docs; `README.md` (`v1.0.6`).

---

## R3. Pure-Rust Color Management (`moxcms`) and Lightweight HDR Tone Mapping

- **Decision**:
  1. Use **`moxcms`** (pure Rust ICC v2/v4 engine supporting Display P3, Adobe RGB, Lab, and CMYK separation/LUT transforms) to convert non-sRGB or CMYK embedded profiles into sRGB during decode, taking a zero-cost pass-through path for untagged or standard sRGB images and catching malformed ICC payloads without crashing (`Clarifications Q2/Q4`).
  2. For floating-point HDR inputs (`.exr`, `.hdr`), apply lightweight parallelizable auto-exposure / Reinhard-style soft tone mapping followed by sRGB gamma encoding; normalize `16-bit` integer channels consistently into display-ready buffers so `Smooth`/`Smooth+Sharpen` filters execute in consistent sRGB space scaled by display pixels (`sigma / display_scale`).
- **Rationale**: `moxcms` provides full ICC v4 and CMYK LUT coverage without C FFI risks (`lcms2`) or legacy `qcms` CMYK/v4 omissions. Fast pass-through for untagged/sRGB preserves `<= 36 ms` single-frame latency on 99% of common web/camera photos.
- **Alternatives Considered**:
  - *`lcms2` (`rust-lcms2`)*: C dependency with FFI thread-context boilerplate and potential C parser CVEs.
  - *`qcms`*: Lacks comprehensive CMYK and complex ICC v4 LUT profiles.
- **Sources**: `moxcms` vs `lcms2` vs `qcms` safety/feature/performance comparison (`crates.io`, `docs.rs`, `littlecms.com`).

---

## R4. Hybrid Host OS Platform Codecs (`WIC`, `ImageIO`, Linux Fallback) Under Zero Trust

- **Decision**: Define the narrow `OsCodecProvider` trait and `OsDecodedBuffer { width, height, rgba8, orientation_applied, srgb_converted, exif_bytes }` DTO in **`crates/iv-core`** (`#![forbid(unsafe_code)]`, keeping the Cargo workspace strictly an acyclic DAG: `iv-media -> iv-core`, `iv-platform -> iv-core`, `iv-app -> all`), and implement platform adapters in `crates/iv-platform` to delegate camera RAW (`.dng`, `.cr2`, `.cr3`, `.nef`, `.arw`, `.orf`, `.rw2`, `.raf`), `.heic`/`.heif`, `.avif`, and `.jp2` (`iv-media` executes EXIF `apply_orientation` / `moxcms` ICC->sRGB ONLY when `!orientation_applied` / `!srgb_converted` so OS codecs never double-rotate or double-gamma):
  - **Windows**: `windows` crate -> COM `IWICImagingFactory` (`CreateDecoderFromStream` + `IWICFormatConverter` to `32bppPBGRA`/`RGBA8`) with per-thread STA/MTA COM guard and strict `// SAFETY:` blocks.
  - **macOS**: `objc2-image-io` / `objc2-core-graphics` (`CGImageSourceCreateWithData` with `kCGImageSourceShouldCache: NO`) in isolated `unsafe` adapter copying raw pixels safely into owned `Vec<u8>`.
  - **Linux**: Probe optional system shared codecs safely via isolated loader interface (if absent or unverified, fail closed with localized missing-codec message without crashing).
  - **Mandatory Pre-Allocation Guard**: BEFORE calling any OS codec decoder to materialize pixels, `crates/iv-media` runs the `8,192`-byte multi-BOM script/document header rejection check, `512 MB` stream cap, and metadata dimension/pixel-limit query (`checked_mul` stride + `<= MaxImageMegapixels`). If upfront dimensions cannot be obtained cleanly, fail closed without allocating pixel buffers.
- **Rationale**: Respects `FR-005` / `FR-008` and `Constitution Principle IV/VII`: avoids bundling patent-encumbered/C++ HEIC/RAW libraries everywhere while isolating required `unsafe` FFI cleanly behind safe Rust ownership boundaries.
- **Alternatives Considered**:
  - *Bundling `libheif` + `libraw` + `dav1d` globally*: Rejected per clarified hybrid policy (`Q2=B`) due to binary bloat and C++ FFI build complexity across 3 OSes.
- **Sources**: Microsoft `windows` crate `IWICImagingFactory` / `CreateBitmapFromWicBitmap` documentation; Apple `ImageIO` (`CGImageSourceCreateWithData`, `objc2-image-io`, `kCGImageSourceShouldCache`) safety patterns.

---

## R5. Zero Trust Archive (`zip`), Header Guards, and Path Containment

- **Decision**:
  1. Centralize path canonicalization (`canonical_base` ending in platform separator + case-sensitivity-aware containment matching OS filesystem semantics), archive key normalization (`\` -> `/`, rejecting `..`, leading `/`, drive prefixes, null bytes, and POSIX `S_IFLNK` `(mode >> 16) & 0xF000 == 0xA000` / DOS `0x0400`), and immediate symlink metadata checks (`std::fs::symlink_metadata`) inside `crates/iv-core` (`#![forbid(unsafe_code)]`).
  2. Implement a **pre-flight zero-allocation EOCD / ZIP64 EOCD trailer parser** (checking container length `<= 4 GB`, EOCD signature `0x06054b50` / ZIP64 locator, Central Directory size `<= 64 MB`, and total declared entries `<= 50,000`) **strictly BEFORE invoking `zip::ZipArchive::new`**, because `zip::ZipArchive::new` eagerly parses and allocates the Central Directory in RAM (`CVE-2026-0429` class OOM risk). Wrap entry decompression streams in `Read::take(max_entry_bytes + 1)` (`MaxZipEntryMegabytes` default `512 MB`) and reject encrypted entries cleanly (`Err(EncryptedArchiveEntry)`).
  3. Port and preserve `8,192`-byte binary header sniffing (UTF-32 LE/BE `FF FE 00 00` / `00 00 FE FF` checked before 2-byte UTF-16 LE/BE and UTF-8 BOMs, BOM-less UTF-16 heuristics `<\0`/`\0<`, `>= 512`-byte pure-whitespace header rejection, and case-insensitive `<`, `<?xml`, `<svg`, `<html`, `%!`, `%PDF`, `push `, `viewbox` prefixes) prior to **both** core and OS-codec pipelines.
- **Rationale**: Prevents Zip-Slip (CWE-22), zip-bomb/tiny-entry OOM exhaustion (CWE-409/CWE-400), and polyglot script smuggling (SVG/MVG/PDF) even when headers are spoofed with large UTF-16 whitespace padding.
- **Alternatives Considered**:
  - *Reading entire `ZipArchive` entries via `read_to_end` uncapped*: Rejected due to deceptive header decompression bombs.
- **Sources**: Rust `zip` crate `enclosed_name` & POSIX permissions (`external_attributes`) advisories; OWASP CWE-22 / CWE-409 mitigation guidance; `specs/001-*` security baseline audit.

---

## R6. EXIF Metadata (`kamadak-exif`), Natural Sort, and Folder Watching (`notify-debouncer-mini`)

- **Decision**:
  - **EXIF**: Use pure-Rust **`kamadak-exif`** (`exif` crate) inside `crates/iv-media` to extract all 13 documented fields (`Make`/`Model` camera join, `LensModel`, `DateTimeOriginal`/`DateTime`, `ExposureTime`, `FNumber`, `ISOSpeedRatings`, `FocalLength` + `35mm` equiv, `ExposureBiasValue`, `Flash`, `GPSLatitude`/`GPSLongitude` DMS->decimal, `Software`, `Artist`, `Copyright`), failing silently on corrupt tags and returning available fields matching `ExifReader.cs`.
  - **Folder Watch**: Use **`notify-debouncer-mini`** (`notify` crate) configured with a `700 ms` debounce window (off for ZIPs and toggleable via `Navigation.WatchFolder`), filtering symlink/reparse entries via `symlink_metadata` so watching preserves `v1.0.6` rules (updating list automatically and keeping selection index if the active image is deleted).
  - **Natural Sort**: Implement an allocation-light ASCII/Unicode numeric-chunk comparator matching Windows `image2 < image10` natural sorting consistently across Windows, macOS, and Linux.
- **Rationale**: Avoids invoking heavier C metadata tools while maintaining exact parity across EXIF formats (JPEG, TIFF, PNG, WebP, HEIF, AVIF).
- **Sources**: `kamadak-exif` (`github.com/kamadak/exif-rs`) supported formats docs; `notify-debouncer-mini` docs; direct audit of `src/ImageViewer/Services/ExifReader.cs` and `README.md`.

---

## R7. Cross-Platform Windowing, Wayland Quirks, Shortcuts, and Multi-Instance Compatibility

- **Decision**:
  1. **Process & Concurrent Settings Model**: Preserve multi-instance opening (`each CLI target opens its own viewer window`), **while preventing multi-instance `settings.json` Lost Updates & corruption** via OS advisory file locking (`fs4`) + **re-reading latest disk state before selectively merging dirty fields** (Preferences saved on dialog OK / `SkipVersion`; `Window` geometry fields merged on close without clobbering sibling windows' settings) using randomized temp atomic rename (`settings.<rand>.tmp`).
  2. **Wayland & Multi-Monitor HiDPI Compatibility (`FR-002`, `FR-007`)**: Persist `Window.Left` / `Window.Top` / `Width` / `Height` / `IsMaximized`; apply `(Left, Top)` on Windows, macOS, and Linux X11; under **Linux Wayland** restore `Width`/`Height`/`IsMaximized` only, defer positioning to the compositor, invoke `window.drag_window()` on pointer serial inside titlebar/canvas drag, compute edge hover relative to client surface bounds, and update `ViewportState::dpi_scale_factor` immediately on `WindowEvent::ScaleFactorChanged` across Retina/Windows DPI/Wayland `wp-fractional-scale-v1` (matching `MainWindow.xaml.cs` `OnDpiChanged`).
  3. **macOS & Windows Shell Integration (`FR-005`, `FR-008`)**: Ignore macOS LaunchServices `-psn_*` session flags cleanly in `CliOptions::parse` (still fail closed on `>= 2` paths or unknown flags, and echo user path only on `IsPathNotFoundError`); use our **owned `winit 0.30` `ApplicationHandler` + `egui-winit`/`egui-wgpu` loop (`R1`)** paired with an `objc2-app-kit` `NSApplicationDelegate` (`application:openURLs:`) hook registered at startup to route Finder file-open events cleanly. Windows preserves `v1.0.6` zero-elevation HKCU Explorer verbs (visible under Windows 11 classic "Show more options" / `Shift+F10`; `IExplorerCommand` + MSIX Sparse Package excluded from v1 portable parity).
  4. **Trackpad & Modifiers (`FR-002`)**: Map primary shortcuts to `Cmd` (`macOS`) vs `Ctrl` (`Windows`/`Linux`), support both `Alt+Enter` and `Ctrl+Cmd+F` fullscreen on macOS, close dialogs then active window on `Esc` (exit when empty), handle `winit::event::WindowEvent::PinchGesture` directly inside the `winit::ApplicationHandler` shell for cursor-anchored zoom, and apply an accumulated pixel-delta threshold (`>= 48.0` logical px equivalent per step + short cooldown) on precision smooth-scroll devices when wheel is configured to `Navigate`.
  5. **Windows ARM64 Future-Readiness**: Default release focus remains Windows `x64`, macOS (`aarch64` + `x86_64`), and Linux (`x86_64` X11/Wayland); all Rust crates avoid `x86_64`-only assumptions (`aarch64-pc-windows-msvc` builds cleanly as an optional CI expansion).
- **Rationale**: Directly resolves GNOME Wayland `set_outer_position` `NotSupportedError`, MacBook trackpad overshooting, Finder `-psn_*` rejection traps, and concurrent `settings.json` clobbering.
- **Sources**: `winit` Wayland `set_outer_position`/`drag_window` & GNOME Mutter `libdecor` docs; `objc2-app-kit` `NSApplicationDelegate` `openURLs` integration patterns; `fs4` cross-platform locking; audit of `SettingsService.cs`.

---

## R8. Settings Backward Compatibility, Privacy Logging, Localization, and Secure Updates

- **Decision**:
  1. **Settings (`settings.json`)**: `serde` + `serde_json` schema matching `v1.0.6` `SettingsService.JsonOptions` (`SettingsService.cs:15-22`: `camelCase` field names, string enum serialization e.g. `"Fill"`, `"Black"`, `"Sequential"`, `v1–v3` schema migration, comment/trailing-comma tolerant parsing, `<= 65,536` byte file cap, `symlink_metadata` check on immediate app config dir + file, and finite/clamped recovery of `NaN`/`Infinity` and `Update.SkippedVersion`). Stored in idiomatic `directories::ProjectDirs` / `LocalAppData` (`ImageViewer/settings.json`).
  2. **Diagnostics & Redaction**: Pure-Rust `PathRedactor` stripping quoted/space-containing Windows, UNC, and Unix directory paths down to filename-only (`<redacted>/filename` or filename) and stripping `\r`/`\n`/control chars (CWE-117), writing via a bounded async/sync channel logger (`DropOldest`, 7-day daily log file rotation, `2s` bounded drain on exit, `>= 300 ms` slow-render trace logging).
  3. **Localization & Non-Blocking Dialogs**: Compile-time/embedded fluent or typed **`fluent` / `ko` + `en`** localization table preserving every existing Korean and English label/hotkey tooltip; native file/folder pickers via **`rfd::AsyncFileDialog`** dispatched asynchronously to `winit::event_loop::EventLoopProxy` (non-blocking across macOS AppKit `NSOpenPanel` and Linux `xdg-desktop-portal` + `zenity` fallback, avoiding event-loop deadlocks), paired with independent OS child windows (`Settings`, `ImageList`, `About`).
  4. **Auto-Updater (`reqwest` with `rustls-tls` + `sha2`)**: Unconditional `https://` enforcement across `FeedBase`, `ReleaseInfo.page_url`, `SHA256SUMS.txt`, asset `download_url`, and release-page launching (zero `#[cfg(debug_assertions)]`, `danger_accept_invalid_certs`, or env bypasses). Define a narrow `HttpFeedTransport` trait / `UpdateClient::with_root_certs(...)` constructor accepting a custom `rustls::RootCertStore` (defaulting to OS/WebPKI roots in production, and accepting an ephemeral `rcgen` self-signed loopback CA in offline `cargo test` runs). Enforce bounded HTTP streams (`1 MB` release JSON, `64 KB` sums, `256 MB` installer/package), strict SHA-256 verification before activating `.part` files, redacted update logs/errors, Windows NSIS `/S /RELAUNCH=` (strict char sanitization) & Portable `.old`/`.new` swap, and macOS (`.dmg`) / Linux (`.AppImage`/package) verified artifact download + launch/open-folder fallback.
- **Rationale**: Preserves 100% upgrade continuity for existing `v1.0.6` Windows users while delivering identical memory-safe TLS and path-privacy guarantees across all 3 platforms.
- **Sources**: `reqwest` `rustls-tls` + `sha2` docs; `directories` & `rfd` & `fluent` crate docs; direct audit of `SettingsService.cs`, `ViewerSettings.cs`, `UpdateService.cs`, and `UpdateInstaller.cs`.
