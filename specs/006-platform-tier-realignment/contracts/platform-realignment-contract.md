# Interface & Configuration Contracts: Platform Tier Realignment (`v3.0.0`)

**Feature Branch**: `006-platform-tier-realignment`  
**Date**: 2026-11-24  
**Spec**: [spec.md](../spec.md) | **Data Model**: [data-model.md](../data-model.md)

This contract defines the exact API payloads, workflow job schemas, release-note templates, issue-form selectors, and draft-regeneration commands for `006-platform-tier-realignment`.

---

## 1. Maintainer-Approved `Protect main` Ruleset Update (`FR-007`, `FR-012`, `SC-004`)

**Execution Order Invariant**: Execute **before** merging the `006` PR whose `.github/workflows/ci.yml` removes `windows-latest` and `launch-windows`, preventing merge deadlock under `bypass_actors: []`.

- **Method & Path**: `PUT repos/Ezenitac-AiService/ImageViewer/rulesets/27903505`
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
          { "context": "Launch check (macOS)", "integration_id": 15368 },
          { "context": "Dependencies (cargo-deny, budget)", "integration_id": 15368 }
        ]
      }
    }
  ]
}
```

---

## 2. CI & Release Workflow Contracts (`FR-001`–`FR-005`, `SC-001`–`SC-003`)

All external Actions remain pinned to the verified 40-character commit SHAs established in `005` (`repository-governance-contract.md` §3), with explicit `permissions: { contents: read }` at workflow top level and `permissions: { contents: write }` scoped strictly to `publish-draft`.

### 2.1 `.github/workflows/ci.yml` Job Graph
- `checks`: `matrix.os: [ubuntu-latest, macos-latest]` (names `Checks (ubuntu-latest)` and `Checks (macos-latest)`). Runs Linux `apt-get` GUI dependencies + `cargo fmt` on Linux, and `clippy` (both `--all-features` and default), `test` (`--all-features` and default), and `cargo bench --no-run` on both `ubuntu-latest` and `macos-latest`.
- `launch-macos`: `name: Launch check (macOS)` on `macos-latest` — builds universal bundle (`aarch64-apple-darwin`, `x86_64-apple-darwin`), runs `packaging/verify/launch-check.sh`, uploads `launch-verification-macos-arm64.json`.
- `dependencies`: `name: Dependencies (cargo-deny, budget)` on `ubuntu-latest` — runs `cargo deny check advisories bans licenses sources` and `packaging/verify/component-count.sh` (3 active targets).
- **Removed jobs**: `checks` (`windows-latest`), `launch-windows`, `launch-linux` (`Launch check (Linux, experimental)`).

### 2.2 `.github/workflows/release.yml` Job Graph
- `build-macos`: `macos-latest` universal DMG → uploads `asset-macos` (`target/release/ImageViewer-macos-universal.dmg`).
- `build-linux`: `ubuntu-latest` tarball (`packaging/linux/package-linux.sh`) → uploads `asset-linux` (`dist/ImageViewer-linux-x64.tar.gz`).
- `verify-macos`: `needs: build-macos`, matrix `{macos-latest -> macos-arm64, macos-15-intel -> macos-x64}` → mounts DMG, runs `launch-check.sh`, uploads `record-${{ matrix.record }}`.
- `deny`: `ubuntu-latest` → `cargo deny check advisories bans licenses sources`.
- `publish-draft`: `needs: [verify-macos, build-linux, deny]`, `if: always() && needs.verify-macos.result == 'success' && needs.build-linux.result == 'success' && needs.deny.result == 'success'`, `permissions: { contents: write }`.
  - Preserves `Checksums and release notes` step `env.TAG` + Bash SemVer guard (`^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$`).
  - Computes `sha256sum ImageViewer-macos-universal.dmg ImageViewer-linux-x64.tar.gz > SHA256SUMS.txt`.
  - Attaches strictly: `ImageViewer-macos-universal.dmg`, `ImageViewer-linux-x64.tar.gz`, `SHA256SUMS.txt`, and `launch-verification-macos-*.json` (`draft: true`).
- **Removed jobs**: `build-windows`, `verify-windows`, `verify-linux`.

---

## 3. Release Notes Template & Renderer Contract (`FR-006`, `SC-002`–`SC-003`)

`packaging/release-notes.md.in` and `packaging/verify/render-release-notes.py` MUST omit Windows rows/Smart App Control text and render exact `v3.0.0` + `006`/`007` platform disclosure:

```markdown
### 검증된 플랫폼 / Verified platforms

| Platform | Arch | Tier | Automated | Clean device | Status |
|----------|------|------|-----------|--------------|--------|
| macOS | Apple Silicon (`arm64`) | Top priority | @MAC_ARM_AUTO@ | @MAC_ARM@ | @MAC_ARM_STATUS@ |
| macOS | Intel (`x86_64`) | Top priority | @MAC_X64_AUTO@ | @MAC_X64@ | @MAC_X64_STATUS@ |
| iOS | `arm64` | Top priority (planned — feature `007`) | n/a | n/a | planned (not in this release) |
| Linux | `x86_64` | Lower priority (build-only) | n/a (no runtime verification) | n/a | build-only, unverified |
| Android | `arm64` | Lower priority (planned — feature `007`) | n/a (no runtime verification) | n/a | planned build-only (not in this release) |
```

- `render-release-notes.py` reads `macos-arm64` and `macos-x64` records only; zero `windows-x64` or `linux-x64` keys are queried.
- `Known limitations` replaces Windows Smart App Control and experimental Linux notes with accurate `macOS` Gatekeeper guidance and explicit `Linux` build-only/unverified + `Windows` excluded notices.

---

## 4. Bilingual Bug Report OS Selector Contract (`.github/ISSUE_TEMPLATE/bug_report.yml`, `FR-009`)

```yaml
  - type: dropdown
    id: os
    attributes:
      label: 운영체제 / Operating System
      options:
        - macOS (Apple Silicon / Intel)
        - Linux (하위 우선순위·빌드 전용·미검증 / Lower Priority, Build-Only, Unverified)
        - iOS (지원 예정 — v3.0.0 / Planned — Not Yet Released)
        - Android (빌드 전용 예정 — v3.0.0 / Planned Build-Only — Not Yet Released)
    validations:
      required: true
```

---

## 5. Pre-Amendment `v1.1.0-alpha.2` Draft Regeneration & Metadata Contract (`FR-002`, `FR-010`, `FR-012`)

1. **Draft Disposal & Regeneration (ONLY after `006` merges to `main`, under maintainer approval `FR-012`)**:
   ```bash
   gh release delete v1.1.0-alpha.2 --yes
   gh workflow run release.yml --ref main -f tag=v1.1.0-alpha.2
   ```
   Verify `gh release view v1.1.0-alpha.2 --json isDraft,assets` shows `isDraft: true` and assets contain zero `.exe` files and zero `launch-verification-linux-*.json` / `launch-verification-windows-*.json` files.
2. **Optional Metadata Topic Audit (`FR-010`, maintainer-approved)**:
   Verify `description` (`description == description.trim()`, zero Windows claims) and `topics` (`len >= 5`, e.g., include `macos` alongside `rust`, `image-viewer`, `comic-viewer`, `cbz`, `egui`, `wgpu`, `cross-platform`, `zero-trust`).
