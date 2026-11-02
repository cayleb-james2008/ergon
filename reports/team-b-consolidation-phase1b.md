# Team B — Consolidation Phase 1b: consolidated variant prototype (throwaway copy)

**Filed:** 2026-11-02T09:25Z (04:25 EST) · **Author:** fleet-lead (Team B) · **For:** Lead → Nox ruling on the pinned `fleet-mcp` row
**Fence observed:** throwaway copy only (`~/workspace/harness-build/scratch-tb-consolidation`), no live profile write, no install into the desktop home, no unit change, no spend, no credential values read or printed. Copy host stopped; ports closed.

## 1. Purpose

Nox ruled (091500Z) Option B — a sibling `fleet` profile under the one desktop DSH home — is "one setup", and approved Phase 2 (create the live profile, no start). Phase 3 (cutover) is held pending the ruling on the integrity-pinned `fleet-mcp` row. This report closes the prerequisite question the ruling needs: **can the consolidated variant be built and does it work?** It was prototyped and run in a copy of the desktop home, not in the live home.

The shipped `fleet-mcp` row is an `insert` whose **config** is pinned in `rows.manifest.json`. Any overlay override fails the rows check (`rows fleet-mcp: config changed`), and the shipped config points at unit-only paths (`/var/lib/fleet`, `/run/fleet`, port 8789), which the desktop user cannot create (`EACCES`). A working consolidated MCP therefore requires a **build variant**: re-rooted `mcp.row.yml` + regenerated `rows.manifest.json`.

## 2. Method

| Step | Action | Result |
|---|---|---|
| copy home | fresh `DSH_HOME` copy at `…/scratch-tb-consolidation/home` (profiles/sessions/storages only) | isolated from live home |
| install | candidate r2 tarball `cayleb-dsh-fleet-0.3.1.tgz` (sha256 `5ede7484…aef4`) added to the copy's `fleet` profile | 123 package files |
| variant | edit `node_modules/@cayleb/dsh-fleet/mcp.row.yml`: port 8789→8793, `bindSourceAllow` loopback only, `statePath`/`logPath`/`adminSocket`/`fleetRoot` → copy home + copy runtime dir | diff below |
| manifest | run the package's own generator `node bin/rows-manifest.mjs` (the script `npm run release` uses) | 8 pinned / 12 disabled; byte-reproducible |
| run | `DSH_HOME=<copy> node <cli> --profile fleet --no-open --host 127.0.0.1 --port 8792` | MCP on 8793, web on 8792 |

Variant `mcp.row.yml` delta (only these lines differ; sha `2fb21c89…`):

```
-        port: 8789                      +        port: 8793
-          - tailnet                    (bindSourceAllow reduced to loopback)
-        statePath: /var/lib/fleet/state/mcp
-        logPath: /var/lib/fleet/log/mcp-actions.jsonl
-        adminSocket: /run/fleet/mcp-admin.sock
-        fleetRoot: /var/lib/fleet
+        statePath/logPath/adminSocket/fleetRoot under the copy home and copy runtime dir
```

## 3. Manifest regeneration (the ruling-critical bit)

| File | Original (shipped 0.3.1) | Variant |
|---|---|---|
| `rows.manifest.json` sha256 | `1970f9954b7131c58191327bb0e60b9d31a704bc309d083cd53971bc9271edba` | `fcead374c23d0343b45111e3e785c2d03ff5bba712c923db1b12bfb142313296` |
| pinned `fleet-mcp` config hash | `c100a90c98cc302a4003077589ac4ed7e9d18dc71bcf53a42f4596bc104e78ad` | `d136c44be94d8f3186ee54b1853021a31e08887bb9223cfffb11c7d641c68d09` |
| pinned `fleet-mcp` name hash | `1765ad50…3eea` | unchanged `1765ad50…3eea` |
| all other pins (7) | — | unchanged |

**Reproducible:** rerunning `node bin/rows-manifest.mjs` in the copy re-emits `fcead374…` byte-for-byte, prints `8 pinned row(s), 12 disabled id(s)`, rc=0 (`phase1b-manifest-gen.log`). The generator reads the package's own patch files (`cordis.patch.yml`, `presets/*.patch.yml`) and needs the DSH runtime node_modules (`DSH_RUNTIME_NM`, else the pinned path); it is not a hand edit.

## 4. Results (copy host, transcript sha256 `470557ac…f18`)

- **No degraded rows.** `host5.log` = one `dsh web:` line, **0** `DEGRADED` lines. Control run with the *shipped* row (`host2.log`) produced 3: `token-file-unavailable: EACCES … /var/lib/fleet/state/mcp`, `action-log-unavailable: EACCES … /var/lib/fleet/log`, `not listening`.
- **MCP listening:** `127.0.0.1:8793` and `[::1]:8793` (loopback only — tailnet removed in this prototype).
- **initialize:** ok, `serverInfo {name: fleet-mcp, version: 0.3.1-mcp}`, protocol negotiated `2025-03-26`, session issued.
- **tools/list:** **9 tools** — `fleet.status, session.create, session.prompt, session.status, board.op, order.drive_roundtrip, order.stage_real, fleet.selftest, actions.status`.
- **`fleet.status`:** `integrity {ok: true, failures: []}`, `lanes_allowed: true`, `real_lane: "fleetRoster mounted"`, action log `jsonl-hash-chain` open at seq 15, control socket under the copy runtime dir, `model_route: deterministic-stub (fleet-selftest-stub)`.
- **`order.stage_real`:** ok, `mode: "real-lane"`, order `o3` `state: open`, staged file under `<copy>/fleet/state/mcp/staged/real-order-1793611270610-05c33e60c5.json`, `payload_sha256 c7fb0cc8cfca1925bb33a4eb4bd18597b147f5e5af709224f9a9b6657a0eec5c`. (An earlier run in the same copy staged `o2` at 09:03:36Z.)
- **`board.op board-get`:** ok, resolved `board-get`, `transport: control`, order row `fixture: false`, `state: open`.
- **`actions.status`:** ok, hash chain `broken: null`, head `b73e5340…`→`dc067f0c…`, intent+outcome row per call, argument **names and sha256 digests only** (no values).
- **Preset session probe** (`phase1b-session-probe.log`): `session.create {preset: builder}` ok — real DSH session, `mode: stub`, `provider: fleet-selftest-stub`, cwd under `<copy>/fleet/work/mcp-sessions/…`; with a first prompt the turn completes as `transport: "stub-turn"`, `ledger_gate: "stub-provider-selftest-only"`, `provider_calls: 0`. Note: `assistant_text: ""` on the stub-turn path — corroborates the known stub-turn residual (B3); not new.

Earlier in-session observation (09:03Z, not re-captured in the durable transcript, so treated as observed-only): a roster `lane-spawn {start:false}` produced lane `builder:1` on provider `opencode-go` with cwd under the copy home's `fleet/work/lanes`, and `board-get` reported `fixture:false`, `tasks: 0`.

## 5. Conclusion for the ruling

1. **Option (a) is buildable and reproducible**: the consolidated variant differs from shipped 0.3.1 by one row config + one regenerated pin, and the package's own generator reproduces the manifest deterministically. In the copy it boots clean (no degraded rows), integrity passes, MCP serves the 9 tools, and the scoped acceptance set passes.
2. **A second pinned-config family was found in Phase 2** (see `team-b-consolidation-phase2.md` §6): the four `preset-fleet-*` insert rows pin `customSkillDirs: [/var/lib/fleet/skills]`. The copy proves a preset session still *starts* with that path unreadable (no mount failure), so option (b) degrades skills only — but skill availability was not exercised. If the ruling wants full parity, the variant must also re-root the four preset patch files (same generator step).
3. **Still open before Phase 3:** live variant port (prototype used 8793; live would need a port distinct from the unit's 8789 only if both ever run — single-writer forbids that), whether the variant keeps `tailnet` binding (the prototype dropped it; Nox's remote access may want it back), and whether presets are re-rooted.
4. **Fence held:** the live desktop profile was not touched by any of this; Phase 2's before/after hashes prove it (`team-b-consolidation-phase2.md` §7).

## 6. Evidence (all under `board/probes/team-b-consolidation/`)

| File | sha256 (head) | What it shows |
|---|---|---|
| `phase1b-mcp-transcript.log` | `470557ac…` | 9 tools, fleet.status integrity ok, order.stage_real o3, board-get, actions.status |
| `phase1b-session-probe.log` | `82ce3a84…` | preset session create + stub turn in the copy |
| `phase1b-manifest-gen.log` | `3a5ba1bb…` | generator rc=0, 8 pinned/12 disabled |
| `phase1b-copy-host5-nodegraded.log` | `ae541dbb…` | copy host start, no DEGRADED lines (web token redacted) |
| `phase1b-copy-host6-session.log` | `ae541dbb…` | session-probe host start (web token redacted) |
| `host-copy-proof.log` | `51f23d3f…` | shipped-row control run: 3 DEGRADED lines |
| `dump-composed-copy.yml` | `bb1bb4b1…` | composed copy tree (59,749 B) |
| `phase1b-mcp-client.py` / `phase1b-session-probe.py` | — | the clients used (token read from file, never printed) |
