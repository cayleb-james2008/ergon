# S5 re-verify verdict #2 — FINAL release tarball 0.3.0 (sha256 8173d13e…)

Checker: `checker-verify` (fresh-context; built none of this artifact or the fleet). Same S5 method as `s5/` and
`s5-rerun/`, run unchanged on the final bytes, plus a supplementary read-only smoke of the two changed first-run
surfaces. The builder's `mcp-selftest`/scratch gate were not used as evidence.

**VERDICT: ACCEPT** — the final artifact passes the whole S5 battery over loopback and the real tailnet bind, boot
integrity is ok against the shipped rows manifest, and the delta is exactly the two first-run files the Lead named.
The first-run CLI's own `--status` manifest matches the tarball's 104 members by path, sha256 and mode (independently
cross-checked), and the corrected capability map classifies the OpenCode bridge as a dependency rather than ABSORB.

## Artifact under test

| | |
|---|---|
| Tarball | `/home/cayleb/workspace/harness-build/strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz` |
| SHA-256 | `8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070` — matches the Lead-declared final digest |
| Members / modes | **104** files: **101 × 0644 + 3 × 0755** (executables: `bin/dsh-fleet.mjs`, `bin/mcp-selftest.mjs`, `src/core/bin/dsh-fleet.mjs`) |
| Delta vs `d03ea2ac` (previously verified) | `diff -rq` over the two extracted trees: **exactly 2 differing files** — `src/firstrun/credentials.mjs`, `src/firstrun/capabilities.mjs`; every other file identical. `rows.manifest.json` byte-identical (`1bcac986…`) (`01b-artifact-delta.txt`) |
| Installed profile | throwaway `DSH_HOME=/tmp/cv-s5.ALdnCQ/dsh`, profile `fleet` from the shipped web template, plugin `0.3.0`; installed tree `diff -rq` vs tarball = **0 differing lines** |
| Composed profile | 15 `fleet-*` rows incl. `fleet-mcp` (scratch overlay: paths + port 18889 + `bindHosts: []` only) |
| Listeners :18889 | `127.0.0.1`, `[::1]`, `100.114.17.67` — exact binds, no catch-all |
| Run | 2026-11-01T23:01:59Z → 23:02:21Z; scratch root deleted after the run |

## Boot integrity with the shipped rows manifest (`05b-boot-integrity-rows.json`)

`integrity.ok true`, chain ok, sandbox ok, **rows ok** (`pinned: 7`, `disabled: 12`, `failures: []`, manifest sha
`1bcac986…` == installed `rows.manifest.json`), guard ok, `lanes_allowed: true`. The corrected shipped generator again
reproduces `rows.manifest.json` byte-identically and pins only
`fleet-ledger, fleet-policy, permission, preset-fleet-{builder,checker,lead,researcher}` — **`fleet-mcp` is not pinned
as an exact insert** (`05c-rows-manifest-generator.log`, `05c-rows-manifest-pins.txt`).

## My MCP battery (unchanged) — `probes/mcp-verify.mjs`

87 checks · **84 passed · 0 failed** · 3 informational; `verdict: PASS`; `tailnet_established: true`
(`mcp-evidence/mcp-verify.json`, transcript `07-mcp-probe.log`).

| Test | Decisive result |
|---|---|
| **T1** auth | missing/wrong token → HTTP 401, `-32001`, **no result envelope**; action log gained `refused` rows `missing-token`/`invalid-token` with `principal:null`; **zero intent rows** (refused before dispatch) |
| **T2** both legs | initialize 200 + `Mcp-Session-Id` + `serverInfo fleet-mcp`; `notifications/initialized` 202; `tools/list` = exactly the nine contracted tools, identical on loopback and tailnet |
| **T3** sessions | create → `fleet-mcp-…`, mode stub, model `deterministic-stub (fleet-selftest-stub)`; prompt → `status:completed`, `transport:"stub-turn"`, `turn_agent:"builder:selftest-…"`, `ledger_gate:"stub-provider-selftest-only"`, non-empty assistant text; status read-back (found, turns 1, same transport/text); `mode:"model"` → named refusal `model-credential-not-entered`, no faked call |
| **T4** roundtrips | two chains `o1`/`t1` and `o2`/`t3`, 12 posts each, final state done; **re-read over the control socket**: `order-get o1` → state done, `fixture:true`, task `t1` done+banked, final `t2` done; posts present; labels `fixture-validated` / `real_lane: not-built (Stage 4 roster)` |
| **T5** stage_real | refusal `lanes-unavailable`, `real-lane-staged`, staged path, **payload_sha256 equal to my own recomputation**, non-fixture payload with the sent done-when, done-when executed (`true`, exit 0), **no order row created** |
| **T6** action log | every call intent-before-outcome (`ref_seq`), principal `cayleb-local`, args only names+digest; whole log file: **24 rows**, sha256 `c61462cef780f29f19efb39faad82428af3028ba62cc676e56c570717766f120`, every line parses, full chain links+recomputes, **0 bearer-token hits**, 0 argument-value hits; `actions.status open:true broken:null` |
| **T7** source gate (partial, stated) | runtime sources loopback+tailnet CIDRs; bind = three exact hosts; no catch-all; unit netfilter inspected; live foreign-source refusal **not proven** (EADDRNOTAVAIL) — recorded as not proven |
| **T8** limits | over-limit → **413 `-32004`** (+`payload-too-large` rows); malformed JSON → **400 `-32700`**; batch → **400 `-32600`**; refusals logged |
| **T9** rate | **429 `-32005`** with `retry-after: 59` (after 43 further requests in the window); `rate-limited` row logged |

Whole-log and evidence secret scans: token **0 hits** in the action log; **0 hits** across the 63 evidence files.

## Supplementary smoke — the two changed first-run surfaces (read-only)

| Check | Result |
|---|---|
| `dsh-fleet first-run --status` (vs scratch host) | exit 0; manifest `@cayleb/dsh-fleet 0.3.0`, DSH pin `0.2.2-alpha.2`, **104 files** |
| status manifest vs tarball, hashed by me | **104/104 paths match with identical sha256 and mode; 0 tarball files unlisted** (`14-firstrun-status-vs-tarball.txt`) |
| `first-run --credentials` non-interactive | exit 0, prints `credential-entry-required` owner-act path (no credential written) |
| `first-run --plan-out <tmp>` | exit 0, plan written; capability map classifies the OpenCode bridge as `KEEP-AS-DEPENDENCY`/dependency, **not ABSORB** (`13-firstrun-smoke/plan-capabilities-check.txt`) — the corrected capability logic is live |
| Interactive TTY secret entry | **not exercisable non-interactively**; the changed `createTTYSecretReader` (echo-off-before-prompt, guarded raw mode) was read in the diff and is covered by the builder's tests — recorded as a limit, not claimed as run |

## Deviations and limits (unchanged, re-observed)

1. Stub echo is the injected runtime-context turn (`S3STUB:Current runtime context…`), not the caller's literal text —
   recorded; the turn is real, model-free, correctly labelled and read-back-consistent.
2. Batch refusal is 400 `-32600` but not appended as a `refused` row (malformed/over-limit bodies are); recorded.
3. Over-limit request produced two `payload-too-large` rows for one client request; recorded exactly.
4. **Installed-service leg remains NOT RUN**: the running `fleet-dsh` unit is still 0.2.2 with no MCP row. This verdict
   covers the final bytes on a host booted from them. The installed-service 0.3.0 battery is **the only open
   acceptance scope**, still queued on the owner act.
5. Live foreign-source refusal not proven (runtime allowlist/bind + unit netfilter only).

Fleet untouched: `fleet-dsh` active, `MainPID 427587`, `InvocationID 9e3929d0…` unchanged; no `/tmp` rehearsal/scratch
leftovers; nothing installed or changed by me.

## Raw evidence (`s5-rerun2/`)

`00-run-log.txt` (full run log) · `01-environment.txt` · `01a-member-modes.txt` · `01b-artifact-delta.txt` (exactly the
two first-run files; rows.manifest identical) · `02-installed-tree-diff.txt` (empty = byte-identical) ·
`03-composed-profile.yml` · `05-control-status-before.json` · `05b-boot-integrity-rows.json` ·
`05c-rows-manifest-generator.log` + `05c-rows-manifest-pins.txt` · `06-listeners.txt` · `07-mcp-probe.log` ·
`08-control-status-after.json` · `09-mcp-actions.jsonl` + meta · `10-whole-log-token-scan.txt` ·
`11-evidence-token-scan.txt` · `12-host-log.txt` · `13-firstrun-smoke/` · `14-firstrun-status-vs-tarball.txt` ·
`15-fleet-health.txt` · `mcp-evidence/mcp-verify.json` + `raw/*.json` · `run-s5-rerun2.sh` (exact script) ·
`SHA256SUMS`.
