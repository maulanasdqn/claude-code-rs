#!/usr/bin/env bash
set -euo pipefail

# Builds the stynx-code-ffi static library, generates Swift bindings, and
# assembles an XCFramework consumed by the SwiftUI app.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DESKTOP_DIR="$REPO_ROOT/stynx-desktop"
GENERATED_DIR="$DESKTOP_DIR/Generated"
XCFRAMEWORK_DIR="$DESKTOP_DIR/Frameworks"
TARGET="aarch64-apple-darwin"
LIB_NAME="libstynx_code_ffi"
FRAMEWORK_NAME="StynxCoreFFI"

cd "$REPO_ROOT"

echo "==> Building stynx-code-ffi ($TARGET, release)"
cargo build -p stynx-code-ffi --release --target "$TARGET"

DYLIB_PATH="$REPO_ROOT/target/$TARGET/release/$LIB_NAME.dylib"
STATICLIB_PATH="$REPO_ROOT/target/$TARGET/release/$LIB_NAME.a"

echo "==> Generating Swift bindings"
rm -rf "$GENERATED_DIR"
mkdir -p "$GENERATED_DIR"
cargo run -p stynx-code-ffi --bin uniffi-bindgen --release -- \
  generate \
  --library "$DYLIB_PATH" \
  --language swift \
  --out-dir "$GENERATED_DIR"

# uniffi emits <name>FFI.modulemap; xcframework headers need it named module.modulemap.
HEADERS_DIR="$GENERATED_DIR/Headers"
mkdir -p "$HEADERS_DIR"
mv "$GENERATED_DIR"/*.h "$HEADERS_DIR/" 2>/dev/null || true
if ls "$GENERATED_DIR"/*.modulemap >/dev/null 2>&1; then
  mv "$GENERATED_DIR"/*.modulemap "$HEADERS_DIR/module.modulemap"
fi

echo "==> Assembling $FRAMEWORK_NAME.xcframework"
rm -rf "$XCFRAMEWORK_DIR"
mkdir -p "$XCFRAMEWORK_DIR"
xcodebuild -create-xcframework \
  -library "$STATICLIB_PATH" \
  -headers "$HEADERS_DIR" \
  -output "$XCFRAMEWORK_DIR/$FRAMEWORK_NAME.xcframework"

echo "==> Done. Generated Swift: $GENERATED_DIR/*.swift"
echo "    XCFramework:        $XCFRAMEWORK_DIR/$FRAMEWORK_NAME.xcframework"
