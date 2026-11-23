# Data Model: Launch Reliability & Evidence-Driven Refactoring

**Date**: 2026-11-16 | **Spec**: [spec.md](./spec.md) | **Research**: [research.md](./research.md)

Entities from spec → Key Entities, plus the runtime models the refactor introduces. "Location" is
where the entity lives once implemented.

## 1. Release File

| Field | Type | Rules |
|-------|------|-------|
| `name` | string | Exactly one of `ImageViewer-windows-x64.exe`, `ImageViewer-macos-universal.dmg`, `ImageViewer-linux-x64.tar.gz` |
| `version` | semver string | Equals the workspace `version` in `Cargo.toml` (FR-006) |
| `targets` | list of `{os, arch}` | Windows → `x86_64`; macOS → `aarch64` + `x86_64` (both slices present); Linux → `x86_64` |
| `tier` | `supported` \| `experimental` | Windows and macOS are `supported`; Linux is `experimental` (constitution v2.3.0) |
| `signature` | `adhoc` \| `publisher` \| `none` | macOS MUST be `adhoc` or `publisher` and pass strict verification; Windows is `none` until credentials exist (FR-005) |
| `sha256` | 64-hex | Listed in `SHA256SUMS.txt` |

**Location**: GitHub release assets, produced by `.github/workflows/release.yml`.

## 2. Launch Verification Record

One per release file × target × layer.

| Field | Type | Rules |
|-------|------|-------|
| `release_file` | string | Name of a Release File |
| `target` | `{os, os_version, arch}` | e.g. `windows 11 23H2 x86_64`, `macos 27.2 aarch64` |
| `layer` | `automated` \| `clean-machine` | Automated = hosted runner; clean-machine = manual checklist |
| `window_shown` | bool | Window exists within 30 s |
| `first_frame` | bool | `first_frame_presented` diagnostic line seen for the reference image |
| `responding` | bool \| null | Windows only: `Process.Responding` true throughout a 10 s sample |
| `console_window` | bool \| null | Windows only: PE subsystem is not `WINDOWS_GUI`, or a console was observed |
| `clean_exit` | bool | Exit code 0 and `exit=clean` diagnostic line after an OS close request |
| `result` | `pass` \| `fail` \| `not-run` | `pass` iff all applicable booleans are true (with `console_window` false) |
| `performed_by` | string | `ci` or the maintainer's handle for manual runs |
| `recorded_at` | ISO 8601 | |
| `notes` | string | Trust dialog wording and screenshots for the first-launch guide |

**State transitions**: `not-run → pass | fail`. A release may be published only when every
`supported` target has `automated = pass` and `clean-machine = pass` (Intel macOS clean-machine may
be `not-run` if no Intel Mac exists; release notes then list it as unverified).

**Location**: `launch-verification-<os>-<arch>.json` uploaded by CI; manual records as
`launch-verification-manual.md` attached to the draft release.

## 3. Diagnostic Record

A per-day log file. Each launch appends a block; the grammar is in
[contracts/launch-and-diagnostics-contract.md](./contracts/launch-and-diagnostics-contract.md).

| Field | Type | Rules |
|-------|------|-------|
| `launch_id` | 8-hex | Random per launch; ties lines of one launch together |
| `app_version` | semver | |
| `os` / `os_version` / `arch` | strings | From the standard library and OS query |
| `renderer` | `wgpu` \| `glow` | Chosen by the probe (R4) |
| `adapter` | `{name, device_type, backend}` | `device_type ∈ discrete, integrated, virtual, cpu, other` |
| `phase` | enum | `start → settings → probe → window → first_frame → exit` |
| `fatal` | `{category, detail}` \| null | `category ∈ no_renderer, window_failed, settings_unwritable, panic` |
| `settings_outcome` | `LoadOutcome` | See §6 |

**Validation**: Every line passes `PathRedactor` (no full home paths; CR/LF stripped). The file
lives under the app logs directory, and symlinked log directories are rejected (existing rule).
Retention stays 7 days.

## 4. Capability Register

| Field | Type | Rules |
|-------|------|-------|
| `id` | kebab-case | e.g. `open-folder`, `archive-zip`, `format-jxl` |
| `tier` | `core` \| `extended` \| `unavailable` | |
| `user_visible` | bool | Reachable in UI or documentation; MUST be false when `tier = unavailable` (FR-022) |
| `evidence` | list of links | Tests (positive and negative), launch records, docs (FR-023) |
| `verified_on` | list of `{os, arch}` | Core requires Windows and macOS |
| `components` | list of package names | Links to the Component Inventory |

**Initial classification**:

| Tier | Capabilities |
|------|-------------|
| Core | Launch; the 4 open methods; navigation, zoom, pan, fit modes, rotate, flip, fullscreen; JPEG, PNG, GIF, WebP, BMP, TIFF stills; animated GIF and WebP; ZIP/CBZ read-only; settings; ko/en; filter modes `None` and `Smooth` |
| Extended candidates (verify in this feature, else Unavailable) | Slideshow; EXIF panel; multi-page TIFF paging; Image List sorting |
| Unavailable | Update check and install; extended formats; OS-codec formats; shell and file-association registration; thumbnail strip; `SmoothSharpen` filter; custom window decoration |

**Transitions**: `unavailable → extended` only through FR-023 evidence; `extended → core` only by
spec change. Any tier can drop to `unavailable` when evidence is invalidated (e.g. a launch-check
regression).

**Location**: `docs/CAPABILITIES.md` (Korean, default) and `docs/CAPABILITIES.en.md` (English).

## 5. Component Inventory

| Field | Type | Rules |
|-------|------|-------|
| `package` | string | From the shipped-components metric (research R13) |
| `license` | SPDX | Allowed by `deny.toml` |
| `justification` | capability id(s) | MUST reference a `core` or verified `extended` capability (FR-034); transitive packages inherit their parent's |
| `advisory_status` | `clean` \| `accepted:<reason>` | From `cargo deny check advisories` (FR-035) |

**Location**: extends `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_NOTICES.en.md` (direct dependencies
with justification); transitive counts are recorded in CI output.

## 6. Settings Load Outcome (runtime)

`LoadOutcome ∈ { Loaded, CreatedDefaults, RecoveredFromDamage, Unreadable }`

| Outcome | Trigger | User-visible effect |
|---------|---------|--------------------|
| `Loaded` | Valid file read | None |
| `CreatedDefaults` | File absent (first launch) | None; language detected from the OS locale (R10) |
| `RecoveredFromDamage` | Parse error, oversized (> 64 KB), or invalid values | One-time notice: settings were reset |
| `Unreadable` | I/O or permission error, or symlink rejected | Notice: settings cannot be loaded or saved; session-only defaults |

Save failures produce a status message and a diagnostic line (FR-010). The schema and field names
stay v1.0.6-compatible (FR-019).

## 7. Viewer Model and Commands (runtime, `iv-core::interaction`)

**ViewerModel**: `source: Option<SourceSnapshot>`, `current: Option<EntryKey>`, `page: u32`,
`viewport: ViewportState` (existing), `fullscreen: bool`, `dialogs: Vec<DialogKind>` (stack; last =
topmost), `status: Option<StatusMessage>`, `language: Language`.

**DialogKind**: `Settings | About | ImageList`.

**ViewerCommand** (closed enum): `Next`, `Prev`, `First`, `Last`, `NextPage`, `PrevPage`,
`ZoomBy{factor, anchor}`, `PanBy{dx, dy}`, `SetFit(DisplayMode)`, `ResetView`, `RotateCw`,
`RotateCcw`, `FlipH`, `FlipV`, `ToggleFullscreen`, `OpenDialog(DialogKind)`, `CloseTopDialog`,
`RequestOpenFile`, `RequestOpenFolder`, `SetFilter(FilterMode)`, `Quit`.

**Invariants**:

- `current` always refers to an entry in `source`, or is `None` when the source is empty.
- `CloseTopDialog` pops exactly one dialog. Esc maps to it whenever `dialogs` is non-empty
  (FR-015).
- Navigation commands are no-ops while a dialog has keyboard focus.

The input-to-command table is in [contracts/interaction-contract.md](./contracts/interaction-contract.md).

## 8. Media Source Snapshot (runtime, `iv-media`)

Extends `MediaEntry` (existing) with:

- `identified: Option<CoreFormat>`, set lazily on first read (R8);
- `metadata_loaded: bool`, since size and date are fetched lazily (R10).

**Rescan rule**: After a watcher event, a new snapshot replaces the old one. `current` is re-bound
by `source_key`. If that key is gone, the index is clamped to `min(old_index, len - 1)` (FR-016).
