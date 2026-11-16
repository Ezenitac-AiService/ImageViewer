# Quickstart: Validating Launch Reliability

**Spec**: [spec.md](./spec.md) | **Contracts**: [contracts/](./contracts/)

This guide shows how to prove the feature works end to end. It is a validation guide, not an
implementation guide.

## 0. Prerequisites

- Rust ≥ 1.95 with targets `aarch64-apple-darwin` and `x86_64-apple-darwin` (macOS host) or
  `x86_64-pc-windows-msvc` (Windows host).
- `cargo-deny` ≥ 0.20.
- macOS: Xcode command-line tools (`codesign`, `lipo`, `iconutil`, `hdiutil`, `swift`).
- Windows: PowerShell 7.

## 1. Library and Interaction Checks (any host)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace
cargo deny check advisories bans licenses sources
```

**Expected**: everything passes, with zero `#[ignore]`. The `interaction` tests cover every row of
[interaction-contract.md](./contracts/interaction-contract.md), and the acceptance tests cover every
row of [input-acceptance-contract.md](./contracts/input-acceptance-contract.md) §4.

**Shipped-component budget**:

```bash
cargo tree -p iv-app -e normal --target all --prefix none | sed 's/ (\*)//' | sort -u | wc -l
```

**Expected**: ≤ 411 (baseline 514).

## 2. Local Launch Check

macOS:

```bash
./packaging/macos/package-macos.sh
```

```bash
./packaging/verify/launch-check.sh target/release/ImageViewer.app packaging/verify/reference.png
```

Windows (PowerShell):

```bash
pwsh packaging/windows/package-windows.ps1
```

```bash
pwsh packaging/verify/launch-check.ps1 dist/ImageViewer-windows-x64.exe packaging/verify/reference.png
```

**Expected**: the script prints a `pass` Launch Verification Record (data-model §2), and the
diagnostic record contains `phase=start`, `phase=probe`, `phase=window`,
`phase=first_frame source=reference.png`, and `exit=clean` for one `launch=` id.

## 3. Failure-Visibility Checks

| Scenario | How to provoke | Expected |
|----------|----------------|----------|
| Damaged settings | Write 70 KB of `x` into `settings.json` | App starts; reset notice; `outcome=RecoveredFromDamage` |
| Unwritable settings | Make the config directory read-only | App starts; unreadable notice; `outcome=Unreadable` |
| Missing launch file | Pass a path that does not exist | Window shows the localized not-found status; no exit |
| Unsupported format | Open a `.qoi` file, or random bytes renamed `.jpg` | Status "not supported in this version" |
| No GPU renderer | Windows VM without a GPU driver, or an RDP session (spike S1) | `renderer=wgpu type=cpu` or `renderer=glow`; image displayed |
| Fatal path | Clean-machine only: a VM with display acceleration disabled and no WARP | Error dialog naming the cause and the log folder; exit code 2 |

## 4. Clean-Machine Checklist (manual, before every release)

Run on clean Windows 10 and Windows 11 VMs (fresh image, no Visual C++ runtime, no developer
tools), on a real Apple-Silicon Mac, and on an Intel Mac if one is available. Use the **downloaded
draft-release asset**, opened through a browser so the quarantine / Mark-of-the-Web applies.

1. Follow only `docs/FIRST_LAUNCH.md` (or `.en.md`). Time from download complete to visible window
   must be ≤ 2 minutes (SC-001). Screenshot every trust dialog.
2. Confirm that no "damaged", "corrupt", or missing-component message appears (SC-002).
3. Windows: confirm there is no console window, and that the title bar never shows
   "(응답 없음)" / "(Not Responding)" (SC-012).
4. **Mouse-only session** (SC-005): open a folder of 100 images, step through 20, zoom in, drag to
   pan, change fit mode, rotate, toggle fullscreen, open Settings, change one value, press OK, then
   quit. Every control must respond on first click.
5. **Keyboard-only session**: repeat step 4 using keys only (interaction contract §2). Esc must
   close an open dialog before anything else.
6. Open a ZIP/CBZ, and drag-and-drop a file onto the window.
7. Open with the OS: macOS Finder → Open With → ImageViewer, both while the app is closed and while
   it is running; Windows Explorer → Open with.
8. Korean system: on first launch the UI is Korean. Switch to English in Settings.
9. Large folder: open a folder of 10,000 images. The first image appears within 1 s and the UI stays
   responsive (SC-006).
10. macOS only, run from the disk image without copying: double-click `ImageViewer.app` inside the
    mounted DMG (App Translocation applies). It must launch, display an image, save a changed
    setting, and quit cleanly. After relaunching from `/Applications`, that setting must still be
    there (spec edge case "opened directly from inside its archive or disk image").
11. **Responsiveness sample (SC-004)**: record the screen at ≥ 60 fps while clicking at least 10
    distinct controls across the main window and every dialog, with the mouse only. For each click,
    count the frames from the pointer-down frame to the first frame showing the control's effect. Every
    control must respond within 6 frames (= 100 ms at 60 fps). Note the slowest control and its
    frame count.
12. Record results in `launch-verification-manual.md` (one row per environment, data-model §2) and
    attach it to the draft release before publishing.

## 5. Release Rehearsal

1. Push a tag `vX.Y.Z-rc.N` to a fork, or run `release.yml` with `workflow_dispatch`.
2. **Expected**: the draft release contains the three assets, `SHA256SUMS.txt`, three blocking JSON
   records, and the Linux record, and the release notes contain the platform table.
3. Corrupt the macOS bundle on purpose (for example, skip the signing step in a branch). **Expected**:
   the build fails `codesign --verify`, and no draft is created (User Story 4 scenario 2).
