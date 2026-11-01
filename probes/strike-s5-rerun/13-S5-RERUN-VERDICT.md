# S5 re-verify verdict — corrected release tarball 0.3.0 (sha256 d03ea2ac…)

Checker: `checker-verify` (fresh-context; built none of this artifact or the fleet). Method identical to the first S5
run: my own throwaway DSH profile booted from the exact tarball bytes, my own MCP client, both loopback and the host's
tailnet address, then independent control-socket re-reads. The builder's `mcp-selftest`/gate were not used as evidence.

**VERDICT: ACCEPT** — the corrected artifact (`bin/rows-manifest.mjs` only) passes the whole S5 battery unchanged, and
the corrected shipped rows-manifest generator reproduces the shipped `rows.manifest.json` byte-identically with
`fleet-mcp` **not** pinned as an exact insert. Boot integrity is ok, including the rows facet against the shipped
manifest.

## Artifact under test

| | |
|---|---|
| Tarball | `/home/cayleb/workspace/harness-build/strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz` |
| SHA-256 | `d03ea2ac8163985732baf1b64a907f698cd62cd41c5ed96be5d72b6d99394aab` — matches the Lead-declared corrected digest |
| Members | 104 (no directory entries) |
| Superseded digest | `b60019a0…` (first S5 run) |
| Delta vs superseded | `diff -rq` over the two extracted package trees: **exactly 1 differing line** — `bin/rows-manifest.mjs` (old sha `9650d753…`, new `143decb2…`). No other shipped file differs. `rows.manifest.json` is **byte-identical** (sha256 `1bcac9862a9b6c9723c27b193b3a9f22ab9b07f9d30e7fef862d01d194d59ee9`) (`01b-artifact-delta.txt`) |
| Installed profile | throwaway `DSH_HOME=/tmp/cv-s5.v43Ga1/dsh`, profile `fleet` from the shipped web template, plugin `0.3.0`; installed tree `diff -rq` vs tarball = **0 differing lines** |
| Composed profile | 15 `fleet-*` rows incl. `fleet-mcp` (my own scratch overlay: paths + port 18889 + `bindHosts: []` only) |
| Listeners :18889 | `127.0.0.1`, `[::1]`, `100.114.17.67` — exact binds, no catch-all |
| Run | 2026-11-01T22:46:33Z → 22:46:39Z; scratch root deleted after the run |

## Boot integrity with the shipped rows manifest (`05b-boot-integrity-rows.json`)

| Component | Result |
|---|---|
| `integrity.ok` | **true**; `failures: []` |
| chain | ok |
| sandbox | ok |
| rows | **ok**, `pinned: 7`, `disabled: 12`, `failures: []` |
| rows manifest identity | facet reports `manifest_sha256 = 1bcac986…`; installed `rows.manifest.json` hashes to the same → match |
| guard | ok (fleet-policy + fleet-ledger required rows mounted) |
| lanes | `lanes_allowed: true` |

The corrected shipped generator was re-run by me against a copy of the installed package
(`DSH_RUNTIME_NM=<codex runtime node_modules> node bin/rows-manifest.mjs`): exit 0, **regenerated `rows.manifest.json`
is byte-identical to the shipped file**, and the pinned list is
`fleet-ledger, fleet-policy, permission, preset-fleet-builder, preset-fleet-checker, preset-fleet-lead,
preset-fleet-researcher` — **`fleet-mcp` is not pinned as an exact insert** (the fix's rule).
Evidence: `05c-rows-manifest-generator.log`, `05c-rows-manifest-pins.txt`.

## My MCP battery result — `probes/mcp-verify.mjs`

87 checks · **84 passed · 0 failed** · 3 informational; `verdict: PASS`; `tailnet_established: true`
(`mcp-evidence/mcp-verify.json`, transcript `07-mcp-probe.log`).

| Test | Result (machine-checked) |
|---|---|
| **T1** auth | missing token → 401 `-32001`, no result envelope; wrong token → 401 `-32001`; action log gained `refused` rows `missing-token` + `invalid-token` with `principal:null`; **zero intent rows** (refused before dispatch); `WWW-Authenticate: Bearer` |
| **T2** both legs | initialize 200 + `Mcp-Session-Id` + `serverInfo fleet-mcp`; `notifications/initialized` 202; `tools/list` = exactly the 9 contracted tools, identical over loopback and tailnet |
| **T3** sessions | `session.create` → `fleet-mcp-…`, mode stub, model `deterministic-stub (fleet-selftest-stub)`; `session.prompt` → `status:completed`, `transport:"stub-turn"`, `turn_agent builder:selftest-…`, `ledger_gate:"stub-provider-selftest-only"`, non-empty assistant text; `session.status` → found, turns 1, `last_transport:"stub-turn"`, same text read back; `mode:"model"` → named refusal `model-credential-not-entered`, no session id |
| **T4** roundtrips | two independent chains: `o1`/`t1` and `o2`/`t3`, each 12 posts, final state done; **re-read over the control socket**: `order-get o1` → state done, `fixture:true`, task `t1` done+banked, final `t2` done; every reported post id present; labels `fixture-validated` / `real_lane: not-built (Stage 4 roster)` |
| **T5** stage_real | named refusal `lanes-unavailable`, `real-lane-staged`, staged path, **`payload_sha256` equals my own recomputation**, payload non-fixture with the sent done-when, done-when executed (`true`, exit 0), **no order row created** |
| **T6** action log | every call has intent-before-outcome (`ref_seq`), principal `cayleb-local`, args only names+digest; whole log file readable on scratch: 24 rows, sha256 `abe4b1367127de8ba42bc08f7bef9b86c2723a884f029d24b1f4017fc50aaa50`, every line parses, full hash chain links+recomputes, **0 bearer-token hits**, 0 argument-value hits; `actions.status open:true broken:null` |
| **T7** source gate (partial, stated) | runtime sources loopback+tailnet CIDRs; bind = three exact hosts; no catch-all; unit netfilter inspected; live foreign-source refusal **not proven** (EADDRNOTAVAIL; needs second host/ns/root) — recorded as not proven |
| **T8** limits | over-limit body → **413 `-32004`** (+`payload-too-large` rows); malformed JSON → **400 `-32700`**; batch → **400 `-32600`**; refusals logged |
| **T9** rate | **429 `-32005`** with `retry-after: 59` (observed after 43 further requests in the window); `rate-limited` refused row logged |

Whole-log and evidence secret scans: token **0 hits** in the action log; **0 hits** across the 62 evidence files
(compared in memory; token never printed/stored).

## Deviations and limits (unchanged from the first S5 run; re-observed, not smoothed)

1. Stub echo: assistant text was `S3STUB:Current runtime context. …` (the session's injected runtime-context turn), not
   the caller's literal text. Recorded as an observed deviation from the tool description; the turn is real, model-free,
   correctly labelled, and read-back-consistent.
2. Batch refusal is refused 400 `-32600` but not appended as a `refused` row (malformed/over-limit bodies are); recorded,
   informational (the log contract does not name batches).
3. Over-limit request produced two `payload-too-large` rows for one client request (observed count 2); recorded exactly.
4. **Installed-service leg remains NOT RUN.** This verdict covers the corrected release bytes on a real host booted from
   them (including the tailnet bind and boot integrity with the shipped rows manifest); the running `fleet-dsh` unit is
   still 0.2.2 with no MCP row. The installed-service 0.3.0 battery is **the only open acceptance scope**.
5. Live foreign-source refusal not proven (runtime allowlist/bind + unit netfilter only).

## Raw evidence (`s5-rerun/`)

`00-run-log.txt` (my full run log) · `01-environment.txt` · `01b-artifact-delta.txt` (only the dev tool moved;
`rows.manifest.json` identical) · `02-installed-tree-diff.txt` (empty = byte-identical) · `03-composed-profile.yml` ·
`05-control-status-before.json` · `05b-boot-integrity-rows.json` · `05c-rows-manifest-generator.log` +
`05c-rows-manifest-pins.txt` · `06-listeners.txt` · `07-mcp-probe.log` · `08-control-status-after.json` ·
`09-mcp-actions.jsonl` + meta · `10-whole-log-token-scan.txt` · `11-evidence-token-scan.txt` · `12-host-log.txt` ·
`mcp-evidence/mcp-verify.json` + `raw/*.json` · `run-s5-rerun.sh` (the exact script used) · `SHA256SUMS`.
