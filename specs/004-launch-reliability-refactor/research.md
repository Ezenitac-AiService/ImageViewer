# Phase 0 Research: Launch Reliability & Evidence-Driven Refactoring

**Date**: 2026-11-16 | **Spec**: [spec.md](./spec.md) | **Review**: [feasibility-review.md](./feasibility-review.md)

Every decision below follows Constitution Principle V. Evidence is tagged:

- `[OBSERVED]` — measured on the review host (macOS 27.2.1, Apple Silicon, `rustc 1.100.0`).
- `[SOURCE]` — read from crate source in the local Cargo registry or from cited documentation.
- `[UNVERIFIED]` — inference or secondhand; the listed **spike** must confirm it before dependent
  tasks are marked complete.

## R1. Application shell: adopt `eframe`, delete the hand-written integration

- **Decision**: Replace `crates/iv-app/src/main.rs` (custom `winit::ApplicationHandler`) and
  `renderer.rs` (custom `wgpu` pipeline) with `eframe 0.37` (`egui 0.37`, `egui-wgpu 0.37`,
  `wgpu 30`). Enable features `wgpu` and `glow`; disable `system_fonts` and `web_screen_reader`;
  keep `default_fonts`, `clipboard`, `x11`, `wayland`. `accesskit` is decided by the dependency
  budget (R13).
- **Rationale**:
  - Every interaction defect in the review (A4–A8, C1, C4) lives in the hand-written glue that
    eframe already provides: input forwarding, repaint-on-input scheduling, surface lifecycle, DPI
    changes, and native window decorations.
  - eframe 0.37 still uses `winit 0.30.13` on native targets `[SOURCE]` (`eframe-0.37.0/Cargo.toml`),
    so platform behaviour knowledge carries over.
  - eframe exposes a `Renderer` choice between `Wgpu` and `Glow` when both features are enabled
    `[SOURCE]` (`eframe-0.37.0/src/epi.rs:615-660`), which gives the fallback path R4 needs.
  - The pinned `wgpu 24` pulls `block 0.1.6`, which the compiler already flags as future-incompatible
    `[OBSERVED]`. Moving to `wgpu 30` is expected to drop it `[UNVERIFIED — check with cargo tree]`.
- **Consequences**:
  - Workspace `rust-version` rises from `1.85` to `1.95` (eframe and egui-wgpu 0.37 declare
    `rust-version: 1.95` `[SOURCE]` `cargo info`).
  - Image display moves from a custom shader to egui textures painted as meshes with UV transforms
    for rotate and flip. Filter modes map to texture sampling: `None` → nearest, `Smooth` → linear.
    `SmoothSharpen` (the GPU unsharp kernel, finding C6) is not Core; it is classified
    **Unavailable** in the capability register until re-admitted.
  - Images larger than the GPU's `max_texture_side` are shown downscaled at fit, and split into
    multiple textures (tiles) when zoomed past fit, replacing `upload_tiled_rgba8`.
  - Settings, About and Image List become `egui::Window`s inside the main viewport instead of
    separate OS windows. This removes multi-surface management (A5) and makes "topmost dialog"
    well defined for FR-015.
- **Alternatives considered**:
  - *Repair the custom shell* (review option 2): keeps ~2,000 lines of owned glue that has no tests
    and caused every reported Windows symptom; rejected.
  - *Upgrade to `winit 0.31`*: still beta (`0.31.0-beta.4`) `[OBSERVED]` `cargo search`; rejected.
  - *Different toolkit* (Slint, Iced, Tauri): discards working egui views and adds migration risk
    with no evidence it fixes more; rejected.

## R2. Windows: no console window, no runtime prerequisite

- **Decision**:
  1. Add `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]` to the `iv-app`
     binary root.
  2. Add `.cargo/config.toml` with `[target.x86_64-pc-windows-msvc] rustflags = ["-C",
     "target-feature=+crt-static"]`.
  3. Embed icon, version resource (from `CARGO_PKG_VERSION`), and an application manifest
     (per-monitor DPI awareness v2) with the `winresource` build-dependency (`0.1.34`
     `[OBSERVED]` `cargo search`). It is build-time only and not shipped.
- **Rationale**:
  - The user-reported Windows symptom includes a console window whose closure ends the program.
    The binary currently has no subsystem attribute, so it links as a console program `[SOURCE]`
    (no `windows_subsystem` in the tree). The Rust Reference defines `windows_subsystem = "windows"`
    as detaching the program from a console; it is ignored on non-Windows targets
    ([Rust Reference: runtime](https://doc.rust-lang.org/reference/runtime.html),
    [RFC 1665](https://rust-lang.github.io/rfcs/1665-windows-subsystem.html)).
  - `crt-static` makes the C runtime statically linked on MSVC
    ([Rust Reference: linkage](https://doc.rust-lang.org/reference/linkage.html)), removing
    candidate cause 2 (missing Visual C++ runtime) on clean machines. The user report suggests it
    was not the primary cause, but FR-001 requires zero prerequisites.
- **Consequence**: With no console, `stderr` output is invisible. Fatal startup errors MUST
  therefore go through the dialog and diagnostic record (R5). This is the reason R2 and R5 must
  land together.
- **Alternatives**: Ship the VC++ redistributable inside an installer: needs the installer, which
  is out of scope (R11); rejected.

## R3. Repaint and responsiveness model

- **Decision**: Drive repaint entirely through egui. Remove the unconditional 16 ms
  `ControlFlow::WaitUntil` loop. Background work (decode, folder scan, watcher) holds a clone of
  `egui::Context` and calls `request_repaint()` on completion. Animation uses
  `request_repaint_after(frame_delay)`.
- **Rationale**: Findings A4–A6 and C4: the custom loop polls every 16 ms yet never repaints on
  pointer input. egui-winit's input handling returns a repaint request for input events and
  eframe acts on it `[SOURCE]` (eframe native run loop), so pointer input repaints without
  application code.
- **Verification**: User Story 3 scenarios 1 and 9; SC-004; SC-012 (Windows `Responding` check in
  [contracts/release-verification-contract.md](./contracts/release-verification-contract.md)).

## R4. Rendering fallback without re-creating the event loop

- **Decision**: Before `eframe::run_native`, run a **renderer probe** on the main thread:
  1. Create a `wgpu::Instance` with the platform's primary backends and enumerate adapters (no
     surface needed).
  2. If any adapter exists, use `Renderer::Wgpu` with a `native_adapter_selector` that prefers
     discrete, then integrated, then virtual, then CPU (software) adapters, and records the choice.
  3. If none exists, use `Renderer::Glow` (OpenGL).
  4. If `run_native` still returns an error, show the fatal dialog (R5) and exit non-zero.
- **Rationale**:
  - eframe has no automatic wgpu → glow fallback `[SOURCE]` (no fallback logic in
    `eframe-0.37.0/src/native/run.rs`), and a winit event loop cannot be created twice in one
    process, so "try wgpu, then retry glow" inside `run_native` is not possible. A probe before the
    event loop exists avoids that.
  - `egui-wgpu 0.37` exposes `WgpuSetupCreateNew::native_adapter_selector`, a closure over the
    enumerated adapters `[SOURCE]` (`egui-wgpu-0.37.0/src/setup.rs:154-203`).
  - On Windows, WARP is Microsoft's software rasterizer, available to Direct3D 12 since Windows 10
    1709 ([Microsoft: WARP](https://learn.microsoft.com/windows/win32/direct3darticles/directx-warp)),
    and wgpu's DX12 backend enumerates it `[SOURCE]`
    ([wgpu-hal dxgi factory](https://docs.rs/crate/wgpu-hal/29.0.3/source/src/auxil/dxgi/factory.rs)).
- **Spike S1** `[UNVERIFIED]`: confirm on a Windows VM with no GPU driver and over Remote Desktop
  that the selector receives a CPU adapter and the viewer renders.
- **Alternatives**: Always use Glow: loses the better default path; macOS OpenGL is deprecated.
  Rejected.

## R5. Failure visibility: diagnostic record, fatal dialog, panic hook

- **Decision**:
  - Start the diagnostic logger first, before settings, probe, or window creation.
  - Write synchronous **launch records** (see the
    [launch-and-diagnostics contract](./contracts/launch-and-diagnostics-contract.md)):
    version, OS, architecture, chosen renderer and adapter, each startup phase, first frame
    presented, clean exit.
  - Install a panic hook that writes the panic location to the record, flushes it, and shows the
    fatal dialog.
  - Fatal dialog: `rfd::MessageDialog` (already a dependency), localized, naming the cause category
    and the log folder in redacted form (`%LOCALAPPDATA%\ImageViewer\logs`,
    `~/Library/Application Support/…/logs`).
  - Process exit code: `0` clean, `2` fatal startup failure, `3` panic.
  - Replace `let _ =` / `.ok()` discards in the app with either a status message or a record line
    (FR-010).
- **Rationale**: Findings A2, A3, D7. The logger exists but `log_info`/`log_error` have no call
  sites `[OBSERVED]`, and the log file was 0 bytes after a run `[OBSERVED]`.
- **Constitution fit**: Path redaction and CR/LF stripping stay in `PathRedactor` (Principle VII).
  The launch records are ordinary production diagnostics, not test-only behaviour (Principle IV);
  R12 relies on them.

## R6. macOS packaging: valid signature and universal binary

- **Decision**: Replace `packaging/macos/package-dmg.sh` with `package-macos.sh`, which:
  1. builds `aarch64-apple-darwin` and `x86_64-apple-darwin` and merges them with `lipo`;
  2. generates `Info.plist` from the template, substituting `CFBundleShortVersionString` and
     `CFBundleVersion` from `cargo metadata` (FR-006) and adding `LSMinimumSystemVersion` `11.0`
     (the binary's `minos` `[OBSERVED]` `otool`);
  3. builds `AppIcon.icns` from `ImageViewer.png` with `sips` and `iconutil`;
  4. signs the **assembled** bundle: `codesign --force --deep --sign - ImageViewer.app` (ad-hoc,
     per clarification Q1), using `--sign "$MACOS_SIGNING_IDENTITY"` plus `--options runtime` when
     that variable is set (FR-005);
  5. verifies with `codesign --verify --deep --strict` and fails the build on error;
  6. creates the DMG with an `/Applications` link.
  Commit packaging scripts with the executable bit.
- **Rationale**:
  - The current bundle fails `codesign --verify --deep --strict` with "code has no resources but
    signature indicates they must be present" `[OBSERVED]`, because the binary keeps only its
    linker signature after being copied into the bundle. Re-signing the bundle after assembly is the
    documented fix ([Apple Developer Forums 740728](https://developer.apple.com/forums/thread/740728);
    [Apple TN3126](https://developer.apple.com/documentation/technotes/tn3126-inside-code-signing-hashes)).
  - Apple Silicon requires every executable to carry at least an ad-hoc signature
    ([bitcoin/bitcoin#22546](https://mirror.b10c.me/bitcoin-bitcoin/22546/)).
  - The current asset is `arm64` only `[OBSERVED]` `lipo -archs`, so Intel Macs cannot start it.
- **Residual**: A quarantined ad-hoc app is still not notarized. The user sees a one-time
  "cannot verify" warning and must allow it under System Settings → Privacy & Security. The exact
  macOS 27 wording is `[UNVERIFIED]`; the clean-machine check captures it for the first-launch guide.
- **Alternatives**: `cargo-bundle` / `cargo-packager`: reported to miss the bundle-signing step
  ([users.rust-lang.org 132678](https://users.rust-lang.org/t/how-to-create-macos-bundles-for-apple-silicon-they-always-seem-broken/132678));
  a script is clearer to audit. Rejected for now.

## R7. macOS "open with" and Finder double-click

- **Decision**: Implement an `NSApplicationDelegate` in `iv-platform` (objc2, already a
  dependency) that handles `application:openURLs:` and forwards file paths through a
  `Box<dyn Fn(Vec<PathBuf>) + Send + Sync>` callback supplied by the app. The app's callback pushes
  them onto a channel and calls `ctx.request_repaint()`. Remove the empty
  `install_macos_open_url_handler` (finding A9, Principle IV).
- **Rationale**: winit 0.30.13 states that it never registers an application delegate and that
  applications may register their own to receive `application:openURLs:`, which may require
  running as a bundle `[SOURCE]`
  ([winit 0.30 platform/macos.rs](https://docs.rs/crate/winit/latest/source/src/platform/macos.rs)).
  Historical winit issue
  [#1751](https://github.com/rust-windowing/winit/issues/1751) confirms there is no built-in event.
- **Spike S2** — **result (2026-11-17): the decision above did not hold and was changed.**
  winit 0.30.13 *does* register its own `WinitApplicationDelegate` in `EventLoop::new` and panics
  in `ApplicationDelegate::get` if the delegate is replaced `[SOURCE]`
  (`winit-0.30.13/src/platform_impl/macos/event_loop.rs:240`, `app_state.rs:176-185`). The doc
  comment quoted above contradicts the 0.30.13 code. A separate delegate installed before the loop
  was overwritten (no files delivered) and made `quit` return "user cancelled (-128)" `[OBSERVED]`.
  **Implemented instead**: `main` creates the winit event loop itself and passes it to
  `eframe::create_native`; between creation and `run_app`, `iv_platform::macos_open_files::attach`
  adds `application:openURLs:` to the class of winit's delegate with `class_addMethod`, which only
  adds the method when the class lacks it. Result `[OBSERVED]` on the bundled app:
  cold `open -a ImageViewer.app a.png` → `first_frame source=a.png`; warm `open -a … reference.png`
  → `navigate … source=reference.png` in the same process; `quit` → `exit=clean`. The `unsafe` blocks it needs MUST carry `// SAFETY:` comments (Principle IV).
- **Windows**: "Open with" passes the path as `argv[1]`; the existing CLI path covers it.

## R8. Input acceptance: content allowlist and pre-read size cap

- **Decision**:
  - Add `FormatGuard::identify(&[u8]) -> Option<CoreFormat>`, an explicit allowlist of leading
    signatures for Core formats: JPEG `FF D8 FF`; PNG `89 50 4E 47 0D 0A 1A 0A`; GIF `GIF87a` /
    `GIF89a`; WebP `RIFF????WEBP`; BMP `BM`; TIFF `II*\0` / `MM\0*`; ZIP `PK\x03\x04` or
    `PK\x05\x06`.
  - Choose the decoder from the identified format, never from the extension.
  - Keep the existing script/document denylist as a second layer.
  - Unidentified content → the localized "not supported in this version" message (spec US5
    scenario 3).
  - `FolderSource::read_entry_bytes` checks `metadata.len()` against the cap before reading, then
    reads through `Read::take(cap + 1)` to defend against the file growing between check and read.
- **Rationale**: Findings D1 and D2. Constitution VII requires verification "via leading binary
  signatures before decoding". `image::guess_format` is documented as best-effort and covers more
  formats than Core ([docs.rs image::guess_format](https://docs.rs/image/0.24.8/image/fn.guess_format.html)),
  so an owned table makes the allowlist explicit and testable. Tests cross-check that every table
  entry agrees with `guess_format`.

## R9. Scope tiers in code

- **Decision**:
  - `iv-media` gains Cargo features: default = Core formats; `extended-formats` adds JXL, DDS, PSD,
    PCX, CUR, QOI, PNM, HDR, EXR, TGA and their crates.
  - `iv-platform` gains features `updater` (current `update.rs`, `reqwest`, `rustls`, `sha2`) and
    `os-codecs` (current `codecs.rs`, `windows`, `objc2-image-io`, `objc2-core-graphics`). Both are
    off by default and not enabled by `iv-app`.
  - Shell registration (`--register-shell`, `winreg`) is removed from the app, and the switches
    are rejected with the existing "unsupported switch" message.
  - The thumbnail strip, the GPU sharpen mode, and the update notice are removed from the UI.
  - CI runs `cargo test --workspace --all-features` so withdrawn code stays tested.
- **Rationale**: FR-022, FR-024, FR-034, FR-037. Features keep the withdrawn work compiling and
  tested without shipping it, and FR-023 re-admission becomes a one-line change plus evidence.
- **Constitution fit**: These are product-scope features, not test bypasses. They neither weaken a
  check nor exist only to pass tests (Principle IV). The temporary narrowing of the constitution's
  *Media Decoding Strategy* is recorded in plan.md → Complexity Tracking.

## R10. Locale, settings outcome, and folder behaviour

- **Locale**: `sys-locale 0.3.2` (MIT/Apache-2.0, 1Password, `rust-version 1.56` `[SOURCE]` `cargo
  info`) reads the OS locale. `Language::from_locale` (already written, no caller) maps it. The
  locale is applied only when `settings.json` has no `language` key, so a returning user's choice
  wins.
- **Settings load outcome**: `SettingsStore::load` returns `(ViewerSettings, LoadOutcome)` where
  `LoadOutcome ∈ {Loaded, CreatedDefaults, RecoveredFromDamage, Unreadable}`; save returns a
  result the UI shows (FR-019, US2 scenario 4).
- **Folder scan off the UI thread**: Enumerate on a worker using `DirEntry::file_type()` (no extra
  `stat` per entry on most platforms). When a file was targeted, show it immediately and insert the
  sorted list when the scan completes (SC-006). Size and date for Image List sorting are fetched
  lazily.
- **Watcher keeps position**: On change, re-scan, then re-select the current entry by its source
  key. If it was removed, select the entry now at the same index, clamped (FR-016, finding A10).

## R11. Windows distribution format

- **Decision**: Keep a single portable `ImageViewer-windows-x64.exe` asset. Do not wire the NSIS
  installer this release.
- **Rationale**: File-association registration is Unavailable (FR-024), and an installer adds a
  second artifact to verify without serving a Core capability. Mark-of-the-Web applies to
  downloaded `.exe` and `.zip` alike, so wrapping changes nothing for SmartScreen.
- **First-launch guide**: Documents SmartScreen "More info → Run anyway", and that Smart App Control
  can block unsigned apps outright (spec Assumptions residual limitation).

## R12. Automated launch check without test-only behaviour

- **Decision**: The launch check is an external script that drives the real artifact:
  1. start it with the reference image path as `argv[1]`;
  2. wait up to 30 s for the `first_frame_presented` record (R5);
  3. assert the process is alive and a window exists. On Windows, also assert `MainWindowHandle ≠
     0`, `Responding = true` sampled for 10 s (SC-012), and PE subsystem `WINDOWS_GUI` (no
     console). On macOS, use a `CGWindowList` probe, which the review already used `[OBSERVED]`;
  4. request close the way a user would: `CloseMainWindow()` on Windows, a `quit` Apple Event on
     macOS;
  5. assert exit code `0` and an `exit=clean` record.
- **Rationale**: Constitution IV forbids flags written only to pass tests, so the check uses OS
  close requests and production diagnostics. A `quit` Apple Event is what Dock → Quit sends; if it
  does not exit cleanly, that is a product defect to fix, not a test problem.
- **Spike S3** `[UNVERIFIED]`: confirm that eframe/winit 0.30 exits cleanly on a `quit` Apple
  Event, and that GitHub-hosted Windows runners expose an interactive desktop where
  `MainWindowHandle` and `Responding` are meaningful.
- **Runners**: `windows-latest`; `macos-latest` (arm64) for the arm64 slice; `macos-15-intel` for
  the x86_64 slice. An Intel image was announced for macOS 15
  ([runner-images release notes](https://newreleases.io/project/github/actions/runner-images/release/macos-15%2F20251216.0081)),
  and GitHub reportedly ends macOS x64 support when macOS 15 runners retire in autumn 2027
  ([JDK-8377375](https://bugs.openjdk.org/browse/JDK-8377375)) `[UNVERIFIED — secondhand]`.
  Linux runs under `xvfb-run` as an experimental job (`continue-on-error: true`).
- **Not clean**: Hosted runners carry developer runtimes, so they cannot prove FR-001 alone. The
  clean-machine check (FR-027 second layer) covers that gap.

## R13. Dependency budget and vulnerability gate

- **Metric**: *shipped components* = unique packages (including the four workspace crates) in
  `cargo tree -p iv-app -e normal --target <t>` unioned over the four shipped desktop targets
  `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, `x86_64-apple-darwin`,
  `x86_64-unknown-linux-gnu` (`packaging/verify/component-count.sh`).
- **Baseline**: **446** `[OBSERVED]`, measured at commit `9f400dd` (before this feature) with the
  same script. *Correction (2026-11-17)*: the first plan used `--target all`, which also counts
  wasm-only packages that are never shipped (for example `tokio` via `wasm-bindgen-futures`) and
  gave 514. The spec's original 579 was the whole `Cargo.lock`.
- **Target**: ≤ 356 (a 20 % reduction) with all Core capabilities present.
- **Expected removals from the shipped tree**: `tokio` (used only to wait on one dialog future;
  replaced by `pollster`, already a dependency), `reqwest` / `rustls` / `ring` / `hyper` family,
  `sha2`, `winreg`, `windows` (WIC), `objc2-image-io` / `objc2-core-graphics`, long-tail decoders,
  direct `arboard` (egui-winit's `clipboard` feature still provides clipboard access), duplicate
  `objc2 0.5` / `windows 0.58` majors where the upgrade removes them.
- **Expected additions**: eframe, glow/glutin, `sys-locale`; `accesskit` only if the budget allows.
  If including it breaks the budget, it is excluded and listed in the capability register as an
  accessibility gap.
- **Vulnerability gate**: Extend `deny.toml` with `[advisories]` (deny unresolved, explicit ignore
  list with reasons), `[bans]` (warn on multiple versions), and `[sources]` (crates.io only). CI and
  release run `cargo deny check` for all four sections (FR-035, finding D3).
- **SC-010 note (resolved 2026-11-16, analysis finding I1)**: The spec originally cited 579, the
  full `Cargo.lock` count including packages the product does not ship. SC-010 now states the
  shipped-components metric above (baseline 446, target ≤ 356).

## R14. Interaction rules testable without a screen

- **Decision**: Add `iv-core::interaction` with a pure function
  `map_input(InputEvent, &InteractionContext) -> Vec<ViewerCommand>` and
  `ViewerModel::apply(ViewerCommand)`. The egui app translates egui input into `InputEvent`,
  calls `map_input`, applies commands, then renders from `ViewerModel`. The full table is in the
  [interaction contract](./contracts/interaction-contract.md).
- **Rationale**: FR-036 and finding E7. Keyboard, mouse and Esc semantics become unit-testable
  under Constitution III, using real state assertions and no mocks.

## R15. Release flow with the manual layer

- **Decision**: The release workflow runs build → automated launch check (blocking for Windows,
  macOS arm64, and macOS x86_64; non-blocking for Linux) → `cargo deny` → **draft** GitHub release
  with artifacts, `SHA256SUMS.txt`, per-target launch-verification JSON, and a release-notes
  platform table. A maintainer runs the clean-machine checklist
  ([quickstart.md](./quickstart.md) §4), attaches the record, and publishes manually.
- **Rationale**: FR-027 (two layers), FR-029, and clarification Q4. A draft release is the
  enforcement point for the manual layer without new repository settings.

## Spikes to Run First

| Spike | Question | Blocks |
|-------|----------|--------|
| S0 | Reproduce the reported Windows symptom on clean Windows 10 and 11 with the `v1.1.0-alpha.1` asset; record whether Windows marks the window "not responding" | Confirms the R2 + R3 diagnosis |
| S1 | Does the adapter selector get a CPU adapter (WARP) in a GPU-less VM and over RDP? | R4 |
| S2 | Does a custom `NSApplicationDelegate` coexist with eframe and receive cold- and warm-launch files? | R7 |
| S3 | Does a `quit` Apple Event exit cleanly? Do hosted Windows runners give a usable desktop for `Responding` checks? | R12 |

## Spike Results & Implementation Evidence

Recorded during `/speckit-implement` on the review host (macOS 27.2.1, Apple M5, `rustc 1.100.0`).

### S0 / S1 (Windows) — not run

No Windows host is available to this session. T007 and T008 stay open and must be run on clean
Windows 10/11 VMs before release (the clean-machine check covers the same ground).

### eframe shell (T019–T022)

- `target/debug/iv-app packaging/verify/reference.png` → one on-screen 1280×800 window
  (`CGWindowList`), idle CPU **0.0 %** (custom shell: 0.4–1.2 %), RSS **171 MB** (custom shell:
  203 MB) `[OBSERVED]`.
- First run found a real panic: `epaint::Mesh::colored_vertex` asserts that the mesh has no
  texture. The panic aborted the process with no diagnostic record, which is the US2 failure mode.
  The canvas now pushes textured vertices directly, and the panic hook (T041) records any future
  panic.
- Synthetic keyboard input via `osascript` System Events is refused on this host ("not allowed to
  send keystrokes", error 1002) `[OBSERVED]`. The first-click toolbar check in T022 therefore
  stays a manual step.

### US2 evidence (T048)

Diagnostic record per scenario `[OBSERVED]`:

| Scenario | Record |
|----------|--------|
| Normal launch with `reference.png` | `start` → `settings outcome=loaded` → `probe renderer=wgpu adapter="Apple M5" type=integrated backend=metal` → `window` → `first_frame source=reference.png w=96 h=96` |
| `settings.json` replaced by 70 KB of `x` | `settings outcome=recovered_from_damage`; window shown; `first_frame source=none` |
| CLI path that does not exist | window shown; `first_frame source=none` (status message shown in-window) |
| Settings directory `chmod 555` | `settings outcome=unreadable`; window shown |

### Defect found by T035: settings save race (pre-existing since v1.1.0-alpha.1)

`SettingsStore::locked_merge_write` deleted orphaned `settings.*.tmp` files **before** taking the
`fs4` lock, so one instance could delete another instance's in-flight temp file and fail its
rename. Running the concurrent-save test binary 48 times in parallel failed **15/48** before the
fix and **0/48** after moving the cleanup inside the lock `[OBSERVED]`.

### US1 evidence (T030, T034)

- `packaging/macos/package-macos.sh`: universal binary (`x86_64 arm64`), all five
  `assert-macos-bundle.sh` checks pass; the old bundle failed codesign, universal, and icon.
- Quarantined copy: `codesign --verify --deep --strict` passes; `syspolicy_check` no longer reports
  the codesign *Fatal* error. Remaining: ad-hoc warning and "notarization ticket not stapled",
  as expected without credentials (FR-002, FR-005). `spctl` rejects the copy, which is the one-time
  "Open Anyway" confirmation.
- Bundle launch + `osascript … to quit` → `exit=clean` within 1 s (spike S3, macOS part).
- The x86_64 slice could not be executed on this host (Rosetta not installed); it is unverified
  until the `macos-15-intel` CI job runs.
- `assert-windows-exe.ps1` and `package-windows.ps1` were not run: no PowerShell or Windows host.

### T098 result: shipped components 446 → 296 (−33.6 %)

Measured with `packaging/verify/component-count.sh` `[OBSERVED]`:

| Step | Count | Main removals |
|------|-------|---------------|
| Baseline (`9f400dd`) | 446 | — (includes `reqwest`, `hyper`, `rustls`, `tokio`) |
| eframe shell, updater / OS codecs / shell registration / extended formats behind features | 364 | HTTP/TLS stack, `windows` WIC, `objc2-image-io`, `winreg`, `jxl-oxide`, `image_dds`, `rawpsd`, `image-extras`, `exr` |
| Logger timestamps via `std` (RFC 3339 UTC) instead of `chrono` | 362 | `chrono`, `iana-time-zone` |
| `rfd` GTK3 backend on Linux instead of xdg-portal + async-std | 296 | `ashpd`, `zbus` and the async-std stack (Linux only) |

Decisions: `accesskit` stays excluded (eframe `default-features = false`); this is an accessibility
gap recorded in the capability register. Log file names and timestamps are now UTC
(`imageviewer-YYYY-MM-DD.log`, `…Z`). The Linux file dialog now needs the GTK3 runtime, which the
experimental Linux tarball documents. Duplicate majors remain where upstream crates pin them (`objc2 0.5`/`0.6`; `windows-sys`
0.52/0.59/0.61); they cannot be consolidated without forking dependencies `[OBSERVED]`.
