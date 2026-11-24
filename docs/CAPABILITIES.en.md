# Capability Register

**Language / 언어**: [한국어](CAPABILITIES.md) | **[English](CAPABILITIES.en.md)**

ImageViewer ships **only capabilities that have evidence behind them**. Every capability has one of three tiers:

- **Core**: must be verified on active top-priority targets (`macOS`; `iOS` is planned for feature `007`). `Linux` (and planned `Android`) are lower-priority **build-only, unverified** targets; `Windows` is excluded under Constitution `v3.0.0`.
- **Extended**: verified one by one.
- **Unavailable**: not provided in this version. It does not appear in the app or the documentation.

Moving an Unavailable capability to Extended requires all four of:

1. automated tests with real sample files;
2. automated tests with hostile or malformed input;
3. launch-verification records on active top-priority targets (`macOS`, and `iOS` once feature `007` lands);
4. user documentation in Korean and English.

A capability drops back to Unavailable as soon as any of that evidence becomes invalid. The tier rules come from the Media Decoding Strategy in the [project constitution](../.specify/memory/constitution.md) (`v3.0.0`); full definitions are in [specs/006](../specs/006-platform-tier-realignment/spec.md) and [specs/004](../specs/004-launch-reliability-refactor/spec.md) FR-020–FR-024.

> **Verification status (2026-11-24)**: automated tests and the launch check (`ImageViewer.app`) pass on macOS Apple Silicon (local and CI). Linux (`x86_64`) passes compile/test checks in CI (`Checks (ubuntu-latest)`) with zero runtime or launch verification, and Windows is excluded under Constitution `v3.0.0`. **Clean Apple Silicon/Intel Macs and first-launch Gatekeeper confirmation screens have not been checked yet.** No release is published until the clean-device checklist passes.

## Core

| id | Capability | Visible | Evidence | Checked on |
|---|---|---|---|---|
| `launch` | Downloaded file starts, failures explained with a diagnostic record | yes | `diagnostics_tests`, `panic_hook_tests`, `settings_tests::load_outcome`, `packaging/verify/launch-check.sh` | macOS arm64 |
| `open-file-dialog` | Open file or archive dialog | yes | `interaction_tests` (Primary+O) | macOS arm64 |
| `open-folder-dialog` | Open folder dialog | yes | `interaction_tests` (Primary+Shift+O) | macOS arm64 |
| `open-drag-drop` | Drop a file on the window | yes | `interaction_tests::dropped_file_opens_that_path` | not yet (manual checklist) |
| `open-command-line` | Path passed at launch | yes | `cli_tests`, `launch-check.sh` | macOS arm64 |
| `open-with-os` | macOS Finder "Open With" | yes | bundled cold/warm open records (research.md S2) | macOS arm64 |
| `navigate` | Previous, next, first, last; mouse wheel, trackpad, back/forward buttons | yes | `interaction_tests`, `interaction_model_tests`, `launch-check.sh` key step | macOS arm64 |
| `zoom-pan` | Zoom and pan | yes | `interaction_tests`, `viewport_and_chrome_tests` | macOS arm64 |
| `fit-modes` | Original, fit width, fit height, fit window (`0`/`7`/`8`/`9`) | yes | `interaction_tests`, `viewport_and_chrome_tests` | macOS arm64 |
| `rotate-flip` | Rotate, flip horizontally and vertically | yes | `viewport_and_chrome_tests::image_quad_*` | macOS arm64 |
| `fullscreen` | Toggle fullscreen | yes | `interaction_tests` | not yet (manual checklist) |
| `format-jpeg` `format-png` `format-gif` `format-webp` `format-bmp` `format-tiff` | Still-image decoding, identified by content signature | yes | `format_guard_tests::identify`, `decode_color_resize_tests` | macOS arm64 |
| `animation` | Animated GIF and WebP playback | yes | `animation_and_multipage_tests` | not yet (manual checklist) |
| `archive-zip` | Read-only ZIP and CBZ browsing (stored, deflate, zstd) | yes | `sources_tests` (path escape, symlinks, entry count, zip bomb, encryption) | macOS arm64 |
| `settings` | Settings window, saving, notice when settings were damaged | yes | `settings_tests` | macOS arm64 |
| `language` | Korean and English; first launch follows the OS language | yes | `settings_tests::first_launch_language`, `logger_and_i18n_tests` | macOS arm64 |
| `filter-basic` | Magnified display: pixelated (`U`) or smooth (`I`) | yes | `interaction_tests` | macOS arm64 |

## Extended (candidates)

These stay only if they pass the clean-device check on active top-priority targets (`macOS`, and `iOS` once feature `007` lands) before the release. If they fail, they are hidden through `EXTENDED` in `crates/iv-app/src/app.rs`.

| id | Capability | Visible | Evidence | Known limitation |
|---|---|---|---|---|
| `slideshow` | Slideshow (`Space`) | yes | `filter_and_slideshow_tests`, `interaction_tests` | — |
| `info-panel` | Image info and EXIF panel (`Tab`) | yes | `exif_thumbnail_watch_tests` | — |
| `multipage-tiff` | Multi-page TIFF paging (`PageUp`/`PageDown`) | yes | `animation_and_multipage_tests`, `interaction_model_tests` | — |
| `image-list` | Image list window (`Enter`) | yes | `interaction_tests` | Sorting applies to the list only, not to navigation order |

## Unavailable

| id | Capability | Note |
|---|---|---|
| `updater` | Update check and install | The shipped build makes no network connections. Code kept behind the `iv-platform` `updater` feature |
| `format-extended` | JPEG XL, PSD, DDS, PCX, CUR, ICO, QOI, PNM, HDR, EXR, TGA | Kept and tested behind the `iv-media` `extended-formats` feature |
| `os-codecs` | Camera RAW, HEIC/HEIF, AVIF, JPEG 2000 (OS codecs) | Kept behind the `iv-platform` `os-codecs` feature (`ImageIO`) |
| `shell-registration` | OS shell context-menu and file-association registration (`--register-shell`) | The switches explain they are not available in this version |
| `archive-bzip2` | bzip2-compressed ZIP entries | Excluded pending license review |
| `thumbnail-strip` | Bottom thumbnail strip | The previous implementation only showed file names |
| `filter-sharpen` | Smooth + sharpen filter (`S`) | |
| `custom-window-chrome` | Custom title bar | Native OS window decorations are used |
| `accessibility` | Screen-reader support (AccessKit) | Excluded by the dependency budget; a known accessibility gap |
