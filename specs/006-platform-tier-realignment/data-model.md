# Data Model: Platform Tier Realignment (Constitution v3.0.0)

**Feature Branch**: `006-platform-tier-realignment`  
**Date**: 2026-11-24  
**Spec**: [spec.md](./spec.md) | **Research**: [research.md](./research.md)

This model defines the 5 entities, field invariants, validation rules, and state transitions governing `Ezenitac-AiService/ImageViewer` under Constitution `v3.0.0` and Clarifications `Q1`–`Q5`.

---

## 1. Platform Tier Policy

Maps every OS target to its constitutional governance tier, active `006` delivery scope vs. `007` roadmap, runtime verification mandate, and release-blocking behavior.

| Target | `tier` | `006_delivery_status` | `runtime_verification` | `release_blocking` | `SC / FR` |
|---|---|---|---|---|---|
| `macOS` (`aarch64`, `x86_64`) | `top-priority` | `active` (`ImageViewer-macos-universal.dmg`) | `mandatory` (automated CI + clean-machine) | `true` (build & verification failure blocks PR/release) | `FR-003`, `SC-002` |
| `iOS` (`aarch64`) | `top-priority` | `planned` (deferred to `007` per `Q1`) | `planned` (automated + clean-device in `007`; **never** claimed shipped/verified in `006`) | `n/a` in `006` (`true` once `007` ships) | `FR-003`, `FR-006` |
| `Linux` (`x86_64`) | `lower-priority-build-only` | `active` (`ImageViewer-linux-x64.tar.gz`) | `prohibited` (`0` runtime/launch verification jobs) | PR build/compile gate `true` (`Q3`); runtime `false`; release artifact build failure blocks archive publish | `FR-004`, `FR-007`, `SC-003` |
| `Android` (`aarch64`) | `lower-priority-build-only` | `planned` (deferred to `007` per `Q1`) | `prohibited` (`build-only` in `007`; `0` runtime verification) | `n/a` in `006` | `FR-004`, `FR-006` |
| `Windows` (`x64`) | `excluded` | `removed` (`0` active builds, deps, or packaging scripts; historical `legacy-v1.0.6` preserved) | `prohibited` | `false` (`0` checks/gates) | `FR-001`, `FR-002`, `FR-013`, `SC-001` |

---

## 2. Top-Priority Verification Record

Observable launch, window, first-frame, and clean-exit record restricted strictly to active top-priority targets.

| Field | Type / Format | Invariant / Constraint | Requirement |
|---|---|---|---|
| `release_file` | String | MUST be `ImageViewer.app` (`ImageViewer-macos-universal.dmg`) in `006`; never a Windows or Linux file | `FR-003`, `FR-006` |
| `target` | `{os, os_version, arch}` | `os == "darwin"`, `arch ∈ {arm64, x86_64}` in `006` | `FR-003` |
| `layer` | Enum (`automated` \| `clean-machine`) | `automated` on CI (`macos-latest`, `macos-15-intel`); `clean-machine` on Apple Silicon & Intel hardware | `FR-003`, `FR-011` |
| `window_shown` | Boolean | `true` within 30 s (`window-probe.swift`) | `FR-003` |
| `first_frame` | Boolean | `true` (`phase=first_frame source=<ref>` logged) | `FR-003` |
| `clean_exit` | Boolean | `true` (`exit_code == 0` and `exit=clean`) | `FR-003` |
| `result` | Enum (`pass` \| `fail`) | `pass` iff all required booleans hold (`navigated != false`) | `FR-003` |

---

## 3. Build-Only Artifact

Distributable package produced for an active lower-priority target (`Linux` in `006`, joined by `Android` in `007`) with zero runtime launch verification.

| Field | Type / Format | Invariant / Constraint | Requirement |
|---|---|---|---|
| `name` | String | `ImageViewer-linux-x64.tar.gz` in `006` | `FR-004`, `SC-003` |
| `tier` | Enum (`lower-priority-build-only`) | Fixed per Constitution `v3.0.0` | `FR-004` |
| `sha256` | 64-hex | Recorded in `SHA256SUMS.txt` | `FR-004` |
| `verification_records` | List | MUST be `[]` (empty — **0** launch-verification JSONs emitted or uploaded) | `FR-004`, `SC-003` |
| `disclosure_label` | String | Release notes & docs MUST state `build-only` and `unverified` (`n/a` automated / clean-machine) | `FR-006`, `SC-003` |

---

## 4. Branch Protection Gate Set

Exact ruleset configuration bound to `~DEFAULT_BRANCH` (`main`, `id: 27903505`) preventing merge deadlocks while enforcing Constitution `v3.0.0`.

| Field | Type / Format | Invariant / Constraint | Requirement |
|---|---|---|---|
| `name` | String | `"Protect main"` (`id: 27903505`) | `FR-007` |
| `bypass_actors` | List | `[]` (zero administrator bypass) | `FR-007`, `FR-012` |
| `strict_required_status_checks_policy` | Boolean | `true` | `FR-007` |
| `required_status_checks` | List of 4 contexts + `integration_id: 15368` | Exact 4 contexts: `Checks (ubuntu-latest)`, `Checks (macos-latest)`, `Launch check (macOS)`, `Dependencies (cargo-deny, budget)` | `FR-007`, `SC-004`, `Q3` |
| `excluded_contexts` | List | MUST omit retired `Checks (windows-latest)`, `Launch check (Windows)`, and eliminated `Launch check (Linux, experimental)` | `FR-001`, `FR-004`, `FR-007` |

**Deadlock-Safe Transition & Draft Regeneration Lifecycle**:
- `LegacyRuleset (6 checks incl. Windows)` → `[Maintainer Approval]` → `PUT rulesets/27903505 (4 checks: ubuntu, macos, Launch macOS, Dependencies)` → `[Open non-NNN-* PR #N removing Windows/Linux-launch]` → `PR #N passes 4 checks & merges to main` → `[Maintainer Approval]` → `Delete pre-amendment v1.1.0-alpha.2 draft` → `Workflow_dispatch release.yml (main, tag=v1.1.0-alpha.2)` → `Clean v3.0.0 Draft Ready (macOS verified + Linux build-only)`.

---

## 5. Bilingual Platform Documentation & Security Preservation Record

Tracks synchronized Korean/English documentation and preserved Zero-Trust input sanitization boundaries across active files.

| Field | Scope | Invariant / Constraint | Requirement |
|---|---|---|---|
| `bilingual_pairs` | `README*`, `CHANGELOG*`, `CONTRIBUTING*`, `NOTICE*`, `THIRD_PARTY_NOTICES*`, `SECURITY*`, `docs/CAPABILITIES*`, `docs/FIRST_LAUNCH*` | Lockstep `ko`/`en` `v3.0.0` tiers, working switchers, `legacy-v1.0.6` preserved, updated `THIRD_PARTY_NOTICES*` component count & no `winreg`/`windows` | `FR-008`, `FR-013`, `SC-005`, `I`/`VIII` |
| `issue_os_dropdown` | `.github/ISSUE_TEMPLATE/bug_report.yml` | Lists `macOS`, `Linux (build-only / unverified)`, `iOS (planned)`, `Android (planned build-only)`; omits `Windows` | `FR-009`, `SC-005` |
| `preserved_zero_trust_guards` | `iv-core::path_security` (`normalize_archive_entry_key`, `is_archive_symlink`, `PathRedactor`), `diagnostics_tests` | All Windows-formatted untrusted-input guards (`\` archive traversal, UNC/`C:\` redaction) remain active and tested (`Q5 Option A`) | `FR-013`, `SC-001`, Constitution `VII` |
| `constitution_state` | `.specify/memory/constitution.md` | Version `3.0.0`, temporary `Sync Impact Report` HTML comment removed prior to commit | `FR-012` |
