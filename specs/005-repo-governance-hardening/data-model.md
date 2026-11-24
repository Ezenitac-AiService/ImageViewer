# Data Model: Repository Governance Hardening & Post-Separation Operations

**Feature Branch**: `005-repo-governance-hardening`  
**Date**: 2026-11-23  
**Spec**: [spec.md](./spec.md) | **Research**: [research.md](./research.md)

This model defines the 5 governance entities, field invariants, validation rules, and state transitions governing `Ezenitac-AiService/ImageViewer` post-separation operations (`FR-001`–`FR-014`).

---

## 1. Issue Intake Channel

Represents the repository's public issue reporting surface paired with its private vulnerability disclosure boundary.

| Field | Type / Format | Invariant / Constraint | Requirement |
|---|---|---|---|
| `private_vulnerability_reporting_enabled` | Boolean | MUST be `true` **before or simultaneously with** `has_issues_enabled = true` | `FR-003`, `SC-001` |
| `has_issues_enabled` | Boolean | MUST be `true` after maintainer approval | `FR-001`, `FR-013` |
| `blank_issues_enabled` | Boolean | MUST be `false` in `.github/ISSUE_TEMPLATE/config.yml` | `FR-002` |
| `templates` | List (`bug_report.yml`, `feature_request.yml`) | Bilingual (`ko`/`en`), MUST enforce `fork-confirmation` (`required: true`) | `FR-002` |
| `contact_links` | List (2 entries) | (1) Private Vulnerability Reporting (`.../security/advisories/new`); (2) Upstream `edith2580/ImageViewer/issues` | `FR-002`, `FR-003` |
| `security_policy_direct_link` | Boolean | `SECURITY.md` and `SECURITY.en.md` MUST include a direct clickable link to `https://github.com/Ezenitac-AiService/ImageViewer/security/advisories/new` in lockstep | `FR-003`, `SC-001`, Constitution I |

**State Transitions**:
- `DisabledBoth (current)` → `[Maintainer Approval]` → `EnablePVR` (`private_vulnerability_reporting = true`) → `EnablePublicIntake` (`has_issues = true` + `config.yml` deployed on default branch).
- Transition to `has_issues = true` while `private_vulnerability_reporting = false` is **INVALID** (`FR-003` violation).

---

## 2. Branch Protection Policy

Represents the Repository Ruleset guarding `main` (`~DEFAULT_BRANCH`).

| Field | Type / Format | Invariant / Constraint | Requirement |
|---|---|---|---|
| `name` | String | Unique human-readable ruleset name (e.g. `Protect main`) | `FR-004` |
| `target` | Enum (`branch`) | Applied to `~DEFAULT_BRANCH` (`main`) | `FR-004` |
| `enforcement` | Enum (`active`) | MUST be `active` **only after** `004-launch-reliability-refactor` lands on `main` | `FR-005` |
| `bypass_actors` | List | MUST be `[]` (empty — zero default administrator bypass) | `FR-004` |
| `deletion_blocked` | Boolean | Rule type `deletion` present | `FR-004` |
| `non_fast_forward_blocked` | Boolean | Rule type `non_fast_forward` present | `FR-004` |
| `required_approving_review_count` | Integer | MUST be `0` within active `pull_request` rule (PR required, solo self-merge allowed) | `FR-004` |
| `strict_required_status_checks_policy` | Boolean | MUST be `true` (reject stale branches on concurrent merges) | `FR-006` |
| `required_status_checks` | List of 6 exact contexts + `integration_id` | Each entry pins `"integration_id": 15368` (GitHub Actions) against spoofing across: `Checks (ubuntu-latest)`, `Checks (macos-latest)`, `Checks (windows-latest)`, `Launch check (macOS)`, `Launch check (Windows)`, `Dependencies (cargo-deny, budget)` | `FR-005`, `SC-002`, Constitution VII |
| `excluded_experimental_checks` | List of 1 exact context | `Launch check (Linux, experimental)` MUST NOT appear in `required_status_checks` | `FR-005`, Constitution v2.4.0 |

**Bootstrap State Machine (Deadlock Prevention)**:
- `PreBootstrap (legacy main checks)` → `[004 PR passes all 7 checks & merges to main]` → `MainUpdatedWithPost004Checks` → `[Maintainer Approval]` → `RulesetActive`.
- Activating `RulesetActive` while `main` still holds legacy `verify` / `licenses` job names is **INVALID** (`Protection Bootstrapping Check-Name Mismatch`).

---

## 3. Scheduled Dependency Audit

Represents the recurring read-only dependency, advisory, license, and source verification automation alongside passive platform alerts.

| Field | Type / Format | Invariant / Constraint | Requirement |
|---|---|---|---|
| `workflow_path` | Path | `.github/workflows/dependency-audit.yml` | `FR-007` |
| `triggers` | Set (`schedule`, `workflow_dispatch`) | Weekly cron (`17 3 * * 1`) + manual dispatch | `FR-007` |
| `permissions` | Map | MUST be `{ contents: read }` (strictly read-only; no `issues: write`) | `FR-008` |
| `action_references` | List | 100% pinned to immutable 40-char commit SHAs | `FR-014`, `SC-007` |
| `audit_scopes` | Set (`shipped-default`, `all-features`) | Runs both `cargo deny check advisories bans licenses sources` and `cargo deny --all-features check advisories bans licenses sources` | `FR-007` |
| `passive_vulnerability_alerts` | Boolean | MUST remain `true` (`HTTP 204`) to cover >60-day schedule dormancy | `FR-007`, `SC-003` |
| `failure_reporting` | Enum (`run_status_only`) | Surfaces failures via workflow run failure status & native notifications; zero automated issue writes | `FR-008` |

**Lifecycle States**:
- `ActiveSchedule` (repo activity within 60 days) → `[>= 60 days without repo activity]` → `PlatformPaused` (passive vulnerability alerts remain active) → `[Manual re-enable + workflow_dispatch before release]` → `ActiveSchedule`.

---

## 4. Repository Metadata Profile

Represents public discoverability, surface minimization, and supply-chain repository flags.

| Field | Type / Format | Invariant / Constraint | Requirement |
|---|---|---|---|
| `description` | String | Non-empty, `description == description.trim()` (0 leading/trailing whitespace) | `FR-009`, `SC-004` |
| `topics` | Set of Strings | `len(topics) >= 5`, lowercase alphanumeric/hyphen only (e.g., 8 curated tags) | `FR-009`, `SC-004` |
| `has_wiki` | Boolean | MUST be `false` | `FR-009`, `SC-007` |
| `has_projects` | Boolean | MUST be `false` | `FR-009`, `SC-007` |
| `delete_branch_on_merge` | Boolean | MUST be `true` | `FR-011`, `SC-005` |
| `sha_pinning_required` | Boolean | MUST be `true` (after all workflow files are SHA-pinned) | `FR-014`, `SC-007` |
| `code_search_indexed` | Boolean | Verified `true` via live query (`gh search code`) | `FR-010`, `SC-004` |

---

## 5. Branch & Release Handoff Record

Represents consolidation of branches and synchronization of bilingual root governance documents prior to `v1.1.0-alpha.2`.

| Field | Type / Format | Invariant / Constraint | Requirement |
|---|---|---|---|
| `004_analysis_verified` | Boolean | `004` `T109` completed before merging `004-launch-reliability-refactor` | `FR-011` |
| `active_remote_branches` | Set of Strings | After consolidation & `005` completion, only `main` (and any unmerged active feature branch) remains; `002-rust-crossplatform-viewer` and merged `004-launch-reliability-refactor` MUST be deleted | `FR-011`, `SC-005` |
| `bilingual_doc_pairs` | Paired Files (`*.md`, `*.en.md`, `NOTICE`/`NOTICE.en.md`) | Every updated root doc keeps `ko`/`en` lockstep content, bidirectional top language switcher, matching dates (`2026-11-16` initial separation + `2026-11-17` `004` modification), and accurate `1.1.0-alpha.2` architecture/scope | `FR-012`, `SC-006`, Constitution I |
| `004_handoff_ready` | Boolean | `main` contains verified `004` + `005` governance baseline ready for `v1.1.0-alpha.2` draft release + `T110` clean-machine verification | `FR-011` |
