#!/usr/bin/env bash
set -euo pipefail
APPDIR="target/release/ImageViewer.AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications"
cargo build --release -p iv-app
cp target/release/iv-app "$APPDIR/usr/bin/imageviewer"
sed 's|@EXEC_PATH@|imageviewer|g' packaging/linux/imageviewer.desktop.in > "$APPDIR/usr/share/applications/imageviewer.desktop"
cp "$APPDIR/usr/share/applications/imageviewer.desktop" "$APPDIR/imageviewer.desktop"
echo "Prepared $APPDIR (run appimagetool to produce .AppImage)"
