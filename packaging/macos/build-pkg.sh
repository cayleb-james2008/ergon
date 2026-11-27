#!/usr/bin/env bash
# Builds Ergon.app + LaunchAgent + macOS .pkg installer via pkgbuild & productbuild.
# Automatically signs (codesign + productsign/productbuild --sign) and notarizes
# (xcrun notarytool + xcrun stapler) when Apple signing env vars are configured;
# builds cleanly unsigned with a warning when signing secrets are absent.
#
# Usage:
#   ./packaging/macos/build-pkg.sh --binary <path> --version <semver> [--arch aarch64-apple-darwin] [--out-dir dist]
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

STAGE_ROOT="$WORK_DIR/payload"
APP_DIR="$STAGE_ROOT/Applications/Ergon.app"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources" "$STAGE_ROOT/Library/LaunchAgents"

cp "$BINARY" "$APP_DIR/Contents/MacOS/ergon"
chmod 0755 "$APP_DIR/Contents/MacOS/ergon"

# Launcher script inside Ergon.app: starts the local server if needed and opens the app window.
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
cp "$SCRIPT_DIR/com.ergon.agent.plist.template" "$STAGE_ROOT/Library/LaunchAgents/com.ergon.agent.plist"
chmod 0644 "$APP_DIR/Contents/Info.plist" "$STAGE_ROOT/Library/LaunchAgents/com.ergon.agent.plist"

# Sign the binary and Ergon.app bundle when Developer ID Application identity is configured.
if [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
  echo "Signing Ergon.app with identity: $APPLE_SIGNING_IDENTITY"
  codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$APP_DIR/Contents/MacOS/ergon"
  codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$APP_DIR/Contents/MacOS/ergon-launcher"
  codesign --force --deep --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$APP_DIR"
  codesign --verify --deep --strict --verbose=2 "$APP_DIR"
else
  echo "WARNING: APPLE_SIGNING_IDENTITY is not set; Ergon.app will be unsigned." >&2
fi

SCRIPTS_DIR="$WORK_DIR/scripts"
mkdir -p "$SCRIPTS_DIR"
cp "$SCRIPT_DIR/scripts/postinstall" "$SCRIPTS_DIR/postinstall"
chmod 0755 "$SCRIPTS_DIR/postinstall"

COMP_PKG="$WORK_DIR/ergon-component.pkg"
pkgbuild \
  --root "$STAGE_ROOT" \
  --scripts "$SCRIPTS_DIR" \
  --identifier "com.ergon.pkg" \
  --version "$VERSION" \
  --install-location "/" \
  "$COMP_PKG"

DIST_XML="$WORK_DIR/distribution.xml"
sed "s/__VERSION__/${VERSION}/g" "$SCRIPT_DIR/distribution.xml.template" > "$DIST_XML"

FINAL_PKG="$OUT_DIR/ergon-${VERSION}-${ARCH}.pkg"
if [ -n "${APPLE_INSTALLER_IDENTITY:-}" ]; then
  echo "Building and signing installer package: $FINAL_PKG"
  productbuild \
    --distribution "$DIST_XML" \
    --package-path "$WORK_DIR" \
    --sign "$APPLE_INSTALLER_IDENTITY" \
    --timestamp \
    "$FINAL_PKG"
  pkgutil --check-signature "$FINAL_PKG"
else
  echo "WARNING: APPLE_INSTALLER_IDENTITY is not set; building UNSIGNED .pkg installer." >&2
  productbuild \
    --distribution "$DIST_XML" \
    --package-path "$WORK_DIR" \
    "$FINAL_PKG"
fi

# Notarize and staple the .pkg when App Store Connect API credentials are present.
if [ -n "${APPLE_INSTALLER_IDENTITY:-}" ] && \
   [ -n "${APPLE_NOTARY_KEY_ID:-}" ] && \
   [ -n "${APPLE_NOTARY_ISSUER_ID:-}" ] && \
   [ -n "${APPLE_NOTARY_KEY_P8:-}" ]; then
  NOTARY_KEY_FILE="$WORK_DIR/AuthKey.p8"
  printf '%s\n' "$APPLE_NOTARY_KEY_P8" > "$NOTARY_KEY_FILE"
  chmod 0600 "$NOTARY_KEY_FILE"
  echo "Submitting $FINAL_PKG to Apple notarization service..."
  xcrun notarytool submit "$FINAL_PKG" \
    --key "$NOTARY_KEY_FILE" \
    --key-id "$APPLE_NOTARY_KEY_ID" \
    --issuer "$APPLE_NOTARY_ISSUER_ID" \
    --wait
  xcrun stapler staple "$FINAL_PKG"
  xcrun stapler validate "$FINAL_PKG"
else
  echo "WARNING: Apple notarization secrets not complete; skipping notarytool/stapler for .pkg." >&2
fi

echo "Built macOS package: $FINAL_PKG"
