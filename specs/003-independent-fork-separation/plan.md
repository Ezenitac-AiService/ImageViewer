# Implementation Plan: Independent Project Separation, Licensing & Release Cutover

**Branch**: `003-independent-fork-separation` | **Date**: 2026-11-16 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/003-independent-fork-separation/spec.md`

## Summary

Separate `Ezenitac-AiService/ImageViewer` cleanly, legally, and courteously from `edith2580/ImageViewer` (`Q1=B`, `Q2=B`): harmonize all package manifests and `About` UI metadata to `GPL-3.0-only` with GPLv3 §5(a)/(d) modification notices (`NOTICE`, `CHANGELOG.md`, `THIRD_PARTY_NOTICES.md`, `deny.toml`), isolate all runtime update endpoints and OS bundle IDs to `Ezenitac-AiService/ImageViewer` (`com.ezenitac.imageviewer`) while preserving `ImageViewer` config continuity, update all major root/governance/spec documentation (`README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `.github/ISSUE_TEMPLATE/`, `.specify/memory/constitution.md`, `specs/001-*`, `specs/002-*`), archive the legacy C# baseline under tag `legacy-v1.0.6`, migrate `ImageViewer.nsi` into `packaging/windows/`, cut `main` over to the cross-platform Rust release pipeline, and complete GitHub Support fork detachment.

## Technical Context

**Language/Version**: Rust `edition = "2024"` (MSRV `1.85`, verified `1.100.0`), Markdown/YAML/NSIS/PowerShell/Bash
**Primary Dependencies**: Existing 4-crate Cargo workspace (`iv-core`, `iv-media`, `iv-platform`, `iv-app`), `cargo-deny` (`deny.toml`) for GPLv3 license compatibility checks
**Storage**: Repository files, `.github/workflows/`, `packaging/`, `specs/`
**Testing**: `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check licenses`, repository URL/license grep audits
**Target Platform**: Windows 10/11 (`x64`), macOS (`aarch64`/`x86_64`), Linux (`x86_64` X11/Wayland) + GitHub repository hosting (`Ezenitac-AiService/ImageViewer`)
**Project Type**: Cross-platform desktop application + repository governance/release cutover
**Performance Goals**: Zero regression to `002` `SC-001`–`SC-003` benchmarks; zero runtime `upstream` calls
**Constraints**: Preserve root `LICENSE` (`GPLv3`) and `edith2580` copyright notices; preserve `ImageViewer` `settings.json` directory path (`Q2=B`); keep `legacy-v1.0.6` git tag intact before removing `src/`/`tests/`
**Scale/Scope**: ~20 repository/code/packaging/doc files updated or created + legacy `src/`/`tests/` cutover + git tag/remote/GitHub Support detach

## Constitution Check

| Principle | Gate Verification |
|---|---|
| **I. Korean Communication & Commit Policy** | User-facing summaries, PR/commit messages, and clarification prompts in Korean. **PASS** |
| **II. English-First Reasoning & Implementation** | Code, manifests, `NOTICE`, `THIRD_PARTY_NOTICES.md`, `CONTRIBUTING.md`, `SECURITY.md`, `specs/` authored in English (`README.md` may provide clear English/Korean accessibility). **PASS** |
| **III. Test-Driven Development (NON-NEGOTIABLE)** | Update/identity unit tests (`update_tests.rs`, `os_integration_tests.rs`, `about_view`) updated and verified with `cargo test --workspace`. **PASS** |
| **IV. Zero Hardcoding, Mocking, or Test-Bypass Logic** | All update/URL checks test real `Ezenitac-AiService/ImageViewer` HTTPS endpoints without test-only bypasses. **PASS** |
| **V. Evidence-Based Research & Verification** | Grounded in GNU GPLv3 §5(a)/(d), `cargo-deny`/`cargo-about`, open-source fork etiquette, and direct repo grep audit (`research.md`). **PASS** |
| **VI. Skeptical Verification & Fallibility Awareness** | Explicit disclosure of GitHub Support manual step (cannot be executed purely via local git CLI) and `legacy-v1.0.6` tag verification before `src/` removal. **PASS** |
| **VII. Zero Trust Security & Untrusted Input Stance** | Update feed isolated strictly to `https://api.github.com/repos/Ezenitac-AiService/ImageViewer/releases/latest` + `SHA256SUMS.txt` + `SECURITY.md`. **PASS** |

## Project Structure

### Documentation (this feature)

```text
specs/003-independent-fork-separation/
├── spec.md                                            # Clarified requirements (FR-001..FR-010, SC-001..SC-004)
├── research.md                                        # Phase 0 GPLv3 §5, cargo-deny, GitHub detach & etiquette evidence
├── data-model.md                                      # License, Identity, Separation & Documentation Update Inventory
├── quickstart.md                                      # Automated audit, test & cutover verification commands
├── contracts/
│   └── separation-and-compliance-contract.md          # Binding rules for GPLv3, URLs, About UI, docs & release pipeline
├── checklists/
│   └── requirements.md                                # 16/16 passing
└── plan.md                                            # This file (/speckit-plan output)
```

### Target Repository Touch-Points

```text
Cargo.toml                                             # license = "GPL-3.0-only"
deny.toml                                              # cargo-deny GPLv3 license compatibility config
LICENSE                                                # Unmodified GNU GPLv3
NOTICE                                                 # Upstream edith2580 + Ezenitac-AiService copyright + GPLv3 §5(a) notice
CHANGELOG.md                                           # v1.0.6 -> 2026-11-16 independent Rust cross-platform cutover
THIRD_PARTY_NOTICES.md                                 # Bundled Rust frameworks & libraries copyright/license notices
README.md                                              # Cross-platform docs + courteous upstream banner
CONTRIBUTING.md                                        # Independent detached-fork contribution workflow
SECURITY.md                                            # Vulnerability reporting policy for Ezenitac-AiService/ImageViewer
.github/
├── ISSUE_TEMPLATE/
│   ├── bug_report.yml                                 # Fork vs upstream issue routing guard
│   └── feature_request.yml
└── workflows/
    ├── ci.yml                                         # Add cargo-deny license check
    └── release.yml                                    # Replace legacy .NET release with 3-OS Rust release + SHA256SUMS.txt
crates/
├── iv-platform/src/paths_dialogs.rs                   # ProjectDirs("com", "Ezenitac-AiService", "ImageViewer")
├── iv-platform/tests/update_tests.rs                  # Test Ezenitac-AiService/ImageViewer URLs
└── iv-app/src/
    ├── main.rs                                        # Ezenitac-AiService/ImageViewer update feed & releases URLs
    └── views/about_view.rs                            # GPLv3 §5(d), upstream origin, framework credits, repo URL
packaging/
├── linux/package-appimage.sh                          # Linux AppDir staging script used by release.yml
├── macos/
│   ├── Info.plist.in                                  # CFBundleIdentifier = com.ezenitac.imageviewer
│   └── package-dmg.sh                                 # macOS DMG packaging script used by release.yml
└── windows/
    ├── ImageViewer.ico                                # Migrated from src/ImageViewer/Resources/ImageViewer.ico
    ├── ImageViewer.nsi                                # Migrated from src/ImageViewer.Installer/ImageViewer.nsi
    └── package-windows.ps1                            # Updated NSI path & /DPUBLISH_DIR=dist
specs/
├── 001-architecture-security-refactoring/spec.md      # Archived legacy v1.0.6 banner
└── 002-rust-crossplatform-viewer/                     # Align any placeholder repo URLs
```

## Complexity Tracking

> No Constitution violations exist. All changes enforce GPLv3 legal compliance, Zero Trust update isolation, and clean repository governance.
