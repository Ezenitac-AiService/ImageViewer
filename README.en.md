# ImageViewer

**Language / 언어**: [한국어](README.md) | **[English](README.en.md)**

[![CI](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/ci.yml/badge.svg)](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/ci.yml)
[![Release Build](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/release.yml/badge.svg)](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/Ezenitac-AiService/ImageViewer?include_prereleases&sort=semver&label=release&logo=github)](https://github.com/Ezenitac-AiService/ImageViewer/releases)
[![License: GPL-3.0-only](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Rust: 1.85+ (Edition 2024)](https://img.shields.io/badge/rust-1.85%2B%20(2024)-orange.svg?logo=rust)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](#supported-platforms--releases)
[![Upstream Origin](https://img.shields.io/badge/upstream%20origin-edith2580%2FImageViewer%20(v1.0.6)-informational?logo=github)](https://github.com/edith2580/ImageViewer)

A fast, GPU-accelerated, Zero-Trust cross-platform image and comic archive viewer written in Rust (`winit`, `wgpu`, `egui`), supporting **Windows**, **macOS**, and **Linux**.

> [!NOTE]
> **Independent Cross-Platform Fork Notice & Etiquette**
>
> This repository (**[Ezenitac-AiService/ImageViewer](https://github.com/Ezenitac-AiService/ImageViewer)**) is an independent cross-platform Rust fork derived from **[edith2580/ImageViewer](https://github.com/edith2580/ImageViewer)** (`v1.0.6`, Windows C#/.NET), respectfully preserving the original `GPL-3.0-only` license and `edith2580` copyright.
>
> - **Why this fork exists**: Re-architected from the ground up in pure Rust (`iv-core`, `iv-media`, `iv-platform`, `iv-app`) to run natively across Windows, macOS, and Linux with GPU rendering and Zero-Trust path/archive/update hardening.
> - **Legacy baseline**: The pre-cutover Windows C#/.NET `v1.0.6` baseline is preserved under tag [`legacy-v1.0.6`](https://github.com/Ezenitac-AiService/ImageViewer/tree/legacy-v1.0.6).
> - **Issue routing etiquette**: **Please do NOT report cross-platform Rust fork issues to the upstream `edith2580/ImageViewer` maintainer.** Open all bug reports and feature requests here at [Ezenitac-AiService/ImageViewer/issues](https://github.com/Ezenitac-AiService/ImageViewer/issues).
> - **Looking for the original Windows C#/.NET version?** Visit [edith2580/ImageViewer](https://github.com/edith2580/ImageViewer).

---

## What's Different from Upstream (`v1.0.6`)

| Area | Upstream (`edith2580/ImageViewer` `v1.0.6`) | Independent Fork (`Ezenitac-AiService/ImageViewer`) |
|---|---|---|
| **Target OS** | Windows (`x64`) | **Windows**, **macOS** (`arm64`/`x64`), **Linux** (X11/Wayland) |
| **Stack & Rendering** | C# / .NET WPF + WIC / GDI+ | **Rust (Edition 2024)** + `winit` / `wgpu` (Metal, DX12, Vulkan) / `egui` |
| **Color & Resampling** | Windows WIC | Cross-platform ICC profiles (`moxcms`) + SIMD resampling (`fast_image_resize`) |
| **Update & Security** | Windows installer updater | Isolated HTTPS-only feed (`Ezenitac-AiService/ImageViewer`), OS-matching asset selection, `SHA256SUMS.txt` verification |
| **Settings Continuity** | `%LOCALAPPDATA%\ImageViewer\settings.json` | Preserves `ImageViewer/settings.json` with automatic `v1.0.6` camelCase migration |

---

## Key Features

- **Wide Format Support**: JPEG (`zune-jpeg`), PNG, APNG, GIF, WebP, BMP, TIFF, ICO/CUR, JPEG XL (`jxl-oxide`), PSD, DDS, TGA, QOI, OpenEXR (`exr`), Radiance HDR, AVIF/HEIC, RAW previews, and `.zip` / `.cbz` archives.
- **GPU Rendering & Color Management**: `wgpu` hardware-accelerated rendering, SIMD resampling (`fast_image_resize`), and ICC color profile conversion (`moxcms`).
- **Two-Page Spread & Reading Modes**: Single page, two-page spread, RTL/LTR manga direction, fit-to-window / 1:1 / free zoom, slideshow, and borderless fullscreen.
- **Zero-Trust Security**: Strict path-traversal and symlink/junction guards, bounded archive extraction and HTTP update payloads, HTTPS-only update checks with SHA-256 verification.
- **Bilingual UI**: Korean (`ko`) and English (`en`).

---

## Supported Platforms & Releases

Download pre-built binaries and verify checksums (`SHA256SUMS.txt`) on the [Releases page](https://github.com/Ezenitac-AiService/ImageViewer/releases).

| OS | Architecture | Release Artifact | Packaging Script |
|---|---|---|---|
| **Windows 10 / 11** | `x86_64` | `ImageViewer-windows-x64.exe` | `packaging/windows/package-windows.ps1` (`ImageViewer.nsi`) |
| **macOS** | `aarch64` / `x86_64` | `ImageViewer-macos.dmg` | `packaging/macos/package-dmg.sh` |
| **Linux (X11 / Wayland)** | `x86_64` | `ImageViewer-linux-x64.tar.gz` | `packaging/linux/package-appimage.sh` |

### Verifying Release Integrity

```bash
sha256sum -c SHA256SUMS.txt --ignore-missing
```

---

## Keyboard Shortcuts

| Action | Shortcut |
|---|---|
| Open File / Folder | `Ctrl+O` / `Cmd+O` |
| Next / Previous Image | `Right` / `Left`, `PageDown` / `PageUp`, `Space` |
| First / Last Image | `Home` / `End` |
| Toggle Two-Page Spread | `D` |
| Toggle Reading Direction (LTR / RTL) | `R` |
| Toggle Fullscreen | `F11` / `Enter` |
| Zoom In / Out / Reset (1:1 / Fit) | `+` / `-` / `0` / `F` |
| Toggle Slideshow | `S` |
| Settings / About | `F2` / `F1` |

---

## Build & Run from Source

### Prerequisites
- Rust toolchain `1.85+` (Edition 2024)
- On Linux: `libwayland-dev libxkbcommon-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`

### Commands

```bash
# Run the viewer in release mode
cargo run -p iv-app --release

# Run workspace checks, strict lints, and tests
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

---

## Contributing & Security

- **Contributing**: See [CONTRIBUTING.en.md](CONTRIBUTING.en.md) ([한국어](CONTRIBUTING.md)) for our independent detached-fork workflow (`no upstream merge` policy) and quality gates.
- **Security**: See [SECURITY.en.md](SECURITY.en.md) ([한국어](SECURITY.md)) for private vulnerability reporting on `Ezenitac-AiService/ImageViewer`.

## License & Attribution

- **License**: GNU General Public License v3.0 (`GPL-3.0-only`) — see [LICENSE](LICENSE).
- **Copyright & Modification Notice (GPLv3 §5)**: See [NOTICE.en.md](NOTICE.en.md) ([한국어](NOTICE)) and [CHANGELOG.en.md](CHANGELOG.en.md) ([한국어](CHANGELOG.md)).
- **Third-Party Rust Frameworks & Libraries**: Verified in CI via [`deny.toml`](deny.toml) (`cargo-deny`) — see [THIRD_PARTY_NOTICES.en.md](THIRD_PARTY_NOTICES.en.md) ([한국어](THIRD_PARTY_NOTICES.md)).
