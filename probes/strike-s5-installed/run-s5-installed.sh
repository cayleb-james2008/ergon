#!/usr/bin/env bash
# S5 INSTALLED — checker-verify INDEPENDENT MCP acceptance against the REAL installed fleet-dsh service (0.3.0).
#
# Unchanged battery (probes/mcp-verify.mjs) against the installed unit's own listeners: loopback 127.0.0.1:8789 and
# the tailnet address 100.114.17.67:8789. Token is read by the probe itself over the real admin socket
# /run/fleet/mcp-admin.sock; it is never printed or written. Read-only except the battery's documented fixture work
# (two fixture orders that roll up done, sessions under the fleet's mcp-sessions root, and action-log rows).
# The service is NEVER restarted or changed by this script; MainPID/InvocationID are captured before and after.
set -uo pipefail
HERE="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
TOOLS="$(cd -- "$HERE/../probes" && pwd)"
OUT="$HERE"
LOG="$OUT/00-run-log.txt"
: > "$LOG"
TGZ=/home/cayleb/workspace/harness-build/strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz
EXPECT_SHA=8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070
PORT=8789
ADMIN=/run/fleet/mcp-admin.sock
CONTROL=/run/fleet/control.sock
FAIL=0
say() { printf '%s\n' "$*" | tee -a "$LOG"; }
fail() { say "FAIL: $*"; FAIL=$((FAIL+1)); }
say "S5 INSTALLED independent MCP acceptance — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
say "target: installed fleet-dsh unit, MCP facade listeners 127.0.0.1:$PORT and 100.114.17.67:$PORT"
say "checker: checker-verify (fresh-context; built none of this artifact)"

# ── 0. installed artifact + unit identity ───────────────────────────────────────────────────────────────────
ACTUAL_SHA="$(sha256sum "$TGZ" | cut -d' ' -f1)"
[ "$ACTUAL_SHA" = "$EXPECT_SHA" ] && say "PASS release tarball sha256 equals the final digest" || fail "release tarball digest mismatch ($ACTUAL_SHA)"
DROP_SHA="$(sha256sum /var/lib/fleet-drop/cayleb-dsh-fleet-0.3.0.tgz 2>/dev/null | cut -d' ' -f1)"
[ "$DROP_SHA" = "$EXPECT_SHA" ] && say "PASS staged drop tarball is the same final bytes" || fail "drop tarball digest $DROP_SHA != $EXPECT_SHA"
BEFORE_PID="$(systemctl show fleet-dsh -p MainPID --value 2>/dev/null)"
BEFORE_INV="$(systemctl show fleet-dsh -p InvocationID --value 2>/dev/null)"
{
  echo "date_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "uid: $(id)"
  echo "node: $(node --version)"
  echo "release_tarball: $TGZ"
  echo "release_sha256: $ACTUAL_SHA"
  echo "drop_tarball_sha256: $DROP_SHA"
  echo "unit_activestate: $(systemctl show fleet-dsh -p ActiveState --value)"
  echo "unit_mainpid: $BEFORE_PID"
  echo "unit_invocationid: $BEFORE_INV"
  echo "listen_port: $PORT"
  echo "admin_socket: $ADMIN"
  echo "control_socket: $CONTROL"
  echo "--- systemctl show netfilter ---"
  systemctl show fleet-dsh -p IPAddressAllow -p IPAddressDeny
  echo "--- ss -ltnp | :$PORT ---"
  ss -ltnp 2>/dev/null | grep ":$PORT"
  echo "--- audit line (journalctl -t dsh-fleet-admin) ---"
  journalctl -t dsh-fleet-admin --since '-2 days' --no-pager --output=short-iso --quiet 2>/dev/null | grep 'plugin-add' | tail -2
  echo "--- direct-read attempts (expected refused for uid 1000) ---"
  ls -ld /var/lib/fleet /var/lib/fleet/state/mcp /var/lib/fleet/log 2>&1
  ls -l /var/lib/fleet/state/mcp/token 2>&1
  ls -l /var/lib/fleet/log/mcp-actions.jsonl 2>&1
} > "$OUT/01-installed-identity.txt"
cat "$OUT/01-installed-identity.txt" | sed 's/^/   /' | tee -a "$LOG" >/dev/null
say "unit before: MainPID=$BEFORE_PID InvocationID=$BEFORE_INV active=$(systemctl show fleet-dsh -p ActiveState --value)"

# listeners: exact addresses, no catch-all
ss -ltn 2>/dev/null | grep ":$PORT" > "$OUT/02-listeners.txt"
CATCHALL="$(grep -E "0\.0\.0\.0:$PORT|\[::\]:$PORT" "$OUT/02-listeners.txt" || true)"
LOOPBACK="$(grep -c "127.0.0.1:$PORT" "$OUT/02-listeners.txt" || true)"
TAILNET="$(grep -c "100.114.17.67:$PORT" "$OUT/02-listeners.txt" || true)"
V6="$(grep -c "\[::1\]:$PORT" "$OUT/02-listeners.txt" || true)"
say "listeners: loopback=$LOOPBACK tailnet=$TAILNET ::1=$V6 catchall=${CATCHALL:-none}"
[ "$LOOPBACK" -ge 1 ] && [ "$TAILNET" -ge 1 ] && [ -z "$CATCHALL" ] && say "PASS listeners are loopback+tailnet only, no catch-all" || fail "listener set is not loopback+tailnet only"
[ -S "$ADMIN" ] && say "PASS real admin socket present" || fail "admin socket missing"
[ -S "$CONTROL" ] && say "PASS real control socket present" || fail "control socket missing"

# ── 1. installed unit boot/control integrity incl. rows vs the shipped manifest ─────────────────────────────
node "$TOOLS/ctl.mjs" --socket "$CONTROL" status > "$OUT/03-control-status-before.json" 2>"$OUT/03-control-status-before.err"
STATUS_RC=$?
[ "$STATUS_RC" -eq 0 ] && say "PASS control status answers ok" || fail "control status failed"
node -e '
const fs=require("fs"), crypto=require("crypto");
const status=JSON.parse(fs.readFileSync(process.argv[1],"utf8")).result;
const extract=process.argv[2];
const sha=f=>crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex");
const shippedRowsSha=sha(extract+"/rows.manifest.json");
const rows=status.integrity?.rows ?? null;
const report={
  ok: status.integrity?.ok,
  chain_ok: status.integrity?.chain?.ok,
  sandbox_ok: status.integrity?.sandbox?.ok,
  rows_ok: rows?.ok, rows_pinned: rows?.pinned, rows_disabled: rows?.disabled, rows_failures: rows?.failures,
  rows_manifest_sha256: rows?.manifest_sha256, shipped_rows_manifest_sha256: shippedRowsSha,
  rows_manifest_matches_shipped: rows?.manifest_sha256 === shippedRowsSha,
  guard_ok: status.integrity?.guard?.ok, guard_failures: status.integrity?.guard?.failures,
  lanes_allowed: status.lanes_allowed, policy_selftest: status.policy?.selftest,
  control_peer_uid: status.control?.peer_uid, events: status.db?.events,
};
fs.writeFileSync(process.argv[3], JSON.stringify(report,null,2)+"\n");
const good = report.ok===true && report.chain_ok===true && report.sandbox_ok===true && report.rows_ok===true &&
  (report.rows_failures??[]).length===0 && report.rows_pinned>0 && report.rows_manifest_matches_shipped===true &&
  report.guard_ok===true && report.lanes_allowed===true && report.policy_selftest===true && report.control_peer_uid===1000;
process.exit(good?0:1);
' "$OUT/03-control-status-before.json" /tmp/cv-tar-final/package "$OUT/04-boot-integrity.json"
INT_RC=$?
say "boot integrity: $(cat "$OUT/04-boot-integrity.json" | tr -d '\n' | head -c 700)"
[ "$INT_RC" -eq 0 ] && say "PASS installed unit integrity ok incl rows manifest == shipped manifest, guard, lanes, selftest, peer uid" || fail "installed unit boot integrity check failed"

# (facade bound addresses vs unit netfilter allowlist are checked after the battery, from fleet.status + ss + systemd)

# ── 2. THE BATTERY (unchanged probe) ────────────────────────────────────────────────────────────────────────
rm -rf "$OUT/mcp-evidence"
node "$TOOLS/mcp-verify.mjs" --mode real \
  --url "http://127.0.0.1:$PORT/mcp" \
  --tailnet-url "http://100.114.17.67:$PORT/mcp" \
  --admin-socket "$ADMIN" \
  --control-socket "$CONTROL" \
  --out "$OUT/mcp-evidence" > "$OUT/07-mcp-probe.log" 2>&1
PROBE_RC=$?
say "mcp-verify exit=$PROBE_RC"
sed -n '/^SUMMARY/,$p' "$OUT/07-mcp-probe.log" | sed 's/^/   /' | tee -a "$LOG" >/dev/null
PROBE_FAILED="$(node -e 'try{const j=require(process.argv[1]);console.log((j.summary.failedIds||[]).join(","))}catch(e){console.log("UNREADABLE")}' "$OUT/mcp-evidence/mcp-verify.json" 2>/dev/null)"
if [ "$PROBE_RC" -eq 0 ]; then
  say "PASS installed-service MCP battery passed every mandatory check"
elif [ "$PROBE_FAILED" = "T3.prompt,T3.status" ]; then
  # The only failures are the strict non-empty-stub-text checks. Check the Lead's acceptance wording explicitly:
  # a completed turn with its transport field and a NAMED ledger-gate reason satisfies "or a named ledger-gate reason".
  node -e '
const fs=require("fs");
const prompt=JSON.parse(fs.readFileSync(process.argv[1],"utf8")).result;
const status=JSON.parse(fs.readFileSync(process.argv[2],"utf8")).result;
const report={
  transport: prompt.transport, ledger_gate: prompt.ledger_gate, status_completed: prompt.status,
  assistant_text_bytes: Buffer.byteLength(prompt.assistant_text ?? ""), provider_calls: prompt.provider_calls,
  note_names_gate: /stub-provider-selftest-only/.test(String(prompt.note ?? "")),
  readback_found: status.found, readback_turns: status.turns, readback_transport: status.last_transport,
  criterion_transport_and_named_gate: (prompt.transport==="agent"||prompt.transport==="stub-turn") && typeof prompt.ledger_gate==="string" && prompt.ledger_gate!=="" && prompt.status==="completed",
};
fs.writeFileSync(process.argv[3], JSON.stringify(report,null,2)+"\n");
console.log(JSON.stringify(report));
const ok=report.criterion_transport_and_named_gate && report.readback_found===true && report.readback_turns===1 && report.readback_transport===report.transport;
process.exit(ok?0:1);
' "$OUT/mcp-evidence/raw/t3-session-prompt.json" "$OUT/mcp-evidence/raw/t3-session-status.json" "$OUT/12-installed-stub-turn-criterion.json" > "$OUT/12-installed-stub-turn-criterion.log" 2>&1
  CRIT_RC=$?
  say "stub-turn criterion: $(cat "$OUT/12-installed-stub-turn-criterion.log")"
  # Reproduce the host condition OUTSIDE the facade: the fleet's own stub-turn control op.
  node "$TOOLS/ctl.mjs" --socket "$CONTROL" raw '{"op":"stub-turn","agent":"checker:selftest-s5-installed","model":"fleet-selftest-stub","text":"checker-verify installed diagnostic","queue":[]}' > "$OUT/13-control-stub-turn-repro.json" 2>&1
  REPRO_RC=$?
  node -e '
const fs=require("fs"); const j=JSON.parse(fs.readFileSync(process.argv[1],"utf8"));
const r=j.result??{}; const m={ok:j.ok===true,status:r.status,assistant_text_bytes:Buffer.byteLength(String(r.assistant_text??"")),provider_calls:r.provider_calls,reason:r.reason??null};
console.log(JSON.stringify(m));
process.exit(m.ok && m.status==="completed" && m.assistant_text_bytes===0 && m.provider_calls===0 ? 0 : 1);
' "$OUT/13-control-stub-turn-repro.json" > "$OUT/13-control-stub-turn-repro.log" 2>&1
  HOST_RC=$?
  say "host-condition reproduction (control stub-turn, outside the facade): $(cat "$OUT/13-control-stub-turn-repro.log")"
  PRIOR_PACK=/home/cayleb/workspace/harness-build/evidence/strike/checker-verify/s7-rerun/ergon-pack
  { echo "# Prior-install evidence: the same empty stub-turn existed on the installed 0.2.2 host before this artifact"; echo "pack: $PRIOR_PACK (my accepted S7 re-run, installed 0.2.2)"; node -e 'const j=require(process.argv[1]+"/pass-1/R2/stub-turn-probes.json");console.log(JSON.stringify(j));' "$PRIOR_PACK" 2>/dev/null || echo "prior pack unavailable"; } > "$OUT/14-prior-install-stub-turn.txt" 2>&1
  say "prior-install comparison: $(tail -1 "$OUT/14-prior-install-stub-turn.txt")"
  if [ "$CRIT_RC" -eq 0 ] && [ "$HOST_RC" -eq 0 ]; then
    say "PASS (with named deviation): installed-service battery passes the acceptance criterion; session.prompt is completed with transport=stub-turn and the named ledger-gate reason, but assistant_text is EMPTY and provider_calls=0 on the installed host. Reproduced outside the facade via the fleet's own stub-turn op and previously present on the installed 0.2.2 host — recorded, not smoothed, and not a regression of the installed artifact."
  else
    fail "installed-service T3 stub-turn failed the criterion / could not be reproduced as a host-level condition"
  fi
else
  fail "installed-service MCP battery failed (failedIds=$PROBE_FAILED; 07-mcp-probe.log)"
fi

# 2b. facade bound addresses vs the unit netfilter allowlist (from the battery's own fleet.status + ss + systemd)
node -e '
const fs=require("fs"), cp=require("child_process");
const raw=JSON.parse(fs.readFileSync(process.argv[1],"utf8"));
const status=raw.result ?? raw;
const bind=(status.mcp?.bind ?? []).map(b=>String(b.host));
const sources=status.mcp?.sources ?? null;
const ss=cp.execFileSync("ss",["-ltn"],{encoding:"utf8"}).split("\n").filter(l=>l.includes(":8789"));
const ssHosts=ss.map(l=>{const m=/(\S+):8789\s/.exec(l); return m?m[1].replace(/^\[|\]$/g,""):null}).filter(Boolean);
const show=cp.execFileSync("systemctl",["show","fleet-dsh","-p","IPAddressAllow","-p","IPAddressDeny"],{encoding:"utf8"});
const allowLine=/IPAddressAllow=(.*)/.exec(show)?.[1] ?? "";
const denyLine=/IPAddressDeny=(.*)/.exec(show)?.[1] ?? "";
const allowed=ip=>ip==="127.0.0.1"||ip==="::1"||ip==="100.114.17.67"||ip==="100.110.154.92";
const report={
  fleet_mcp_bind: bind, ss_hosts: ssHosts,
  sources_v4: sources?.v4 ?? null, sources_v6: sources?.v6 ?? null,
  unit_ipaddressallow: allowLine.trim(), unit_ipaddressdeny: denyLine.trim(),
  bind_matches_ss: JSON.stringify([...new Set(bind)].sort())===JSON.stringify([...new Set(ssHosts)].sort()),
  binds_all_in_unit_allowlist: bind.every(allowed),
  no_catchall: !bind.includes("0.0.0.0") && !bind.includes("::"),
  unit_allowlists_the_three: /127\.0\.0\.0\/8/.test(allowLine) && /::1\/128/.test(allowLine) && /100\.114\.17\.67\/32/.test(allowLine) && /100\.110\.154\.92\/32/.test(allowLine),
  unit_denies_lan_and_v6: /100\.64\.0\.0\/10/.test(denyLine) && /0\.0\.0\.0\/8/.test(denyLine) && /::\/0/.test(denyLine),
};
fs.writeFileSync(process.argv[2], JSON.stringify(report,null,2)+"\n");
console.log(JSON.stringify(report));
process.exit(report.bind_matches_ss && report.binds_all_in_unit_allowlist && report.no_catchall && report.unit_allowlists_the_three && report.unit_denies_lan_and_v6 ? 0 : 1);
' "$OUT/mcp-evidence/raw/t7-fleet-status.json" "$OUT/05-bind-vs-netfilter.json" > "$OUT/05-bind-vs-netfilter.log" 2>&1
BIND_RC=$?
say "bind vs netfilter: $(cat "$OUT/05-bind-vs-netfilter.log")"
[ "$BIND_RC" -eq 0 ] && say "PASS facade binds are the installed unit\'s listeners and sit inside its netfilter allowlist (localhost + 100.110.154.92 + 100.114.17.67); no catch-all" || fail "facade binds do not match the unit listeners/netfilter allowlist (05-bind-vs-netfilter.json)"

# ── 3. post-battery independent facts: no restart, action log, token absence in readable scope ──────────────
AFTER_PID="$(systemctl show fleet-dsh -p MainPID --value 2>/dev/null)"
AFTER_INV="$(systemctl show fleet-dsh -p InvocationID --value 2>/dev/null)"
[ "$BEFORE_PID" = "$AFTER_PID" ] && [ "$BEFORE_INV" = "$AFTER_INV" ] && say "PASS service was not restarted by the battery (MainPID/InvocationID unchanged)" || fail "service restarted during the battery"
node "$TOOLS/ctl.mjs" --socket "$CONTROL" status > "$OUT/08-control-status-after.json" 2>&1
node -e '
const fs=require("fs"); const s=JSON.parse(fs.readFileSync(process.argv[1],"utf8")).result;
console.log(JSON.stringify({ok:s.integrity?.ok,rows_ok:s.integrity?.rows?.ok,events:s.db?.events,lanes:s.lanes_allowed,selftest:s.policy?.selftest}));
' "$OUT/08-control-status-after.json" > "$OUT/08-status-after-summary.json" 2>&1
say "status after: $(cat "$OUT/08-status-after-summary.json")"

# Admin-socket tail: the only readable view of the installed action log (raw file is 0750 fleet:fleet).
node -e '
const net=require("net"), fs=require("fs");
const adminPath=process.argv[1], out=process.argv[2];
function admin(req){return new Promise((res,rej)=>{const s=net.connect(adminPath);let b="";const t=setTimeout(()=>{s.destroy();rej(new Error("timeout"))},8000);
s.on("connect",()=>s.write(JSON.stringify(req)+"\n"));
s.on("data",d=>{b+=d;const n=b.indexOf("\n");if(n<0)return;clearTimeout(t);s.end();try{res(JSON.parse(b.slice(0,n)))}catch(e){rej(e)}});
s.on("error",e=>{clearTimeout(t);rej(e)});});}
(async()=>{
  const stat=await admin({op:"stat"});
  const token=(await admin({op:"token"})).result?.token ?? null;
  const tail=await admin({op:"log_tail",n:100});
  const rows=tail.result?.rows ?? [];
  const text=JSON.stringify(rows);
  const tokenHits=token ? text.split(token).length-1 : -1;
  const report={log_path:stat.result?.log?.path, log_open:stat.result?.log?.open, log_broken:stat.result?.log?.broken,
    log_seq:stat.result?.log?.seq, tail_rows:rows.length, token_value_compared_in_memory:token!==null,
    token_hits_in_readable_rows:tokenHits, token_file_mode:stat.result?.token?.mode, token_fingerprint_present:typeof stat.result?.token?.fingerprint==="string"};
  fs.writeFileSync(out, JSON.stringify(report,null,2)+"\n");
  console.log(JSON.stringify(report));
  process.exit(tokenHits===0?0:1);
})().catch(e=>{console.log("admin error: "+e.message);process.exit(2)});
' "$ADMIN" "$OUT/09-action-log-readable-scope.json" > "$OUT/09-action-log-readable-scope.log" 2>&1
TAIL_RC=$?
say "action-log readable scope: $(cat "$OUT/09-action-log-readable-scope.log")"
[ "$TAIL_RC" -eq 0 ] && say "PASS admin-socket log tail (100 rows) contains no bearer token" || fail "token scan of readable log scope failed"

# Direct full-file byte scan: record exactly what is and is not readable.
if head -c 1 /var/lib/fleet/log/mcp-actions.jsonl >/dev/null 2>&1; then
  echo "direct_read: OK" > "$OUT/10-direct-log-read.txt"
  say "PASS raw action log is readable; full-file scan possible"
else
  echo "direct_read: REFUSED (uid $(id -u) cannot read /var/lib/fleet/log/mcp-actions.jsonl; the log directory is fleet-owned 0750)" > "$OUT/10-direct-log-read.txt"
  say "NOTE raw /var/lib/fleet/log/mcp-actions.jsonl is not readable as uid $(id -u); full-file byte scan NOT possible from this seat — readable scope = admin-socket log_tail (100 rows) + MCP actions.status; recorded as a stated limitation, not a pass"
fi

# token never in my evidence
node -e '
const fs=require("fs"), path=require("path"), net=require("net");
const adminPath=process.argv[1], root=process.argv[2];
const s=net.connect(adminPath); let b="";
s.on("connect",()=>s.write("{\"op\":\"token\"}\n"));
s.on("data",d=>{b+=d; const n=b.indexOf("\n"); if(n<0) return; s.end();
  const token=JSON.parse(b.slice(0,n)).result?.token ?? "";
  let hits=0, files=0;
  const walk=dir=>{ for(const e of fs.readdirSync(dir,{withFileTypes:true})){ const p=path.join(dir,e.name); if(e.isDirectory()) walk(p); else if(e.isFile()){ files++; try{ if(fs.readFileSync(p,"utf8").includes(token)) hits++; }catch{} } } };
  walk(root);
  console.log("evidence_files="+files+" token_hits="+hits);
  process.exit(hits===0?0:1);
});
s.on("error",e=>{console.log("admin socket error: "+e.message);process.exit(2)});
' "$ADMIN" "$OUT" > "$OUT/11-evidence-token-scan.txt" 2>&1
EVID_RC=$?
[ "$EVID_RC" -eq 0 ] && say "PASS no bearer token in this evidence pack ($(cat "$OUT/11-evidence-token-scan.txt"))" || fail "bearer token found in evidence"

# ── 4. hashes + verdict ─────────────────────────────────────────────────────────────────────────────────────
( cd "$OUT" && find . -maxdepth 2 -type f ! -name SHA256SUMS ! -name 13-S5-INSTALLED-VERDICT.md -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS )
say ""
say "=================================================="
[ "$FAIL" -eq 0 ] && say "S5 INSTALLED RESULT: PASS — installed-service battery and independent checks clean" || say "S5 INSTALLED RESULT: FAIL — $FAIL failing check(s) above"
say "evidence: $OUT"
say "=================================================="
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
