---

description: "Actionable, dependency-ordered Classical TDD task list for Cross-Platform Rust ImageViewer Parity & Performance Upgrade"
---

# Tasks: Cross-Platform Rust ImageViewer Parity & Performance Upgrade

**Input**: Design documents from `/specs/002-rust-crossplatform-viewer/`
**Prerequisites**: [plan.md](plan.md) (required), [spec.md](spec.md) (required), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts), [quickstart.md](quickstart.md)

**Tests**: REQUIRED by [constitution.md](../../.specify/memory/constitution.md) Principle III (Classical Red-Green-Refactor TDD via `cargo test --workspace`, zero mocking crates, zero `#[ignore]`, zero `#[cfg(test)]`/`#[cfg(debug_assertions)]` security bypasses). Failing tests MUST be written and observed failing before implementing the corresponding production code in every phase.

**Organization**: Tasks are grouped by user story (`US1 P1 -> US2 P2 -> US3 P3`) so each increment can be built, tested headlessly, benchmarked, and verified independently across Windows 10/11 (x64), macOS (`aarch64` & `x86_64`), and Linux (`x86_64` X11 & Wayland).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files/crates, no incomplete dependencies)
- **[Story]**: User story label (`[US1]`, `[US2]`, `[US3]`) required for user story phases only
- Every task cites exact repository file paths and verbatim invariants from `data-model.md` and `contracts/`

---

## Phase 1: Setup (Workspace Initialization & Test Harnesses)

**Purpose**: Initialize the 4-crate Rust Cargo workspace, enforce lint/SIMD profiles, configure 3-OS CI, and provide authentic in-memory/filesystem test fixtures.

- [X] T001 Initialize Cargo workspace manifest `Cargo.toml` (`resolver = "2"`, `edition = "2024"`, MSRV `1.85`, members `crates/iv-core`, `crates/iv-media`, `crates/iv-platform`, `crates/iv-app`, plus `[profile.dev.package.zune-jpeg] opt-level = 3` and `[profile.dev.package.fast_image_resize] opt-level = 3`) and crate manifests `crates/iv-core/Cargo.toml`, `crates/iv-media/Cargo.toml`, `crates/iv-platform/Cargo.toml`, `crates/iv-app/Cargo.toml` (wiring `winit` `0.30` + `wgpu` + `egui`/`egui-winit`/`egui-wgpu`, `image` `0.25.4+`, `image-extras`, `image_dds`, `rawpsd`, `rcgen` dev-dependency)
- [X] T002 [P] Configure strict workspace Clippy/Rust lints in `Cargo.toml` and `#![forbid(unsafe_code)]` at crate root `crates/iv-core/src/lib.rs`, plus shared localization resource scaffolding in `crates/iv-app/src/resources/`
- [X] T003 [P] Create multi-OS CI workflow running `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and Criterion benchmarks across `windows-latest`, `macos-latest`, and `ubuntu-latest` in `.github/workflows/ci.yml`
- [X] T004 [P] Create deterministic real-fixture test helpers (fake/manual `Clock` helper for slideshow/timers, temp directory cleanup, UTF-32/16/8 BOM polyglot generators, `> 512`-byte whitespace headers, Zip-Slip / POSIX-`0xA000` / DOS-`0x0400` / forged-EOCD >50,000-entry & >64 MB CD ZIP bomb builders, synthetic `.cur`/`.dds`/`.psd`/HDR/ICC/EXIF fixtures, and an ephemeral `rcgen` self-signed loopback TLS CA helper for `HttpFeedTransport` update tests) in `crates/iv-core/tests/fixtures.rs`, `crates/iv-media/tests/fixtures.rs`, and `crates/iv-platform/tests/fixtures.rs`

---

## Phase 2: Foundational (Blocking Trust-Boundary & Configuration Core)

**Purpose**: Pure-Rust, zero-`unsafe` foundational modules in `crates/iv-core` required across US1, US2, and US3.

**⚠️ CRITICAL**: Complete this phase before beginning user story phases.

### Red-First Tests (Foundational)

- [X] T005 [P] Write failing unit tests for `CliOptions::parse` covering `0` args, `1` valid path (drive `<letter>:\...`, UNC `\\...`, Unix `/...`, Windows Explorer `C:\"` repair), ignoring benign macOS Finder `-psn_*` session flags, Windows-only `--register-shell` / `/register-shell` / `--unregister-shell` / `/unregister-shell` switches, `is_path_not_found_error = true` on missing single path, and fail-closed errors on `>= 2` positional paths, unknown/duplicate switches, combined path+switch, and shell switches invoked on non-Windows targets in `crates/iv-core/tests/cli_tests.rs`
- [X] T006 [P] Write failing unit tests for `PathSecurity` (canonical trailing-separator root containment rejecting sibling-prefix bypasses such as `logs-evil` vs `logs`, archive entry normalization `\`->`/` rejecting `..`/leading-`/`/drive-root/`\0`, `symlink_metadata` / DOS `0x0400` / POSIX `0xA000` symlink checks) and `PathRedactor` (stripping quoted and space-containing Windows, UNC, and Unix directory prefixes down to filename-only, plus escaping `\r`/`\n`/control chars against CWE-117 log injection) in `crates/iv-core/tests/path_security_tests.rs`
- [X] T007 [P] Write failing unit tests for `FormatGuard` (`<= 8,192`-byte sniffer enforcing 4-byte UTF-32 LE `FF FE 00 00` & BE `00 00 FE FF` BOMs before 2-byte UTF-16 LE `FF FE` / BE `FE FF` and UTF-8 `EF BB BF`, BOM-less UTF-16 `<\0`/`\0<` heuristics, `>= 512`-byte pure-whitespace header rejection, and case-insensitive blocking of `<`, `<?xml`, `<svg`, `<html`, `%!`, `%PDF`, `push `, `viewbox`) in `crates/iv-core/tests/format_guard_tests.rs`
- [X] T008 [P] Write failing unit tests for `ViewerSettings` and `SettingsStore` covering full `v1.0.6` JSON compatibility (`camelCase` keys, string enum names like `"Fill"`/`"Black"`/`"Sequential"`, JSON comments & trailing commas, unknown-field tolerance, `Version < 2` -> `Navigate` migration, `Version < 3` with `smoothSigma ~= 0.8` -> `0.5` migration, exact `v1.0.6` bool/numeric defaults `topBarPinned = true`/`bottomBarPinned = true`/`playAnimations = true`/`wrapAround = true`/`watchFolder = true`/`thumbnail.visible = true`/`update.autoCheck = true`/`sharpenSigma = 1.0`/`sharpenAmount = 1.0`/`slideshow.intervalSeconds = 5`/`slideshow.repeat = true`), `> 65,536`-byte (`64 KB`) & symlink rejection falling back to defaults, `f64::is_finite` recovery (`window.width` `[320, 20000]` default `1280`, `window.height` `[240, 20000]` default `800`, `left`/`top` -> `None`, `filter.smoothSigma` `[0.1, 3.0]` default `0.5`, `sharpenSigma`/`sharpenAmount` `[0.1, 5.0]`, `limits.maxImageMegapixels` `[10, 1000]` default `150`, `limits.maxZipEntryMegabytes` `[10, 4096]` default `512`, `update.skippedVersion` `<= 32` ASCII version validation), and `fs4`-locked selective-merge saves preventing multi-instance Lost Updates (background/window-close merging latest disk preferences instead of overwriting sibling edits) in `crates/iv-core/tests/settings_tests.rs`
- [X] T009 [P] Write failing unit tests for `i18n` (`ko` and `en` localized keys parity) and `AsyncFileLogger` (bounded `DropOldest` channel, filename-only path redaction, CRLF safety, `>= 300 ms` slow-render trace formatting, 7-day rotation cleanup, fail-silent read-only log recovery, and synchronous bounded `<= 2s` shutdown drain on `Drop`) in `crates/iv-core/tests/logger_and_i18n_tests.rs`

### Implementation (Foundational)

- [X] T010 [P] Implement fail-closed `CliOptions::parse` matching `contracts/core-security-and-cli-contract.md` Section 1 in `crates/iv-core/src/cli.rs`
- [X] T011 [P] Implement `PathSecurity` and `PathRedactor` matching `data-model.md` Section 2 and `contracts/core-security-and-cli-contract.md` in `crates/iv-core/src/path_security.rs`
- [X] T012 [P] Implement `FormatGuard` (`8,192`-byte BOM/whitespace/script sniffer) in `crates/iv-core/src/format_guard.rs`
- [X] T013 [P] Implement `ViewerSettings`, enums, `v1.0.6` `serde` schema compatibility, and `fs4`-locked selective-merge atomic `SettingsStore` (re-reading disk state under lock before merging dirty preference or `Window` sections) in `crates/iv-core/src/settings.rs`
- [X] T014 [P] Implement `ko`/`en` localized catalog in `crates/iv-core/src/i18n.rs` and non-blocking bounded `AsyncFileLogger` in `crates/iv-core/src/logger.rs`

**Checkpoint**: `cargo test -p iv-core` passes all foundational security, CLI, settings compatibility, localization, and redacted logging tests.

---

## Phase 3: User Story 1 - Fast Core Viewing, Navigation, and Viewport Manipulation (Priority: P1) 🎯 MVP

**Goal**: Deliver blazing-fast cross-platform core viewing (`Windows 10/11`, `macOS`, `Linux X11/Wayland`) covering natural-sorted folder & `.zip`/`.cbz` loading, memory-safe pure-Rust decoding with EXIF `Orientation (1–8)` auto-rotation + `fast_image_resize` SIMD downscaling, `moxcms` ICC->sRGB & HDR tone mapping, generation-canceled prefetching (`+1`/`+2`/`-1`/burst `+4`), opaque frameless `winit 0.30` + `wgpu` + `egui` windowing (`ScaleFactorChanged` DPI awareness), `0`/`7`/`8`/`9` display modes, transforms, precision trackpad scroll/pinch, and Wayland-safe geometry/dragging hitting `SC-001` (`<= 36 ms` / `<= 49 ms`) and `SC-002` (`<= 22 ms`).

**Independent Test**: Run headless `iv-core` / `iv-media` tests + real `test.zip`-class benchmark suite (`2592×1728` 32 images at `1280×800` & `2560×1440`) and verify UI chrome/viewport parity across host OSes.

### Tests for User Story 1 (Red First — MUST FAIL before implementation)

- [X] T015 [P] [US1] Write failing unit tests for `NaturalSort` (`image2 < image10` across Unicode/ASCII), `ViewportLayoutCalculator` (`Original`/`FitWidth`/`FitHeight`/`Fill`, cursor-anchored zoom, pan clamping, `dpi_scale_factor` Retina/Windows/Wayland fractional logical-to-physical pixel scaling, small-image fit enlargement, R/L/H/V screen transforms, `250 ms` thumbnail placeholder bounds), `ChromeState` (independent Top/Bottom pin, `` ` `` global toggle, fullscreen restore, Wayland `Width`/`Height`/`IsMaximized`-only state), and `TrackpadAccumulator` (accumulated pixel-delta threshold `>= 48.0` px + step cooldown preventing rapid skip, continuous pinch scale) in `crates/iv-core/tests/viewport_and_chrome_tests.rs`
- [X] T016 [P] [US1] Write failing integration tests for `FolderSource` (natural sorting, skipping symlink/reparse file entries while allowing explicitly opened symlink root) and `ZipSource` (zero disk extraction, zero-allocation pre-flight EOCD/ZIP64 EOCD scan rejecting `> 50,000` entries, `> 64 MB` central directory, and `> 4 GB` archive container BEFORE calling `zip::ZipArchive::new` to block `CVE-2026-0429`-class OOM, `> MaxZipEntryMegabytes` uncompressed entry stream via `Read::take`, POSIX `0xA000`/DOS `0x0400` symlinks, `../` traversal, and graceful encrypted-ZIP error) in `crates/iv-media/tests/sources_tests.rs`
- [X] T017 [P] [US1] Write failing integration tests for memory-safe core decoding covering full standard-format parity (`zune-jpeg`, `image` `0.25.4+`, `image-extras` `.pcx`, `image_dds` BC1–BC7 `.dds`, `rawpsd` RGB/CMYK/Gray `.psd`, `jxl-oxide` `.jxl`, `exr` `.exr`, `zenwebp`/`image-webp`, pure-Rust `.cur` parser `type == 2` largest-frame alongside `.ico`), automatic EXIF `Orientation (1–8)` correction (`apply_orientation` + `5–8` swapped width/height pre-cap parity with `AutoOrient`), `checked_mul(width, 4)` + `MaxImageMegapixels` pre-allocation overflow rejection, `moxcms` Display P3/Adobe RGB/CMYK -> sRGB conversion vs untagged/sRGB pass-through & corrupt ICC recovery, HDR `.exr`/`.hdr` soft tone mapping & 16-bit integer normalization, and `fast_image_resize` SIMD downscaling in `crates/iv-media/tests/decode_color_resize_tests.rs`
- [X] T018 [P] [US1] Write failing unit/integration tests for `DecodedFrame` LRU cache (`<= 8` entries), dual-lane `rayon` worker pool (`clamp(cpus / 2, 1, 4)`), monotonic `open_gen`/`load_gen`/`render_gen` cancellation preemption, directional `+1`/`+2`/`-1`/burst `+4` prefetch attachment (awaiting in-flight prefetch without duplicate decode), and asynchronous background full-resolution upgrade when zooming past cached viewport size in `crates/iv-media/tests/session_and_cache_tests.rs`

### Implementation for User Story 1 (Green & Refactor)

- [X] T019 [P] [US1] Implement `NaturalSort`, DPI-aware `ViewportLayoutCalculator`, `ChromeState` (Wayland-aware), `TrackpadAccumulator`, and acyclic `OsCodecProvider` trait + `OsDecodedBuffer` DTO (`orientation_applied`, `srgb_converted`) in `crates/iv-core/src/natural_sort.rs`, `crates/iv-core/src/layout.rs`, `crates/iv-core/src/chrome_state.rs`, `crates/iv-core/src/trackpad.rs`, and `crates/iv-core/src/codecs.rs`
- [X] T020 [P] [US1] Implement `MediaSource`, `FolderSource` (symlink skip), and bounded `ZipSource` with zero-allocation pre-flight EOCD/ZIP64 EOCD checks strictly before `ZipArchive::new` (enforcing `<= 50,000` entries, `<= 64 MB` CD, `<= 4 GB` container & encrypted error handling) in `crates/iv-media/src/sources/mod.rs`, `crates/iv-media/src/sources/folder.rs`, and `crates/iv-media/src/sources/zip.rs`
- [X] T021 [P] [US1] Implement `FormatGuard`-gated core decoders (`zune-jpeg`, `image` `0.25.4+`, `image-extras` `.pcx`, `image_dds` `.dds`, `rawpsd` `.psd`, `jxl-oxide` `.jxl`, `exr` `.exr`, `zenwebp`/`image-webp`, checked pure-Rust `.cur` parser) preserving post-orientation `source_width`/`source_height` separately from `buffer_width`/`buffer_height`, EXIF `Orientation (1–8)` `apply_orientation()` (`5–8` swapped dimension check, skipping duplicate rotation/ICC when `OsDecodedBuffer` flags are true), `checked_mul` + `MaxImageMegapixels` pre-checks in `crates/iv-media/src/decode.rs`, and `moxcms` ICC->sRGB + lightweight HDR tone mapping & 16-bit normalization in `crates/iv-media/src/color_hdr.rs`
- [X] T022 [P] [US1] Implement `fast_image_resize` AVX2/SSE4.1/NEON SIMD downscaling in `crates/iv-media/src/resize_filter.rs` and `Arc<DecodedFrame>` bounded LRU cache (`<= 8`) in `crates/iv-media/src/cache.rs`
- [X] T023 [US1] Implement `SessionCoordinator` & `PrefetchCoordinator` (`open_gen`/`load_gen`/`render_gen` cancellation, dual-lane bounded `rayon` pool, `250 ms` placeholder & `500 ms` spinner timers, `+1`/`+2`/`-1`/burst+4 prefetch, async full-res zoom upgrade) in `crates/iv-media/src/session.rs`
- [X] T024 [P] [US1] Implement `OsPaths` (`directories` + `%LOCALAPPDATA%\ImageViewer` on Windows) and non-blocking `OsDialogs` (`rfd::AsyncFileDialog` + `winit::event_loop::EventLoopProxy`, macOS AppKit / Linux `xdg-desktop-portal` + `zenity` fallback) in `crates/iv-platform/src/paths_dialogs.rs`
- [X] T025 [US1] Implement `wgpu` opaque swapchain presenter (explicit `max_texture_dimension_2d: min(adapter.limits().max_texture_dimension_2d, 16384)` device request + tiled sub-texture splitting with split UVs for oversized webtoons/panoramas, deferred pre-render `Resized` reconfiguration skipping zero-size minimizes, `ScaleFactorChanged` surface update, checkerboard/solid backgrounds, GPU bilinear display downscaling for `>= 60%` physical viewport images, and GPU separable Gaussian Blur / Unsharp Mask shader pipelines matching screen-pixel `Smooth`/`SmoothSharpen` uniforms) in `crates/iv-app/src/renderer.rs`
- [X] T026 [US1] Implement owned `winit 0.30` `ApplicationHandler` + custom `wgpu` canvas + `egui-winit` / `egui-wgpu` frameless multi-window shell (`decorations: false`, `transparent: false`, independent `WindowId -> WindowSurfaceState` OS child windows for auxiliary dialogs, system CJK font registration, custom titlebar, auto-hiding pinned/edge-hover floating top/bottom bars, `` ` ``/display-mode/zoom/pan/rotate/flip/`Esc` shortcuts with `Cmd`/`Ctrl` & `Alt+Enter`/`Ctrl+Cmd+F`, `window.drag_window()` canvas move, native `WindowEvent::PinchGesture` + `TrackpadAccumulator` smooth scroll, `ScaleFactorChanged` DPI handling, OS drag-and-drop, and Wayland size/maximize-only restoration) in `crates/iv-app/src/main.rs` and `crates/iv-app/src/views/main_view.rs`
- [X] T027 [P] [US1] Create Criterion latency & memory benchmark suite verifying `<= 36 ms` windowed (`1280×800`), `<= 49 ms` fullscreen (`2560×1440`), and `<= 22 ms` rapid navigation parity on representative 32-image archive workloads (`SC-001`, `SC-002`) in `crates/iv-media/benches/pipeline_bench.rs`

**Checkpoint**: User Story 1 (MVP) delivers full cross-platform core image/archive viewing, frameless GPU presentation, navigation, transforms, and latency parity (`cargo test --workspace` + `cargo bench`).

---

## Phase 4: User Story 2 - Rich Media Formats, Animations, Multi-Page, Filters, Slideshow, EXIF & OS Codecs (Priority: P2)

**Goal**: Complete format and workflow parity including lazy/ring-buffered GIF/WebP/APNG animations (`<= 512 MB` / `<= 128 MB`, `< 250 ms` frame 1, `<= 470 MB` RAM, `<= 5%` CPU), multi-page TIFF & `.ico`/`.cur`, screen-pixel filters, slideshow, 13-field EXIF panel, thumbnail strip (`<= 300`), sortable Image List dialog, 0.7 s folder watcher, and Zero Trust `OsCodecProvider` for RAW/HEIC/AVIF/JP2.

**Independent Test**: Run `animation_tests`, `filter_and_tiff_tests`, `exif_tests`, `slideshow_and_watch_tests`, `os_codec_security_tests`, and the `1080×1920` 267-frame animated WebP benchmark (`SC-003`).

### Tests for User Story 2 (Red First — MUST FAIL before implementation)

- [X] T028 [P] [US2] Write failing unit/integration tests for `AnimationPlayer` covering lazy frame-1 immediate decode, `<= 512 MB` pre-decode vs `<= 128 MB` forward ring-buffer eviction, wall-clock frame pacing, delay normalization (`GIF <= 10 ms -> 100 ms`; `WebP` preserving per-frame delays including `1 ms` unless all frames `<= 10 ms`; `APNG` `DisposeOp::None|Background|Previous`, `BlendOp::Source|Over` compositing & rational `delay_num/delay_den`), animation filter bypass, and multi-page TIFF per-page IFD `Orientation (1–8)` + `16-bit`/CMYK (`moxcms`) indexing + `.ico`/`.cur` largest-frame selection (`PgUp`/`PgDn` preserving rotation/flip) in `crates/iv-media/tests/animation_and_multipage_tests.rs`
- [X] T029 [P] [US2] Write failing unit tests for screen-pixel calibrated filter math (`Smooth` sigma, `SmoothSharpen` unsharp sigma/amount, `sigma / display_scale`, `< 0.05` effective skip, animation bypass) driving `wgpu` shader filter uniforms and `SlideShowController` (`Clock`-driven `5–90 s` intervals, `Sequential` / non-repeating `Random`, `Repeat`, manual move timer reset, lookahead index) in `crates/iv-media/tests/filter_and_slideshow_tests.rs`
- [X] T030 [P] [US2] Write failing integration tests for `ExifExtractor` (`kamadak-exif` reading all 13 documented fields gracefully on partial/corrupt tags), thumbnail cache (`<= 300` LRU), sortable `ImageListModel` (`Name` natural / `Modified` / `Size`, unsupported dimming), and debounced `0.7 s` `FolderWatcher` (ignoring symlinks, keeping index on active-file delete) in `crates/iv-media/tests/exif_thumbnail_watch_tests.rs`
- [X] T031 [P] [US2] Write failing security & contract tests for `iv_core::codecs::OsCodecProvider` verifying mandatory `8 KB` binary signature check + `512 MB` stream cap + pre-decode `MaxImageMegapixels` dimension gate BEFORE pixel allocation in Windows `WIC` / macOS `ImageIO` / Linux probes, `OsDecodedBuffer` `orientation_applied`/`srgb_converted` double-transform prevention, and graceful fail-closed localized missing-codec error when optional Linux system codecs are absent in `crates/iv-platform/tests/os_codec_tests.rs`

### Implementation for User Story 2 (Green & Refactor)

- [X] T032 [P] [US2] Implement `AnimationPlayer` (`<= 512 MB` / `<= 128 MB` ring buffer, wall-clock scheduler, GIF/WebP delay rules, and APNG `DisposeOp`/`BlendOp`/`delay_num/delay_den` compositing) in `crates/iv-media/src/animation.rs` and multi-page TIFF (per-page IFD `Orientation`/`16-bit`/CMYK `moxcms`) + `.ico`/`.cur` frame dispatch in `crates/iv-media/src/decode.rs`
- [X] T033 [P] [US2] Implement screen-pixel `Smooth` / `SmoothSharpen` filter uniform & kernel computation (`sigma / display_scale`, `< 0.05` skip, animation bypass) wired into the `wgpu` shader passes (`T025`) in `crates/iv-media/src/resize_filter.rs` and `Clock`-injected `SlideShowController` in `crates/iv-core/src/slideshow.rs`
- [X] T034 [P] [US2] Implement `kamadak-exif` 13-field extractor in `crates/iv-media/src/exif.rs`, bounded `<= 300` thumbnail generator/LRU in `crates/iv-media/src/cache.rs`, and `notify-debouncer-mini` (`700 ms`) watcher in `crates/iv-media/src/sources/watcher.rs`
- [X] T035 [US2] Implement `iv_core::codecs::OsCodecProvider` returning `OsDecodedBuffer` (`orientation_applied`, `srgb_converted`) with strict pre-decode `FormatGuard` + dimension guards and platform adapters (`windows` `IWICImagingFactory` on Windows, `objc2-image-io`/`ImageIO` `kCGImageSourceShouldCache: false` on macOS, safe optional probe + missing-codec localized error on Linux) in `crates/iv-platform/src/codecs.rs`
- [X] T036 [US2] Wire EXIF info panel (`Tab`), bottom-left animation/multi-page HUD overlay (reporting true post-orientation `source_width × source_height` even during downscaled preview), thumbnail strip (`48–256 px` + optional scrollbar), separate OS child-window sortable Image List dialog (`Enter`), and slideshow controls (`Space` + status bar) into `crates/iv-app/src/views/main_view.rs` and `crates/iv-app/src/views/image_list_view.rs`
- [X] T037 [P] [US2] Add Criterion/memory benchmark verifying `< 250 ms` frame 1, `<= 470 MB` RAM, and `<= 5%` CPU playback on a `1080×1920`, `267`-frame animated WebP (`SC-003`) in `crates/iv-media/benches/animation_bench.rs`

**Checkpoint**: User Stories 1 and 2 deliver 100% viewing, format, animation, EXIF, filter, thumbnail, image-list, watcher, and slideshow parity across all 3 OSes.

---

## Phase 5: User Story 3 - Settings/About UI, Auto-Updater, and OS-Idiomatic Integration & Packaging (Priority: P3)

**Goal**: Deliver Settings & About dialogs, unconditional `https://` + SHA-256 auto-updating with OS-idiomatic execution, Windows HKCU Explorer/Default Apps + NSIS/portable parity, macOS `.app`/Finder `openURLs`/`.dmg` integration, and Linux `.desktop`/MIME integration.

**Independent Test**: Run `update_security_tests`, `os_integration_tests`, and headless UI model verification across Settings/About/Update flows (`SC-004`, `SC-005`).

### Tests for User Story 3 (Red First — MUST FAIL before implementation)

- [X] T038 [P] [US3] Write failing unit tests for `UpdateClient` (`HttpFeedTransport` + `reqwest` `rustls-tls` + `sha2` + ephemeral `rcgen` loopback CA cert store — zero `danger_accept_invalid_certs` or debug/env bypasses) verifying unconditional `https://` enforcement on `FeedBase`, `page_url`, `SHA256SUMS.txt`, `download_url`, and `OpenReleasePage`, bounded HTTP body streams (`<= 1 MB` JSON, `<= 64 KB` sums, `<= 256 MB` binary even with forged/missing `Content-Length`), strict 64-hex SHA-256 mismatch rejection deleting `.part`, Windows `/RELAUNCH=` sanitization (`"`, `" /"`, `\r`, `\n`, `\0`), safe temp path checks, and path-redacted update logs/errors in `crates/iv-platform/tests/update_tests.rs`
- [X] T039 [P] [US3] Write failing platform integration tests for `OsIntegration` covering Windows HKCU Explorer command registration + exact normalized `IsRegistered()` match (`Rebase` capabilities parity, `quote`/control-char rejection on `exePath`), macOS Finder URL validation/dispatch & `Info.plist` document association generation, Linux user `.desktop`/MIME generation & `PathSecurity` temp verification, and Settings/About model validation (`AboutInfo` runtime/OS/dependency version formatting & `SettingsViewModel` bounds) in `crates/iv-platform/tests/os_integration_tests.rs`

### Implementation for User Story 3 (Green & Refactor)

- [X] T040 [P] [US3] Implement `HttpFeedTransport` + `UpdateClient` (`reqwest` `rustls-tls` with caller-provided `RootCertStore` option for offline loopback tests + `sha2`, `<= 1 MB`/`64 KB`/`256 MB` stream caps, unconditional `https://`, `PathRedactor` diagnostics) + OS-specific apply logic (Windows NSIS `/S /RELAUNCH` + Portable `.old`/`.new` swap & startup cleanup; macOS `.dmg`/archive download + open; Linux AppImage/package download + open; HTTPS release page fallback) in `crates/iv-platform/src/update.rs`
- [X] T041 [P] [US3] Implement `OsIntegration` adapters (`winreg` HKCU Explorer & Default Apps `IsRegistered` exact match on Windows; `objc2-app-kit` custom `NSApplicationDelegate` `openURLs:` hook bound cleanly into the owned `winit::ApplicationHandler` startup on macOS; user XDG `.desktop` + `xdg-mime` handler on Linux) in `crates/iv-platform/src/integration.rs`
- [X] T042 [US3] Implement separate OS child-window Settings dialog (`Background`, `MouseWheel`, `WrapAround`, `WatchFolder`, `SlideShowRepeat`, `Thumbnail` size `48–256` + scrollbar, `Filter` sigmas/amount, `Limits` `MaxImageMegapixels` `10–500`, `Update.AutoCheck`, OS association controls), separate OS child-window About dialog (dynamic Cargo/library metadata + license URLs + clipboard Copy + manual update check), and top-right `UpdateNotice` overlay (`SkipVersion`, `Dismiss`, `ProgressBar`, `Install`) in `crates/iv-app/src/views/settings_view.rs`, `crates/iv-app/src/views/about_view.rs`, and `crates/iv-app/src/views/update_notice.rs`
- [X] T043 [P] [US3] Wire startup `AutoCheck` (`~5 s` delay, `20–24 h` cadence), leftover `.old` cleanup, non-blocking `Update` scheduling, and shutdown flush order (`RunPending` log before `AsyncFileLogger` `Drop` `2s` drain) cleanly in `crates/iv-app/src/main.rs`
- [X] T044 [P] [US3] Create cross-platform packaging templates and release build scripts (`packaging/macos/Info.plist.in` with `CFBundleDocumentTypes` & `packaging/macos/package-dmg.sh`, `packaging/linux/imageviewer.desktop.in` & `packaging/linux/package-appimage.sh`, plus Rust-target NSIS/portable build script aligned with `src/ImageViewer.Installer/ImageViewer.nsi`) in `packaging/`

**Checkpoint**: All 3 User Stories (`US1`, `US2`, `US3`) are implemented, secure, localized (`ko`/`en`), and packageable across Windows, macOS, and Linux.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final constitutional governance audit, security sweep, benchmark verification, and full multi-OS validation.

- [X] T045 [P] Audit entire Rust workspace (`Cargo.toml`, `crates/*`) to confirm `#![forbid(unsafe_code)]` on `iv-core`, audited `// SAFETY:` invariants on `iv-platform` OS FFI blocks, zero mocking frameworks, zero `#[ignore]` tests, zero `#[cfg(test)]`/`#[cfg(debug_assertions)]` security bypasses, and English-first identifiers/rustdoc (`Constitution Principles II & IV`)
- [X] T046 [P] Verify `ko`/`en` bilingual coverage completeness across all UI strings, tooltips, shortcuts, About notices, and localized error messages (`Constitution Principle I & II`), and verify all contracts in `specs/002-rust-crossplatform-viewer/contracts/` match implemented behavior
- [X] T047 Run full verification (`cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo bench --workspace`) per `specs/002-rust-crossplatform-viewer/quickstart.md` on the active host OS (`macOS` locally) AND verify Windows/Linux execution via `.github/workflows/ci.yml` (`windows-latest`, `ubuntu-latest`), reporting exact per-OS observed pass/fail counts and benchmark numbers without hiding unexecuted targets (`Constitution Principles V & VI`)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1 completion — BLOCKS Phases 3, 4, and 5 (`CliOptions`, `PathSecurity`/`PathRedactor`, `FormatGuard`, `ViewerSettings`/`SettingsStore`, `i18n`, `AsyncFileLogger` live in `iv-core`).
- **Phase 3 (User Story 1 — P1 MVP)**: Depends on Phase 2 completion. Delivers standalone working fast viewer across Windows/macOS/Linux.
- **Phase 4 (User Story 2 — P2)**: Depends on Phase 3 core decoding/session/UI pipeline (`iv-media` + `iv-app`).
- **Phase 5 (User Story 3 — P3)**: Depends on Phase 2 (`ViewerSettings`, `PathSecurity`, `AsyncFileLogger`) and integrates into `iv-app` + `iv-platform` alongside US1/US2.
- **Phase 6 (Polish)**: Depends on completion of Phases 3, 4, and 5.

### Within Each Story (Classical Red-Green-Refactor Order)

1. Write failing unit/integration tests FIRST (`cargo test`) and observe expected failure reasons.
2. Implement `iv-core` pure math/state types before `iv-media` pipeline workers and `iv-platform` OS adapters.
3. Wire into `crates/iv-app` GUI views/renderer without blocking UI frame loops.
4. Confirm tests transition Red -> Green with zero `#[ignore]` or relaxed assertions, run Clippy cleanly, and validate benchmarks before advancing.

---

## Parallel Execution Examples

### Phase 2 (Foundational Red-Phase Tests & Core Modules)

```text
# Parallel Red tests in iv-core:
- T005: cli_tests.rs (fail-closed CLI + -psn_* + C:\" repair)
- T006: path_security_tests.rs (canonical root containment, ZIP flags, PathRedactor CR/LF safety)
- T007: format_guard_tests.rs (8 KB UTF-32/16/8 BOMs & whitespace polyglot blocking)
- T008: settings_tests.rs (v1.0.6 JSON compatibility + 64 KB cap + fs4 concurrent lock)
- T009: logger_and_i18n_tests.rs (bounded logger + 2s flush + ko/en parity)
```

### Phase 3 (User Story 1 — Fast Core Viewing MVP)

```text
# Parallel Red tests:
- T015: viewport_and_chrome_tests.rs
- T016: sources_tests.rs
- T017: decode_color_resize_tests.rs
- T018: session_and_cache_tests.rs

# Parallel Green core/media modules:
- T019: iv-core math/chrome/trackpad
- T020: iv-media FolderSource + ZipSource
- T021: iv-media decode + color_hdr (moxcms + HDR tone mapping)
- T022: iv-media resize_filter (SIMD) + LRU cache
- T024: iv-platform paths_dialogs
```

### Phase 4 (User Story 2 — Animations, EXIF, Filters, OS Codecs)

```text
- T028: animation_and_multipage_tests.rs -> T032: animation.rs
- T029: filter_and_slideshow_tests.rs -> T033: resize_filter.rs + slideshow.rs
- T030: exif_thumbnail_watch_tests.rs -> T034: exif.rs + cache.rs + watcher.rs
- T031: os_codec_tests.rs -> T035: iv-platform codecs.rs (WIC / ImageIO / Linux)
```

---

## Implementation Strategy

### MVP First (User Story 1 — Fast Cross-Platform Core Viewer)

1. Complete **Phase 1 (Setup)** and **Phase 2 (Foundational)** (`T001`–`T014`).
2. Complete **Phase 3 (User Story 1 / P1)** (`T015`–`T027`).
3. **STOP and VALIDATE**: Confirm `cargo test --workspace` passes and Criterion benchmarks match or beat `<= 36 ms` (`1280×800`) / `<= 49 ms` (`2560×1440`) / `<= 22 ms` rapid navigation targets (`SC-001`, `SC-002`).

### Incremental Delivery (`P1 -> P2 -> P3`)

1. **Increment 1 (`US1`)**: Cross-platform Rust core viewer, SIMD resize, ICC/HDR, frameless GPU presentation, shortcuts & trackpad parity (`T015`–`T027`).
2. **Increment 2 (`US2`)**: Animations, multi-page TIFF, filters, EXIF, slideshow, thumbnails, Image List, folder watch, and pre-checked OS platform codecs (`T028`–`T037`).
3. **Increment 3 (`US3`)**: Unconditional HTTPS+SHA-256 updater, Settings/About/UpdateNotice UI, OS-idiomatic packaging & file associations (`T038`–`T044`).
4. **Final Verification (`Phase 6`)**: Security/`unsafe`/anti-bypass audit + 3-OS verification across macOS locally and Windows/Linux CI (`T045`–`T047`).
