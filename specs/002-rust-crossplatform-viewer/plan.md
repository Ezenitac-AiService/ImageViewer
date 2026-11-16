# Implementation Plan: Cross-Platform Rust ImageViewer Parity & Performance Upgrade

**Branch**: `002-rust-crossplatform-viewer` | **Date**: 2026-11-16 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-rust-crossplatform-viewer/spec.md`

## Summary

Re-architect **ImageViewer** as a high-performance, Zero Trust, cross-platform Rust Cargo workspace targeting **Windows 10/11 (x64)**, **macOS (Apple Silicon `aarch64` & Intel `x86_64`)**, and **Linux (`x86_64`, X11 & Wayland)**:
1. **UI-Independent Core & Security (`crates/iv-core`)**: Zero-`unsafe` pure-Rust domain models, backward-compatible `v1.0.6` `settings.json` loader/saver (`camelCase`, string enums, v1–v3 migrations including `SmoothSigma 0.8 -> 0.5`, `64 KB` cap, `fs4` lock + selective-merge save preventing multi-instance Lost Updates), `PathSecurity` / `PathRedactor` (CWE-22/117/532), `8,192`-byte multi-BOM binary sniffer, fail-closed `CliOptions::parse` (ignoring macOS `-psn_*`), DPI-aware viewport/chrome/slideshow/trackpad state machines (`Clock` + monotonic generations), bilingual `ko`/`en` resources, and bounded non-blocking file logger.
2. **SIMD Media & Bounded Streaming Engine (`crates/iv-media`)**: Memory-safe core decoders (`zune-jpeg`, `image` `0.25.4+`, `image-extras` for `.pcx`, `image_dds` for `.dds`, `rawpsd` for RGB/CMYK/Gray PSD, `jxl-oxide`, `exr`, `zenwebp`/`image-webp`, pure-Rust `.cur` parser, `zip` guarded by pre-flight zero-allocation EOCD/ZIP64 EOCD scan strictly before `ZipArchive::new`, `kamadak-exif`) enforcing EXIF `Orientation` (`1–8`, swapped width/height on `5–8`, `AutoOrient` parity), paired with `fast_image_resize` (AVX2/SSE4.1/NEON SIMD viewport scaling), `moxcms` (non-sRGB/CMYK -> sRGB ICC conversion + soft HDR tone mapping), filter radius calculation (`sigma / display_scale`) feeding zero-readback **`wgpu` shader blur/unsharp passes** in `iv-app`, lazy animation ring buffering (`<= 512 MB` / `<= 128 MB`), dual-lane `rayon` concurrency, `notify-debouncer-mini` (`0.7 s` folder watch), and pre-decode dimension/archive-bomb caps (`150 MP`, `<= 50,000` ZIP entries, `<= 64 MB` CD, `<= 4 GB` container, `<= 512 MB` entry).
3. **OS Adapters, Platform Codecs & `winit 0.30` + `wgpu` + `egui` UI (`crates/iv-platform`, `crates/iv-app`)**: Narrow `OsCodecProvider` delegating RAW/HEIC/AVIF/JP2 cleanly to Windows `WIC`, macOS `ImageIO`, or optional Linux system codecs after passing `iv-core`/`iv-media` Zero Trust pre-checks; `reqwest` (`rustls-tls`) + `sha2` unconditional HTTPS updater with `HttpFeedTransport` / custom root-cert test seam and OS-idiomatic installation/associations; and an **owned `winit 0.30` `ApplicationHandler` + custom `wgpu` canvas + `egui-winit`/`egui-wgpu` overlay UI** using opaque frameless windows (`transparent: false`, deferred resize swapchain config, `ScaleFactorChanged` DPI handling, native macOS `objc2` `openURLs:` delegate, `WindowEvent::PinchGesture`, Wayland `drag_window()`, GPU downscaling + `Smooth`/`SmoothSharpen` shaders, and all `v1.0.6` dialogs and shortcuts).

## Technical Context

**Language/Version**: Stable Rust (`edition = "2024"`, MSRV `>= 1.85`; verified host `rustc 1.100.0` / `cargo 1.100.0`), strict `clippy` lints, `[profile.dev.package.*] opt-level = 3` for SIMD codecs/resizers

**Primary Dependencies**:
- **GUI & GPU**: Owned `winit 0.30` `ApplicationHandler` + custom `wgpu` canvas renderer + `egui` / `egui-winit` / `egui-wgpu` overlay UI (DX12 / Metal / Vulkan)
- **Core Media, SIMD, Color, EXIF, Archives**: `zune-jpeg`, `image` (`0.25.4+` EXIF `apply_orientation`), `image-extras` (`.pcx`), `image_dds` (`.dds`), `rawpsd` (RGB/CMYK/Gray `.psd`), `jxl-oxide`, `exr`, `image-webp` / `zenwebp`, pure-Rust `.cur` parser, `fast_image_resize` (`rayon`), `moxcms`, `kamadak-exif`, `zip` (with custom pre-flight EOCD/ZIP64 EOCD reader)
- **OS, Watching, Pickers, Network, Logging**: `serde` / `serde_json`, `fs4`, `directories`, `notify-debouncer-mini`, `rfd`, `reqwest` (`rustls-tls`) + `rustls` root-cert store option (and `rcgen` dev-dependency for offline loopback TLS tests), `sha2`, `fluent` / `ko`+`en` catalog; platform crates gated via `#[cfg(target_os)]`: `windows` + `winreg` (Windows WIC/HKCU/Shell), `objc2` + `objc2-image-io` + `objc2-app-kit` (macOS ImageIO/Finder `openURLs:` delegate)

**Storage**: Per-user OS app data dir (`settings.json <= 64 KB`, 7-day rotated diagnostic logs) via `directories::ProjectDirs` / `%LOCALAPPDATA%\ImageViewer` (Windows compatibility), read-only local folders and ZIP/CBZ archives

**Testing**: Cargo built-in `cargo test --workspace` (unit + real-fixture integration suites without mocking frameworks or `#[ignore]`) + `criterion` performance benchmark suite (`SC-001`–`SC-003`)

**Target Platform**: Cross-platform desktop — **Windows 10/11 (x64; ARM64 target-clean)**, **macOS (Apple Silicon `aarch64` & Intel `x86_64`)**, and **Linux (`x86_64`, X11 & Wayland)**

**Project Type**: Multi-crate Rust Cargo workspace desktop application + OS-specific installers/CI (`.github/workflows/ci.yml`, Windows NSIS, macOS `.app`/`.dmg`, Linux `.desktop`/AppImage)

**Performance Goals**: Match or beat `v1.0.6`: `<= 36 ms` avg (`<= 95 ms` max) at `1280×800` Fit Width, `<= 49 ms` avg (`<= 137 ms` max) at `2560×1440` fullscreen, `<= 22 ms` avg rapid key navigation, `<= 16 ms` avg slideshow transition, `< 250 ms` animation frame 1, `<= 470 MB` RAM & `<= 5%` CPU on `1080×1920` 267-frame WebP

**Constraints**: Zero breaking regressions to shortcuts, display modes, screen-scale filters, EXIF fields, `v1.0.6` `settings.json` compatibility, or `ko`/`en` UI; Zero Trust caps (`150 MP` default, `512 MB` entry cap, `50,000` ZIP entries, `64 MB` CD, `4 GB` archive, `64 KB` settings); Wayland coordinate limitations; unconditional `https://` + SHA-256 updates without debug bypasses

**Scale/Scope**: 4 workspace crates (`iv-core`, `iv-media`, `iv-platform`, `iv-app`), OS packaging/installer scripts, multi-OS CI workflow; existing C# `src/` and `specs/001-*` preserved strictly as reference baseline until cutover

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Pre-Research Gate | Post-Design Verification |
|---|---|---|
| **I. Korean Communication & Commit Policy** | All clarifications (`spec.md`) and user reports delivered in Korean. | **PASS** — Plan responses and commit suggestions remain Korean; code, crates, docs remain English. |
| **II. English-First Reasoning & Implementation** | Workspace crates, modules, contracts, and tests designed with English identifiers and rustdoc; `ko`/`en` resources preserved. | **PASS** — All workspace modules, data models, and contracts (`contracts/*`) are English; `iv-core::i18n` covers full `ko`/`en` parity. |
| **III. Test-Driven Development (NON-NEGOTIABLE)** | Cargo workspace splits pure `iv-core` and `iv-media` for fast headless `cargo test` Red-Green-Refactor cycles without `#[ignore]`. | **PASS** — Layout, trackpad, CLI, security, archive, decode, ICC/HDR, and coordinator suites run headlessly via real fixtures and injected `Clock`. |
| **IV. Zero Hardcoding, Mocking, or Test-Bypass Logic (NON-NEGOTIABLE)** | Core crates target zero `unsafe`; zero `#[cfg(test)]`/`#[cfg(debug_assertions)]` security switches; narrow OS traits (`OsCodecProvider`, `OsIntegration`) only for non-headless OS calls. | **PASS** — No mocking frameworks, no HTTP/debug feed bypasses (`UpdateClient` uses injected transport/local HTTPS), and `unsafe` strictly isolated to `WIC`/`ImageIO`/AppKit adapters with `// SAFETY:` comments. |
| **V. Evidence-Based Research & Verification (NON-NEGOTIABLE)** | Phase 0 `research.md` (`R1`–`R8`) cites retrieved docs/benchmarks (`iced`, `winit`/Wayland, `zune-jpeg`, `fast_image_resize`, `image-webp`/`zenwebp`, `moxcms`, `zip`, `objc2`, `windows`) and codebase audits. | **PASS** — Every library selection, Wayland compromise, and host-OS verification boundary traces directly to retrieved evidence. |
| **VI. Skeptical Verification & Fallibility Awareness (NON-NEGOTIABLE)** | Addresses Wayland positioning limits, trackpad overshooting, multi-instance settings locking, missing Linux OS codecs, and cross-OS testing disclosure explicitly. | **PASS** — Failure modes, fallbacks, RAII teardown, and OS-specific verification gaps documented in `contracts/*` and `quickstart.md`. |
| **VII. Zero Trust Security & Untrusted Input Stance (NON-NEGOTIABLE)** | Enforces 8 KB sniffer before both core & OS codecs, composite archive bomb caps (`50k` entries / `64 MB` CD / `4 GB` / `512 MB`), `150 MP` overflow-checked pixel limits, `PathSecurity`/`PathRedactor`, and HTTPS+SHA-256 updates. | **PASS** — Trust boundaries and negative security requirements are codified cleanly in `contracts/core-security-and-cli-contract.md`. |

## Project Structure

### Documentation (this feature)

```text
specs/002-rust-crossplatform-viewer/
├── spec.md                                                        # Functional & non-functional requirements + 2026-11-16 clarifications
├── research.md                                                    # Phase 0 crate, SIMD/GPU, OS codec & Wayland evidence (R1–R8)
├── data-model.md                                                  # Entities, v1.0.6 settings compatibility, caches & session state machine
├── quickstart.md                                                  # Cross-OS Cargo check/clippy/test/bench & validation scenarios
├── checklists/
│   └── requirements.md                                            # Spec quality checklist (16/16 passing)
├── contracts/
│   ├── core-security-and-cli-contract.md                          # CLI fail-closed, 8 KB sniffer, zip bombs, settings, redaction, HTTPS updates
│   ├── media-pipeline-and-os-codecs-contract.md                   # Core vs OS codecs, moxcms/HDR, SIMD resize, prefetch, animation streaming
│   └── ui-chrome-shortcuts-and-os-integration-contract.md         # Frameless chrome, shortcuts, trackpad, dialogs, Windows/macOS/Linux OS hooks
└── plan.md                                                        # This file (/speckit-plan output)
```

### Source Code (target Cargo workspace at repository root)

```text
Cargo.toml                                   # Workspace manifest (iv-core, iv-media, iv-platform, iv-app; profile opt-level tuning)
crates/
├── iv-core/                                 # (#![forbid(unsafe_code)]) Pure domain, security, layout & state orchestration
│   └── src/
│       ├── cli.rs                           # Fail-closed CLI parsing + macOS -psn_* filtering + Explorer C:\" repair
│       ├── path_security.rs                 # Trailing-separator root containment, ZIP traversal/symlink guards, PathRedactor
│       ├── format_guard.rs                  # 8,192-byte multi-BOM (UTF-32/16/8) + whitespace script sniffer
│       ├── settings.rs                      # v1.0.6-compatible camelCase/enum JSON settings, 64 KB cap, fs4 lock save
│       ├── natural_sort.rs                  # Locale-neutral natural ordering (image2 < image10)
│       ├── layout.rs                        # DisplayMode (Original/FitWidth/FitHeight/Fill), cursor-anchor zoom, pan clamping
│       ├── chrome_state.rs                  # Pin/hover/fullscreen restore state (Wayland-aware)
│       ├── trackpad.rs                      # Accumulated smooth scroll step threshold + native pinch math
│       ├── codecs.rs                        # OsCodecProvider trait & OsDecodedBuffer DTO (orientation_applied, srgb_converted) keeping Cargo DAG acyclic
│       ├── slideshow.rs                     # Clock-abstracted 5–90s sequential/random non-repeating slideshow
│       ├── logger.rs                        # Non-blocking bounded channel logger (7-day retention, >=300ms trace, 2s Drop flush)
│       └── i18n.rs                          # Bilingual Korean (ko) and English (en) message & label catalog
├── iv-media/                                # Memory-safe decoding, archives, SIMD scaling, ICC/HDR, EXIF, session & prefetch
│   └── src/
│       ├── sources/                         # MediaSource trait, FolderSource (+ notify-debouncer-mini 0.7s), ZipSource (pre-flight EOCD check + ZipArchive + composite caps)
│       ├── decode.rs                        # Core pure-Rust decoders (zune-jpeg, image, image-extras, image_dds, rawpsd, jxl-oxide, exr, zenwebp, pure-Rust .cur) + per-page TIFF & EXIF AutoOrient (skipping OS-already-oriented) + source_width/height
│       ├── color_hdr.rs                     # moxcms ICC (P3/AdobeRGB/CMYK/Cmyk16 -> sRGB when !srgb_converted) + soft HDR tone mapping & 16-bit normalization
│       ├── resize_filter.rs                 # fast_image_resize SIMD downscaling + screen-pixel filter parameter math (sigma / display_scale)
│       ├── animation.rs                     # <=512 MB pre-decode vs <=128 MB forward ring streaming + GIF/WebP/APNG (DisposeOp/BlendOp) wall-clock sync
│       ├── exif.rs                          # kamadak-exif 13-field extraction
│       ├── cache.rs                         # Arc<DecodedFrame> LRU (<=8) & thumbnail LRU (<=300)
│       └── session.rs                       # Monotonic open/load/render generation cancellation, +1/+2/-1/burst+4 prefetch, 250ms/500ms timers
├── iv-platform/                             # Narrow OS adapters (#[cfg(target_os = ...)]) implementing iv-core traits
│   └── src/
│       ├── codecs.rs                        # Implements iv_core::codecs::OsCodecProvider (pre-checked 8 KB + dimensions before Windows WIC / macOS ImageIO / Linux probe)
│       ├── integration.rs                   # OsIntegration trait (Windows HKCU Explorer/Default Apps; macOS Finder OpenURLs delegate/.app; Linux XDG .desktop)
│       ├── update.rs                        # HttpFeedTransport + reqwest (rustls-tls) + sha2 unconditional HTTPS updater, bounded streams, OS apply flows
│       └── paths_dialogs.rs                 # OS app-data dirs (directories) & non-blocking pickers (rfd::AsyncFileDialog + EventLoopProxy; xdg-portal/zenity)
└── iv-app/                                  # Binary GUI crate (owned winit 0.30 ApplicationHandler + custom wgpu canvas + egui-winit/egui-wgpu overlays)
    └── src/
        ├── main.rs                          # Owned winit 0.30 loop (PinchGesture + macOS openURLs: + ScaleFactorChanged + multi-window OS sub-dialogs), shutdown flush
        ├── renderer.rs                      # wgpu opaque swapchain (16K limit request, oversized webtoon/panorama tile splitting, deferred Resized, >=60% downscale, blur/unsharp/checkerboard shaders)
        ├── views/                           # Custom titlebar, floating bars, UpdateNotice + separate OS child windows (Settings, ImageList, About), EXIF panel
        └── resources/                       # Icons + bundled fluent ko/en localization assets

packaging/                                   # OS packaging scripts (Windows NSIS reuse/update, macOS Info.plist/.dmg, Linux .desktop/AppImage)
.github/workflows/ci.yml                     # 3-OS CI matrix: windows-latest, macos-latest, ubuntu-latest (check, clippy, test, bench)
```

*(Note: During transition, legacy C# code in `src/` and `specs/001-*` remain untouched until cutover tasks explicitly archive or replace them.)*

## Complexity Tracking

> No Constitution violations exist. Multi-crate separation (`iv-core`, `iv-media`, `iv-platform`, `iv-app`) directly enforces Principles III, IV, VI, and VII by isolating zero-`unsafe` security/domain logic from GPU UI and `#[cfg(target_os)]` FFI adapters.
