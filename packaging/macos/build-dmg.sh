#!/usr/bin/env bash
# Builds a drag-to-Applications macOS .dmg disk image containing Ergon.app,
# an /Applications symlink, and an optional LaunchAgent helper command.
# Automatically signs (codesign) and notarizes (xcrun notarytool + xcrun stapler)
# when Apple signing secrets are configured; builds cleanly unsigned otherwise.
#
# Usage:
#   ./packaging/macos/build-dmg.sh --binary <path> --version <semver> [--arch aarch64-apple-darwin] [--out-dir dist]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY=""
VERSION=""
ARCH="aarch64-apple-darwin"
OUT_DIR="dist"

while [ $# -gt 0 ]; do
  case "$1" in
    --binary) BINARY="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --arch) ARCH="$2"; shift 2 ;;
    --out-dir) OUT_DIR="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

[ -n "$BINARY" ] || { echo "error: --binary is required" >&2; exit 2; }
[ -n "$VERSION" ] || { echo "error: --version is required" >&2; exit 2; }
[ -f "$BINARY" ] || { echo "error: binary not found: $BINARY" >&2; exit 1; }

mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"
WORK_DIR="$(mktemp -d)"
NOTARY_KEY_FILE=""
cleanup() {
  if [ -n "$NOTARY_KEY_FILE" ] && [ -f "$NOTARY_KEY_FILE" ]; then
    rm -f "$NOTARY_KEY_FILE"
  fi
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

DMG_ROOT="$WORK_DIR/dmg-root"
APP_DIR="$DMG_ROOT/Ergon.app"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cp "$BINARY" "$APP_DIR/Contents/MacOS/ergon"
chmod 0755 "$APP_DIR/Contents/MacOS/ergon"

cat > "$APP_DIR/Contents/MacOS/ergon-launcher" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
APP_DIR="$(cd "$(dirname "$0")" && pwd)"
PORT="${ERGON_PORT:-8787}"
HOST="${ERGON_HOST:-127.0.0.1}"
DATA_DIR="${ERGON_DATA_DIR:-$HOME/Library/Application Support/Ergon}"
mkdir -p "$DATA_DIR"
export ERGON_DATA_DIR="$DATA_DIR"
export ERGON_HOST="$HOST"
export ERGON_PORT="$PORT"

if curl -fsS "http://127.0.0.1:${PORT}/api/version" >/dev/null 2>&1; then
  open "http://127.0.0.1:${PORT}/"
  exit 0
fi

exec "$APP_DIR/ergon" --app --host "$HOST" --port "$PORT"
EOF
chmod 0755 "$APP_DIR/Contents/MacOS/ergon-launcher"

sed "s/__VERSION__/${VERSION}/g" "$SCRIPT_DIR/Info.plist.template" > "$APP_DIR/Contents/Info.plist"
chmod 0644 "$APP_DIR/Contents/Info.plist"

ln -s /Applications "$DMG_ROOT/Applications"

cat > "$DMG_ROOT/Install Background Service.command" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [ ! -x "/Applications/Ergon.app/Contents/MacOS/ergon" ]; then
  echo "Drag Ergon.app into Applications first, then run this helper again."
  exit 1
fi
"/Applications/Ergon.app/Contents/MacOS/ergon" service install
EOF
chmod 0755 "$DMG_ROOT/Install Background Service.command"

if [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
  echo "Signing Ergon.app for DMG with identity: $APPLE_SIGNING_IDENTITY"
  codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$APP_DIR/Contents/MacOS/ergon"
  codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$APP_DIR/Contents/MacOS/ergon-launcher"
  codesign --force --deep --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$APP_DIR"
  codesign --verify --deep --strict --verbose=2 "$APP_DIR"
else
  echo "WARNING: APPLE_SIGNING_IDENTITY is not set; building UNSIGNED .dmg." >&2
fi

FINAL_DMG="$OUT_DIR/ergon-${VERSION}-${ARCH}.dmg"
rm -f "$FINAL_DMG"
hdiutil create \
  -volname "Ergon ${VERSION}" \
  -srcfolder "$DMG_ROOT" \
  -ov \
  -format UDZO \
  "$FINAL_DMG"

if [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
  codesign --force --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$FINAL_DMG"
  codesign --verify --verbose=2 "$FINAL_DMG"
fi

if [ -n "${APPLE_SIGNING_IDENTITY:-}" ] && \
   [ -n "${APPLE_NOTARY_KEY_ID:-}" ] && \
   [ -n "${APPLE_NOTARY_ISSUER_ID:-}" ] && \
   [ -n "${APPLE_NOTARY_KEY_P8:-}" ]; then
  NOTARY_KEY_FILE="$WORK_DIR/AuthKey.p8"
  printf '%s\n' "$APPLE_NOTARY_KEY_P8" > "$NOTARY_KEY_FILE"
  chmod 0600 "$NOTARY_KEY_FILE"
  echo "Submitting $FINAL_DMG to Apple notarization service..."
  xcrun notarytool submit "$FINAL_DMG" \
    --key "$NOTARY_KEY_FILE" \
    --key-id "$APPLE_NOTARY_KEY_ID" \
    --issuer "$APPLE_NOTARY_ISSUER_ID" \
    --wait
  xcrun stapler staple "$FINAL_DMG"
  xcrun stapler validate "$FINAL_DMG"
else
  echo "WARNING: Apple notarization secrets not complete; skipping notarytool/stapler for .dmg." >&2
fi

echo "Built macOS disk image: $FINAL_DMG"
