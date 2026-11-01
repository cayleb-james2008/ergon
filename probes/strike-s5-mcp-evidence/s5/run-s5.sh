#!/usr/bin/env bash
# S5 — checker-verify INDEPENDENT MCP acceptance run on the FINAL release tarball.
#
# Boots a throwaway DSH profile + scratch-root fleet host from the exact release bytes (never touching the installed
# service, /var/lib/fleet, or the owner's profile), then runs checker-verify's own MCP probe over loopback and the
# host's tailnet address. The builder's gate script and mcp-selftest are cross-checks, never the primary verdict.
#
#   bash s5/run-s5.sh
#
# Exit 0 only when the probe passes every mandatory check. Evidence is written beside this script.
set -uo pipefail
HERE="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
TOOLS="$(cd -- "$HERE/../probes" && pwd)"
OUT="$HERE"
TGZ="${TGZ:-/home/cayleb/workspace/harness-build/strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz}"
EXPECT_SHA="${EXPECT_SHA:-b60019a0f3611c83fb13a374444c46b05092df89737f803661408fd7151a0b10}"
DSH="${DSH:-/home/cayleb/.local/opt/deepseek-harness-official/apps/cli/lib/bin.js}"
PORT="${MCP_PORT:-18889}"
LOG="$OUT/00-run-log.txt"
: > "$LOG"
say() { printf '%s\n' "$*" | tee -a "$LOG"; }
FAIL=0
fail() { say "FAIL: $*"; FAIL=$((FAIL+1)); }
trap 'say "── stopping scratch host"; [ -f "${ROOT:-/nonexistent}/host.pid" ] && kill "$(cat "$ROOT/host.pid")" 2>/dev/null; sleep 1; if [ "${KEEP_ROOT:-0}" != "1" ] && [ -n "${ROOT:-}" ]; then rm -rf -- "$ROOT"; fi' EXIT

say "S5 independent MCP acceptance — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
say "checker: checker-verify (fresh-context; built none of this artifact)"
say "tarball: $TGZ"

# ── 0. artifact identity ──────────────────────────────────────────────────────────────────────────────────────
[ -f "$TGZ" ] || { say "FATAL: no tarball"; exit 2; }
ACTUAL_SHA="$(sha256sum "$TGZ" | cut -d' ' -f1)"
MEMBERS="$(tar -tzf "$TGZ" | grep -c '/')"
say "sha256: $ACTUAL_SHA"
say "members (entries with '/'): $MEMBERS"
[ "$ACTUAL_SHA" = "$EXPECT_SHA" ] && say "PASS artifact sha256 equals the Lead-declared digest" || fail "artifact sha256 differs from the Lead-declared digest ($EXPECT_SHA)"
[ "$MEMBERS" = "104" ] && say "PASS member count 104" || fail "member count $MEMBERS != 104"
{
  echo "date_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "uid: $(id)"
  echo "node: $(node --version)"
  echo "tarball: $TGZ"
  echo "sha256: $ACTUAL_SHA"
  echo "expected_sha256: $EXPECT_SHA"
  echo "members: $MEMBERS"
  echo "hostname: $(hostname)"
  ss -ltn 2>/dev/null | head -40
} > "$OUT/01-environment.txt"

# ── 1. throwaway profile from the exact bytes ───────────────────────────────────────────────────────────────
ROOT="$(mktemp -d /tmp/cv-s5.XXXXXX)"
export DSH_HOME="$ROOT/dsh"
mkdir -p "$ROOT/fleet"/{dsh,state,log,work,backups,checks,skills} "$ROOT/ctl" "$ROOT/fleet/state/mcp"
say "scratch root: $ROOT"
timeout 180 node "$DSH" fleet --from-default-profile web --dump-config > "$ROOT/dump0.yml" 2> "$ROOT/dump0.err"
[ $? -eq 0 ] && say "PASS profile initialised from the shipped web template" || fail "profile init failed (see $ROOT/dump0.err)"
timeout 300 node "$DSH" plugin --profile fleet add --save-exact --ignore-scripts "$TGZ" > "$ROOT/add.log" 2>&1
ADD_RC=$?
say "plugin add exit=$ADD_RC ; installed version: $(node -p "require('$ROOT/dsh/profiles/fleet/node_modules/@cayleb/dsh-fleet/package.json').version" 2>/dev/null || echo UNREADABLE)"
[ "$ADD_RC" -eq 0 ] || fail "plugin add failed"
INST="$ROOT/dsh/profiles/fleet/node_modules/@cayleb/dsh-fleet"

# ── 2. independent installed-tree identity vs tarball bytes ─────────────────────────────────────────────────
mkdir -p "$ROOT/tar"
tar -xzf "$TGZ" -C "$ROOT/tar"
diff -rq "$ROOT/tar/package" "$INST" > "$OUT/02-installed-tree-diff.txt" 2>&1
DIFF_RC=$?
say "installed tree vs tar members: exit=$DIFF_RC ($(wc -l < "$OUT/02-installed-tree-diff.txt") differing lines)"
[ "$DIFF_RC" -eq 0 ] && say "PASS installed profile tree is byte-identical to the tarball" || fail "installed tree differs from the tarball (02-installed-tree-diff.txt)"

# ── 3. scratch host overlay (my own; the row config is the shipped contract, paths are scratch) ────────────
cat > "$ROOT/overlay.yml" <<EOF
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
    port: $PORT
    bindSourceAllow: [loopback, tailnet]
    bindHosts: []
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
/usr/bin/age-keygen -o "$ROOT/recovery.key" 2>/dev/null; /usr/bin/age-keygen -y "$ROOT/recovery.key" > "$ROOT/recovery.pub" 2>/dev/null
timeout 180 node "$DSH" --profile fleet --patch "$ROOT/overlay.yml" --dump-config > "$ROOT/dump1.yml" 2> "$ROOT/dump1.err"
DUMP_RC=$?
say "composed overlay dump-config exit=$DUMP_RC"
grep -q 'id: fleet-mcp' "$ROOT/dump1.yml" && say "PASS fleet-mcp row composes" || fail "fleet-mcp row missing from composed profile"
say "composed rows (fleet-*): $(grep -c 'id: fleet-' "$ROOT/dump1.yml" 2>/dev/null || echo 0)"
cp "$ROOT/dump1.yml" "$OUT/03-composed-profile.yml" 2>/dev/null

# ── 4. boot the scratch host ────────────────────────────────────────────────────────────────────────────────
node "$DSH" --profile fleet --patch "$ROOT/overlay.yml" --no-open --host 127.0.0.1 --port 0 > "$ROOT/host.log" 2>&1 &
echo $! > "$ROOT/host.pid"
say "scratch host pid $(cat "$ROOT/host.pid" 2>/dev/null || echo '?'); waiting for sockets…"
for _ in $(seq 1 60); do
  [ -S "$ROOT/ctl/control.sock" ] && [ -S "$ROOT/ctl/mcp-admin.sock" ] && break
  sleep 1
done
if [ ! -S "$ROOT/ctl/control.sock" ] || [ ! -S "$ROOT/ctl/mcp-admin.sock" ]; then
  fail "scratch host did not open its control/admin sockets"
  tail -40 "$ROOT/host.log" > "$OUT/04-host-log-tail.txt" 2>&1
  say "---- host.log tail ----"; tail -20 "$ROOT/host.log" | tee -a "$LOG"
  exit 1
fi
say "PASS control + admin sockets are up"

# ── 5. independent status + listener capture ───────────────────────────────────────────────────────────────
node "$TOOLS/ctl.mjs" --socket "$ROOT/ctl/control.sock" status > "$OUT/05-control-status-before.json" 2>"$OUT/05-control-status-before.err"
STATUS_RC=$?
node -e '
const j=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"));
const r=j.result||{}; console.log("ok="+j.ok+" integrity="+r.integrity?.ok+" lanes="+r.lanes_allowed+" failures="+JSON.stringify(r.integrity?.failures??null));
process.exit(j.ok===true && r.integrity?.ok===true && r.lanes_allowed===true ? 0 : 1);
' "$OUT/05-control-status-before.json" > "$OUT/05-status-summary.txt" 2>&1
STATUS_OK_RC=$?
say "control status: $(cat "$OUT/05-status-summary.txt")"
[ "$STATUS_OK_RC" -eq 0 ] || fail "scratch fleet status is not ok/integrity/lanes"
ss -ltn 2>/dev/null | grep -E ":$PORT\b" > "$OUT/06-listeners.txt"
say "listeners :$PORT:"; sed 's/^/   /' "$OUT/06-listeners.txt" | tee -a "$LOG"

# ── 6. my MCP probe over loopback + tailnet (my client, no builder code) ─────────────────────────────────
rm -rf "$OUT/mcp-evidence"
node "$TOOLS/mcp-verify.mjs" --mode real \
  --url "http://127.0.0.1:$PORT/mcp" \
  --tailnet-url "http://100.114.17.67:$PORT/mcp" \
  --tailnet-optional \
  --admin-socket "$ROOT/ctl/mcp-admin.sock" \
  --control-socket "$ROOT/ctl/control.sock" \
  --out "$OUT/mcp-evidence" > "$OUT/07-mcp-probe.log" 2>&1
PROBE_RC=$?
say "mcp-verify exit=$PROBE_RC"
sed -n '/^SUMMARY/,$p' "$OUT/07-mcp-probe.log" | sed 's/^/   /' | tee -a "$LOG" >/dev/null
[ "$PROBE_RC" -eq 0 ] && say "PASS my MCP probe passed every mandatory check" || fail "my MCP probe failed (07-mcp-probe.log)"

# ── 7. post-run independent facts: action log, status, listeners ────────────────────────────────────────────
node "$TOOLS/ctl.mjs" --socket "$ROOT/ctl/control.sock" status > "$OUT/08-control-status-after.json" 2>&1
node -e '
const j=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"));
const r=j.result||{}; console.log(JSON.stringify({ok:j.ok,integrity_ok:r.integrity?.ok,lanes:r.lanes_allowed,events:r.db?.events,control_peer_uid:r.control?.peer_uid,policy_selftest:r.policy?.selftest}));
' "$OUT/08-control-status-after.json" > "$OUT/08-status-after-summary.json" 2>&1
say "status after: $(cat "$OUT/08-status-after-summary.json")"

LOG_PATH="$ROOT/fleet/log/mcp-actions.jsonl"
if [ -f "$LOG_PATH" ]; then
  LOG_SHA="$(sha256sum "$LOG_PATH" | cut -d' ' -f1)"
  LOG_LINES="$(wc -l < "$LOG_PATH")"
  say "action log: $LOG_LINES rows sha256=$LOG_SHA"
  cp "$LOG_PATH" "$OUT/09-mcp-actions.jsonl"
  echo "path: $LOG_PATH" > "$OUT/09-mcp-actions-meta.txt"
  echo "rows: $LOG_LINES" >> "$OUT/09-mcp-actions-meta.txt"
  echo "sha256: $LOG_SHA" >> "$OUT/09-mcp-actions-meta.txt"
  # independent whole-file token grep, in memory only (result is just a count)
  node -e '
const fs=require("fs"), net=require("net");
const adminPath=process.argv[1], logPath=process.argv[2];
const s=net.connect(adminPath); let b="";
s.on("connect",()=>s.write("{\"op\":\"token\"}\n"));
s.on("data",d=>{b+=d; const nl=b.indexOf("\n"); if(nl<0) return; s.end();
  const token=JSON.parse(b.slice(0,nl)).result?.token ?? "";
  const text=fs.readFileSync(logPath,"utf8");
  const hits=token ? text.split(token).length-1 : 0;
  const rows=text.split("\n").filter(l=>l.trim()!=="").length;
  console.log("whole_file_token_hits="+hits+" rows="+rows+" bytes="+Buffer.byteLength(text));
  process.exit(hits===0?0:1);
});
s.on("error",e=>{console.log("admin socket error: "+e.message);process.exit(2);});
' "$ROOT/ctl/mcp-admin.sock" "$LOG_PATH" > "$OUT/10-whole-log-token-scan.txt" 2>&1
  [ $? -eq 0 ] && say "PASS whole action-log file contains no bearer token ($(cat "$OUT/10-whole-log-token-scan.txt"))" || fail "bearer token found in the action log (10-whole-log-token-scan.txt)"
  # token never in my evidence either (compare in memory; print count only)
  node -e '
const fs=require("fs"), path=require("path"), net=require("net");
const adminPath=process.argv[1], root=process.argv[2];
const s=net.connect(adminPath); let b="";
s.on("connect",()=>s.write("{\"op\":\"token\"}\n"));
s.on("data",d=>{b+=d; const nl=b.indexOf("\n"); if(nl<0) return; s.end();
  const token=JSON.parse(b.slice(0,nl)).result?.token ?? "";
  let hits=0, files=0;
  const walk=dir=>{ for(const e of fs.readdirSync(dir,{withFileTypes:true})){ const p=path.join(dir,e.name); if(e.isDirectory()) walk(p); else if(e.isFile()){ files++; try{ if(fs.readFileSync(p,"utf8").includes(token)) hits++; }catch{} } } };
  walk(root);
  console.log("evidence_files="+files+" token_hits="+hits);
  process.exit(hits===0?0:1);
});
s.on("error",e=>{console.log("admin socket error: "+e.message);process.exit(2);});
' "$ROOT/ctl/mcp-admin.sock" "$OUT" > "$OUT/11-evidence-token-scan.txt" 2>&1
  [ $? -eq 0 ] && say "PASS no bearer token in my evidence pack ($(cat "$OUT/11-evidence-token-scan.txt"))" || fail "bearer token found in my evidence ($(cat "$OUT/11-evidence-token-scan.txt"))"
else fail "no action log at $LOG_PATH"
fi
cp "$ROOT/host.log" "$OUT/12-host-log.txt" 2>/dev/null
cp "$ROOT/dump1.err" "$OUT/12-dump1.err" 2>/dev/null
cp "$ROOT/add.log" "$OUT/12-add.log" 2>/dev/null

# ── 8. hashes + verdict ─────────────────────────────────────────────────────────────────────────────────────
( cd "$OUT" && find . -maxdepth 2 -type f ! -name SHA256SUMS ! -name 13-S5-VERDICT.md -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS )
say ""
say "=================================================="
if [ "$FAIL" -eq 0 ]; then say "S5 RESULT: PASS — probe and independent checks clean"; else say "S5 RESULT: FAIL — $FAIL failing check(s) above"; fi
say "evidence: $OUT"
say "scratch root (deleted unless KEEP_ROOT=1): $ROOT"
say "=================================================="
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
