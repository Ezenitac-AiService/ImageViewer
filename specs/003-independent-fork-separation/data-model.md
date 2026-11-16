# Data Model & Artifact Inventory: Independent Project Separation

**Feature**: `003-independent-fork-separation`
**Date**: 2026-11-16
**Spec**: [spec.md](spec.md)

---

## 1. `LicenseAndAttributionRecord`

- **Governing License**: `GPL-3.0-only` (GNU General Public License v3.0, root `LICENSE`).
- **Copyright Lines**:
  - Upstream preserved: `Copyright (C) edith2580` (original `ImageViewer` `v1.0.6` author).
  - Fork appended: `Copyright (C) 2026 Ezenitac-AiService`.
- **GPLv3 §5(a) Modification Notice**: Explicit notice in `NOTICE`, `CHANGELOG.md`, and `README.md` stating that `Ezenitac-AiService` forked and substantially rewrote the viewer into a cross-platform Rust implementation on `2026-11-16`.
- **Third-Party Notices**: `THIRD_PARTY_NOTICES.md` + `deny.toml` (`cargo-deny`) license policy (`MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `MPL-2.0`, `Unicode-3.0`, `GPL-3.0-only`).

---

## 2. `ProjectIdentityProfile` (`Q2=B`)

| Field | Old / Placeholder Value | Canonical Independent Value |
| :--- | :--- | :--- |
| Application Display Name | `ImageViewer` | `ImageViewer` (preserved for `settings.json` & user continuity) |
| Config Directory Name | `ImageViewer` (`%LOCALAPPDATA%\ImageViewer`, etc.) | `ImageViewer` (preserved) |
| Organization / Publisher | `ImageViewer` / `edith2580` | `Ezenitac-AiService` |
| Reverse-DNS Bundle ID | `com.imageviewer.app` | `com.ezenitac.imageviewer` |
| GitHub Repository Slug | `ImageViewer/ImageViewer` / `edith2580/ImageViewer` | `Ezenitac-AiService/ImageViewer` |
| Releases Page URL | `https://github.com/ImageViewer/ImageViewer/releases` | `https://github.com/Ezenitac-AiService/ImageViewer/releases` |
| Update Feed URL | `https://api.github.com/repos/ImageViewer/ImageViewer/releases/latest` | `https://api.github.com/repos/Ezenitac-AiService/ImageViewer/releases/latest` |

---

## 3. `RepositorySeparationState` (`Q1=B`)

- **Archive Tag**: `legacy-v1.0.6` created on the final pre-cutover C# commit (`960b627` / `d35e62f`) and pushed to `origin`.
- **Default Branch Cutover**: `002-rust-crossplatform-viewer` + `003-independent-fork-separation` merged to `main`; superseded legacy paths (`src/`, `tests/`, `ImageViewer.sln`, `build/publish.ps1`, legacy `.github/workflows/release.yml`) removed from active `main` after migrating `ImageViewer.nsi` into `packaging/windows/ImageViewer.nsi`.
- **Remote Policy**: `upstream` remote removed locally (`git remote remove upstream`); no CI workflows reference `upstream`.
- **GitHub Fork Detach**: GitHub Support request submitted for `Ezenitac-AiService/ImageViewer` ("leave fork network") after `main` CI passes (`cargo check`, `cargo clippy`, `cargo test`, `cargo deny`).

---

## 4. `DocumentationUpdateInventory` (`FR-008`)

| File Path | Action | Required Content |
| :--- | :--- | :--- |
| `LICENSE` | Verify | Unmodified GNU GPLv3 text (`FR-001`) |
| `NOTICE` | Create | Upstream `edith2580` credit, `Ezenitac-AiService` copyright, GPLv3 §5(a) modification + date `2026-11-16` |
| `CHANGELOG.md` | Create | `v1.0.6` legacy baseline -> `2026-11-16` cross-platform Rust rewrite & fork separation |
| `THIRD_PARTY_NOTICES.md` | Create | Attributions & licenses for core Rust frameworks/libraries (`winit`, `wgpu`, `egui`, `zune-jpeg`, `image`, `fast_image_resize`, `moxcms`, `jxl-oxide`, `reqwest`, `rustls`, etc.) |
| `deny.toml` | Create | `cargo-deny` GPLv3 license compatibility allow-list & policy |
| `README.md` | Rewrite | Cross-platform Windows/macOS/Linux overview, build/run/package guide, courteous top banner linking `edith2580/ImageViewer` & issue non-confusion notice |
| `CONTRIBUTING.md` | Create | Detached-fork contribution guide (no `upstream` merge policy, Spec-Kit workflow, `cargo test`/`clippy`/`deny` gates) |
| `SECURITY.md` | Create | Vulnerability reporting instructions for `Ezenitac-AiService/ImageViewer` |
| `.github/ISSUE_TEMPLATE/bug_report.yml` & `feature_request.yml` | Create | Issue templates with explicit upstream vs cross-platform fork routing notice |
| `.specify/memory/constitution.md` | Update | Note independent project ownership (`Ezenitac-AiService/ImageViewer`) & `GPL-3.0-only` license alignment |
| `specs/001-architecture-security-refactoring/spec.md` | Update header | Add `Status: Archived (Legacy v1.0.6 C# Baseline)` banner |
| `specs/002-rust-crossplatform-viewer/` | Audit/Update | Replace any placeholder `ImageViewer/ImageViewer` URLs with `Ezenitac-AiService/ImageViewer` |
