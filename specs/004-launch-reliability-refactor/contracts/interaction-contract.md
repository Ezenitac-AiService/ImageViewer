# Contract: Input → Viewer Command Mapping

**Covers**: FR-011–FR-016, FR-036; User Story 3 | **Research**: R1, R3, R14

The mapping is implemented as the pure function `iv_core::interaction::map_input` and is unit-tested
row by row. "Primary" means **Cmd** on macOS and **Ctrl** on Windows and Linux.

## 1. Precedence

1. If a text field or other egui widget has keyboard focus, keyboard events go to the widget,
   except Esc.
2. If any dialog is open, Esc → `CloseTopDialog`, and navigation keys do not reach the viewer.
3. Otherwise, the tables below apply.

## 2. Keyboard

| Input | Command | Notes |
|-------|---------|-------|
| `→` / `↓` | `Next` | Wraps if `navigation.wrap_around` |
| `←` / `↑` | `Prev` | |
| `Home` / `End` | `First` / `Last` | |
| `PageDown` / `PageUp` | `NextPage` / `PrevPage` | Falls through to `Next` / `Prev` on single-page images |
| `+` or `=` / `-` | `ZoomBy{1.2 / 0.83, centre}` | |
| `0` | `SetFit(Original)` | |
| Primary+`0` | `ResetView` | |
| `7` / `8` / `9` | `SetFit(FitWidth / FitHeight / Fill)` | |
| `R` / `L` | `RotateCw` / `RotateCcw` | |
| `H` / `V` | `FlipH` / `FlipV` | |
| `U` / `I` | `SetFilter(None / Smooth)` | `S` (`SmoothSharpen`) is unmapped while the filter is Unavailable |
| `Alt+Enter`; macOS also `Ctrl+Cmd+F` | `ToggleFullscreen` | |
| `Enter` | `OpenDialog(ImageList)` | Only if Image List is Core or verified Extended |
| `Tab` | Toggle EXIF panel | Only if the EXIF panel is verified Extended |
| `Space` | Toggle slideshow | Only if slideshow is verified Extended |
| Primary+`O` | `RequestOpenFile` | |
| Primary+`Shift+O` | `RequestOpenFolder` | |
| Primary+`,` | `OpenDialog(Settings)` | New: OS-conventional settings shortcut |
| `Esc` | `CloseTopDialog` if any dialog is open; else exit fullscreen if fullscreen; else `Quit` | FR-015. Replaces the unconditional `exit()` at `main.rs:1029` |
| Primary+`Q` (macOS), `Alt+F4` (Windows) | `Quit` | Delivered by the OS as a close request |

## 3. Pointer

| Input | Command | Notes |
|-------|---------|-------|
| Click on any control | That control's action | Visible response within 100 ms with no other input (FR-011) |
| Wheel (setting = Navigate) | `Next` / `Prev` | Trackpad pixel deltas accumulate (existing `TrackpadAccumulator`) |
| Wheel (setting = Zoom) | `ZoomBy{1.1 / 0.9, cursor}` | |
| Primary+Wheel | The opposite of the setting | Existing behaviour |
| Shift+Wheel | `PanBy{0, dy}` | |
| Left-drag on canvas when the image exceeds the viewport | `PanBy{dx, dy}` | Replaces "drag the whole window" (A7) |
| Left-drag on canvas when the image fits | No-op | Windows move through native decorations (FR-014) |
| Middle-drag | `PanBy{dx, dy}` | |
| Pinch | `ZoomBy{factor, cursor}` | |
| Mouse Back / Forward buttons | `Prev` / `Next` | |
| Double-click on canvas | `ToggleFullscreen` | |
| File dropped on window | Open that path | Folder, image, or ZIP/CBZ |

When a zoom crosses 1.2× on a downscaled frame, the full-resolution reload continues as today.

## 4. Window

- Native OS decorations. The custom titlebar and the ✕ / □ / — buttons are removed (FR-014).
- Window geometry is saved on close and restored on start (existing behaviour; Wayland restores
  size and maximised state only).

## 5. Toolbar Contents (Core only)

Top bar: Open File / Archive · Open Folder · Settings · About.
Bottom bar: Zoom − · zoom % (click = `ResetView`) · Zoom + · Prev · `n / total` · Next.
Extended-candidate controls appear only when verified: Image List, EXIF, Slideshow.

## 6. Tests Required

- One unit test per table row above: input event plus context → exact command list.
- Precedence tests: Esc with 0, 1, and 2 dialogs; navigation keys with a dialog open; typing in a
  settings text field.
- Model tests: `ViewerModel::apply` for every command, including `Next` on the last entry with wrap
  on and off, and `Prev` on an empty source.
- A launch-check interaction step (release-verification contract §3) clicks Next via the OS and
  observes the position change in the diagnostic record.
