#!/usr/bin/env bash
# Ergon installer — installs the binary, creates the data directory,
# and (on systemd user hosts) registers a user service.
#
#   ./install.sh [--prefix DIR] [--data DIR] [--no-service]
#
# Defaults: prefix ~/.local, data ~/.local/share/ergon.
# Ergon itself has no accounts and no passwords: access is gated by
# whoever hosts it. To require the hosting layer's gate header, set
# ERGON_GATE_TOKEN in the service environment (see the printed unit).
set -euo pipefail

PREFIX="${HOME}/.local"
DATA_DIR="${HOME}/.local/share/ergon"
SERVICE=1
while [ $# -gt 0 ]; do
  case "$1" in
    --prefix)
      [ $# -ge 2 ] || { echo "error: --prefix needs a value" >&2; exit 2; }
      PREFIX="$2"; shift 2 ;;
    --data)
      [ $# -ge 2 ] || { echo "error: --data needs a value" >&2; exit 2; }
      DATA_DIR="$2"; shift 2 ;;
    --no-service) SERVICE=0; shift ;;
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
  (cd "$ROOT" && "$CARGO" build --release --locked)
  BIN_SRC="$ROOT/target/release/ergon"
fi

mkdir -p "$PREFIX/bin" "$DATA_DIR"
install -m 0755 "$BIN_SRC" "$PREFIX/bin/ergon"
echo "Installed: $PREFIX/bin/ergon"
echo "Data dir:  $DATA_DIR (people, library, suites and work records live here)"

# Dedicated owner-only secrets/environment file (mode 0600) sourced by
# ergon-serve and loaded by systemd via EnvironmentFile=-...
ENV_FILE="$DATA_DIR/ergon.env"
if [ ! -f "$ENV_FILE" ]; then
  (
    umask 077
    cat > "$ENV_FILE" <<'EOF'
# Ergon runtime environment & secrets (mode 0600 — owner-readable only).
# Format: KEY=VALUE (compatible with both systemd EnvironmentFile and ergon-serve).
# ERGON_MODEL_API_KEY=
# ERGON_GATE_TOKEN=
# ERGON_ALLOWED_HOSTS=
# ERGON_SHUTDOWN_GRACE_SECS=10
EOF
  )
fi
chmod 0600 "$ENV_FILE"
echo "Env file:  $ENV_FILE (mode 0600 — keep API keys and tokens here)"

# A private launcher (mode 0700) that sources $DATA_DIR/ergon.env.
# Values are embedded single-quoted (see shq) — never interpolated
# raw into the script.
(
  umask 077
  {
    printf '%s\n' '#!/usr/bin/env bash'
    printf '%s\n' '# Ergon launcher — secrets live in $ERGON_ENV_FILE (mode 0600), not here.'
    printf 'ERGON_ENV_FILE=%s\n' "$(shq "$ENV_FILE")"
    printf '%s\n' 'if [ -f "$ERGON_ENV_FILE" ]; then'
    printf '%s\n' '  set -a'
    printf '%s\n' '  # shellcheck disable=SC1090'
    printf '%s\n' '  . "$ERGON_ENV_FILE"'
    printf '%s\n' '  set +a'
    printf '%s\n' 'fi'
    printf 'export ERGON_DATA_DIR=%s\n' "$(shq "$DATA_DIR")"
    printf '%s\n' 'export ERGON_PORT="${ERGON_PORT:-8787}"'
    printf '%s\n' 'export ERGON_NO_OPEN="${ERGON_NO_OPEN:-1}"'
    printf 'exec %s "$@"\n' "$(shq "$PREFIX/bin/ergon")"
  } > "$PREFIX/bin/ergon-serve"
)
chmod 0700 "$PREFIX/bin/ergon-serve"
echo "Launcher:  $PREFIX/bin/ergon-serve (mode 0700)"

SERVICE_READY=0
if [ "$SERVICE" = "1" ] && command -v systemctl >/dev/null 2>&1; then
  UNIT_DIR="${HOME}/.config/systemd/user"
  mkdir -p "$UNIT_DIR"
  cat > "$UNIT_DIR/ergon.service" <<EOF
[Unit]
Description=Ergon — an AI agent for every employee
After=network.target

[Service]
EnvironmentFile=-$ENV_FILE
ExecStart="$PREFIX/bin/ergon-serve"
Restart=on-failure

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
  if command -v loginctl >/dev/null 2>&1; then
    CUR_USER="${USER:-$(id -un)}"
    LINGER_LINE="$(loginctl show-user "$CUR_USER" -p Linger 2>/dev/null || true)"
    LINGER_VAL="${LINGER_LINE#Linger=}"
    if [ "$LINGER_VAL" != "yes" ]; then
      echo "           NOTE: systemd user linger is not enabled for $CUR_USER (Linger=${LINGER_VAL:-unknown})."
      echo "           On headless Linux servers, user services stop when you log out."
      echo "           Enable linger once with: sudo loginctl enable-linger $CUR_USER"
    fi
  fi
fi

cat <<EOF

Next steps:
  1. Add the model key: edit ERGON_MODEL_API_KEY in $ENV_FILE
     (mode 0600, owner-readable only), from your OpenCode Go account. Without a key
     Ergon runs but honestly refuses to invent answers.
  2. Start it:  ergon-serve   (or: systemctl --user enable --now ergon)
  3. Open http://127.0.0.1:8787 — desktop faces (Work Graph, Conversation,
     Today). The phone app is the same server at http://127.0.0.1:8787/phone —
     bookmark it on each phone; there are no Ergon passwords to manage.
EOF
if [ "$SERVICE" = "1" ] && [ "$SERVICE_READY" = "0" ]; then
  echo "  (Reminder: the systemd service above was written but not verified against a live user bus.)"
fi
