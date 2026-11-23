# Feature Specification: Repository Governance Hardening & Post-Separation Operations

**Feature Branch**: `005-repo-governance-hardening`  
**Created**: 2026-11-23  
**Status**: Draft  

**Input**: User description: "Comparative post-separation review against EasyViewer identifying remaining repository governance, intake, branch protection, proactive dependency monitoring, metadata hygiene, and branch/release handoff tasks needed for ImageViewer."

## Clarifications

### Session 2026-11-23

- Q: How should default-branch protection enforce pull requests, review approvals, and administrator bypass? → A: Require pull requests and mandatory status checks with 0 required approving reviews (enabling solo-maintainer merges) and no administrator bypass.
- Q: When public issues are enabled, should blank issues be disabled so every report goes through fork-confirming templates? → A: Disable blank issues and add explicit issue-chooser links for private vulnerability reporting and the upstream repository.
- Q: How should scheduled dependency audit failures notify maintainers? → A: Keep strictly read-only permissions and surface failures solely through workflow run failure status and built-in platform notifications without creating repository issues.
- Q: When should the verified `004-launch-reliability-refactor` branch be merged into the default branch relative to remaining release verification tasks? → A: Merge `004-launch-reliability-refactor` into the default branch via pull request after completing its analysis consistency check (`T109`), then execute the `v1.1.0-alpha.2` draft release and clean-machine checklist (`T110`) from the default branch and retire merged obsolete branches.
- Q: Private Vulnerability Reporting is currently disabled on the repository; should enabling it before or alongside public issues be a mandatory requirement? → A: Yes; require enabling Private Vulnerability Reporting under maintainer approval before or simultaneously with public issue intake and verify it in `FR-003` and `SC-001`.
- Q: What dependency scope should the weekly audit cover, and how should `SC-003` account for public-repository 60-day scheduled-workflow inactivity pauses? → A: Audit both the shipped default configuration and the full all-features workspace configuration on the weekly schedule, retain continuous platform vulnerability alerts for inactivity coverage, and qualify `SC-003` with the 60-day inactivity pause caveat plus manual re-enablement/dispatch guidance.
- Q: In what order should default-branch protection be activated relative to merging `004-launch-reliability-refactor`, and should merged PR branches be auto-deleted? → A: Merge `004-launch-reliability-refactor` into the default branch via pull request first so updated verification check names are established on the default branch, then activate the default-branch protection ruleset against those post-`004` checks, and enable automatic head-branch deletion upon PR merge.
- Q: How should the factual contradiction between `003`-era statements in `NOTICE`/`NOTICE.en.md` and `004` shipped changes be resolved? → A: Preserve the `2026-11-16` initial fork-separation history in `NOTICE` and `NOTICE.en.md` while adding the `2026-11-17` modification notice and reconciling the shipped architecture and capability summary (no network/auto-updater, verified core formats) with `README`, `CHANGELOG`, and `SECURITY`.
- Q: Under Zero Trust security (Constitution VII), should immutable commit-hash pinning for external automation actions and disabling unused public surfaces (Wiki, Projects) be included in scope? → A: Yes; include both pinning all external automation actions to immutable commit hashes (with repository-level pinning enforcement) and disabling unused Wiki and Projects surfaces under maintainer approval.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Enable Public Issue Reporting & Fork-Etiquette Routing (Priority: P1)

As an end user or contributor reading the project README, security policy, or support guidance, I want to open bug reports and feature requests directly through the repository's public issue intake forms so that documented feedback channels actually work, while security vulnerabilities remain routed exclusively to private reporting.

**Why this priority**: Public documentation, license/security policies, and bilingual intake templates already instruct users to file issues on this repository instead of the upstream project, yet the repository currently has public issue tracking disabled, creating an immediate broken user journey.

**Independent Test**: Can be fully tested by visiting the repository's issue entry point from a non-maintainer account or public browser session, selecting each available intake form (bug report and feature request), verifying fork-confirmation prompts are enforced, and confirming security advisories link only to private reporting.

**Acceptance Scenarios**:

1. **Given** the repository currently has public issue reporting turned off while documentation points users to its issue tracker, **When** public issue intake is enabled with explicit maintainer approval, **Then** users clicking issue links from the README or documentation land on working bilingual bug-report and feature-request forms.
2. **Given** a user starts submitting a bug report or feature request, **When** they complete the intake form, **Then** the form requires explicit confirmation that the report applies to this independent fork rather than the upstream project.
3. **Given** a user intends to report a security vulnerability, **When** they inspect the issue chooser and security policy after private vulnerability reporting has been enabled prior to or alongside public issues, **Then** public issue templates and policy documents direct them to an active, working private vulnerability reporting channel instead of opening a public issue.

---

### User Story 2 - Protect Default Branch with Mandatory Verification Gates (Priority: P1)

As a maintainer safeguarding release quality and security, I want the default branch protected so that no code or documentation change can be merged unless all mandatory automated checks—cross-platform builds/tests, supported-platform launch checks, and dependency/license audits—pass, while experimental-platform checks remain non-blocking and direct history rewriting or deletion is prohibited.

**Why this priority**: Currently the default branch has no protection rules at all, allowing accidental direct pushes, unverified merges, concurrent merge races that drop commits, or merges before required platform launch checks complete.

**Independent Test**: Can be fully tested by attempting to merge a change with failing required checks, attempting a force-push or branch deletion on the default branch, and observing a scenario where only an experimental platform check fails while all required supported-platform checks succeed.

**Acceptance Scenarios**:

1. **Given** a proposed change to the default branch has not passed all required cross-platform verification, supported-platform launch verification, and dependency/license checks, **When** a merge is attempted, **Then** the repository blocks merging until every required check succeeds.
2. **Given** a proposed change passes all supported-platform checks and dependency/license checks while an experimental-platform launch check fails, **When** merge eligibility is evaluated, **Then** the experimental-platform result is recorded as informational and does not block the merge.
3. **Given** a contributor or maintainer attempts to push directly, force-push history changes, or delete the default branch (even with administrator privileges), **When** the action is submitted, **Then** the repository rejects direct or destructive actions without default administrator bypass and requires changes to enter through pull requests with 0 required approving reviews and passing mandatory checks.
4. **Given** another commit lands on the default branch while a proposed change is waiting to merge, **When** merge readiness is re-evaluated, **Then** the proposed change must pass verification against the updated default branch state so concurrent commits cannot bypass checks.

---

### User Story 3 - Proactive Scheduled Dependency & Advisory Monitoring (Priority: P2)

As a maintainer preparing releases, I want dependencies and license policies audited automatically on a recurring weekly schedule even when no repository code has changed so that newly published security advisories or license compliance issues are discovered immediately rather than blocking an imminent release.

**Why this priority**: Security advisories can appear independently of repository commits and unexpectedly break verification gates immediately prior to tagging or publishing a release, delaying delivery.

**Independent Test**: Can be fully tested by triggering the scheduled dependency audit independently of any code push, confirming read-only execution with least privilege, and verifying that a known advisory or license violation produces a visible failure alert while a clean dependency tree passes.

**Acceptance Scenarios**:

1. **Given** no commits have been pushed to the repository during the past week, **When** the scheduled weekly audit window arrives, **Then** the repository automatically evaluates both the shipped default dependency graph and the full optional-feature workspace dependency graph against current security advisories, allowed licenses, prohibited packages, and approved sources.
2. **Given** a new security advisory affects a dependency in the repository without any recent code changes, **When** the scheduled audit completes, **Then** the audit finishes with a visible failure status so maintainers can remediate dependencies ahead of release packaging.
3. **Given** an untrusted contributor inspects or triggers scheduled repository automation, **When** the audit runs, **Then** it operates solely with read-only access and writes no release artifacts or repository modifications.

---

### User Story 4 - Repository Discoverability, Branch Hygiene & Release Handoff Readiness (Priority: P2)

As a maintainer and prospective user, I want the repository's public description, topic tags, search index status, branch inventory, and governance/notice documentation cleaned up and synchronized so that users can easily find the detached project, obsolete work branches are retired, and verified launch-reliability work is cleanly consolidated for the next alpha release.

**Why this priority**: Trailing metadata formatting defects, missing search topics, stale remote feature branches, and unmerged completed work behind the default branch obscure project status and create confusion before the next pre-release verification cycle.

**Independent Test**: Can be fully tested by inspecting the repository's public metadata (description, topics, search visibility), listing active remote branches, verifying that pending launch-reliability changes are cleanly merged onto the default branch after review/CI, and auditing bilingual governance/notice files for synchronized dates and status accuracy.

**Acceptance Scenarios**:

1. **Given** the repository currently lacks discovery topics and has trailing whitespace in its description, **When** repository metadata is updated with maintainer approval, **Then** the public description is cleanly formatted and accurate discovery topics are attached alongside verified code search indexing.
2. **Given** completed or superseded remote feature branches remain alongside active work, **When** verified feature work is merged to the default branch, **Then** obsolete merged remote branches are removed so only active development branches and release tags remain.
3. **Given** the repository maintains paired Korean and English governance documents (`NOTICE`, `CHANGELOG`, `README`, `SECURITY`, `CONTRIBUTING`, `THIRD_PARTY_NOTICES`), **When** detachment verification records, release-readiness status, or notice summaries are updated, **Then** both language versions are updated in lockstep with accurate dates, verified upstream detachment status, and current capability summaries.

---

### Edge Cases

- **Security Disclosure via Public Issues**: A user ignores the security policy and tries to file a vulnerability publicly once issues are enabled. The issue intake experience must prominently redirect vulnerability reports to private advisory intake before submission.
- **Experimental Platform Failure During Merge**: An experimental desktop target fails its launch check due to headless environment constraints while all supported targets pass. Merge protection must distinguish mandatory supported targets from non-blocking experimental targets.
- **Unplanned Advisory Between Releases**: A third-party dependency receives a newly disclosed vulnerability days before a planned release with zero code edits. The scheduled audit must flag this proactively without waiting for a new pull request or tag.
- **Scheduled Audit Inactivity Pause**: If a public repository receives no commit or repository activity for 60 consecutive days, the hosting platform automatically pauses recurring scheduled audits. Continuous repository vulnerability alerts must remain active as passive coverage during inactivity, and maintainers must re-enable and manually run the audit before cutting any release.
- **Concurrent Merge Race**: Two updates pass checks separately and one merges first, leaving the second stale against the default branch. Branch protection must prevent merging stale check results without re-verification against the latest default branch tip.
- **Protection Bootstrapping Check-Name Mismatch**: Enabling a no-bypass ruleset referencing updated verification check names before `004-launch-reliability-refactor` lands on the default branch causes a merge deadlock when the default branch still uses legacy check names; protection activation must follow landing the updated verification workflows on the default branch.
- **Admin Permission & Maintainer Approval Gate**: Modifying live repository settings (enabling issues, updating metadata/topics, activating branch protection rulesets, deleting remote branches) affects public repository state and requires explicit maintainer confirmation and appropriate repository admin privileges; if permissions are lacking or approval is withheld, automation must report the exact manual steps cleanly without bypassing controls.
- **Bilingual Drift During Document Updates**: Updating a date, upstream reference, or capability summary in only one language root document creates constitutional noncompliance; both Korean default and English counterpart documents must stay synchronized.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The repository MUST enable public issue intake so that every issue-tracker reference in user-facing documentation resolves to an accessible issue creation view.
- **FR-002**: The public issue intake experience MUST disable blank unstructured issues so all public issues pass through bilingual templates with mandatory fork-origin confirmation, and MUST include a direct issue-chooser link routing original-project inquiries to the upstream repository.
- **FR-003**: Repository-level private vulnerability reporting MUST be enabled under maintainer approval before or simultaneously with enabling public issues, and issue intake MUST provide a prominent direct issue-chooser link to that active private reporting channel so security vulnerability disclosures remain routed exclusively away from public issues.
- **FR-004**: The default branch MUST prohibit direct pushes, force-pushes, and branch deletion, enforce pull-request-only merges with 0 required approving reviews (enabling solo-maintainer merges once checks pass), and disallow default administrator bypass.
- **FR-005**: Merges into the default branch MUST require passing status checks for cross-platform build/test suites, supported-platform launch verification, and dependency/license/source audits, while keeping experimental-platform launch checks non-blocking; to prevent check-name mismatch deadlocks under no-bypass enforcement, the default-branch protection ruleset MUST be activated immediately after `004-launch-reliability-refactor` establishes those updated checks on the default branch via pull request.
- **FR-006**: Branch protection rules MUST require proposed changes to pass required checks against the current state of the default branch (or equivalent merge-queue/up-to-date enforcement) before merging to prevent concurrent-merge regressions.
- **FR-007**: Repository automation MUST include a recurring weekly dependency, security advisory, and license audit covering both the shipped default dependency graph and the full optional-feature workspace dependency graph without requiring code commits, while also supporting manual maintainer triggering and retaining continuous repository vulnerability alerts.
- **FR-008**: The scheduled dependency audit MUST operate under strictly read-only least-privilege permissions (creating no automated issues or repository writes) and surface failures solely through visible run failure status and built-in platform notifications whenever a new advisory, banned dependency, unapproved source, or incompatible license appears.
- **FR-009**: Repository public metadata MUST be normalized by removing trailing whitespace from the description, assigning relevant, accurate discovery topics matching the project's scope, and disabling unused public repository surfaces (such as Wiki and Projects) to minimize unmonitored spam or abuse surfaces.
- **FR-010**: Post-detachment repository verification records MUST confirm whether repository code search indexing is operational and record the verified standalone status across applicable feature records.
- **FR-011**: Pending launch-reliability work (`004-launch-reliability-refactor`) MUST be merged into the default branch via pull request after completing its analysis consistency check (`T109`) so that `v1.1.0-alpha.2` draft-release packaging and clean-machine verification (`T110`) execute from the default branch; completed or superseded remote feature branches (including `002-rust-crossplatform-viewer` and merged `004-launch-reliability-refactor`) MUST be cleaned up once integrated, and automatic head-branch deletion upon pull-request merge MUST be enabled to prevent future stale branches.
- **FR-012**: Root governance, notice, and changelog documents MUST maintain synchronized Korean default and English counterpart files with accurate detachment dates, accurate summaries of shipped capabilities, and truthful release status ahead of the next alpha release; specifically, `NOTICE` and `NOTICE.en.md` MUST preserve the `2026-11-16` initial fork-separation notice while adding the `2026-11-17` modification notice and reconciling the shipped architecture and capability summary (offline/no-auto-updater posture and verified core formats) with `README`, `CHANGELOG`, and `SECURITY`.
- **FR-013**: Any operation that mutates live remote repository settings (including enabling public issues, enabling private vulnerability reporting, enabling automatic branch deletion on merge, disabling unused Wiki/Projects surfaces, enforcing immutable action pinning, and updating metadata/topics), branch protection rules, default-branch merges, or remote branch deletions MUST be gated behind explicit maintainer approval and fail closed with actionable guidance if administrative permissions are absent.
- **FR-014**: All external third-party automation actions used across repository verification, release, and scheduled audit workflows MUST be pinned to immutable commit identifiers, and repository-level enforcement requiring immutable action pinning MUST be enabled under maintainer approval.

### Key Entities

- **Issue Intake Channel**: Public repository issue configuration paired with bilingual bug/feature intake forms and private security disclosure redirection.
- **Branch Protection Policy**: Governance rules bound to the default branch specifying forbidden history operations, required status gates, up-to-date branch constraints, and non-blocking treatment for experimental platforms.
- **Scheduled Dependency Audit**: Recurring time-triggered verification run evaluating project dependencies against advisory databases, license allowlists, bans, and source restrictions independently of code changes.
- **Repository Metadata Profile**: Public identity fields for the repository including trimmed description text, discovery topic tags, and confirmed standalone code-search index status.
- **Branch & Release Handoff Record**: Inventory of active vs. retired branches and synchronized bilingual governance/notice/changelog records tracking readiness for the next alpha release.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of issue-reporting links in root Korean and English documentation navigate directly to active, openable issue forms while repository private vulnerability reporting is verified enabled and 100% of security disclosure links route to that active private intake channel.
- **SC-002**: 0% of pull requests failing any mandatory supported-platform or dependency check can merge into the default branch, while 100% of changes failing only experimental-platform launch checks remain mergeable.
- **SC-003**: During active repository periods (activity within 60 days), 100% of newly disclosed dependency security advisories or license policy violations across both shipped default and full optional-feature dependency graphs are surfaced within at most 7 days without requiring a developer commit; when prolonged inactivity pauses scheduled runs after 60 days, continuous vulnerability alerts remain enabled and manual pre-release audit execution passes or flags findings cleanly.
- **SC-004**: Repository public metadata exhibits 0 formatting defects (no leading/trailing whitespace in description), at least 5 accurate discovery topics, and verified code-search indexing for the standalone repository.
- **SC-005**: 0 obsolete merged remote feature branches remain after consolidation of verified launch-reliability work onto the default branch.
- **SC-006**: 100% of modified root governance, notice, and changelog files have matching Korean and English versions with identical factual dates, release status, and bidirectional navigation links.
- **SC-007**: 100% of external third-party automation action references across repository workflows are pinned to immutable commit identifiers with repository-level pinning enforcement active, and 0 unused public repository surfaces (Wiki, Projects) remain enabled.

## Assumptions

- **Existing Bilingual & Fork-Etiquette Foundation**: Unlike sibling projects that adopted these practices later, `ImageViewer` already complies with Constitution v2.4.0 Principle I (`*.md` and `*.en.md` bilingual root pairs), already includes README badges and fork-etiquette notices, already provides bilingual issue forms with fork-confirmation checkboxes, and already carries launch-reliability and diagnostic verification in specification `004`; therefore no constitution amendment or duplicate feature rewrite is required in this specification.
- **Explicit Maintainer Approval for Live Repository Settings**: Enabling public issues, enabling private vulnerability reporting, enabling automatic branch deletion on merge, disabling unused Wiki/Projects surfaces, enforcing immutable action pinning, modifying repository description/topics, activating branch protection rulesets, merging verified branches into the default branch, and deleting remote branches change live shared repository state; planning and implementation will separate repository file changes (such as workflow/documentation updates) from explicit maintainer-approved remote configuration steps.
- **Weekly Audit Cadence**: A once-weekly scheduled dependency audit strikes an appropriate balance between rapid detection of new security advisories and conserving runner minutes, while remaining manually triggerable before any release tag.
- **Experimental Platform Policy**: Under Constitution v2.4.0, Windows and macOS are supported release-blocking targets while Linux remains an experimental non-blocking target; branch protection rules must preserve that distinction rather than treating experimental launch checks as hard merge blockers.
- **Pending Release Verification Scope**: Under the clarified merge sequence, `004-launch-reliability-refactor` is merged via pull request into the default branch after completing its analysis consistency check (`T109`), and the `v1.1.0-alpha.2` draft release plus clean-machine manual verification (`T110`) are then executed from the default branch; this specification governs landing and verifying that handoff cleanly rather than duplicating `004` runtime testing.
