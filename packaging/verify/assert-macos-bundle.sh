#!/usr/bin/env bash
# Build-time assertions for the macOS bundle
# (specs/004-launch-reliability-refactor/contracts/release-verification-contract.md §1).
#
# Usage: assert-macos-bundle.sh <path/to/ImageViewer.app>
# Exits non-zero if any check fails; prints one line per check.
set -uo pipefail

APP="${1:?usage: assert-macos-bundle.sh <ImageViewer.app>}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
status=0

check() {
    local name="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        echo "PASS ${name}"
    else
        echo "FAIL ${name}"
        status=1
    fi
}

workspace_version() {
    (cd "$ROOT" && cargo metadata --format-version 1 --no-deps) |
        python3 -c 'import json,sys; d=json.load(sys.stdin); print(next(p["version"] for p in d["packages"] if p["name"]=="iv-app"))'
}

has_both_arches() {
    local archs
    archs="$(lipo -archs "$APP/Contents/MacOS/iv-app")"
    [[ "$archs" == *x86_64* && "$archs" == *arm64* ]]
}

version_matches() {
    local plist_version
    plist_version="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP/Contents/Info.plist")"
    [[ "$plist_version" == "$(workspace_version)" ]]
}

check "codesign-strict" codesign --verify --deep --strict "$APP"
check "universal-binary" has_both_arches
check "plist-lint" plutil -lint "$APP/Contents/Info.plist"
check "version-matches-workspace" version_matches
check "icon-present" test -s "$APP/Contents/Resources/AppIcon.icns"

exit "$status"
