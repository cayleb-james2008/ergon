#!/usr/bin/bash
# ROLLBACK — stop the consolidated host, restore the previous state (unit path untouched until Phase 3 ran).
set -euo pipefail
PIDF="$HOME/.local/state/dsh-fleet-consolidated.pid"
[ -f "$PIDF" ] && kill "$(cat "$PIDF")" 2>/dev/null && echo "consolidated host stopped" || echo "no consolidated pid file"
rm -f "$PIDF"
echo "restarting the old unit (if it was the writer before):"; systemctl --no-ask-password start fleet-dsh 2>/dev/null || echo "(unit start not permitted in this context)"
systemctl is-active fleet-dsh || true
echo "desktop profile was never modified; profiles/fleet and <desktop-home>/fleet remain for inspection."
