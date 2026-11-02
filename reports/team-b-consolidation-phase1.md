# TEAM B — CONSOLIDATION PHASE 1 (offline prep): proof in a throwaway copy + exact Phase-2 delta

Phase 1 executed 2026-11-02T03:57–04:07Z in a **throwaway credential-less copy**, per the Lead's scope. **No live
profile write, no restart, no unit stop, no credential value read/copied.** The owner's desktop profile and his
`.credentials.yaml` were not opened for values (hash-only stat recorded in the script) and not modified.
Artifacts: `board/probes/team-b-consolidation/` (`profile-overlay.yml`, `dump-composed-copy.yml`,
`host-copy-proof.log`, `phase1-evidence.txt`, `phase2-install-consolidated-profile.sh`,
`phase3-cutover-consolidated.sh`, `rollback-consolidated.sh`).

## 1. What was built in the copy

```
COPY_HOME=/home/cayleb/workspace/harness-build/scratch-tb-consolidation/home   (fresh, empty, no credentials)
DSH_HOME=$COPY_HOME node <checkout>/apps/cli/lib/bin.js fleet --from-default-profile web --dump-config
   -> profile fleet created from the shipped web template; bundles = [@deepseek-ai/dsh-base, @deepseek-ai/dsh-web-app]
DSH_HOME=$COPY_HOME node ... plugin --profile fleet add --save-exact --ignore-scripts cayleb-dsh-fleet-0.3.1.tgz
   -> rc=0, 410 ms, bundles = [..., @cayleb/dsh-fleet]
# then the profile overlay (the Phase-2 patch content) was written to home/profiles/fleet/cordis.patch.yml
```
Package identity: installed `node_modules/@cayleb/dsh-fleet` is **byte-identical to the 0.3.1 tarball** (`diff -rq`
rc=0, 123 files); `rows.manifest.json` sha256 **`1970f9954b7131c58191327bb0e60b9d31a704bc309d083cd53971bc9271edba`**;
`package.json` sha256 `631eaeb18a257173765ed3fd7334d78fd213697bb52cbb219ef148386b87de81` (expected).

## 2. Composed config proof (copy, `--dump-config`, 59,749 B)

- 15 `fleet-*` rows compose (`fleet-core`, `fleet-orders`, `fleet-roster`, `fleet-lessons`, `fleet-policy`,
  `fleet-ledger`, `fleet-housekeeper`, `fleet-sandbox`, `fleet-egress`, `fleet-fsguard`, `fleet-firstrun`, 4 presets,
  `fleet-mcp`).
- `fleet-roster` row present with the re-rooted config from the overlay (checksRoot/gitRoot/engineRoot under
  `__FLEET_ROOT__`, credentialStore = `<desktop-home>/.credentials.yaml`).
- `subagent` row: **`maxActiveSubagents: 32`**.
- `llm-pi-ai` providers: **`opencode-go { apiKeyEnv: OPENCODE_GO_API_KEY, displayName: … }`** — **no value** anywhere
  in the composed config.
- The overlay re-roots every non-pinned row under the DSH home (`<home>/fleet/{state,log,work,checks,…}`) and the
  control/egress/admin sockets under the user runtime dir; no absolute `/var/lib/fleet` path remains for those rows.

## 3. Live scratch-host proof in the copy (user-level host, no unit)

Started `DSH_HOME=$COPY_HOME node <cli> --profile fleet --no-open --host 127.0.0.1 --port 8792` (control socket came
up in **1–2 s**). Read-only query results (verbatim values):

| Probe | Result |
|---|---|
| control `status` | `ok:true`, `integrity:true`, rows **8 pinned / 12 disabled**, `failures:[]`, `lanes_allowed:true`, `selftest:true` |
| `policy-get roster.models` | lead anthropic/claude-sonnet-5-5; **builder opencode-go/deepseek-v4.1-flash**; checker anthropic/claude-haiku-5-5; researcher opencode-go/deepseek-v4.1-flash |
| `policy-get rates` | keys incl. **`opencode-go/deepseek-v4.1-flash` {input 0.15, output 0.6, cache_read 0.003, cache_write 0}** |
| `providers-get` | `opencode-go` route present, models `["deepseek-v4.1-flash"]`, **credential.present:false** (copy has no credential store; correct) |
| `roster-get` | doctrine roles as above, cap 5 |
| `first-run --status` | **version 0.3.1, 123 files**, `dshVersion: 0.2.2-alpha.2` |
| state dirs created | `<home>/fleet/{backups,checks,dsh,log,state,state/mcp,work}` + `fleet.db`, `state/mcp/token`, backup `fleet-*.db.age`, logs |
| copy credential store | DSH boot auto-created an **empty** `<copy-home>/.credentials.yaml` (161 B, mode 600) — copy only, no values used |

**Pinned-row finding (material for Phase 2):** `fleet-mcp` is **integrity-pinned** in `rows.manifest.json`
(name+config hashes). Overriding its config in the profile → `integrity.ok:false`, `lanes_allowed:false`,
`failures:["rows fleet-mcp: config changed"]`. Leaving the shipped row → the MCP facade **DEGRADED**: `EACCES mkdir
/var/lib/fleet/state/mcp` and `/var/lib/fleet/log`, "not listening" (uid 1000 cannot write the fleet unit's paths).
Every other fleet row re-roots cleanly (they are not pinned). So a user-level consolidated setup **cannot re-root the
MCP row by patch**.

## 4. Exact Phase-2 delta (file-by-file, path-by-path)

**Created (new only):**
- `<desktop-home>/profiles/fleet/` — from the shipped web template: `cordis.yml`, `package.json`, `pnpm-lock.yaml`,
  `pnpm-workspace.yaml`, `node_modules/` (dsh-base/web-app + dependencies).
- `plugin add` writes into that profile: `package.json` (+dependency, +`dsh.profile.bundles` entry),
  `pnpm-lock.yaml`, `node_modules/@cayleb/dsh-fleet/**` (123 files), `node_modules/.pnpm/**`.
- `<desktop-home>/profiles/fleet/cordis.patch.yml` = our overlay (non-pinned rows: fleet-core, fleet-sandbox,
  fleet-egress, fleet-lessons, fleet-housekeeper, session-query-sqlite, fleet-policy, fleet-roster; **no fleet-mcp
  entry**), with recovery key `age-keygen` output at `<desktop-home>/fleet/{recovery.key,recovery.pub}` (600/644).
- On first start: `<desktop-home>/fleet/{dsh,state,state/mcp,log,work,backups,checks,skills,gitwork,engines}`,
  `fleet.db`, sockets `${XDG_RUNTIME_DIR:-/run/user/1000}/fleet/{control.sock,egress.sock,mcp-admin.sock}`.
- A consolidated host log at `~/.local/state/dsh-fleet-consolidated.log` + pid file.

**Explicitly untouched:** `profiles/desktop/**` (hash record only — `cordis.patch.yml` + `package.json` sha256 before,
`stat` of `.credentials.yaml`; **never read its values**); `<desktop-home>/.credentials.yaml` (no copy, no edit);
`~/.dsh/profiles/{web,tauri}`; `/var/lib/fleet/**`; the `fleet-dsh` unit files; `~/.config/opencode/**` and
`~/.local/share/opencode/**`; `~/.dsh/strike-door/**`.

**Rollback:** `rollback-consolidated.sh` — kill the consolidated pid, `systemctl --no-ask-password start fleet-dsh`
(if Phase 3 ran), delete/keep `profiles/fleet` + `<desktop-home>/fleet` for inspection; desktop profile never changed.

## 5. Phase-4 acceptance command set that does NOT depend on the fleet-dsh unit (pending Nox ruling)

```bash
export DSH_FLEET_SOCKET=${XDG_RUNTIME_DIR}/fleet/control.sock
FCLI=<desktop-home>/profiles/fleet/node_modules/@cayleb/dsh-fleet/bin/dsh-fleet.mjs
node "$FCLI" op status                       # ok/integrity/rows 8-12/lanes_allowed
node "$FCLI" first-run --status              # version 0.3.1, 123 files
node "$FCLI" op providers-get                # opencode-go route, credential.present=true on the LIVE home
node "$FCLI" op policy-get '{"key":"roster.models"}'
node "$FCLI" op policy-get '{"key":"rates"}'
node "$FCLI" op lane-spawn '{"role":"builder","domain":"acceptance","start":true}'   # Option-A real-provider proof
node "$FCLI" op lane-get '{"id":"builder:1"}'; node "$FCLI" op ledger-get '{"agent":"builder:1"}'
node "$FCLI" op lane-stop '{"id":"builder:1"}'
# first real order: order.stage_real with a real checkpoint_at, then board-get read-back (MCP door path once the
# MCP row question is resolved; otherwise the same staging through the fleet CLI's control ops is not available ->
# named blocker below)
```
**N/A in the single setup (recorded, not skipped silently):** stage2 unit legs (installer/unit identity, systemd
stop/start/SIGKILL 2.11, fleet-uid ownership, unit netfilter/ProtectHome/NoNewPrivs). The S4B proof stays as build
evidence; the consolidated acceptance must be scoped (Nox ruling).

## 6. Named blockers discovered in Phase 1 (need resolution before Phase 2/3)

1. **MCP row is pinned with unit paths:** re-rooting `fleet-mcp` breaks rows integrity; shipped row cannot write
   `/var/lib/fleet` as uid 1000 → MCP DEGRADED. Resolution options: **(a) build a consolidated variant** (0.3.1-cons:
   edit `mcp.row.yml` paths/socket/port for the desktop home + regenerate `rows.manifest.json` + fresh checker; no
   logic change, small patch set) — recommended; or **(b) Nox ruling** that the systemd unit remains the
   MCP/acceptance-only host (an explicit exception to "one setup"); or (c) run the consolidated host with MCP
   disabled (door/proofs provided by another path) — weakest.
2. **Acceptance delta + containment:** unit-grade containment and the unit-driven check legs are not reproducible as
   uid cayleb (unchanged from the plan; ruling needed).
3. **Interpretation:** sibling `fleet` profile under the desktop home = "the single setup" (unchanged; ruling needed).
4. **Phase 3 cutover** additionally needs the Lead's explicit approval at an app-idle moment (single writer: unit
   stopped first; never both writers).

## 7. Recommendation

Proceed with **Phase 1b (offline, safe): build the consolidated variant** — a small patch set on the 0.3.1 branch:
`mcp.row.yml` re-rooted (`statePath`/`logPath`/`adminSocket`/`fleetRoot` under `<desktop-home>/fleet` + runtime dir;
port 8789 kept or a free port, operator choice), rows.manifest regenerated, version bump, suites, fresh checker. That
removes blocker 1 without any live write and makes Phase 3 a start/verification step under the two Nox rulings. The
alternative is ruling (b) — keep the unit as the MCP/acceptance host — which needs no build but keeps a systemd unit
alive contrary to the one-setup direction.
