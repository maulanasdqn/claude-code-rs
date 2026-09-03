#!/usr/bin/env bash
set -euo pipefail

DESKTOP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_PATH="${DESKTOP_DIR}/build/Stynx.app"
VERSION=$(defaults read "${APP_PATH}/Contents/Info.plist" CFBundleShortVersionString)
DMG_NAME="Stynx-${VERSION}.dmg"
DMG_OUT="${DESKTOP_DIR}/build/${DMG_NAME}"
STAGING_DIR=$(mktemp -d)

trap 'rm -rf "$STAGING_DIR"' EXIT

if [[ ! -d "$APP_PATH" ]]; then
  echo "ERROR: ${APP_PATH} not found. Run build-app.sh first." >&2
  exit 1
fi

echo "Staging app bundle..."
cp -R "$APP_PATH" "${STAGING_DIR}/Stynx.app"
ln -s /Applications "${STAGING_DIR}/Applications"

echo "Creating DMG: ${DMG_NAME}"
hdiutil create \
  -volname "Stynx ${VERSION}" \
  -srcfolder "$STAGING_DIR" \
  -ov \
  -format UDZO \
  -fs HFS+ \
  "$DMG_OUT"

echo "Done: ${DMG_OUT}"
