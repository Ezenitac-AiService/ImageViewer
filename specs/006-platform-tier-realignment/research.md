# Phase 0 Research: Platform Tier Realignment (Constitution v3.0.0)

**Feature Branch**: `006-platform-tier-realignment`  
**Date**: 2026-11-24  
**Spec**: [spec.md](./spec.md)

This document consolidates empirical findings from codebase, workflow, packaging, bilingual documentation, and live GitHub repository audits governing downstream alignment with **Constitution v3.0.0** (`macOS` & `iOS` top priority with exclusive build-result verification, `macOS` active verified + `iOS` planned `007`; `Linux` & `Android` lower priority build-only without verification, `Linux` active build-only + `Android` planned `007`; `Windows` excluded from active support while preserving OS-agnostic Zero-Trust guards and historical GPLv3 origin records).

---

## R1. Deadlock-Safe `Protect main` Ruleset Transition (`FR-007`, `FR-012`, `SC-004`)

- **Decision**:
  Under explicit maintainer approval (`FR-012`), update the live `Protect main` Repository Ruleset (`id: 27903505`, `PUT repos/Ezenitac-AiService/ImageViewer/rulesets/27903505`) **immediately before** merging the `006` realignment PR onto `main`, replacing the 6 post-`004` required status check contexts with the exact 4 realigned `006` contexts pinning `integration_id: 15368`:
  1. `Checks (ubuntu-latest)` (`integration_id: 15368`) — Linux build/clippy/test/bench compile check (`Q3 Option A`)
  2. `Checks (macos-latest)` (`integration_id: 15368`) — macOS build/clippy/test/bench compile check
  3. `Launch check (macOS)` (`integration_id: 15368`) — exclusive top-priority launch/render/exit gate
  4. `Dependencies (cargo-deny, budget)` (`integration_id: 15368`) — `cargo deny` + 3-target component budget gate
  Preserve `enforcement: "active"`, `bypass_actors: []`, `deletion`, `non_fast_forward`, `pull_request` (`required_approving_review_count: 0`), and `strict_required_status_checks_policy: true`. Retires both obsolete Windows contexts (`Checks (windows-latest)`, `Launch check (Windows)`).
- **Rationale**:
  - `Protect main` has `bypass_actors: []` (`current_user_can_bypass: "never"`). Once `006` removes `windows-latest` and `launch-windows` from `.github/workflows/ci.yml`, a PR opened against `main` will not emit `Checks (windows-latest)` or `Launch check (Windows)`. If `Protect main` still requires those 2 retired Windows checks, the PR deadlocks permanently.
  - Updating the ruleset immediately *before* merging the `006` PR ensures the PR only needs those 4 checks, while all 4 checks already run cleanly and pass on the `006` PR head (`ubuntu-latest`, `macos-latest`, `Launch check (macOS)`, `Dependencies`).
- **Alternatives Considered**:
  - *Delete and recreate the ruleset after merging*: Rejected; leaves `main` briefly unprotected or forces an unmergeable deadlock on the `006` PR itself.
  - *Drop `Checks (ubuntu-latest)` from required checks*: Rejected per Clarification `Q3 (Option A)` and Constitution `v3.0.0` ("Lower-priority targets (`Linux`, `Android`) require build checks only with zero runtime verification").
- **Primary Evidence / Observed Output**:
  - Live query `gh api repos/Ezenitac-AiService/ImageViewer/rulesets/27903505` confirmed `id: 27903505`, `bypass_actors: []`, `strict_required_status_checks_policy: true`, and 6 active contexts including `Checks (windows-latest)@15368` and `Launch check (Windows)@15368`.
  - `.github/workflows/ci.yml` job names: `Checks (${{ matrix.os }})`, `Launch check (macOS)`, `Dependencies (cargo-deny, budget)`.

---

## R2. CI & Release Workflow Realignment (`FR-001`–`FR-005`, `SC-001`–`SC-003`)

- **Decision**:
  1. Update `.github/workflows/ci.yml`:
     - Keep `permissions: { contents: read }` and 100% 40-char SHA-pinned external Actions (`actions/checkout@11d5960a326750d5838078e36cf38b85af677262`, `dtolnay/rust-toolchain@39008827e6b4319a349e9a42dfc90579dd25f1ca`, `Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0`, `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`, `EmbarkStudios/cargo-deny-action@17aeb3c839e42274e2b8adf721c8c65edfdc74dd`).
     - Update `checks.strategy.matrix.os` from `[ubuntu-latest, macos-latest, windows-latest]` to `[ubuntu-latest, macos-latest]`.
     - Retain `launch-macos` (`Launch check (macOS)`) as the sole launch-verification job and `dependencies` (`Dependencies (cargo-deny, budget)`).
     - Remove `launch-windows` **and** remove `launch-linux` (`Launch check (Linux, experimental)` — Constitution `v3.0.0` prohibits runtime/launch verification on lower-priority `Linux`).
     - Update header comments (lines 3–9) removing Windows references cleanly.
  2. Update `.github/workflows/release.yml`:
     - Remove `build-windows`, `verify-windows`, and `verify-linux` completely.
     - Retain `build-macos`, `build-linux` (renaming step to package build-only unverified Linux tarball), `verify-macos` (`macos-latest` arm64 & `macos-15-intel` x64), `deny`, and `publish-draft`.
     - Update `publish-draft.needs` to `[verify-macos, build-linux, deny]` and `if:` condition to `always() && needs.verify-macos.result == 'success' && needs.build-linux.result == 'success' && needs.deny.result == 'success'` (zero `verify-linux`, zero `verify-windows`).
     - Update `Checksums and release notes` (`sha256sum ImageViewer-macos-universal.dmg ImageViewer-linux-x64.tar.gz > SHA256SUMS.txt`, keeping `env.TAG` + SemVer regex guard from `005` `T044`) and `softprops/action-gh-release` `files:` to upload `ImageViewer-macos-universal.dmg`, `ImageViewer-linux-x64.tar.gz`, `SHA256SUMS.txt`, and `launch-verification-macos-*.json` only (zero Windows deliverables, zero Linux verification JSONs).
     - Update `.github/workflows/release.yml` lines 3–6 comments to reflect `v3.0.0` (top-priority `macOS` launch verification + build-only unverified `Linux` tarball).
- **Rationale**:
  - Satisfies `FR-001`–`FR-005` and Clarification `Q3 (Option A)`: `macOS` remains exclusively launch-verified; `Linux` retains compile/lint/workspace checks in CI and produces a build-only tarball in releases with zero runtime verification (`xvfb-run launch-check.sh` completely eliminated); `Windows` is completely absent.
  - Preserves every supply-chain invariant from `005` (`contents: read` top-level, job-scoped `contents: write` on `publish-draft`, 40-char SHA pinning, `env.TAG` shell-injection protection).
- **Alternatives Considered**:
  - *Keep `verify-linux` as `continue-on-error: true`*: Rejected because Constitution `v3.0.0` explicitly states lower-priority targets MUST NOT execute build-result or launch verification ("하위는 빌드만 제공 - 확인 작업을 하지 않음").
  - *Drop `ubuntu-latest` from `ci.yml` `checks` matrix*: Rejected per Clarification `Q3 (Option A)`.
- **Primary Evidence / Observed Output**:
  - `.github/workflows/ci.yml` lines 20–127 (`checks`, `launch-macos`, `launch-windows`, `launch-linux`, `dependencies`).
  - `.github/workflows/release.yml` lines 26–202 (`build-windows`, `verify-windows`, `verify-linux`, `publish-draft`).

---

## R3. Windows-Only Code, Dependencies & Packaging Removal vs. Zero-Trust Preservation (`FR-013`, `SC-001`, Clarifications Q4 & Q5)

- **Decision**:
  Separate active workspace files strictly into **Remove (Windows-target-only)** vs. **Preserve (OS-agnostic Zero-Trust & `legacy-v1.0.6` archive)**:
  1. **Remove (Packaging, Manifests, Scripts)**:
     - Delete directory `packaging/windows/` (`ImageViewer.ico`, `ImageViewer.manifest`, `ImageViewer.nsi`, `package-windows.ps1`).
     - Delete `packaging/verify/assert-windows-exe.ps1` and `packaging/verify/launch-check.ps1`.
  2. **Remove (Cargo Manifests & Build Scripts)**:
     - `Cargo.toml`: remove `winresource = "0.1"` (`L56`); update `Cargo.lock` via `cargo update` or `cargo check --workspace` (preserving pinned `url = 2.5.9` / `zip = 9.6.1`).
     - `crates/iv-app/Cargo.toml`: remove `[target.'cfg(windows)'.build-dependencies]` and `winresource.workspace = true` (`L21-22`).
     - Delete `crates/iv-app/build.rs` (only exists to embed Windows resources and print non-Windows build warnings).
     - `crates/iv-platform/Cargo.toml`: remove `[target.'cfg(target_os = "windows")'.dependencies]` (`windows = 0.59`, `winreg = 0.55`, `L26-33`), remove `"dep:windows"` from `os-codecs` (`L48`), and change `shell-registration = ["dep:winreg"]` (`L49`) to `shell-registration = []` (preserving `#[cfg(feature = "shell-registration")]` in `crates/iv-platform/src/lib.rs:L8` and `crates/iv-platform/tests/os_integration_tests.rs:L4` for Linux XDG desktop integration under `clippy -D warnings`, while keeping `--register-shell`/`--unregister-shell` fail-closed rejection in `iv-core`).
  3. **Remove (Windows-Target Runtime & Test Branches)**:
     - `crates/iv-app/src/main.rs`: remove `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]` (`L7`), and stop passing `cfg!(target_os = "windows")` (`L87`).
     - `crates/iv-app/src/fonts.rs`: remove `"C:\\Windows\\Fonts\\malgun.ttf"` (`L9`) from `CJK_CANDIDATES`.
     - `crates/iv-core/src/cli.rs`: simplify `CliOptions::parse(&args, lang)` (removing `is_windows: bool` param, `/` switch handling `L48-50` + `/register-shell`/`/unregister-shell`, and `repair_explorer_quoted_drive` `L87-98`) while keeping `--register-shell` / `--unregister-shell` fail-closed `error.unsupported_switch` and unknown `-` flag rejection. Update `crates/iv-core/tests/cli_tests.rs` accordingly (removing `repairs_explorer_trailing_quote_on_windows_drive` and `is_windows` loop).
     - `crates/iv-core/src/path_security.rs`: remove `#[cfg(target_os = "windows")]` case-insensitive branch in `is_within_root` (`L36-46`), leaving unified containment check cleanly.
     - `crates/iv-platform/src/codecs.rs`: remove `#[cfg(target_os = "windows")]` WIC implementation (`L163-238`), update `#[cfg(not(target_os = "macos"))]` fallback guard, and update doc comment (`L1`). Also update `crates/iv-core/src/codecs.rs` (`L5`) comment.
     - `crates/iv-platform/src/paths_dialogs.rs`: remove `#[cfg(target_os = "windows")]` `LOCALAPPDATA` branch (`L21-26`).
     - `crates/iv-platform/src/fatal_dialog.rs`: remove `"windows"` arm in `redacted_log_folder` (`L11`) and corresponding `"windows"` expectation in `crates/iv-platform/tests/os_integration_tests.rs` (`L33, L44`).
     - `crates/iv-platform/src/integration.rs`: remove `format_windows_open_command`, `register_windows_shell`, `unregister_windows_shell` (`L25-77`) and `format_windows_open_command` test (`os_integration_tests.rs:L17`).
     - `crates/iv-platform/src/update.rs`: remove `#[cfg(target_os = "windows")]` `cmd /C start` arm (`L278-283`) and `"windows"` arm in `select_asset_for_os` (`L312`), plus the `"windows"` asset test in `crates/iv-platform/tests/update_tests.rs` (`L140-161`).
  4. **Strictly Preserve (Clarification Q5 & Constitution VII)**:
     - Preserve `PathSecurity::normalize_archive_entry_key` (`crates/iv-core/src/path_security.rs:L65-92`) normalizing `\` → `/` and blocking drive prefixes (`C:`), leading `/`, and `..` traversal in ZIP/CBZ archives across macOS/Linux.
     - Preserve `PathSecurity::is_archive_symlink` (`L95-104`, checking both POSIX `0xA000` and DOS `0x0400` attributes) and `PathSecurity::is_safe_single_filename` (`L107-114`, rejecting `\` and `/`).
     - Preserve `PathRedactor::redact` (`L129-240`) stripping UNC `\\server\share` and Windows `C:\...` drive paths alongside Unix `/...` paths, and keep `diagnostics_tests::source_keeps_file_name_only_for_unix_and_windows_paths` (`crates/iv-core/tests/diagnostics_tests.rs:L71`) + all `path_security_tests` / `sources_tests` negative tests intact.
- **Rationale**:
  - Eliminates dead Windows-only dependencies (`windows 0.59`, `winreg 0.55`, `winresource 0.1`), reduces `Cargo.lock` surface, removes dead `#[cfg(windows)]` build/platform code that would never be tested or shipped under `v3.0.0`, and satisfies `Q4 (Option A)` + `Q5 (Option A)` without relaxing any archive/log sanitization guarantees on `macOS` or `Linux`.
- **Alternatives Considered**:
  - *Keep `#[cfg(windows)]` Rust code untouched (Q4 Option B/C)*: Rejected by maintainer in Q4 (`Option A`).
  - *Also strip backslash Zip-Slip or Windows-path log redaction*: Rejected by maintainer in Q5 (`Option A`) under Zero Trust (malicious `.zip`/`.cbz` archives or strings created on Windows still reach `macOS`/`Linux` viewers).
- **Primary Evidence / Observed Output**:
  - Verified line-by-line via `grep -rnE` and `read_file` across `Cargo.toml`, `crates/iv-app/{Cargo.toml,build.rs,src/main.rs,src/fonts.rs}`, `crates/iv-core/src/{cli.rs,path_security.rs}`, `crates/iv-platform/{Cargo.toml,src/*,tests/*}`, and `packaging/`.
  - Notice `iv-media/src/{sources/zip.rs,animation.rs}` `.windows(4)` calls are Rust slice iteration (`slice::windows`), **not** Microsoft Windows code — must NOT be modified!

---

## R4. Packaging Scripts, Release Notes & Component Budget (`FR-006`, `SC-002`–`SC-003`, `SC-006`)

- **Decision**:
  1. Update `packaging/verify/component-count.sh`:
     - Update `TARGETS=(aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu)` (removing `x86_64-pc-windows-msvc`). Keep `FORBIDDEN=(reqwest hyper rustls tokio)` and verify resulting count `<= 356` (previously `298` across 4 targets; 3 targets will be equal or slightly lower).
     - Run `rustup target list --installed` check/note in `ci.yml` `dependencies` job? Notice `packaging/verify/component-count.sh` uses `cargo tree --target "$t"` (which resolves metadata from `Cargo.lock` / Rust `target` specs without needing compiled stdlib targets installed, already working cleanly on `ubuntu-latest` CI!).
  2. Update `packaging/release-notes.md.in` and `packaging/verify/render-release-notes.py`:
     - Remove `Windows 10`, `Windows 11` rows and Smart App Control notice from `packaging/release-notes.md.in`.
     - Replace `experimental` on `Linux` with `build-only (unverified)` and set `Automated` column for `Linux` (`x64`) to `n/a (build-only)` (zero automated verification performed).
     - Add honest roadmap rows for `iOS` (`aarch64`, `n/a` / `n/a` / `planned top-priority (feature 007; not yet built)`) and `Android` (`aarch64`, `n/a` / `n/a` / `planned build-only (feature 007; not yet built)`), so release notes match Constitution `v3.0.0` + Clarification `Q1 (Option B)` with zero overstated mobile claims.
     - Simplify `packaging/verify/render-release-notes.py` so it reads `macos-arm64` and `macos-x64` records only (drop `WIN_AUTO`, `LINUX_AUTO`, `@WIN10@`, `@WIN11@`).
  3. Update `packaging/linux/package-linux.sh` comment (`experimental` → `lower-priority build-only, unverified`) and `packaging/verify/launch-check.sh` header/platform scope (`macOS` top-priority verification).
- **Rationale**:
  - Prevents `render-release-notes.py` from printing `"not run"` for absent `windows-x64`/`linux-x64` records and guarantees `SC-002` & `SC-003` honest platform status disclosure in every release.
- **Alternatives Considered**:
  - *Omit `iOS` and `Android` rows from release notes completely*: Considered; including explicit `planned (feature 007)` rows or clear known-limitation notes avoids confusing readers comparing repository constitution `v3.0.0` against `v1.1.0-alpha.2` assets. Keeping concise table rows with `n/a` + `planned (007; not in this release)` satisfies both `FR-006` and `SC-002`/`SC-003`.
- **Primary Evidence / Observed Output**:
  - `packaging/release-notes.md.in` lines 14–34, `packaging/verify/render-release-notes.py` lines 40–68, `packaging/verify/component-count.sh` lines 13–14.

---

## R5. Bilingual Documentation, User Guides & Issue Intake Alignment (`FR-008`–`FR-009`, `SC-005`)

- **Decision**:
  Update every bilingual pair in lockstep (`*.md` Korean default + `*.en.md` English counterpart with intact top language switchers per Constitution Principle I):
  1. `README.md` / `README.en.md`:
     - Update platform badge, intro paragraph, upstream comparison table (`지원 OS` / `OS`: active **macOS** + planned **iOS**, build-only **Linux** + planned build-only **Android**, excluded **Windows**), shortcut note (`Cmd` on macOS, `Ctrl` on Linux — removing active Windows reference), and `Supported Platforms & Releases` table (`macOS` top-priority `supported`, `iOS` `planned (007)`, `Linux` `build-only, unverified`, `Android` `planned build-only (007)`).
     - Keep `Fork Etiquette`, `edith2580/ImageViewer (v1.0.6)` upstream link, and `legacy-v1.0.6` archive link intact (`SC-001` / Principle VIII).
  2. `docs/CAPABILITIES.md` / `docs/CAPABILITIES.en.md`:
     - Update Core/Extended rules to cite top-priority targets (`macOS` active, `iOS` planned `007`) and lower-priority build-only `Linux` (unverified; `Android` planned `007`), remove `Windows` from active verification status and `EXTENDED` release criteria, and remove `Windows "Open with"` from active `open-with-os` claim. Keep `specs/004` evidence trace accurate.
  3. `docs/FIRST_LAUNCH.md` / `docs/FIRST_LAUNCH.en.md`:
     - Remove `## Windows 10 / 11` installation/SmartScreen section and `%LOCALAPPDATA%\ImageViewer\logs` table row.
     - Keep `## macOS` first-launch steps cleanly as top-priority, change `## Linux` heading/copy from `experimental` to `Lower-Priority Build-Only (Unverified)` (`하위 우선순위 빌드 전용 (미검증)`), and add a short clear note that `iOS` / `Android` guides will arrive with `007`.
  4. `CONTRIBUTING.md` / `CONTRIBUTING.en.md`:
     - Update Definition of Done and Release Procedure (`macOS` active launch verification; `Linux` build-only without verification; `Windows` excluded; clean-machine checklist targets `macOS` (`Apple Silicon` / `Intel`), removing Windows VM obligation).
  5. `NOTICE` / `NOTICE.en.md` & `THIRD_PARTY_NOTICES.md` / `THIRD_PARTY_NOTICES.en.md` & `CHANGELOG.md` / `CHANGELOG.en.md`:
     - `NOTICE` / `NOTICE.en.md`: preserve `2026-11-16` initial separation and `2026-11-17` (`004`) entries, add `2026-11-24` platform-tier realignment modification date (`macOS` active top-priority, `Linux` build-only, `Windows` retired from active runtime targets), preserving `edith2580` `v1.0.6` origin notice (`FR-008`, GPLv3 §5(a)).
     - `THIRD_PARTY_NOTICES.md` / `THIRD_PARTY_NOTICES.en.md`: remove `windows (WIC)` from `iv-platform` `os-codecs` row (`L47`) cleanly.
     - `CHANGELOG.md` / `CHANGELOG.en.md`: update unreleased `1.1.0-alpha.2` section (`2026-11-16–24` date, retire Windows runtime/fix entries from active shipped target promises or add `Changed` entry documenting Constitution `v3.0.0` Windows retirement, `macOS` top-priority verification, and `Linux` build-only status accurately prior to regenerating the `v1.1.0-alpha.2` draft). *Check `Cargo.lock` / `298` count text in `THIRD_PARTY_NOTICES*` and re-sync exact `component-count.sh` count after removing `winresource`/`windows`/`winreg`!*
  6. `.github/ISSUE_TEMPLATE/bug_report.yml`:
     - Replace `options: [Windows, macOS, Linux]` (`L24-26`) with `macOS`, `Linux (빌드 전용·미검증 / Build-Only, Unverified)`, `iOS (출시 예정 / Planned — Not Yet Available)`, `Android (빌드 전용 출시 예정 / Planned Build-Only)` (omitting `Windows` as an active target while keeping bilingual `fork-confirmation` `required: true`). Audit `feature_request.yml` and `config.yml` (already aligned with upstream routing!).
- **Rationale**:
  - Prevents contradiction between Constitution `v3.0.0` and public repository pages (`SC-005`) while keeping `THIRD_PARTY_NOTICES*` component counts truthful after Windows dependency removal.
- **Alternatives Considered**:
  - *Edit only `README*` and leave `CAPABILITIES*`/`FIRST_LAUNCH*`/`CONTRIBUTING*` stale*: Rejected; leaves broken instructions and violates Constitution Principle I/VI/VIII and `FR-008`.
- **Primary Evidence / Observed Output**:
  - `README.md`/`README.en.md`, `docs/CAPABILITIES.md`/`docs/CAPABILITIES.en.md`, `docs/FIRST_LAUNCH.md`/`docs/FIRST_LAUNCH.en.md`, `CONTRIBUTING.md`/`CONTRIBUTING.en.md`, `THIRD_PARTY_NOTICES.md`/`THIRD_PARTY_NOTICES.en.md`, `.github/ISSUE_TEMPLATE/bug_report.yml` inspected verbatim.

---

## R6. Pre-Amendment `v1.1.0-alpha.2` Draft Regeneration & Constitution Finalization (`FR-002`, `FR-012`, Clarification Q2)

- **Decision**:
  1. Clean up `.specify/memory/constitution.md` (`v3.0.0`) by removing its temporary HTML review comment (`Sync Impact Report`, currently lines 1–18) so `# ImageViewer Constitution` sits cleanly at line 1 before committing with `006`.
  2. Update `specs/004-launch-reliability-refactor/tasks.md`? Wait — **Important Boundary (`Assumptions`)**: spec `Assumptions` states completed/historical records `specs/001-*`–`specs/005-*` MUST NOT be retroactively rewritten; instead, `006`'s own post-merge release verification / draft regeneration task executes clean `v1.1.0-alpha.2` draft creation (`gh release delete v1.1.0-alpha.2 --yes` then `gh workflow run release.yml --ref main -f tag=v1.1.0-alpha.2`) under explicit maintainer approval (`FR-012`), leaving old `004` historical prose untouched (or only referencing clean-machine manual checklist updates cleanly). Wait: what about active release checklists (`FR-011`)? `docs/FIRST_LAUNCH.md`, `CONTRIBUTING*`, and a clean `macOS`-only manual verification guide (`specs/006-platform-tier-realignment/quickstart.md` / `v1.1.0-alpha.2` regenerated `launch-verification-manual.md` scaffold) fulfill `FR-011` without rewriting archived `specs/001`–`005` specs!
- **Rationale**:
  - Preserves historical integrity of `specs/001`–`005` (`Assumptions`), cleanly finalizes `constitution.md` (`v3.0.0`), and ensures the unpublished `v1.1.0-alpha.2` draft on GitHub contains zero Windows assets and zero Linux verification JSONs (`Q2 Option A`).
- **Alternatives Considered**:
  - *Retroactively edit `specs/004-launch-reliability-refactor/`*: Rejected by `006` `Assumptions` ("Historical Spec Preservation").
- **Primary Evidence / Observed Output**:
  - Existing draft `v1.1.0-alpha.2` observed via `gh release view v1.1.0-alpha.2` (`isDraft: true`, `isPrerelease: true`, contains `ImageViewer-windows-x64.exe` and `launch-verification-linux-x64.json`).
  - `.specify/memory/constitution.md` lines 1–18 observed containing temporary `Sync Impact Report` HTML comment.

---

## R7. Repository Metadata & Verification Check Continuity (`FR-010`, `FR-012`, `SC-006`)

- **Decision**:
  - Audit repository `description` (`"A fast, GPU-accelerated, Zero-Trust cross-platform image and comic archive viewer written in Rust"`) and `topics` (`cbz`, `comic-viewer`, `cross-platform`, `egui`, `image-viewer`, `rust`, `wgpu`, `zero-trust`). None explicitly claim Windows; optionally add `macos` topic under maintainer approval (`FR-010`, `FR-012`) while preserving zero trailing spaces in `description` and `>= 5` curated topics.
  - Preserve all `005` workflow security guarantees: `dependency-audit.yml` (`shipped-default` & `--all-features`), `cargo deny check advisories bans licenses sources` + `--all-features` (`deny.toml` `ignore = []`), 40-char Action SHA pinning + `sha_pinning_required: true`, `env:` injection guard (`T044`), and clean local/remote branch hygiene via non-`NNN-*` PR branch (e.g., `chore/006-platform-realignment`).
- **Rationale**:
  - Avoids duplicate `ci.yml` `push` + `pull_request` runs under active `Protect main` (`005` `P1` lesson learned), keeps metadata truthful, and preserves 100% of `005` governance hardening.
- **Alternatives Considered**:
  - *Open PR from `006-platform-tier-realignment` branch*: Rejected; `[0-9][0-9][0-9]-*` triggers duplicate `push` and `pull_request` CI workflows, wasting macOS runner capacity.
- **Primary Evidence / Observed Output**:
  - Live `gh repo view` output verified `description` has 0 trailing whitespace and 8 clean topics; `.github/workflows/ci.yml:L10` matches `[0-9][0-9][0-9]-*`.
