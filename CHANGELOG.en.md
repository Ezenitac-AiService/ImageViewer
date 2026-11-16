# Changelog

**Language / 언어**: [한국어](CHANGELOG.md) | **[English](CHANGELOG.en.md)**

All notable changes to this project are documented in this file.

## [1.1.0-alpha.1] - 2026-11-16 (Independent Cross-Platform Rust Fork — Alpha Pre-release)

### Changed
- **Independent Fork Separation**: Separated `Ezenitac-AiService/ImageViewer` as an independent cross-platform fork derived from [edith2580/ImageViewer](https://github.com/edith2580/ImageViewer) (`v1.0.6`).
- **Cross-Platform Rust Rewrite**: Replaced the Windows-only C#/.NET codebase with a 4-crate Rust workspace (`iv-core`, `iv-media`, `iv-platform`, `iv-app`) targeting Windows, macOS, and Linux.
- **License Harmonization**: Standardized Cargo workspace metadata to `GPL-3.0-only` matching the root `LICENSE` (GNU GPLv3), added `NOTICE` (`NOTICE.en.md`), `THIRD_PARTY_NOTICES.md` (`THIRD_PARTY_NOTICES.en.md`), and `deny.toml` (`cargo-deny` license gate).
- **Update & Identity Isolation**: Isolated auto-update feeds to `Ezenitac-AiService/ImageViewer` with OS-matching release asset selection and updated macOS bundle identifier to `com.ezenitac.imageviewer` while preserving the `ImageViewer` config folder name for `settings.json` continuity.
- **Legacy Archival**: Preserved the pre-cutover Windows C#/.NET `v1.0.6` baseline under immutable git tag `legacy-v1.0.6`.

## [1.0.6] - Upstream Legacy Baseline

- Original Windows C#/.NET release by `edith2580` (`edith2580/ImageViewer`), archived at git tag `legacy-v1.0.6`.
