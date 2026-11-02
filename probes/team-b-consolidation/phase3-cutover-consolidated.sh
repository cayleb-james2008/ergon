#!/usr/bin/bash
# PHASE 3 (LIVE CUTOVER) — BLOCKED until: Nox rulings (acceptance delta + single-setup interpretation),
# Lead approval at an app-idle moment, and the fleet-mcp row question is resolved (consolidated build variant
# OR an explicit "acceptance-only unit exception" ruling). Single-writer: the old unit must stop before the
# consolidated fleet profile starts; never both at once.
set -euo pipefail
DESKTOP_HOME=${DESKTOP_HOME:-$HOME/.local/opt/deepseek-harness-official/apps/desktop/.desktop-build/development/home}
CLI=${CLI:-$HOME/.local/opt/deepseek-harness-official/apps/cli/lib/bin.js}
RUNDIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/fleet"
echo "1/4 stopping the current fleet unit (single-writer rule)"; systemctl --no-ask-password stop fleet-dsh
systemctl is-active fleet-dsh && { echo "unit still active — abort"; exit 1; } || true
echo "2/4 starting the consolidated fleet host (user level)"; 
DSH_HOME="$DESKTOP_HOME" setsid nohup node "$CLI" --profile fleet --no-open --host 127.0.0.1 --port 0 > "$HOME/.local/state/dsh-fleet-consolidated.log" 2>&1 &
echo $! > "$HOME/.local/state/dsh-fleet-consolidated.pid"
echo "3/4 waiting for the control socket $RUNDIR/control.sock"; for i in $(seq 1 60); do [ -S "$RUNDIR/control.sock" ] && break; sleep 1; done
echo "4/4 identity readback"; DSH_FLEET_SOCKET="$RUNDIR/control.sock" node "$DESKTOP_HOME/profiles/fleet/node_modules/@cayleb/dsh-fleet/bin/dsh-fleet.mjs" op status
