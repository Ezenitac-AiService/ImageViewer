# Contract: Media Decoding, Caching, Scaling, Color & Host OS Codecs

**Feature**: `002-rust-crossplatform-viewer`
**Covers**: `FR-001`, `FR-003`, `FR-004`, `FR-008`, `SC-001`, `SC-002`, `SC-003`

---

## 1. Core vs Host OS Codec Dispatch (`iv-core` + `iv-media` + `iv-platform`)

| Format Class | Extensions | Handling Strategy | Pre-Check Requirement |
| :--- | :--- | :--- | :--- |
| **Core Standard & Modern Raster / HDR / Layered / Archive** | `.jpg`, `.jpeg`, `.jpe`, `.jfif`, `.png`, `.apng`, `.gif`, `.bmp`, `.dib`, `.tif`, `.tiff`, `.webp`, `.ico`, `.cur`, `.jxl`, `.tga`, `.psd`, `.dds`, `.pcx`, `.qoi`, `.ppm`, `.pgm`, `.pbm`, `.pnm`, `.hdr`, `.exr`, `.zip`, `.cbz` | Built-in memory-safe Rust core decoders (`zune-jpeg`, `image`, `image-extras` for `.pcx`, `image_dds` for `.dds`, `rawpsd` for RGB/CMYK/Gray PSD, `jxl-oxide`, `exr`, `zenwebp`/`image-webp`, pure-Rust `.cur` parser, EOCD pre-checked `zip`) across Windows, macOS, Linux | 8 KB binary signature guard + pre-flight EOCD zip caps + `MaxZipEntryMegabytes` + EXIF Orientation (`5–8` swap) + `checked` stride & `MaxImageMegapixels` pre-allocation check |
| **Host OS Platform Codecs** | Camera RAW (`.dng`, `.cr2`, `.cr3`, `.nef`, `.arw`, `.orf`, `.rw2`, `.raf`), `.heic`, `.heif`, `.avif`, `.jp2` | `OsCodecProvider` trait & `OsDecodedBuffer` DTO defined in `crates/iv-core` (`#![forbid(unsafe_code)]`) -> implemented in `crates/iv-platform` (Windows `WIC` / macOS `ImageIO` / Linux optional system probe) so Cargo remains strictly an acyclic DAG (`iv-media -> iv-core`, `iv-platform -> iv-core`) | Mandatory 8 KB signature guard + `512 MB` cap + pre-decode dimension query `<= MaxImageMegapixels` BEFORE pixel allocation; fail closed with localized missing/unsupported notice on missing codec or unverified bounds |

---

## 2. EXIF Orientation, Color Management & HDR Normalization

- **EXIF Orientation (`1–8`) & Double-Transform Prevention**: Automatically read and apply EXIF `Orientation` (`AutoOrient` parity) via `image::ImageDecoder::orientation()` + `apply_orientation()`, swapping width/height dimensions on tags `5–8` **before** pixel-cap verification and viewport sizing. For host OS decoders returning `OsDecodedBuffer { orientation_applied, srgb_converted, .. }`, `iv-media` skips EXIF `apply_orientation` when `orientation_applied == true` and skips `moxcms` conversion when `srgb_converted == true` (preventing macOS `ImageIO` / Windows `WIC` double-rotation or double-gamma).
- **Multi-Page TIFF Per-Page Parity**: Multi-page TIFF switching (`PgUp`/`PgDn`) evaluates IFD dimensions + pixel caps, per-page EXIF `Orientation (1–8)`, bit depth (`16-bit`), and color space (e.g. `ExtendedColorType::Cmyk16` -> `moxcms`) independently per page, while preserving active user screen transforms (`R`/`L`/`H`/`V`).
- **ICC Profiles (`moxcms`)**:
  - Untagged or standard sRGB images take zero-transform fast path (preserving `SC-001` speed).
  - Embedded non-sRGB (Display P3, Adobe RGB, etc.) and CMYK profiles convert to standard 8-bit/channel sRGB for display (when `!srgb_converted`); malformed ICC blocks fall back cleanly to raw RGB without crashing.
- **HDR & High-Bit-Depth**:
  - Floating-point HDR images (`.exr`, `.hdr`) pass through lightweight auto-exposure + soft Reinhard/ACES tone mapping + sRGB gamma encoding to preserve highlight detail.
  - `16-bit` integer channels normalize linearly/gamma-consistently into standard display pixels.

---

## 3. Performance, Concurrency & Caching Contract

- **Dual-Lane Thread Budgeting**:
  - Foreground decode/render workers run independently from bounded background prefetch & thumbnail worker pools (`rayon` thread cap `clamp(cpus / 2, 1, 4)`).
  - Preemption / Generation Cancellation: Every navigation increments monotonic `load_gen`/`render_gen` and cancels superseded async tokens so newly selected images start immediately without waiting behind obsolete heavy decodes.
- **Viewport Scaling, DPI & Filters**:
  - Viewport fitting, `>= 60%` display-stage threshold, and `SmoothSigma` "screen pixels" scale accurately using physical-surface DPI dimensions (`logical_size * dpi_scale_factor` updated cleanly on `ScaleFactorChanged`).
  - High-res images significantly larger than physical display viewport (`< 60%` target ratio vs source) are SIMD-downscaled (`fast_image_resize` AVX2/SSE4.1/ARM NEON) into `buffer_width × buffer_height` while preserving true post-orientation `source_width × source_height` (`DecodedFrame`); images modestly larger (`>= 60%`) upload cleanly for GPU bilinear/high-quality shader downscaling without extra CPU pass.
  - **`wgpu` Texture Dimension Cap & Large Webtoon/Panorama Tiling**: `wgpu::Device` explicitly requests `wgpu::Limits { max_texture_dimension_2d: adapter.limits().max_texture_dimension_2d.min(16384), .. }` instead of the default `8,192`. Images whose uploaded width or height exceeds `device.limits().max_texture_dimension_2d` (such as vertical webtoons or wide panoramas at `FullRes`) are split cleanly into GPU sub-texture tiles/strips (`<= tile_dim`) and drawn with split UV coordinates without panicking.
  - Zooming past a downscaled cached image or switching to `Original` (`0`) triggers asynchronous background reload of full-resolution pixels (`FullRes`) and swaps seamlessly when ready.
  - Filters (`Smooth`, `SmoothSharpen`) compute effective kernel radii (`sigma / display_scale`, skipping `< 0.05`) in pure-Rust `iv-core`/`iv-media` helpers and execute separable Gaussian blur + Unsharp Mask passes via **`wgpu` GPU shaders** on already-resident GPU textures (`crates/iv-app/src/renderer.rs`, avoiding CPU re-filtering + PCIe re-upload on zoom), while remaining bypassed on animated playback.
- **Prefetching & Placeholder Timers**:
  - Directional lookahead: `+1`, `+2`, `-1`; rapid burst (`<= 600 ms` key cadence) expands forward up to `+4` across 2 parallel background slots; active slideshow also prefetches next sequence/random entry.
  - Navigating into an in-flight prefetch item awaits/attaches to the existing task rather than restarting from scratch.
  - Placeholder timer (`250 ms`) reveals scaled cached thumbnail (skipped in `Original` mode or if missing thumbnail); spinner triggers at `500 ms`; render completion `>= 300 ms` writes a redacted timing trace.
- **Memory Budgets & Animation Rules**:
  - Decoded bitmap LRU cache: max `8` items; Thumbnail cache: max `300` items; Display texture LRU bounded.
  - Animation decoder decodes frame 1 immediately (`< 250 ms` target); pre-decodes all frames if total buffer `<= 512 MB` (downscaling to display viewport size if appropriate), or streams via `<= 128 MB` forward ring buffer dropping past frames.
  - Frame delays track wall-clock elapsed time (`GIF <= 10 ms -> 100 ms`; `WebP` preserves per-frame delays including `1 ms`, normalizing to `100 ms` only when *all* frames are `<= 10 ms`; `APNG` executes `DisposeOp` `None`/`Background`/`Previous` and `BlendOp` `Source`/`Over` compositing onto a persistent RGBA canvas with rational `delay_num / delay_den` duration where `delay_den == 0 -> 100` and `<= 10 ms -> 100 ms`).
