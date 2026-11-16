# Phase 0 Research: Independent Project Separation, Licensing & Release Cutover

**Feature**: `003-independent-fork-separation`
**Date**: 2026-11-16
**Spec**: [spec.md](spec.md)
**Constitution**: [constitution.md](../../.specify/memory/constitution.md) (`v2.0.0`)

---

## R1. GNU GPLv3 Section 5(a) & 5(d) Compliance and Manifest Harmonization

- **Decision**: Harmonize `Cargo.toml` (`workspace.package.license`) from `"MIT"` to `"GPL-3.0-only"` to match the repository root `LICENSE` (`GNU GENERAL PUBLIC LICENSE Version 3`), preserve all original `edith2580` copyright notices intact, add prominent GPLv3 §5(a) modification and date notices (`2026-11-16`) in `NOTICE`, `CHANGELOG.md`, and `README.md`, and display GPLv3 §5(d) origin/license/modification notices in `crates/iv-app/src/views/about_view.rs`.
- **Rationale**:
  - Root `LICENSE` is `GPLv3`. Under GPLv3 §5(a), a modified work MUST carry *"prominent notices stating that you modified it, and giving a relevant date."* Under §5(d), interactive UIs (`About` dialog) MUST display appropriate legal and origin notices.
  - Leaving `Cargo.toml` at `license = "MIT"` creates a direct legal contradiction with root `LICENSE` (`GPL-3.0-only`).
- **Alternatives Considered**:
  - *Dual-licensing or relicensing Rust rewrite to MIT*: Rejected because the project shares design specs, installer assets, and repository lineage under `GPLv3` without explicit written relicensing consent from all upstream copyright holders.
- **Sources**: GNU General Public License v3.0 (`gnu.org/licenses/gpl-3.0.html`, Sections 5(a), 5(b), 5(d)); repository root `LICENSE`.

---

## R2. Third-Party Framework/Library Notices & GPLv3 Compatibility Gate

- **Decision**: Publish `THIRD_PARTY_NOTICES.md` at repository root covering all bundled/transitive Rust crates (`winit`, `wgpu`, `egui`, `epaint_default_fonts`, `zune-jpeg`, `image`, `fast_image_resize`, `moxcms`, `jxl-oxide`, `reqwest`, `rustls`, etc.) and add a `deny.toml` (`cargo-deny`) license check via `EmbarkStudios/cargo-deny-action@v2` (`command: 'check'`, `arguments: ''`, `command-arguments: 'licenses'`) in `.github/workflows/ci.yml` allowing GPLv3-compatible permissive/font licenses (`MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `MPL-2.0`, `Unicode-3.0`, `CC0-1.0`, `BSL-1.0`, `CDLA-Permissive-2.0`, `GPL-3.0-only`, `Unlicense`, `OFL-1.1`, `Ubuntu-font-1.0`) while denying unlicensed or GPLv3-incompatible dependencies.
- **Rationale**: Automated license auditing in CI ensures no incompatible transitive crate enters `Cargo.lock` unnoticed (`FR-009`), while `THIRD_PARTY_NOTICES.md` + `About` view satisfy redistribution notice requirements.
- **Alternatives Considered**:
  - *Manual ad-hoc dependency tracking*: Rejected due to 500+ transitive crates in `Cargo.lock`.
- **Sources**: Embark Studios `cargo-deny` (`embarkstudios.github.io/cargo-deny`) & `cargo-about` documentation; GNU GPLv3 license compatibility matrix.

---

## R3. GitHub Fork Detachment (`Q1=B`) & Legacy `v1.0.6` Archival

- **Decision**: Retain the existing repository `Ezenitac-AiService/ImageViewer` (`Q1=B`), create and push an immutable `legacy-v1.0.6` archive tag on the pre-cutover C# commit, merge `002-rust-crossplatform-viewer` into `main`, migrate `src/ImageViewer.Installer/ImageViewer.nsi` to `packaging/windows/ImageViewer.nsi`, remove superseded C# source/installer paths (`src/`, `tests/`, `ImageViewer.sln`, `build/publish.ps1`, legacy `.github/workflows/release.yml`), remove local `upstream` remotes, and submit a GitHub Support fork-detachment request ("leave fork network") after CI passes on `main`.
- **Rationale**: Preserves existing URL, stars, and issue history while enabling independent repository code search and self-targeted PRs once detached by GitHub Support.
- **Alternatives Considered**:
  - *Creating a brand-new repository and mirror-pushing (`Q1=A`)*: Rejected by user (`Q1=B`) to preserve existing repository identity (`Ezenitac-AiService/ImageViewer`).
- **Sources**: GitHub Docs ("About forks", "Detaching a fork via GitHub Support"); local git remote/branch state inspection.

---

## R4. Courteous Upstream Attribution & Issue Routing Etiquette (`Q2=B`)

- **Decision**: Retain the `ImageViewer` application name and user `settings.json` directory continuity (`Q2=B`), while updating all repository URLs, HTTPS update endpoints, and OS reverse-DNS identifiers (`com.ezenitac.imageviewer`) to `Ezenitac-AiService/ImageViewer`. Place a prominent, polite top-level banner in `README.md` and `.github/ISSUE_TEMPLATE/` forms (`bug_report.yml`, `feature_request.yml`) crediting `edith2580/ImageViewer (v1.0.6)` and instructing users not to open cross-platform fork issues on the upstream repository.
- **Rationale**: Prevents accidental cross-traffic or bug-report burden on the upstream maintainer while preserving seamless config upgrades for existing Windows `v1.0.6` users.
- **Alternatives Considered**:
  - *Renaming app to `EasyViewer` (`Q2=A`)*: Rejected by user (`Q2=B`) to preserve `ImageViewer` continuity.
- **Sources**: Open-source fork etiquette standards; repository `crates/iv-platform/src/paths_dialogs.rs` & `crates/iv-app/src/main.rs` audit.
