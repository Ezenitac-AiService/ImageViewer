# Adversarial Multi-Persona Feasibility Review: ImageViewer v1.1.0-alpha.1

**Created**: 2026-11-16

**Trigger**: User report — "윈도우, 맥 둘다 실행되지 않아" (the app does not run on Windows or macOS).

**Scope**: Features, requirements, frameworks, and libraries of the Rust workspace on `main` (`9f400dd`) and the published `v1.1.0-alpha.1` pre-release.

**Stance**: Deliberately hostile. Each persona assumes the project is wrong until evidence says otherwise (Constitution Principles V and VI).

**Companion**: [spec.md](./spec.md) turns the verdicts below into refactoring requirements.

## Evidence Legend

Every finding carries one of these tags. Nothing below is asserted from memory alone.

| Tag | Meaning |
|-----|---------|
| `[OBSERVED]` | Measured or reproduced on the review host: macOS 27.2.1, Apple Silicon (`arm64`), `rustc 1.100.0`. |
| `[CODE]` | Established by reading the cited source lines. Not exercised at runtime. |
| `[UNVERIFIED]` | Inferred. No Windows or Intel Mac host was available; the published release assets were not downloaded or executed. |

## 1. What "Does Not Run" Actually Means

The first job was to reproduce the failure instead of theorising about it.

| # | Experiment | Result | Tag |
|---|-----------|--------|-----|
| 1 | `cargo build -p iv-app`, run `target/debug/iv-app` | Process stays alive; one on-screen 1280×800 window exists. | `[OBSERVED]` |
| 2 | `packaging/macos/package-dmg.sh` (the exact script the release workflow runs), then `open ImageViewer.app` locally | Launches; window on screen. | `[OBSERVED]` |
| 3 | `codesign --verify --deep --strict ImageViewer.app` on that bundle | **Fails**: "code has no resources but signature indicates they must be present". Signature is `adhoc,linker-signed`, `Info.plist=not bound`, `Sealed Resources=none`. | `[OBSERVED]` |
| 4 | `spctl --assess --type execute` and `syspolicy_check distribution` on a copy carrying the download quarantine attribute | **Rejected**; `syspolicy_check` reports a *Fatal* codesign error. | `[OBSERVED]` |
| 5 | `lipo -archs` on the bundle executable | `arm64` only. | `[OBSERVED]` |
| 6 | `cargo test --workspace` | 41 passed, 0 failed, 0 ignored, 22 test binaries. | `[OBSERVED]` |
| 7 | Launch log after a run | `imageviewer-2026-11-16.log` is 0 bytes. | `[OBSERVED]` |

**Reading of the evidence**

- The code launches on macOS when built locally. The thing that fails is the **distributed artifact**. A bundle whose signature does not verify, once downloaded through a browser, is what macOS presents to users as "damaged" or unopenable. The exact dialog on the reporter's Mac was not observed.
- The published macOS asset is single-architecture. On an Intel Mac an `arm64`-only executable cannot start at all. `[UNVERIFIED]` for the published asset itself; `[OBSERVED]` for the script's output on an Apple Silicon host, and the release workflow builds on one `macos-latest` runner with no second architecture.
- **Windows root cause is unknown.** No Windows host was available. Section 3 lists candidate causes ranked by likelihood; none is confirmed. Reproducing on real Windows is the first task of any plan.
- No test launched the application before it was published. Experiment 6 is why CI was green while users could not start the app.

## 2. Persona Reviews

### Persona A — First-Launch User ("I downloaded it and double-clicked")

> "I do not care about your architecture. I clicked the thing and nothing happened."

| ID | Finding | Evidence |
|----|---------|----------|
| A1 | The macOS download fails OS integrity validation. There is no first-launch guidance anywhere: `README.md` has no mention of Gatekeeper, SmartScreen, quarantine, signing, or required runtimes. | `[OBSERVED]` experiments 3–4; `grep` over `README.md` returned nothing. |
| A2 | When startup fails, the app tells the user nothing. Window-creation failure and graphics-initialisation failure both call `event_loop.exit()` and discard the error; event-loop construction failure calls `exit(1)` with no message. | `[CODE]` `crates/iv-app/src/main.rs:546-554`, `:1381-1383`, `:1387` |
| A3 | The diagnostic logger is created but never written to at startup. `log_info` and `log_error` have zero call sites in the application; only `log_slow_render` is used. A user who finds the log directory finds an empty file. | `[CODE]` + `[OBSERVED]` experiment 7 |
| A4 | Mouse use is broken by design. A redraw is requested in only 8 places; the "repaint needed" answer from the UI toolkit is discarded (only `.consumed` is read), and neither pointer-move nor mouse-button events request a redraw. Clicking a toolbar button therefore has no visible effect until some unrelated event (a key press, a scroll, a decode completing) forces a frame. | `[CODE]` `main.rs:886-889`, `:926-938`, `:1008-1019` |
| A5 | Settings, About, and Image List windows are effectively frozen after their first paint: their event branch forwards input to the toolkit and never requests a redraw. Clicking OK or Cancel in Settings does nothing visible. | `[CODE]` `main.rs:765-878` |
| A6 | Hover-to-reveal toolbars cannot reveal on hover: the hover flags are set on pointer-move, but no frame is drawn. | `[CODE]` `main.rs:930-936` |
| A7 | A left click that the toolkit did not claim starts a window drag. Because the toolkit's pointer position is stale (A4), a click aimed at a button can be treated as a drag of the whole window. During an untouched run macOS logged "Window move completed without beginning". | `[CODE]` `main.rs:1009-1013`; log line `[OBSERVED]`, causal link `[UNVERIFIED]` |
| A8 | `Esc` quits the whole application unconditionally, including when a dialog is open. Spec 002 required "Esc closes active dialogs first". | `[CODE]` `main.rs:1029` |
| A9 | Double-clicking an image in Finder cannot open it. The bundle declares 29 document extensions, but the handler that should receive the file from the OS is an empty function. | `[CODE]` `crates/iv-platform/src/integration.rs:98-100`, `packaging/macos/Info.plist.in` |
| A10 | While browsing, any file change in the folder snaps the viewer back to the image that was originally opened: the folder watcher re-opens the *original* path, which recomputes the starting index. | `[CODE]` `main.rs:675-683`, `:733-735`, `folder.rs:71-74` |
| A11 | The Korean UI is unreachable. Language defaults to English, the locale-detection function has no call site, there is no language control in Settings, and the Settings labels are hard-coded English strings. | `[CODE]` `settings.rs:271`, `settings_view.rs:23-87`; `from_locale` unused |
| A12 | The window has no native title bar, so OS-standard move, resize, snap, and window-button behaviour are replaced by hand-drawn text buttons ("✕", "□", "—") that suffer from A4. | `[CODE]` `main.rs:532`, `main_view.rs:53-89` |

**Verdict**: Even where the binary starts, a mouse user would reasonably report that the app "does not work".

### Persona B — Release & Distribution Engineer

> "You shipped a file. You did not ship a product."

| ID | Finding | Evidence |
|----|---------|----------|
| B1 | The macOS bundle is assembled with `cp` and never signed as a bundle, so its signature is structurally invalid. | `[OBSERVED]` experiments 3–4; `packaging/macos/package-dmg.sh` |
| B2 | No Intel Mac build, although the constitution names Intel `x86_64` as a target. | `[OBSERVED]` experiment 5; `.github/workflows/release.yml` |
| B3 | The bundle has an empty `Resources` directory, no icon, no minimum-OS declaration, and a hard-coded `CFBundleVersion` of `1.1.0` that will drift from the workspace version. | `[OBSERVED]` bundle listing; `Info.plist.in` |
| B4 | Windows ships a renamed raw executable. A 240-line installer script and a packaging script exist in `packaging/windows/` but the release workflow invokes neither. | `[CODE]` `release.yml` "Build Windows binary" step |
| B5 | No build configuration statically links the C runtime (no `.cargo/config.toml`, no `build.rs`). The executable may require a separately installed Visual C++ runtime on a clean Windows install. | `[CODE]` absence; runtime effect `[UNVERIFIED]` |
| B6 | No Windows code signing, no embedded icon, version resource, or application manifest; the binary is a console-subsystem program, so a console window accompanies the GUI. Unsigned downloads meet SmartScreen, and Smart App Control where enabled. | `[CODE]` no `windows_subsystem` attribute anywhere; OS behaviour `[UNVERIFIED]` |
| B7 | The release workflow never executes what it built. There is no step between "build" and "publish" that starts the artifact. | `[CODE]` `release.yml` |
| B8 | Packaging scripts are committed without the executable bit; CI papers over it with `chmod +x`, and running the script as documented fails locally with "permission denied". | `[OBSERVED]` |
| B9 | The Linux "AppImage" step produces a directory tarball and prints "run appimagetool"; no AppImage is produced. | `[CODE]` `packaging/linux/package-appimage.sh` |
| B10 | The in-app update check rejects pre-releases, and the only release that exists is a pre-release. The "Install" button opens a web page. The download-and-verify routine has no caller. | `[CODE]` `update.rs:143`, `main.rs:1238-1246` |

**Verdict**: The distribution pipeline has never been run end-to-end by anyone acting as a user. Root cause of the macOS failure is here.

### Persona C — GUI & Rendering Architect

> "You wrote your own application framework and it is the buggiest file in the repository."

| ID | Finding | Evidence |
|----|---------|----------|
| C1 | `main.rs` is a 1,388-line hand-written integration of the windowing, GPU, and UI-toolkit layers: event routing, repaint scheduling, multi-window management, surface lifecycle, and DPI handling. Findings A4–A8 all live in this hand-written glue, which duplicates what an off-the-shelf application shell for the same toolkit already provides. | `[CODE]` |
| C2 | The UI stack is pinned to major versions published in early 2025 (`wgpu 24.0.5`, `egui 0.31.1`, `winit 0.30.13`), roughly 21 months old at review time (publication dates are reviewer recall and `[UNVERIFIED]`; confirm against the registry during `/speckit-plan`). The build already warns that `block v0.1.6`, pulled in by the GPU layer's macOS backend, "will be rejected by a future version of Rust". | `[OBSERVED]` build output, `cargo tree -i block` |
| C3 | Rendering has one path. The app requests a high-performance adapter with no fallback request and no retry; if that fails, A2 applies. Indexing `caps.formats[0]` panics when a surface reports no formats. A still-image viewer should not be unusable on a VM, a remote session, or an old driver. | `[CODE]` `renderer.rs:148-153`, `:174-179`; field impact `[UNVERIFIED]` |
| C4 | The event loop wakes every 16 ms forever, even when idle, yet does not redraw when the user moves the mouse. It pays the cost of polling and gets none of the benefit. | `[CODE]` `main.rs:629-632`; idle CPU 0.4–1.2 % `[OBSERVED]` |
| C5 | Every animation frame and every cache hit allocates fresh GPU textures, vertex buffers, and a full CPU copy of the pixels. | `[CODE]` `main.rs:644-651`, `renderer.rs:368-469` |
| C6 | The "blur/sharpen" shader is a fixed 9-tap kernel whose radius is scaled by sigma; it is not the Gaussian or unsharp filter the settings names promise. | `[CODE]` `renderer.rs:110-130` |
| C7 | Idle memory with no image open is about 200 MB. | `[OBSERVED]` RSS 203 MB debug; 220 MB release with one 1.2 MB PNG |
| C8 | Directory enumeration runs on the UI thread, with one metadata call per entry. | `[CODE]` `main.rs:165-212`, `folder.rs:32-67` |
| C9 | The platform crate depends on the windowing library only to name an event type, coupling OS adapters to the GUI framework. | `[CODE]` `crates/iv-platform/Cargo.toml`, `paths_dialogs.rs:5-15` |

**Verdict on the framework choice**: The individual libraries are defensible. The decision to hand-assemble them is not: it produced the interaction defects and cannot be tested (Persona E). Two candidate directions for `/speckit-plan`, to be chosen on evidence:

1. Keep the toolkit, adopt its maintained application shell, delete the custom glue, and gain a non-GPU fallback.
2. Keep the custom shell and fix repaint scheduling, input routing, and fallback by hand, accepting ongoing ownership of that code.

Custom window chrome should be dropped in favour of native decorations either way.

### Persona D — Security Auditor

> "Your constitution says Zero Trust. Your code says 'trust me'."

| ID | Finding | Evidence |
|----|---------|----------|
| D1 | Format validation is a **denylist** of text prefixes (`<`, `%!`, `push `, `viewbox`, …). Any other bytes pass. The decoder is then chosen by **file extension**. The constitution requires formats to be "verified via leading binary signatures before decoding". | `[CODE]` `format_guard.rs:47-56`, `decode.rs:158-168` |
| D2 | Folder files are read fully into memory with no size check; the 512 MB cap is tested only after the read, and only on the OS-codec path. | `[CODE]` `folder.rs:92-98`, `decode.rs:99-101` |
| D3 | The licence gate checks licences only. There is no known-vulnerability, banned-crate, or source check, for a lockfile of 579 packages. | `[CODE]` `ci.yml` (`command-arguments: licenses`); count `[OBSERVED]` |
| D4 | 30 `unsafe` occurrences in production code, concentrated in two hand-written OS-codec bridges that parse untrusted camera and HEIF data, plus a hand-rolled busy-spin future executor built on a raw no-op waker. | `[CODE]` `codecs.rs`, `paths_dialogs.rs:79-95` |
| D5 | The app makes an unsolicited network request five seconds after every eligible launch, on by default, for an update feature that cannot install anything (B10). | `[CODE]` `main.rs:694-716` |
| D6 | The Windows codec bridge initialises COM on pooled worker threads and never uninitialises it. | `[CODE]` `codecs.rs:174-175`; impact `[UNVERIFIED]` |
| D7 | 24 call sites in `main.rs` discard errors with `let _ =` or `.ok()`. Fail-closed is the stated policy; fail-silent is the implemented one. | `[OBSERVED]` count by `grep` |
| D8 | Distributing unsigned binaries trains users to bypass OS trust prompts, which is a security regression for an app whose charter is Zero Trust. | Follows from B1, B6 |

**Verdict**: The security claims in the documentation exceed what the code enforces. Claims must be narrowed to what tests demonstrate, or the code raised to the claims.

### Persona E — QA & Test Skeptic

> "Forty-one green ticks and not one of them opened a window."

| ID | Finding | Evidence |
|----|---------|----------|
| E1 | The application crate — 2,600 lines containing every defect in Persona A — has exactly one test, which checks the text of the About box. | `[OBSERVED]` experiment 6 |
| E2 | No test creates a window, initialises rendering, dispatches an input event, or starts a packaged artifact, on any OS. CI green on three operating systems certifies compilation and library logic only. | `[CODE]` test sources; `ci.yml` |
| E3 | All 61 tasks in `specs/002-*/tasks.md` are ticked complete, and spec 003 is marked "Implemented". The definition of done did not include starting the product. | `[OBSERVED]` |
| E4 | Constitution VI forbids citing a green run as proof of cross-OS verification and requires unverified targets to be disclosed. The release notes disclose nothing; the body is an auto-generated changelog link. | `[OBSERVED]` `gh release view` |
| E5 | Constitution IV forbids stubbed production paths. `install_macos_open_url_handler` is one (A9). `register_linux_desktop`, `download_and_verify`, `Language::from_locale`, `begin_open`, and `begin_render` are public functions with no production caller: scaffolding counted as features. | `[CODE]` |
| E6 | The performance success criteria of spec 002 (36 ms, 22 ms, 470 MB, 5 % CPU) have no end-to-end measurement harness. Two micro-benchmarks exist for the decode pipeline only. | `[CODE]` `crates/iv-media/benches/` |
| E7 | The interaction logic cannot be tested because it is fused into the window event handler alongside GPU and OS calls. Testability was never a design constraint. | `[CODE]` `main.rs:759-1337` |

**Verdict**: The process measured task completion. It did not measure whether the product works.

### Persona F — Product Owner & Scope Critic

> "You tried to ship three operating systems, thirty-eight formats, and an auto-updater in one afternoon."

| ID | Finding | Evidence |
|----|---------|----------|
| F1 | The entire Rust port, the fork separation, the release pipeline, and the first public pre-release are all dated 2026-11-16, with the release tagged about an hour after the last feature merge. | `[OBSERVED]` `git log` |
| F2 | Spec 002 demanded simultaneous full parity: 3 OS families, 38 extensions, OS-codec delegation, shell integration, verified self-update, numeric performance parity, bilingual UI, and Wayland nuances. It contained **no requirement that the packaged app start** and no success criterion about installation or first launch. | `[CODE]` `specs/002-*/spec.md` |
| F3 | Result: breadth without depth. Nearly every feature exists as a partial implementation; none is verified end-to-end. Examples: the "thumbnail strip" draws file names, not thumbnails; Image List sorting changes the list view but not navigation order and is not persisted; shell registration covers folder context menus only. | `[CODE]` `main_view.rs:209-231`, `image_list_view.rs`, `integration.rs:42-62` |
| F4 | The feature surface shown to users (toolbar buttons, declared document types, README claims) is larger than the working feature set, which converts every gap into a visible defect. | Follows from A9, B10, F3 |
| F5 | Linux receives equal billing with Windows and macOS while having the weakest delivery (B9) and no OS codecs. | `[CODE]` `codecs.rs:239-243` |

**Verdict**: The requirements were infeasible as a single increment and were missing their most important line. Scope must be tiered, and "it launches from the download" must be requirement zero.

### Persona G — Dependency Steward

> "Every crate is a promise someone else has to keep for you."

| Component | Used for | Verdict | Reason |
|-----------|----------|---------|--------|
| Windowing + GPU + UI toolkit (hand-integrated) | Entire UI | **Re-evaluate integration** | See Persona C. Keep the toolkit; stop owning the glue. |
| Async runtime (`tokio`) | Building a throw-away runtime to wait on one file-dialog future | **Drop from app code** | A blocking helper is already a dependency. `paths_dialogs.rs:69-77` |
| HTTP + TLS stack (`reqwest`, `rustls`, `ring`, transitive HTTP/2) | One JSON GET, then open a browser | **Defer with the updater** | Largest single contributor to the dependency graph for a feature that does not function (B10). |
| Core raster decoders (JPEG, PNG, GIF, WebP, BMP, TIFF) | Everyday formats | **Keep** | Mature, widely used. Need allowlist signature gating (D1). |
| Long-tail decoders (`jxl-oxide`, `image_dds`, `rawpsd 0.1`, `image-extras 0.1`, EXR, PCX, CUR, …) | 20+ rarely used formats | **Defer to an extended tier** | Several are `0.1` releases. Each is untrusted-input attack surface with no negative-test evidence per format. |
| OS codec bridges (`windows 0.59`, `objc2 0.6` family) | RAW, HEIF, AVIF, JP2 | **Defer; redesign before re-admission** | Hand-written `unsafe` FFI (D4, D6), unverified on Windows, absent on Linux. |
| ZIP reader | Archives | **Defer to extended tier, first to return** | Valuable legacy feature; needs its bomb and path-escape tests re-run against the shipped artifact. |
| Colour management, SIMD resize | ICC to sRGB, downscale | **Keep** | In use on the core path. |
| File watcher | Folder refresh | **Keep, fix behaviour** | A10. |
| Clipboard crate | One "Copy Info" button | **Question** | A whole dependency for one button in About. |
| Date/time crate | Log timestamps | **Question** | Heavy for one format call. |
| Duplicate majors | `objc2 0.5.3` + `0.6.5`; `windows 0.58` + `0.59` | **Consolidate** | `[OBSERVED]` `Cargo.lock` |

**Verdict**: 579 resolved packages `[OBSERVED]` for an application whose verified capability is "opens a window on the developer's Mac". The graph should shrink to what verified features need, and grow back one justified feature at a time.

## 3. Windows: Candidate Causes

> **Update (clarification session, 2026-11-16)** — the user reports: *the main window does not respond, a console window opens alongside it, and closing the console also closes the program.* This confirms candidate 4 and the console half of `B6` as the observed symptom, and makes candidates 1 and 2 unlikely as primary causes (the program did start). Still `[UNVERIFIED]` by the review itself: whether the window never repaints (A4–A7) or is marked hung by Windows. Candidates 3 and 5 remain open until reproduced.

Original ranking below, by the reviewer's estimate of likelihood before the report. None was reproduced.

| Rank | Candidate | Why plausible | How to confirm |
|------|-----------|---------------|----------------|
| 1 | OS trust block on an unsigned download (SmartScreen; Smart App Control blocks outright where enabled) | B6 | Launch the published asset on clean Windows 10 and 11; record the dialog. |
| 2 | Missing C runtime library on a clean install | B5 | Inspect the executable's imports; launch on an image with no Visual C++ runtime. |
| 3 | Graphics initialisation fails and the app exits silently | A2, C3 | Run from a terminal; run in a VM and over Remote Desktop. |
| 4 | App starts but is unusable by mouse, reported as "does not run" | A4–A7 | Launch and attempt to open a file using the mouse only. |
| 5 | A panic in the rendering layer, visible only in a console that closes immediately | A2, B6 | Run from a terminal and capture output. |

Candidates 3–5 would also explain a failure on macOS for a user who built from source, so they are in scope regardless of which one bit the reporter.

## 4. Synthesis

### Root causes, in order

1. **Nothing in the process ever launched the product** (B7, E1–E3). Every other defect survived because of this one.
2. **The distributed artifacts are not valid deliverables** (B1–B6). Confirmed for macOS; probable for Windows.
3. **Failures are silent** (A2, A3, D7), so neither users nor maintainers can tell cause 2 from cause 4.
4. **The hand-written application shell mishandles input and repaint** (A4–A8, C1).
5. **Scope was set to "everything at once"** (F1–F3), which left no room for 1–4.

### Verdict table

| Area | Item | Verdict |
|------|------|---------|
| Requirement | "Packaged app launches on a clean machine" | **Missing — add as requirement zero** |
| Requirement | Simultaneous full v1.0.6 parity on three OS families | **Infeasible as one increment — tier it** |
| Requirement | Numeric performance parity | **Keep as a goal; not release-blocking until measurable end-to-end** |
| Requirement | Zero Trust input handling | **Keep; align claims with enforced behaviour** |
| Requirement | Linux as an equal first-class target | **Demote to experimental until it passes the same launch gate** |
| Feature | Open, navigate, zoom, pan, fit modes, rotate/flip, fullscreen | **Core — fix and verify** |
| Feature | Animated GIF/WebP playback, slideshow, EXIF panel | **Core candidates — verify, else extended** |
| Feature | Custom frameless window chrome | **Drop** |
| Feature | Thumbnail strip (currently text) | **Hide until real** |
| Feature | Finder / Explorer "open with" and file associations | **Core for open-with delivery; registration is extended** |
| Feature | In-app update check and install | **Disable until verified end-to-end** |
| Feature | Archives, long-tail formats, OS-codec formats | **Extended tier with re-admission criteria** |
| Framework | UI toolkit and GPU layer | **Keep libraries; replace or repair the integration; add fallback rendering** |
| Framework | Four-crate workspace split | **Keep; remove GUI coupling from the platform crate; make interaction logic testable** |
| Library | Async runtime, HTTP/TLS stack | **Remove from the shipped product for now** |
| Library | Core decoders, colour, resize, watcher | **Keep** |
| Library | Long-tail decoders, OS FFI bridges | **Defer** |
| Process | Tests-first on library logic | **Keep** |
| Process | Definition of done | **Change: done means launched from the artifact on the target OS** |

## 5. Limits of This Review

- No Windows host and no Intel Mac. Every Windows statement about runtime behaviour is `[UNVERIFIED]`.
- The published release assets were not downloaded; the macOS findings come from running the same packaging script locally.
- Screen capture was not permitted on the review host, so correct pixel output was not confirmed; only that a window of the expected size exists.
- Pointer-interaction defects (A4–A7) were established by code reading. They were not reproduced by injecting input.
- Running the app during this review created `~/Library/Application Support/com.Ezenitac-AiService.ImageViewer/` (settings and an empty log) on the review host.
