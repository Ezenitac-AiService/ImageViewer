---

description: "Actionable dependency-ordered task list for Constitution v3.0.0 platform tier realignment"
---

# Tasks: Platform Tier Realignment (Constitution v3.0.0)

**Input**: Design documents from `/specs/006-platform-tier-realignment/`  
**Prerequisites**: [plan.md](./plan.md) (required), [spec.md](./spec.md) (required), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/platform-realignment-contract.md](./contracts/platform-realignment-contract.md), [quickstart.md](./quickstart.md)

**Tests**: Test tasks (`T007`, `T012`, `T018`, `T029`, `T030`) enforce Constitution v3.0.0 Principles III, IV, V, and VII (`cargo fmt`, `cargo clippy`, `cargo test` default and `--all-features`, `cargo deny`, 3-target component budget, release-notes rendering, and OS-agnostic Zero-Trust preservation) without `#[ignore]` bypasses. All live remote governance mutations (`T015`, `T031`, `T032`, `T033`, `T034`) require explicit maintainer approval (`FR-012`).

**Organization**: Grouped into Setup (Phase 1), Foundational Windows-code/dependency cleanup & Constitution finalization (Phase 2), User Stories `US1`–`US4` in priority order (Phases 3–6), and Polish/Release Regeneration (Phase 7).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: User story mapping (`[US1]`, `[US2]`, `[US3]`, `[US4]`) — omitted on Setup, Foundational, and Polish phases
- Every task states concrete target file paths or remote endpoints

## Path Conventions

- Constitution & templates: `.specify/memory/constitution.md`
- CI, release workflows, & issue forms: `.github/workflows/`, `.github/ISSUE_TEMPLATE/`
- Rust workspace manifests, crates, & tests: `Cargo.toml`, `Cargo.lock`, `crates/iv-core/`, `crates/iv-platform/`, `crates/iv-app/`
- Packaging & release helpers: `packaging/`
- Bilingual root & guide documentation: `README*`, `CONTRIBUTING*`, `SECURITY*`, `CHANGELOG*`, `NOTICE*`, `THIRD_PARTY_NOTICES*`, `docs/`

---

## Phase 1: Setup (Branch Isolation & Baseline)

**Purpose**: Establish a non-`NNN-*` working branch (preventing duplicate `push` + `pull_request` CI runs under `Protect main`) and record pre-change live/local baselines before editing repository files.

- [X] T001 Create and switch to non-`NNN-*` working branch `chore/006-platform-realignment` from `main` carrying `.specify/memory/constitution.md` (`v3.0.0`) and `specs/006-platform-tier-realignment/` (`FR-007`, `research.md` R7)
- [X] T002 [P] Capture read-only live remote pre-change baseline via `gh api repos/Ezenitac-AiService/ImageViewer/rulesets/27903505`, `gh release view v1.1.0-alpha.2`, and `gh repo view` (`specs/006-platform-tier-realignment/quickstart.md` §3)
- [X] T003 [P] Verify pre-change red baseline confirming `packaging/windows/`, `crates/iv-app/build.rs`, `winresource`/`windows`/`winreg` dependencies, Windows/Linux-runtime workflow jobs, and `bug_report.yml` `Windows` dropdown exist (`specs/006-platform-tier-realignment/quickstart.md` §1 #3)

---

## Phase 2: Foundational (Constitution Finalization & Windows-Target Code/Dependency Removal)

**Purpose**: Finalize Constitution `v3.0.0` (`Sync Impact Report` cleanup), remove all Windows-only packaging scripts, Cargo dependencies, and `#[cfg(target_os = "windows")]` adapters/branches while strictly preserving OS-agnostic Zero-Trust security guards (`Q4` + `Q5`), and verify clean compilation and tests across default and `--all-features` before configuring CI/release workflows and documentation stories.

**⚠️ CRITICAL**: No user story phase starts until `T012` passes all host Rust, `cargo deny`, and 3-target component-budget checks.

- [X] T004 [P] Remove the temporary HTML `Sync Impact Report` review comment (`L1`–`L18`) from `.specify/memory/constitution.md` so `# ImageViewer Constitution` begins cleanly at line 1 with `**Version**: 3.0.0 | **Ratified**: 2026-11-15 | **Last Amended**: 2026-11-24` (`FR-012`, `research.md` R6)
- [X] T005 [P] Delete Windows-only packaging and verification artifacts (`packaging/windows/ImageViewer.ico`, `packaging/windows/ImageViewer.manifest`, `packaging/windows/ImageViewer.nsi`, `packaging/windows/package-windows.ps1`, `packaging/verify/assert-windows-exe.ps1`, `packaging/verify/launch-check.ps1`) (`FR-013`, `SC-001`, `research.md` R3)
- [X] T006 [P] Remove Windows-only Cargo dependencies and build scripts (`winresource = "0.1"` in `Cargo.toml`, `[target.'cfg(windows)'.build-dependencies]` in `crates/iv-app/Cargo.toml`, delete `crates/iv-app/build.rs`, remove `[target.'cfg(target_os = "windows")'.dependencies]` (`windows`, `winreg`), `"dep:windows"` in `os-codecs`, and change `shell-registration = ["dep:winreg"]` to `shell-registration = []` in `crates/iv-platform/Cargo.toml` so `#[cfg(feature = "shell-registration")]` in `lib.rs` and `os_integration_tests.rs` compiles cleanly under `clippy -D warnings`) (`FR-013`, `Q4`)
- [X] T007 [P] Update workspace tests first (Red→Green TDD per Constitution III) in `crates/iv-core/tests/cli_tests.rs`, `crates/iv-platform/tests/os_integration_tests.rs`, and `crates/iv-platform/tests/update_tests.rs` removing Windows-only expectations (`repairs_explorer_trailing_quote_on_windows_drive`, `is_windows` loop, `format_windows_open_command`, `"windows"` log folder/asset checks) while keeping `--register-shell`/`--unregister-shell` negative tests and preserving `crates/iv-core/tests/diagnostics_tests.rs` + `crates/iv-core/tests/path_security_tests.rs` OS-agnostic guards (`FR-013`, `Q5`)
- [X] T008 [P] Remove Windows-target CLI and containment branches in `crates/iv-core/src/cli.rs` (drop `is_windows` param, `/` switches, and `repair_explorer_quoted_drive` while keeping `--register-shell`/`--unregister-shell` fail-closed `error.unsupported_switch`), `crates/iv-core/src/codecs.rs` (doc comment), and `crates/iv-core/src/path_security.rs` (drop `#[cfg(target_os = "windows")]` arm in `is_within_root` ONLY; strictly preserve OS-agnostic `normalize_archive_entry_key` `\` → `/`, `is_archive_symlink`, and `PathRedactor`) (`FR-013`, `Q4`, `Q5`; depends on `T007`)
- [X] T009 [P] Remove `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]` and `cfg!(target_os = "windows")` argument from `crates/iv-app/src/main.rs`, and remove `C:\Windows\Fonts\malgun.ttf` from `CJK_CANDIDATES` in `crates/iv-app/src/fonts.rs` (`FR-013`, `Q4`; depends on `T007`)
- [X] T010 [P] Remove Windows-target platform implementations (`WIC` decoder in `crates/iv-platform/src/codecs.rs`, `LOCALAPPDATA` branch in `crates/iv-platform/src/paths_dialogs.rs`, `"windows"` arm in `crates/iv-platform/src/fatal_dialog.rs`, `format_windows_open_command`/`register_windows_shell`/`unregister_windows_shell` in `crates/iv-platform/src/integration.rs`, and `cmd /C start`/`.exe` selection in `crates/iv-platform/src/update.rs`) (`FR-013`, `Q4`; depends on `T007`)
- [X] T011 [P] Update `packaging/verify/component-count.sh` line 14 to `TARGETS=(aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu)` preserving `FORBIDDEN=(reqwest hyper rustls tokio)` and `BUDGET="${BUDGET:-356}"` (`SC-001`, `SC-006`, `research.md` R4)
- [X] T012 Refresh `Cargo.lock` (`cargo check --workspace --all-features`) pruning direct workspace `windows`/`winreg`/`winresource` entries (upstream target-conditional metadata entries remain unused on `macOS`/`Linux` and excluded from 3-target `component-count.sh`), and verify green host checks (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --all-features`, `cargo test --workspace`, `cargo deny check advisories bans licenses sources`, `cargo deny --all-features check advisories bans licenses sources`, and `./packaging/verify/component-count.sh` confirming `<= 356` and recording the updated 3-target count) (`FR-005`, `SC-001`, `SC-006`; depends on `T006`–`T011`)

**Checkpoint**: Codebase, lockfile, and `constitution.md` `v3.0.0` compile and pass all tests cleanly on macOS with zero Windows dependencies/scripts and 100% preserved Zero-Trust archive/log sanitization.

---

## Phase 3: User Story 1 - Retire Windows & Align Governance Baseline (Priority: P1) 🎯 MVP

**Goal**: Remove Windows jobs from CI/release workflows and synchronize the live `Protect main` Repository Ruleset (`id: 27903505`) in deadlock-safe order so PRs merge cleanly with zero Windows gates.

**Independent Test**: Open a PR after `T015`, inspect `gh api .../rulesets/27903505` and `gh pr checks`, confirm mandatory checks require only the 4 realigned `@15368` contexts with zero orphaned Windows contexts, and verify CI/release workflows hold zero Windows jobs or deliverables.

- [X] T013 [US1] Remove `windows-latest` from `checks.strategy.matrix.os` (`[ubuntu-latest, macos-latest]`), remove the `launch-windows` job (`Launch check (Windows)`), and update header comments in `.github/workflows/ci.yml` while keeping `permissions: { contents: read }` and 40-char Action SHA pinning (`FR-001`, `FR-007`, `contracts/platform-realignment-contract.md` §2.1)
- [X] T014 [P] [US1] Remove `build-windows` and `verify-windows` jobs, remove `ImageViewer-windows-x64.exe` from `publish-draft` `sha256sum` and `softprops/action-gh-release` `files:`, and update comments in `.github/workflows/release.yml` while keeping `env.TAG` + SemVer regex guard (`FR-002`, `FR-012`, `contracts/platform-realignment-contract.md` §2.2)
- [X] T015 [US1] Request explicit maintainer approval (`FR-012`) and update live `Protect main` Ruleset (`PUT repos/Ezenitac-AiService/ImageViewer/rulesets/27903505`) BEFORE merging the `006` PR per `contracts/platform-realignment-contract.md` §1 (`enforcement: "active"`, `bypass_actors: []`, `pull_request` `required_approving_review_count: 0`, `strict_required_status_checks_policy: true`, exact 4 contexts each pinning `"integration_id": 15368`: `Checks (ubuntu-latest)`, `Checks (macos-latest)`, `Launch check (macOS)`, `Dependencies (cargo-deny, budget)`) (`FR-007`, `SC-004`, `research.md` R1)
- [X] T016 [US1] Audit live `Protect main` Ruleset (`gh api repos/Ezenitac-AiService/ImageViewer/rulesets/27903505` and `gh api repos/Ezenitac-AiService/ImageViewer/rules/branches/main`) confirming exact 4 required `@15368` contexts, zero administrator bypass, and zero retired Windows contexts (`SC-004`; depends on `T015`)

**Checkpoint**: Windows gates are eliminated across CI, release packaging, and branch protection without any merge deadlock window.

---

## Phase 4: User Story 2 - Exclusive Top-Priority Verification for macOS & Honest iOS Roadmap Boundary (Priority: P1)

**Goal**: Enforce automated build-result and launch verification exclusively on `macOS` (`aarch64` & `x86_64`) as the active top-priority target while truthfully documenting `iOS` (`aarch64`) as a planned top-priority target deferred to feature `007`.

**Independent Test**: Run mock `macOS`-only test on `packaging/verify/render-release-notes.py`, inspect `.github/workflows/ci.yml` (`launch-macos`) and `.github/workflows/release.yml` (`build-macos`, `verify-macos`, `publish-draft`), and verify release notes display verified `macOS` rows and `planned (feature 007)` `iOS` status with zero false shipped claims.

- [X] T017 [P] [US2] Update `packaging/verify/launch-check.sh` header/scope comments (`L1`–`L10`) to reflect exclusive top-priority `macOS` launch verification (`release_file == ImageViewer.app`, `window_shown`, `first_frame`, `clean_exit`, `result == pass`) (`FR-003`, `data-model.md` §2)
- [X] T018 [US2] Update `packaging/release-notes.md.in` and `packaging/verify/render-release-notes.py` per `contracts/platform-realignment-contract.md` §3 (remove Windows/Smart App Control rows, show `macOS` Apple Silicon & Intel `Top priority` verified rows, mark `Linux` `Lower priority (build-only)` with `n/a (no runtime verification)` / `build-only, unverified`, and include honest `iOS` & `Android` `planned — feature 007` rows), then verify output via mock `macOS`-only JSON fixture test in `specs/006-platform-tier-realignment/quickstart.md` §1 #5 (`FR-003`, `FR-006`, `SC-002`, `SC-003`)
- [X] T019 [US2] Audit `launch-macos` in `.github/workflows/ci.yml` and `build-macos`/`verify-macos`/`publish-draft` in `.github/workflows/release.yml` confirming `macOS` universal bundle signing + arm64/x64 launch verification strictly gate draft creation and zero unbuilt `iOS` release assertions appear (`FR-003`, `SC-002`; depends on `T013`, `T014`, `T018`)

**Checkpoint**: Top-priority verification and release-note generation cleanly enforce `macOS` launch verification and honestly scope `iOS` to `007`.

---

## Phase 5: User Story 3 - Lower-Priority Build-Only Delivery for Linux & Honest Android Roadmap Boundary (Priority: P2)

**Goal**: Retain `Linux` (`x86_64`) compile checks in PRs and build-only tarball packaging in releases while eliminating every `Linux` runtime/launch verification job and labeling `Linux` (and planned `Android`) honestly as build-only and unverified.

**Independent Test**: Inspect `.github/workflows/ci.yml`, `.github/workflows/release.yml`, and `packaging/linux/package-linux.sh` to confirm `Checks (ubuntu-latest)` and `build-linux` pass build-only packaging while zero `launch-linux` / `verify-linux` jobs or `launch-verification-linux-*.json` artifacts exist.

- [X] T020 [US3] Remove the `launch-linux` (`Launch check (Linux, experimental)`) job completely from `.github/workflows/ci.yml` while keeping `checks` (`ubuntu-latest`) required for PR compilation/lint/test verification per `Q3` (`FR-004`, `FR-007`, `SC-003`; depends on `T013`)
- [X] T021 [US3] Remove `verify-linux` completely from `.github/workflows/release.yml`, update `build-linux` step labels (`Package lower-priority build-only tarball`), update `publish-draft` `needs: [verify-macos, build-linux, deny]` and `if: always() && needs.verify-macos.result == 'success' && needs.build-linux.result == 'success' && needs.deny.result == 'success'`, and ensure zero `launch-verification-linux-*.json` records are generated or attached (`FR-004`, `SC-003`, `data-model.md` §3; depends on `T014`)
- [X] T022 [P] [US3] Update `packaging/linux/package-linux.sh` header/comments removing `experimental` wording and stating `Linux` (`x86_64`) output is a lower-priority build-only unverified tarball under Constitution `v3.0.0` (`FR-004`, `FR-006`, `SC-003`)

**Checkpoint**: `Linux` runs compile-only PR checks and produces a build-only release tarball with zero runtime verification jobs.

---

## Phase 6: User Story 4 - Bilingual Documentation & Issue Intake Alignment (Priority: P2)

**Goal**: Synchronize all bilingual Korean default and English counterpart documentation pairs (`README*`, `docs/CAPABILITIES*`, `docs/FIRST_LAUNCH*`, `CONTRIBUTING*`, `NOTICE*`, `THIRD_PARTY_NOTICES*`, `CHANGELOG*`) and `.github/ISSUE_TEMPLATE/bug_report.yml` with `v3.0.0` platform tiers while preserving `v1.0.6` GPLv3 origin attribution.

**Independent Test**: Run `specs/006-platform-tier-realignment/quickstart.md` §1 #3–#4 verifying bug-report OS selector omits `Windows`, all 8 bilingual pairs pass cross-link checks with matching language switchers, and zero active-Windows runtime promises remain.

- [X] T023 [P] [US4] Update `.github/ISSUE_TEMPLATE/bug_report.yml` OS dropdown (`id: os`) per `contracts/platform-realignment-contract.md` §4 removing `Windows` and offering `macOS`, `Linux (build-only / unverified)`, `iOS (planned)`, and `Android (planned build-only)` while preserving `fork-confirmation` (`required: true`) (`FR-009`, `SC-005`)
- [X] T024 [P] [US4] Update `README.md` and `README.en.md` in lockstep refreshing platform badge, overview, upstream comparison table, shortcut note (`Primary`), and release table (`macOS` active top priority, `iOS` planned `007`, `Linux` lower-priority build-only/unverified, `Android` planned build-only `007`, `Windows` excluded) while preserving `edith2580/ImageViewer (v1.0.6)` origin and `legacy-v1.0.6` archive links (`FR-008`, `SC-001`, `SC-005`)
- [X] T025 [P] [US4] Update `docs/CAPABILITIES.md` and `docs/CAPABILITIES.en.md` in lockstep aligning Core/Extended verification tiers to `macOS` (`iOS` planned `007`), documenting `Linux` as build-only/unverified (`Android` planned `007`), and removing active `Windows` claims (`Windows Server 2025`, `Windows "Open with"`, `EXTENDED` Windows criteria) (`FR-008`, `SC-005`)
- [X] T026 [P] [US4] Update `docs/FIRST_LAUNCH.md` and `docs/FIRST_LAUNCH.en.md` in lockstep removing the active `Windows 10 / 11` and `%LOCALAPPDATA%` sections, retaining `macOS` Gatekeeper guidance, reclassifying `Linux` as `하위 우선순위 빌드 전용 (미검증)` / `Lower-Priority Build-Only (Unverified)`, and noting mobile guides await `007` (`FR-008`, `FR-011`, `SC-005`)
- [X] T027 [P] [US4] Update `CONTRIBUTING.md` and `CONTRIBUTING.en.md` in lockstep updating Definition of Done (`macOS` observed launch; `Linux` build-only without runtime verification; `Windows` excluded) and manual release checklist (`macOS` Apple Silicon & Intel clean-device verification replacing clean Windows VMs) (`FR-008`, `FR-011`, `SC-005`)
- [X] T028 [P] [US4] Update `NOTICE`/`NOTICE.en.md`, `THIRD_PARTY_NOTICES.md`/`THIRD_PARTY_NOTICES.en.md`, and `CHANGELOG.md`/`CHANGELOG.en.md` in lockstep adding `2026-11-24` platform-tier realignment modification notes while preserving `2026-11-16`/`17` and `edith2580` GPLv3 §5(a) attribution, removing `windows (WIC)`/`winreg` rows and updating the exact 3-target `component-count.sh` count in `THIRD_PARTY_NOTICES*`, and verifying `SECURITY.md`/`SECURITY.en.md` consistency (`FR-008`, `FR-013`, `SC-001`, `SC-005`; depends on `T012`)
- [X] T029 [US4] Audit `.github/ISSUE_TEMPLATE/` and all 8 Korean/English document pairs via `specs/006-platform-tier-realignment/quickstart.md` §1 #3–#4 confirming working language switchers, 0 broken relative links, 0 stale active-Windows instructions, and preserved historical `legacy-v1.0.6` attribution (`FR-008`, `FR-009`, `SC-001`, `SC-005`; depends on `T023`–`T028`)

**Checkpoint**: Public repository pages, guides, legal notices, and issue forms consistently enforce Constitution `v3.0.0` across Korean and English.

---

## Phase 7: Polish, Merge, Metadata Audit & Clean Draft Regeneration

**Purpose**: Verify the complete local suite, land `006` onto protected `main`, audit repository metadata, regenerate the `v1.1.0-alpha.2` draft cleanly under `v3.0.0` (`Q2`), verify Apple Silicon DMG locally, and run final `/speckit-analyze` consistency sign-off.

- [X] T030 Run full `specs/006-platform-tier-realignment/quickstart.md` §1 validation suite (`cargo fmt`, `cargo clippy` default & `--all-features`, `cargo test` default & `--all-features`, `cargo deny` default & `--all-features`, `./packaging/verify/component-count.sh`, YAML/SHA/shell-injection scan, zero Windows dependency/script/dropdown audit, preserved Zero-Trust guard check, `render-release-notes.py` test, and 8-pair bilingual link audit) on macOS host after final file edits (`FR-005`, `SC-001`–`SC-006`; depends on `T004`–`T029`)
- [X] T031 Commit all changes on `chore/006-platform-realignment` with Korean commit message (Constitution Principle I), push, open PR targeting `main`, confirm all 4 required checks (`Checks (ubuntu-latest)`, `Checks (macos-latest)`, `Launch check (macOS)`, `Dependencies (cargo-deny, budget)`) pass with zero Windows or Linux-runtime checks, merge PR under maintainer approval (`FR-012`), pull `main`, and prune `chore/006-platform-realignment` (`FR-007`, `SC-004`; depends on `T015`, `T016`, `T019`–`T021`, `T030`)
- [X] T032 Audit live repository metadata (`gh repo view`) ensuring trimmed `description` and accurate topics (`FR-010`, optionally updating topics with maintainer approval per `FR-012` via `PUT repos/Ezenitac-AiService/ImageViewer/topics`) (`FR-010`, `FR-012`, `SC-005`; depends on `T031`)
- [X] T033 Under explicit maintainer approval (`FR-012`, Clarification `Q2 Option A`), delete pre-amendment draft release (`gh release delete v1.1.0-alpha.2 --yes`), dispatch `gh workflow run release.yml --ref main -f tag=v1.1.0-alpha.2`, watch workflow completion, verify regenerated `v1.1.0-alpha.2` draft contains ONLY `ImageViewer-macos-universal.dmg`, `ImageViewer-linux-x64.tar.gz`, `SHA256SUMS.txt`, and `launch-verification-macos-*.json` (zero `.exe`, zero `linux` verification JSON), verify local macOS DMG (`shasum`, `codesign`, `lipo`, `launch-check.sh`), and refresh clean-device `macOS` manual checklist scaffold (`FR-002`, `FR-003`, `FR-004`, `FR-006`, `FR-011`, `SC-001`–`SC-003`; depends on `T031`)
- [X] T034 Run read-only remote verification audit (`quickstart.md` §3), update `**Status**: Implemented` in `specs/006-platform-tier-realignment/spec.md`, record final remote/DMG evidence in `specs/006-platform-tier-realignment/research.md`, mark `tasks.md` `[X]`, land final status sync via clean non-`NNN-*` PR onto `main` under `Protect main`, and run `/speckit-analyze` across `specs/006-platform-tier-realignment/{spec.md,plan.md,tasks.md}` reporting macOS host vs. CI verification scope (`FR-012`; depends on `T032`, `T033`)

---

## Dependencies & Execution Order

### Strict Phase & Ordering Constraints

- **Setup (Phase 1, `T001`–`T003`)**: Starts immediately; `T002` and `T003` run in parallel once `T001` isolates `chore/006-platform-realignment`.
- **Foundational (Phase 2, `T004`–`T012`)**: `T004`–`T007` and `T011` run in parallel after `T001`–`T003`; `T008`–`T010` run in parallel **after** `T007` updates workspace tests first (Red→Green TDD per Constitution III); `T012` validates the entire clean workspace and 3-target component count before Phases 3–6.
- **US1 (Phase 3, `T013`–`T016`)**: `T013` and `T014` run in parallel; `T015` (`Protect main` ruleset update) MUST complete **before** `T031` merges the PR changing `ci.yml` check names (`research.md` R1).
- **US2 (Phase 4, `T017`–`T019`)**: `T017` and `T018` start once Phase 2 passes; `T019` verifies `.github/workflows/{ci.yml,release.yml}` after `T013`/`T014`/`T018`.
- **US3 (Phase 5, `T020`–`T022`)**: `T020` follows `T013` (`ci.yml`); `T021` follows `T014` (`release.yml`); `T022` runs in parallel with `T020`/`T021`.
- **US4 (Phase 6, `T023`–`T029`)**: `T023`–`T027` run in parallel; `T028` runs after `T012` gives the exact updated 3-target component count; `T029` audits all pairs after `T023`–`T028`.
- **Polish & Draft Regeneration (Phase 7, `T030`–`T034`)**: `T030` runs after all local file edits (`T004`–`T029`); `T031` merges PR into `main` after `T015` and `T030`; `T032` and `T033` run after `T031`; `T034` finalizes status sync after `T032` and `T033`.
- **Convergence (Phase 8, `T035`–`T040`)**: `T035`–`T039` execute cleanly in parallel after `T034` (closing `.cargo/config.toml`, `deny.toml`, `quickstart.md` §4 manual checklist, `launch-check.sh` non-Darwin cleanup, and OS-neutral `i18n.rs`/`update.rs` wording), followed sequentially by `T040` verification, PR `#8` merge, and final `v1.1.0-alpha.2` draft regeneration.

---

## Parallel Example: Foundational Cleanup & Bilingual Docs

```bash
# Phase 2 disjoint cleanups in parallel:
Task: "T004 Remove temporary Sync Impact Report comment in .specify/memory/constitution.md"
Task: "T005 Delete packaging/windows/ and Windows PowerShell verification scripts"
Task: "T006 Remove winresource/windows/winreg dependencies and crates/iv-app/build.rs"
Task: "T007 Update Windows-only unit/integration tests while keeping Zero-Trust tests"
Task: "T011 Update packaging/verify/component-count.sh targets to 3 active targets"

# Phase 6 bilingual doc updates in parallel:
Task: "T023 Update .github/ISSUE_TEMPLATE/bug_report.yml OS options"
Task: "T024 Update README.md and README.en.md in lockstep"
Task: "T025 Update docs/CAPABILITIES.md and docs/CAPABILITIES.en.md in lockstep"
Task: "T026 Update docs/FIRST_LAUNCH.md and docs/FIRST_LAUNCH.en.md in lockstep"
Task: "T027 Update CONTRIBUTING.md and CONTRIBUTING.en.md in lockstep"
```

---

## Implementation Strategy

### MVP First (Governance, Clean Build & Exclusive macOS Verification — Phases 1–4)

1. Complete Phase 1 (`T001`–`T003`) and Phase 2 (`T004`–`T012`) to strip Windows build dependencies/adapters while preserving Zero-Trust guards (`Q4`, `Q5`).
2. Complete Phase 3 (`US1`, `T013`–`T016`) to remove Windows workflow jobs and update `Protect main` to the 4 post-`v3.0.0` contexts without deadlock.
3. Complete Phase 4 (`US2`, `T017`–`T019`) so top-priority `macOS` build-result verification and release-note disclosures are ready.

### Incremental Delivery

1. **Increment 1 (`T001`–`T019`)**: Clean Windows-free Rust workspace, `v3.0.0` constitution ready, `Protect main` ruleset aligned, and `macOS`-only launch verification active (`US1` + `US2` MVP).
2. **Increment 2 (`T020`–`T022`)**: `Linux` stripped of runtime launch verification and packaged strictly as build-only unverified (`US3`).
3. **Increment 3 (`T023`–`T029`)**: Bug-report OS dropdown and all 8 Korean/English documentation pairs synchronized in lockstep (`US4`).
4. **Increment 4 (`T030`–`T034`)**: Full local + PR CI pass, `main` merge, repository metadata audit, clean `v1.1.0-alpha.2` draft regeneration, and `/speckit-analyze` verification sign-off.
5. **Increment 5 (`T035`–`T040`)**: Phase 8 convergence cleanup (`.cargo/config.toml`, `deny.toml`, `quickstart.md` §4 `macOS` clean-device manual checklist, `launch-check.sh` Darwin-only cleanup, OS-neutral `i18n.rs`/`update.rs` wording), PR `#8` merge, and final `v1.1.0-alpha.2` draft regeneration.

---

## Phase 8: Convergence

- [X] T035 Remove Windows-only Cargo build configuration `.cargo/config.toml` (`[target.x86_64-pc-windows-msvc]` MSVC CRT static-link flags) per FR-013, SC-001 (contradicts)
- [X] T036 Remove excluded `"x86_64-pc-windows-msvc"` target from `deny.toml` `[graph].targets` and re-verify `cargo deny` default and `--all-features` checks per FR-005, SC-001, SC-006 (contradicts)
- [X] T037 Add clean-device `macOS` (Apple Silicon & Intel) manual release verification checklist and `launch-verification-manual.md` scaffold (retiring Windows clean-machine obligations and deferring `iOS` to `007`) to `specs/006-platform-tier-realignment/quickstart.md` referenced by `CONTRIBUTING.md`/`CONTRIBUTING.en.md` per FR-011, T033 (missing)
- [X] T038 Remove dead non-Darwin Linux X11/Xvfb/`xdotool` runtime launch-check branches from `packaging/verify/launch-check.sh` so the script strictly contains `macOS` (`Darwin`) verification logic per FR-003, FR-004, SC-003 (partial)
- [X] T039 Replace residual Windows-specific wording in `crates/iv-core/src/i18n.rs` (`error.unsupported_os_switch`) and `crates/iv-platform/src/update.rs` (`UpdateError::UnsafeRelaunchPath`, preserving OS-agnostic `validate_relaunch_path` sanitization) with OS-neutral messages per FR-013, SC-001 (partial)
- [X] T040 Verify Phase 8 convergence edits with TDD test updates (`logger_and_i18n_tests.rs`, `update_tests.rs`) and full `quickstart.md` §1 + `cargo deny` (default & `--all-features`) suite, merge via non-`NNN-*` PR (`chore/006-convergence`) onto `Protect main` with all 4 required checks passing, restore `**Status**: Implemented` in `spec.md`, and confirm/regenerate `v1.1.0-alpha.2` draft under maintainer approval (`FR-012`) if shipped `iv-app` binary changes per FR-005, FR-012, SC-001, SC-006 (partial)
