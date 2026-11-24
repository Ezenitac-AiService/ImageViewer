# Changelog

**Language / 언어**: [한국어](CHANGELOG.md) | **[English](CHANGELOG.en.md)**

All notable changes to this project are documented in this file.

## [1.1.0-alpha.2] - Unreleased (launch reliability refactor & platform tier realignment)

Fixes `v1.1.0-alpha.1` being unusable on macOS and realigns platform support tiers under Constitution `v3.0.0`. Modification notice under GPLv3 §5(a): modified by `Ezenitac-AiService` on 2026-11-16–17 and 2026-11-24.

### Fixed
- **macOS**: the window did not respond to input and the downloaded app appeared "damaged". The hand-written window and rendering code was replaced with `eframe`, and the whole bundle is ad-hoc signed and shipped as a universal Apple Silicon and Intel file.
- Startup failures exited silently. The app now shows a window explaining the cause and writes a diagnostic record.
- A race in which concurrent settings saves from several instances could fail.
- A folder change jumped back to the first image that was opened.

### Changed
- **Platform Tier Realignment (2026-11-24, Constitution v3.0.0)**:
  - **Top Priority (build-result and launch verified)**: `macOS` (`aarch64` & `x86_64`) is the active verified target; `iOS` (`arm64`) follows in feature `007` (planned).
  - **Lower Priority (build-only, unverified)**: `Linux` (`x86_64`) retains CI compilation/unit checks and ships a build-only tarball with zero runtime/launch verification (`Android` planned build-only follows in `007`).
  - **Excluded**: removed `Windows` targets, packaging, launch checks, and Windows-only dependencies (`windows`, `winreg`, `winresource`) from active builds (pre-cutover Windows C#/.NET `v1.0.6` remains archived at `legacy-v1.0.6`).
- This version ships verified capabilities only ([capability register](docs/CAPABILITIES.en.md)). Update checks, extended formats, OS codecs, shell registration, and the thumbnail strip are excluded, and the app makes no network connections.
- File formats are identified from content rather than extension, and files over 512 MB are refused before they are read.
- The first launch follows the OS language (Korean or English); the language can be changed in Settings.
- macOS Finder "Open With" is supported.
- Diagnostic record timestamps are UTC.

### Security
- Upgraded to `zip` 9 (4 advisories), `lru` 0.19 (2 unsoundness advisories), and `notify` 8 (removes an unmaintained dependency).
- Releases are started and checked on top-priority targets (`macOS`), and no draft is created if `cargo deny` detects any known vulnerability.

## [1.1.0-alpha.1] - 2026-11-16 (Independent Cross-Platform Rust Fork — Alpha Pre-release)

### Changed
- **Independent Fork Separation**: Separated `Ezenitac-AiService/ImageViewer` as an independent cross-platform fork derived from [edith2580/ImageViewer](https://github.com/edith2580/ImageViewer) (`v1.0.6`).
- **Cross-Platform Rust Rewrite**: Replaced the Windows-only C#/.NET codebase with a 4-crate Rust workspace (`iv-core`, `iv-media`, `iv-platform`, `iv-app`) targeting Windows, macOS, and Linux.
- **License Harmonization**: Standardized Cargo workspace metadata to `GPL-3.0-only` matching the root `LICENSE` (GNU GPLv3), added `NOTICE` (`NOTICE.en.md`), `THIRD_PARTY_NOTICES.md` (`THIRD_PARTY_NOTICES.en.md`), and `deny.toml` (`cargo-deny` license gate).
- **Update & Identity Isolation**: Isolated auto-update feeds to `Ezenitac-AiService/ImageViewer` with OS-matching release asset selection and updated macOS bundle identifier to `com.ezenitac.imageviewer` while preserving the `ImageViewer` config folder name for `settings.json` continuity.
- **Legacy Archival**: Preserved the pre-cutover Windows C#/.NET `v1.0.6` baseline under immutable git tag `legacy-v1.0.6`.

## [1.0.6] - Upstream Legacy Baseline

- Original Windows C#/.NET release by `edith2580` (`edith2580/ImageViewer`), archived at git tag `legacy-v1.0.6`.
