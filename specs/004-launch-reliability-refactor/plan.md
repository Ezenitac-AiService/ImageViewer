# Implementation Plan: Launch Reliability & Evidence-Driven Refactoring

**Branch**: `004-launch-reliability-refactor` (spec directory; work currently sits on `main`, so create
the branch before implementation) | **Date**: 2026-11-16 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/004-launch-reliability-refactor/spec.md`

## Summary

`v1.1.0-alpha.1` fails for users on both supported OS families:

- **macOS**: the bundle fails the OS's strict signature check and is `arm64` only.
- **Windows** (user report): a console window opens with an unresponsive main window, and closing
  the console kills the app.

The approach:

1. Replace the hand-written winit/wgpu shell with **eframe 0.37**. This gives correct
   repaint-on-input, native window decorations, and a `wgpu` → `glow` renderer choice made by a
   pre-flight probe.
2. Make Windows builds **GUI-subsystem and static-CRT**.
3. Make macOS builds **universal and bundle-signed** (ad-hoc).
4. Make every launch **write a diagnostic record**, and fail loudly with a native dialog.
5. Move interaction rules into a **pure, tested `iv-core::interaction`** module.
6. Narrow the shipped product to **Core capabilities** with Cargo features, so withdrawn code stays
   tested but unshipped.
7. Replace extension-based decoding with a **content allowlist**.
8. Gate releases on an **automated launch check** of the real artifacts plus a **manual
   clean-machine check** enforced through draft releases.

Details and evidence: [research.md](./research.md).

## Technical Context

**Language/Version**: Rust stable; workspace `rust-version` raised `1.85 → 1.95` (required by eframe
0.37). The review host runs 1.100.0.

**Primary Dependencies**:

| Area | Packages |
|------|----------|
| Shell and rendering (new) | `eframe 0.37` with `wgpu` + `glow` features (brings `egui 0.37`, `egui-wgpu 0.37`, `wgpu 30`, `winit 0.30.13`) |
| Retained | `image 0.25` (Core codecs), `zune-jpeg`, `fast_image_resize`, `moxcms`, `kamadak-exif`, `zip`, `notify`, `rfd`, `directories`, `fs4`, `serde`, `serde_json`, `thiserror` |
| New | `sys-locale 0.3`; `winresource` (build-only, Windows); `objc2-app-kit` delegate (already present) |
| Behind non-default features | `reqwest`, `rustls`, `sha2` (`updater`); `windows`, `objc2-image-io`, `objc2-core-graphics` (`os-codecs`); long-tail decoders (`extended-formats`) |
| Removed | `tokio` (app and platform), `winreg`, direct `arboard`, direct `winit` / `wgpu` / `egui-winit` / `egui-wgpu` pins in `iv-app` |

**Storage**: `settings.json` (≤ 64 KB, v1.0.6 schema, unchanged location); daily diagnostic logs (7-day
retention).

**Testing**: `cargo test --workspace` (default = shipped features) and `--all-features`;
`cargo deny check` (all four sections); external launch-check scripts against packaged artifacts
(`packaging/verify/`); a manual clean-machine checklist ([quickstart.md](./quickstart.md) §4).

**Target Platform**: Supported: Windows 10 / 11 x64, macOS 11+ universal (Apple Silicon and Intel).
Experimental: Linux x86_64 (X11 / Wayland). Tiers per constitution v2.3.0.

**Project Type**: Desktop application (4-crate Cargo workspace).

**Performance Goals**: Controls respond within 100 ms (SC-004). A 10,000-file folder shows its first
image within 1 s (SC-006). The v1.0.6 numeric parity targets remain goals and do not gate this
release (spec Assumptions).

**Constraints**:

- No network access in the shipped build (FR-025).
- Shipped components ≤ 356, from a baseline of 446 measured with the research R13 metric.
- No test-only production flags (Principle IV).
- `unsafe` only in the macOS delegate and any remaining FFI, each with `// SAFETY:` comments.

**Scale/Scope**:

| Crate | Change |
|-------|--------|
| `iv-app` | Rewritten; about 2,600 lines replaced by eframe app, views, loader, and bootstrap |
| `iv-core` | Gains `interaction`, `diagnostics`, `FormatGuard::identify`, and settings `LoadOutcome` |
| `iv-media` | Feature-gated formats, lazy and async folder scan, watcher that keeps position |
| `iv-platform` | Feature-gated `updater` and `os-codecs`; macOS open-files delegate; locale; fatal dialog; `winit` dependency removed |
| `packaging/`, `.github/workflows/`, `docs/` | Rewritten |

**Open items carried as spikes** (research): S0 reproduce on Windows; S1 WARP / CPU adapter; S2 macOS
delegate coexistence; S3 `quit` Apple Event and hosted-runner desktop. No `NEEDS CLARIFICATION`
remains. Spike outcomes can change task details but not the design direction.

## Constitution Check

*GATE: checked before Phase 0 and re-checked after Phase 1 design against constitution v2.3.0;
re-checked after the v2.4.0 amendment (2026-11-16).*

| Principle / Gate | Pre-research | Post-design | Evidence |
|------------------|--------------|-------------|----------|
| I. Korean communication; root docs ko + en | PASS | PASS | New user docs `docs/FIRST_LAUNCH.md` / `.en.md` and `docs/CAPABILITIES.md` / `.en.md` are paired; README / THIRD_PARTY_NOTICES are updated in both languages; commits in Korean |
| II. English specs and code | PASS | PASS | All `specs/004-*` artifacts and planned identifiers are English |
| III. TDD (NON-NEGOTIABLE) | PASS | PASS | Contracts list required tests per row; tasks will order tests first; interaction logic is moved out of the event handler so it can be tested (R14) |
| IV. No hardcoding, mocks, or test bypass | PASS | PASS | Launch check uses OS close requests and production diagnostics, with no `--smoke-test` flag (R12). Cargo features are product scope, not bypasses (R9). Removes the stub `install_macos_open_url_handler`. New `unsafe` is confined to the R7 delegate with `// SAFETY:` comments |
| V. Evidence-based research | PASS | PASS | research.md tags each claim `[OBSERVED]` / `[SOURCE]` / `[UNVERIFIED]` with citations; unverified items become spikes |
| VI. Skeptical verification | PASS | PASS | Two-layer launch verification; release-notes platform table; spikes S0–S3; Intel macOS disclosed as unverified when not checked |
| VII. Zero Trust input | PARTIAL (current code) | PASS | Content allowlist plus pre-read size cap (R8, input-acceptance contract) closes findings D1 and D2; ZIP caps re-verified; path redaction extended to launch records |
| VIII. License and attribution | PASS | PASS | `deny.toml` keeps the license allowlist and adds advisories, bans, and sources; THIRD_PARTY_NOTICES gains justification columns; eframe, glow, and sys-locale are MIT/Apache-2.0 |
| Tech Stack: Target Platforms (tiered) | PASS | PASS | Supported = Windows + macOS universal; Linux experimental and non-blocking |
| Tech Stack: Media Decoding Strategy (capability tiers, v2.4.0) | DEVIATION (v2.3.0) | **PASS (v2.4.0)** | The shipped build carries Core formats; extended formats and OS codecs sit behind default-off features and are tested with `--all-features` (R9). Ratified by the v2.4.0 amendment |
| Tech Stack: Safety Caps & Performance Parity Baseline (v2.4.0) | DEVIATION (v2.3.0) | **PASS (v2.4.0)** | All safety caps retained. Numeric parity is a target and not release-gating until a harness exists; release notes state "performance parity unmeasured". Interim floor: SC-004, SC-006 |
| Gate: Capability Tier (v2.4.0) | — | PASS | Capability register (data-model §4) plus CI testing of default and all-features configurations |
| Gate: Performance Regression (v2.4.0) | — | N/A (inactive) | No harness yet; release-notes template carries the "unmeasured" statement |
| Gate: cross-platform build / lint / test | PASS | PASS | CI runs fmt, clippy, and tests with both feature sets on 3 OSes |
| Gate: launch verification (v2.3.0) | FAIL (current) | PASS | release-verification contract §2–§4 |
| Gate: language and docs | PASS | PASS | — |
| Gate: license and attribution | PASS | PASS | `cargo deny check` in CI and release |

**Result**: All gates pass under constitution v2.4.0. The two Technical Stack deviations recorded
during planning were ratified by the v2.4.0 amendment (2026-11-16) and no longer appear in
Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/004-launch-reliability-refactor/
├── feasibility-review.md   # Adversarial multi-persona review (/speckit-specify input)
├── spec.md                 # Feature spec (clarified 2026-11-16)
├── plan.md                 # This file
├── research.md             # Phase 0: decisions R1–R15, spikes S0–S3
├── data-model.md           # Phase 1: entities and runtime models
├── quickstart.md           # Phase 1: validation guide and clean-machine checklist
├── contracts/
│   ├── launch-and-diagnostics-contract.md
│   ├── interaction-contract.md
│   ├── input-acceptance-contract.md
│   └── release-verification-contract.md
├── checklists/requirements.md
└── tasks.md                # Phase 2 (/speckit-tasks; not created here)
```

### Source Code (repository root)

```text
.cargo/config.toml                   # NEW: static CRT for x86_64-pc-windows-msvc
Cargo.toml                           # rust-version 1.95; eframe/sys-locale added; tokio/winreg dropped from shipped graph
deny.toml                            # + [advisories] [bans] [sources]

crates/iv-core/src/
├── interaction.rs                   # NEW: InputEvent, InteractionContext, ViewerCommand, ViewerModel, map_input
├── diagnostics.rs                   # NEW: launch-record formatting (grammar in launch contract §3)
├── format_guard.rs                  # + identify() Core allowlist
├── settings.rs                      # + LoadOutcome; language detection only when key absent
├── i18n.rs                          # + fatal.*, settings.*, error.unsupported_format; Settings labels
└── cli.rs                           # register/unregister switches → unsupported in shipped build
crates/iv-core/tests/                # + interaction_tests.rs, diagnostics_tests.rs, identify_tests.rs

crates/iv-media/
├── Cargo.toml                       # features: default (core), extended-formats
└── src/
    ├── decode.rs                    # dispatch by CoreFormat; extended decoders behind cfg(feature)
    └── sources/{folder,watcher}.rs  # pre-read cap; async scan; position-keeping rescan

crates/iv-platform/
├── Cargo.toml                       # features: updater, os-codecs (off); winit/tokio/winreg removed
└── src/
    ├── macos_open_files.rs          # NEW: NSApplicationDelegate application:openURLs: → callback
    ├── locale.rs                    # NEW: sys-locale wrapper
    ├── fatal_dialog.rs              # NEW: rfd MessageDialog with redacted log folder
    ├── paths_dialogs.rs             # async pickers via pollster; AppUserEvent removed
    ├── update.rs                    # cfg(feature = "updater")
    └── codecs.rs                    # cfg(feature = "os-codecs")

crates/iv-app/
├── build.rs                         # NEW: winresource icon, version, manifest (Windows only)
└── src/
    ├── main.rs                      # REWRITE: windows_subsystem, panic hook, logger, probe, run_native
    ├── probe.rs                     # NEW: renderer/adapter selection (pure selection fn + wgpu enumeration)
    ├── app.rs                       # NEW: eframe::App; egui input → interaction::map_input → apply
    ├── canvas.rs                    # NEW: textures/tiles painted with rotate/flip/zoom/pan
    ├── loader.rs                    # NEW: generation-tagged decode/prefetch worker, ctx.request_repaint
    ├── views/{toolbar,settings,about,image_list,status}.rs  # egui::Window dialogs
    └── renderer.rs                  # DELETED

packaging/
├── macos/{package-macos.sh,Info.plist.in}   # universal + icns + bundle codesign + verify (+x)
├── windows/package-windows.ps1              # used by CI; PE/import assertions
├── linux/package-linux.sh                   # tarball, labelled experimental (+x)
├── verify/{launch-check.sh,launch-check.ps1,window-probe.swift,reference.png,fixtures/}
└── release-notes.md.in                      # platform table template

.github/workflows/{ci.yml,release.yml}       # launch-check jobs; deny all sections; draft-only publish
docs/{FIRST_LAUNCH,CAPABILITIES}{.md,.en.md} # NEW bilingual user docs
README.md / README.en.md / SECURITY*.md / THIRD_PARTY_NOTICES*.md  # claims aligned to capability register
```

**Structure Decision**: Keep the 4-crate workspace. `iv-app` becomes a thin eframe shell over
testable `iv-core` logic. `iv-platform` stops depending on the GUI framework (finding C9). Withdrawn
capabilities stay in their crates behind non-default features rather than moving to new crates, to
minimise churn.

## Delivery Order (input to `/speckit-tasks`)

1. **Spikes S0–S3** (time-boxed). Record outcomes in research.md.
2. **Stop the bleeding**, shippable on its own. Do User Stories 1, 2 and the Windows half of
   User Story 3 first:
   - R2 (subsystem, CRT, resources)
   - R6 (macOS universal and signing)
   - R5 (diagnostics and fatal dialog)
   - R12 (automated launch check)
3. **Shell migration**: R1, R3, R4, R14. Dialogs become `egui::Window`s.
4. **Open paths**: R7 (macOS open-with) and R10 (locale, settings outcome, async scan, watcher).
5. **Scope and audit**: R8, R9, R13, R15, plus docs and the capability register.

## Complexity Tracking

| Deviation | Why needed | Simpler alternative rejected because |
|-----------|-----------|--------------------------------------|
| Workspace `rust-version` 1.85 → 1.95 | Required by eframe / egui 0.37 | Staying on egui 0.31 keeps `block 0.1.6` future-incompat and the 21-month-old stack |
