# Feature Specification: Platform Tier Realignment (Constitution v3.0.0)

**Feature Branch**: `006-platform-tier-realignment`  
**Created**: 2026-11-24  
**Status**: Implemented  

**Input**: User description: "Follow-up downstream alignment for Constitution v3.0.0: macOS and iOS as top-priority targets with exclusive build-result verification, Linux and Android as lower-priority build-only targets without verification, Windows excluded from support."

## Clarifications

### Session 2026-11-24

- Q: Should 006-platform-tier-realignment implement iOS and Android mobile app packaging and iOS automated launch verification in the same feature, or first complete Windows retirement, Linux build-only conversion, governance/CI/docs alignment, and Constitution v3.0.0 finalization while deferring iOS and Android implementation to a dedicated follow-up feature (007)? → A: Focus 006 on retiring Windows, keeping macOS as the active verified top-priority target, converting Linux to build-only (zero runtime verification), updating default-branch protection and bilingual documentation, and finalizing Constitution v3.0.0; defer iOS and Android target implementation and packaging to a dedicated follow-up feature (007) while honestly documenting iOS (planned top-priority) and Android (planned lower-priority build-only) in public docs and release notes.
- Q: How should the existing unpublished v1.1.0-alpha.2 draft release (created under Constitution v2.4.0 with Windows assets and Windows verification records) be handled? → A: After 006 merges onto main and under explicit maintainer approval, delete the pre-amendment v1.1.0-alpha.2 draft release and regenerate a clean v3.0.0-aligned v1.1.0-alpha.2 draft containing only verified macOS artifacts and build-only/unverified Linux artifacts with zero Windows deliverables.
- Q: Should build or compilation failures on the lower-priority Linux target block pull-request merges and release draft creation, or remain non-blocking? → A: Keep Linux build and compilation checks as a required pull-request merge gate (with zero runtime or launch verification) per Constitution v3.0.0's build verification gate, and package the Linux build-only unverified artifact alongside macOS in release runs.
- Q: Should existing Windows-specific packaging scripts, Windows-only build dependencies, and Windows-only platform adapter/conditional code be removed in 006 or left dormant in the codebase? → A: Remove Windows-only packaging and verification scripts, Windows-only build dependencies, and Windows-only platform adapter/conditional branches cleanly during 006 (preserving the immutable legacy-v1.0.6 archive tag and historical specs/001–005 records intact).
- Q: When removing Windows-only code, scripts, and build dependencies under FR-013, should OS-agnostic Zero-Trust security guards handling Windows-formatted untrusted inputs (such as ZIP/CBZ backslash Zip-Slip traversal rejection and Windows-style path redaction in logs) be preserved? → A: Preserve all OS-agnostic Zero-Trust security guards and their negative security tests (including archive backslash traversal rejection and path redaction), removing only Windows-target-specific adapters/conditional compilation branches, Windows-only build dependencies, and Windows packaging/verification scripts.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Retire Windows & Align Governance Baseline (Priority: P1)

As a project maintainer operating under the amended platform policy, I want all active build pipelines, release pipelines, merge-protection gates, Windows-only packaging scripts/build dependencies/conditional platform paths, repository metadata, and Constitution v3.0.0 finalization to remove active Windows requirements without deadlocking merges or erasing legal GPLv3 attribution, so that every repository gate accurately enforces the new platform priorities.

**Why this priority**: Currently, automated verification checks, release packaging, and mandatory default-branch merge checks still enforce Windows builds and launch verification as hard blockers; until Windows gates and references are retired in the proper order, no alignment work can cleanly merge or ship under Constitution v3.0.0.

**Independent Test**: Can be fully tested by opening a change against the default branch, observing that required merge checks succeed without any Windows build or verification job, verifying that release packaging produces zero Windows deliverables, and confirming repository discovery topics and descriptions no longer advertise Windows support while preserving historical upstream origin notices.

**Acceptance Scenarios**:

1. **Given** the default branch currently blocks merges on legacy Windows build and launch-verification checks, **When** automated verification checks and default-branch merge protection rules are updated under maintainer approval in deadlock-safe order, **Then** mandatory merge gates cover only the realigned target checks and reject zero valid changes due to retired Windows check names.
2. **Given** the project originally forked from a Windows-only baseline and retains historical GPLv3 attribution records, **When** active repository metadata, capability promises, and user-facing intake prompts are audited after excluding Windows, **Then** active runtime promises exclude Windows while historical origin notices and archived `v1.0.6` references remain intact.
3. **Given** an existing pre-amendment `v1.1.0-alpha.2` draft pre-release still holds legacy Windows assets and verification records, **When** `006` merges onto the default branch and maintainer approval is granted, **Then** the pre-amendment draft is deleted and regenerated as a clean `v3.0.0`-aligned `v1.1.0-alpha.2` draft containing zero Windows deliverables.

---

### User Story 2 - Exclusive Top-Priority Verification for macOS & Honest iOS Roadmap Boundary (Priority: P1)

As a maintainer and Apple-ecosystem end user, I want `macOS` (`aarch64` and `x86_64`) maintained as the active top-priority target where every release candidate is built and verified exclusively for application launch, reference rendering, and clean lifecycle termination before publication, while `iOS` is honestly documented as a planned top-priority target deferred to dedicated follow-up feature `007`, so that active Apple releases stay strictly verified with zero overstated mobile claims.

**Why this priority**: Under Constitution v3.0.0, build-result verification belongs exclusively to the top-priority Apple tier (`macOS`, `iOS`); keeping `macOS` launch verification strictly enforced while deferring `iOS` mobile app implementation to `007` prevents governance stagnation and avoids claiming unbuilt `iOS` deliverables.

**Independent Test**: Can be fully tested by building `macOS` (`aarch64` and `x86_64`) release artifacts, executing automated launch-and-render verification on `macOS` (producing traceable verification records showing window display, reference image presentation, and clean exit), confirming a `macOS` launch failure blocks release draft creation, and confirming capability docs and release notes explicitly state `iOS` is planned (`007`) and not yet built.

**Acceptance Scenarios**:

1. **Given** `macOS` (`aarch64` and `x86_64`) is the active top-priority target in `006`, **When** continuous integration or release validation runs, **Then** `macOS` produces distributable universal build artifacts and executes automated build-result and launch verification.
2. **Given** a `macOS` artifact fails to launch, render a reference image, or exit cleanly during automated verification, **When** merge eligibility or release draft readiness is evaluated, **Then** the failure strictly blocks release draft creation and required merge approval.
3. **Given** `iOS` (`aarch64`) is elevated in Constitution v3.0.0 as top-priority but its mobile implementation is deferred to `007`, **When** automated verification completes and capability documentation or release notes are generated, **Then** release notes and documentation explicitly disclose `iOS` as a planned top-priority target not yet built or verified rather than implying shipped coverage.

---

### User Story 3 - Lower-Priority Build-Only Delivery for Linux & Honest Android Roadmap Boundary (Priority: P2)

As a Linux user downloading community builds, and as a maintainer conserving verification effort, I want `Linux` (`x86_64`) maintained strictly as a lower-priority build-only target that delivers packaged build artifacts without running any runtime launch or build-result verification checks, while `Android` is clearly documented as a planned lower-priority build-only target deferred to `007`.

**Why this priority**: Constitution v3.0.0 keeps `Linux` and `Android` as build-only deliverables while strictly prohibiting runtime verification overhead and preventing lower-priority behavior from blocking top-priority Apple releases.

**Independent Test**: Can be fully tested by inspecting continuous integration and release pipelines to confirm `Linux` performs build/packaging steps and outputs downloadable artifacts with zero launch-verification or runtime-result checks, confirming zero `Linux` runtime checks exist in default-branch merge protection rules, and confirming release notes and capability docs label `Linux` as build-only/unverified and `Android` as planned build-only (`007`).

**Acceptance Scenarios**:

1. **Given** a release workflow or pull-request pipeline previously executed a launch check on `Linux`, **When** pipelines are realigned to Constitution v3.0.0, **Then** every `Linux` runtime or launch-verification job is removed so `Linux` executes build and compilation checks only with zero runtime verification.
2. **Given** `Linux` build-only artifacts are attached to a release, **When** release notes and capability documents are generated, **Then** both documents label `Linux` explicitly as lower-priority, build-only, and unverified (and list `Android` honestly as planned build-only under `007`) without claiming runtime pass status.
3. **Given** default-branch merge protection gates are inspected, **When** mandatory checks are compared against all active workflow jobs, **Then** zero `Linux` launch or runtime checks appear in required merge gates.

---

### User Story 4 - Bilingual Documentation & Issue Intake Alignment (Priority: P2)

As a Korean- or English-speaking user, contributor, or bug reporter reading the project's public pages and issue templates, I want all bilingual root documentation, capability registers, first-launch guides, and issue-submission forms updated in lockstep to reflect `macOS` (active top-priority), `iOS` (planned top-priority), `Linux` (active build-only, unverified), `Android` (planned build-only, unverified), and `Windows` (excluded), so users never encounter conflicting instructions or submit requests for excluded platforms.

**Why this priority**: After amending platform tiers, stale Windows installation guides or outdated OS selectors in public documentation and bug-report forms immediately confuse users and contradict Constitution v3.0.0 across Korean and English pages.

**Independent Test**: Can be fully tested by auditing every bilingual root document pair (`README`, `CHANGELOG`, `CONTRIBUTING`, `SECURITY`, `NOTICE`, `THIRD_PARTY_NOTICES`), capability and first-launch user guides, and issue templates in both Korean and English to verify synchronized platform tiers, honest planned status for mobile targets (`007`), working bidirectional links, and updated OS selectors.

**Acceptance Scenarios**:

1. **Given** a prospective user opens either the Korean default landing documentation or its synchronized English counterpart, **When** they review supported platforms, build/download options, and capability status, **Then** `macOS` appears as the active verified top-priority target, `iOS` appears as planned top-priority (`007`), `Linux` appears as build-only/unverified, `Android` appears as planned build-only/unverified (`007`), and `Windows` is clearly stated as excluded from active support.
2. **Given** a user follows the first-launch guide after downloading a release artifact, **When** they view OS-specific trust or installation steps, **Then** active-Windows instructions are retired in favor of accurate `macOS` first-launch guidance alongside clear build-only caveats for `Linux` (and notes that mobile guides await `007`).
3. **Given** a user submits a public bug report or feature request via the repository's bilingual intake forms, **When** they select the target operating system or review platform expectations, **Then** selectable options cover active `macOS` and `Linux (build-only / unverified)` targets without accepting active `Windows` support requests.

---

### Edge Cases

- **Branch Protection Transition Deadlock**: Updating default-branch protection required status checks to retire Windows checks and remove `Linux` runtime checks in the wrong order causes merges to deadlock if required check names do not match the live workflows on the default branch. Realignment must update workflow check names first (or coordinate the ruleset update safely under maintainer approval) without disabling zero-admin-bypass enforcement.
- **Historical Windows GPLv3 Attribution vs. Active Exclusion**: Retiring Windows support must NOT delete legally mandated GNU GPLv3 attribution pointing to the original `v1.0.6` Windows baseline or alter the immutable historical archive tag and archived `specs/001`–`specs/005` historical records. Only active support claims, active builds, and user-facing active-platform guidance change.
- **Mobile Top-Priority & Lower-Priority Honesty Gap (`iOS` & `Android` Deferred to `007`)**: Because mobile app shells and packaging are scoped to follow-up feature `007`, `006` documentation and release notes must strictly avoid presenting `iOS` as already verified or `Android` artifacts as already downloadable until `007` delivers them (Principle VI).
- **Strict Absence of Lower-Priority Runtime Verification**: Even helpful informational runtime smoke tests on `Linux` directly contradict Constitution v3.0.0 ("하위는 빌드만 제공 - 확인 작업을 하지 않음" / zero build-result verification on lower-priority targets); `Linux` must run compilation/packaging checks only.
- **Unpublished Pre-Amendment Draft Releases**: A draft pre-release created under `v2.4.0` (`v1.1.0-alpha.2` Draft) contains Windows assets and a legacy platform verification table. After `006` lands on the default branch and with explicit maintainer approval, the old draft MUST be deleted and regenerated as a clean `v3.0.0`-aligned `v1.1.0-alpha.2` draft containing zero Windows artifacts.
- **Constitution Amendment Housekeeping**: The amended `constitution.md` (`v3.0.0`) still carries its temporary review comment (`Sync Impact Report`), which must be removed prior to committing `constitution.md` alongside this realignment work.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Active continuous integration workflows MUST remove Windows build, test, and launch-verification jobs so Windows is completely excluded from build and merge-verification obligations.
- **FR-002**: Active release packaging workflows MUST remove Windows artifact production and Windows launch verification, producing zero Windows deliverables in any subsequent release, and once aligned on the default branch MUST regenerate the unpublished `v1.1.0-alpha.2` draft release after deleting the pre-amendment draft under maintainer approval.
- **FR-003**: Top-priority release and verification workflows MUST build `macOS` (`aarch64` and `x86_64`) artifacts and execute automated build-result and launch verification (startup, UI display, reference-image render, and clean termination) exclusively on `macOS` during `006`, blocking release draft creation if `macOS` fails verification, while explicitly scoping `iOS` implementation and automated verification to dedicated follow-up feature `007`.
- **FR-004**: Lower-priority pipelines MUST maintain `Linux` (`x86_64`) compilation and build checks as a required pull-request gate and produce `Linux` release build artifacts strictly as build-only, while executing zero runtime launch verification or build-result verification checks on `Linux`, and explicitly scoping `Android` build-only implementation to dedicated follow-up feature `007`.
- **FR-005**: Automated verification workflows MUST continue compiling and checking library/workspace health (formatting, static lints, unit/integration tests on active hosts, and dependency/license/advisory/source compliance) including default and full feature configurations across active targets (`macOS` checks/launch and `Linux` build-only static/compile checks) without mock bypasses or disabled tests.
- **FR-006**: Release note generation and verification records MUST list only `macOS` under verified active top-priority targets in `006` (distinguishing automated vs. maintainer clean-device checks), label `Linux` outputs explicitly as `build-only` and `unverified` with zero runtime pass claims, and honestly indicate `iOS` (planned top-priority) and `Android` (planned build-only) as not yet built/shipped until `007`.
- **FR-007**: Default-branch protection rules MUST be synchronized with maintainer approval in deadlock-safe order so required merge checks retain `macOS` build/launch checks, `Linux` build/compilation checks, and dependency/security checks, while retiring obsolete Windows build and launch-verification check names and containing zero lower-priority runtime checks.
- **FR-008**: Bilingual Korean and English user-facing root documents, capability registers, and first-launch guides MUST be updated in lockstep with working bidirectional language links so every document accurately presents `macOS` (active top-priority verified), `iOS` (planned top-priority, deferred to `007`), `Linux` (lower-priority build-only, unverified), `Android` (planned build-only, unverified, deferred to `007`), and `Windows` (excluded from active support), while preserving historical `v1.0.6` GPLv3 §5(a) origin notices and legacy archive links.
- **FR-009**: Bilingual public issue intake templates MUST update their target operating system selections to active `macOS` and `Linux (build-only / unverified)` options (optionally noting planned `iOS`/`Android` roadmap status without accepting active `Windows` support requests), while preserving mandatory fork-confirmation prompts and upstream attribution routing.
- **FR-010**: Public repository discovery metadata (description and topics) MUST be reviewed and updated under maintainer approval so advertised platforms accurately match the `v3.0.0` realignment without trailing formatting defects.
- **FR-011**: Clean-device release verification checklists and manual verification procedures MUST update their active validation environments in `006` to `macOS` (Apple Silicon & Intel), retiring active Windows clean-machine obligations (`iOS` clean-device verification rows deferred to `007`).
- **FR-012**: Every live remote governance mutation (branch protection updates, repository metadata/topic changes, default-branch merges, and deleting plus regenerating the pre-amendment `v1.1.0-alpha.2` draft release) MUST require explicit maintainer approval and fail closed with actionable guidance if permissions are absent; finalizing `constitution.md` (`v3.0.0`) MUST remove its temporary review comment prior to commit.
- **FR-013**: Windows-only packaging scripts, Windows-only release verification scripts, Windows-only build dependencies, and Windows-target platform adapter or conditional compilation branches MUST be cleanly removed from active repository files during `006`, while strictly preserving OS-agnostic Zero-Trust security guards and their negative security tests (including Windows-formatted untrusted inputs such as archive backslash Zip-Slip traversal rejection and path redaction in logs), the immutable `legacy-v1.0.6` archive tag, and historical specification records (`specs/001-*` through `specs/005-*`).

### Key Entities

- **Platform Tier Policy**: Governance mapping assigning each platform target (`macOS`, `iOS`, `Linux`, `Android`, `Windows`) to its tier (`top-priority`, `lower-priority-build-only`, or `excluded`), active delivery status in `006` vs. `007` roadmap, verification requirement, and release-blocking status.
- **Top-Priority Verification Record**: Observable launch, reference-frame render, and clean-exit record restricted strictly to active top-priority targets (`macOS` in `006`, joined by `iOS` in `007`).
- **Build-Only Artifact**: Packaged distributable artifact produced for an active lower-priority target (`Linux` in `006`, joined by `Android` in `007`) accompanied by checksums and build-only/unverified disclosure, with zero runtime launch verification records.
- **Branch Protection Gate Set**: Exact set of mandatory default-branch merge checks matching live post-realignment workflow job names without orphaned Windows check names or lower-priority runtime checks.
- **Bilingual Platform Documentation Set**: Lockstep Korean default and English counterpart documents (`README`, `CHANGELOG`, `CONTRIBUTING`, `SECURITY`, `NOTICE`, `THIRD_PARTY_NOTICES`, capability register, first-launch guide, and issue forms) presenting consistent `v3.0.0` platform scope.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 0% of active build, test, packaging scripts/dependencies, Windows-target conditional runtime code paths, launch-verification, or merge-blocking checks target Windows, while 100% of OS-agnostic Zero-Trust security guards (including archive backslash traversal rejection and log path redaction), historical GPLv3 origin copyright notices, and the archived `legacy-v1.0.6` tag remain preserved.
- **SC-002**: 100% of active top-priority targets in `006` (`macOS` `aarch64` & `x86_64`) produce packaged release artifacts and pass automated launch/render verification before release draft creation, and any verification failure blocks draft release creation 100% of the time, with `0` misleading claims that `iOS` (`007`) is already shipped or verified.
- **SC-003**: 100% of active lower-priority targets in `006` (`Linux` `x86_64`) produce build artifacts in release runs while executing 0 runtime launch or build-result verification jobs and displaying explicit build-only/unverified labeling (plus planned `007` status for `Android`) on 100% of release notes and capability tables.
- **SC-004**: 100% of mandatory default-branch merge protection status checks correspond 1-to-1 with active verification jobs without deadlocks, orphaned Windows checks, or lower-priority runtime checks.
- **SC-005**: 100% of public bilingual documentation pairs, user guides, and issue-intake forms reflect identical `v3.0.0` platform tiers and `006` vs. `007` scope across Korean and English versions with 0 stale active-Windows installation or runtime promises and 0 broken cross-links.
- **SC-006**: 100% of dependency security, license compatibility (`GPL-3.0-only`), banned-package, and source checks pass cleanly across active default and all-feature configurations.

## Assumptions

- **Mobile Targets Scoped to Dedicated Feature `007`**: Per Clarification Q1 (`Option B`), `006` completes governance, CI/release realignment (retiring `Windows`, verifying `macOS`, making `Linux` build-only with zero verification), `constitution.md` `v3.0.0` commit readiness, and honest bilingual documentation updates; implementing `iOS` and `Android` application shells, cross-compilation, packaging, and `iOS` automated launch verification proceeds in dedicated follow-up feature `007`.
- **Historical Spec Preservation**: Completed or superseded specification records (`specs/001-*` through `specs/005-*`) represent historical baselines ratified under earlier constitution versions (`v1.x`–`v2.4.0`) and MUST NOT be retroactively rewritten; only active codebase behavior, workflows, live repository settings, and live documentation are brought into compliance with Constitution `v3.0.0`.
- **Pre-Amendment `v1.1.0-alpha.2` Draft Release Disposition**: Per Clarification Q2 (`Option A`), once `006-platform-tier-realignment` lands on the default branch and with explicit maintainer approval, the existing pre-amendment `v1.1.0-alpha.2` draft release (containing Windows artifacts created under `v2.4.0`) MUST be deleted and regenerated as a clean `v3.0.0`-aligned `v1.1.0-alpha.2` draft.
- **Zero-Trust Security & Dependency Compliance Preserved**: Excluding Windows (including removing Windows-only packaging scripts, Windows-only build dependencies, and Windows-target platform adapter/conditional branches per Clarification Q4) and converting Linux to build-only MUST NOT weaken or remove OS-agnostic Zero-Trust security guards handling Windows-formatted untrusted inputs (archive backslash Zip-Slip traversal checks, log path redaction per Clarification Q5), nor alter safety caps (`150 MP`, `512 MB`, `64 KB` settings), file-signature guards, symlink/traversal protections, or action commit-hash pinning established under Constitution VII and feature `005`.
