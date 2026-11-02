# Team B — Consolidation Phase 3 cutover runbook (HELD — ready to paste, not executed)

**Filed:** 2026-11-02T09:40Z (04:40 EST) · **Author:** fleet-lead (Team B) · **Status:** HELD — Phase 3 requires (1) Nox's F1/F2 ruling, (2) Lead's explicit approval, (3) an app-idle moment. Nothing in this file has been run against the live home.
**Scope assumption:** ruling = **option (a), combined variant** (F1 `fleet-mcp` + F2 four `preset-fleet-*` rows re-rooted, manifest regenerated). The option-(b) fallback deltas are in §9.
**Who runs what:** every `sudo` step is a **credential-holder step** (Cayleb/Nox). Team B does not execute root actions and stays within its fence. Read-only `cayleb` checks may be run by the operator.

## 0. Prerequisites and hard guards

1. Nox ruling recorded (F1+F2 → option a) and Lead approval quoted with a UTC timestamp.
2. App-idle: owner confirms no live DSH conversation is in use, and the desktop app is **quit** for the window (coexistence of a running Electron app with a consolidated profile in the same home is an **untested blocked item** — see §8; do not run the window with the app open).
3. **Never start the consolidated profile while the `fleet-dsh` unit still writes.** The guard in §3 is fail-closed: if any clause fails, stop.
4. Only one writer to a DSH home at a time: the unit (fleet home `/var/lib/fleet`) and the consolidated profile (desktop home) must never both be active.

Shell setup for every paste block:

```bash
DH="$HOME/.local/opt/deepseek-harness-official/apps/desktop/.desktop-build/development/home"
CLI="$HOME/.local/opt/deepseek-harness-official/apps/cli/lib/bin.js"
RT="/run/user/$(id -u)/fleet"
PKGDIR="$DH/profiles/fleet/node_modules/@cayleb/dsh-fleet"
PROBES="$HOME/Documents/deepseek-harness/default-workspace/board/probes/team-b-consolidation"
EV="$PROBES/phase3-cutover-$(date -u +%Y%m%dT%H%M%SZ).md"
mkdir -p "$(dirname "$EV")"
```

## 1. Step 0 — build and install the live combined variant (no start)

Build in a **scratch extraction**, never in the installed tree; the package's own generator recomputes the pins.

```bash
B="$HOME/workspace/harness-build/variant-031-consolidated"
rm -rf "$B" && mkdir -p "$B"
tar -xzf "$HOME/workspace/harness-build/candidate-031/dist/cayleb-dsh-fleet-0.3.1.tgz" -C "$B" --strip-components=1
# F1 — mcp row: port 8790 (unit used 8789), state/log/admin/fleetRoot re-rooted; keep tailnet so Nox keeps access
sed -i \
  -e 's#port: 8789#port: 8790#' \
  -e "s#statePath: /var/lib/fleet/state/mcp#statePath: $DH/fleet/state/mcp#" \
  -e "s#logPath: /var/lib/fleet/log/mcp-actions.jsonl#logPath: $DH/fleet/log/mcp-actions.jsonl#" \
  -e "s#adminSocket: /run/fleet/mcp-admin.sock#adminSocket: $RT/mcp-admin.sock#" \
  -e "s#fleetRoot: /var/lib/fleet#fleetRoot: $DH/fleet#" "$B/mcp.row.yml"
# F2 — the four preset rows' skills dir
for f in lead builder checker researcher; do
  sed -i "s#- /var/lib/fleet/skills#- $DH/fleet/skills#" "$B/presets/$f.patch.yml"
done
# regenerate pins with the package's own generator
(cd "$B" && node bin/rows-manifest.mjs)        # expect: rows.manifest.json: 8 pinned row(s), 12 disabled id(s)
# static checks before packing
grep -c '/var/lib/fleet' "$B/mcp.row.yml" "$B"/presets/*.patch.yml   # every count must be 0
sha256sum "$B/rows.manifest.json"              # record as LIVE_MANIFEST_SHA
# pack and install into the live profile
(cd "$B" && npm pack --pack-destination "$B/dist")   # -> dist/cayleb-dsh-fleet-0.3.1.tgz
VARIANT_TGZ="$B/dist/cayleb-dsh-fleet-0.3.1.tgz"
node "$CLI" plugin --profile fleet add "file:$VARIANT_TGZ"   # if pnpm keeps the old tarball: update the dep to this file: spec, then `node "$CLI" plugin --profile fleet install`
sha256sum "$PKGDIR/rows.manifest.json"         # must equal LIVE_MANIFEST_SHA
DSH_HOME="$DH" node "$CLI" --profile fleet --dump-config > "$PROBES/phase3-live-variant-dump.yml"
grep -c "$DH/fleet/skills" "$PROBES/phase3-live-variant-dump.yml"   # >= 5 (4 presets + fleet-lessons.skillDir)
grep -c '/var/lib/fleet' "$PROBES/phase3-live-variant-dump.yml" || true   # must be 0
grep -c 'port: 8790' "$PROBES/phase3-live-variant-dump.yml"          # 1
```

**Gate 0:** dump rc=0, `LIVE_MANIFEST_SHA` matches the installed file, zero `/var/lib/fleet`, one `port: 8790`. If not, stop and report; the profile is still not running and can be repaired or removed.

Also record the profile's pre-cutover identity (for rollback comparison):

```bash
sha256sum "$DH/profiles/desktop/cordis.patch.yml" "$DH/profiles/desktop/package.json"   # expect 77476b79…, ce364672…
stat -c 'creds mode=%a size=%s mtime=%y' "$DH/.credentials.yaml"
```

## 2. Step 1 — app-idle evidence (record before touching anything)

```bash
{
  echo "# Phase 3 app-idle evidence"
  date -u; date
  echo; echo "## Owner confirmation (paste verbatim, with timestamp)"
  echo "Lead approval: <paste>"
  echo "Owner app-idle statement: <paste>"
  echo; echo "## Desktop app (expect: no electron process — quit for the window)"
  pgrep -af 'electron' | head -5 || echo 'no electron process'
  echo; echo "## Any DSH CLI sessions running (expect none)"
  pgrep -af 'dsh .*--profile' | grep -v 'fleet-dsh' || echo 'none'
  echo; echo "## Session files touched in the last 15 minutes (expect none)"
  find "$DH/sessions" -type f -newermt '-15 minutes' -printf '%TY-%Tm-%TdT%TH:%TM %p\n' 2>/dev/null | sort
  echo; echo "## Store files touched in the last 15 minutes (expect none except our own probes)"
  find "$DH/storages" "$DH/fleet" -type f -newermt '-15 minutes' -printf '%TY-%Tm-%TdT%TH:%TM %p\n' 2>/dev/null | sort
  echo; echo "## Load (expect low, no build/video/render job)"
  uptime
  echo; echo "## Unit state (must still be active here; it is stopped next)"
  systemctl show fleet-dsh -p ActiveState,SubState,InvocationID,ActiveEnterTimestamp
} | tee "$EV"
```

**Gate 1:** no session file newer than 15 minutes, no Electron process, owner confirmation present, load quiet. Any failure → reschedule; do not proceed.

## 3. Step 2 — credentialed stop of the unit (root; credential holder)

```bash
sudo systemctl stop fleet-dsh
sudo systemctl show fleet-dsh -p ActiveState,SubState,Result,InvocationID,ExecMainPID,NRestarts
journalctl -u fleet-dsh -n 30 --no-pager | tail -20
```

**Gate 2 (clean exit; wait up to ~20 s, then re-check):**

```bash
for i in $(seq 1 20); do
  systemctl is-active --quiet fleet-dsh || break; sleep 1
done
systemctl is-active fleet-dsh            # expect: inactive
systemctl show fleet-dsh -p Result       # expect: Result=success
pgrep -af '/var/lib/fleet'             || echo 'no fleet-user processes left'
ss -ltn | grep -E ':(8789|3091)\b'       || echo 'no unit listeners'
test ! -S /run/fleet/control.sock && echo 'unit control socket gone'
```

If `Result` is not `success`, or any process/listener/socket survives → **STOP**, restart the unit (`sudo systemctl start fleet-dsh`), report, do not start the consolidated profile.

## 4. Step 3 — fail-closed single-writer guard (re-check, then start)

```bash
sleep 5
GUARD_OK=1
systemctl is-active --quiet fleet-dsh && { echo 'GUARD-FAIL: unit active'; GUARD_OK=0; }
pgrep -f '/var/lib/fleet/npm/bin/dsh' >/dev/null && { echo 'GUARD-FAIL: unit process alive'; GUARD_OK=0; }
ss -ltn | grep -qE ':(8789|3091)\b' && { echo 'GUARD-FAIL: unit listeners alive'; GUARD_OK=0; }
[ -S /run/fleet/control.sock ] && { echo 'GUARD-FAIL: unit control socket alive'; GUARD_OK=0; }
[ "$GUARD_OK" = 1 ] && echo 'GUARD-OK: no unit writer' || { echo 'ABORT: never start the consolidated profile while the unit writes'; exit 1; }
```

## 5. Step 4 — start the consolidated profile (as `cayleb`, not root)

```bash
mkdir -p "$DH/fleet/log"
cd "$DH"
DSH_HOME="$DH" XDG_RUNTIME_DIR="/run/user/$(id -u)" \
  setsid node "$CLI" --profile fleet --no-open --host 127.0.0.1 --port 3092 \
  > "$DH/fleet/log/consolidated-$(date -u +%Y%m%dT%H%M%SZ).out" 2>&1 &
echo $! > "$DH/fleet/consolidated.pid"
sleep 2; LOG=$(ls -t "$DH/fleet/log"/consolidated-*.out | head -1); echo "pid=$(cat "$DH/fleet/consolidated.pid") log=$LOG"
```

**Wait for health (up to ~60 s), then check for errors: 0**

```bash
for i in $(seq 1 60); do
  ss -ltn | grep -q '127.0.0.1:8790' && [ -S "$RT/control.sock" ] && break; sleep 1
done
ss -ltn | grep -E ':(8790|3092)\b'
grep -inE 'error|degraded|EACCES|EADDRINUSE|ECONNREFUSED' "$LOG" || echo 'no startup errors'
```

Expected: 8790 on 127.0.0.1, ::1 (and the tailnet address if kept), 3092 on 127.0.0.1; control socket at `$RT/control.sock`; `no startup errors`. A `fleet-mcp` DEGRADED line here is a **Gate failure** under option (a) → rollback (§7).

## 6. Step 5 — health / gate checks (all must pass before acceptance)

Record each command's output into `$EV`.

| # | Check | Command | Pass criteria |
|---|---|---|---|
| H1 | control status | `DSH_FLEET_SOCKET="$RT/control.sock" node "$PKGDIR/bin/dsh-fleet.mjs" status` | rc=0; `integrity.ok true`; `rows {ok true, pinned 8, disabled 12, failures [], manifest_sha256=LIVE_MANIFEST_SHA}`; `guard.ok true`; `sandbox.ok true`; `lanes_allowed true` |
| H2 | composed config | `DSH_HOME="$DH" node "$CLI" --profile fleet --dump-config \| grep -E 'var/lib/fleet\|fleet/skills\|port: 8790'` | zero `/var/lib/fleet`; ≥5 live skills paths; `port: 8790` |
| H3 | providers presence (Boolean only) | `DSH_FLEET_SOCKET="$RT/control.sock" node "$PKGDIR/bin/dsh-fleet.mjs" op providers-get` | `opencode-go.credential.present: true` (ref exists in the desktop home); no credential values printed; `anthropic` may be false |
| H4 | MCP nine tools | `python3 "$PROBES/phase1b-mcp-client.py" http://127.0.0.1:8790 "$DH/fleet/state/mcp/token"` | `9 tools: [fleet.status, session.create, session.prompt, session.status, board.op, order.drive_roundtrip, order.stage_real, fleet.selftest, actions.status]`; `integrity.ok true` in `fleet.status` |
| H5 | roster lane (Option-A) | `DSH_FLEET_SOCKET="$RT/control.sock" node "$PKGDIR/bin/dsh-fleet.mjs" op lane-spawn '{"role":"builder","start":false}'` | rc=0; lane `builder:1` `state: "parked"`, `provider: "opencode-go"`, cwd under `$DH/fleet/work/lanes/`; then `op lane-stop '{"id":"builder:1"}'` → stopped |
| H6 | order staging + board read | included in H4 transcript (`order.stage_real` on the Phase 1b payload fields: title/goal/done_when/example/est_minutes) | ok; `order` `state: open`; `payload_sha256` recorded; `board-get` resolves `transport: "control"`, `fixture: false` |
| H7 | action log chain | included in H4 transcript (`actions.status`) | `broken: null`, `open: true`, chain head advances |
| H8 | first-run surface | `DSH_FLEET_SOCKET="$RT/control.sock" node "$PKGDIR/bin/dsh-fleet.mjs" first-run --status` | rc=0; installed-tree manifest reported; no `credential-entry-required` for `OPENCODE_GO_API_KEY` |
| H9 | no unit residue | `ss -ltn \| grep -E ':(8789\|3091)\b'` | empty |

Any H-failure → **do not run acceptance**; go to rollback (§7) unless the failure is a known, explicitly accepted delta from the ruling.

## 7. Step 6 — scoped acceptance battery (option a)

Run after all H-checks pass; append results to `$EV`.

| # | Acceptance item | How | Pass |
|---|---|---|---|
| A1 | consolidated containment rows mounted | H1 rows ok, guard ok | yes |
| A2 | MCP facade live on the consolidated profile | H4 | 9 tools, 9 calls logged |
| A3 | real-order staging path works on the new fleet root | H6 | staged json under `$DH/fleet/state/mcp/staged/`, digest recorded |
| A4 | board read path | H6 `board-get` | control transport, `fixture:false` |
| A5 | roster lanes on the consolidated host | H5 | parked lane on `opencode-go`, clean stop |
| A6 | model-free session + turn | `session.create {preset:'builder', prompt:'phase1b stub turn probe'}` via H4 client or `phase1b-session-probe.py` | session created, turn `completed`, `provider_calls: 0` |
| A7 | presets carry the consolidated skills dir | H2 (4 preset rows) + `test -r "$DH/fleet/skills"` | dir exists/readable, no unit path |
| A8 | action audit | H7 | chain unbroken |
| A9 | provider presence | H3 | Boolean only, `OPENCODE_GO_API_KEY` present |
| A10 | desktop profile untouched | `sha256sum "$DH/profiles/desktop/cordis.patch.yml" "$DH/profiles/desktop/package.json"` | `77476b79…`, `ce364672…` |
| A11 | single writer confirmed | `pgrep -af '/var/lib/fleet'` | empty |

File the result as `board/reports/team-b-consolidation-phase3-result.md` (Lead approval quote, evidence file, pass/fail per item, known deltas).

## 8. Step 7 — rollback (any failure or after the window)

```bash
# R1 stop the consolidated profile (as cayleb): negative PID = the setsid process group
PID=$(cat "$DH/fleet/consolidated.pid")
kill -TERM -"$PID" 2>/dev/null || kill -TERM "$PID" 2>/dev/null
for i in $(seq 1 30); do ss -ltn | grep -qE ':(8790|3092)\b' || break; sleep 1; done
ss -ltn | grep -E ':(8790|3092)\b' && echo 'STILL LISTENING — kill -9 group and investigate' || echo 'consolidated listeners closed'
pgrep -af 'profile fleet --no-open' | grep -v 'fleet-dsh' || echo 'no consolidated process'
# R2 restore the unit (root; credential holder)
sudo systemctl start fleet-dsh
systemctl is-active fleet-dsh
ss -ltn | grep -E ':(8789|3091)\b'
journalctl -u fleet-dsh -n 10 --no-pager
# R3 remove the consolidated profile and its fleet data (never touch /var/lib/fleet)
rm -rf "$DH/profiles/fleet" "$DH/fleet" "$RT"
# R4 prove the desktop profile is unchanged
sha256sum "$DH/profiles/desktop/cordis.patch.yml" "$DH/profiles/desktop/package.json"   # 77476b79…, ce364672…
stat -c 'creds mode=%a size=%s mtime=%y' "$DH/.credentials.yaml"
```

Keep `$EV`, the consolidated log, and `phase3-live-variant-dump.yml` as the rollback record. The old `/var/lib/fleet` board is **read-only evidence**: never delete or migrate it in a rollback.

## 9. Option-(b) fallback deltas (only if Nox rules b)

- Build the variant with **F1 only** (mcp row re-rooted to port 8790) and leave presets at `/var/lib/fleet/skills`.
- The consolidated profile's MCP row may then be left at the shipped config instead — expected: `fleet-mcp: DEGRADED — EACCES /var/lib/fleet/...` in the startup log; the systemd unit remains the MCP/acceptance host until retired later.
- Health checks H4, H6, H7 are **skipped or marked N/A** (MCP not part of this profile); H1 must still show `integrity.rows.ok` with the shipped manifest (`1970f995…`) and `lanes_allowed true`; A2/A3/A4/A8/A9 become unit-host checks, not consolidated checks.

## 10. Known blocked / untested items before approval

1. **Storage/session coexistence (new).** The base rows are home-scoped: `storage-json.root = dshHomePath('storages')` and `session-persistence-jsonl` writes under the same home. A consolidated profile running from the desktop home **at the same time as the Electron app** shares `<DH>/storages` and `<DH>/sessions` with it. Two DSH processes in one home were **not tested** (Phase 1b used its own home). This runbook therefore requires the app to be quit for the window. Whether normal operation is app + consolidated side by side, or the fleet profile must re-root `storage-json`/`session-persistence-jsonl` into `<DH>/fleet/state/`, is a ruling item. Do not treat app-open operation as validated.
2. **Runtime integrity was never observed on the live profile** (Phase 2 created it without starting); H1 is the first live observation. Under option (a) the copy proves it, but the live verdict is still a check, not a fact.
3. **Variant install mechanics** (`node "$CLI" plugin --profile fleet add file:<variant>.tgz` on top of the existing file: dep) are untested on the live profile; the fallback is to rewrite the dependency spec and run `node "$CLI" plugin --profile fleet install`, then verify `LIVE_MANIFEST_SHA`.
4. **Port 8790** is a proposal (unit used 8789); if the tailnet binding is kept, confirm `100.114.17.67:8790` binds after the unit stops and that Nox's client is repointed.
5. **Desktop app relaunch after cutover** is out of scope of the window; see item 1.

## 11. Evidence to keep

| Artifact | Where |
|---|---|
| app-idle evidence | `$EV` (`board/probes/team-b-consolidation/phase3-cutover-<UTC>.md`) |
| live variant dump | `board/probes/team-b-consolidation/phase3-live-variant-dump.yml` |
| consolidated start log | `$DH/fleet/log/consolidated-<UTC>.out` |
| H/A outputs | appended to `$EV` |
| result report | `board/reports/team-b-consolidation-phase3-result.md` |
