# Contract: CLI Parsing & Windows Shell Integration

**Feature**: `001-architecture-security-refactoring`
**Covers**: `FR-005`, `FR-006`, `FR-010`

---

## 1. CommandLineHelper.Parse Contract

- **Inputs**: `IReadOnlyList<string> args`
- **CommandLineOptions Compatibility**: `CommandLineOptions(string? Path, bool RegisterShell, bool UnregisterShell, string? Error, bool IsPathNotFoundError = false)` preserves 4-argument caller compatibility.
- **Accepted Modes** (mutually exclusive):
  1. **No arguments** (or whitespace-only entries): `CommandLineOptions(Path: null, RegisterShell: false, UnregisterShell: false, Error: null, IsPathNotFoundError: false)`
  2. **Single target path**: Exactly 1 non-flag positional argument (Windows drive `<letter>:\...`, UNC `\\...`, or relative `.\...`) whose normalized `Path.GetFullPath` exists as a file or directory.
  3. **Shell verb switch**: Exactly 1 valid switch (`--register-shell` | `/register-shell` OR `--unregister-shell` | `/unregister-shell`) with zero positional paths.
- **Fail-Closed Rejections** (`Error != null`, `Path == null`, `RegisterShell == false`, `UnregisterShell == false`):
  - Two or more positional arguments.
  - Any unknown or duplicated flag starting with `-`, `--`, or `/` (other than a single valid shell switch; `/` flags are rejected consistently on Windows).
  - Simultaneous `--register-shell` and `--unregister-shell` (or duplicated switches).
  - Shell switch combined with a positional path.
  - Malformed paths (invalid characters, `PathTooLongException`, `ArgumentException`, `SecurityException`).
  - Non-existent target path: returns localized Korean error echoing the normalized user-supplied path (`IsPathNotFoundError = true`).

---

## 2. ShellIntegrationService Contract

- **Scope**: HKCU per-user registry (`Software\Classes\...`), zero administrator elevation required.
- **Verb Format**:
  - Folder: `Directory\shell\ImageViewer\command` -> `"<exePath>" "%1"`
  - Folder background: `Directory\Background\shell\ImageViewer\command` -> `"<exePath>" "%V"`
  - Archives (`.zip`, `.cbz`) & supported extensions: `SystemFileAssociations\<ext>\shell\ImageViewer\command` -> `"<exePath>" "%1"`
- **Safety Invariants**:
  - `ExecutablePath` MUST be validated to contain no embedded `"` or control characters before writing registry keys (`ArgumentException` / fail closed).
  - `IsRegistered()` MUST verify exact normalized command match (`string.Equals(actualCommand?.Trim(), BuildCommand(exe, background), StringComparison.OrdinalIgnoreCase)`) and exact menu label (`ImageViewer에서 보기(&3)`), never loose substring `Contains(...)`.
  - `App.RunShellCommand` (which runs prior to `ConfigureServices()`) MUST initialize and flush/dispose a scoped `PathRedactor`-enabled `FileLoggerProvider` on failure before returning exit code `1`.
