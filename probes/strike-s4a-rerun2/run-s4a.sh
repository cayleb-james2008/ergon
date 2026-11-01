#!/usr/bin/bash
# CHECKER harness — S4 part A: package-alone install proof in a throwaway DSH_HOME.
# Independent of the Lead's scratch-gate.sh (own run and own verification). Never touches
# /var/lib/fleet, /var/lib/fleet-drop, the fleet service or the owner's profile.
set -uo pipefail
# The fleet service runs under the wrapper's `umask 027`; mirror it so pnpm's
# masked modes (0644->0640, 0755->0750) are exercised, then normalise restores the archive modes.
umask 027
EVID=/home/cayleb/workspace/harness-build/evidence/strike/checker-verify/s4a-rerun2
TGZ=/home/cayleb/workspace/harness-build/strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz
DSH=/home/cayleb/.local/opt/deepseek-harness-official/apps/cli/lib/bin.js
# Use a port this checker owns: the Lead's earlier scratch listener/driver may hold 8789.
# 0 in bind() => kernel assigns a free port on 127.0.0.1; the number is recorded in evidence.
if [ -z "${MCP_PORT:-}" ]; then
  MCP_PORT=$(python3 -c "import socket; s=socket.socket(); s.bind(('127.0.0.1',0)); print(s.getsockname()[1]); s.close()")
fi
ROOT=${ROOT:-$(mktemp -d /tmp/checker-s4a.XXXXXX)}
export DSH_HOME=$ROOT/dsh
# `dsh plugin` forwards to `pnpm` on PATH. The desktop PATH shim needs env a harness CLI
# child does not provide, so use the DSH installation's own pinned pnpm (11.28.5 = the
# checkout's packageManager pin) through a one-binary shim dir — the analogue of the
# sanctioned installer's installation-owned pnpm in /var/lib/fleet/npm/bin.
mkdir -p "$ROOT/bin"
ln -sf /home/cayleb/.local/opt/deepseek-harness-official/node_modules/.bin/pnpm "$ROOT/bin/pnpm"
export PATH="$ROOT/bin:/usr/bin:/bin"
# Keep pnpm's content-addressable store inside the scratch root (same fs => hard links work).
export npm_config_store_dir="$ROOT/pnpm-store"
FAILS=0
mkdir -p "$ROOT"/fleet/{dsh,state,log,work,backups,checks,skills} "$ROOT"/ctl "$ROOT/fleet/state/mcp" "$EVID/logs"
PKG="$DSH_HOME/profiles/fleet/node_modules/@cayleb/dsh-fleet"
HOST_PID=""
cleanup() {
  if [ -n "$HOST_PID" ] && kill -0 "$HOST_PID" 2>/dev/null; then
    kill "$HOST_PID" 2>/dev/null; for _ in 1 2 3 4 5; do kill -0 "$HOST_PID" 2>/dev/null || break; sleep 1; done
    kill -9 "$HOST_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT
step() { echo; echo "################ $* ################"; }
bad() { echo "FAIL  $*"; FAILS=$((FAILS + 1)); }
good() { echo "PASS  $*"; }

step "0 environment"
{
  echo "date=$(date -Is)"; echo "host=$(hostname) user=$(id -un) uid=$(id -u) groups=$(id -Gn)"
  echo "root=$ROOT"; echo "DSH_HOME=$DSH_HOME"; echo "tarball=$TGZ"
  sha256sum "$TGZ"
  echo "node=$(node --version) dsh_cli=$DSH"
  echo "pnpm_path=$(command -v pnpm) pnpm_real=$(readlink -f "$(command -v pnpm)") pnpm_version=$(pnpm --version 2>&1)"
  echo "umask=$(umask) pnpm_store=$npm_config_store_dir mcp_port=$MCP_PORT"
  echo "port_$MCP_PORT=$(python3 -c "
import socket
s=socket.socket(); s.settimeout(0.5)
try: s.bind(('127.0.0.1',$MCP_PORT)); print('FREE')
except OSError as e: print('IN USE', e)
finally: s.close()")"
} | tee "$EVID/logs/10-env.txt"

step "1 profile init from the default web template"
timeout 300 node "$DSH" fleet --from-default-profile web --dump-config >"$ROOT/dump0.yml" 2>"$ROOT/dump0.err"
rc=$?
echo "init rc=$rc dump0_bytes=$(wc -c <"$ROOT/dump0.yml") dump0_rows=$(grep -c '^- id:' "$ROOT/dump0.yml")" | tee "$EVID/logs/11-profile-init.txt"
[ $rc -eq 0 ] || { bad "profile init rc=$rc"; tail -5 "$ROOT/dump0.err"; }
cp "$ROOT/dump0.yml" "$ROOT/dump0.err" "$EVID/logs/" 2>/dev/null || true

step "2 install ONLY the release tarball: dsh plugin --profile fleet add --save-exact --ignore-scripts"
start=$(date +%s)
timeout 900 node "$DSH" plugin --profile fleet add --save-exact --ignore-scripts "$TGZ" >"$ROOT/add.log" 2>&1
rc=$?
{
  echo "plugin-add rc=$rc duration_s=$(( $(date +%s) - start ))"
  echo "--- add.log ---"; cat "$ROOT/add.log"
} >"$EVID/logs/12-plugin-add.log"
cat "$EVID/logs/12-plugin-add.log"
[ $rc -eq 0 ] || bad "plugin add rc=$rc"
if [ -f "$PKG/package.json" ]; then
  good "installed package present at $PKG"
  echo "installed_version=$(node -p "require('$PKG/package.json').version" 2>&1)" | tee "$EVID/logs/13-installed-identity.txt"
  {
    echo "--- profile manifest ---"; cat "$DSH_HOME/profiles/fleet/package.json"
    echo "--- dsh.profile.bundles ---"; node -p "JSON.stringify(require('$DSH_HOME/profiles/fleet/package.json').dsh.profile.bundles)" 2>&1
    echo "--- installed package dirs ---"; find "$DSH_HOME/profiles/fleet/node_modules/@cayleb/dsh-fleet" -type f | wc -l
  } | tee -a "$EVID/logs/13-installed-identity.txt"
else
  bad "installed package missing at $PKG"
fi

step "3 pnpm post-install state BEFORE normalise (nlink / masked modes)"
python3 - "$PKG" "$TGZ" >"$EVID/logs/14-pnpm-state.txt" 2>&1 <<'PY'
import os, stat, sys, tarfile
from collections import Counter
pkg, tgz = sys.argv[1], sys.argv[2]
rows = []
for dp, dns, fns in os.walk(pkg):
    dns[:] = [d for d in dns if d != 'node_modules']
    for fn in fns:
        p = os.path.join(dp, fn)
        st = os.lstat(p)
        rows.append((os.path.relpath(p, pkg), stat.S_IMODE(st.st_mode), st.st_nlink))
arch = {}
with tarfile.open(tgz, 'r:gz') as tf:
    for m in tf.getmembers():
        if m.isfile():
            arch[m.name[len('package/'):]] = m.mode & 0o7777
masked = {0o644: 0o640, 0o755: 0o750}
print(f'installed_files={len(rows)} archive_members={len(arch)}')
print('installed mode distribution:', dict(Counter(oct(m) for _r, m, _n in rows)))
print('installed nlink distribution:', dict(Counter(n for _r, _m, n in rows)))
archmodes = Counter(oct(arch[r]) for r in arch)
pre_ok = sum(1 for r, m, _n in rows if r in arch and m == arch[r])
print('archive mode distribution:', dict(archmodes))
print(f'pre-normalise files already at archive mode: {pre_ok}/{len(rows)}  (normalise must set the rest)')
print('pre-normalise nlink>1 files:', sum(1 for _r, _m, n in rows if n > 1), '| nlink==1:', sum(1 for _r, _m, n in rows if n == 1))
print('NOTE: pre-normalise modes are pnpm+umask output, not the install result; step 4 sets exact archive modes and step 4b re-asserts them.')
PY
cat "$EVID/logs/14-pnpm-state.txt"

step "4 apply the reviewed normalise step to the scratch tree (as the sanctioned installer does)"
python3 -I -S -B "$EVID/../wrapper-review/normalise-heredoc.py" "$TGZ" "$PKG" >"$ROOT/norm.out" 2>"$ROOT/norm.err"
rc=$?
{
  echo "normalise rc=$rc"
  echo "stdout: $(cat "$ROOT/norm.out")"
  echo "stderr: $(cat "$ROOT/norm.err")"
} | tee "$EVID/logs/15-normalise.txt"
[ $rc -eq 0 ] || bad "normalise rc=$rc"
python3 "$EVID/../wrapper-review/assert-normalised.py" "$TGZ" "$PKG" >"$EVID/logs/16-assert-normalised.txt" 2>&1
rc=$?
echo "independent assert rc=$rc" >>"$EVID/logs/16-assert-normalised.txt"
cat "$EVID/logs/16-assert-normalised.txt"
[ $rc -eq 0 ] || bad "post-normalise assert rc=$rc"

step "5 scratch overlay + composed profile"
age-keygen -o "$ROOT/recovery.key" 2>/dev/null || bad "age-keygen failed"
age-keygen -y "$ROOT/recovery.key" >"$ROOT/recovery.pub" 2>/dev/null || bad "age-keygen -y failed"
cat >"$ROOT/overlay.yml" <<EOF
- id: fleet-core
  config:
    fleetRoot: $ROOT/fleet
    dbPath: $ROOT/fleet/state/fleet.db
    logDir: $ROOT/fleet/log
    workRoot: $ROOT/fleet/work
    controlSocket: $ROOT/ctl/control.sock
    sandboxProviderId: fleet-sandbox
    peerUid: $(id -u)
    denyCgroupUnits: []
- id: fleet-sandbox
  config:
    bwrapPath: /usr/bin/bwrap
    nodePath: /usr/bin/node
    maskRoots: [$ROOT/fleet/dsh, $ROOT/fleet/state, $ROOT/fleet/log, $ROOT/fleet/work, $ROOT/fleet/backups, $ROOT/fleet/checks]
    egressSocket: $ROOT/ctl/egress.sock
    tmpSizeMiB: 512
- id: fleet-egress
  config:
    socketPath: $ROOT/ctl/egress.sock
    logPath: $ROOT/fleet/log/egress.jsonl
    allow: [registry.npmjs.org]
    allowPorts: [80, 443]
    connectTimeoutMs: 10000
    idleTimeoutMs: 300000
    guardInProcess: true
- id: fleet-lessons
  config:
    skillDir: $ROOT/fleet/skills
- id: fleet-housekeeper
  config:
    fleetRoot: $ROOT/fleet
    recoveryFile: $ROOT/recovery.pub
    agePath: /usr/bin/age
    ageKeygenPath: /usr/bin/age-keygen
- id: session-query-sqlite
  config:
    path: $ROOT/fleet/state/session-query.sqlite
    openAt: first-search
- id: fleet-policy
  config:
    logFile: $ROOT/fleet/log/actions.jsonl
    confirmDir: $ROOT/fleet/state/confirms
    peerUid: $(id -u)
- id: fleet-mcp
  config:
    port: $MCP_PORT
    bindSourceAllow: [loopback, tailnet]
    bindHosts: [127.0.0.1]
    statePath: $ROOT/fleet/state/mcp
    logPath: $ROOT/fleet/log/mcp-actions.jsonl
    adminSocket: $ROOT/ctl/mcp-admin.sock
    fleetRoot: $ROOT/fleet
    peerUid: $(id -u)
    presetPrefix: fleet-
    stubProvider: fleet-selftest-stub
    stubModel: fleet-selftest-stub
    maxBodyBytes: 262144
    requestsPerMinute: 60
    maxSessions: 8
    turnTimeoutMs: 120000
    toolTimeoutMs: 180000
    controlTimeoutMs: 120000
    roundtripWaitMs: 30000
    pollMs: 500
    koffiPath: ''
    serverName: fleet-mcp
    protocolVersions: [2025-03-26, 2024-11-05]
    selftestAllowTools: []
EOF
cp "$ROOT/overlay.yml" "$EVID/logs/17-overlay.yml"
timeout 300 node "$DSH" --profile fleet --patch "$ROOT/overlay.yml" --dump-config >"$ROOT/dump1.yml" 2>"$ROOT/dump1.err"
rc=$?
echo "overlay dump-config rc=$rc dump1_bytes=$(wc -c <"$ROOT/dump1.yml")" | tee "$EVID/logs/18-compose.txt"
[ $rc -eq 0 ] || { bad "overlay dump-config rc=$rc"; tail -5 "$ROOT/dump1.err"; }
cp "$ROOT/dump1.yml" "$ROOT/dump1.err" "$EVID/logs/" 2>/dev/null || true
python3 "$EVID/check-compose.py" "$ROOT/dump1.yml" "$TGZ" "$ROOT" "$ROOT/dump0.yml" "$MCP_PORT" >"$ROOT/compose.txt" 2>&1
rc=$?
cat "$ROOT/compose.txt" | tee -a "$EVID/logs/18-compose.txt"
[ $rc -eq 0 ] || bad "compose check rc=$rc"

step "6 boot the scratch fleet host"
node "$DSH" --profile fleet --patch "$ROOT/overlay.yml" --no-open --host 127.0.0.1 --port 0 >"$ROOT/host.log" 2>&1 &
HOST_PID=$!
echo "host_pid=$HOST_PID"
ok=1
for _i in $(seq 1 120); do [ -S "$ROOT/ctl/control.sock" ] && { ok=0; break; }; kill -0 "$HOST_PID" 2>/dev/null || break; sleep 1; done
if [ $ok -eq 0 ]; then good "control socket appeared"; else bad "control socket did not appear"; tail -25 "$ROOT/host.log"; fi
sleep 2
echo "listening sockets:"; ss -ltn 2>/dev/null | grep -E ":$MCP_PORT\\b" || true
cp "$ROOT/host.log" "$EVID/logs/19-host.log" 2>/dev/null || true
echo "--- fleet-mcp host.log lines ---"; grep -i 'fleet-mcp\|mcp-admin\|token' "$ROOT/host.log" | sed 's/\b[0-9a-f]\{64\}\b/<redacted-64hex>/g' | head -10

step "7 control socket status (my own client)"
python3 - "$ROOT/ctl/control.sock" >"$ROOT/status.json" 2>"$ROOT/status.err" <<'PY'
import json, socket, sys, time
sock = sys.argv[1]
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM); s.settimeout(10)
s.connect(sock); s.sendall(b'{"op":"status"}\n')
buf = b''
while b'\n' not in buf:
    d = s.recv(65536)
    if not d: break
    buf += d
json.dump(json.loads(buf.split(b'\n')[0]), sys.stdout, indent=2)
PY
cat "$ROOT/status.err" 2>/dev/null
python3 - "$ROOT/status.json" >"$ROOT/status-check.txt" 2>&1 <<'PY'
import json, sys
j = json.load(open(sys.argv[1]))
r = j.get('result') or {}
integrity = r.get('integrity') or {}
checks = [
    ('status ok=true', j.get('ok') is True),
    ('integrity.ok=true', integrity.get('ok') is True),
    ('integrity.failures empty', integrity.get('failures') == []),
    ('lanes_allowed=true', r.get('lanes_allowed') is True),
    ('control.peer_uid=1000 accepted, 0 rejected', (r.get('control') or {}).get('peer_uid') == 1000 and (r.get('control') or {}).get('rejected') == 0),
    ('policy.selftest=true', (r.get('policy') or {}).get('selftest') is True),
    ('db journal_mode=wal', (r.get('db') or {}).get('journal_mode') == 'wal'),
]
fails = 0
for label, ok in checks:
    print(('PASS  ' if ok else 'FAIL  ') + label)
    fails += 0 if ok else 1
print(f'status-check: {"PASS" if fails == 0 else "FAIL"}')
sys.exit(0 if fails == 0 else 1)
PY
rc=$?
cat "$ROOT/status-check.txt" | tee "$EVID/logs/20-status.txt"
[ $rc -eq 0 ] || bad "control status checks rc=$rc"

step "8 dsh-fleet first-run --status vs tarball members"
DSH_FLEET_SOCKET="$ROOT/ctl/control.sock" timeout 120 node "$PKG/bin/dsh-fleet.mjs" first-run --status >"$ROOT/frstatus.json" 2>"$ROOT/frstatus.err"
rc=$?
echo "first-run --status rc=$rc bytes=$(wc -c <"$ROOT/frstatus.json")"
[ $rc -eq 0 ] || { bad "first-run --status rc=$rc"; tail -5 "$ROOT/frstatus.err"; }
cp "$ROOT/frstatus.json" "$ROOT/frstatus.err" "$EVID/logs/" 2>/dev/null || true
python3 "$EVID/compare-frstatus.py" "$ROOT/frstatus.json" "$TGZ" >"$ROOT/frcheck.txt" 2>&1
rc=$?
cat "$ROOT/frcheck.txt" | tee "$EVID/logs/21-firstrun-status.txt"
[ $rc -eq 0 ] || bad "first-run --status != tarball rc=$rc"

step "9 MCP facade: my own initialize / notifications / tools/list / auth.refuse"
python3 "$EVID/mcp-probe.py" "$ROOT/ctl/mcp-admin.sock" "http://127.0.0.1:$MCP_PORT/mcp" >"$ROOT/mcp.txt" 2>&1
rc=$?
cat "$ROOT/mcp.txt" | tee "$EVID/logs/22-mcp-probe.txt"
[ $rc -eq 0 ] || bad "mcp probe rc=$rc"

step "9b first-run --credentials under a REAL PTY (fake key; value never printed)"
FAKE="FAKE-CHECKER-CRED-$(python3 -c 'import secrets; print(secrets.token_hex(8))')"
printf '%s' "$FAKE" >"$ROOT/fake-value.txt"; chmod 0600 "$ROOT/fake-value.txt"
STORE="$DSH_HOME/.credentials.yaml"
{
  echo "store_exists_before=$( [ -e "$STORE" ] && echo YES || echo NO )"
  if [ -e "$STORE" ]; then
    echo "store_mode_before=$(stat -c '%a' "$STORE")"
    echo "store_top_level_keys_before=$(grep -E '^[A-Za-z_]+:' "$STORE" | cut -d: -f1 | tr '\n' ' ')"
    echo "store_refs_keys_before=$(awk '/^refs:/{f=1;next} /^[A-Za-z_]/{f=0} f && /^  [A-Z]/{print $1}' "$STORE" | tr -d ':' | tr '\n' ' ')"
    echo "opencode_ref_present_before=$(grep -cE '^  OPENCODE_GO_API_KEY:' "$STORE")"
  fi
} >"$ROOT/cred-pre-state.txt" 2>&1
cat "$ROOT/cred-pre-state.txt" | tee "$EVID/logs/23-credentials-pre-state.txt"
python3 "$EVID/pty-credentials.py" --node "$(command -v node)" --cli "$PKG/bin/dsh-fleet.mjs" \
  --store "$STORE" --socket "$ROOT/ctl/control.sock" --dsh-home "$DSH_HOME" \
  --value-file "$ROOT/fake-value.txt" --transcript "$ROOT/pty-credentials.txt" --timeout 120 \
  >"$ROOT/pty-driver.txt" 2>&1
rc=$?
cat "$ROOT/pty-driver.txt" | tee "$EVID/logs/23-pty-driver.txt"
[ $rc -eq 0 ] || bad "PTY credentials driver rc=$rc"
python3 "$EVID/verify-credentials.py" --store "$STORE" --transcript "$ROOT/pty-credentials.txt" \
  --value-file "$ROOT/fake-value.txt" --scratch-root "$ROOT" --pre-state "$ROOT/cred-pre-state.txt" \
  >"$ROOT/cred-verify.txt" 2>&1
rc=$?
cat "$ROOT/cred-verify.txt" | tee "$EVID/logs/24-credentials-verify.txt"
[ $rc -eq 0 ] || bad "credentials verification rc=$rc"
if grep -qF "$FAKE" "$ROOT/pty-credentials.txt"; then
  sed "s/$FAKE/<redacted-fake-credential>/g" "$ROOT/pty-credentials.txt" >"$EVID/logs/25-pty-transcript.redacted.txt"
  bad "fake value WAS present in the PTY transcript (redacted copy filed)"
else
  cp "$ROOT/pty-credentials.txt" "$EVID/logs/25-pty-transcript.txt"
  good "PTY transcript contains no fake value; filed verbatim"
fi
stat -c 'store=%n mode=%a size=%s owner=%U:%G' "$STORE" 2>/dev/null | tee "$EVID/logs/26-store-metadata.txt"
echo "store refs keys: $(grep -oE '^  [A-Z][A-Z0-9_]+:' "$STORE" 2>/dev/null | tr -d ' :' | tr '\n' ' ')" | tee -a "$EVID/logs/26-store-metadata.txt"

step "10 teardown scratch host"
kill "$HOST_PID" 2>/dev/null || true
for _i in 1 2 3 4 5 6 7 8 9 10; do kill -0 "$HOST_PID" 2>/dev/null || break; sleep 1; done
if kill -0 "$HOST_PID" 2>/dev/null; then kill -9 "$HOST_PID" 2>/dev/null || true; fi
HOST_PID=""
echo "scratch host stopped; scratch root kept at $ROOT"

echo
echo "############ S4A RESULT (final artifact 8173d13e) ############"
echo "scratch_root=$ROOT"
echo "FAILS=$FAILS"
if [ "$FAILS" -eq 0 ]; then echo "S4A: PASS"; exit 0; else echo "S4A: FAIL ($FAILS)"; exit 1; fi
