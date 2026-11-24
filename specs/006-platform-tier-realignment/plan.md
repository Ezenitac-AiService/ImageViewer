# Implementation Plan: Platform Tier Realignment (Constitution v3.0.0)

**Branch**: `006-platform-tier-realignment` | **Date**: 2026-11-24 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/006-platform-tier-realignment/spec.md`

## Summary

Realign CI/release workflows, default-branch merge gates, workspace manifests/crates, packaging scripts, issue intake templates, bilingual root/user documentation, and the unpublished `v1.1.0-alpha.2` draft release with **Constitution v3.0.0** and Clarifications `Q1`–`Q5`:
1. **Retire Windows (`FR-001`–`FR-002`, `FR-013`, `Q4`, `Q5`)**: Remove Windows CI/release jobs, `packaging/windows/`, Windows-only PowerShell verification scripts, `winresource`/`windows`/`winreg` dependencies, and `#[cfg(target_os = "windows")]` adapters/branches, while strictly preserving all OS-agnostic Zero-Trust security guards (ZIP backslash Zip-Slip normalization, `PathRedactor` UNC/drive redaction, `150 MP`/`512 MB`/`64 KB` caps) and historical `legacy-v1.0.6` GPLv3 attribution.
2. **Exclusive Top-Priority `macOS` Verification & Honest `iOS` Roadmap Boundary (`FR-003`, `FR-006`, `Q1`)**: Keep `macOS` (`aarch64` & `x86_64`) builds and automated launch/render verification strictly blocking; defer `iOS` mobile shell/verification to follow-up feature `007` and mark `iOS` honestly as planned (`007`) in docs and release notes.
3. **Lower-Priority Build-Only `Linux` (`FR-004`, `FR-007`, `Q3`)**: Retain Linux compile/lint/test build checks as a required PR gate (`Checks (ubuntu-latest)`) and package `ImageViewer-linux-x64.tar.gz` in releases with **zero** runtime launch checks (`launch-linux` and `verify-linux` removed) and explicit `build-only / unverified` labeling (`Android` planned for `007`).
4. **Deadlock-Safe Governance & Draft Regeneration (`FR-007`–`FR-012`, `Q2`)**: Update `Protect main` (`id: 27903505`) to the 4 post-realignment required checks before merging the `006` PR, finalize `.specify/memory/constitution.md` (`v3.0.0`, removing temporary review comment), sync all 8 bilingual Korean/English documentation pairs + `bug_report.yml`, and delete/regenerate the `v1.1.0-alpha.2` draft cleanly from `main`.

## Technical Context

**Language/Version**: Rust Workspace `1.95+` (`Edition 2024`), GitHub Actions YAML, GitHub Issue Form YAML, Bash, Python 3, Bilingual Markdown (`ko`/`en`), GitHub REST API (`gh` CLI)

**Primary Dependencies**: Workspace crates `iv-core`, `iv-media`, `iv-platform`, `iv-app`; `eframe` `0.37`, `winit` `0.30`, `objc2*` (`0.3`/`0.6` on macOS), `rfd` `0.15`, `directories` `6`, `zip` `9` (`deflate`, `zstd`); GitHub Actions pinned to verified 40-char commit SHAs (`actions/checkout`, `actions/upload-artifact`, `actions/download-artifact`, `dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `EmbarkStudios/cargo-deny-action`, `softprops/action-gh-release`)

**Storage**: Repository files (`Cargo.toml`, `Cargo.lock`, `crates/`, `packaging/`, `.github/`, `.specify/memory/constitution.md`, bilingual root/`docs/` Markdown) and live GitHub Ruleset `Protect main` (`id: 27903505`) + Draft Release `v1.1.0-alpha.2` (no runtime database changes)

**Testing**: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets` (default & `--all-features`), `cargo test --workspace` (default & `--all-features` on macOS host & Ubuntu/macOS CI), `cargo deny check advisories bans licenses sources` (default & `--all-features`), `./packaging/verify/component-count.sh` (3-target budget), `packaging/verify/launch-check.sh` (`macOS` only), YAML/SHA/script-injection/bilingual-link verification (`quickstart.md`)

**Target Platform**: Top Priority (`macOS` `aarch64` & `x86_64` verified; `iOS` `aarch64` planned in `007`); Lower Priority Build-Only (`Linux` `x86_64` build-only unverified; `Android` planned in `007`); Excluded (`Windows`)

**Project Type**: Governance, CI/release pipeline, build/platform-cleanup, and bilingual documentation realignment across a 4-crate Rust desktop workspace

**Performance Goals**: Eliminate all Windows runner and Linux Xvfb launch-verification runtime overhead from CI and release workflows while preserving `<= 356` 3-target shipped component count and `004` runtime responsiveness on `macOS`

**Constraints**: Constitution `v3.0.0` (`I`–`VIII`), Clarification `Q1`–`Q5`, explicit maintainer approval prior to every live remote mutation (`FR-012`), deadlock-safe `Protect main` transition prior to `006` PR merge (`R1`), zero runtime verification jobs on `Linux` (`FR-004`), strict preservation of OS-agnostic Zero-Trust backslash Zip-Slip & path-redaction guards (`Q5`)

**Scale/Scope**: 3 Cargo manifests + `Cargo.lock` + `.cargo/config.toml` (removed) + `deny.toml` (updated `[graph].targets`), ~13 Rust source/test files (surgical Windows-target removal & OS-neutral `crates/iv-core/src/i18n.rs` wording), 1 removed build script (`iv-app/build.rs`), 1 removed directory (`packaging/windows/`) + 2 removed PowerShell scripts, 4 updated packaging/verify files, 2 updated workflows + 1 issue form, `.specify/memory/constitution.md`, 8 bilingual documentation pairs (`16` files), 1 Ruleset update (`id: 27903505`), optional repo topic audit, and 1 draft release deletion/regeneration (`v1.1.0-alpha.2`)

## Constitution Check

*GATE: Evaluated against Constitution **v3.0.0** (pre-Phase 0 and post-Phase 1 design).*

| Principle / Gate | Pre-Phase 0 Status | Post-Phase 1 Design Verification |
|---|---|---|
| **I. Korean Comm & Bilingual Root Docs (`*.md` / `*.en.md`)** | PASS | All 8 bilingual pairs (`README*`, `CHANGELOG*`, `CONTRIBUTING*`, `SECURITY*`, `NOTICE*`, `THIRD_PARTY_NOTICES*`, `docs/CAPABILITIES*`, `docs/FIRST_LAUNCH*`) and bilingual `.github/ISSUE_TEMPLATE/bug_report.yml` updated in lockstep (`quickstart.md` §1 #4). |
| **II. English-First Specs & Code** | PASS | All `specs/006-platform-tier-realignment/` design artifacts and all Rust/script identifiers & comments remain English. |
| **III. Test-Driven Development (NON-NEGOTIABLE)** | PASS | Removing Windows-target CLI/platform code (`Q4`) preserves/updates authentic unit & integration tests on active targets (`macOS`, `Linux`) and adds `render-release-notes.py` verification (`quickstart.md` §1 #5) with zero `#[ignore]` bypasses. |
| **IV. Zero Hardcoding, Mocking, or Test-Bypass (NON-NEGOTIABLE)** | PASS | Zero mocking, zero `deny.toml` ignores (`ignore = []`), zero test bypass flags. |
| **V. Evidence-Based Research & Verification (NON-NEGOTIABLE)** | PASS | `research.md` `R1`–`R7` cite exact observed file paths, line ranges, symbol names, `gh api` ruleset state, and release-record structures. |
| **VI. Skeptical Verification & Fallibility Awareness (NON-NEGOTIABLE)** | PASS | Prevents `Protect main` merge deadlock (`R1`), prevents false `iOS`/`Android` shipped claims (`Q1`, `R4`, `R5`), separates `macOS` CI vs. clean-device status (`R4`), and prohibits silent stale Windows drafts (`R6`). |
| **VII. Zero Trust Security & Untrusted Input (NON-NEGOTIABLE)** | PASS | Preserves 100% of OS-agnostic Zero-Trust guards (`normalize_archive_entry_key` `\` Zip-Slip, `PathRedactor` UNC/Windows/Unix redaction, `format_guard`, caps) per `Q5`, plus `005` SHA pinning & `env.TAG` injection prevention (`quickstart.md` §1 #3). |
| **VIII. License Compliance, Attribution & Stewardship (NON-NEGOTIABLE)** | PASS | Preserves `edith2580` `v1.0.6` copyright, `GPL-3.0-only`, `legacy-v1.0.6` archive tag, and `2026-11-24` GPLv3 §5(a) modification dates in `NOTICE*`, `README*`, and `CHANGELOG*`, and keeps `THIRD_PARTY_NOTICES*` accurate. |
| **Target Platforms & Quality Gates (`v3.0.0`)** | PASS | Concentrates build-result verification exclusively on `macOS` (`iOS` planned `007`), switches `Linux` to build-only (`Checks (ubuntu-latest)` compile gate + unverified tarball, zero launch jobs), removes all `Windows` builds/deps/checks, and preserves default + `--all-features` CI tests. |

**Gate Outcome**: **PASS** (zero constitutional violations; Complexity Tracking not triggered).

## Project Structure

### Documentation (this feature)

```text
specs/006-platform-tier-realignment/
├── spec.md                                        # Clarified feature spec (Clarifications Q1–Q5)
├── plan.md                                        # This file (/speckit-plan output)
├── research.md                                    # Phase 0 output: R1–R7 empirical research & decisions
├── data-model.md                                  # Phase 1 output: 5 governance/tier entities & transitions
├── quickstart.md                                  # Phase 1 output: local verification, bootstrap order & audits
├── checklists/
│   └── requirements.md                            # Spec quality checklist (16/16 passing)
└── contracts/
    └── platform-realignment-contract.md           # Phase 1 output: Ruleset, CI/release YAML, notes, OS form, draft CLI
```

### Repository Touchpoints (Remove / Update Map)

```text
.specify/memory/constitution.md                    # UPDATE: strip temporary Sync Impact Report comment (v3.0.0 ready to commit)
.cargo/config.toml                                 # REMOVE: Windows-only MSVC static CRT build flags ([target.x86_64-pc-windows-msvc])

.github/
├── ISSUE_TEMPLATE/bug_report.yml                  # UPDATE: replace Windows OS dropdown with macOS, Linux (build-only), planned iOS/Android
└── workflows/
    ├── ci.yml                                     # UPDATE: drop windows-latest, launch-windows, launch-linux; keep checks(ubuntu/macos), launch-macos, dependencies
    └── release.yml                                # UPDATE: drop build-windows, verify-windows, verify-linux; publish-draft depends on verify-macos + build-linux + deny

Cargo.toml                                         # UPDATE: remove winresource workspace dependency
Cargo.lock                                         # UPDATE: prune direct workspace windows/winreg/winresource entries cleanly (upstream target-conditional metadata entries remain unused on macOS/Linux)
deny.toml                                          # UPDATE: remove excluded "x86_64-pc-windows-msvc" from [graph].targets
crates/
├── iv-app/
│   ├── Cargo.toml                                 # UPDATE: remove [target.'cfg(windows)'.build-dependencies]
│   ├── build.rs                                   # REMOVE: Windows-only resource/manifest embedding script
│   └── src/
│       ├── main.rs                                # UPDATE: remove windows_subsystem attr & cfg!(target_os = "windows")
│       └── fonts.rs                               # UPDATE: remove C:\Windows\Fonts\malgun.ttf candidate
├── iv-core/
│   ├── src/
│   │   ├── cli.rs                                 # UPDATE: remove is_windows param, `/` switches, repair_explorer_quoted_drive (keep --register-shell fail-closed)
│   │   ├── codecs.rs                              # UPDATE: doc comment (remove WIC)
│   │   ├── i18n.rs                                # UPDATE: make error.unsupported_os_switch wording OS-neutral (remove Windows phrasing)
│   │   └── path_security.rs                       # UPDATE: remove Windows cfg in is_within_root ONLY; PRESERVE normalize_archive_entry_key (\) & PathRedactor
│   └── tests/cli_tests.rs                         # UPDATE: remove Windows Explorer drive quote test & is_windows arg (PRESERVE diagnostics_tests & path_security_tests)
└── iv-platform/
    ├── Cargo.toml                                 # UPDATE: remove windows/winreg deps and dep:windows; keep shell-registration = [] (preserving #[cfg(feature = "shell-registration")] in lib.rs and Linux XDG tests under clippy -D warnings)
    ├── src/
    │   ├── codecs.rs                              # UPDATE: remove Windows WIC implementation & comments
    │   ├── fatal_dialog.rs                        # UPDATE: remove "windows" log folder match arm
    │   ├── integration.rs                         # UPDATE: remove format/register/unregister_windows_shell helpers
    │   ├── paths_dialogs.rs                       # UPDATE: remove Windows LOCALAPPDATA branch
    │   └── update.rs                              # UPDATE: remove Windows cmd /C start & .exe/.zip/.msi selection
    └── tests/
        ├── os_integration_tests.rs                # UPDATE: remove format_windows_open_command & "windows" case
        └── update_tests.rs                        # UPDATE: remove "windows" asset selection test

packaging/
├── windows/                                       # REMOVE DIRECTORY (ImageViewer.ico, ImageViewer.manifest, ImageViewer.nsi, package-windows.ps1)
├── linux/package-linux.sh                         # UPDATE: label lower-priority build-only, unverified
├── release-notes.md.in                            # UPDATE: macOS verified + iOS planned(007) + Linux build-only + Android planned(007); no Windows
└── verify/
    ├── assert-windows-exe.ps1                     # REMOVE
    ├── launch-check.ps1                           # REMOVE
    ├── component-count.sh                         # UPDATE: 3 active targets (aarch64/x86_64-apple-darwin, x86_64-unknown-linux-gnu)
    ├── launch-check.sh                            # UPDATE: comment/scope macOS-only
    └── render-release-notes.py                    # UPDATE: drop WIN_AUTO, LINUX_AUTO, @WIN10@/@WIN11@

README.md, README.en.md                            # UPDATE: lockstep v3.0.0 platform tiers & legacy-v1.0.6 attribution
docs/CAPABILITIES.md, docs/CAPABILITIES.en.md      # UPDATE: lockstep macOS Core/Extended + Linux build-only + iOS/Android planned(007)
docs/FIRST_LAUNCH.md, docs/FIRST_LAUNCH.en.md      # UPDATE: lockstep remove active Windows guide; macOS + Linux build-only + mobile 007 note
CONTRIBUTING.md, CONTRIBUTING.en.md                # UPDATE: lockstep DoD & release checklist (macOS only; Linux build-only)
NOTICE, NOTICE.en.md                               # UPDATE: lockstep preserve 2026-11-16/17 + add 2026-11-24 platform realignment notice
THIRD_PARTY_NOTICES.md, THIRD_PARTY_NOTICES.en.md  # UPDATE: lockstep update 3-target component count & remove windows/winreg rows
CHANGELOG.md, CHANGELOG.en.md                      # UPDATE: lockstep 1.1.0-alpha.2 Constitution v3.0.0 alignment notes
```

**Structure Decision**: Existing 4-crate Cargo architecture (`iv-core`, `iv-media`, `iv-platform`, `iv-app`) stays intact; edits remove strictly Windows-target build/runtime files (`packaging/windows/`, `iv-app/build.rs`, `windows`/`winreg`/`winresource`, `#[cfg(target_os = "windows")]` platform adapters) while preserving OS-agnostic Zero-Trust security modules and updating CI/release workflows, Ruleset `Protect main`, draft `v1.1.0-alpha.2`, and bilingual documentation in lockstep.

## Complexity Tracking

No Constitution Check violations or exceptions required.
