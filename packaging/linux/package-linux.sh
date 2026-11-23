#!/usr/bin/env bash
# Builds the experimental Linux tarball (specs/004 FR-026; Linux is not a supported target yet).
#
# Usage: packaging/linux/package-linux.sh
# Output: dist/ImageViewer-linux-x64.tar.gz
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

cargo build --release --locked -p iv-app

STAGE="$(mktemp -d)/ImageViewer"
mkdir -p "$STAGE"
cp target/release/iv-app "$STAGE/imageviewer"
sed 's|@EXEC_PATH@|imageviewer|g' packaging/linux/imageviewer.desktop.in >"$STAGE/imageviewer.desktop"
cat >"$STAGE/README-EXPERIMENTAL.txt" <<'TXT'
ImageViewer for Linux is EXPERIMENTAL and UNVERIFIED.

It is built and passes automated checks, but it has not passed the launch verification that
Windows and macOS releases must pass. Please report problems at
https://github.com/Ezenitac-AiService/ImageViewer/issues

Runtime libraries: GTK 3 (file dialogs) and libxkbcommon-x11 (keyboard on X11),
for example on Debian/Ubuntu: sudo apt install libgtk-3-0 libxkbcommon-x11-0

Run: ./imageviewer [image-or-folder]
TXT

mkdir -p dist
tar -czf dist/ImageViewer-linux-x64.tar.gz -C "$(dirname "$STAGE")" ImageViewer
rm -rf "$(dirname "$STAGE")"
echo "built dist/ImageViewer-linux-x64.tar.gz"
