#!/usr/bin/env bash
set -euo pipefail
APP_DIR="target/release/ImageViewer.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"
cargo build --release -p iv-app
cp target/release/iv-app "$APP_DIR/Contents/MacOS/iv-app"
cp packaging/macos/Info.plist.in "$APP_DIR/Contents/Info.plist"
hdiutil create -volname "ImageViewer" -srcfolder "$APP_DIR" -ov -format UDZO "target/release/ImageViewer-macos.dmg"
