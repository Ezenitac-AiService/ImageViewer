#!/usr/bin/env bash
# Builds the universal macOS app bundle and DMG, signs the assembled bundle, and verifies it
# (specs/004-launch-reliability-refactor research R6, contracts/release-verification-contract.md §1).
#
# Usage: packaging/macos/package-macos.sh
# Output: target/release/ImageViewer.app and target/release/ImageViewer-macos-universal.dmg
#
# Signing (FR-005): ad-hoc ("-") by default. Set MACOS_SIGNING_IDENTITY to a Developer ID
# identity to sign with it and the hardened runtime instead.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

OUT="target/release"
APP="$OUT/ImageViewer.app"
DMG="$OUT/ImageViewer-macos-universal.dmg"
IDENTITY="${MACOS_SIGNING_IDENTITY:--}"

VERSION="$(cargo metadata --format-version 1 --no-deps |
    python3 -c 'import json,sys; d=json.load(sys.stdin); print(next(p["version"] for p in d["packages"] if p["name"]=="iv-app"))')"
BUILD="${VERSION%%-*}"

echo "== building iv-app ${VERSION} (aarch64 + x86_64)"
for target in aarch64-apple-darwin x86_64-apple-darwin; do
    cargo build --release --locked --target "$target" -p iv-app
done

echo "== assembling bundle"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create \
    target/aarch64-apple-darwin/release/iv-app \
    target/x86_64-apple-darwin/release/iv-app \
    -output "$APP/Contents/MacOS/iv-app"
sed -e "s/@VERSION@/${VERSION}/g" -e "s/@BUILD@/${BUILD}/g" \
    packaging/macos/Info.plist.in >"$APP/Contents/Info.plist"

echo "== building AppIcon.icns"
ICONSET="$(mktemp -d)/AppIcon.iconset"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
    sips -s format png -z "$size" "$size" ImageViewer.png --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -s format png -z "$double" "$double" ImageViewer.png --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
rm -rf "$(dirname "$ICONSET")"

echo "== signing bundle (identity: ${IDENTITY})"
if [ "$IDENTITY" = "-" ]; then
    codesign --force --deep --sign - "$APP"
else
    codesign --force --deep --options runtime --timestamp --sign "$IDENTITY" "$APP"
fi

echo "== verifying bundle"
packaging/verify/assert-macos-bundle.sh "$APP"

echo "== creating DMG"
STAGE="$(mktemp -d)"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
rm -f "$DMG"
hdiutil create -volname "ImageViewer" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
rm -rf "$STAGE"
echo "built $DMG"
