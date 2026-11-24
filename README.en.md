# ImageViewer

**Language / 언어**: [한국어](README.md) | **[English](README.en.md)**

[![CI](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/ci.yml/badge.svg)](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/ci.yml)
[![Release Build](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/release.yml/badge.svg)](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/Ezenitac-AiService/ImageViewer?include_prereleases&sort=semver&label=release&logo=github)](https://github.com/Ezenitac-AiService/ImageViewer/releases)
[![License: GPL-3.0-only](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Rust: 1.95+ (Edition 2024)](https://img.shields.io/badge/rust-1.95%2B%20(2024)-orange.svg?logo=rust)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![Platform](https://img.shields.io/badge/platform-macOS%20(Top%20Priority)%20%7C%20Linux%20(Build--Only)-lightgrey.svg)](#supported-platforms--releases)
[![Upstream Origin](https://img.shields.io/badge/upstream%20origin-edith2580%2FImageViewer%20(v1.0.6)-informational?logo=github)](https://github.com/edith2580/ImageViewer)

An image and comic archive (`.zip`/`.cbz`) viewer targeting **macOS** (top priority, verified) and **Linux** (lower priority, build-only, unverified). **iOS** (planned top priority) and **Android** (planned build-only) follow in feature `007` (not yet built), while **Windows is excluded under Constitution v3.0.0** (see [`legacy-v1.0.6`](https://github.com/Ezenitac-AiService/ImageViewer/tree/legacy-v1.0.6)). Written in Rust on top of `eframe`/`egui` and `wgpu`.

> [!IMPORTANT]
> **First launch**: releases are not notarized by Apple yet, so macOS Gatekeeper asks once after you download. Follow [First Launch](docs/FIRST_LAUNCH.en.md).

> [!NOTE]
> **Independent Cross-Platform Fork Notice & Etiquette**
>
> This repository (**[Ezenitac-AiService/ImageViewer](https://github.com/Ezenitac-AiService/ImageViewer)**) is an independent cross-platform Rust fork derived from **[edith2580/ImageViewer](https://github.com/edith2580/ImageViewer)** (`v1.0.6`, Windows C#/.NET), respectfully preserving the original `GPL-3.0-only` license and `edith2580` copyright.
>
> - **Legacy baseline**: The pre-cutover Windows C#/.NET `v1.0.6` baseline is preserved under tag [`legacy-v1.0.6`](https://github.com/Ezenitac-AiService/ImageViewer/tree/legacy-v1.0.6).
> - **Issue routing etiquette**: **Please do NOT report cross-platform Rust fork issues to the upstream `edith2580/ImageViewer` maintainer.** Open all bug reports and feature requests here at [Ezenitac-AiService/ImageViewer/issues](https://github.com/Ezenitac-AiService/ImageViewer/issues).
> - **Looking for the original Windows C#/.NET version?** Visit [edith2580/ImageViewer](https://github.com/edith2580/ImageViewer).

---

## What's Different from Upstream (`v1.0.6`)

| Area | Upstream (`edith2580/ImageViewer` `v1.0.6`) | Independent fork (`Ezenitac-AiService/ImageViewer`) |
|---|---|---|
| **OS** | Windows (`x64`) | **macOS** (universal Apple Silicon & Intel, top priority verified), Linux `x64` (lower priority, build-only, unverified); `iOS`/`Android` planned (`007`), **Windows excluded** |
| **Stack** | C# / .NET WPF + WIC | Rust (Edition 2024) + `eframe`/`egui` + `wgpu` (with an OpenGL fallback) |
| **Formats** | Everything WIC supports | Only the verified core formats in this version ([capability register](docs/CAPABILITIES.en.md)) |
| **Updates** | Automatic updates | Not in this version; the app makes no network connections |
| **Settings** | `%LOCALAPPDATA%\ImageViewer\settings.json` | macOS `~/Library/Application Support/com.Ezenitac-AiService.ImageViewer/settings.json`, Linux `~/.config/imageviewer/settings.json` (`v1.0.6` JSON schema compatible) |

---

## What This Version Provides

The [capability register](docs/CAPABILITIES.en.md) lists every capability with its evidence. In short:

- **Images**: JPEG, PNG, GIF, WebP, BMP, TIFF. The format is identified from the content, not the file name.
- **Animation**: GIF and WebP playback
- **Archives**: read-only `.zip` / `.cbz`
- **Opening**: file and folder dialogs, drag and drop, command-line path, macOS Finder "Open With"
- **Viewing**: zoom, pan, fit modes, rotate, flip, fullscreen
- **Slideshow, image info (EXIF), multi-page TIFF, image list**: Extended candidates that stay only if they pass the pre-release top-priority check
- **Safety limits**: image size cap (150 MP by default), file size checked before reading (512 MB), archive path escapes, symlinks, zip bombs and encrypted entries refused
- **Languages**: Korean and English; the first launch follows the OS language

Update checks, extended formats (JPEG XL, PSD, HEIC, RAW, …), shell registration, and the thumbnail strip are **not included in this version.**

Performance parity with v1.0.6 has not been measured yet.

---

## Supported Platforms & Releases

Executables and checksums (`SHA256SUMS.txt`) are on the [Releases page](https://github.com/Ezenitac-AiService/ImageViewer/releases). Under Constitution v3.0.0, **build-result and launch verification runs strictly on top-priority targets (`macOS`, and planned `iOS` in feature `007`)**; `Linux` (and planned `Android`) are shipped as **lower-priority build-only artifacts with no runtime verification**, and `Windows` is excluded.

| OS | Architecture | Tier | File | Status |
|---|---|---|---|---|
| **macOS 11 or later** | Apple Silicon · Intel | Top Priority | `ImageViewer-macos-universal.dmg` | supported (automated + manual verification) |
| **iOS** | `arm64` | Top Priority (planned — `007`) | — | not yet built or released |
| **Linux** (X11 / Wayland) | `x86_64` | Lower Priority | `ImageViewer-linux-x64.tar.gz` | build-only, runtime-unverified |
| **Android** | `arm64` | Lower Priority (planned — `007`) | — | planned build-only (not yet built) |
| **Windows** | — | Excluded (`v3.0.0`) | — | excluded (archive: [`legacy-v1.0.6`](https://github.com/Ezenitac-AiService/ImageViewer/tree/legacy-v1.0.6)) |

```bash
sha256sum -c SHA256SUMS.txt --ignore-missing
```

---

## Keyboard & Mouse

`Primary` is `Cmd` on macOS and `Ctrl` on Linux.

| Action | Input |
|---|---|
| Open file / open folder | `Primary+O` / `Primary+Shift+O`, drop onto the window |
| Next / previous image | `→` `↓` / `←` `↑`, mouse wheel, mouse forward/back buttons |
| First / last | `Home` / `End` |
| Next / previous page (TIFF) | `PageDown` / `PageUp` |
| Zoom in / out | `+` `=` / `-`, `Primary`+wheel, trackpad pinch |
| Original / fit width / fit height / fit window | `0` / `7` / `8` / `9` |
| Reset view | `Primary+0` |
| Rotate / flip | `R` `L` / `H` `V` |
| Pan (when zoomed) | drag, middle-button drag, `Shift`+wheel |
| Fullscreen | `Alt+Enter`, double-click the canvas, macOS `Ctrl+Cmd+F` |
| Slideshow / image info / image list | `Space` / `Tab` / `Enter` |
| Settings | `Primary+,` |
| Close | `Esc` (closes an open dialog first) |

---

## Build & Run from Source

### Prerequisites
- Rust toolchain `1.95+` (Edition 2024)
- Linux: `libwayland-dev libxkbcommon-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`

### Commands

```bash
cargo run -p iv-app --release
```

See [CONTRIBUTING.en.md](CONTRIBUTING.en.md) for the quality checks and the definition of done.

---

## Contributing & Security

- **Contributing**: See [CONTRIBUTING.en.md](CONTRIBUTING.en.md) ([한국어](CONTRIBUTING.md)) for our independent detached-fork workflow (`no upstream merge` policy) and quality gates.
- **Security**: See [SECURITY.en.md](SECURITY.en.md) ([한국어](SECURITY.md)) for private vulnerability reporting on `Ezenitac-AiService/ImageViewer`.

## License & Attribution

- **License**: GNU General Public License v3.0 (`GPL-3.0-only`) — see [LICENSE](LICENSE).
- **Copyright & Modification Notice (GPLv3 §5)**: See [NOTICE.en.md](NOTICE.en.md) ([한국어](NOTICE)) and [CHANGELOG.en.md](CHANGELOG.en.md) ([한국어](CHANGELOG.md)).
- **Third-Party Rust Frameworks & Libraries**: Verified in CI via [`deny.toml`](deny.toml) (`cargo-deny`) — see [THIRD_PARTY_NOTICES.en.md](THIRD_PARTY_NOTICES.en.md) ([한국어](THIRD_PARTY_NOTICES.md)).
