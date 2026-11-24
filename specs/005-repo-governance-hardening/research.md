# Phase 0 Research: Repository Governance Hardening & Post-Separation Operations

**Feature Branch**: `005-repo-governance-hardening`  
**Date**: 2026-11-23  
**Spec**: [spec.md](./spec.md)

Every architectural, workflow, dependency, and repository-configuration decision in this plan is grounded in directly observed repository outputs, cargo-deny execution results, GitHub REST API state, and official GitHub documentation (Constitution Principle V).

---

## R1. Private Vulnerability Reporting & Issue Intake Routing (`FR-001`–`FR-003`, `FR-013`)

- **Decision**:
  1. Enable GitHub Private Vulnerability Reporting (`PUT /repos/Ezenitac-AiService/ImageViewer/private-vulnerability-reporting`) **before or simultaneously with** enabling public repository issues (`PATCH /repos/Ezenitac-AiService/ImageViewer` with `has_issues: true`).
  2. Add `.github/ISSUE_TEMPLATE/config.yml` setting `blank_issues_enabled: false` and configuring bilingual `contact_links` for Private Vulnerability Reporting (`https://github.com/Ezenitac-AiService/ImageViewer/security/advisories/new`) and the original upstream repository (`https://github.com/edith2580/ImageViewer/issues`).
  3. Retain existing bilingual intake forms (`.github/ISSUE_TEMPLATE/bug_report.yml`, `.github/ISSUE_TEMPLATE/feature_request.yml`), which already enforce a required `fork-confirmation` checkbox.
- **Rationale**:
  - Enabling public issues while Private Vulnerability Reporting is disabled violates `SECURITY.md` / `SECURITY.en.md`, which explicitly instruct reporters to use `Security -> Advisories -> Report a vulnerability`.
  - Disabling blank issues prevents users from bypassing the mandatory `fork-confirmation` checkbox and chooser guidance.
- **Alternatives Considered**:
  - *Keep blank issues enabled*: Rejected; allows unformatted reports, upstream Windows C#/.NET bug confusion, and accidental public security disclosures.
  - *Replace GitHub Private Vulnerability Reporting with an email contact*: Rejected; adds unmonitored email infrastructure and conflicts with the documented GitHub Security Advisories workflow.
- **Primary Evidence / Observed Output**:
  - `gh repo view --json hasIssuesEnabled,securityPolicyUrl` returned `"hasIssuesEnabled": false` and `"securityPolicyUrl": "https://github.com/Ezenitac-AiService/ImageViewer/security/policy"`.
  - `gh api repos/Ezenitac-AiService/ImageViewer/private-vulnerability-reporting` returned `{"enabled": false}`.
  - `.github/ISSUE_TEMPLATE/` currently contains only `bug_report.yml` (42 lines, `fork-confirmation` required) and `feature_request.yml` (30 lines, `fork-confirmation` required), with no `config.yml`.
  - `SECURITY.md` line 13 directs users to GitHub Private Vulnerability Reporting (`Security -> Advisories -> Report a vulnerability`).

---

## R2. Default-Branch Ruleset & Tiered CI Status Checks (`FR-004`–`FR-006`, Constitution v2.4.0)

- **Decision**:
  Configure a GitHub Repository Ruleset on `main` (`enforcement: active`, `target: branch`, `conditions.ref_name.include: ["~DEFAULT_BRANCH"]`) with:
  - `bypass_actors: []` (no default administrator bypass).
  - `deletion` rule and `non_fast_forward` rule (blocks branch deletion and force-pushes).
  - `pull_request` rule with `required_approving_review_count: 0`, `dismiss_stale_reviews_on_push: false`, `require_code_owner_review: false`, `require_last_push_approval: false`, `required_review_thread_resolution: false` (enforces PR-only entry and audit trail while permitting solo-maintainer self-merges).
  - `required_status_checks` rule with `strict_required_status_checks_policy: true` (requires branches to pass checks against the up-to-date `main` tip, eliminating concurrent-merge races) and **exactly 6 required check contexts**, each pinned to `"integration_id": 15368` (GitHub Actions App ID, preventing status spoofing per Constitution VII):
    1. `Checks (ubuntu-latest)`
    2. `Checks (macos-latest)`
    3. `Checks (windows-latest)`
    4. `Launch check (macOS)`
    5. `Launch check (Windows)`
    6. `Dependencies (cargo-deny, budget)`
  - Explicitly **exclude** `Launch check (Linux, experimental)` from `required_status_checks` so experimental Linux failures remain informational and non-blocking per Constitution v2.4.0 (`Target Platforms (Tiered)`).
- **Rationale**:
  - Rulesets support `required_approving_review_count: 0` alongside `pull_request`, `non_fast_forward`, `deletion`, and `strict_required_status_checks_policy: true` cleanly via REST API and allow zero-bypass (`bypass_actors: []`) configuration.
  - Using exact check-run names from the verified post-`004` workflow prevents silent mismatch or non-matching context hangs.
- **Alternatives Considered**:
  - *Classic Branch Protection API (`branches/main/protection`)*: Rejected; older API couples admin enforcement (`enforce_admins`) and review counts less cleanly than Repository Rulesets and is not used when configuring modern rulesets.
  - *Requiring `Launch check (Linux, experimental)`*: Rejected; directly violates Constitution v2.4.0 ("A launch verification failure on an experimental target MUST be reported but MUST NOT block publication") and `FR-005`.
  - *Setting `required_approving_review_count >= 1`*: Rejected; blocks solo maintainer merges on PRs (clarified in `Session 2026-11-23`).
- **Primary Evidence / Observed Output**:
  - `gh api repos/Ezenitac-AiService/ImageViewer/rulesets` returned `[]`; `gh api repos/Ezenitac-AiService/ImageViewer/branches/main/protection` returned HTTP 404 (`Branch not protected`).
  - `gh run view 44020931940 --json jobs --jq '.jobs[] | {name: .name, conclusion: .conclusion}'` on `004-launch-reliability-refactor` returned exact job check-run names: `Checks (macos-latest)`, `Checks (ubuntu-latest)`, `Checks (windows-latest)`, `Launch check (Linux, experimental)`, `Launch check (macOS)`, `Launch check (Windows)`, `Dependencies (cargo-deny, budget)` — all `success`.
  - GitHub Docs (Repository Rulesets): `pull_request.parameters.required_approving_review_count` accepts `0..10`; `required_status_checks.parameters.strict_required_status_checks_policy: true` enforces up-to-date branch verification before merge.

---

## R3. Safe Bootstrapping Sequence & Branch Cleanup (`FR-005`, `FR-011`, `FR-013`, `SC-005`)

- **Decision**:
  Execute consolidation and branch protection in strict sequence:
  1. Complete `004-launch-reliability-refactor` `T109` (`/speckit-analyze` consistency verification) before merging `004`.
  2. Open a Pull Request from `004-launch-reliability-refactor` → `main` (and optionally incorporate `005` governance artifacts/workflow fixes cleanly or in a follow-up PR on `main`), confirm all 7 CI jobs pass, and merge into `main`.
  3. Immediately activate the `main` Repository Ruleset referencing the 6 post-`004` required check contexts (`R2`).
  4. Enable `delete_branch_on_merge: true` via `PATCH /repos/Ezenitac-AiService/ImageViewer` so merged PR head branches are automatically deleted.
  5. Delete obsolete remote branch `002-rust-crossplatform-viewer` (and `004-launch-reliability-refactor` once merged if not auto-deleted) via maintainer-approved `git push origin --delete` / GitHub API, then execute `v1.1.0-alpha.2` draft release + `T110` clean-machine checklist from `main`.
- **Rationale**:
  - `main` currently carries the older `003`-era `.github/workflows/ci.yml` whose job names are `ubuntu-latest` / `macos-latest` / `windows-latest` (`verify` matrix) and `GPLv3 License Compatibility (cargo-deny)` (`licenses`), whereas `004` changed the job names to `Checks (...)`, `Launch check (...)`, and `Dependencies (cargo-deny, budget)`.
  - Activating a zero-bypass ruleset requiring the 6 post-`004` job names *before* `004` merges into `main` would deadlock any PR whose checks emit legacy job names or whose merge order conflicts.
- **Alternatives Considered**:
  - *Enable ruleset with legacy `main` check names first, merge `004`, then rewrite the ruleset*: Rejected; introduces unnecessary two-phase churn and breaks during the transition PR whose workflow emits post-`004` job names.
  - *Keep `delete_branch_on_merge: false` and rely solely on manual deletion*: Rejected per clarified `Session 2026-11-23` Q3/FR-011.
- **Primary Evidence / Observed Output**:
  - `git show main:.github/workflows/ci.yml` defines jobs `verify` (`name: ${{ matrix.os }}`) and `licenses` (`name: GPLv3 License Compatibility (cargo-deny)`).
  - `.github/workflows/ci.yml` on `004-launch-reliability-refactor` defines `checks`, `launch-macos`, `launch-windows`, `launch-linux`, `dependencies`.
  - `git rev-list --left-right --count main...004-launch-reliability-refactor` returned `0 15`.
  - `gh api repos/Ezenitac-AiService/ImageViewer/branches --jq '.[].name'` returned `002-rust-crossplatform-viewer`, `004-launch-reliability-refactor`, `main`.
  - `gh repo view --json deleteBranchOnMerge` returned `"deleteBranchOnMerge": false`.

---

## R4. Proactive Weekly Dependency, Advisory & License Audit (`FR-007`–`FR-008`, `SC-003`)

- **Decision**:
  Add a dedicated workflow `.github/workflows/dependency-audit.yml` triggered on:
  - `schedule`: `- cron: '17 3 * * 1'` (weekly Monday 03:17 UTC — off-peak minute to avoid thundering herd)
  - `workflow_dispatch`: `{}` (manual maintainer trigger prior to any release or after 60-day pause recovery)
  With strict least-privilege permissions (`permissions: { contents: read }`) and two matrix runs passing `arguments` before `command` (per `EmbarkStudios/cargo-deny-action/action.yml` arg order: `${{ inputs.arguments }} ${{ inputs.command }} ${{ inputs.command-arguments }}`):
  1. Shipped default configuration: `arguments: ""`, `command-arguments: advisories bans licenses sources` (`cargo deny check advisories bans licenses sources` using `deny.toml` `all-features = false`).
  2. Full workspace optional-feature configuration: `arguments: "--all-features"`, `command-arguments: advisories bans licenses sources` (`cargo deny --all-features check advisories bans licenses sources`).
  Do **not** grant `issues: write` or create automated GitHub issues; rely on workflow run failure notifications and already-enabled Dependabot vulnerability alerts (`HTTP 204`).
- **Rationale**:
  - Separates recurring advisory checks from heavy multi-OS GUI builds in `ci.yml`, saving macOS/Windows runner minutes while catching newly published RustSec advisories between commits.
  - Checking `--all-features` prevents silent bitrot/vulnerability accumulation in `updater`, `os-codecs`, `shell-registration`, and `extended-formats` features that CI compiles and tests in `Checks`.
  - Public GitHub repositories automatically pause `schedule` triggers after 60 days without repository activity; retaining active Dependabot vulnerability alerts covers passive dormancy, and `workflow_dispatch` plus workflow re-enablement ensures deterministic pre-release checks.
- **Alternatives Considered**:
  - *Add `schedule` directly inside `.github/workflows/ci.yml`*: Rejected; triggers expensive multi-OS Rust compilation and GUI launch checks every week when dependencies have not changed.
  - *Check only `deny.toml` default (`all-features = false`)*: Rejected; leaves `url` (`updater`) and `extended-formats` dependencies unmonitored until an `--all-features` promotion or audit.
  - *Create keep-alive dummy commits every 50 days*: Rejected; pollutes git history and triggers unnecessary CI/build runs.
- **Primary Evidence / Observed Output**:
  - `deny.toml` line 6 sets `all-features = false`.
  - `gh api repos/Ezenitac-AiService/ImageViewer/vulnerability-alerts -i` returned `HTTP/2.0 204 No Content` (Dependabot vulnerability alerts active).
  - `gh api repos/Ezenitac-AiService/ImageViewer/actions/permissions/workflow` returned `{"default_workflow_permissions": "read", "can_approve_pull_request_reviews": false}`.
  - `gh api 'repos/EmbarkStudios/cargo-deny-action/contents/action.yml?ref=17aeb3c839e42274e2b8adf721c8c65edfdc74dd'` confirmed `runs.args` order puts `inputs.arguments` before `inputs.command` and `inputs.command-arguments`, so `--all-features` MUST be supplied via `arguments`, not `command-arguments`.
  - GitHub Docs ("Disabling and enabling a workflow"): scheduled workflows in public repositories are automatically disabled after 60 days without repository activity.

---

## R5. Remediation of Pre-Existing `--all-features` RustSec Advisories (`FR-007`–`FR-008`, `SC-003`)

- **Decision**:
  Before enabling the `--all-features` audit step in `.github/workflows/dependency-audit.yml`, remediate the **8 directly observed vulnerabilities** currently failing `cargo deny --all-features check advisories`:
  1. Run `cargo update -p url` to upgrade `url` from `2.5.8` to `>=2.5.9` (resolves `RUSTSEC-2026-0592` triggered via `iv-platform`'s optional `updater` feature -> `reqwest 0.12.28`).
  2. Change `image-extras` in root `Cargo.toml` line 34 from `image-extras = "0.1"` to `image-extras = { version = "0.1", default-features = false, features = ["pcx"] }`, and refresh `Cargo.lock`. This removes unused default features (`dds`, `icns`, `ora`, `otb`, `sgi`, `xbm`, `xpm`, `wbmp`), eliminating `ora`'s transitive dependency on `zip v5.1.1` (resolving all 7 zip advisories: `RUSTSEC-2026-0367`, `0368`, `0369`, `0429`, `0545`, `0547`, `0548`).
- **Rationale**:
  - Running `cargo deny --all-features check advisories bans licenses sources` locally proved that `all-features` currently fails with 8 real advisories (`advisories FAILED`), so adding an `--all-features` job without fixing those 2 dependencies would fail on arrival.
  - Workspace code inspection confirmed `crates/iv-media/src/decode.rs:541` uses **only** `image_extras::pcx::PCXDecoder`, and `image-extras 0.1.1/Cargo.toml` cleanly isolates `pcx = ["dep:pcx"]` from `ora = ["image/png", "dep:zip", "dep:ouroboros"]`.
- **Alternatives Considered**:
  - *Ignore all 8 advisories in `deny.toml`*: Rejected under Constitution Principle VI/VII and `004` precedent (which explicitly upgraded `zip` to 9 to resolve zip advisories rather than ignoring them); narrowing `image-extras` to `features = ["pcx"]` removes the vulnerable transitive crate altogether and shrinks `--all-features` build tree size.
- **Primary Evidence / Observed Output**:
  - `cargo deny --all-features check advisories` failed with `RUSTSEC-2026-0367`, `0368`, `0369`, `0429`, `0545`, `0547`, `0548` (via `image-extras 0.1.1 -> zip 5.1.1`) and `RUSTSEC-2026-0592` (`url 2.5.8 -> upgrade to >=2.5.9` via `reqwest 0.12.28 -> iv-platform`).
  - `image-extras-0.1.1/Cargo.toml` defines `default = ["dds", "icns", "ora", "otb", "pcx", "sgi", "xbm", "xpm", "wbmp"]`, `ora = ["image/png", "dep:zip", "dep:ouroboros"]`, `pcx = ["dep:pcx"]`.
  - `grep -rn "image_extras" crates/ packaging/ docs/` returned a single match: `crates/iv-media/src/decode.rs:541: let dec = image_extras::pcx::PCXDecoder::new(Cursor::new(bytes))`.

---

## R6. CI & Release Supply-Chain Pinning & Least-Privilege (`FR-008`, `FR-013`–`FR-014`, `SC-007`)

- **Decision**:
  1. Pin every external GitHub Action across `.github/workflows/ci.yml`, `.github/workflows/release.yml`, and `.github/workflows/dependency-audit.yml` to its verified 40-character commit SHA for the already CI-tested `v4`/`v2`/`stable` series (`actions/checkout`, `upload-artifact`, `download-artifact`, `rust-cache`, `cargo-deny-action`, `action-gh-release`, `dtolnay/rust-toolchain`) with an inline version comment (e.g., `uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4`). Defer breaking major-version action upgrades (`actions/checkout@v7`, `upload-artifact@v7`, `download-artifact@v8`, `action-gh-release@v3`) to a separate, dedicated CI-verified PR after `005`.
  2. Enforce least-privilege workflow `permissions:` on `.github/workflows/ci.yml` (add explicit top-level `contents: read`) and `.github/workflows/release.yml` (narrow top-level `contents: write` down to `contents: read`, placing `permissions: { contents: write }` exclusively on the `publish-draft` job).
  3. Enable repository-level enforcement (`PUT /repos/Ezenitac-AiService/ImageViewer/actions/permissions` with `{"enabled": true, "allowed_actions": "all", "sha_pinning_required": true}`) under maintainer approval **after** all active workflow files on `main` use SHA-pinned `uses:` references, preventing workflow rejection on unpinned refs.
- **Rationale**:
  - Floating tags (`@v4`, `@v2`, `@stable`) can be retargeted upstream if a third-party maintainer account or release tag is compromised.
  - Narrowing `.github/workflows/release.yml` prevents build, launch-check, or cargo-deny steps from carrying repository write tokens unnecessarily.
  - Because `sha_pinning_required: true` blocks any workflow job referencing a mutable tag/branch, workflow file pinning MUST land in `ci.yml` and `release.yml` before toggling the repository policy bit.
  - Pinning the already green `v4`/`v2` line avoids mixing major Node/runner runtime upgrades into a governance hardening PR; major upgrades remain explicitly logged for follow-up.
- **Alternatives Considered**:
  - *Upgrade all actions to 2026 major tags (`v7`/`v8`/`v3`) simultaneously inside `005`*: Rejected to isolate governance hardening from breaking major-release changes (logged with measured latest versions below).
  - *Restrict `allowed_actions` to `selected` allowlist without SHA pinning*: Rejected; still allows mutable tag mutation within allowed repos and fails `FR-014`.
  - *Keep top-level `contents: write` across all 8 `release.yml` jobs*: Rejected under Constitution Principle VII (least privilege).
- **Primary Evidence / Observed Output**:
  - `.github/workflows/release.yml` lines 19–20 set top-level `permissions: { contents: write }` across all 8 jobs even though only `publish-draft` writes a draft release; `.github/workflows/ci.yml` omits an explicit `permissions:` block.
  - `gh api repos/Ezenitac-AiService/ImageViewer/actions/permissions` returned `{"enabled": true, "allowed_actions": "all", "sha_pinning_required": false}`.
  - `grep -rn "uses:" .github/workflows/` listed 7 unique actions across 45 occurrences (`ci.yml` + `release.yml`), all currently using mutable refs (`@v4`, `@v2`, `@stable`).
  - `gh api repos/.../releases/latest` (checked 2026-11-23) recorded newer upstream releases: `actions/checkout` `v7.0.1`, `actions/upload-artifact` `v7.0.2`, `actions/download-artifact` `v8.0.2`, `Swatinem/rust-cache` `v2.9.3`, `EmbarkStudios/cargo-deny-action` `v2.1.3`, `softprops/action-gh-release` `v3.0.4`.
  - Verified SHAs resolved from `repos/{owner}/{repo}/git/ref/...` via `gh api`:
    - `actions/checkout@v4` → `11d5960a326750d5838078e36cf38b85af677262`
    - `actions/upload-artifact@v4` → `ea165f8d65b6e75b540449e92b4886f43607fa02`
    - `actions/download-artifact@v4` → `d3f86a106a0bac45b974a628896c90dbdf5c8093`
    - `dtolnay/rust-toolchain@stable` → `39008827e6b4319a349e9a42dfc90579dd25f1ca`
    - `Swatinem/rust-cache@v2` → `d1527c8f67ef9af357290475d4af4b0fab8f12e0`
    - `EmbarkStudios/cargo-deny-action@v2` → `17aeb3c839e42274e2b8adf721c8c65edfdc74dd`
    - `softprops/action-gh-release@v2` → `3bb12739c298aeb8a4eeaf626c5b8d85266b0e65`
  - `gh api 'repos/dtolnay/rust-toolchain/contents/action.yml?ref=39008827e6b4319a349e9a42dfc90579dd25f1ca'` confirmed `inputs.toolchain` defaults to `stable` and dynamically installs the latest `stable` toolchain via `rustup`, so pinning `dtolnay/rust-toolchain` by commit SHA preserves rolling stable Rust compiler updates without freezing the compiler version.

---

## R7. Repository Metadata, Code Search & Attack Surface Hygiene (`FR-009`–`FR-010`, `FR-013`, `SC-004`, `SC-007`)

- **Decision**:
  1. Trim the trailing space in repository `description`: `"A fast, GPU-accelerated, Zero-Trust cross-platform image and comic archive viewer written in Rust"`.
  2. Disable unused public repository surfaces (`has_wiki: false`, `has_projects: false`) in the same maintainer-approved `PATCH /repos/Ezenitac-AiService/ImageViewer` call.
  3. Assign >= 5 accurate lowercase hyphenated GitHub topics (`PUT /repos/Ezenitac-AiService/ImageViewer/topics`): `["rust", "image-viewer", "comic-viewer", "cbz", "egui", "wgpu", "cross-platform", "zero-trust"]` (8 topics).
  4. Record the confirmed standalone code search status (60 results observed via `gh search code`) in `specs/005` verification artifacts and update `specs/003-independent-fork-separation` verification record if any open tracking note remains.
- **Rationale**:
  - `hasWikiEnabled: true` exposes a publicly writable wiki by default unless explicitly restricted or disabled; `hasProjectsEnabled: true` creates unused project tabs; neither is used.
  - `repositoryTopics: null` harms discoverability; trimmed description removes formatting defect.
- **Alternatives Considered**:
  - *Keep Wiki/Projects enabled for hypothetical future docs*: Rejected; documentation lives version-controlled under `README*`, `docs/`, and bilingual root files under Constitution Principle I.
- **Primary Evidence / Observed Output**:
  - `gh repo view --json description,repositoryTopics,hasWikiEnabled,hasProjectsEnabled` returned `"description": "...written in Rust "` (trailing space), `"repositoryTopics": null`, `"hasWikiEnabled": true`, `"hasProjectsEnabled": true`.
  - `gh search code --repo Ezenitac-AiService/ImageViewer "ImageViewer" --limit 3` returned `Showing 3 of 60 results`, proving standalone code search indexing is live.

---

## R8. Bilingual Root Documentation Reconciliation (`FR-012`, `SC-006`)

- **Decision**:
  Update `NOTICE` and `NOTICE.en.md` in lockstep (Constitution Principle I) to:
  1. Preserve the `2026-11-16` initial independent fork separation date, `edith2580` copyright, and `Ezenitac-AiService` copyright.
  2. Add the `2026-11-17` `004` modification notice matching `CHANGELOG.md` / `CHANGELOG.en.md` line 9 (`2026-11-16–17`).
  3. Reconcile the prominent modifications summary with actual `004` / `1.1.0-alpha.2` architecture: replace the stale `winit, wgpu, egui, zune-jpeg, image, fast_image_resize, moxcms, and jxl-oxide` + `auto-update endpoints` phrasing with accurate current behavior (`eframe`/`egui` + `wgpu`/`glow` fallback, `zune-jpeg`, `image`, `image-webp`, `tiff`, `moxcms`, `fast_image_resize`, `zip` read-only archive streaming; extended formats including `jxl-oxide` excluded by default and no runtime network/auto-update endpoints in `1.1.0-alpha.2`, while OS bundle identifier `com.ezenitac.imageviewer` and repo/issue/release workflows target `Ezenitac-AiService/ImageViewer`).
  4. Verify `README.md`/`README.en.md`, `CHANGELOG.md`/`CHANGELOG.en.md`, `SECURITY.md`/`SECURITY.en.md`, `CONTRIBUTING.md`/`CONTRIBUTING.en.md`, and `THIRD_PARTY_NOTICES.md`/`THIRD_PARTY_NOTICES.en.md` carry matching issue links, dates, and capability claims (and reflect any `Cargo.lock` dependency update from R5 if `THIRD_PARTY_NOTICES` mentions version/count).
- **Rationale**:
  - `NOTICE` and `NOTICE.en.md` currently claim `jxl-oxide` and active auto-update endpoints, contradicting `README.md`, `CHANGELOG.md`, `SECURITY.md`, and `docs/CAPABILITIES.md` produced in `004`.
- **Alternatives Considered**:
  - *Leave `NOTICE` untouched as a `2026-11-16` snapshot*: Rejected in `Session 2026-11-23` Q4 (Option A); GPLv3 §5(a) notices distributed with `1.1.0-alpha.2` must state current prominent modifications and dates truthfully.
- **Primary Evidence / Observed Output**:
  - `NOTICE` / `NOTICE.en.md` lines 22–33 list `2026-11-16` and mention `winit, wgpu, ... jxl-oxide` and `Isolated auto-update endpoints`.
  - `CHANGELOG.md` / `CHANGELOG.en.md` lines 9, 20 state `2026-11-16–17`, `eframe`, excluded extended formats, and no network connections.

---

## Final Implementation & Remote Verification Record (2026-11-23)

- **PR #1 (`004-launch-reliability-refactor` → `main`)**: All 14 checks passed; merged into `main` (`f7c76a5`).
- **PR #2 (`005-repo-governance-hardening` → `main`)**: All 14 checks passed under `Protect main` ruleset (`id: 27903505`); merged into `main` (`bbfc8c9`).
- **Live Ruleset (`Protect main`, `id: 27903505`)**: `enforcement: "active"`, `bypass_actors: []`, `deletion`, `non_fast_forward`, `pull_request` (`required_approving_review_count: 0`), `required_status_checks` (`strict_required_status_checks_policy: true`, exact 6 non-experimental contexts pinning `integration_id: 15368`, excluding `Launch check (Linux, experimental)`).
- **Live Intake & Supply Chain**: `private-vulnerability-reporting.enabled: true`, `hasIssuesEnabled: true`, `.github/ISSUE_TEMPLATE/config.yml` (`blank_issues_enabled: false` + bilingual PVR & upstream chooser links) live on `main`; `actions/permissions` enforces `sha_pinning_required: true`.
- **Live Dependency Audit (`dependency-audit.yml` run `45191568190`)**: `Cargo Deny (all-features)` (`success`, 28s) and `Cargo Deny (shipped-default)` (`success`, 35s); `vulnerability-alerts` returns `HTTP/2.0 204`.
- **Metadata, Surfaces & Branch Hygiene**: `description` trimmed with 0 trailing spaces; 8 topics (`cbz`, `comic-viewer`, `cross-platform`, `egui`, `image-viewer`, `rust`, `wgpu`, `zero-trust`); `hasWikiEnabled: false`; `hasProjectsEnabled: false`; `deleteBranchOnMerge: true`; remote branches `002-rust-crossplatform-viewer` and `004-launch-reliability-refactor` deleted (only `main` remains); `gh search code` returned 80 matching files.
- **Host vs. CI Verification Scope**: `cargo deny` (default & `--all-features`), `cargo fmt`, `cargo clippy` (default & `--all-features`), `cargo test` (default & `--all-features`), `component-count.sh` (`298`, budget `356`), YAML parsing, and 40-char SHA pinning verified natively on macOS host; Windows and Linux builds/launch checks verified on GitHub Actions runners (`PR #1` & `PR #2`). `v1.1.0-alpha.2` draft release and clean-machine checklist (`004` `T110`) remain ready to execute from `main`.
- **Phase 9 Convergence (`T044`–`T045`, 2026-11-24)**: Moved `TAG` into step `env:` with strict SemVer regex validation (`^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$`) in `.github/workflows/release.yml` (`publish-draft`, eliminating shell script injection under `FR-014`), added direct clickable Private Vulnerability Reporting links (`https://github.com/Ezenitac-AiService/ImageViewer/security/advisories/new`) in lockstep to `SECURITY.md` and `SECURITY.en.md` (`SC-001`), and verified `quickstart.md` §1 #5 untrusted-interpolation scan passes with 0 findings.
