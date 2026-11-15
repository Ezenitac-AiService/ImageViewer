# Feature Specification: Project-Wide Architecture, Security, and Reliability Refactoring

**Feature Branch**: `001-architecture-security-refactoring`

**Created**: 2026-11-15

**Status**: Draft

**Input**: User description: "프로젝트를 전반적으로 분석하고 검토하여 리펙토링을 진행하려 하고 있는데, 프로젝트 분석 작업과, 개선할 사항에 대한 내용을 판단하기 위한 리서치 작업을 진행하고 스펙을 작성해줘"

## Clarifications

### Session 2026-11-15

- Q: Should the refactoring scope include all three pillars (P1 Zero Trust security hardening, P2 lifecycle/concurrency stability, and P3 modular decomposition for headless testing) sequentially? → A: Include all three pillars (P1 -> P2 -> P3) sequentially while preserving existing feature compatibility.
- Q: How should the system handle multiple positional paths, unknown flags, or conflicting shell switches on the command line? → A: Accept at most one target path or a single valid shell flag; fail closed with an error on extra paths, unknown flags, or conflicting switches.
- Q: How should symbolic links or filesystem junctions (reparse points) be handled during local folder enumeration? → A: Reject links/junctions in archives and app-managed config/log paths, and skip symbolic-link/junction file entries during local folder enumeration, while allowing explicitly user-opened top-level paths.
- Q: What maximum file size should be enforced before reading the persisted settings file? → A: Reject persisted configuration files larger than 64 KB (65,536 bytes) and fall back to safe default settings.
- Q: How should full host paths and raw runtime exception messages be handled in user-facing UI error dialogs compared to diagnostic logs? → A: Omit all full host paths from diagnostic log files, while permitting full path display only in explicit user-supplied command-line path-not-found error prompts.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Zero Trust Input, Archive, and Privacy Hardening (Priority: P1)

As a user opening arbitrary folders, compressed archives, image files, drag-and-drop items,
or persisted settings from untrusted sources, I want the viewer to reject disguised,
malformed, oversized, or path-escaping inputs safely and protect my host paths and system
integrity without crashing or executing unintended content.

**Why this priority**: The application directly parses untrusted local and downloaded binary
images, archives, command-line arguments, Windows shell verbs, and configuration files. Any
trust-boundary failure exposes the user to resource exhaustion, path traversal, argument
injection, crash states, or path disclosure.

**Independent Test**: Can be fully tested by feeding crafted hostile inputs—spoofed file
extensions hiding script/document headers across character encodings, archives with
deceptive header sizes or traversal entries, extreme-dimension images, non-finite numeric
settings, malformed command-line arguments, and storage exceptions containing full paths—and
verifying fail-closed rejection, safe recovery, and sanitized logs with zero crashes.

**Acceptance Scenarios**:

1. **Given** a file bearing a supported image extension whose binary content is a script,
   markup document (including non-ASCII-BOM text encodings), or unsupported document format,
   **When** the viewer attempts to inspect or open it, **Then** the viewer MUST reject the
   payload based on actual content signatures before full decoding and display a safe error
   without executing or rendering unsafe content.
2. **Given** an archive entry whose declared metadata understates its real uncompressed size
   or whose path attempts directory traversal, sibling-prefix bypass, or link redirection,
   **When** the viewer lists or reads the archive, **Then** the viewer MUST enforce the
   configured uncompressed byte cap during streaming and reject unsafe entry paths without
   reading unbounded memory or writing outside permitted boundaries.
3. **Given** an image advertising dimensions whose pixel buffer stride or total pixel count
   would overflow 32-bit calculations or exceed the configured megapixel cap, **When**
   metadata inspection occurs, **Then** the viewer MUST abort allocation prior to pixel
   decoding and report an unambiguous size-limit error.
4. **Given** a persisted configuration file containing corrupted syntax, out-of-range numbers,
   non-finite floating-point values (`NaN` or infinity), or excessive file size, **When**
   the application loads configuration on startup, **Then** all invalid fields MUST fall
   back to safe finite defaults without crashing or corrupting window placement.
5. **Given** shell context-menu registration verification, diagnostic logging, or a runtime
   error whose underlying system message includes full directory paths, **When** status is
   checked, logged, or surfaced in UI dialogs, **Then** shell registration validation MUST
   verify exact quoted command alignment, diagnostic logs MUST redact or omit all full host
   paths, and runtime UI error dialogs MUST avoid leaking internal system paths (full path
   display is permitted only when echoing an explicit user-entered command-line path that
   could not be found).

---

### User Story 2 - Deterministic Lifecycle, Concurrency, and Responsiveness Stability (Priority: P2)

As a user rapidly browsing large folders, multi-page documents, or animated images—and
switching sources or closing the application mid-operation—I want background decoding,
rendering, folder watching, thumbnail generation, and logging to remain responsive, cancel
cleanly, and release all memory and OS handles deterministically.

**Why this priority**: Rapid image navigation, background prefetching, and animated decoding
combine unmanaged native buffers, UI timers, concurrent queues, and filesystem watchers.
Unbounded thread contention, missing disposal paths, synchronous blocking on UI or shared
locks, and fire-and-forget async handlers risk UI stalls, memory growth, and shutdown races.

**Independent Test**: Can be fully tested by rapidly switching across large and animated
images, triggering drag-and-drop and folder-change events during active decoding, verifying
bounded concurrent worker execution, and disposing active sessions to confirm clean
cancellation, zero leaked handles/timers, and zero unhandled async exceptions.

**Acceptance Scenarios**:

1. **Given** rapid consecutive navigation commands across mixed high-resolution static and
   animated images, **When** earlier load or render requests become obsolete, **Then**
   superseded work MUST cancel or discard obsolete outputs promptly without overwriting the
   currently selected image or starving foreground display.
2. **Given** simultaneous foreground rendering, background prefetching, and thumbnail
   generation under heavy workloads, **When** parallel workers decode and transform images,
   **Then** total concurrent native/CPU worker saturation MUST remain bounded so UI input
   and frame presentation stay responsive.
3. **Given** a user triggers drag-and-drop, opens a new folder/archive, or closes the
   application while decoding, playback, or timers are active, **When** the transition or
   shutdown completes, **Then** all active timers, watchers, concurrency gates, streams, and
   unmanaged pixel buffers MUST stop and dispose cleanly without unhandled exceptions.
4. **Given** high-frequency diagnostic logging during slow renders or transient errors,
   **When** log records are emitted, **Then** file logging MUST NOT block interactive display
   threads and MUST fail silently without crashing if the log directory is read-only or
   unavailable.

---

### User Story 3 - Modular Architecture and Headless Verification Coverage (Priority: P3)

As a maintainer evolving ImageViewer under strict Test-Driven Development, I want
monolithic presentation and orchestration logic decomposed into focused, UI-independent
components so existing features remain intact while core behaviors and edge cases are
verified automatically without relying on UI-thread coupling or test bypasses.

**Why this priority**: Concentrating navigation, zoom/pan math, layout calculation,
decoding orchestration, window chrome state, and dialog coordination inside oversized
monolithic components impedes safe refactoring, hides concurrency bugs, and blocks headless
automated testing of critical user journeys.

**Independent Test**: Can be fully tested by executing automated unit and integration tests
against separated domain/orchestration modules (layout/zoom math, navigation state,
prefetch/placeholder timing, security validation, settings recovery) headlessly while
verifying all existing user shortcuts, display modes, and visual behaviors remain preserved.

**Acceptance Scenarios**:

1. **Given** the refactored separation between view-independent orchestration/domain
   services and desktop window presentation, **When** automated tests exercise navigation,
   placeholder timing, zoom/pan calculations, filter/transform state, and error handling,
   **Then** those behaviors MUST be verifiable without instantiating live desktop windows or
   substituting fake production logic.
2. **Given** every existing documented keyboard shortcut, mouse wheel action, display mode,
   filter, slideshow mode, EXIF panel, multi-page TIFF flow, and animation playback rule,
   **When** refactoring completes, **Then** all existing user-facing capabilities and persisted
   settings compatibility MUST remain intact with zero behavioral regressions.

---

### Edge Cases

- What happens when an image file with a valid raster extension starts with a UTF-16 BOM,
  leading whitespace, or polyglot header containing markup or script payloads?
- How does the system behave when a non-seekable archive stream expands beyond the
  configured uncompressed-byte limit even though its archive header declares a small size?
- How does image buffer creation handle extreme pixel widths or heights where `width * 4`
  or `width * height` approaches or exceeds 32-bit signed integer limits?
- What happens when command-line arguments include multiple positional paths, unknown
  flags, conflicting shell switches, embedded double quotes, trailing drive-root
  backslashes, or non-existent targets?
- How does configuration normalization behave when numeric settings in the persisted
  configuration file contain `NaN`, `Infinity`, `-Infinity`, negative numbers, or when the
  file itself is truncated or exceeds 64 KB (65,536 bytes)?
- What happens when Windows shell context-menu registry values are partially overwritten
  or contain another executable sharing a substring prefix of the current application path?
- How does the viewer recover if a user drops an unreadable archive, triggers rapid folder
  watcher bursts, or closes the window while thumbnail workers and decode tasks are in flight?
- What happens when an OS storage error message includes the user's full directory path and
  is passed to the diagnostic logger or surfaced in UI dialogs?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST validate every untrusted file payload (local files and archive
  entries) using binary content signatures and format inspection before pixel decoding,
  strictly rejecting unsupported, scriptable, or document formats across all supported text
  and binary encodings regardless of filename extension.
- **FR-002**: System MUST enforce configured pixel-count caps and overflow-safe dimension/stride
  arithmetic prior to allocating pixel buffers, and MUST enforce configured uncompressed
  byte caps during actual stream copying for both archive entries and non-seekable streams.
- **FR-003**: System MUST canonicalize filesystem and archive paths with strict root-boundary
  delimiter checks, reject symbolic-link or filesystem-junction traversals within archive
  entries and application-managed configuration/log paths, and skip symbolic-link or
  filesystem-junction file entries during local folder enumeration (while permitting
  explicitly user-opened top-level paths).
- **FR-004**: System MUST validate persisted configuration files against a 64 KB
  (65,536-byte) maximum file-size cap, schema deserialization failures, and non-finite
  (`NaN`, `Infinity`) or out-of-bounds values, clamping or restoring safe defaults while
  preserving valid user preferences across existing schema versions.
- **FR-005**: System MUST sanitize command-line inputs and Windows shell context-menu
  registry entries by accepting at most one target path or a single valid shell switch,
  failing closed with an explicit error on extra positional paths, unknown flags, or
  conflicting switches, enforcing strict quoted-command matching during status checks, and
  reporting shell command failures cleanly.
- **FR-006**: System MUST redact or omit all full directory paths from persisted diagnostic
  log entries—including paths embedded within underlying runtime exception messages—while
  retaining actionable error types and filenames; UI error dialogs MUST likewise omit raw
  internal runtime paths, except when echoing an explicit user-supplied command-line target
  in a path-not-found prompt.
- **FR-007**: System MUST coordinate concurrency limits across foreground decoding,
  prefetching, rendering, and thumbnail generation so parallel CPU/native operations do not
  oversubscribe system cores or starve interactive UI operations.
- **FR-008**: System MUST release all unmanaged pixel buffers, active streams, timers,
  concurrency gates, filesystem watchers, and event subscriptions deterministically upon
  source changes, invalidation, or application exit, and MUST guard async entry points
  (including drag-and-drop and background timers) against unhandled exceptions.
- **FR-009**: System MUST decouple view-independent application orchestration, viewport/zoom
  geometry, and playback/navigation state machines from desktop window presentation so
  core behavior is testable headlessly while keeping desktop window rendering thin.
- **FR-010**: System MUST preserve all existing documented user workflows, keyboard/mouse
  controls, display modes, filters, slideshow behaviors, EXIF inspection, multi-page/animated
  playback, and Windows shell integration flags without breaking existing user settings.

### Key Entities

- **Untrusted Input Source**: Any externally supplied folder path, archive file, dropped
  item, or command-line argument subject to canonicalization, format verification, and
  access-boundary enforcement.
- **Media Item & Decoded Frame**: A validated image or archive entry paired with immutable
  metadata, pixel/memory cost accounting, and deterministic lifetime ownership.
- **Viewer Configuration**: User preferences for window geometry, display, filters, limits,
  and navigation, governed by version migration and finite-bounds normalization.
- **Shell Integration Entry**: Per-user desktop context-menu registration binding exact
  quoted executable paths and shell placeholders under least-privilege boundaries.
- **Diagnostic Log Entry**: Sanitized operational trace capturing timestamps, severity,
  filename-only identifiers, and path-redacted failure details.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of audited Zero Trust attack vectors (extension-spoofed script/document
  files, deceptive-size archive streams, overflow-prone dimensions, non-finite settings,
  substring-spoofed shell registrations, and path-leaking exception logs) are covered by
  automated negative tests that pass consistently.
- **SC-002**: Rapidly switching across at least 50 consecutive mixed-resolution/animated
  images and closing sessions mid-load completes with zero unhandled exceptions, zero leaked
  timers/watchers/gates, and stabilized memory within configured cache budgets.
- **SC-003**: 100% of core orchestration, layout/zoom calculation, navigation, security
  validation, and settings-recovery behaviors execute in headless automated verification
  suites without instantiating a live desktop window, with zero single-component coupling
  across presentation and business logic.
- **SC-004**: 100% of existing automated test scenarios continue to pass alongside newly
  added regression, concurrency, and security tests with zero skipped or weakened
  assertions.
- **SC-005**: Diagnostic log files contain zero full user directory paths across normal
  operations and simulated I/O permission or not-found failure scenarios.
- **SC-006**: All existing user-facing workflows (navigation, display modes, filters,
  rotation/flip, slideshow, EXIF panel, animations, multi-page TIFF, settings, and shell
  menu registration) operate identically from the user's perspective before and after
  refactoring.

## Assumptions

- Existing user-facing features, keyboard shortcuts, mouse bindings, Korean/English UI
  localization, settings schema compatibility (`settings.json`), and `--register-shell` /
  `--unregister-shell` CLI flags MUST remain backward-compatible.
- Confirmed Scope: Refactoring MUST cover all three pillars incrementally in priority order
  (P1 Security & Zero Trust -> P2 Resource Lifetime & Concurrency -> P3 Modular Decomposition
  & Testability) under strict Red-Green-Refactor TDD without mock-based pseudo-testing or
  production test bypasses.
- Target runtime remains 64-bit Windows 10/11 desktop environments; if verification runs on a
  non-Windows host where Windows-specific targets cannot execute, that host limitation
  will be reported explicitly and Windows verification deferred to a Windows runner/CI.
- Explicitly user-opened top-level local folders or files remain viewable even if linked,
  whereas local folder enumeration skips linked/junctioned entries and archive/app-data paths
  strictly reject symbolic-link or filesystem-junction escapes.
