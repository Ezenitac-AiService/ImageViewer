# Quickstart & Verification Guide: Independent Project Separation (`003`)

**Feature**: `003-independent-fork-separation`
**Spec**: [spec.md](spec.md)
**Contract**: [separation-and-compliance-contract.md](contracts/separation-and-compliance-contract.md)

---

## 1. Automated Code, License & Identity Audits

```bash
# 1. Verify zero placeholder or upstream runtime URLs remain
grep -rn "ImageViewer/ImageViewer\|com\.imageviewer\.app" Cargo.toml crates/ packaging/ .github/

# 2. Verify Cargo license metadata & workspace checks
grep -n 'license = "GPL-3.0-only"' Cargo.toml
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 3. Verify third-party dependency license compatibility (if cargo-deny is installed)
cargo deny check licenses
```

## 2. Documentation & Etiquette Verification

- Confirm presence and content of:
  - `LICENSE` (GPLv3), `NOTICE`, `CHANGELOG.md`, `THIRD_PARTY_NOTICES.md`, `deny.toml`
  - `README.md` (cross-platform guide + top-level `edith2580/ImageViewer` attribution & issue-routing banner)
  - `CONTRIBUTING.md`, `SECURITY.md`, `.github/ISSUE_TEMPLATE/bug_report.yml`, `.github/ISSUE_TEMPLATE/feature_request.yml`
  - `specs/001-architecture-security-refactoring/spec.md` (`Archived` notice)

## 3. Repository Detach & Release Cutover Checklist

1. Create and push immutable archive tag `legacy-v1.0.6` on the pre-cutover C# commit (`960b627`).
2. Merge feature branch (`003-independent-fork-separation` / `002-rust-crossplatform-viewer`) into `main`.
3. Remove local `upstream` remote (`git remote remove upstream`).
4. Confirm `.github/workflows/ci.yml` passes across `ubuntu-latest`, `macos-latest`, and `windows-latest`.
5. Submit GitHub Support request to detach `Ezenitac-AiService/ImageViewer` from the upstream fork network ("leave fork network").
