# Feature Specification: Independent Project Separation, Licensing & Release Cutover

**Feature Branch**: `003-independent-fork-separation`

**Created**: 2026-11-16

**Status**: In Verification

**Input**: User description: "1,2,3단계를 진행하기 위한 스펙 작성 (라이선스·저작권 정리, GitHub 포크 네트워크 및 Git 리모트 분리, 프로젝트 정체성·식별자·업데이트 피드·CI/레거시 분리)"

## Clarifications

### Session 2026-11-16

- **Q1 (Fork Detachment Method)**: `B` — Keep existing repository `Ezenitac-AiService/ImageViewer` (preserving URL, stars, and issues) and request GitHub Support to detach it from the upstream fork network, while removing local/CI `upstream` sync remotes.
- **Q2 (Project Name & Identifiers)**: `B` — Retain the `ImageViewer` product name and user `settings.json` path continuity, while updating repository ownership, HTTPS update feeds, and reverse-DNS identifiers to `Ezenitac-AiService/ImageViewer` (`com.ezenitac.imageviewer`).
- **Q3 (Major Project Documentation Scope)**: User requested explicit update contents and planning for major project documents (`README.md`, `LICENSE`/`NOTICE`, `CONTRIBUTING.md`, `SECURITY.md`, `.specify/memory/constitution.md`, and `specs/001-*`/`specs/002-*` status alignment).
- **Q4 (GPLv3 §5, Third-Party Attribution & Courteous Fork Etiquette)**: User accepted all 4 researched additions: (1) GPLv3 §5(a)/§5(d) prominent modification & date notices in `NOTICE`/`CHANGELOG.md`/`README.md` and in-app `About` dialog; (2) `THIRD_PARTY_NOTICES.md` + automated dependency license compatibility gate; (3) courteous upstream origin & issue-routing notices in `README.md` top banner and `.github/ISSUE_TEMPLATE/`; (4) safe detachment verification gates (`legacy-v1.0.6` tag, full CI pass, release `SHA256SUMS.txt`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - License Compliance, Attribution, and Legal Continuity (Priority: P1)

As a maintainer turning a forked codebase with a divergent cross-platform architecture into an independent project, I want all upstream license obligations, original copyright notices, and derivative attribution records to be preserved and harmonized consistently across project documentation and package metadata so that the independent project remains legally compliant.

**Why this priority**: Distributing an independent release with mismatched or missing license/copyright notices creates immediate legal compliance risk under copyleft terms (`GPLv3`).

**Independent Test**: Can be fully tested by auditing repository license files, attribution notices, package manifests, and the application About view to verify consistent `GPLv3` compliance and preserved upstream copyright notices alongside fork maintainer attribution.

**Acceptance Scenarios**:

1. **Given** the repository currently carries an upstream `GPLv3` license alongside divergent cross-platform package metadata, **When** license harmonization is completed, **Then** every package manifest, installer notice, and `About` view MUST declare a single compatible `GPLv3` license without conflicting permissive declarations.
2. **Given** the project originated from an upstream repository (`v1.0.6`), **When** attribution files are inspected, **Then** original upstream copyright notices MUST remain intact, fork maintainer copyright notices MUST be appended, and a clear origin notice MUST credit the upstream project.

---

### User Story 2 - Repository Detachment, Git Remote Separation, and Legacy Archival (Priority: P2)

As a maintainer no longer merging upstream architectural changes, I want the repository detached from the upstream fork network, local/CI upstream sync paths cleanly severed, and the legacy single-OS implementation archived so that contributors can search code, open pull requests, and develop on the default branch without upstream target confusion.

**Why this priority**: While linked as a GitHub fork, pull requests default to the upstream repository and repository-wide code search is disabled, blocking normal independent team workflows.

**Independent Test**: Can be fully tested by verifying repository fork status, git remote configuration, preserved historical tag/branch for the legacy `v1.0.6` baseline, and a clean default branch free of superseded legacy runtime source.

**Acceptance Scenarios**:

1. **Given** the repository is currently linked in the upstream fork network, **When** separation is executed by retaining `Ezenitac-AiService/ImageViewer` and completing a GitHub Support fork-detachment request alongside removing `upstream` remotes, **Then** the target repository MUST be a standalone repository whose default branch receives pull requests directly and is indexed for code search.
2. **Given** the legacy single-OS implementation (`v1.0.6`) exists on the main branch alongside the new cross-platform implementation branch, **When** branch cutover runs, **Then** a read-only archive tag (`legacy-v1.0.6`) MUST preserve the historical baseline, the cross-platform branch MUST become the default main branch, and superseded legacy build/source directories MUST be removed from active development paths.

---

### User Story 3 - Independent Project Identity, Update Feed Isolation, and Release Cutover (Priority: P3)

As an end user installing and running the independent cross-platform viewer, I want the application name, OS identifiers, documentation links, and automatic update feed to point exclusively to the independent project's own repository and releases so that my app never downloads mismatched upstream updates or collides with the upstream application.

**Why this priority**: Pointing an active auto-updater or OS application ID at the upstream repository would cause users to be prompted to overwrite their cross-platform app with incompatible upstream releases.

**Independent Test**: Can be fully tested by searching all source, packaging, and workflow files for upstream repository URLs and verifying that update checks, About links, OS bundle/app IDs, and release automation target only the independent project identity.

**Acceptance Scenarios**:

1. **Given** the application currently references upstream or placeholder repository URLs and identifiers in update checks, About metadata, and OS packaging templates, **When** identity separation is applied while retaining the `ImageViewer` product name and user config continuity under `Ezenitac-AiService/ImageViewer` (`com.ezenitac.imageviewer`), **Then** every update endpoint, project homepage link, OS bundle ID, desktop entry, and installer identifier MUST resolve exclusively to `Ezenitac-AiService/ImageViewer`.
2. **Given** automated CI and release workflows exist for both legacy and cross-platform builds, **When** release cutover completes, **Then** active CI and release automation MUST build, verify, and publish artifacts solely for the independent cross-platform project across all supported desktop OSes.

---

### Edge Cases

- What happens if an existing user upgrades from a pre-separation `v1.0.6` configuration file when the application identifier or settings directory changes vs stays the same? (Must define deterministic migration or documented cutover behavior so user preferences are not silently lost.)
- What happens if `upstream` git remote references remain on developer machines or CI scripts after detachment? (Documentation and verification checks must ensure no automated workflow pushes to or pulls from the detached upstream.)
- What happens if third-party dependency licenses in the About notice or packaging conflict with `GPLv3` redistribution requirements? (Must verify license compatibility across bundled dependencies.)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001 (License, Copyright & GPLv3 §5 Integrity)**: System and repository artifacts MUST preserve the existing `GPLv3` license text and original upstream copyright lines intact, append `Ezenitac-AiService` copyright lines, include prominent GPLv3 §5(a) modification and date notices (`2026-11-16`) in `NOTICE`/`CHANGELOG.md`/`README.md`, and harmonize all workspace package manifests and in-app `About` license metadata (§5(d)) to `GPLv3`.
- **FR-002 (Courteous Upstream Attribution & Issue Routing)**: Repository documentation (`README.md` top banner, `NOTICE`), `.github/ISSUE_TEMPLATE/` forms, and in-app `About` information MUST respectfully credit the upstream origin (`edith2580/ImageViewer`, `v1.0.6`), clearly state the project's independent cross-platform fork status, and instruct users not to report cross-platform fork issues to the upstream maintainer while linking upstream users back to `edith2580/ImageViewer`.
- **FR-003 (Fork Network & Remote Detachment)**: Maintainers MUST execute fork detachment for `Ezenitac-AiService/ImageViewer` via GitHub Support ("leave fork network") and remove or read-only lock any `upstream` synchronization references so default pull-request targets and repository code search operate independently.
- **FR-004 (Legacy Archival & Default Branch Cutover)**: Project history MUST preserve the `v1.0.6` legacy baseline via an immutable archive tag (`legacy-v1.0.6`), merge/promote the cross-platform implementation to the default `main` branch, and remove superseded legacy source/installer workflows from active `main` paths.
- **FR-005 (Project Identity & OS Identifier Separation)**: All user-facing titles, OS application IDs, bundle identifiers, desktop entries, installer metadata, and repository/homepage URLs MUST retain the `ImageViewer` application name and config continuity while updating all ownership, reverse-DNS bundle IDs (`com.ezenitac.imageviewer`), and URLs consistently to `Ezenitac-AiService/ImageViewer` with zero leftover placeholder or upstream URLs.
- **FR-006 (Auto-Updater Endpoint Isolation)**: Application update checks and release page links MUST target exclusively the independent project's HTTPS release feed and never query or download releases from the upstream repository.
- **FR-007 (CI & Release Pipeline Cutover)**: Repository automation MUST disable or remove superseded legacy single-OS release workflows and route tag/release publishing strictly through the independent cross-platform build and verification pipeline.
- **FR-008 (Major Project Documentation & Governance Updates)**: The separation plan MUST include explicit update contents and verification for all major project documents:
  - `README.md`: Rewrite feature overview, supported OSes (Windows, macOS, Linux), build/install instructions, screenshots/usage, courteous top-level independent fork banner, and `edith2580/ImageViewer` (`v1.0.6`) origin attribution.
  - `LICENSE`, `NOTICE`, `CHANGELOG.md` & `THIRD_PARTY_NOTICES.md`: Preserve `GPLv3` license text and original copyright lines, append `Ezenitac-AiService` copyright + GPLv3 §5(a) modification/date notices, and publish complete third-party framework/library copyright & license notices.
  - `CONTRIBUTING.md`, `SECURITY.md` & `.github/ISSUE_TEMPLATE/`: Establish independent contributor guidelines (detached-fork workflow, no `upstream` merge policy), issue templates separating fork vs upstream support, and security vulnerability reporting channels for `Ezenitac-AiService/ImageViewer`.
  - Governance & Spec History (`.specify/memory/constitution.md`, `specs/001-*`, `specs/002-*`): Mark `001` artifacts as archived legacy `v1.0.6` baseline and align any repository/feed references in `002` and governance docs with `Ezenitac-AiService/ImageViewer`.
- **FR-009 (Third-Party License Compatibility Gate)**: Automated verification MUST audit all bundled/transitive dependencies for `GPLv3` compatibility (rejecting unlicensed or GPLv3-incompatible dependencies) and ensure `THIRD_PARTY_NOTICES.md` and the in-app `About` view accurately reflect used frameworks and libraries.
- **FR-010 (Safe Detachment & Release Verification Gates)**: Before submitting the GitHub Support fork-detachment request and publishing independent releases, maintainers MUST verify `legacy-v1.0.6` tag creation, full cross-platform build/test passage on `main`, and `SHA256SUMS.txt` checksum generation for release artifacts.

### Key Entities

- **License & Attribution Record**: Governing copyleft license (`GPLv3`), original author copyright lines, fork maintainer copyright lines, and origin acknowledgment notice.
- **Repository Separation State**: Standalone hosting status (detached from fork network), remote configuration, archival baseline tag (`legacy-v1.0.6`), and active default branch.
- **Project Identity Profile**: Canonical application display name, executable/package slug, reverse-DNS OS bundle/desktop ID, repository owner/slug, and HTTPS update feed URL.
- **Documentation Update Inventory**: Target root, governance, and specification documents (`README.md`, `NOTICE`, `CONTRIBUTING.md`, `SECURITY.md`, `.specify/memory/constitution.md`, `specs/001-*`, `specs/002-*`) paired with required section changes and owner verification status.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001 (100% License, GPLv3 §5 & Third-Party Attribution Consistency)**: Automated repository audit finds **0** conflicting non-GPLv3 license declarations in project manifests, **0** GPLv3-incompatible or unlicensed dependencies, **100%** preservation of upstream copyright notices + GPLv3 §5(a) modification/date notices, and verified `NOTICE`/`THIRD_PARTY_NOTICES.md`/`README.md` attribution.
- **SC-002 (Zero Upstream Runtime Coupling)**: Full-text audit of all active runtime source, packaging templates, and CI workflows returns **0** references to the upstream repository URL (`edith2580/ImageViewer`) or placeholder `ImageViewer/ImageViewer` update endpoints.
- **SC-003 (Standalone Repository & Clean Default Branch)**: The independent repository operates outside the upstream fork network (code search enabled, PRs default to self), preserves `legacy-v1.0.6` as a tagged commit, and passes full cross-platform automated build, test, and `SHA256SUMS.txt` release verification on the `main` branch.
- **SC-004 (100% Major Documentation & Etiquette Alignment)**: All major project documents (`README.md`, `NOTICE`, `CHANGELOG.md`, `THIRD_PARTY_NOTICES.md`, `CONTRIBUTING.md`, `SECURITY.md`, `.github/ISSUE_TEMPLATE/`, `.specify/memory/constitution.md`, `specs/001-*`, `specs/002-*`) reflect the cross-platform `Ezenitac-AiService/ImageViewer` identity, `GPLv3` licensing, courteous upstream routing, and detached-fork workflow with **0** stale legacy-only or upstream-repo instructions.

## Assumptions

- **Existing Cross-Platform Implementation**: Feature `002-rust-crossplatform-viewer` is already implemented on branch `002-rust-crossplatform-viewer` and serves as the baseline promoted to `main` during cutover.
- **GPLv3 Governing License**: Because the repository root `LICENSE` is `GPL-3.0`, maintaining `GPL-3.0` compliance (rather than relicensing to MIT) is assumed unless explicit written relicensing consent is obtained from all upstream copyright holders.
- **Selective Future Adoption**: Future ideas from the upstream repository, if desired, will be re-specified and re-implemented independently rather than merged via `git merge upstream`.
