# Contract: Independent Project Licensing, Identity, Cutover & Documentation

**Feature**: `003-independent-fork-separation`
**Covers**: `FR-001`–`FR-010`, `SC-001`–`SC-004`

---

## 1. License & Attribution Contract (`FR-001`, `FR-002`, `FR-009`)

- Root `LICENSE` MUST remain verbatim GNU GPLv3.
- `Cargo.toml` `[workspace.package]` MUST set `license = "GPL-3.0-only"`.
- `NOTICE` and `CHANGELOG.md` MUST preserve `edith2580` copyright attribution, append `Copyright (C) 2026 Ezenitac-AiService`, and state the `2026-11-16` cross-platform Rust modification per GPLv3 §5(a).
- `THIRD_PARTY_NOTICES.md` MUST document all major frameworks and libraries (`winit`, `wgpu`, `egui`, `zune-jpeg`, `image`, `fast_image_resize`, `moxcms`, `jxl-oxide`, `exr`, `image-webp`, `zip`, `reqwest`, `rustls`, `sha2`, `rfd`).
- `deny.toml` MUST pass `cargo deny check licenses` (or CI license audit) with zero GPLv3-incompatible or unlicensed dependencies.

---

## 2. Runtime Identity & Endpoint Isolation Contract (`FR-005`, `FR-006`)

- `crates/iv-app/src/main.rs` & `crates/iv-platform/src/update.rs`: Auto-update feed URL MUST be `"https://api.github.com/repos/Ezenitac-AiService/ImageViewer/releases/latest"`, release page MUST be `"https://github.com/Ezenitac-AiService/ImageViewer/releases"`, and `UpdateClient::check_latest_with_transport` MUST select the OS-matching release asset (`windows` `.exe`/`.zip`, `macos` `.dmg`, `linux` `.AppImage`/`.tar.gz`) rather than blindly picking the first asset.
- `crates/iv-app/src/views/about_view.rs`: MUST display `GPL-3.0-only (GNU GPLv3)`, `https://github.com/Ezenitac-AiService/ImageViewer`, upstream origin credit (`edith2580/ImageViewer v1.0.6`), `2026-11-16` fork modification notice (GPLv3 §5(d)), and bundled framework/library credits.
- `crates/iv-platform/src/paths_dialogs.rs`: `ProjectDirs::from("com", "Ezenitac-AiService", "ImageViewer")` MUST preserve the `"ImageViewer"` config folder name for `v1.0.6` `settings.json` continuity.
- `packaging/macos/Info.plist.in`: `CFBundleIdentifier` MUST be `com.ezenitac.imageviewer`.
- `packaging/windows/ImageViewer.nsi` & `packaging/windows/package-windows.ps1`: Must live cleanly under `packaging/windows/` (no dependency on removed `src/`) and reference `Ezenitac-AiService`.

---

## 3. Courteous Fork Etiquette & Repository Cutover Contract (`FR-003`, `FR-004`, `FR-007`, `FR-008`, `FR-010`)

- `README.md` MUST include a top-level banner crediting `edith2580/ImageViewer (v1.0.6)` and politely asking users not to report cross-platform fork issues upstream.
- `.github/ISSUE_TEMPLATE/` MUST include bug/feature templates confirming the issue pertains to `Ezenitac-AiService/ImageViewer` (not upstream).
- `.github/workflows/release.yml` MUST build cross-platform Rust release binaries (`windows`, `macos`, `linux`) on tag push and publish `SHA256SUMS.txt` alongside binaries.
- `legacy-v1.0.6` tag MUST preserve the pre-cutover C# commit before superseded `src/`, `tests/`, `ImageViewer.sln`, and `build/publish.ps1` are removed from `main`.
- Zero active runtime, packaging, or CI files may reference `edith2580/ImageViewer` (except attribution text in `NOTICE`/`README.md`/`About`/`ISSUE_TEMPLATE`) or placeholder `ImageViewer/ImageViewer`.
