# Quickstart & Validation Guide: Repository Governance Hardening

**Feature Branch**: `005-repo-governance-hardening`  
**Date**: 2026-11-23  
**Contracts**: [repository-governance-contract.md](./contracts/repository-governance-contract.md) | **Data Model**: [data-model.md](./data-model.md)

This guide covers local verification, maintainer-approved live execution order, remote audit checks (`SC-001`–`SC-007`), and recovery from the 60-day schedule inactivity pause.

---

## 1. Local Verification (Safe, Zero Remote Side Effects)

Run after all repository file changes (`Cargo.toml`, `Cargo.lock`, `.github/`, `NOTICE`, `NOTICE.en.md`, and any synced docs) are complete:

```bash
# 1. Dependency, advisory, ban, license, and source checks (both configurations)
cargo deny check advisories bans licenses sources
cargo deny --all-features check advisories bans licenses sources

# 2. Workspace lint and test checks (host OS; report unexecuted OS targets per Constitution)
cargo fmt --all -- --check
cargo check --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-features
cargo test --workspace

# 3. Verify 100% of external `uses:` lines in .github/workflows/ are pinned to 40-char SHAs
python3 - <<'PY'
import pathlib, re, sys
unpinned = []
for wf in sorted(pathlib.Path(".github/workflows").glob("*.yml")):
    for i, line in enumerate(wf.read_text().splitlines(), 1):
        m = re.search(r"uses:\s*([^\s#]+)", line)
        if m and not re.match(r"^[^@]+@[0-9a-f]{40}$", m.group(1)):
            unpinned.append(f"{wf}:{i}: {m.group(1)}")
if unpinned:
    print("UNPINNED ACTIONS:\n" + "\n".join(unpinned))
    sys.exit(1)
print("All workflow actions are pinned to 40-char commit SHAs.")
PY

# 4. Verify bilingual root documentation links and YAML validity
python3 - <<'PY'
import pathlib, yaml
for yml in pathlib.Path(".github").rglob("*.yml"):
    yaml.safe_load(yml.read_text())
print("All .github YAML files parse cleanly.")
PY
```

---

## 2. Maintainer-Approved Live Execution Order (`FR-005`, `FR-011`, `FR-013`)

Follow this exact sequence so no deadlock or broken security redirect occurs:

1. **Complete `004` `T109` & Land `004` onto `main`**: Run `/speckit-analyze` on `004-launch-reliability-refactor` (`T109`), then merge `004-launch-reliability-refactor` into `main` via PR (verifying all 7 post-`004` CI checks pass and establishing the updated check names on `main`).
2. **Activate `Protect main` Ruleset & Land `005` PR onto `main`**:
   - Create and activate the `Protect main` ruleset (§4.5, with `"integration_id": 15368`) immediately after `004` lands on `main`.
   - Rebase/branch `005-repo-governance-hardening` onto post-`004` `main`, open the `005` PR carrying `.github/ISSUE_TEMPLATE/config.yml`, `.github/workflows/dependency-audit.yml`, SHA-pinned `ci.yml` & `release.yml`, `Cargo.toml`/`Cargo.lock` advisory fixes, and `NOTICE`/`NOTICE.en.md` reconciliation, and merge once all 6 required checks pass.
3. **Apply remaining maintainer-approved repository settings** (using exact contracts in [repository-governance-contract.md](./contracts/repository-governance-contract.md) §4, ONLY after `.github/ISSUE_TEMPLATE/config.yml` and SHA-pinned workflows exist on `main`):
   - Enable Private Vulnerability Reporting **first** (§4.1).
   - Enable `has_issues: true`, trim `description`, set `has_wiki: false`, `has_projects: false`, set `delete_branch_on_merge: true` (§4.2).
   - Apply discovery topics (§4.3).
   - Enable `sha_pinning_required: true` (§4.4).
4. **Clean up obsolete remote branches & trigger initial scheduled audit check**:
   - Delete `002-rust-crossplatform-viewer` (and `004-launch-reliability-refactor` if still present after merge).
   - Manually trigger `gh workflow run dependency-audit.yml` (`workflow_dispatch`) on `main` and confirm both `shipped-default` and `all-features` pass.
5. **Continue `004` `T110` from post-`005` `main`**:
   - Tag/dispatch `v1.1.0-alpha.2` draft release from a post-`005` `main` commit (which contains the SHA-pinned `release.yml`) and run the clean-machine checklist (`specs/004-launch-reliability-refactor/quickstart.md` §4).

---

## 3. Read-Only Remote Verification Audit (`SC-001`–`SC-007`)

Run after Step 2 completes to confirm every success criterion:

```bash
# SC-001, SC-004, SC-005, SC-007 (Repo settings, PVR, topics, surfaces, branches)
gh repo view --json description,hasIssuesEnabled,hasWikiEnabled,hasProjectsEnabled,deleteBranchOnMerge,repositoryTopics
gh api repos/Ezenitac-AiService/ImageViewer/private-vulnerability-reporting
gh api repos/Ezenitac-AiService/ImageViewer/actions/permissions
gh api repos/Ezenitac-AiService/ImageViewer/branches --jq '.[].name'
gh search code --repo Ezenitac-AiService/ImageViewer "ImageViewer" --limit 3

# SC-002 (Ruleset enforcement, effective branch rules, & exact 6 required check contexts)
gh api repos/Ezenitac-AiService/ImageViewer/rulesets
gh api repos/Ezenitac-AiService/ImageViewer/rules/branches/main

# SC-003 (Manual dispatch verification & passive vulnerability alerts active)
gh api repos/Ezenitac-AiService/ImageViewer/vulnerability-alerts -i | head -n 5
gh workflow run dependency-audit.yml && gh run list --workflow dependency-audit.yml --limit 3
```

Expected results:
- `hasIssuesEnabled: true`, `private-vulnerability-reporting.enabled: true` (`SC-001`)
- `Protect main` ruleset `enforcement: "active"`, `bypass_actors: []`, `required_approving_review_count: 0`, `strict_required_status_checks_policy: true`, exact 6 non-experimental contexts (`SC-002`)
- `vulnerability-alerts` returns `HTTP/2.0 204` and manual `dependency-audit.yml` run succeeds on both matrix jobs (`SC-003`)
- Trimmed `description`, >= 5 topics, live `gh search code` hits (`SC-004`)
- Only `main` (and any active unmerged feature branch) remains (`SC-005`)
- `sha_pinning_required: true`, `hasWikiEnabled: false`, `hasProjectsEnabled: false` (`SC-007`)

---

## 4. 60-Day Schedule Inactivity Recovery Runbook (`SC-003`)

If the public repository receives no commit activity for 60 consecutive days, GitHub automatically disables `dependency-audit.yml` schedule runs. Before cutting any release after dormancy:

```bash
# Re-enable the scheduled workflow and trigger an immediate pre-release audit
gh workflow enable dependency-audit.yml
gh workflow run dependency-audit.yml
```
