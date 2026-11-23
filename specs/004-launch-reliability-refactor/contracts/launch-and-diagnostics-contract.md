# Contract: Launch Sequence & Diagnostic Record

**Covers**: FR-003, FR-007–FR-010, FR-019, FR-025 | **Research**: R2, R4, R5

## 1. Startup Sequence (normative order)

| # | Phase | On success | On failure |
|---|-------|-----------|------------|
| 1 | Install panic hook; open diagnostic record | write `phase=start` | Continue with a stderr-only fallback, then show the fatal dialog at the first UI-capable point (record unavailable) |
| 2 | Parse CLI | — | CLI errors become a status message in the window (existing behaviour); never a silent exit |
| 3 | Load settings | write `phase=settings outcome=<LoadOutcome>` | Never fatal; see data-model §6 |
| 4 | Renderer probe (R4) | write `phase=probe renderer=<wgpu\|glow> adapter="<name>" type=<device_type> backend=<backend>` | No adapter → `renderer=glow` |
| 5 | `eframe::run_native` creates the window | write `phase=window` | `fatal category=window_failed` → dialog → exit 2 |
| 6 | First frame with the requested image painted | write `phase=first_frame source=<file name only> w=<px> h=<px>` | Decode errors are status messages, not fatal |
| 7 | User or OS close request | flush settings → write `exit=clean` → exit 0 | Save error → write `settings_save_failed` → exit 0 |

- A panic at any phase writes `fatal category=panic at=<file:line>`, flushes, shows the dialog, and
  exits **3**.
- If no image was requested, phase 6 writes `phase=first_frame source=none`.
- The application MUST NOT open any network connection in any phase (FR-025). A test asserts that
  the shipped dependency tree contains no HTTP client (research R13).

## 2. Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Clean exit after a close request |
| 2 | Fatal startup failure (phases 4–5) |
| 3 | Panic |

## 3. Diagnostic Line Grammar

```text
line      = "[" timestamp "] [" level "] launch=" launch_id SP body
level     = "INFO" / "ERROR" / "PERF"
launch_id = 8HEXDIG
body      = ("phase=" phase *(SP kv)) / ("fatal category=" category *(SP kv)) / ("exit=clean") / message
phase     = "start" / "settings" / "probe" / "window" / "first_frame" / "navigate"
            ; "navigate" is written at INFO for each image change: index=<n> total=<n>
category  = "no_renderer" / "window_failed" / "settings_unwritable" / "panic"
kv        = key "=" (token / DQUOTE text DQUOTE)
```

- The `start` line MUST include `version=`, `os=`, `os_version=`, `arch=`.
- All values pass `PathRedactor`. `source=` carries the file name only, never a directory.
- The line format is stable: the launch check (release-verification contract) parses
  `phase=first_frame` and `exit=clean`.

## 4. Fatal Dialog

- Native message box (`rfd::MessageDialog`), level Error, in the active language.
- **Title**: `ImageViewer`.
- **Body**: the localized cause sentence for `category`, then the log folder in redacted form:

| OS | Log folder as shown |
|----|---------------------|
| Windows | `%LOCALAPPDATA%\ImageViewer\logs` |
| macOS | `~/Library/Application Support/com.Ezenitac-AiService.ImageViewer/logs` |
| Linux | `~/.config/imageviewer/logs` (resolved via the existing `directories` rules) |

- **Single button**: OK. The process exits after it is dismissed.

**New i18n keys** (ko and en both required): `fatal.no_renderer`, `fatal.window_failed`,
`fatal.panic`, `fatal.log_location`, `settings.reset_notice`, `settings.unreadable_notice`,
`settings.save_failed`, `error.unsupported_format`.

## 5. Windows Process Properties

- PE subsystem `WINDOWS_GUI`; no console window is ever allocated (FR-003).
- Statically linked C runtime: the import table contains no `VCRUNTIME*.dll` or
  `api-ms-win-crt-*.dll`.
- Embedded icon, `FileVersion` and `ProductVersion` equal to `CARGO_PKG_VERSION`, and a manifest
  declaring per-monitor-v2 DPI awareness.

## 6. Negative Tests Required (Constitution III / VII)

1. A settings file > 64 KB produces `RecoveredFromDamage`, and the app starts.
2. A read-only settings directory produces `Unreadable`, the notice is shown, and the app starts.
3. A symlinked log directory is rejected; the app still starts and the fatal path remains
   reachable.
4. A path supplied on the CLI is redacted in every diagnostic line.
5. A forced panic in a test binary using the real hook writes a `category=panic` line and exits 3.
6. The probe with zero adapters selects `glow`. This is a unit test of the pure selection function
   over an empty adapter-info list.
