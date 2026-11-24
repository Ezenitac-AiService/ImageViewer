# Implementation Plan: Repository Governance Hardening & Post-Separation Operations

**Branch**: `005-repo-governance-hardening` | **Date**: 2026-11-23 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-repo-governance-hardening/spec.md`

## Summary

Harden repository governance, security intake boundaries, branch protection, supply-chain pinning, dependency monitoring, and release-handoff readiness for `Ezenitac-AiService/ImageViewer` following the comparative EasyViewer review and empirical repository inspection (`research.md` R1–R8). The implementation combines (1) repository file updates—adding `.github/ISSUE_TEMPLATE/config.yml`, adding a read-only weekly `.github/workflows/dependency-audit.yml` covering both shipped-default and `--all-features` dependency graphs, pinning all 7 external GitHub Actions across `ci.yml`/`release.yml`/`dependency-audit.yml` to verified 40-character commit SHAs, fixing 8 pre-existing `--all-features` RustSec advisories (`Cargo.toml` `image-extras` feature narrowing + `Cargo.lock` `url` update), and reconciling bilingual `NOTICE`/`NOTICE.en.md` with `004` reality—and (2) strictly ordered, maintainer-approved live GitHub mutations after `004-launch-reliability-refactor` passes `T109` and merges into `main`: enabling Private Vulnerability Reporting before/with Public Issues, trimming repository description and adding 8 topics, disabling unused Wiki/Projects, enabling `delete_branch_on_merge` and `sha_pinning_required`, activating the no-bypass `Protect main` ruleset against the 6 non-experimental post-`004` check contexts, and retiring obsolete remote branches (`002-rust-crossplatform-viewer` and merged `004-launch-reliability-refactor`).

## Technical Context

**Language/Version**: GitHub Actions YAML, GitHub Issue Form/Chooser YAML, Rust Workspace `1.95+` (`Edition 2024` for `Cargo.toml` / `Cargo.lock` dependency audit remediation), Bilingual Markdown (`ko`/`en`), GitHub REST API (`gh` CLI)

**Primary Dependencies**: `cargo-deny` (`EmbarkStudios/cargo-deny-action@17aeb3c839e42274e2b8adf721c8c65edfdc74dd`), `actions/checkout@11d5960a326750d5838078e36cf38b85af677262`, `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`, `actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093`, `dtolnay/rust-toolchain@39008827e6b4319a349e9a42dfc90579dd25f1ca`, `Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0`, `softprops/action-gh-release@3bb12739c298aeb8a4eeaf626c5b8d85266b0e65`; workspace crate `image-extras = { version = "0.1", default-features = false, features = ["pcx"] }` and `url >= 2.5.9` (via `reqwest`)

**Storage**: Version-controlled repository configuration files (`.github/`, `Cargo.toml`, `Cargo.lock`, root Markdown/notice files, `specs/`) and remote GitHub repository settings/rulesets (no runtime application storage added)

**Testing**: `cargo deny check advisories bans licenses sources`, `cargo deny --all-features check advisories bans licenses sources`, `cargo fmt --all -- --check`, `cargo check/clippy/test --workspace` (default and `--all-features`), YAML & 40-char SHA-pinning verification scripts, and read-only `gh api` / `gh repo view` / `gh search code` audits (`quickstart.md`)

**Target Platform**: GitHub repository hosting (`Ezenitac-AiService/ImageViewer`) and cross-platform CI runners (`ubuntu-latest`, `macos-latest`, `macos-15-intel`, `windows-latest`)

**Project Type**: Repository governance, CI/CD security hardening, dependency remediation, and bilingual documentation maintenance within a 4-crate Rust desktop workspace

**Performance Goals**: Zero additional macOS/Windows runner overhead on weekly audits (scheduled audit runs solely on `ubuntu-latest` for `shipped-default` and `all-features`), zero unnecessary transitive `ora`/`zip` compilation under `--all-features`

**Constraints**: Constitution v2.4.0 (`I`–`VIII`), explicit maintainer approval before every live remote repository mutation (`FR-013`), strictly read-only permissions (`contents: read`) for scheduled dependency audit (`FR-008`), non-blocking experimental status preserved for `Launch check (Linux, experimental)` (`FR-005`), GitHub public-repo 60-day schedule auto-pause handled via passive vulnerability alerts + manual re-enablement runbook (`SC-003`), bootstrap order (`004` merge before `Protect main` ruleset activation) to prevent check-name mismatch deadlocks

**Scale/Scope**: 1 repository (`Ezenitac-AiService/ImageViewer`), 3 workflow files (1 new + 2 SHA-pinned), 1 new issue chooser config, 1 `Cargo.toml`/`Cargo.lock` dependency update, 1 bilingual notice pair (`NOTICE`/`NOTICE.en.md`) + 1 bilingual security policy pair (`SECURITY.md`/`SECURITY.en.md`) + `specs/003` verification note check, 5 remote repository API endpoints, and 2 obsolete remote branches (`002-rust-crossplatform-viewer`, `004-launch-reliability-refactor` after merge)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / Gate | Pre-Phase 0 Status | Post-Phase 1 Design Verification |
|---|---|---|
| **I. Korean Comm & Bilingual Root Docs (`*.md` / `*.en.md`)** | PASS | `NOTICE` & `NOTICE.en.md` updated in lockstep with bidirectional switcher links; `.github/ISSUE_TEMPLATE/config.yml` uses bilingual (`ko`/`en`) chooser text; all `specs/005-*` artifacts excluded from `.en.md` pairing per Principle I. |
| **II. English-First Specs & Code** | PASS | All files under `specs/005-repo-governance-hardening/` are authored in English; `Cargo.toml` / workflow updates use English identifiers and comments. |
| **III. Test-Driven Development (NON-NEGOTIABLE)** | PASS | No new production Rust behavior added; `Cargo.toml` feature narrowing (`image-extras` `features = ["pcx"]`) and `url` lockfile update will be verified red-to-green (`cargo deny --all-features` currently fails with 8 advisories and must turn green alongside `cargo test --workspace --all-features`). |
| **IV. Zero Hardcoding, Mocking, or Test-Bypass (NON-NEGOTIABLE)** | PASS | Zero `deny.toml` advisory ignores added (`ignore = []` remains clean); zero test-bypass flags; pre-existing `--all-features` vulnerabilities eliminated cleanly at dependency/feature root instead of suppressing checks. |
| **V. Evidence-Based Research & Verification (NON-NEGOTIABLE)** | PASS | `research.md` R1–R8 cite exact observed outputs (`gh repo view`, `gh api`, `gh search code`, `gh run view 44020931940`, `image-extras-0.1.1/Cargo.toml`, `cargo deny --all-features` output, and GitHub Docs). |
| **VI. Skeptical Verification & Fallibility Awareness (NON-NEGOTIABLE)** | PASS | Identified and prevented 4 real failure modes during planning: disabled PVR while public issues open, 8 hidden `--all-features` RustSec advisories, GitHub 60-day schedule auto-pause falsifying unconditional `SC-003`, and ruleset check-name bootstrap deadlock prior to `004` merge. |
| **VII. Zero Trust Security & Untrusted Input (NON-NEGOTIABLE)** | PASS | Enforces strictly `contents: read` on scheduled audits, pins all external Actions to 40-char commit SHAs + `sha_pinning_required: true`, eliminates `zip 5.1.1` decompression/traversal advisories, disables blank issues & unused Wiki/Projects, and gates live mutations behind explicit maintainer approval (`FR-013`). |
| **VIII. License Compliance, Attribution & Independent Stewardship** | PASS | Preserves GPLv3 §5(a) `2026-11-16` fork-separation notice in `NOTICE`/`NOTICE.en.md`, adds truthful `2026-11-17` `004` modification notice, keeps `cargo deny` license/source gates green, and routes upstream inquiries to `edith2580/ImageViewer` via `config.yml`. |
| **Platform Tiering & Release Verification Gates** | PASS | `Protect main` requires exactly the 6 supported/core checks (`Checks (...)`, `Launch check (macOS)`, `Launch check (Windows)`, `Dependencies (cargo-deny, budget)`) and excludes `Launch check (Linux, experimental)` so Linux remains informational and non-blocking. |

**Gate Outcome**: **PASS** (zero unjustified violations; Complexity Tracking not triggered).

## Project Structure

### Documentation (this feature)

```text
specs/005-repo-governance-hardening/
├── plan.md                                  # This file (/speckit-plan output)
├── research.md                              # Phase 0 output: R1–R8 empirical evidence & decisions
├── data-model.md                            # Phase 1 output: 5 governance entities & state transitions
├── quickstart.md                            # Phase 1 output: local verification, live bootstrap order, & audit commands
├── checklists/
│   └── requirements.md                      # Specification quality checklist (16/16 passing)
└── contracts/
    └── repository-governance-contract.md    # Phase 1 output: config.yml, workflow YAML, SHA table, & API payloads
```

### Source Code & Repository Governance Paths (repository root)

```text
.github/
├── ISSUE_TEMPLATE/
│   ├── bug_report.yml                       # Existing bilingual bug form (fork-confirmation required)
│   ├── feature_request.yml                  # Existing bilingual feature form (fork-confirmation required)
│   └── config.yml                           # NEW: blank_issues_enabled: false + PVR & upstream contact_links
└── workflows/
    ├── ci.yml                               # UPDATE: pin all external actions to 40-char commit SHAs
    ├── release.yml                          # UPDATE: pin all external actions to 40-char commit SHAs, least-privilege permissions, and env:-mediated TAG handling (no shell injection)
    └── dependency-audit.yml                 # NEW: weekly + manual read-only cargo-deny (default & --all-features)

Cargo.toml                                   # UPDATE: narrow image-extras to default-features = false, features = ["pcx"]
Cargo.lock                                   # UPDATE: upgrade url >= 2.5.9 and drop transitive ora/zip 5.1.1

NOTICE                                       # UPDATE: preserve 2026-11-16 separation + add 2026-11-17 (004) summary
NOTICE.en.md                                 # UPDATE: synchronized English counterpart to NOTICE
SECURITY.md                                  # UPDATE: add direct clickable Private Vulnerability Reporting link (SC-001)
SECURITY.en.md                               # UPDATE: synchronized English counterpart to SECURITY.md
specs/003-independent-fork-separation/
└── tasks.md                                 # CHECK/RECORD (if needed): verify SC-003 code-search indexing evidence
```

**Structure Decision**: No Rust source modules (`crates/*/src/`) require behavioral changes. All repository-level files reside in `.github/`, workspace dependency metadata (`Cargo.toml` / `Cargo.lock`), and bilingual GPLv3 root notices (`NOTICE` / `NOTICE.en.md`), paired with maintainer-approved remote GitHub API configurations specified in `contracts/repository-governance-contract.md`.

## Complexity Tracking

No Constitution Check violations or exceptions required.
