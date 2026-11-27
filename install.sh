#!/usr/bin/env bash
# Ergon installer — installs the binary, initializes TLS certificates in the
# data directory, creates the launcher, and (on systemd user hosts) registers
# a user service.
#
#   ./install.sh [--prefix DIR] [--data DIR] [--no-service] [--no-tls]
#
# Defaults: prefix ~/.local, data ~/.local/share/ergon, HTTPS/TLS enabled with
# an auto-generated self-signed cert in $DATA_DIR/tls/{cert.pem,key.pem} and
# multi-user login enabled. Set ERGON_TLS_CERT + ERGON_TLS_KEY for CA-signed
# certificates, or ERGON_OIDC_* for SSO.
set -euo pipefail

PREFIX="${HOME}/.local"
DATA_DIR="${HOME}/.local/share/ergon"
SERVICE=1
TLS=1
while [ $# -gt 0 ]; do
  case "$1" in
    --prefix)
      [ $# -ge 2 ] || { echo "error: --prefix needs a value" >&2; exit 2; }
      PREFIX="$2"; shift 2 ;;
    --data)
      [ $# -ge 2 ] || { echo "error: --data needs a value" >&2; exit 2; }
      DATA_DIR="$2"; shift 2 ;;
    --no-service) SERVICE=0; shift ;;
    --no-tls) TLS=0; shift ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

# Paths are made ABSOLUTE before anything is generated from them: a
# relative prefix produced a systemd unit systemd could never start
# (ExecStart must be absolute) and a launcher whose data dir moved
# with the caller's cwd.
mkdir -p "$PREFIX" "$DATA_DIR"
PREFIX="$(cd "$PREFIX" && pwd)"
DATA_DIR="$(cd "$DATA_DIR" && pwd)"

# Single-quote a value for safe embedding in the generated launcher
# script. Paths are data, never code: a value containing $(...) or
# backticks must be written literally, not executed when the
# launcher runs.
shq() {
  printf "'%s'" "$(printf '%s' "$1" | sed "s/'/'\\\\''/g")"
}

ROOT="$(cd "$(dirname "$0")" && pwd)"
BIN_SRC=""

# Prefer a prebuilt binary shipped next to this script; else build.
if [ -x "$ROOT/ergon" ]; then
  BIN_SRC="$ROOT/ergon"
elif [ -x "$ROOT/target/release/ergon" ]; then
  BIN_SRC="$ROOT/target/release/ergon"
else
  CARGO="$(command -v cargo || true)"
  if [ -z "$CARGO" ] && [ -x "$HOME/.local/rust/bin/cargo" ]; then
    CARGO="$HOME/.local/rust/bin/cargo"
  fi
  if [ -z "$CARGO" ] && [ -x "$HOME/.cargo/bin/cargo" ]; then
    CARGO="$HOME/.cargo/bin/cargo"
  fi
  if [ -z "$CARGO" ]; then
    echo "error: no prebuilt ergon binary found and cargo is not installed." >&2
    echo "Install Rust (https://rustup.rs) or place a built 'ergon' binary next to this script." >&2
    exit 1
  fi
  echo "Building Ergon (release)…"
  (cd "$ROOT" && "$CARGO" build --release)
  BIN_SRC="$ROOT/target/release/ergon"
fi

mkdir -p "$PREFIX/bin" "$DATA_DIR"
install -m 0755 "$BIN_SRC" "$PREFIX/bin/ergon"
echo "Installed: $PREFIX/bin/ergon"
echo "Data dir:  $DATA_DIR (people, accounts, TLS certs, library and work records live here)"

if [ "$TLS" = "1" ]; then
  ERGON_DATA_DIR="$DATA_DIR" "$PREFIX/bin/ergon" tls init
fi

# A launcher that carries the configuration in one obvious place.
# Values are embedded single-quoted (see shq) — never interpolated
# raw into the script.
{
  printf '%s\n' '#!/usr/bin/env bash'
  printf '%s\n' '# Ergon launcher — set the model key, TLS certs, or OIDC settings here.'
  printf '%s\n' '# Credentials are never hardcoded into the binary; read from env or 0600 data files.'
  printf 'export ERGON_DATA_DIR=%s\n' "$(shq "$DATA_DIR")"
  printf '%s\n' 'export ERGON_PORT="${ERGON_PORT:-8787}"'
  printf '%s\n' 'export ERGON_NO_OPEN="${ERGON_NO_OPEN:-1}"'
  if [ "$TLS" = "1" ]; then
    printf '%s\n' 'export ERGON_TLS="${ERGON_TLS:-1}"'
    printf '%s\n' 'export ERGON_AUTH_REQUIRED="${ERGON_AUTH_REQUIRED:-1}"'
  else
    printf '%s\n' 'export ERGON_TLS="${ERGON_TLS:-0}"'
  fi
  printf '%s\n' '# export ERGON_TLS_CERT=""             # optional CA-signed PEM certificate path'
  printf '%s\n' '# export ERGON_TLS_KEY=""              # optional CA-signed PEM private key path'
  printf '%s\n' '# export ERGON_MODEL_API_KEY=""        # model API key'
  printf '%s\n' '# export ERGON_OIDC_ISSUER=""          # optional OIDC SSO issuer URL'
  printf '%s\n' '# export ERGON_OIDC_CLIENT_ID=""       # optional OIDC client ID'
  printf '%s\n' '# export ERGON_GATE_TOKEN=""           # optional admin-only break-glass token'
  printf 'exec %s "$@"\n' "$(shq "$PREFIX/bin/ergon")"
} > "$PREFIX/bin/ergon-serve"
chmod 0755 "$PREFIX/bin/ergon-serve"
echo "Launcher:  $PREFIX/bin/ergon-serve"

SERVICE_READY=0
if [ "$SERVICE" = "1" ] && command -v systemctl >/dev/null 2>&1; then
  UNIT_DIR="${HOME}/.config/systemd/user"
  mkdir -p "$UNIT_DIR"
  cat > "$UNIT_DIR/ergon.service" <<EOF
[Unit]
Description=Ergon — an AI agent for every employee
After=network.target

[Service]
ExecStart="$PREFIX/bin/ergon-serve"
Restart=on-failure
# Environment=ERGON_MODEL_API_KEY=
# Environment=ERGON_GATE_TOKEN=

[Install]
WantedBy=default.target
EOF
  echo "Service:   $UNIT_DIR/ergon.service (unit written)"
  # The unit being written is not the service being ready: only
  # claim readiness when a user bus actually answers and reloads.
  if systemctl --user daemon-reload 2>/dev/null; then
    SERVICE_READY=1
    echo "           user bus reachable — daemon reloaded; enable with: systemctl --user enable --now ergon"
  else
    echo "           NOTE: no systemd user bus answered from this shell, so the service is NOT"
    echo "           verified ready. From a logged-in session run: systemctl --user enable --now ergon"
    echo "           — or just start Ergon directly with ergon-serve."
  fi
fi

SCHEME="https"
[ "$TLS" = "1" ] || SCHEME="http"
cat <<EOF

Next steps:
  1. Add the model key: edit ERGON_MODEL_API_KEY into $PREFIX/bin/ergon-serve
     (or the service unit). Without a key Ergon runs deterministic workflows
     and refuses to fabricate LLM answers.
  2. Start it:  ergon-serve   (or: systemctl --user enable --now ergon)
  3. Open ${SCHEME}://127.0.0.1:8787 to create the initial admin account (or run
     ERGON_DATA_DIR="$DATA_DIR" ergon user add admin --role admin --password ...).
     The field phone view is at ${SCHEME}://127.0.0.1:8787/phone behind the same
     HTTPS/TLS and login gate.
EOF
if [ "$SERVICE" = "1" ] && [ "$SERVICE_READY" = "0" ]; then
  echo "  (Reminder: the systemd service above was written but not verified against a live user bus.)"
fi
