---

description: "Actionable, dependency-ordered task list for Independent Project Separation, Licensing & Release Cutover (003)"
---

# Tasks: Independent Project Separation, Licensing & Release Cutover

**Input**: Design documents from `/specs/003-independent-fork-separation/`
**Prerequisites**: [plan.md](plan.md) (required), [spec.md](spec.md) (required), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts), [quickstart.md](quickstart.md)

**Tests**: Required for runtime code/URL/About changes per [constitution.md](../../.specify/memory/constitution.md) Principle III (`cargo test --workspace`) alongside automated repository grep & `cargo deny` license audits.

**Organization**: Tasks are grouped by user story (`US1 P1` -> `US2 P2` -> `US3 P3`) so license compliance, repository/legacy archival cutover, and identity/docs/release isolation can be executed and verified cleanly.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: User story label (`[US1]`, `[US2]`, `[US3]`) required for user story phases only

---

## Phase 1: Setup (Baseline Audit & Archive Safety)

**Purpose**: Verify baseline repository state, license inventory, and `legacy-v1.0.6` archive readiness before modifying manifests or cutting over `main`.

- [X] T001 Audit all upstream (`edith2580/ImageViewer`), placeholder (`ImageViewer/ImageViewer`), and `license = "MIT"` references across `Cargo.toml`, `crates/`, `packaging/`, `.github/`, and `specs/`
- [X] T002 [P] Create immutable archive tag `legacy-v1.0.6` on the pre-cutover C# commit (`960b627`) and push tag `legacy-v1.0.6` to `origin` (`FR-004`, `FR-010`)

---

## Phase 2: Foundational (License Policy & Identity Constants)

**Purpose**: Establish the `GPL-3.0-only` license policy and `cargo-deny` allow-list blocking incompatible dependencies before US1–US3 updates.

- [X] T003 Harmonize `[workspace.package]` in `Cargo.toml` from `license = "MIT"` to `license = "GPL-3.0-only"` matching root `LICENSE` (`FR-001`)
- [X] T004 [P] Create `deny.toml` configuring `cargo-deny` GPLv3-compatible license allow-list (`MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `MPL-2.0`, `Unicode-3.0`, `Unicode-DFS-2016`, `CC0-1.0`, `BSL-1.0`, `OpenSSL`, `CDLA-Permissive-2.0`, `GPL-3.0-only`) and denying unlicensed crates (`FR-009`)

**Checkpoint**: `Cargo.toml` and `deny.toml` establish `GPL-3.0-only` compliance baseline.

---

## Phase 3: User Story 1 - License Compliance, Attribution, and Legal Continuity (Priority: P1) 🎯 MVP

**Goal**: Satisfy GNU GPLv3 §5(a)/(d) prominent modification/date notices, preserve `edith2580` copyright, publish third-party framework/library attributions, and expose accurate legal notices in `About`.

**Independent Test**: Run `cargo test -p iv-app` / `cargo deny check licenses` and inspect `LICENSE`, `NOTICE`, `CHANGELOG.md`, `THIRD_PARTY_NOTICES.md`, and `crates/iv-app/src/views/about_view.rs`.

### Tests for User Story 1 (Red First)

- [X] T005 [P] [US1] Add unit test verifying `about_info_text()` outputs `GPL-3.0-only (GNU GPLv3)`, `https://github.com/Ezenitac-AiService/ImageViewer`, upstream origin `edith2580/ImageViewer`, modification date `2026-11-16`, and core framework credits in `crates/iv-app/src/views/about_view.rs`

### Implementation for User Story 1

- [X] T006 [P] [US1] Create `NOTICE` preserving `Copyright (C) edith2580`, appending `Copyright (C) 2026 Ezenitac-AiService`, and stating GPLv3 §5(a) cross-platform Rust modification on `2026-11-16` (`FR-001`, `FR-002`)
- [X] T007 [P] [US1] Create `CHANGELOG.md` recording `v1.0.6` legacy baseline and `2026-11-16` independent cross-platform Rust fork separation (`FR-001`, `FR-008`)
- [X] T008 [P] [US1] Create `THIRD_PARTY_NOTICES.md` listing bundled/core Rust frameworks & libraries (`winit`, `wgpu`, `egui`, `zune-jpeg`, `image`, `fast_image_resize`, `moxcms`, `jxl-oxide`, `exr`, `image-webp`, `zip`, `reqwest`, `rustls`, `sha2`, `rfd`) with licenses and upstream credits (`FR-008`, `FR-009`)
- [X] T009 [US1] Update `crates/iv-app/src/views/about_view.rs` with `about_info_text()` displaying `GPL-3.0-only (GNU GPLv3)`, `https://github.com/Ezenitac-AiService/ImageViewer`, `edith2580/ImageViewer (v1.0.6)` origin attribution, GPLv3 §5(d) notice, and framework/library credits (`FR-001`, `FR-002`, `FR-009`)

**Checkpoint**: US1 delivers full GPLv3 §5(a)/(d) and third-party attribution compliance.

---

## Phase 4: User Story 2 - Repository Detachment, Git Remote Separation, and Legacy Archival (Priority: P2)

**Goal**: Migrate needed Windows installer script (`ImageViewer.nsi`) out of `src/`, remove superseded legacy C# paths from active `main`, sever local `upstream` remote, and prepare GitHub Support fork detachment (`Q1=B`).

**Independent Test**: Verify `packaging/windows/ImageViewer.nsi` exists, `src/`/`tests/`/`ImageViewer.sln`/`build/` are removed from active branch, `git remote` has no `upstream`, and `legacy-v1.0.6` tag resolves cleanly.

- [X] T010 [US2] Copy and adapt `src/ImageViewer.Installer/ImageViewer.nsi` to `packaging/windows/ImageViewer.nsi` (updating publisher to `Ezenitac-AiService`, website to `https://github.com/Ezenitac-AiService/ImageViewer`, and harmonizing `/DVERSION` + `/DPUBLISH_DIR` + `OutFile` paths) and align `packaging/windows/package-windows.ps1` (`makensis /DVERSION=$Version /DPUBLISH_DIR=dist`) (`FR-004`, `FR-005`)
- [X] T011 [US2] Remove superseded legacy C# artifacts (`src/`, `tests/`, `ImageViewer.sln`, `build/publish.ps1`) after confirming `legacy-v1.0.6` tag is preserved (`FR-004`)
- [X] T012 [US2] Remove local `upstream` git remote (`git remote remove upstream`) so no automated or accidental `upstream` merges occur (`FR-003`)

**Checkpoint**: Active branch is 100% pure Rust cross-platform with `legacy-v1.0.6` preserved and `upstream` remote detached.

---

## Phase 5: User Story 3 - Independent Project Identity, Update Feed Isolation, Documentation & Release Cutover (Priority: P3)

**Goal**: Isolate all runtime URLs, OS bundle/app IDs (`com.ezenitac.imageviewer`, `Ezenitac-AiService/ImageViewer`), major documentation (`README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `.github/ISSUE_TEMPLATE/`, `specs/`), and CI/release workflows (`SHA256SUMS.txt`).

**Independent Test**: Run `grep` audit for `ImageViewer/ImageViewer` and `com.imageviewer.app` (expect 0 matches), run `cargo test --workspace`, and inspect updated docs & workflows.

### Tests for User Story 3 (Red First)

- [X] T013 [P] [US3] Update `crates/iv-platform/tests/update_tests.rs` to assert `Ezenitac-AiService/ImageViewer` release feed URLs and OS-matching asset selection (`windows` `.exe`/`.zip`, `macos` `.dmg`, `linux` `.AppImage`/`.tar.gz`) (`FR-006`)

### Implementation for User Story 3

- [X] T014 [P] [US3] Update auto-update feed URL and releases page URL in `crates/iv-app/src/main.rs` (`"https://api.github.com/repos/Ezenitac-AiService/ImageViewer/releases/latest"`, `"https://github.com/Ezenitac-AiService/ImageViewer/releases"`) and add OS-matching release asset selection in `crates/iv-platform/src/update.rs` (`FR-005`, `FR-006`)
- [X] T015 [P] [US3] Update `ProjectDirs::from("com", "Ezenitac-AiService", "ImageViewer")` in `crates/iv-platform/src/paths_dialogs.rs` (preserving `"ImageViewer"` config dir continuity) and `CFBundleIdentifier` (`com.ezenitac.imageviewer`) in `packaging/macos/Info.plist.in` (`FR-005`)
- [X] T016 [P] [US3] Rewrite `README.md` for cross-platform Rust `ImageViewer` (Windows/macOS/Linux features, build/install, shortcuts) with a courteous top-level banner crediting `edith2580/ImageViewer (v1.0.6)` and routing cross-platform issues strictly to `Ezenitac-AiService/ImageViewer` (`FR-002`, `FR-008`)
- [X] T017 [P] [US3] Create `CONTRIBUTING.md` (detached-fork workflow, no `upstream` merge policy, Spec-Kit & Cargo gates) and `SECURITY.md` (vulnerability reporting for `Ezenitac-AiService/ImageViewer`) (`FR-008`)
- [X] T018 [P] [US3] Create `.github/ISSUE_TEMPLATE/bug_report.yml` and `.github/ISSUE_TEMPLATE/feature_request.yml` with explicit upstream vs cross-platform fork routing confirmation (`FR-002`, `FR-008`)
- [X] T019 [P] [US3] Add `Status: Archived (Legacy v1.0.6 C# Baseline)` notice to `specs/001-architecture-security-refactoring/spec.md` and align any placeholder `ImageViewer/ImageViewer` URLs in `specs/002-rust-crossplatform-viewer/` and `.specify/memory/constitution.md` (`FR-008`)
- [X] T020 [US3] Update `.github/workflows/ci.yml` (add `EmbarkStudios/cargo-deny-action@v2` with `command: 'check'`, `arguments: ''`, `command-arguments: 'licenses'`) and replace legacy `.github/workflows/release.yml` with a 3-OS Rust tag-release workflow publishing OS-distinct binaries (`ImageViewer-windows-x64.exe`, `ImageViewer-macos.dmg`, `ImageViewer-linux-x64.tar.gz`) and `SHA256SUMS.txt` (`FR-007`, `FR-009`, `FR-010`)

**Checkpoint**: All runtime endpoints, OS identifiers, docs, issue templates, and CI/release workflows target `Ezenitac-AiService/ImageViewer` exclusively.

---

## Phase 6: Polish & Cross-Cutting Verification

**Purpose**: Final automated verification, branch cutover to `main`, and GitHub Support fork-detachment submission.

- [X] T021 Run `specs/003-independent-fork-separation/quickstart.md` audits (`grep` for `ImageViewer/ImageViewer` / `com.imageviewer.app`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) (`SC-001`–`SC-004`)
- [ ] T022 Merge `002-rust-crossplatform-viewer` into `main`, push `main` + `legacy-v1.0.6` to `origin`, verify CI green on `Ezenitac-AiService/ImageViewer`, and submit GitHub Support fork-detachment request ("leave fork network") (`FR-003`, `FR-010`, `SC-003`)

---

## Phase 7: Convergence

- [X] T023 Fix `.github/workflows/release.yml` and `README.md` to invoke the actual packaging scripts (`packaging/macos/package-dmg.sh` producing `target/release/ImageViewer-macos.dmg`, and `packaging/linux/package-appimage.sh` archiving `target/release/ImageViewer.AppDir` into `dist/ImageViewer-linux-x64.tar.gz`) instead of non-existent `package-macos.sh` / `package-linux.sh` per `FR-007` (`contradicts`)
- [X] T024 Add `"Unlicense"` to `allow` list in `deny.toml` alongside `"MIT"` for explicit `Unlicense/MIT` (`same-file`, `walkdir`) clarity per `FR-009` (`partial`)

---

## Dependencies & Execution Order

- **Phase 1 (Setup)** -> **Phase 2 (Foundational)** -> **US1 (`T005`–`T009`)**, **US2 (`T010`–`T012`)**, **US3 (`T013`–`T020`)** -> **Phase 6 (`T021`–`T022`)**.
- Note: `T011` (removing `src/`) MUST execute strictly AFTER `T002` (`legacy-v1.0.6` tag) and `T010` (migrating `ImageViewer.nsi` to `packaging/windows/`).
