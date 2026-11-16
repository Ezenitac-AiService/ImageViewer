# ImageViewer

> [!NOTE]
> **Independent Cross-Platform Fork Notice**
>
> This repository (**[Ezenitac-AiService/ImageViewer](https://github.com/Ezenitac-AiService/ImageViewer)**) is an independent cross-platform Rust fork derived from **[edith2580/ImageViewer](https://github.com/edith2580/ImageViewer)** (`v1.0.6`, Windows C#/.NET), respectfully preserving the original `GPL-3.0-only` license and `edith2580` copyright.
>
> - The pre-cutover Windows C#/.NET `v1.0.6` baseline is preserved under tag [`legacy-v1.0.6`](https://github.com/Ezenitac-AiService/ImageViewer/tree/legacy-v1.0.6).
> - **Please do NOT report cross-platform Rust fork issues to the upstream `edith2580/ImageViewer` maintainer.** Open all bug reports and feature requests here at [Ezenitac-AiService/ImageViewer/issues](https://github.com/Ezenitac-AiService/ImageViewer/issues).
> - Looking for the original Windows-only C#/.NET version? Visit [edith2580/ImageViewer](https://github.com/edith2580/ImageViewer).

A fast, GPU-accelerated, Zero-Trust cross-platform image and comic archive viewer written in Rust (`winit`, `wgpu`, `egui`), supporting **Windows**, **macOS**, and **Linux**.

## Key Features

- **Wide Format Support**: JPEG (`zune-jpeg`), PNG, APNG, GIF, WebP, BMP, TIFF, ICO/CUR, JPEG XL (`jxl-oxide`), PSD, DDS, TGA, QOI, OpenEXR (`exr`), Radiance HDR, AVIF/HEIC, RAW previews, and `.zip` / `.cbz` archives.
- **GPU Rendering & Color Management**: `wgpu` rendering (Metal / DX12 / Vulkan), SIMD-accelerated resampling (`fast_image_resize`), and ICC color profile conversion (`moxcms`).
- **Two-Page Spread & Reading Modes**: Single page, two-page spread, RTL/LTR manga direction, fit-to-window / 1:1 / free zoom, slideshow, and borderless fullscreen.
- **Zero-Trust Security**: Strict path-traversal and symlink/junction guards, bounded archive extraction and HTTP update payloads, HTTPS-only update checks with SHA-256 verification (`Ezenitac-AiService/ImageViewer` releases only).
- **Config Continuity**: Preserves the `ImageViewer` config folder (`settings.json`) for seamless migration from `v1.0.6`.
- **Bilingual UI**: Korean (`ko`) and English (`en`).

## Supported Platforms

| OS | Architecture | Packaging |
|---|---|---|
| Windows 10 / 11 | `x86_64` | `ImageViewer-windows-x64.exe` / NSIS installer (`packaging/windows/package-windows.ps1`) |
| macOS | `aarch64` / `x86_64` | `ImageViewer-macos.dmg` (`packaging/macos/package-dmg.sh`) |
| Linux (X11 / Wayland) | `x86_64` | `ImageViewer-linux-x64.tar.gz` (`packaging/linux/package-appimage.sh`) |

## Build & Run from Source

### Prerequisites
- Rust toolchain `1.85+` (Edition 2024)
- On Linux: `libwayland-dev libxkbcommon-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`

### Commands

```bash
# Run the viewer
cargo run -p iv-app --release

# Run workspace checks, strict lints, and tests
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## License & Attribution

- **License**: GNU General Public License v3.0 (`GPL-3.0-only`) — see [LICENSE](LICENSE).
- **Copyright & Modification Notice (GPLv3 §5)**: See [NOTICE](NOTICE) and [CHANGELOG.md](CHANGELOG.md).
- **Third-Party Rust Frameworks & Libraries**: See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
