#!/usr/bin/env bash
# Automated launch check for macOS and (experimental) Linux
# (specs/004-launch-reliability-refactor/contracts/release-verification-contract.md §2–§3).
#
# Usage: launch-check.sh <ImageViewer.app | binary> <reference-image> [record.json]
#
# Starts the artifact exactly as a user would, waits for the `first_frame` diagnostic record,
# checks that a window is on screen (macOS), optionally steps to the next image, asks the app to
# quit the way the OS does, and requires a clean exit. Writes a Launch Verification Record
# (data-model.md §2). Uses only production behaviour: no test-only flags.
set -uo pipefail

ARTIFACT="${1:?usage: launch-check.sh <app-or-binary> <image> [record.json]}"
IMAGE="$(cd "$(dirname "${2:?image}")" && pwd)/$(basename "$2")"
RECORD="${3:-launch-verification-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m).json}"
HERE="$(cd "$(dirname "$0")" && pwd)"
OS="$(uname -s)"
TIMEOUT=30

case "$OS" in
    Darwin) LOG_DIR="$HOME/Library/Application Support/com.Ezenitac-AiService.ImageViewer/logs" ;;
    *) LOG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/imageviewer/logs" ;;
esac
LOG="$LOG_DIR/imageviewer-$(date -u +%F).log"
mkdir -p "$LOG_DIR"
OFFSET=$( [ -f "$LOG" ] && wc -c <"$LOG" | tr -d ' ' || echo 0 )

new_log() { tail -c +"$((OFFSET + 1))" "$LOG" 2>/dev/null; }
wait_for() {
    local pattern="$1" secs="$2"
    for _ in $(seq 1 "$((secs * 2))"); do
        new_log | grep -q -- "$pattern" && return 0
        sleep 0.5
    done
    return 1
}

window_shown=false first_frame=false clean_exit=false navigated=null notes=""

# --- start
if [ "$OS" = "Darwin" ] && [ -d "$ARTIFACT" ]; then
    BIN="$ARTIFACT/Contents/MacOS/iv-app"
    BUNDLE_ID="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$ARTIFACT/Contents/Info.plist")"
    "$BIN" "$IMAGE" >/dev/null 2>&1 &
else
    BIN="$ARTIFACT"
    "$BIN" "$IMAGE" >/dev/null 2>&1 &
fi
PID=$!

# --- first frame
name="$(basename "$IMAGE")"
if wait_for "phase=first_frame source=$name" "$TIMEOUT"; then first_frame=true; fi

# --- window
if [ "$OS" = "Darwin" ]; then
    if swift "$HERE/window-probe.swift" "$PID" >/dev/null 2>&1; then window_shown=true; fi
else
    # No portable window query under Xvfb; a live process that drew its first frame has a window.
    if kill -0 "$PID" 2>/dev/null && [ "$first_frame" = true ]; then window_shown=true; fi
fi

# --- interaction step (§3): Right Arrow → navigate index=1
if [ "$OS" = "Darwin" ] && [ -n "${BUNDLE_ID:-}" ]; then
    if osascript -e "tell application id \"$BUNDLE_ID\" to activate" \
        -e 'tell application "System Events" to key code 124' >/dev/null 2>&1; then
        if wait_for "phase=navigate index=1" 2; then navigated=true; else navigated=false; fi
    else
        notes="interaction step moved to clean-machine (synthetic input not permitted)"
    fi
fi

# --- quit the way the OS does
if [ "$OS" = "Darwin" ] && [ -n "${BUNDLE_ID:-}" ]; then
    osascript -e "tell application id \"$BUNDLE_ID\" to quit" >/dev/null 2>&1
elif command -v xdotool >/dev/null 2>&1; then
    WID="$(xdotool search --pid "$PID" 2>/dev/null | head -1)"
    [ -n "$WID" ] && xdotool windowclose "$WID"
else
    notes="${notes:+$notes; }no OS close request available"
fi
exit_code=124
for _ in $(seq 1 20); do
    if ! kill -0 "$PID" 2>/dev/null; then
        wait "$PID"
        exit_code=$?
        break
    fi
    sleep 0.5
done
if [ "$exit_code" = 124 ]; then kill "$PID" 2>/dev/null; fi
if [ "$exit_code" = 0 ] && wait_for "exit=clean" 2; then clean_exit=true; fi

result=pass
for v in "$window_shown" "$first_frame" "$clean_exit"; do [ "$v" = true ] || result=fail; done
[ "$navigated" = false ] && result=fail

cat >"$RECORD" <<JSON
{
  "release_file": "$(basename "$ARTIFACT")",
  "target": {"os": "$(uname -s | tr '[:upper:]' '[:lower:]')", "os_version": "$( [ "$OS" = Darwin ] && sw_vers -productVersion || uname -r)", "arch": "$(uname -m)"},
  "layer": "automated",
  "window_shown": $window_shown,
  "first_frame": $first_frame,
  "navigated": $navigated,
  "responding": null,
  "console_window": null,
  "clean_exit": $clean_exit,
  "exit_code": $exit_code,
  "result": "$result",
  "performed_by": "$( [ -n "${GITHUB_ACTIONS:-}" ] && echo ci || echo local )",
  "recorded_at": "$(date -u +%FT%TZ)",
  "notes": "$notes"
}
JSON
cat "$RECORD"
[ "$result" = pass ]
