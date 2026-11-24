# Quickstart & Validation Guide: Platform Tier Realignment (`v3.0.0`)

**Feature Branch**: `006-platform-tier-realignment`  
**Date**: 2026-11-24  
**Contracts**: [platform-realignment-contract.md](./contracts/platform-realignment-contract.md) | **Data Model**: [data-model.md](./data-model.md)

This guide defines local verification checks, deadlock-safe maintainer-approved live execution order, and remote audit commands validating `SC-001`–`SC-006`.

---

## 1. Local Verification (macOS Host, Safe Side-Effect-Free Checks)

Run after all repository edits (`.specify/memory/constitution.md`, `Cargo.toml`, `Cargo.lock`, `crates/`, `packaging/`, `.github/`, and bilingual docs) are complete:

```bash
# 1. Code formatting, clippy, and test suites (both configurations)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-features
cargo test --workspace

# 2. Dependency compliance and 3-target shipped-component budget
cargo deny check advisories bans licenses sources
cargo deny --all-features check advisories bans licenses sources
./packaging/verify/component-count.sh

# 3. Verify 0 Windows packaging scripts, 0 winresource/windows/winreg Cargo references,
#    and preserved OS-agnostic Zero-Trust guards (Q4 + Q5)
python3 - <<'PY'
import pathlib, re, sys, yaml

assert not pathlib.Path("packaging/windows").exists()
assert not pathlib.Path("packaging/verify/assert-windows-exe.ps1").exists()
assert not pathlib.Path("packaging/verify/launch-check.ps1").exists()
assert not pathlib.Path("crates/iv-app/build.rs").exists()

for cf in ["Cargo.toml", "crates/iv-app/Cargo.toml", "crates/iv-platform/Cargo.toml"]:
    t = pathlib.Path(cf).read_text()
    assert not re.search(r"\b(winresource|windows|winreg)\b", t), f"Windows dep left in {cf}"

# OS-agnostic Zero-Trust backslash Zip-Slip & path redaction must remain intact
ps = pathlib.Path("crates/iv-core/src/path_security.rs").read_text()
assert "raw.replace('\\\\', \"/\")" in ps
assert "source_keeps_file_name_only_for_unix_and_windows_paths" in pathlib.Path("crates/iv-core/tests/diagnostics_tests.rs").read_text()

# Workflows: valid YAML, 40-char SHA pinned, 0 Windows jobs, 0 Linux runtime checks, 0 unsafe run: interpolations
sha_re = re.compile(r"^[^@]+@[0-9a-f]{40}$")
unsafe_re = re.compile(r"\$\{\{\s*github\.(?:event\.|ref_name|head_ref|base_ref)[^}]*\}\}")
for yml in sorted(pathlib.Path(".github").rglob("*.yml")):
    yaml.safe_load(yml.read_text())
for wf in sorted(pathlib.Path(".github/workflows").glob("*.yml")):
    txt = wf.read_text()
    data = yaml.safe_load(txt)
    for m in re.findall(r"uses:\s*([^\s#]+)", txt):
        assert sha_re.match(m), f"Unpinned in {wf}: {m}"
    for jid, job in data.get("jobs", {}).items():
        assert "windows" not in jid and jid not in ("launch-linux", "verify-linux"), f"Disallowed job {wf}:{jid}"
        for idx, step in enumerate(job.get("steps", [])):
            assert not unsafe_re.findall(step.get("run") or ""), f"Unsafe interpolation {wf}:{jid}[{idx}]"

# Bug report OS dropdown must omit Windows
bug = yaml.safe_load(pathlib.Path(".github/ISSUE_TEMPLATE/bug_report.yml").read_text())
os_opts = next(b["attributes"]["options"] for b in bug["body"] if b.get("id") == "os")
assert not any("windows" in o.lower() for o in os_opts), os_opts

# Constitution v3.0.0 must have temporary Sync Impact Report removed
const_txt = pathlib.Path(".specify/memory/constitution.md").read_text()
assert const_txt.startswith("# ImageViewer Constitution")
assert "**Version**: 3.0.0" in const_txt
print("All local 006 invariants verified.")
PY

# 4. Bilingual link integrity across all 8 root & docs pairs
python3 - <<'PY'
import pathlib, re
pairs = [
    ("README.md", "README.en.md"),
    ("SECURITY.md", "SECURITY.en.md"),
    ("NOTICE", "NOTICE.en.md"),
    ("CHANGELOG.md", "CHANGELOG.en.md"),
    ("CONTRIBUTING.md", "CONTRIBUTING.en.md"),
    ("THIRD_PARTY_NOTICES.md", "THIRD_PARTY_NOTICES.en.md"),
    ("docs/CAPABILITIES.md", "docs/CAPABILITIES.en.md"),
    ("docs/FIRST_LAUNCH.md", "docs/FIRST_LAUNCH.en.md"),
]
for ko, en in pairs:
    kt, et = pathlib.Path(ko).read_text(), pathlib.Path(en).read_text()
    assert pathlib.Path(en).name in kt and pathlib.Path(ko).name in et, f"Broken switcher: {ko}/{en}"
    for f, txt in [(ko, kt), (en, et)]:
        for m in re.findall(r"\]\(([^)#]+)", txt):
            if not m.startswith(("http://", "https://", "mailto:")):
                assert (pathlib.Path(f).parent / m).resolve().exists(), f"Missing link {f}->{m}"
print("All 8 bilingual pairs pass cross-link checks.")
PY

# 5. Test release notes renderer with macOS-only mock inputs in temp dir
python3 - <<'PY'
import json, pathlib, subprocess, tempfile
with tempfile.TemporaryDirectory() as td:
    d = pathlib.Path(td)
    for k in ("macos-arm64", "macos-x64"):
        (d / f"launch-verification-{k}.json").write_text(json.dumps({"result": "pass"}))
    out = d / "notes.md"
    subprocess.check_call(["python3", "packaging/verify/render-release-notes.py", "1.1.0-alpha.2", str(d), str(out)])
    txt = out.read_text()
    assert "Windows" not in txt or "excluded" in txt.lower()
    assert "build-only" in txt.lower()
print("render-release-notes.py verified.")
PY
```

---

## 2. Maintainer-Approved Live Execution Order (`FR-007`, `FR-010`, `FR-012`)

1. **Create non-`NNN-*` branch** (e.g., `chore/006-platform-realignment`) so `ci.yml` executes a single `pull_request` workflow run without duplicate `push` runs.
2. **Update `Protect main` Ruleset FIRST** (`T015`, explicit maintainer approval required per `FR-012`, contract §1):
   - Apply `PUT repos/Ezenitac-AiService/ImageViewer/rulesets/27903505` with the exact 4 required contexts (`Checks (ubuntu-latest)`, `Checks (macos-latest)`, `Launch check (macOS)`, `Dependencies (cargo-deny, budget)`).
3. **Open PR targeting `main` & merge** (`T031`): verify PR passes all 4 required checks (`gh pr checks <num> --watch`), merge (`gh pr merge <num> --merge`), switch to `main`, pull latest `main`, and prune the temporary branch.
4. **Update repository topics (optional if adding `macos`, maintainer-approved)** per contract §5.2.
5. **Delete & regenerate `v1.1.0-alpha.2` Draft Release** (`T033`, explicit maintainer approval per `FR-012` + `Q2`):
   - `gh release delete v1.1.0-alpha.2 --yes`
   - `gh workflow run release.yml --ref main -f tag=v1.1.0-alpha.2`
   - Watch release run cleanly, verify draft assets (`ImageViewer-macos-universal.dmg`, `ImageViewer-linux-x64.tar.gz`, `SHA256SUMS.txt`, `launch-verification-macos-*.json` only), and run Apple Silicon local DMG check (`codesign`, `lipo`, `launch-check.sh`) updating maintainer `macOS` clean-machine checklist scaffold.

---

## 3. Read-Only Remote Verification Audit (`SC-001`–`SC-006`)

```bash
gh api repos/Ezenitac-AiService/ImageViewer/rulesets/27903505 --jq '{id, enforcement, bypass_actors, contexts: [.rules[] | select(.type=="required_status_checks") | .parameters.required_status_checks[] | "\(.context)@\(.integration_id)"]}'
gh release view v1.1.0-alpha.2 --json isDraft,isPrerelease,assets,body
gh api repos/Ezenitac-AiService/ImageViewer/branches --jq '[.[].name]'
gh repo view --json description,repositoryTopics
```
