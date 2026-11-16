# Copyright & Modification Notice (NOTICE)

**Language / 언어**: [한국어](NOTICE) | **[English](NOTICE.en.md)**

```text
ImageViewer (Cross-Platform Rust Edition)
Copyright (C) edith2580
Copyright (C) 2026 Ezenitac-AiService
```

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3 of the License (`GPL-3.0-only`).

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License in [LICENSE](LICENSE) for more details.

---

## Modification & Origin Notice (GNU GPLv3 Section 5(a))

- **Original Upstream Project**: https://github.com/edith2580/ImageViewer (Windows C#/.NET `v1.0.6` baseline)
- **Independent Fork & Modification Date**: `2026-11-16` — `Ezenitac-AiService` (https://github.com/Ezenitac-AiService/ImageViewer)
- **Summary of Prominent Modifications**:
  - Re-architected from the Windows-only C#/.NET `v1.0.6` codebase into a cross-platform Rust workspace (`iv-core`, `iv-media`, `iv-platform`, `iv-app`) supporting Windows, macOS, and Linux.
  - Replaced Windows-only GDI+/DirectComposition/WPF rendering with `winit`, `wgpu`, `egui`, `zune-jpeg`, `image`, `fast_image_resize`, `moxcms`, and `jxl-oxide`.
  - Isolated auto-update endpoints, bundle identifiers (`com.ezenitac.imageviewer`), and issue/release workflows to `Ezenitac-AiService/ImageViewer`.

For third-party Rust framework and library attributions, see [THIRD_PARTY_NOTICES.en.md](THIRD_PARTY_NOTICES.en.md) ([한국어](THIRD_PARTY_NOTICES.md)).
