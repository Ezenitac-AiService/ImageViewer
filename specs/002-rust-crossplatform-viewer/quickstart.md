# Quickstart & Cross-Platform Validation Guide: Rust ImageViewer

**Feature**: `002-rust-crossplatform-viewer`
**Spec**: [spec.md](spec.md)
**Data Model**: [data-model.md](data-model.md)
**Contracts**: [contracts/](contracts)

---

## 1. Prerequisites & Multi-OS Environment Matrix

- **Toolchain**: Stable Rust (`rustc >= 1.85`, host verified on `rustc 1.100.0` / `cargo 1.100.0` `aarch64-apple-darwin`), Cargo workspace resolver `2`.
- **Target Verification Hosts**:
  - **macOS (`aarch64-apple-darwin` / `x86_64-apple-darwin`)**: Native execution for core tests, Metal `iced`/`wgpu` rendering, `ImageIO` codecs (`HEIC`/`RAW`), `.app` Finder integration, and trackpad gestures.
  - **Windows 10/11 (`x86_64-pc-windows-msvc`)**: Native execution for DX12 `iced`/`wgpu` rendering, `WIC` codecs, HKCU Explorer verbs (`--register-shell`), Default Apps registration, NSIS installer, and Portable self-swap tests.
  - **Linux (`x86_64-unknown-linux-gnu`, X11 & Wayland)**: Native execution for Vulkan/GLES `wgpu`, Wayland `Width`/`Height`/`IsMaximized`-only restore, compositor `drag_window()`, drag-and-drop, `.desktop`/MIME integration, and missing-codec graceful error handling.
- **Transparency Rule (Constitution Principle V & VI)**: Running `cargo test --workspace` on macOS verifies all cross-platform core, media, security, layout, and macOS adapter suites natively; Windows (`WIC`/HKCU/NSIS) and Linux (Wayland/X11/XDG) paths MUST be verified on their respective hosts or multi-OS GitHub Actions runners (`windows-latest`, `ubuntu-latest`, `macos-latest`) before claiming full 3-OS release pass.

---

## 2. Build, Lint, Benchmark & Test Commands

```bash
# 1. Static check and strict lints across entire workspace
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings

# 2. Run all unit & integration tests headlessly (security, formats, zip, settings, layout, session)
cargo test --workspace

# 3. Release build & performance benchmarks (opt-level = 3 for zune-jpeg/fast_image_resize SIMD)
cargo build --release -p iv-app
cargo bench --workspace
```

---

## 3. End-to-End Verification Scenarios

1. **Zero Trust Security & Privacy (`SC-005`, `contracts/core-security-and-cli-contract.md`)**:
   - Feed UTF-32 LE/BE (`FF FE 00 00` before `FF FE`), UTF-16 LE/BE, and `> 512`-byte space-padded `<svg>` / `<?xml>` / `%PDF` / `push ` payloads disguised as `.png` or `.heic` -> verify binary sniffer blocks before both core decoders and `OsCodecProvider` (`WIC`/`ImageIO`).
   - Feed malicious `.zip`/`.cbz` archives exceeding `50,000` entries, `64 MB` central directory, `4 GB` container size, `MaxZipEntryMegabytes`, Zip-Slip `../` traversal, POSIX `0xA000` symlinks, or encrypted entries -> verify fail-closed safe errors without crashes or disk writes.
   - Verify legacy `v1.0.6` `settings.json` (`camelCase`, string enums, comments/trailing commas, `> 64 KB`, `NaN`/`Infinity`, concurrent saves), strict `https://` + SHA-256 updater rejections, and filename-only redacted diagnostic logs (`\r`/`\n` stripped).
2. **Performance Parity & Benchmarks (`SC-001`, `SC-002`, `SC-003`, `contracts/media-pipeline-and-os-codecs-contract.md`)**:
   - **Baseline Gallery Benchmark**: Measure 32-image mixed JPEG/PNG archive (up to `2592×1728`) in Fit Width (`1280×800`) and Fullscreen (`2560×1440`) on modern desktop hardware (Apple Silicon M-series / modern 8-core x64 NVMe SSD) -> verify `<= 36 ms` avg (`<= 95 ms` max) windowed, `<= 49 ms` avg (`<= 137 ms` max) fullscreen, and `<= 22 ms` avg rapid navigation (`100–150 ms` cadence) + `<= 16 ms` slideshow step.
   - **Large Animation Benchmark**: Load `1080×1920`, `267`-frame animated WebP -> verify frame 1 visible `< 250 ms`, total RAM `<= 470 MB`, steady playback CPU `<= 5%`, and zero timer drift.
3. **Cross-Platform UX & OS Integration (`SC-004`, `SC-006`, `contracts/ui-chrome-shortcuts-and-os-integration-contract.md`)**:
   - Verify frameless chrome, `Top`/`Bottom` pin toggle & `` ` ``, client edge hover, screen-pixel `Smooth`/`SmoothSharpen` filters, multi-page TIFF `PgUp`/`PgDn`, EXIF 13-field panel (`Tab`), Image List dialog (`Enter`), precision trackpad accumulated scroll & pinch-to-zoom, Wayland geometry fallback, `Cmd`/`Ctrl` shortcuts, `ko`/`en` switching, and OS-specific associations/updates on Windows, macOS, and Linux.
