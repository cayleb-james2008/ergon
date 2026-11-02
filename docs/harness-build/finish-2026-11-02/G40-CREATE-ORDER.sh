#!/usr/bin/env bash
# G40-CREATE-ORDER.sh: creates the ONE throwaway G40 probe order (Nox's go, 2 Nov 18:08 ET). NOT RUN YET.
# Creating the order dispatches the Lead at once and a Builder runs the probe within minutes, so it is created only when Lane 6 (Team E)
# is in place to OBSERVE (Nox: option (i), wait for the DSH app to reopen). Refuses unless given --create AND the observer is named.
#   G40-CREATE-ORDER.sh --create --observer "<who confirmed Lane 6 is observing>"
set -euo pipefail
[ "${1:-}" = "--create" ] && [ "${2:-}" = "--observer" ] && [ -n "${3:-}" ] || { echo "REFUSING: run with --create --observer '<who confirmed Lane 6 is observing>'. This order starts a live probe the moment it exists."; exit 2; }
DH=/home/cayleb/.local/opt/deepseek-harness-official/apps/desktop/.desktop-build/development/home
P=$DH/profiles/fleet/node_modules/@cayleb/dsh-fleet
export DSH_FLEET_SOCKET=/run/user/1000/fleet/control.sock
DOC=/home/cayleb/workspace/harness-build/finish-docs/G40-ORDER.json
cd /tmp
python3 -I -c 'import json,sys; d=json.load(open(sys.argv[1])); assert d["title"].startswith("G40 PROBE") and d["tier"]==1 and d["budget"]["usd"]<=0.5, "unexpected order doc"; print("order doc ok:", d["title"])' "$DOC"
# nothing else may be in flight: refuse if any non-fixture order is open or in progress
LIVE=$(sqlite3 "file:$DH/fleet/state/fleet.db?mode=ro" "select count(*) from orders where fixture=0 and state not in ('done','failed')")
[ "$LIVE" = 0 ] || { echo "REFUSING: $LIVE live order(s) are not terminal"; exit 3; }
KEY=$(cat /proc/sys/kernel/random/uuid)
BODY=$(python3 -I -c 'import json,sys; print(json.dumps({"order":json.load(open(sys.argv[1])),"idempotency_key":sys.argv[2]}))' "$DOC" "$KEY")
echo "creating the G40 probe order; observer: $3; key $KEY"
timeout 40 node "$P/bin/dsh-fleet.mjs" op order-create "$BODY" | head -c 600
echo
logger -t g40 -- "G40 probe order created (Nox go 2 Nov 18:08 ET); observer named: $3; idempotency key $KEY" 2>/dev/null || true
