# Data Model & Invariants: Cross-Platform Rust ImageViewer

**Feature**: `002-rust-crossplatform-viewer`
**Date**: 2026-11-16
**Spec**: [spec.md](spec.md)

---

## 1. `CliOptions` (Untrusted Command-Line & OS Launch Input)

- **Fields**:
  - `path: Option<PathBuf>` — Canonicalized single target file, folder, or archive path (`None` if no target, shell flag used, or validation fails).
  - `register_shell: bool` — True only when `--register-shell` or `/register-shell` is given on Windows.
  - `unregister_shell: bool` — True only when `--unregister-shell` or `/unregister-shell` is given on Windows.
  - `error: Option<String>` — Localized (`ko`/`en`) error message; non-`None` causes fail-closed exit or error prompt.
  - `is_path_not_found_error: bool` — Default `false`; `true` only when a single explicit user-supplied CLI path does not exist (only case permitted to echo the user-entered path in UI prompt).
- **Invariants**:
  - Accept `0` positional args, OR `1` valid path, OR `1` Windows shell switch (`register_shell XOR unregister_shell`).
  - Ignore benign macOS LaunchServices `-psn_*` session tokens; reject any other unknown `-`/`--`/`/` flag, `>= 2` positional paths, path + shell-switch combinations, or Windows `--register-shell`/`--unregister-shell` invocations on macOS/Linux (fail closed with localized error).
  - Repair broken Windows Explorer trailing drive-root quote escaping (`C:\"` -> `C:\`) before path resolution.

---

## 2. `PathValidation` & `SafeArchiveEntry` (Cross-Platform CWE-22 Boundary)

- **Fields**:
  - `canonical_path: PathBuf` — Normalized path without symlink resolution bypass.
  - `is_symlink_or_reparse: bool` — Evaluated via `symlink_metadata().file_type().is_symlink()` and Windows `FILE_ATTRIBUTE_REPARSE_POINT` (`0x0400`).
  - `normalized_entry_key: String` — Forward-slash normalized archive key (`\` -> `/`).
- **Validation Rules**:
  - **Root Containment**: Canonical root directory converted to trailing directory separator before OS-appropriate prefix check (`OrdinalIgnoreCase` equivalent on case-insensitive volumes, exact ordinal on Unix). Rejects sibling-prefix bypasses (`logs-evil` vs `logs`).
  - **Symlink Matrix**:
    - Explicitly user-opened top-level file/folder (`CLI`, `Open`, `DragDrop`, `Finder openURLs`): symlinks permitted.
    - Local folder enumeration & `WatchFolder` updates (`FolderSource`): symlink/reparse entries skipped.
    - Archive entries (`ZipSource`): reject `..`, leading `/`, Windows drive roots (`C:\`), null bytes `\0`, or POSIX `S_IFLNK` (`(mode >> 16) & 0xF000 == 0xA000`) / DOS `ReparsePoint` (`0x0400`).
    - App-managed config/log/update temp directories & files: inspect immediate parent directory + target file via `symlink_metadata`; fail safely on unexpected symlinks.

---

## 3. `ViewerSettings` (`v1.0.6` Backward-Compatible Configuration)

Matches `SettingsService.cs` & `ViewerSettings.cs` (`camelCase` JSON property names, string enum serialization, `CommentHandling::Skip`, trailing-comma tolerance, and ignoring unknown extra JSON fields):

- **Fields & Finite Bounds (with exact `v1.0.6` defaults)**:
  - `version: u32` (`CurrentVersion = 3`; `Version < 2` forces `display.mouse_wheel = MouseWheelAction::Navigate`; `Version < 3` with `(filter.smooth_sigma - 0.8).abs() < 0.001` migrates `filter.smooth_sigma` to `0.5`).
  - `window`: `width: f64` (`[320.0, 20000.0]`, default `1280.0`), `height: f64` (`[240.0, 20000.0]`, default `800.0`), `left: Option<f64>` (`None`), `top: Option<f64>` (`None`), `is_maximized: bool` (`false`).
  - `display`: `mode: DisplayMode` (`Original | FitWidth | FitHeight | Fill`, default `Fill`), `top_bar_pinned: bool` (`true`), `bottom_bar_pinned: bool` (`true`), `info_panel_visible: bool` (`false`), `play_animations: bool` (`true`), `background: ViewerBackground` (`Black | DarkGray | Gray | White | Checkerboard`, default `Black`), `filter: ImageFilter` (`None | Smooth | SmoothSharpen`, default `None`), `mouse_wheel: MouseWheelAction` (`Zoom | Navigate`, default `Navigate`).
  - `navigation`: `wrap_around: bool` (`true`), `watch_folder: bool` (`true`).
  - `slideshow`: `interval_seconds: i32` (`[5, 90]`, default `5`), `mode: SlideShowMode` (`Sequential | Random`, default `Sequential`), `repeat: bool` (`true`).
  - `thumbnail`: `visible: bool` (`true`), `size: i32` (`[48, 256]`, default `96`, UI step `8`), `show_scroll_bar: bool` (`false`).
  - `filter`: `smooth_sigma: f64` (`[0.1, 3.0]`, default `0.5`, UI `[0.1, 2.0]`), `sharpen_sigma: f64` (`[0.1, 5.0]`, default `1.0`, UI `[0.1, 3.0]`), `sharpen_amount: f64` (`[0.1, 5.0]`, default `1.0`, UI `[0.1, 3.0]`).
  - `limits`: `max_image_megapixels: i32` (`[10, 1000]`, default `150`, UI `[10, 500]`), `max_zip_entry_megabytes: i32` (`[10, 4096]`, default `512`).
  - `update`: `auto_check: bool` (`true`), `last_check_utc: Option<DateTime<Utc>>` (`None`), `skipped_version: Option<String>` (`<= 32` ASCII version chars, no whitespace/control chars).
- **Persistence & Concurrency Rules**:
  - Reject files `> 65,536` bytes (`64 KB`) or symlink targets -> return `ViewerSettings::default().normalize()`.
  - Validate every `f64` with `f64::is_finite` prior to range clamping (non-finite `left`/`top` reset to `None`).
  - Concurrent multi-instance saves acquire advisory file lock (`fs4`), **re-read the latest disk `settings.json`**, and selectively merge dirty fields (Settings-dialog OK / `SkipVersion` persist preference sections directly, while window close/move updates only `window` fields) via randomized temp file (`settings.<rand>.tmp`) + atomic rename + orphan cleanup, preventing Lost Updates across concurrent windows.
  - On Wayland, `left`/`top` restore is skipped safely without corrupting or clearing stored `left`/`top` on shared configs.

---

## 4. `MediaEntry`, `DecodedFrame`, `AnimationSession`, and `SessionState`

- **`MediaEntry` (`ImageInfo`)**:
  - `id: u64`, `name: String`, `source_key: String` (filesystem path or archive entry key), `source_type: ImageSourceType` (`File | Zip`), `byte_size: u64`, `modified_utc: Option<i64>`, `is_supported: bool`. Sorted with locale-neutral natural ordering (`image2 < image10`).
- **Pre-Decode Validation Guardrails**:
  - Binary sniffer inspects `<= 8,192` bytes (UTF-32 LE/BE -> UTF-16 LE/BE -> UTF-8 -> BOM-less UTF-16 -> leading whitespace/reject `>= 512`-byte pure whitespace -> `<`, `<?xml`, `<svg`, `<html`, `%!`, `%PDF`, `push `, `viewbox`) **before** both pure-Rust core and host OS codecs.
  - Archive caps: pre-flight EOCD/ZIP64 EOCD scan **before** initializing `ZipArchive::new` enforces `<= 50,000` entries, `<= 64 MB` central directory metadata, and `<= 4 GB` archive container size, paired with `<= max_zip_entry_bytes` chunked stream read.
  - Dimension & arithmetic check (`checked_mul(width, 4)` + `width * height <= max_pixels`) and EXIF `Orientation` (`1–8`, swapping `width`/`height` for tags `5–8` cleanly) verified prior to allocating pixel buffers across static images, `.ico`/`.cur` largest-frame selection, multi-page TIFF per-page IFDs, and animation frames.
- **`OsCodecProvider` & `OsDecodedBuffer` (`iv-core`)**:
  - Defined in `crates/iv-core` (`#![forbid(unsafe_code)]`) to preserve an acyclic Cargo DAG (`iv-media -> iv-core`, `iv-platform -> iv-core`): `OsDecodedBuffer { width: u32, height: u32, rgba8: Vec<u8>, orientation_applied: bool, srgb_converted: bool, exif_bytes: Option<Vec<u8>> }`. `iv-media` runs EXIF `apply_orientation` / `moxcms` ICC->sRGB ONLY when `!orientation_applied` / `!srgb_converted` (preventing double-rotation or double-sRGB on macOS `ImageIO` / Windows `WIC`).
- **`DecodedFrame` & Caches**:
  - Immutable `Arc<DecodedFrame>` (`source_width`, `source_height` after EXIF `5–8` swap for accurate HUD/EXIF/scale readout even when reduced; `buffer_width`, `buffer_height`, `rgba8` pixel buffer or single/tiled GPU texture handle splitting any axis exceeding `device.limits().max_texture_dimension_2d` into `<= tile_dim` sub-textures; `pixel_bytes`, `is_downscaled`, `page_count`, `exif_orientation_applied: bool`, `exif_items: Vec<MetadataItem>`, `color_profile_converted: bool`).
  - **EXIF Orientation, Multi-Page TIFF & Color/HDR Normalization**: EXIF `Orientation` (`2–8`) transformations apply cleanly during decode (`AutoOrient` parity, evaluated **per page** on multi-page TIFFs while preserving user `R`/`L`/`H`/`V` transforms across `PgUp`/`PgDn`); `.exr`/`.hdr` pass through automatic exposure + soft Reinhard/ACES tone mapping + sRGB gamma; 16-bit integer formats normalize to display pixels; non-sRGB/CMYK (including TIFF `Cmyk16`) ICC profiles convert to sRGB (`moxcms`).
  - **Cache Budgets**: `DecodedFrame` LRU max `8` entries; rendered/GPU texture LRU bounded; thumbnail cache max `300` entries (`ThumbnailItemViewModel` with `Arc` thumbnails).
- **`AnimationStreamState`**:
  - Pre-decode all frames into RAM if estimated frame buffer `<= 512 MB` (downscaling to display viewport size if needed); otherwise lazy-decode stream maintaining `<= 128 MB` lookahead ring buffer ahead of playback cursor and evicting passed frames.
  - Wall-clock elapsed-time scheduler: `GIF` frames with delay `<= 10 ms` -> `100 ms`; `WebP` preserves per-frame delays (including `1 ms`) unless *every* frame is `<= 10 ms` (then `100 ms`); `APNG` executes full frame compositing (`DisposeOp::None | Background | Previous` + `BlendOp::Source | Over` on persistent canvas) and rational frame duration (`delay_num / delay_den` with `delay_den == 0 -> 100`, `<= 10 ms -> 100 ms`). Display filters bypassed on animations.
- **`SessionState` Transitions**:
  - `Idle -> Opening(open_gen) -> Loaded(index, load_gen) -> Rendering(render_gen) -> Closed`
  - Monotonic generation counters (`open_gen`, `load_gen`, `render_gen`) + `CancellationToken` discard stale decodes immediately so newly selected images never queue behind obsolete heavy files.
  - Placeholder timer (`250 ms`) shows scaled cached thumbnail unless `Original` mode or missing thumbnail; spinner timer (`500 ms`) triggers loading indicator; slow-render timer (`>= 300 ms`) logs redacted timing breakdown.

---

## 5. `ViewportState`, `ChromeState`, and `TrackpadAccumulator`

- **`ViewportState`**: `display_mode`, `zoom_scale`, `pan_x`, `pan_y`, `dpi_scale_factor` (tracks physical-to-logical surface ratio across `ScaleFactorChanged` / `wp-fractional-scale-v1`), `rotation_quarter_turns` (`0..4`), `flip_h`, `flip_v`. Transforms are screen-only and reset on image change (preserved across multi-page TIFF `PgUp`/`PgDn`). Filter `sigma` scales by rendered `display_scale` (`sigma / max(scale, eps)`).
- **`ChromeState`**: `top_bar_pinned`, `bottom_bar_pinned`, `hover_top`, `hover_bottom`, `is_fullscreen`, `info_panel_visible`, `saved_window_geometry` (`Option<WindowGeometry>`). Global `` ` `` toggles both pins (either pinned -> unpin both; both unpinned -> pin both).
- **`TrackpadAccumulator`**: Accumulates high-frequency pixel scroll deltas (`acc_y`) per input window, triggering discrete `Navigate` steps only upon crossing threshold (`>= 48.0` logical px equivalent) + short debounce cooldown, while resetting smoothly when gesture ends; forwards native `Pinch(delta)` continuously to cursor-anchored zoom.

---

## 6. `ReleaseInfo`, `UpdatePackage`, and `SanitizedLogRecord`

- **`ReleaseInfo` & `UpdatePackage`**: `version` (SemVer/3-part tag, pre-releases/drafts ignored), `tag`, `page_url` (`https://` required), `notes`, `assets`, `asset_download_url` (`https://` required), `sha256` (mandatory 64-hex lowercase digest from GitHub asset digest or `SHA256SUMS.txt`). HTTP body limits: `1 MB` JSON, `64 KB` sums, `256 MB` binary.
- **`SanitizedLogRecord`**: `timestamp`, `level`, `target`, `sanitized_message` (`PathRedactor` strips host dir prefixes to filename-only + strips `\r`/`\n`/control chars). Bounded non-blocking writer drops oldest on overflow (`DropOldest`), retains 7 daily files, flushes pending shutdown logs (`<= 2s` timeout) before exiting.
