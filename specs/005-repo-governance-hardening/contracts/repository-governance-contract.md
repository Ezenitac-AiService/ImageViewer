# Interface & Configuration Contracts: Repository Governance Hardening

**Feature Branch**: `005-repo-governance-hardening`  
**Date**: 2026-11-23  
**Spec**: [spec.md](../spec.md) | **Data Model**: [data-model.md](../data-model.md)

This document defines the exact schemas, workflow definitions, action pinning table, and maintainer-approved remote API payloads required to implement `005-repo-governance-hardening`.

---

## 1. Issue Chooser Configuration Contract (`.github/ISSUE_TEMPLATE/config.yml`)

Enforces `FR-002` and `FR-003` so all public issues pass through bilingual fork-confirming templates and security/upstream inquiries route away from public issues.

```yaml
blank_issues_enabled: false
contact_links:
  - name: 🔒 보안 취약점 비공개 제보 / Report a Security Vulnerability (Private)
    url: https://github.com/Ezenitac-AiService/ImageViewer/security/advisories/new
    about: 보안 취약점은 공개 이슈 대신 비공개 제보(Private Vulnerability Reporting)를 이용해 주세요. / Please report security vulnerabilities privately rather than opening a public issue.
  - name: ↗️ 원본 프로젝트(Windows C#/.NET v1.0.6) 문의 / Upstream Project (edith2580/ImageViewer)
    url: https://github.com/edith2580/ImageViewer/issues
    about: 크로스 플랫폼 Rust 포크가 아닌 원본 Windows C#/.NET 버전에 관한 이슈는 원작자 저장소에 등록해 주세요. / For issues specific to the original Windows C#/.NET version, please open an issue upstream.
```

---

## 2. Scheduled Dependency Audit Workflow Contract (`.github/workflows/dependency-audit.yml`)

Enforces `FR-007`, `FR-008`, and `FR-014`.

```yaml
name: Dependency Audit

on:
  schedule:
    # Weekly on Monday at 03:17 UTC (active while repository has activity within 60 days).
    - cron: '17 3 * * 1'
  workflow_dispatch: {}

permissions:
  contents: read

jobs:
  cargo-deny:
    name: Cargo Deny (${{ matrix.scope }})
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        include:
          - scope: shipped-default
            cargo-deny-args: ""
          - scope: all-features
            cargo-deny-args: "--all-features"
    steps:
      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
      - uses: EmbarkStudios/cargo-deny-action@17aeb3c839e42274e2b8adf721c8c65edfdc74dd # v2
        with:
          command: check
          arguments: ${{ matrix.cargo-deny-args }}
          command-arguments: advisories bans licenses sources
```

**Contract Guarantees**:
- Top-level `permissions` is strictly `contents: read` (zero `issues: write`, `contents: write`, or `pull-requests: write`).
- Wait-free execution of both `shipped-default` (`deny.toml` `all-features = false`) and `all-features` (`--all-features`).
- Failure surfaces strictly via failed run exit status and native GitHub Actions notifications; no automated issues are created.

---

## 3. Workflow Action SHA-Pinning & Least-Privilege Contract (`FR-008`, `FR-014`, `SC-007`)

1. **Least-Privilege Workflow Permissions (Constitution VII)**:
   - `.github/workflows/ci.yml`: MUST declare explicit top-level `permissions: { contents: read }` (currently omitted).
   - `.github/workflows/release.yml`: MUST narrow top-level `permissions` from `contents: write` to `contents: read` and scope `permissions: { contents: write }` strictly to the `publish-draft` job only, leaving the 7 build/verify/deny jobs strictly read-only.
   - `.github/workflows/dependency-audit.yml`: MUST keep top-level `permissions: { contents: read }`.
2. **Commit-SHA Pinning**: Every `uses:` directive across `.github/workflows/ci.yml`, `.github/workflows/release.yml`, and `.github/workflows/dependency-audit.yml` MUST use the verified 40-character commit SHA below before `sha_pinning_required: true` is activated:

| Action Reference (Current Tag) | Verified 40-Character Commit SHA | Inline Comment |
|---|---|---|
| `actions/checkout@v4` | `11d5960a326750d5838078e36cf38b85af677262` | `# v4` |
| `actions/upload-artifact@v4` | `ea165f8d65b6e75b540449e92b4886f43607fa02` | `# v4` |
| `actions/download-artifact@v4` | `d3f86a106a0bac45b974a628896c90dbdf5c8093` | `# v4` |
| `dtolnay/rust-toolchain@stable` | `39008827e6b4319a349e9a42dfc90579dd25f1ca` | `# stable` |
| `Swatinem/rust-cache@v2` | `d1527c8f67ef9af357290475d4af4b0fab8f12e0` | `# v2` |
| `EmbarkStudios/cargo-deny-action@v2` | `17aeb3c839e42274e2b8adf721c8c65edfdc74dd` | `# v2` |
| `softprops/action-gh-release@v2` | `3bb12739c298aeb8a4eeaf626c5b8d85266b0e65` | `# v2` |

---

## 4. Maintainer-Approved Remote Mutation Contracts (`FR-013`)

**Approval & Fail-Closed Gate**: No mutating remote call below may execute without explicit maintainer confirmation in the session. If administrative permissions are absent (HTTP 403/404) or confirmation is withheld, execution MUST halt cleanly and present the exact UI/CLI steps for manual maintainer application.

### 4.1 Enable Private Vulnerability Reporting (`FR-003`) — MUST precede or accompany `4.2`

- **Method & Path**: `PUT repos/Ezenitac-AiService/ImageViewer/private-vulnerability-reporting`
- **Verification**: `GET repos/Ezenitac-AiService/ImageViewer/private-vulnerability-reporting` returns `{"enabled": true}`.

### 4.2 Repository Settings & Surface Minimization (`FR-001`, `FR-009`, `FR-011`)

- **Method & Path**: `PATCH repos/Ezenitac-AiService/ImageViewer`
- **Payload**:
```json
{
  "description": "A fast, GPU-accelerated, Zero-Trust cross-platform image and comic archive viewer written in Rust",
  "has_issues": true,
  "has_wiki": false,
  "has_projects": false,
  "delete_branch_on_merge": true
}
```

### 4.3 Repository Discovery Topics (`FR-009`, `SC-004`)

- **Method & Path**: `PUT repos/Ezenitac-AiService/ImageViewer/topics`
- **Payload**:
```json
{
  "names": [
    "rust",
    "image-viewer",
    "comic-viewer",
    "cbz",
    "egui",
    "wgpu",
    "cross-platform",
    "zero-trust"
  ]
}
```

### 4.4 Actions SHA-Pinning Enforcement (`FR-014`, `SC-007`) — Execute ONLY after all default-branch workflows are SHA-pinned

- **Method & Path**: `PUT repos/Ezenitac-AiService/ImageViewer/actions/permissions`
- **Payload**:
```json
{
  "enabled": true,
  "allowed_actions": "all",
  "sha_pinning_required": true
}
```

### 4.5 Default-Branch Ruleset (`FR-004`–`FR-006`, `SC-002`) — Execute ONLY after `004-launch-reliability-refactor` lands on `main`

- **Method & Path**: `POST repos/Ezenitac-AiService/ImageViewer/rulesets`
- **Payload**:
```json
{
  "name": "Protect main",
  "target": "branch",
  "enforcement": "active",
  "bypass_actors": [],
  "conditions": {
    "ref_name": {
      "include": ["~DEFAULT_BRANCH"],
      "exclude": []
    }
  },
  "rules": [
    { "type": "deletion" },
    { "type": "non_fast_forward" },
    {
      "type": "pull_request",
      "parameters": {
        "required_approving_review_count": 0,
        "dismiss_stale_reviews_on_push": false,
        "require_code_owner_review": false,
        "require_last_push_approval": false,
        "required_review_thread_resolution": false
      }
    },
    {
      "type": "required_status_checks",
      "parameters": {
        "strict_required_status_checks_policy": true,
        "do_not_enforce_on_create": false,
        "required_status_checks": [
          { "context": "Checks (ubuntu-latest)", "integration_id": 15368 },
          { "context": "Checks (macos-latest)", "integration_id": 15368 },
          { "context": "Checks (windows-latest)", "integration_id": 15368 },
          { "context": "Launch check (macOS)", "integration_id": 15368 },
          { "context": "Launch check (Windows)", "integration_id": 15368 },
          { "context": "Dependencies (cargo-deny, budget)", "integration_id": 15368 }
        ]
      }
    }
  ]
}
```

> **Invariant**: `Launch check (Linux, experimental)` is intentionally omitted from `required_status_checks` so experimental Linux failures remain informational and non-blocking (Constitution v2.4.0).
