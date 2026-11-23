# Third-Party Software Notices

**Language / 언어**: [한국어](THIRD_PARTY_NOTICES.md) | **[English](THIRD_PARTY_NOTICES.en.md)**

The **shipped build** of `ImageViewer` (`Ezenitac-AiService/ImageViewer`, license `GPL-3.0-only`) directly uses the open-source Rust libraries below. Each one names the [capability register](docs/CAPABILITIES.en.md) ids that justify it (specs/004 FR-034). License compatibility, known vulnerabilities, and sources of these and all transitive dependencies are checked automatically in CI and releases with `cargo deny check advisories bans licenses sources` ([`deny.toml`](deny.toml)).

The number of packages in the shipped build, including transitive ones, is measured with `packaging/verify/component-count.sh`: 298 on 2026-11-17 (446 before this release).

## Direct Dependencies in the Shipped Build

| Crate | Purpose | Justification (capability id) | License | Upstream |
|---|---|---|---|---|
| `eframe` | Application shell (window, input, renderer wiring) | `launch`, all UI | `MIT OR Apache-2.0` | https://github.com/emilk/egui |
| `egui` | UI (toolbars, dialogs, status) | all UI | `MIT OR Apache-2.0` | https://github.com/emilk/egui |
| `epaint_default_fonts` | Bundled default UI fonts (*Ubuntu Light*, *Hack*, *Noto Emoji*) | all UI | `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0` | https://github.com/emilk/egui |
| `winit` | Window creation and event loop | `launch`, `open-with-os` | `Apache-2.0` | https://github.com/rust-windowing/winit |
| `wgpu` | GPU rendering (Metal / DX12 / Vulkan, including CPU adapters) | `launch` | `MIT OR Apache-2.0` | https://github.com/gfx-rs/wgpu |
| `glow` | OpenGL fallback renderer | `launch` | `MIT OR Apache-2.0 OR Zlib` | https://github.com/grovesNL/glow |
| `pollster` | Blocking on file-dialog and adapter futures | `open-file-dialog`, `launch` | `Apache-2.0 OR MIT` | https://github.com/zesterer/pollster |
| `rfd` | Native file/folder dialogs and the fatal-error dialog | `open-file-dialog`, `open-folder-dialog`, `launch` | `MIT` | https://github.com/PolyMeilex/rfd |
| `objc2`, `objc2-foundation`, `objc2-app-kit` | macOS "Open With" hook | `open-with-os` | `MIT; Zlib OR Apache-2.0 OR MIT (app-kit)` | https://github.com/madsmtm/objc2 |
| `sys-locale` | OS language on first launch | `language` | `MIT OR Apache-2.0` | https://github.com/1Password/sys-locale |
| `directories` | Per-OS settings and log folders | `settings`, `launch` | `MIT OR Apache-2.0` | https://github.com/soc/directories-rs |
| `serde`, `serde_json` | Settings file reading and writing | `settings` | `MIT OR Apache-2.0` | https://github.com/serde-rs/serde |
| `fs4` | File locking when saving settings | `settings` | `MIT OR Apache-2.0` | https://github.com/al8n/fs4-rs |
| `rand` | Launch IDs and temporary file names | `launch`, `settings` | `MIT OR Apache-2.0` | https://github.com/rust-random/rand |
| `thiserror` | Error type definitions | all | `MIT OR Apache-2.0` | https://github.com/dtolnay/thiserror |
| `image` | PNG, GIF, WebP, BMP, TIFF decoding | `format-*` | `MIT OR Apache-2.0` | https://github.com/image-rs/image |
| `zune-jpeg` | JPEG decoding | `format-jpeg` | `MIT OR Apache-2.0 OR Zlib` | https://github.com/etemesi254/zune-image |
| `image-webp` | Animated WebP decoding | `animation` | `MIT OR Apache-2.0` | https://github.com/image-rs/image-webp |
| `tiff` | Multi-page TIFF decoding | `format-tiff`, `multipage-tiff` | `MIT` | https://github.com/image-rs/image-tiff |
| `moxcms` | ICC colour profile to sRGB conversion | `format-*` | `BSD-3-Clause OR Apache-2.0` | https://github.com/awxkee/moxcms |
| `fast_image_resize` | Viewport-sized downscaling | `format-*` | `MIT OR Apache-2.0` | https://github.com/cykooz/fast_image_resize |
| `kamadak-exif` | EXIF orientation and metadata | `format-jpeg`, `info-panel` | `BSD-2-Clause` | https://github.com/kamadak/exif-rs |
| `lru` | Decoded-frame cache (at most 8) | `navigate` | `MIT` | https://github.com/jeromefroe/lru-rs |
| `rayon` | Background decoding and prefetch | `navigate` | `MIT OR Apache-2.0` | https://github.com/rayon-rs/rayon |
| `zip` | ZIP and CBZ reading (stored, deflate, zstd) | `archive-zip` | `MIT` | https://github.com/zip-rs/zip2 |
| `notify`, `notify-debouncer-mini` | Folder change watching | `navigate` | `CC0-1.0; MIT OR Apache-2.0` | https://github.com/notify-rs/notify |

## Libraries Not in the Shipped Build

These sit behind default-off features, so they are not shipped, but CI keeps building and testing them in the all-features build.

| Feature | Libraries |
|---|---|
| `iv-platform` `updater` | `reqwest`, `rustls`, `sha2` |
| `iv-platform` `os-codecs` | `windows` (WIC), `objc2-image-io`, `objc2-core-graphics`, `objc2-core-foundation` |
| `iv-platform` `shell-registration` | `winreg` |
| `iv-media` `extended-formats` | `jxl-oxide`, `image_dds`, `rawpsd`, `image-extras`, `exr` |

The full dependency tree is shown by `cargo tree -p iv-app -e normal`.
