# Third-Party Software Notices

**Language / 언어**: [한국어](THIRD_PARTY_NOTICES.md) | **[English](THIRD_PARTY_NOTICES.en.md)**

`ImageViewer` (`Ezenitac-AiService/ImageViewer`, licensed under `GPL-3.0-only`) is built with the following open-source Rust frameworks and libraries. All third-party dependencies are verified for GNU GPLv3 compatibility via `deny.toml` (`cargo deny check licenses`).

| Crate | Purpose | License | Upstream Project |
|---|---|---|---|
| `winit` | Cross-platform windowing and event loop | `Apache-2.0` | https://github.com/rust-windowing/winit |
| `wgpu` | Cross-platform GPU rendering (Metal / DX12 / Vulkan) | `MIT OR Apache-2.0` | https://github.com/gfx-rs/wgpu |
| `egui` | Immediate-mode UI overlay & About/Settings dialogs | `MIT OR Apache-2.0` | https://github.com/emilk/egui |
| `epaint_default_fonts` | Embedded default UI fonts (*Ubuntu Light*, *Hack*, *Noto Emoji*) | `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0` | https://github.com/emilk/egui |
| `zune-jpeg` | High-speed JPEG decoding | `MIT OR Apache-2.0 OR Zlib` | https://github.com/etemesi254/zune-image |
| `image` | Multi-format image decoding & encoding | `MIT OR Apache-2.0` | https://github.com/image-rs/image |
| `fast_image_resize` | SIMD-accelerated image resampling | `MIT OR Apache-2.0` | https://github.com/Cykooz/fast_image_resize |
| `moxcms` | ICC color profile management & color space conversion | `BSD-3-Clause OR Apache-2.0` | https://github.com/awxkee/moxcms |
| `jxl-oxide` | Pure-Rust JPEG XL decoding | `MIT OR Apache-2.0` | https://github.com/tirr-c/jxl-oxide |
| `exr` | OpenEXR image decoding | `BSD-3-Clause` | https://github.com/johannesvollmer/exr |
| `image-webp` | WebP image decoding & encoding | `MIT OR Apache-2.0` | https://github.com/image-rs/image-webp |
| `zip` | Comic/archive container reading (`.zip`, `.cbz`) | `MIT` | https://github.com/zip-rs/zip2 |
| `reqwest` | HTTPS client for isolated release update checks | `MIT OR Apache-2.0` | https://github.com/seanmonstar/reqwest |
| `rustls` | Pure-Rust TLS implementation | `Apache-2.0 OR ISC OR MIT` | https://github.com/rustls/rustls |
| `sha2` | SHA-256 checksum verification | `MIT OR Apache-2.0` | https://github.com/RustCrypto/hashes |
| `rfd` | Native OS file/folder open & save dialogs | `MIT` | https://github.com/PolyMeilex/rfd |

For the complete dependency tree and exact license texts, run `cargo metadata` or `cargo deny list` in the workspace root.
