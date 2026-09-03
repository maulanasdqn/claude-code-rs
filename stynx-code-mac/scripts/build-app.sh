#!/usr/bin/env bash
set -euo pipefail

# Builds the SwiftUI app. Must run with a clean environment: a Nix devshell
# exports DEVELOPER_DIR / SDKROOT / LD / CC that hijack Apple's toolchain and
# break xcodebuild's linker. We strip the environment down to the essentials.

DESKTOP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIGURATION="${1:-Debug}"

cd "$DESKTOP_DIR"
xcodegen generate

exec env -i \
  HOME="$HOME" \
  PATH="/usr/bin:/bin:/usr/sbin:/sbin" \
  TERM="${TERM:-xterm}" \
  xcodebuild \
  -project Stynx.xcodeproj \
  -scheme Stynx \
  -configuration "$CONFIGURATION" \
  -derivedDataPath build \
  CODE_SIGNING_ALLOWED=NO \
  build
