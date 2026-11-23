---

description: "Task list for 004 Launch Reliability & Evidence-Driven Refactoring"
---

# Tasks: Launch Reliability & Evidence-Driven Refactoring

**Input**: Design documents from `specs/004-launch-reliability-refactor/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/](./contracts/), [quickstart.md](./quickstart.md)

**Tests**: REQUIRED. Constitution Principle III (TDD, NON-NEGOTIABLE) overrides the template default.

- Every test task precedes the implementation it covers and MUST be observed failing for the
  expected reason before that implementation starts.
- Tests assert real state with real fixtures: no mocks, no `#[ignore]` (Principle IV).
- Packaging-assertion and launch-check scripts are the test layer for packaging. They are written
  first and run red against the current artifacts.

**Organization**: Tasks are grouped by user story. US1, US2, and US3 are all P1, and together they
are the shippable fix for the reported defect.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: The user story the task serves (US1–US6)
- Paths are repository-relative

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Branch, toolchain floor, dependencies, fixtures.

- [X] T001 Create branch `004-launch-reliability-refactor` from `main` at the repository root, and commit the spec artifacts and constitution v2.4.0 (remove the Sync Impact Report comment from `.specify/memory/constitution.md` before committing)
- [X] T002 Raise `rust-version` to `"1.95"` and add workspace dependencies in `Cargo.toml`: `eframe = { version = "0.37", default-features = false, features = ["wgpu", "glow", "default_fonts", "clipboard", "x11", "wayland"] }`, `pollster = "0.4"`, `sys-locale = "0.3"`, `winresource = "0.1"`. Leave the other entries until their own tasks remove them
- [X] T003 [P] Create `.cargo/config.toml` with `[target.x86_64-pc-windows-msvc] rustflags = ["-C", "target-feature=+crt-static"]` (research R2)
- [X] T004 [P] Set the git executable bit on `packaging/macos/package-dmg.sh` and `packaging/linux/package-appimage.sh` with `git update-index --chmod=+x` (finding B8)
- [X] T005 [P] Add launch fixtures: `packaging/verify/reference.png` (≤ 64 KB PNG), and `packaging/verify/fixtures/two-images/a.png` and `b.png` (distinct, ≤ 16 KB each)
- [X] T006 [P] Add `packaging/verify/component-count.sh`. It prints the shipped-component count from `cargo tree -p iv-app -e normal --target all --prefix none | sed 's/ (\*)//' | sort -u | wc -l`, exits 1 if the count is greater than `${BUDGET:-411}`, and exits 1 if any of `reqwest`, `hyper`, `rustls`, `tokio` appears in that tree (FR-025, research R13)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Resolve the spikes and replace the hand-written shell with eframe. All three P1 stories
depend on this phase.

**⚠️ CRITICAL**: No user-story work begins until this phase is complete.

### Spikes (time-boxed; record outcomes under a new "Spike Results" heading in `specs/004-launch-reliability-refactor/research.md`)

- [ ] T007 Spike S0: Start the published `v1.1.0-alpha.1` `ImageViewer-windows-x64.exe` on clean Windows 10 and 11 VMs. Record the console window, whether the title bar shows "(Not Responding)", whether clicks act, and the import table (`dumpbin /dependents`) in `specs/004-launch-reliability-refactor/research.md`
- [ ] T008 [P] Spike S1: In a throw-away binary under `target/spikes/`, enumerate `wgpu 30` adapters on a Windows VM with no GPU driver and over RDP. Record whether a `DeviceType::Cpu` (WARP) adapter appears, in `specs/004-launch-reliability-refactor/research.md`
- [X] T009 [P] Spike S2: In a throw-away bundled eframe app, install a custom `NSApplicationDelegate` implementing `application:openURLs:` before `run_native`. Record cold- and warm-launch delivery from Finder "Open With" in `specs/004-launch-reliability-refactor/research.md`
- [X] T010 [P] Spike S3: In the same throw-away eframe app, confirm `osascript -e 'tell application id "…" to quit'` triggers a clean `on_exit`. On a `windows-latest` runner, confirm `Process.MainWindowHandle` and `Process.Responding` are meaningful and that `SendKeys` or `osascript` System Events input is permitted. Record in `specs/004-launch-reliability-refactor/research.md`

### Diagnostics core (used by US2 and US4)

- [X] T011 [P] Write failing tests in `crates/iv-core/tests/diagnostics_tests.rs` for the line grammar in `contracts/launch-and-diagnostics-contract.md` §3:
  - `launch_id` is exactly 8 hex characters;
  - the `phase=start` line includes `version=`, `os=`, `os_version=`, `arch=`;
  - `phase ∈ start/settings/probe/window/first_frame/navigate`;
  - `source=` keeps the file name only when given a full path;
  - CR/LF is stripped and home paths are redacted via `PathRedactor`;
  - `fatal category ∈ no_renderer/window_failed/settings_unwritable/panic`.
- [X] T012 Implement `crates/iv-core/src/diagnostics.rs` (`LaunchId`, `Phase`, `FatalCategory`, `LaunchRecord` formatter, `ExitCode {Clean=0, FatalStartup=2, Panic=3}`), export it from `crates/iv-core/src/lib.rs`, and add `AsyncFileLogger::write_sync(&str)` plus a flush-before-return path in `crates/iv-core/src/logger.rs` so launch lines reach disk before exit

### Platform decoupling (finding C9)

- [X] T013 Remove the `winit` and `tokio` dependencies from `crates/iv-platform/Cargo.toml`:
  - In `crates/iv-platform/src/paths_dialogs.rs`, replace `AppUserEvent` / `EventLoopProxy` with `pick_image_or_archive_async(on_done: Box<dyn FnOnce(Option<PathBuf>) + Send>)` and `pick_folder_async(...)`.
  - Block on the `rfd` future with `pollster::block_on`.
  - Delete `pollster_block` and `futures_noop_block` (removing their `unsafe`).
  - Delete `install_macos_open_url_handler` from `crates/iv-platform/src/integration.rs`.

### eframe shell (research R1, R3)

- [X] T014 [P] Write failing tests in `crates/iv-core/tests/viewport_and_chrome_tests.rs` for a new pure function `ViewportLayoutCalculator::image_quad(src_w, src_h, &ViewportState) -> ImageQuad` (screen rect in logical px, plus 4 UV corners). Cover: rotation 0/1/2/3 quarter turns × flip H/V give the expected UV permutation; `Fill`/`FitWidth`/`FitHeight`/`Original` sizes; zoom about the cursor keeps the anchor point fixed; pan offsets
- [X] T015 Implement `ImageQuad` and `image_quad` in `crates/iv-core/src/layout.rs` until T014 passes
- [X] T016 [P] Write failing tests in `crates/iv-core/tests/viewport_and_chrome_tests.rs` for `tile_grid(width, height, max_side) -> Vec<TileRect>`: a single tile when both dimensions ≤ `max_side`; full coverage with no overlap for 20000×9000 at 8192; zero-size input yields an empty grid
- [X] T017 Implement `tile_grid` in `crates/iv-core/src/layout.rs` until T016 passes
- [X] T018 Replace the dependencies in `crates/iv-app/Cargo.toml`: remove `winit`, `wgpu`, `egui`, `egui-winit`, `egui-wgpu`, `arboard`, `tokio`, `bytemuck`; add `eframe.workspace = true` and `pollster.workspace = true`. Delete `crates/iv-app/src/renderer.rs`
- [X] T019 Create `crates/iv-app/src/canvas.rs`:
  - upload a `DecodedFrame` as one or more `egui::TextureHandle`s using `tile_grid` with `ctx.input(|i| i.max_texture_side)`;
  - reuse handles via `TextureHandle::set` when dimensions are unchanged;
  - paint each tile as an `egui::Mesh` using `image_quad`;
  - map `FilterMode::None` → `TextureOptions::NEAREST` and `FilterMode::Smooth` → `TextureOptions::LINEAR`.
- [X] T020 Create `crates/iv-app/src/loader.rs`, porting `dispatch_async_load`, prefetch, and `drain_async_decodes` from the old `crates/iv-app/src/main.rs`:
  - keep the generation tags and the `DecodedFrameCache` of 8;
  - each worker holds an `egui::Context` clone and calls `request_repaint()` after sending its result;
  - no `let _ =` on channel sends: a failed send is written as a record line.
- [X] T021 Create `crates/iv-app/src/app.rs` implementing `eframe::App` with the current viewer state (source, index, page, frame, viewport, animation, settings). Paint via `canvas.rs`, poll `loader.rs` each frame, and drive animation with `ctx.request_repaint_after(current_delay)`. Rewrite `crates/iv-app/src/main.rs` to call `eframe::run_native` with `ViewportBuilder` (native decorations, saved size and position) and open the CLI path. Remove the 16 ms `ControlFlow::WaitUntil` loop
- [ ] T022 Run `cargo build -p iv-app` and `cargo test --workspace`, then launch `target/debug/iv-app packaging/verify/reference.png` on the macOS host and confirm the image is displayed and the toolbar buttons respond to the first click

**Checkpoint**: The eframe shell shows an image and responds to the mouse. User-story phases may start.

---

## Phase 3: User Story 1 - The Download Starts on a Clean Computer (Priority: P1) 🎯 MVP

**Goal**: Published Windows and macOS files start on clean machines, with one window, no console,
and no "damaged" message (FR-001–FR-006).

**Independent Test**: [quickstart.md](./quickstart.md) §2 build-time assertions pass locally, then
§4 steps 1–3 on clean VMs.

### Tests for User Story 1 (write first; run red against the current artifacts)

- [X] T023 [P] [US1] Write `packaging/verify/assert-macos-bundle.sh <ImageViewer.app>`. It exits non-zero unless all of the following hold, and printing each check. Run it red against today's bundle:
  - `codesign --verify --deep --strict` succeeds;
  - `lipo -archs Contents/MacOS/iv-app` contains both `x86_64` and `arm64`;
  - `plutil -lint` passes;
  - `CFBundleShortVersionString` equals `cargo metadata` workspace version;
  - `Contents/Resources/AppIcon.icns` exists.
- [X] T024 [P] [US1] Write `packaging/verify/assert-windows-exe.ps1 <exe>`. It exits non-zero unless all of the following hold:
  - the PE optional-header `Subsystem` = 2 (`WINDOWS_GUI`);
  - the import table has no DLL matching `VCRUNTIME*` or `api-ms-win-crt-*`;
  - `VersionInfo.FileVersion` starts with the workspace version;
  - an icon resource is present.

  Run it red against the `v1.1.0-alpha.1` asset.

### Implementation for User Story 1

- [X] T025 [US1] Add `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]` as the first line of `crates/iv-app/src/main.rs` (FR-003)
- [X] T026 [P] [US1] Create `packaging/windows/ImageViewer.manifest`: per-monitor-v2 DPI awareness, Common Controls v6, `asInvoker`
- [X] T027 [US1] Create `crates/iv-app/build.rs` using `winresource`, only when `CARGO_CFG_TARGET_OS == "windows"`:
  - icon `packaging/windows/ImageViewer.ico`;
  - `FileVersion` / `ProductVersion` from `CARGO_PKG_VERSION`;
  - `ProductName` "ImageViewer";
  - the manifest from T026.

  Add `[target.'cfg(windows)'.build-dependencies] winresource.workspace = true` to `crates/iv-app/Cargo.toml`
- [X] T028 [US1] Rewrite `packaging/windows/package-windows.ps1`: build `--release --target x86_64-pc-windows-msvc -p iv-app`, copy to `dist/ImageViewer-windows-x64.exe`, then run `packaging/verify/assert-windows-exe.ps1` and fail on a non-zero result. Signing hook (FR-005):
  - if `WINDOWS_SIGNING_CERT_BASE64` and `WINDOWS_SIGNING_PASSWORD` are set, decode the certificate to a temp `.pfx`, run `signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256`, verify with `signtool verify /pa`, fail on error, and delete the `.pfx`;
  - otherwise print `signature=none` and continue.

  `.github/workflows/release.yml` (T076) passes both secrets when they are defined.
- [X] T029 [P] [US1] Edit `packaging/macos/Info.plist.in`:
  - replace the literal versions with `@VERSION@` (`CFBundleShortVersionString`) and `@BUILD@` (`CFBundleVersion`, numeric `MAJOR.MINOR.PATCH`);
  - add `LSMinimumSystemVersion` `11.0` and `CFBundleIconFile` `AppIcon`;
  - restrict `CFBundleTypeExtensions` to Core: `jpg jpeg png gif webp bmp tif tiff zip cbz` (FR-022).
- [X] T030 [US1] Create `packaging/macos/package-macos.sh` (executable) and delete `packaging/macos/package-dmg.sh`. The script, in order:
  1. builds `aarch64-apple-darwin` and `x86_64-apple-darwin` and `lipo -create`s them into `ImageViewer.app/Contents/MacOS/iv-app`;
  2. renders `Info.plist` from `Info.plist.in` with the version from `cargo metadata --format-version 1 --no-deps`;
  3. builds `AppIcon.icns` from `ImageViewer.png` with `sips` and `iconutil`;
  4. signs with `codesign --force --deep --sign "${MACOS_SIGNING_IDENTITY:--}"`, adding `--options runtime` when the identity is not `-`;
  5. runs `packaging/verify/assert-macos-bundle.sh`;
  6. creates `target/release/ImageViewer-macos-universal.dmg` containing the app and an `/Applications` symlink.
- [X] T031 [P] [US1] Rename `packaging/linux/package-appimage.sh` to `packaging/linux/package-linux.sh` (executable). It produces `ImageViewer-linux-x64.tar.gz` containing the binary, `imageviewer.desktop`, and a `README-EXPERIMENTAL.txt` stating that Linux is experimental and unverified (FR-026)
- [X] T032 [P] [US1] Write `docs/FIRST_LAUNCH.md` (Korean) and `docs/FIRST_LAUNCH.en.md` (English), with a bidirectional language switcher:
  - Windows SmartScreen "More info → Run anyway", and the Smart App Control limitation;
  - macOS: open the DMG, drag to Applications, then System Settings → Privacy & Security → "Open Anyway";
  - Intel and Apple Silicon are both supported by one download.

  Mark where T110 adds the screenshots.
- [X] T033 [US1] Update `README.md` and `README.en.md`: the new asset names, a link to `docs/FIRST_LAUNCH*.md`, and Linux labelled experimental
- [X] T034 [US1] Run `packaging/macos/package-macos.sh` on the macOS host; T023 must now pass. Open the DMG copy with a quarantine attribute and confirm `syspolicy_check` reports no codesign *Fatal* error (a notarization notice is expected and acceptable)

**Checkpoint**: The macOS bundle verifies; the Windows build assertions are ready for CI (T076). Clean-machine confirmation happens in T110.

---

## Phase 4: User Story 2 - A Failed Start Explains Itself (Priority: P1)

**Goal**: Every launch writes a diagnostic record. Every startup failure shows a localized dialog
naming the cause and the log folder. A renderer fallback exists (FR-007–FR-010, FR-019).

**Independent Test**: [quickstart.md](./quickstart.md) §3 table, every row.

### Tests for User Story 2 (write first)

- [X] T035 [P] [US2] Add failing tests to `crates/iv-core/tests/settings_tests.rs` for `SettingsStore::load -> (ViewerSettings, LoadOutcome)`:
  - absent file → `CreatedDefaults`;
  - valid file → `Loaded`;
  - 70 KB file (`> 64 KB`) → `RecoveredFromDamage`;
  - malformed JSON → `RecoveredFromDamage`;
  - out-of-range values normalized → `RecoveredFromDamage`;
  - read-only parent directory → `Unreadable`;
  - symlinked settings path → `Unreadable`.

  Also:
  - `save_preferences` on a read-only directory returns `Err`;
  - two threads calling `save_preferences` concurrently on the same file (simulating two app copies) both return, and the resulting file parses as valid JSON equal to one of the two inputs. It is never truncated or interleaved (spec edge case "Two copies started at once").
- [X] T036 [P] [US2] Add failing tests to `crates/iv-core/tests/diagnostics_tests.rs` for `diagnostics::install_panic_hook(sink)`: inside `std::panic::catch_unwind`, a panic writes exactly one `fatal category=panic at=<file>:<line>` line to a real temp-dir log and invokes the supplied `on_fatal` callback once
- [X] T037 [P] [US2] Create `crates/iv-app/src/probe.rs` with failing unit tests (`#[cfg(test)] mod tests`) for the pure `select_renderer(&[AdapterSummary]) -> RendererChoice`:
  - an empty list gives `Glow`;
  - otherwise the order is `DiscreteGpu > IntegratedGpu > VirtualGpu > Cpu > Other`;
  - the chosen adapter's `name`, `device_type`, and `backend` are returned for the record.
- [X] T038 [P] [US2] Add failing tests to `crates/iv-core/tests/logger_and_i18n_tests.rs`: every key in `fatal.no_renderer`, `fatal.window_failed`, `fatal.panic`, `fatal.log_location`, `settings.reset_notice`, `settings.unreadable_notice`, `settings.save_failed`, `error.unsupported_format` returns a non-empty, distinct string for both `Language::Ko` and `Language::En`. Also add failing tests in `crates/iv-media/tests/sources_tests.rs` (FR-018): a new `user_message(&self, Language) -> String` on every `DecodeError` and `SourceError` variant returns a non-empty Korean string and a non-empty English string that differ, and never contains a full directory path
- [X] T039 [P] [US2] Add failing tests to `crates/iv-platform/tests/os_integration_tests.rs` for `fatal_dialog::redacted_log_folder(os)`, which returns exactly `%LOCALAPPDATA%\ImageViewer\logs` (Windows), `~/Library/Application Support/com.Ezenitac-AiService.ImageViewer/logs` (macOS), and `~/.config/imageviewer/logs` (Linux), and never contains the real user name

### Implementation for User Story 2

- [X] T040 [US2] Implement `LoadOutcome` (`Loaded | CreatedDefaults | RecoveredFromDamage | Unreadable`) and make `save_preferences` / `save_window_geometry` return `Result` in `crates/iv-core/src/settings.rs`. Keep the existing `fs4` lock and write via temp file + rename so concurrent saves are atomic. Pass until T035 passes
- [X] T041 [US2] Implement `install_panic_hook` in `crates/iv-core/src/diagnostics.rs` until T036 passes
- [X] T042 [US2] Add the T038 keys in both languages to `crates/iv-core/src/i18n.rs`, plus one key per `DecodeError` and `SourceError` variant. Implement `user_message(lang)` in `crates/iv-media/src/decode.rs` and `crates/iv-media/src/sources/mod.rs` using those keys, until T038 passes
- [X] T043 [US2] Create `crates/iv-platform/src/fatal_dialog.rs`: `redacted_log_folder` and `show_fatal(lang, category)` using `rfd::MessageDialog` (level Error, title "ImageViewer", OK only). Export it in `crates/iv-platform/src/lib.rs`, until T039 passes
- [X] T044 [US2] Implement `select_renderer` and `probe()` in `crates/iv-app/src/probe.rs`: enumerate `wgpu` adapters for the primary backends with no surface, and build `egui_wgpu::WgpuSetupCreateNew` with a `native_adapter_selector` that applies the same ordering. Pass until T037 passes
- [X] T045 [US2] Rewrite the startup sequence in `crates/iv-app/src/main.rs` exactly per `contracts/launch-and-diagnostics-contract.md` §1:
  - logger → panic hook → `phase=start` → CLI → settings `phase=settings outcome=…` → probe `phase=probe renderer=… adapter="…" type=… backend=…` → `run_native` (`phase=window` on creation);
  - on error: `fatal category=window_failed` → `show_fatal` → exit 2;
  - on panic: exit 3.
- [X] T046 [US2] In `crates/iv-app/src/app.rs`:
  - write `phase=first_frame source=<file name> w=<px> h=<px>` once, after the first paint of the requested image (`source=none` when none was requested);
  - write `exit=clean` in `on_exit` after saving settings and geometry (`settings_save_failed` on error);
  - show the `LoadOutcome` notice once via `crates/iv-app/src/views/status.rs`.
- [X] T047 [US2] Remove every `let _ =` / `.ok()` that discards an error in `crates/iv-app/src/` (the baseline was 24 sites, finding D7). Each becomes a status message or a record line (FR-010). Status messages shown to the user come from `user_message(lang)` / `I18n::tr`, never from `to_string()` on an error (FR-018); the English `Display` text goes only to the diagnostic record. Verify with `grep -rnE "let _ =|\.ok\(\)" crates/iv-app/src`, and justify any remaining hit inline
- [X] T048 [US2] Run every row of [quickstart.md](./quickstart.md) §3 that the macOS host can provoke. Paste the resulting record lines into `specs/004-launch-reliability-refactor/research.md` under "Spike Results / US2 evidence"

**Checkpoint**: Failures are visible and recorded. Silent exits are gone.

---

## Phase 5: User Story 3 - Core Viewing Works with Mouse or Keyboard (Priority: P1)

**Goal**: Every control responds on first input. All four open methods work. Esc closes dialogs
first. Native window behaviour. The folder keeps its position. Korean appears on first launch
(FR-011–FR-019, FR-021).

**Independent Test**: [quickstart.md](./quickstart.md) §4 steps 4–11 on Windows and macOS.

### Tests for User Story 3 (write first)

- [X] T049 [P] [US3] Write failing tests in `crates/iv-core/tests/interaction_tests.rs`, one test per row of `contracts/interaction-contract.md` §2 (keyboard) and §3 (pointer). Each asserts the exact `Vec<ViewerCommand>` from `map_input(InputEvent, &InteractionContext)`, and both "Primary" variants (`is_macos = true/false`) are covered
- [X] T050 [P] [US3] Write failing precedence tests in `crates/iv-core/tests/interaction_tests.rs` (contract §1):
  - Esc with 0, 1, and 2 open dialogs: `Quit` (or exit fullscreen when fullscreen) vs `CloseTopDialog` popping exactly one;
  - arrow keys with a dialog open produce no navigation command;
  - keys while a text widget has focus produce no command except Esc.
- [X] T051 [P] [US3] Write failing tests in `crates/iv-core/tests/interaction_model_tests.rs` for `ViewerModel::apply`, covering every `ViewerCommand` in data-model §7 and the invariant "`current` always refers to an entry in `source`, or is `None` when the source is empty":
  - `Next` on the last entry with `wrap_around` on and off;
  - `Prev` on an empty source;
  - `NextPage` on a single-page image falls through to `Next`;
  - `CloseTopDialog` on an empty stack is a no-op.
- [X] T052 [P] [US3] Add failing tests to `crates/iv-media/tests/sources_tests.rs` using real temp directories:
  - `FolderSource::scan_async(target_file)` reports the target entry before the full list completes;
  - for a 10,000-file folder, the first entry is available in < 1 s and the full natural-sorted list arrives later;
  - `rescan_keeping(current_key)` re-binds `current` by `source_key`;
  - when the current file is deleted, the index is clamped to `min(old_index, len - 1)` (data-model §8).
- [X] T053 [P] [US3] Add failing tests to `crates/iv-core/tests/settings_tests.rs`:
  - a `settings.json` without a `language` key resolves the language from a supplied locale tag (`"ko-KR"` → `Ko`, `"en-US"` / `"fr-FR"` → `En`);
  - a file with `"language": "en"` keeps `En` even when the locale is `"ko-KR"`.
- [X] T054 [P] [US3] Add failing tests to `crates/iv-platform/tests/os_integration_tests.rs` for `locale::language_from_tags(&[&str]) -> Language` (first `ko*` tag → `Ko`, else `En`, empty → `En`)

### Implementation for User Story 3

- [X] T055 [US3] Create `crates/iv-core/src/interaction.rs` with `InputEvent`, `InteractionContext`, `ViewerCommand`, `DialogKind`, `ViewerModel`, `map_input`, and `ViewerModel::apply` exactly as in data-model §7 and the interaction contract. Export it from `crates/iv-core/src/lib.rs`, until T049–T051 pass
- [X] T056 [US3] In `crates/iv-app/src/app.rs`, translate `egui` input (keys, modifiers, pointer, scroll, pinch, dropped files, `focused` widget state) into `InputEvent`, call `map_input`, apply the commands, and route `RequestOpenFile` / `RequestOpenFolder` to `iv_platform::paths_dialogs` with callbacks that push onto a channel and call `ctx.request_repaint()`
- [X] T057 [P] [US3] Create `crates/iv-app/src/views/toolbar.rs` with the Core-only toolbars of interaction contract §5. Every label goes through `I18n` (add the missing keys to `crates/iv-core/src/i18n.rs`). Delete `crates/iv-app/src/views/main_view.rs` and its custom titlebar
- [X] T058 [P] [US3] Rewrite `crates/iv-app/src/views/settings_view.rs` as an `egui::Window` driven by the `ViewerModel` dialog stack:
  - every label comes from `I18n` (add keys in both languages to `crates/iv-core/src/i18n.rs`);
  - add a Language selector (`Ko` / `En`);
  - remove the "Auto Check Updates" control;
  - OK normalizes, saves, and shows `settings.save_failed` on `Err`.
- [X] T059 [P] [US3] Rewrite `crates/iv-app/src/views/about_view.rs` as an `egui::Window`: remove "Check for Updates", and implement "Copy Info" with `ctx.copy_text` instead of `arboard`. Keep the existing `about_info_contains_license_origin_and_credits` test green, updating the library list to match the new stack
- [X] T060 [P] [US3] Rewrite `crates/iv-app/src/views/image_list_view.rs` as an `egui::Window` in the dialog stack. Selecting an entry issues the corresponding navigation command
- [X] T061 [US3] Implement `FolderSource::scan_async` and `rescan_keeping` in `crates/iv-media/src/sources/folder.rs`, using `DirEntry::file_type()` and lazy size/mtime (`metadata_loaded`). Wire the watcher event in `crates/iv-app/src/app.rs` to `rescan_keeping` instead of re-opening the original path (finding A10), until T052 passes
- [X] T062 [US3] Create `crates/iv-platform/src/locale.rs` (`language_from_tags`, `system_language()` via `sys_locale::get_locales()`), export it, and add `sys-locale.workspace = true` to `crates/iv-platform/Cargo.toml`, until T054 passes. Apply it in `crates/iv-core/src/settings.rs` only when the `language` key is absent, until T053 passes
- [X] T063 [US3] Create `crates/iv-platform/src/macos_open_files.rs` (`#[cfg(target_os = "macos")]`) per research R7 and the spike T009 result:
  - an `NSApplicationDelegate` implementing `application:openURLs:`, forwarding file-URL paths to a `Box<dyn Fn(Vec<PathBuf>) + Send + Sync>`;
  - every `unsafe` block carries a `// SAFETY:` comment;
  - install it from `crates/iv-app/src/main.rs` before `run_native`;
  - `app.rs` opens the first received path.
- [X] T064 [US3] Animation in `crates/iv-app/src/app.rs` + `canvas.rs`: advance frames on the wall-clock delay, update the existing `TextureHandle` in place, and schedule `ctx.request_repaint_after(current_delay)`. No per-frame texture allocation (finding C5)
- [X] T065 [US3] Native window behaviour in `crates/iv-app/src/main.rs` / `app.rs`:
  - `ViewportBuilder::with_decorations(true)` and `with_drag_and_drop(true)`;
  - restore size, maximized state, and position (size and maximized only on Wayland);
  - `ToggleFullscreen` via `ViewportCommand::Fullscreen`;
  - `Quit` via `ViewportCommand::Close` (FR-014).
- [ ] T066 [US3] Run [quickstart.md](./quickstart.md) §4 steps 4–11 on the macOS host (mouse-only, keyboard-only, ZIP/CBZ, drag-and-drop, Finder Open With cold and warm, Korean locale, 10,000-file folder). Record pass/fail per step in `specs/004-launch-reliability-refactor/research.md` under "US3 evidence"

**Checkpoint**: US1 + US2 + US3 form the shippable MVP for the reported defect (pending US4 automation and T110 clean-machine runs).

---

## Phase 6: User Story 4 - Nothing Is Published Unless It Has Been Started (Priority: P2)

**Goal**: Automated launch checks on every change and release. Draft-only publishing. Release notes
carry the platform table (FR-027–FR-030).

**Independent Test**: [quickstart.md](./quickstart.md) §5. A deliberately broken macOS bundle
yields no draft release.

### Tests for User Story 4 (verification tooling; write first, run red against `v1.1.0-alpha.1`)

- [X] T067 [P] [US4] Create `packaging/verify/window-probe.swift <pid>`. It prints `onscreen=<n>` for windows owned by `pid` via `CGWindowListCopyWindowInfo(.optionOnScreenOnly)` and exits 0 only if n ≥ 1
- [X] T068 [US4] Create `packaging/verify/launch-check.sh <app-or-binary> <image>` for macOS and Linux (Linux runs under `xvfb-run`, with no window probe):
  - start the artifact with `<image>`;
  - wait ≤ 30 s for `phase=first_frame source=<image name>` in the newest log under the OS log folder;
  - run `window-probe.swift` on macOS;
  - send `quit` via `osascript` on macOS (`SIGTERM` is not used); on Linux close via `xdotool key ctrl+q` if available, else record `clean_exit=null`;
  - require exit code 0 within 10 s and an `exit=clean` line.

  It writes `launch-verification-<os>-<arch>.json` with the fields of data-model §2 (`release_file`, `target`, `layer="automated"`, `window_shown`, `first_frame`, `responding=null`, `console_window=null`, `clean_exit`, `result`, `performed_by="ci"`, `recorded_at`, `notes`).
- [X] T069 [US4] Create `packaging/verify/launch-check.ps1 <exe> <image>` for Windows:
  - `Start-Process -PassThru`;
  - `MainWindowHandle ≠ 0` within 30 s;
  - `phase=first_frame` line seen;
  - `Responding` true for 20 samples over 10 s;
  - no child `conhost.exe`;
  - `CloseMainWindow()`;
  - exit code 0 within 10 s and an `exit=clean` line.

  It writes the JSON as in T068, with `responding` and `console_window` set.
- [X] T070 [US4] Add the interaction step from `contracts/release-verification-contract.md` §3 to both launch-check scripts: open `packaging/verify/fixtures/two-images/a.png`, send Right Arrow via `osascript` System Events / `SendKeys` (as permitted by spike T010), and expect `phase=navigate index=1` within 1 s. If input injection is not permitted, record `notes="interaction step moved to clean-machine"`
- [X] T071 [US4] Run `launch-check.sh` against the `v1.1.0-alpha.1` macOS asset to confirm it fails for the expected reason (no `first_frame` record), and against the T034 bundle to confirm it passes. Record both outputs in `specs/004-launch-reliability-refactor/research.md`

### Implementation for User Story 4

- [X] T072 [US4] Write `phase=navigate index=<n> total=<n>` at INFO on every image change in `crates/iv-app/src/app.rs` (launch contract §3); cover the formatter in `crates/iv-core/tests/diagnostics_tests.rs`
- [X] T073 [P] [US4] Create `packaging/release-notes.md.in`:
  - the platform table from `contracts/release-verification-contract.md` §4, with `@WIN10@`, `@WIN11@`, `@MAC_ARM@`, `@MAC_X64@`, `@LINUX@` placeholders filled by the workflow;
  - known limitations: one-time trust confirmation, Smart App Control, and Unavailable capabilities linked to `docs/CAPABILITIES.md`;
  - the line "Performance parity with v1.0.6: unmeasured" (constitution v2.4.0).
- [X] T074 [P] [US4] Extend `deny.toml` (constitution License Gate and FR-035):
  - `[advisories]` with `version = 2`, an `ignore = []` list where each entry carries a `reason`;
  - `[bans]` with `multiple-versions = "warn"`;
  - `[sources]` with `unknown-registry = "deny"` and `unknown-git = "deny"`.
- [X] T075 [US4] Rewrite `.github/workflows/ci.yml` per release-verification contract §5. Run on `ubuntu-latest`, `macos-latest`, `windows-latest`:
  - `cargo fmt --check`;
  - `clippy --workspace --all-targets --all-features -D warnings`;
  - `cargo test --workspace --all-features`;
  - `cargo test --workspace`;
  - a release build plus `launch-check` (Windows and macOS blocking; Linux under `xvfb-run` with `continue-on-error: true`);
  - `cargo deny check advisories bans licenses sources`;
  - `packaging/verify/component-count.sh`.
- [X] T076 [US4] Rewrite `.github/workflows/release.yml` per contract §4:
  - jobs `build-windows` (T028), `build-macos` (T030), and `build-linux` (T031);
  - `verify-windows` (`windows-latest`), `verify-macos-arm64` (`macos-latest`), and `verify-macos-x64` (`macos-15-intel`, mounting the universal DMG), all blocking;
  - `verify-linux` with `continue-on-error: true`;
  - a `deny` job;
  - `publish-draft` (`needs` all blocking jobs plus `deny`), using `softprops/action-gh-release` with `draft: true`, the assets, `SHA256SUMS.txt`, all `launch-verification-*.json`, and the body rendered from `packaging/release-notes.md.in`.

  Remove every path that publishes a non-draft release.
- [X] T077 [P] [US4] Add a "Definition of Done" section to `CONTRIBUTING.md` and `CONTRIBUTING.en.md`: a task is complete only when its behaviour was observed in the started application on each claimed OS or the gap is recorded (FR-030). Include the release procedure: draft → clean-machine checklist ([quickstart.md](./quickstart.md) §4) → attach `launch-verification-manual.md` → publish
- [ ] T078 [US4] Release rehearsal per [quickstart.md](./quickstart.md) §5 via `workflow_dispatch` with tag `v1.1.0-rc.1`. Then repeat on a branch that skips the signing step in `packaging/macos/package-macos.sh`, and confirm no draft is created. Record the run URLs in `specs/004-launch-reliability-refactor/research.md`

**Checkpoint**: A broken artifact cannot reach a published release.

---

## Phase 7: User Story 5 - The Product Only Promises What Works (Priority: P3)

**Goal**: Only Core and verified Extended capabilities are reachable or documented. No network
access. No placeholders (FR-020–FR-026, FR-037).

**Independent Test**: Compare `docs/CAPABILITIES.md` with the running app and README. Run
`packaging/verify/component-count.sh` and confirm it finds no HTTP client.

### Tests for User Story 5 (write first)

- [X] T079 [P] [US5] Add failing tests to `crates/iv-core/tests/cli_tests.rs`: `--register-shell`, `/register-shell`, `--unregister-shell` on every OS produce `CliCommand::OpenEmpty` with the localized `error.unsupported_switch` message ("not available in this version")
- [X] T080 [P] [US5] Add failing tests to `crates/iv-media/tests/decode_color_resize_tests.rs` under `#[cfg(not(feature = "extended-formats"))]`: valid QOI, JXL, PSD, DDS, PCX, and TGA fixtures produce `DecodeError::UnsupportedFormat`. Move the existing extended-format decode tests under `#[cfg(feature = "extended-formats")]` so they keep running with `--all-features`
- [X] T081 [P] [US5] Add a test to `crates/iv-media/tests/sources_tests.rs`: in the default build, `is_supported_image_extension` returns true only for `jpg jpeg jpe jfif png apng gif webp bmp dib tif tiff` and false for `jxl qoi psd dds pcx tga ppm pgm pbm pnm hdr exr cur ico dng cr2 cr3 nef arw orf rw2 raf heic heif avif jp2`

### Implementation for User Story 5

- [X] T082 [US5] Add `[features] default = [] ; extended-formats = ["dep:jxl-oxide", "dep:image_dds", "dep:rawpsd", "dep:image-extras", "dep:exr"]` in `crates/iv-media/Cargo.toml`. Mark those dependencies `optional = true` and reduce `image` default features to Core codecs (`jpeg`, `png`, `gif`, `bmp`, `tiff`, `webp`), adding the others under the feature. Gate the extended decoders in `crates/iv-media/src/decode.rs` and the extension list in `crates/iv-media/src/sources/mod.rs`. Introduce `DecodeError::UnsupportedFormat` here (T095 later reuses it for content-based rejection). Pass until T080–T081 pass
- [X] T083 [US5] Add `[features] updater = ["dep:reqwest", "dep:rustls", "dep:sha2"]`, `os-codecs = ["dep:windows", "dep:objc2-image-io", "dep:objc2-core-graphics", "dep:objc2-core-foundation"]`, and `shell-registration = ["dep:winreg"]` in `crates/iv-platform/Cargo.toml` (all default-off). Gate `update.rs`, `codecs.rs`, and the register functions in `integration.rs` with `#[cfg(feature = …)]`, and put `#![cfg(feature = "…")]` at the top of `crates/iv-platform/tests/update_tests.rs` and `os_codec_tests.rs`
- [X] T084 [US5] Make `crates/iv-core/src/cli.rs` reject the shell switches with `error.unsupported_switch` (key added to `crates/iv-core/src/i18n.rs` in ko and en), and remove the `RegisterShell` / `UnregisterShell` handling from `crates/iv-app/src/main.rs`, until T079 passes
- [X] T085 [US5] Remove the Unavailable UI from `crates/iv-app/src/`:
  - delete `views/update_notice.rs` and the update-check dispatch;
  - remove the thumbnail strip;
  - remove the `HostOsCodecProvider` usage from `loader.rs` (decode with `os_provider = None`);
  - restrict the `rfd` filter in `crates/iv-platform/src/paths_dialogs.rs` to Core extensions plus `zip cbz`.
- [X] T086 [US5] Evaluate each Extended candidate against FR-023 (slideshow, EXIF panel, multi-page TIFF paging, Image List sorting). For each, write or point to real-fixture tests in `crates/iv-media/tests/` and `crates/iv-core/tests/`, run it on macOS (and on Windows in T110), and either keep it reachable or hide it in `crates/iv-app/src/views/toolbar.rs` and the interaction mapping. Record the verdicts in `docs/CAPABILITIES.md`
- [X] T087 [US5] Write `docs/CAPABILITIES.md` (Korean) and `docs/CAPABILITIES.en.md` (English), with language switchers:
  - one row per capability with the data-model §4 fields `id`, `tier` (`core | extended | unavailable`), `user_visible` ("MUST be false when `tier = unavailable`"), `evidence` (links to tests and records), `verified_on`, and `components`;
  - the initial classification from data-model §4 as amended by T086.
- [X] T088 [US5] Align `README.md`, `README.en.md`, `SECURITY.md`, and `SECURITY.en.md` with `docs/CAPABILITIES*.md`: remove claims about auto-update, extended formats, OS codecs, shell registration, and thumbnails; label Linux experimental; state "performance parity with v1.0.6: unmeasured"
- [X] T089 [US5] Run `packaging/verify/component-count.sh`; it must report no `reqwest` / `hyper` / `rustls` / `tokio` (FR-025)

**Checkpoint**: Everything reachable is in the register. Nothing unverified is advertised.

---

## Phase 8: User Story 6 - A Foundation That Can Be Audited (Priority: P3)

**Goal**: Content-based format allowlist, pre-read size caps, ZIP caps, vulnerability gate,
justified component inventory within budget (FR-031–FR-036).

**Independent Test**: Every row of `contracts/input-acceptance-contract.md` §4 passes.
`cargo deny check` passes. The component count is ≤ 356.

### Tests for User Story 6 (write first)

- [X] T090 [P] [US6] Add failing tests to `crates/iv-core/tests/format_guard_tests.rs` for `FormatGuard::identify`, covering each signature in input-acceptance contract §1:
  - `FF D8 FF` → `Jpeg`;
  - `89 50 4E 47 0D 0A 1A 0A` → `Png`;
  - `GIF87a` / `GIF89a` → `Gif`;
  - `RIFF????WEBP` → `WebP`;
  - `BM` with ≥ 26 bytes and DIB header size ∈ {12, 40, 52, 56, 108, 124} → `Bmp`;
  - `II*\0` / `MM\0*` → `Tiff`;
  - `PK\x03\x04` / `PK\x05\x06` → `Zip`.

  Plus negatives: 1 KB of random bytes, a truncated BMP header, `RIFF` without `WEBP`, and empty input all give `None`.
- [X] T091 [P] [US6] Add a failing cross-check test to `crates/iv-media/tests/decode_color_resize_tests.rs`: for every Core fixture, `FormatGuard::identify` agrees with `image::guess_format`
- [X] T092 [P] [US6] Add failing tests to `crates/iv-media/tests/decode_color_resize_tests.rs` for input-acceptance contract §4:
  - row 1: SVG, HTML, PDF, and PS bytes named `.png` are refused by the denylist;
  - row 2: random bytes named `.jpg` give `UnsupportedFormat`;
  - row 4: a valid PNG named `.jpg` decodes as PNG;
  - row 6: a PNG header declaring 100000×100000 gives `ExceedsPixelLimit`;
  - row 11: BMP magic with a truncated header gives `UnsupportedFormat`.
- [X] T093 [P] [US6] Add failing tests to `crates/iv-media/tests/sources_tests.rs` for input-acceptance contract §4:
  - row 5: a sparse file `set_len(512 MB + 1)` is refused by `FolderSource::read_entry_bytes` with a size-limit error;
  - row 7: a ZIP containing `../evil.png` skips that entry and keeps the others readable;
  - row 8: a ZIP bomb entry over `max_zip_entry_megabytes` is refused;
  - row 9: an encrypted ZIP gives `EncryptedArchive`;
  - row 10: a ZIP with 50,001 entries is refused at `ZipSource::open`.

### Implementation for User Story 6

- [X] T094 [US6] Implement `CoreFormat` and `FormatGuard::identify` in `crates/iv-core/src/format_guard.rs`, until T090 passes
- [X] T095 [US6] In `crates/iv-media/src/decode.rs`, run `validate_header` then `identify`, and dispatch on `CoreFormat` instead of the extension. `None` gives `DecodeError::UnsupportedFormat`, mapped to `error.unsupported_format`. Pass until T091–T092 pass
- [X] T096 [US6] In `crates/iv-media/src/sources/folder.rs`, check `metadata.len() ≤ 512 MB` before reading in `read_entry_bytes`, then read through `File::take(cap + 1)` and refuse if more than `cap` bytes arrive, until T093 row 5 passes
- [X] T097 [US6] In `crates/iv-media/src/sources/zip.rs`, enforce ≤ 50,000 entries, ≤ 64 MB central directory, ≤ 4 GB container, and skip absolute or `..` entry names with a record line, until T093 rows 7–10 pass
- [X] T098 [US6] Bring the shipped-component count to ≤ 356:
  - run `packaging/verify/component-count.sh`;
  - consolidate duplicate `objc2` / `windows` majors where possible;
  - decide `accesskit` against the budget;
  - drop the direct `chrono` use in `crates/iv-core/src/logger.rs` if needed to meet the budget.

  Record the final count and each decision in research.md R13 ("Result" sub-heading).
- [X] T099 [US6] Add a "Justification" column to the direct-dependency tables in `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_NOTICES.en.md`. Each entry links a capability id from `docs/CAPABILITIES.md` (data-model §5: "MUST reference a `core` or verified `extended` capability"). Remove entries for crates no longer shipped, and add eframe, glow, glutin, and sys-locale
- [X] T100 [US6] Map every safety property stated in `SECURITY.md` and `SECURITY.en.md` to a test name from T090–T093 or launch contract §6. Delete any property without one (FR-033)
- [X] T101 [US6] Run `cargo deny check advisories bans licenses sources`. Resolve or record (with a reason in `deny.toml`) every finding

**Checkpoint**: Safety claims equal tested behaviour. Dependencies are justified and within budget.

---

## Phase 9: Polish & Cross-Cutting Concerns

- [X] T102 [P] Remove dead scaffolding (FR-037):
  - `SessionPhase`, `begin_open`, `begin_render`, `open_gen`, `render_gen` in `crates/iv-media/src/session.rs` (and their tests);
  - `register_linux_desktop` and `render_xdg_desktop_entry` in `crates/iv-platform/src/integration.rs`, unless a caller exists;
  - `crates/iv-app/src/resources/strings.json` if unused.
- [X] T103 [P] Remove `install_cjk_fallback_fonts` hard-coded paths, or keep them through eframe font definitions in `crates/iv-app/src/app.rs`. Confirm Korean text renders on Windows (Malgun Gothic) and macOS (Apple SD Gothic Neo); record the result in T110
- [X] T104 Bump the workspace `version` to `1.1.0-alpha.2` in `Cargo.toml` (single source per FR-006) and confirm `packaging/macos/Info.plist.in` and `crates/iv-app/build.rs` pick it up
- [X] T105 [P] Add `1.1.0-alpha.2` entries to `CHANGELOG.md` and `CHANGELOG.en.md`: the console fix, responsiveness, macOS signing and universal build, diagnostics, scope changes, and Linux experimental
- [X] T106 [P] Update the `**Status**` lines: `specs/002-rust-crossplatform-viewer/spec.md` (superseded for the next release by 004) and `specs/004-launch-reliability-refactor/spec.md` (Implemented, pending clean-machine verification)
- [X] T107 Run [quickstart.md](./quickstart.md) §1 in full (`fmt`, `clippy --all-features`, `test --all-features`, `test`, `deny`, component budget) on the macOS host after the final edit, and report any target not run natively (constitution build gate)
- [X] T108 Confirm all CI jobs from T075 are green on Windows, macOS, and Linux (Linux may be marked experimental). Link the run in `specs/004-launch-reliability-refactor/research.md`
- [X] T109 Re-run `/speckit-analyze` after implementation, and confirm that SC-010 (446 → ≤ 356 shipped-component metric; `plan.md`, `spec.md`, `research.md` R13/T101, and `THIRD_PARTY_NOTICES*.md` all agree on baseline 446, budget ≤ 356, and measured `298` via `packaging/verify/component-count.sh`, superseding the earlier 514/411 `--target all` draft) and the final count agree, in `specs/004-launch-reliability-refactor/spec.md` and `research.md`
- [ ] T110 Clean-machine checklist ([quickstart.md](./quickstart.md) §4, steps 1–12) on clean Windows 10, clean Windows 11, a real Apple-Silicon Mac, and an Intel Mac if available, using the draft-release assets from T078/T076:
  - on one clean Windows VM, also run the [quickstart.md](./quickstart.md) §3 rows the macOS host cannot provoke ("No GPU renderer", "Fatal path"), so every SC-003 failure condition has evidence;
  - fill `launch-verification-manual.md` (data-model §2 fields, `layer="clean-machine"`);
  - add the trust-dialog screenshots to `docs/FIRST_LAUNCH.md` and `docs/FIRST_LAUNCH.en.md`;
  - mark Intel macOS as unverified in the release notes if it was not run.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies.
- **Foundational (Phase 2)**: depends on Phase 1 and blocks all stories.
  - Spikes T007–T010 must finish before T044 (S1), T063 (S2), and T068–T070 (S3) are finalized.
- **US1 (Phase 3)**: depends on Phase 2. T025–T031 are independent of US2 and US3 code.
- **US2 (Phase 4)**: depends on Phase 2 (T012, T021).
- **US3 (Phase 5)**: depends on Phase 2 (T021). T058 depends on T042 (i18n keys) and T040 (`save` returns `Result`).
- **US4 (Phase 6)**: depends on US1 (packaging scripts T028, T030, T031) and US2 (record lines T045, T046).
- **US5 (Phase 7)**: depends on US3 (views exist to prune) and US4 (T075 CI tests both feature sets).
- **US6 (Phase 8)**: depends on Phase 2. T098 should follow US5 (removals shrink the tree). T099 depends on T087.
- **Polish (Phase 9)**: depends on all stories. T110 depends on T076 and T078 (draft assets exist).

### Story Dependency Graph

```text
Setup ─► Foundational ─┬─► US1 ──┐
                       ├─► US2 ──┼─► US4 ─► US5 ─┐
                       ├─► US3 ──┘               ├─► Polish (T110 clean-machine)
                       └─► US6 ──────────────────┘
```

### Within Each Story

Tests (red) → models and pure logic → app wiring → manual evidence task (T034, T048, T066, T071, T078).

## Parallel Opportunities

- **Phase 1**: T003, T004, T005, T006 in parallel after T002.
- **Phase 2**: spikes T008–T010 in parallel; T011, T014, and T016 in parallel (different test files or functions).
- **US1**: T023 and T024 in parallel; T026, T029, T031, and T032 in parallel.
- **US2**: T035, T036, T037, T038, and T039 in parallel (five different files).
- **US3**: T049–T054 in parallel (except T049 and T050, which share a file and should run in sequence); T057–T060 in parallel (separate view files).
- **US4**: T067, T073, T074, and T077 in parallel.
- **US5**: T079, T080, and T081 in parallel.
- **US6**: T090–T093 in parallel (T092 and T091 share a file; run them in sequence).
- **Across stories**: US1 packaging work runs alongside US2 and US3 code work; US6 tests run alongside US3.

### Parallel Example: User Story 2

```text
Task: "T035 LoadOutcome tests in crates/iv-core/tests/settings_tests.rs"
Task: "T036 panic hook tests in crates/iv-core/tests/diagnostics_tests.rs"
Task: "T037 select_renderer tests in crates/iv-app/src/probe.rs"
Task: "T038 i18n key tests in crates/iv-core/tests/logger_and_i18n_tests.rs"
Task: "T039 redacted_log_folder tests in crates/iv-platform/tests/os_integration_tests.rs"
```

### Parallel Example: User Story 3

```text
Task: "T051 ViewerModel::apply tests in crates/iv-core/tests/interaction_model_tests.rs"
Task: "T052 scan_async / rescan_keeping tests in crates/iv-media/tests/sources_tests.rs"
Task: "T053 locale-on-first-launch tests in crates/iv-core/tests/settings_tests.rs"
Task: "T054 language_from_tags tests in crates/iv-platform/tests/os_integration_tests.rs"
```

## Implementation Strategy

### MVP (fixes the reported defect)

1. Phase 1 → Phase 2 (including spikes S0–S3).
2. US1 (packaging), US2 (visible failures), and US3 (responsive core viewing). All three are P1;
   shipping any one alone leaves a user-visible "does not work".
3. **Stop and validate**: run T034, T048, and T066 on macOS. Manually run a build on a Windows VM
   (the T110 subset: steps 1–5).
4. Optionally cut `1.1.0-alpha.2` as a draft for testers once US4 automation exists.

### Incremental Delivery

1. MVP (US1 + US2 + US3) → tester build.
2. US4 → the release gate exists; first gated draft release.
3. US5 → scope is honest and there is no network access.
4. US6 → audit-ready; dependency budget met.
5. Polish → T110 clean-machine run → publish `1.1.0-alpha.2`.

### Notes

- `[P]` = different files, no incomplete dependencies.
- Verify that each test fails for the expected reason before implementing (Principle III).
- Commit after each task or logical group, with Korean commit messages (Principle I).
- Never mark a task done on a green library-test run alone when the task's behaviour is observable
  in the running app (FR-030, constitution VI).
