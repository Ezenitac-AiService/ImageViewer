---

description: "Actionable dependency-ordered task list for repository governance hardening and post-separation operations"
---

# Tasks: Repository Governance Hardening & Post-Separation Operations

**Input**: Design documents from `/specs/005-repo-governance-hardening/`  
**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/repository-governance-contract.md](./contracts/repository-governance-contract.md), [quickstart.md](./quickstart.md)

**Tests**: Verification and negative/contract check tasks precede implementation or remote configuration mutations per Constitution Principles III, V, VI, and VII. Every live remote repository setting change, branch-protection ruleset activation, default-branch merge, or remote branch deletion MUST receive explicit maintainer approval (`FR-013`).

**Organization**: Tasks are ordered monotonically (`T001`–`T032`) with zero forward dependencies so sequential execution succeeds top-to-bottom, grouped by user story (`US2` P1 protection bootstrap → `US1` P1 intake → `US3` P2 monitoring → `US4` P2 hygiene/handoff).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files or read-only checks, no incomplete dependencies)
- **[Story]**: User story mapping (`[US1]`, `[US2]`, `[US3]`, `[US4]`) — omitted on Setup, Foundational, and Polish phases
- Every task includes exact repository file paths or design contract paths

## Path Conventions

- Repository governance & workflows: `.github/ISSUE_TEMPLATE/`, `.github/workflows/`
- Workspace manifests & notices: `Cargo.toml`, `Cargo.lock`, `NOTICE`, `NOTICE.en.md`, `CHANGELOG.md`, `CHANGELOG.en.md`, `CONTRIBUTING.md`, `CONTRIBUTING.en.md`
- Feature specifications: `specs/003-independent-fork-separation/`, `specs/004-launch-reliability-refactor/`, `specs/005-repo-governance-hardening/`

---

## Phase 1: Setup (Baseline & Staging Isolation)

**Purpose**: Establish strict staging isolation on `004-launch-reliability-refactor` and record read-only pre-change baselines before any edits or remote mutations.

- [X] T001 Confirm `004-launch-reliability-refactor` isolation (require subsequent `004` commits to stage ONLY `specs/004-launch-reliability-refactor/` and NEVER stage untracked `specs/005-repo-governance-hardening/` onto `004`), and verify `.specify/feature.json` targets `specs/005-repo-governance-hardening`
- [X] T002 [P] Capture read-only live remote pre-change baseline via `gh repo view`, `gh api`, and `gh search code` per `specs/005-repo-governance-hardening/quickstart.md` §3 and store findings in `specs/005-repo-governance-hardening/research.md`
- [X] T003 [P] Capture failing red verification baseline (`cargo deny --all-features check advisories bans licenses sources` failing on 8 RustSec advisories; unpinned `uses:` tags in `.github/workflows/ci.yml` and `.github/workflows/release.yml`) per `specs/005-repo-governance-hardening/quickstart.md` §1

---

## Phase 2: Foundational (Blocking Prerequisites — `004` Landing & Supply-Chain Remediation)

**Purpose**: Land `004-launch-reliability-refactor` onto `main` first so post-`004` check names exist on the default branch, rebase `005-repo-governance-hardening` onto post-`004` `main`, fix the 8 pre-existing `--all-features` advisories, and apply SHA-pinning + least-privilege `permissions:` across existing CI/release workflows.

**⚠️ CRITICAL**: `Protect main` ruleset activation (`US2`) MUST wait until `T005` merges `004` into `main`; all `005` edits (`T007`–`T010`) MUST happen on `005-repo-governance-hardening` after `T006`.

- [X] T004 Complete `004` `T109` (`/speckit-analyze` consistency verification confirming `SC-010` 446 → ≤ 356 shipped-component metric and measured 298 count match, superseding the earlier 514/411 draft) across `specs/004-launch-reliability-refactor/tasks.md`, `specs/004-launch-reliability-refactor/spec.md`, and `specs/004-launch-reliability-refactor/research.md`, staging ONLY `specs/004-launch-reliability-refactor/` per `T001`
- [X] T005 After `T004` completes, request explicit maintainer approval (`FR-013`), open a pull request from `004-launch-reliability-refactor` to `main`, confirm all 7 CI checks pass, and merge into `main` so post-`004` workflow check names are established on the default branch (`FR-005`, `FR-011`, `research.md` R3)
- [X] T006 Create or rebase branch `005-repo-governance-hardening` cleanly from post-`T005` `main` (carrying `specs/005-repo-governance-hardening/` with zero divergence from `main`)
- [X] T007 On `005-repo-governance-hardening` (`T006`), narrow `image-extras` in `Cargo.toml` to `image-extras = { version = "0.1", default-features = false, features = ["pcx"] }` (eliminating unused `ora` → `zip 5.1.1`) and run `cargo update -p url` to upgrade `url >= 2.5.9` in `Cargo.lock` (`FR-007`, `research.md` R5)
- [X] T008 Verify green workspace builds, lints, tests, and dependency audits on host OS (`cargo fmt --all -- --check`, `cargo check/clippy/test --workspace` default and `--all-features`, `cargo deny check advisories bans licenses sources`, `cargo deny --all-features check advisories bans licenses sources`) against `Cargo.toml` and `Cargo.lock` (`T007`)
- [X] T009 [P] On `005-repo-governance-hardening` (`T006`), add explicit top-level `permissions: { contents: read }` and pin all external `uses:` references in `.github/workflows/ci.yml` to the exact verified 40-character commit SHAs (`11d5960a326750d5838078e36cf38b85af677262`, `ea165f8d65b6e75b540449e92b4886f43607fa02`, `39008827e6b4319a349e9a42dfc90579dd25f1ca`, `d1527c8f67ef9af357290475d4af4b0fab8f12e0`, `17aeb3c839e42274e2b8adf721c8c65edfdc74dd`) per `specs/005-repo-governance-hardening/contracts/repository-governance-contract.md` §3 (`FR-008`, `FR-014`)
- [X] T010 [P] On `005-repo-governance-hardening` (`T006`), narrow top-level `permissions: { contents: read }` with job-scoped `permissions: { contents: write }` on `publish-draft` only, and pin all external `uses:` references in `.github/workflows/release.yml` to the exact verified 40-character commit SHAs (`11d5960a326750d5838078e36cf38b85af677262`, `ea165f8d65b6e75b540449e92b4886f43607fa02`, `d3f86a106a0bac45b974a628896c90dbdf5c8093`, `39008827e6b4319a349e9a42dfc90579dd25f1ca`, `d1527c8f67ef9af357290475d4af4b0fab8f12e0`, `17aeb3c839e42274e2b8adf721c8c65edfdc74dd`, `3bb12739c298aeb8a4eeaf626c5b8d85266b0e65`) per `specs/005-repo-governance-hardening/contracts/repository-governance-contract.md` §3 (`FR-008`, `FR-014`)

**Checkpoint**: `main` carries `004` code and updated CI job names; `005-repo-governance-hardening` is rebased cleanly with advisory fixes and least-privilege SHA-pinned `ci.yml` / `release.yml`.

---

## Phase 3: User Story 2 - Protect Default Branch with Mandatory Verification Gates (Priority: P1)

**Goal**: Guard `main` against direct pushes, force-pushes, deletion, status spoofing (`integration_id: 15368`), and concurrent-merge check bypasses while keeping `Launch check (Linux, experimental)` non-blocking, then validate and merge the `005` foundational PR through the protected `main` gates.

**Independent Test**: Inspect `Protect main` ruleset and effective branch rules via `gh api`, confirm `integration_id: 15368` across all 6 required contexts (excluding experimental Linux), and merge the `005` foundational PR via `gh pr checks`.

- [X] T011 [US2] Confirm `T005` has merged `004-launch-reliability-refactor` into `main` (verifying `Checks (ubuntu-latest)`, `Checks (macos-latest)`, `Checks (windows-latest)`, `Launch check (macOS)`, `Launch check (Windows)`, and `Dependencies (cargo-deny, budget)` exist on `main` and `gh api repos/Ezenitac-AiService/ImageViewer/rulesets` is empty) (`FR-005`, `research.md` R3)
- [X] T012 [US2] Request explicit maintainer approval (`FR-013`) and create the `Protect main` Repository Ruleset (`POST repos/Ezenitac-AiService/ImageViewer/rulesets`) per `specs/005-repo-governance-hardening/contracts/repository-governance-contract.md` §4.5 with verbatim constraints (`target: "branch"`, `enforcement: "active"`, `conditions.ref_name.include: ["~DEFAULT_BRANCH"]`, `bypass_actors: []`, `deletion`, `non_fast_forward`, `pull_request` with `required_approving_review_count: 0`, `required_status_checks` with `strict_required_status_checks_policy: true` and exact 6 contexts each pinning `"integration_id": 15368`, omitting `Launch check (Linux, experimental)`) (`FR-004`–`FR-006`)
- [X] T013 [US2] Verify `SC-002` by auditing `gh api repos/Ezenitac-AiService/ImageViewer/rulesets` and `gh api repos/Ezenitac-AiService/ImageViewer/rules/branches/main` against all `Branch Protection Policy` field invariants (including `"integration_id": 15368` on all 6 checks and omission of `Launch check (Linux, experimental)`) in `specs/005-repo-governance-hardening/data-model.md` §2
- [X] T014 [US2] Open and merge (with explicit maintainer approval per `FR-013`) a pull request from `005-repo-governance-hardening` (`T007`–`T010` plus any pre-authored `005` files) against protected `main`, verifying via `gh pr checks` that the 6 mandatory checks gate mergeability while `Launch check (Linux, experimental)` stays non-blocking (`FR-004`, `SC-002`)

**Checkpoint**: `main` is protected with zero-bypass PR/status rules aligned to Constitution v2.4.0 platform tiers and carries clean SHA-pinned `ci.yml` / `release.yml` + advisory-free `Cargo.toml` / `Cargo.lock`.

---

## Phase 4: User Story 1 - Enable Public Issue Reporting & Fork-Etiquette Routing (Priority: P1) 🎯 MVP

**Goal**: Deploy `.github/ISSUE_TEMPLATE/config.yml` onto `main` first, enable GitHub Private Vulnerability Reporting, and then activate public issue intake with zero blank-issue exposure.

**Independent Test**: Visit the repository's issue entry point after `T016`–`T018`, confirm blank issues are blocked, open both bilingual templates (verifying `fork-confirmation` `required: true`), and confirm security/upstream chooser links resolve to active Private Vulnerability Reporting and `edith2580/ImageViewer/issues`.

- [X] T015 [P] [US1] Verify pre-change intake baseline confirming `.github/ISSUE_TEMPLATE/config.yml` is missing, `.github/ISSUE_TEMPLATE/bug_report.yml` and `.github/ISSUE_TEMPLATE/feature_request.yml` preserve `fork-confirmation` (`required: true`), and `private-vulnerability-reporting.enabled == false` / `hasIssuesEnabled == false` per `specs/005-repo-governance-hardening/quickstart.md` §1 and §3
- [X] T016 [US1] Create `.github/ISSUE_TEMPLATE/config.yml` with `blank_issues_enabled: false` and bilingual `contact_links` for Private Vulnerability Reporting (`https://github.com/Ezenitac-AiService/ImageViewer/security/advisories/new`) and upstream (`https://github.com/edith2580/ImageViewer/issues`) per `specs/005-repo-governance-hardening/contracts/repository-governance-contract.md` §1, and land it onto `main` via PR (passes `Protect main` checks) BEFORE enabling public issues (`FR-002`, `FR-003`)
- [X] T017 [US1] Request explicit maintainer approval (`FR-013`) and enable GitHub Private Vulnerability Reporting (`private_vulnerability_reporting_enabled = true` **before or simultaneously with** `has_issues_enabled = true`) via `PUT repos/Ezenitac-AiService/ImageViewer/private-vulnerability-reporting` per `specs/005-repo-governance-hardening/contracts/repository-governance-contract.md` §4.1 (`FR-003`)
- [X] T018 [US1] After `T016` (`.github/ISSUE_TEMPLATE/config.yml` live on `main`) and `T017` (PVR active), request explicit maintainer approval (`FR-013`) and set `has_issues: true` via `PATCH repos/Ezenitac-AiService/ImageViewer` per `specs/005-repo-governance-hardening/contracts/repository-governance-contract.md` §4.2 (`FR-001`, `FR-002`)
- [X] T019 [US1] Verify `SC-001` via read-only remote audit after `T018` (`gh api .../private-vulnerability-reporting` returning `enabled: true`, `gh repo view --json hasIssuesEnabled` returning `true`, `.github/ISSUE_TEMPLATE/config.yml` live on `main` with `blank_issues_enabled: false`, and link verification in `README.md`, `README.en.md`, `SECURITY.md`, `SECURITY.en.md`) per `specs/005-repo-governance-hardening/quickstart.md` §3

**Checkpoint**: User Story 1 delivers active public issue intake and verified Private Vulnerability Reporting with zero blank-issue exposure or broken security routing.

---

## Phase 5: User Story 3 - Proactive Scheduled Dependency & Advisory Monitoring (Priority: P2)

**Goal**: Monitor both shipped-default and full `--all-features` dependency graphs weekly (plus manual dispatch) under strictly read-only permissions and enforce repository-wide commit-SHA pinning for external Actions.

**Independent Test**: Validate `.github/workflows/dependency-audit.yml` YAML and 40-char SHA pinning locally, merge it onto `main` via PR, activate `sha_pinning_required: true`, verify `vulnerability-alerts` returns HTTP `204`, and dispatch `gh workflow run dependency-audit.yml` cleanly.

- [X] T020 [P] [US3] Create `.github/workflows/dependency-audit.yml` per `specs/005-repo-governance-hardening/contracts/repository-governance-contract.md` §2 with `schedule` (`cron: '17 3 * * 1'`), `workflow_dispatch: {}`, `permissions: { contents: read }` (strictly read-only; zero `issues: write`), 40-char SHA-pinned `uses:` (`actions/checkout@11d5960a326750d5838078e36cf38b85af677262`, `EmbarkStudios/cargo-deny-action@17aeb3c839e42274e2b8adf721c8c65edfdc74dd`), and matrix passing `arguments: ""` (`shipped-default`) / `arguments: "--all-features"` (`all-features`) with fixed `command: check` and `command-arguments: advisories bans licenses sources` (`FR-007`, `FR-008`, `FR-014`)
- [X] T021 [US3] Run YAML parse and 40-character SHA-pinning verification scripts across `.github/workflows/ci.yml`, `.github/workflows/release.yml`, and `.github/workflows/dependency-audit.yml` per `specs/005-repo-governance-hardening/quickstart.md` §1, and merge `.github/workflows/dependency-audit.yml` onto `main` via PR (with explicit maintainer approval per `FR-013`) so 100% of default-branch workflows are SHA-pinned (`FR-014`, `SC-007`)
- [X] T022 [US3] After `T014` and `T021` have landed all SHA-pinned workflows on `main`, request explicit maintainer approval (`FR-013`) and enable `sha_pinning_required: true` via `PUT repos/Ezenitac-AiService/ImageViewer/actions/permissions` per `specs/005-repo-governance-hardening/contracts/repository-governance-contract.md` §4.4 (`FR-014`, `SC-007`)
- [X] T023 [US3] Verify `SC-003` and `SC-007`: confirm `gh api repos/Ezenitac-AiService/ImageViewer/vulnerability-alerts -i` returns `HTTP/2.0 204`, `gh api repos/Ezenitac-AiService/ImageViewer/actions/permissions` returns `sha_pinning_required: true`, and `gh workflow run dependency-audit.yml` passes both `Cargo Deny (shipped-default)` and `Cargo Deny (all-features)` matrix jobs (`specs/005-repo-governance-hardening/quickstart.md` §3)

**Checkpoint**: Weekly read-only dependency monitoring (`shipped-default` + `all-features`) and immutable Action SHA enforcement are active on `main`.

---

## Phase 6: User Story 4 - Repository Discoverability, Branch Hygiene & Release Handoff Readiness (Priority: P2)

**Goal**: Reconcile bilingual `NOTICE`/`NOTICE.en.md` and governance docs onto `main`, normalize repository metadata/topics, disable unused Wiki/Projects surfaces, enable automatic merged-branch deletion, retire obsolete remote branches, and hand off post-`005` `main` cleanly for `004` `T110`.

**Independent Test**: Audit `gh repo view`, `gh api .../branches`, `gh search code`, and `NOTICE`/`NOTICE.en.md` against `README`/`CHANGELOG`/`SECURITY` to verify trimmed description, >= 5 topics, disabled Wiki/Projects, 0 obsolete remote branches, and synchronized bilingual notices.

- [X] T024 [P] [US4] Update `NOTICE` preserving the `2026-11-16` initial fork-separation notice and `edith2580`/`Ezenitac-AiService` copyright, adding the `2026-11-17` (`004`) modification notice, and reconciling shipped architecture summary (`eframe`/`egui` + `wgpu`/`glow` fallback, `zune-jpeg`, `image`, `image-webp`, `tiff`, `moxcms`, `fast_image_resize`, `zip`, offline/no-auto-updater posture, extended formats including `jxl-oxide` excluded by default) with `README.md`, `CHANGELOG.md`, and `SECURITY.md` (`FR-012`, `SC-006`, `research.md` R8)
- [X] T025 [P] [US4] Update `NOTICE.en.md` in strict lockstep with `NOTICE`, preserving the top bidirectional language switcher (`[한국어](NOTICE) | **[English](NOTICE.en.md)**`), identical `2026-11-16` and `2026-11-17` dates, and matching capability/architecture summary (`FR-012`, `SC-006`, Constitution Principle I)
- [X] T026 [US4] Audit `CHANGELOG.md`, `CHANGELOG.en.md`, `CONTRIBUTING.md`, `CONTRIBUTING.en.md` (aligning local `--all-features` `cargo deny` guidance if needed), and `specs/003-independent-fork-separation/tasks.md` (`T022` / `SC-003`) to verify `1.1.0-alpha.2` status consistency and record verified standalone `gh search code` indexing evidence (60 results observed), then merge `T024`–`T026` onto `main` via PR (with explicit maintainer approval per `FR-013`) (`FR-010`, `FR-012`, `SC-004`, `SC-006`)
- [X] T027 [US4] Request explicit maintainer approval (`FR-013`) and apply `PATCH repos/Ezenitac-AiService/ImageViewer` (`description == description.trim()`, `has_wiki: false`, `has_projects: false`, `delete_branch_on_merge: true`), apply `PUT repos/Ezenitac-AiService/ImageViewer/topics` (`len(topics) >= 5`: `rust`, `image-viewer`, `comic-viewer`, `cbz`, `egui`, `wgpu`, `cross-platform`, `zero-trust`) per `specs/005-repo-governance-hardening/contracts/repository-governance-contract.md` §4.2–§4.3, and delete obsolete remote branches (`002-rust-crossplatform-viewer` and merged `004-launch-reliability-refactor` if still present) (`FR-009`, `FR-011`, `SC-004`, `SC-005`, `SC-007`)
- [X] T028 [US4] Verify `SC-004`, `SC-005`, `SC-006`, and `SC-007` via read-only remote checks (`gh repo view`, `gh api .../branches`, `gh search code`) and bilingual lockstep inspection of `NOTICE` / `NOTICE.en.md` and `CHANGELOG.md` / `CHANGELOG.en.md` per `specs/005-repo-governance-hardening/quickstart.md` §3
- [X] T029 [US4] Confirm post-`005` `main` release-handoff readiness for `specs/004-launch-reliability-refactor/tasks.md` `T110`, ensuring the `v1.1.0-alpha.2` tag and draft release are created strictly from a post-`T014` `main` commit (containing SHA-pinned `.github/workflows/release.yml`) so `sha_pinning_required: true` (`T022`) passes cleanly (`FR-011`, `FR-014`)

**Checkpoint**: Metadata, discovery topics, surface minimization, branch hygiene, and bilingual GPLv3 notices are synchronized and ready for `v1.1.0-alpha.2`.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Finalize feature status records, run end-to-end validation, and verify constitutional compliance.

- [X] T030 [P] Update `**Status**` in `specs/005-repo-governance-hardening/spec.md` (`Implemented`) and record final observed remote/CI verification evidence in `specs/005-repo-governance-hardening/research.md`
- [X] T031 Run complete validation suite in `specs/005-repo-governance-hardening/quickstart.md` §1 and §3 on macOS host after final file edits, reporting Windows/Linux CI verification targets explicitly per Constitution Build Gate
- [X] T032 Run `/speckit-analyze` across `specs/005-repo-governance-hardening/spec.md`, `specs/005-repo-governance-hardening/plan.md`, and `specs/005-repo-governance-hardening/tasks.md` to confirm zero cross-artifact inconsistencies and full Constitution v2.4.0 adherence

---

## Dependencies & Execution Order

### Strict Monotonic Order (Zero Forward Dependencies)

- **Phase 1 (`T001`–`T003`)**: Starts immediately; `T002` and `T003` run in parallel while `T001` isolates `004` staging.
- **Phase 2 (`T004`–`T010`)**: `T004` (`004` `T109`) → `T005` (`004` PR → `main`) → `T006` (`005` rebase onto post-`T005` `main`) → `T007` → `T008`; `T009` and `T010` run in parallel after `T006`.
- **Phase 3 (`US2`, `T011`–`T014`)**: `T011` → `T012` → `T013` → `T014` (merges `T007`–`T010` into protected `main`).
- **Phase 4 (`US1`, `T015`–`T019`)**: `T015` → `T016` (`config.yml` → `main` via PR) → `T017` (PVR) → `T018` (`has_issues: true`) → `T019` (`SC-001` check).
- **Phase 5 (`US3`, `T020`–`T023`)**: `T020` → `T021` (local check + merge `dependency-audit.yml` onto `main`) → `T022` (`sha_pinning_required: true`) → `T023` (`SC-003` + `SC-007` check).
- **Phase 6 (`US4`, `T024`–`T029`)**: `T024` and `T025` run in parallel → `T026` (doc audit + merge onto `main`) → `T027` (metadata/topics/branch cleanup) → `T028` → `T029` (post-`005` `main` handoff to `004` `T110`).
- **Phase 7 (Polish, `T030`–`T032`)**: Depends on Phases 1–6 completion (`T030` → `T031` → `T032`).

### Parallel & Batching Opportunities

- **Setup**: `T002` (remote baseline) and `T003` (local `cargo deny --all-features` / SHA baseline) in parallel.
- **Foundational Workflows**: `T009` (`ci.yml`) and `T010` (`release.yml`) touch disjoint files and run in parallel after `T006` (`T007` `Cargo.toml`/`Cargo.lock` can run alongside them).
- **Bilingual Notice Pair**: `T024` (`NOTICE`) and `T025` (`NOTICE.en.md`) run in parallel.
- **Optional PR Batching**: During execution, the maintainer may optionally batch file edits (`T016`, `T020`, `T024`–`T026`) into the `T014` PR once `T012` (`Protect main`) is active, provided `T018` (`has_issues: true`), `T022` (`sha_pinning_required: true`), and `T029` (`v1.1.0-alpha.2` tag) still wait until their prerequisite files are live on `main`.

---

## Parallel Example: Foundational Workflow & Manifest Hardening

```bash
# After T006 rebases 005-repo-governance-hardening onto post-004 main:
Task: "T007 Narrow image-extras in Cargo.toml and upgrade url in Cargo.lock"
Task: "T009 Harden permissions and pin actions in .github/workflows/ci.yml"
Task: "T010 Harden permissions and pin actions in .github/workflows/release.yml"
```

---

## Implementation Strategy

### MVP First (Safe `main` Bootstrap in US2 + Public/Private Intake in US1)

1. Complete Phase 1 (`T001`–`T003`) and Phase 2 (`T004`–`T010`) to land `004` on `main`, rebase `005`, eliminate all `--all-features` advisories, and prepare least-privilege SHA-pinned CI/release workflows.
2. Complete Phase 3 (`US2`, `T011`–`T014`) to activate `Protect main` (`integration_id: 15368`) and merge the `005` foundational PR through the protected gates.
3. Complete Phase 4 (`US1` MVP, `T015`–`T019`) to land `.github/ISSUE_TEMPLATE/config.yml`, enable Private Vulnerability Reporting, and activate public issues with zero blank-issue exposure.

### Incremental Delivery

1. **Increment 1 (`T001`–`T014`)**: `004` landed, `Protect main` active with `"integration_id": 15368`, and least-privilege SHA-pinned `ci.yml` / `release.yml` on `main` (`US2` + Foundation).
2. **Increment 2 (`T015`–`T019`)**: Active Private Vulnerability Reporting + Public Issues with `config.yml` live (`US1` MVP).
3. **Increment 3 (`T020`–`T023`)**: Weekly read-only dependency audit + repository-wide `sha_pinning_required: true` (`US3`).
4. **Increment 4 (`T024`–`T032`)**: Bilingual `NOTICE` reconciliation, clean metadata/topics, branch cleanup, post-`005` `main` handoff for `004` `T110` (`US4`), and final Polish validation.

### Notes

- Every live remote mutation (`T005`, `T012`, `T014`, `T016`–`T018`, `T021`–`T022`, `T026`–`T027`) MUST fail closed if maintainer approval or administrative permissions are absent (`FR-013`).
- Commit messages MUST be written in Korean per Constitution Principle I; specs, workflow comments, and task statuses remain in English per Constitution Principle II.
- `T007` (`Cargo.toml` / `Cargo.lock`) resolves real failing advisories rather than ignoring them in `deny.toml` (`ignore = []` remains clean per Constitution Principle IV).

---

## Phase 8: Convergence

**Purpose**: Remaining unmet or contradicting codebase and remote-governance gaps observed during `/speckit-converge` (note: implementation was previously paused at `T005` awaiting `FR-013` maintainer approval; completing `T005`–`T032` satisfies these convergence items).

- [X] T033 CRITICAL: Narrow `image-extras` in `Cargo.toml` to `default-features = false, features = ["pcx"]` and upgrade `url >= 2.5.9` in `Cargo.lock` to resolve the 8 `--all-features` RustSec advisories per `Constitution VII / FR-007` (contradicts)
- [X] T034 CRITICAL: Pin all 7 external GitHub Actions in `.github/workflows/ci.yml` and `.github/workflows/release.yml` to verified 40-char commit SHAs, add explicit top-level `permissions: { contents: read }` to `ci.yml`, and scope `permissions: { contents: write }` in `release.yml` strictly to `publish-draft` per `Constitution VII / FR-008 / FR-014` (contradicts)
- [X] T035 CRITICAL: Reconcile `NOTICE` and `NOTICE.en.md` in lockstep by preserving the `2026-11-16` initial separation notice, adding the `2026-11-17` (`004`) modification notice, and removing stale `jxl-oxide` and active auto-update endpoint claims per `Constitution VIII / FR-012` (contradicts)
- [X] T036 Enable repository-level GitHub Private Vulnerability Reporting under maintainer approval (`FR-013`) before or simultaneously with enabling public issues per `FR-003 / SC-001` (missing)
- [X] T037 Create `.github/ISSUE_TEMPLATE/config.yml` (`blank_issues_enabled: false` + bilingual PVR and upstream contact links), land it on `main`, and enable `has_issues: true` under maintainer approval (`FR-013`) per `FR-001 / FR-002` (missing)
- [X] T038 Merge `004-launch-reliability-refactor` into `main` via PR and activate the `Protect main` Repository Ruleset (`bypass_actors: []`, `required_approving_review_count: 0`, `strict_required_status_checks_policy: true`, exact 6 non-experimental contexts with `"integration_id": 15368`, excluding `Launch check (Linux, experimental)`) under maintainer approval (`FR-013`) per `FR-004 / FR-005 / FR-006` (missing)
- [X] T039 Create `.github/workflows/dependency-audit.yml` with weekly `schedule` (`17 3 * * 1`), `workflow_dispatch`, `permissions: { contents: read }`, SHA-pinned actions, and matrix `arguments: ""` (`shipped-default`) / `"--all-features"` (`all-features`) with `command-arguments: advisories bans licenses sources` per `FR-007 / FR-008` (missing)
- [X] T040 Enable `sha_pinning_required: true` via `PUT repos/Ezenitac-AiService/ImageViewer/actions/permissions` under maintainer approval (`FR-013`) after all SHA-pinned workflows land on `main` per `FR-014 / SC-007` (missing)
- [X] T041 Normalize repository `description` (remove trailing whitespace), assign >= 5 discovery topics, and disable unused `has_wiki` and `has_projects` surfaces under maintainer approval (`FR-013`) per `FR-009 / SC-004 / SC-007` (missing)
- [X] T042 Enable `delete_branch_on_merge: true` and delete obsolete remote branches (`002-rust-crossplatform-viewer` and merged `004-launch-reliability-refactor`) under maintainer approval (`FR-013`) per `FR-011 / SC-005` (missing)
- [X] T043 Audit `CHANGELOG*`, `CONTRIBUTING*`, and `specs/003-independent-fork-separation/tasks.md` (recording verified 60-hit `gh search code` evidence) and confirm post-`005` `main` readiness for `004` `T110` (`v1.1.0-alpha.2` tag) per `FR-010 / FR-012 / SC-006` (partial)

---

## Phase 9: Convergence

**Purpose**: Additional security and intake-link convergence gaps discovered via multi-method adversarial verification against `Constitution VII`, `FR-003`, and `SC-001`.

- [X] T044 CRITICAL: Eliminate GitHub Actions shell script injection in `.github/workflows/release.yml` (`publish-draft` job line 182, `permissions: { contents: write }`) by passing `TAG: ${{ github.event.inputs.tag || github.ref_name }}` through step `env:` instead of interpolating `${{ ... }}` directly inside `run:` and validating `TAG` against `^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$` per `Constitution VII / FR-014` (contradicts)
- [X] T045 Add direct clickable Private Vulnerability Reporting links (`https://github.com/Ezenitac-AiService/ImageViewer/security/advisories/new`) in lockstep to `SECURITY.md` and `SECURITY.en.md` (line 13) so all root security disclosure references navigate directly to the active private intake channel per `SC-001 / FR-003 / Constitution I` (partial)
