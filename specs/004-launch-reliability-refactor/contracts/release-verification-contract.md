# Contract: Packaging, Launch Verification & Release

**Covers**: FR-001–FR-006, FR-026–FR-030, FR-035; SC-001, SC-002, SC-007, SC-012 |
**Research**: R2, R6, R11, R12, R13, R15

## 1. Artifacts

| Asset | Built on | Contents | Signature |
|-------|----------|----------|-----------|
| `ImageViewer-windows-x64.exe` | `windows-latest` | GUI-subsystem, static-CRT executable with icon, version, and manifest | none (FR-005 hook: `WINDOWS_SIGNING_*` secrets → `signtool`) |
| `ImageViewer-macos-universal.dmg` | `macos-latest` | `ImageViewer.app` containing a universal `aarch64` + `x86_64` binary, generated `Info.plist`, `AppIcon.icns`; DMG with an `/Applications` link | ad-hoc, bundle-level; `MACOS_SIGNING_IDENTITY` → Developer ID plus hardened runtime |
| `ImageViewer-linux-x64.tar.gz` | `ubuntu-latest` | Binary plus `.desktop` file; labelled experimental | none |
| `SHA256SUMS.txt` | publish job | sha256 of the three assets | — |
| `launch-verification-*.json` | verify jobs | data-model §2 records | — |

**Build-time assertions** (the build fails if any is false):

- macOS: `codesign --verify --deep --strict` exits 0; `lipo -archs` prints both `x86_64` and
  `arm64`; `plutil -lint` passes; `CFBundleShortVersionString` equals the workspace version.
- Windows: PE subsystem = 2 (`WINDOWS_GUI`); imports contain no `VCRUNTIME` or
  `api-ms-win-crt-`; `FileVersion` equals the workspace version.
- All: packaging scripts carry the git executable bit (no `chmod` in CI).

## 2. Automated Launch Check (`packaging/verify/`)

Inputs: path to the artifact (extracted or mounted) and `packaging/verify/reference.png`, a small
committed fixture.

| Step | Windows (`launch-check.ps1`) | macOS (`launch-check.sh` + `window-probe.swift`) |
|------|------------------------------|-----------------------------------------------|
| Start | `Start-Process exe reference.png -PassThru` | `open -n -W ImageViewer.app --args reference.png &`, or the bundle executable directly |
| Window | `MainWindowHandle ≠ 0` within 30 s | `CGWindowList` shows an on-screen window owned by the pid |
| First frame | Diagnostic record contains `phase=first_frame source=reference.png` | Same |
| Responsive | `Responding` true for 20 samples over 10 s | — (CGWindowList cannot report hangs; covered by the clean-machine check) |
| Console | No `conhost` child process | — |
| Close | `CloseMainWindow()` | `osascript -e 'tell application id "com.ezenitac.imageviewer" to quit'` |
| Exit | Exit code 0 within 10 s, and `exit=clean` present | Same |

- **Jobs**: `verify-windows`, `verify-macos-arm64` (`macos-latest`), `verify-macos-x64`
  (`macos-15-intel`, launching the x86_64 slice), and `verify-linux` under `xvfb-run` with
  `continue-on-error: true`.
- **Output**: one Launch Verification Record JSON per job, uploaded as an artifact.
- In `ci.yml`, the same check runs on every PR against a non-packaged release build (FR-028).

## 3. Interaction Step (automated, Windows and macOS)

After the first frame, with a two-image fixture folder: send the Right Arrow key to the window via
the OS (PowerShell `SendKeys` / `osascript` System Events, where the runner permits it) and expect
a `phase=navigate index=1` line within 1 s.

If the runner forbids synthetic input `[UNVERIFIED]` (spike S3), this step moves to the
clean-machine checklist and the record notes it.

## 4. Release Gate (`release.yml`)

```text
build-{windows,macos,linux}
  → verify-{windows,macos-arm64,macos-x64}  (blocking)
  → verify-linux                            (non-blocking)
  → deny (cargo deny check advisories bans licenses sources)
  → publish-draft (needs: all blocking verify jobs + deny)
```

- `publish-draft` creates a **draft** release containing the assets, `SHA256SUMS.txt`, the JSON
  records, and release notes generated from `packaging/release-notes.md.in`.
- The maintainer runs the clean-machine checklist ([quickstart.md](../quickstart.md) §4), attaches
  `launch-verification-manual.md`, and only then publishes. No workflow path publishes a
  non-draft release.

**Release-notes platform table** (FR-029; required, filled from the records):

| Platform | Arch | Automated | Clean machine | Status |
|----------|------|-----------|---------------|--------|
| Windows 10 | x64 | pass / fail | pass / fail / not run | supported / unverified |
| Windows 11 | x64 | … | … | … |
| macOS | Apple Silicon | … | … | … |
| macOS | Intel | … | … | … |
| Linux | x64 | … | n/a | experimental |

The notes also list the known limitations: the one-time trust confirmation, Smart App Control
blocking, and Unavailable capabilities.

## 5. CI on Every Change (`ci.yml`)

`fmt` → `clippy -D warnings` (workspace, all targets, all features) → `test --workspace
--all-features` → `test --workspace` (default features, i.e. the shipped configuration) → launch
check on Windows and macOS → `cargo deny check` (all sections) → shipped-component count reported,
failing if it exceeds 411 (research R13).
