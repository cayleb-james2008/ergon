# TEAM D — DIAGNOSTIC PLAY REPORT: `traffic:abort-contact` (one Play, 2026-11-02)

Lead GO: 2026-11-02T01:27Z. Machine `outis-desktop`. Run: **one** Play of
`python3 tools/vinegar_run.py run traffic:abort-contact` on the accepted instrumented block (branch tip
`659779ce89fa18c8ab2dc13a29906f7ab05e1499`), followed by the staged `preflight` and `verify-clean`. No retries, no
extra Plays, no image reads, no build, no publication, no credential values. Spend $0.

Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T022940Z-traffic-abort-contact`
Log: `board/probes/team-d/diagnostic-play-20261102.log`.

## 1 · Verdict in one paragraph

The single Play ran and **failed the cell honestly (exit 1)** — the bounded diagnostic now explains **why, in mechanism
terms**: after the cut-in/abort legs worked (`signalled:true`, `maxProgress 0.537`, `abortObservedAt 4.98 s`), the
traffic proxy **resumed normal speed and outran the probe**; the ego↔proxy distance rose from **23.9 studs at
`contactDriveAt 9.00 s` to 862 studs at 23.1 s**, crossed the controller's **`RETIRE_RADIUS = 800` at t=22.283 s**, and
**0.95 s later** `TrafficController:step` executed its documented unobserved-retirement rule
(`car.unobservedSince > 1` → `_retire`). The `GenerationGuard` recorded **exactly one bounded event** — `step 1386,
generation 2→3, bodyReplaced true, oldBodyAlive true, newBodyAlive false, bodiesCount 0, hasChange false, contacts 0,
touches 0` — i.e. **not contact-correlated churn and not a controller defect**: the probe held a proxy that the controller
is entitled to retire once no observer is within 800 studs, and the intended contact never happened (`contacts:0`,
`touches:0`, `contactObservedAt:null`). So the assertion correctly stayed fail-closed; the *cell's* remaining defect is
the **fixture geometry**: the abort-contact maneuver lets the proxy leave the observation radius instead of staying
engaged, so the cell cannot reach its contact leg. Chunk transport was proven again (6 parts); sealed artifacts unchanged
before/after; 1/3 Plays used; Studio alive and clean.

## 2 · Evidence table (verbatim facts)

| Fact | Value |
|---|---|
| Pre-state | Studio pid **388820**, window `0x558c7f6b0160`, 2200×1000 floating, Edit; plays **0/3** for this opening; place == build == `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98`; `preflight` rc **0**, refusals `[]`, host quiet, MCP studio `cdefa06f-…` |
| First `preflight` attempt | **refused (rc 2)**: `host not quiet: memory full avg10 2.24 > 2.0` — caused by my own just-finished test run; nothing touched. Retried after pressure fell (avg10 0.14): rc **0** |
| Run | `run traffic:abort-contact`, started 02:29:40Z, finished 02:30:17Z (**37 s** total; Play runtime 23.23 s) |
| Exit / cell verdict | **exit 1 (FAIL)** — `runtime reported failure`; runner states `LOCKED→PREFLIGHT→STAGED→PREPARED→PLAYING→RAN`, abort `{stop: ok, cleanup: ok, verifyClean: ok}` |
| Runtime report | state `failed`; reason `ServerScriptService.Nightshift.HandlingServer:178: physical generation retired/replaced at step 1386: 2->3 (see generationEvents)`; `mode abort-contact`, `elapsed 23.233 s`, `samples 177`, `contacts 0`, `touches 0`, `signalled true`, `maximumProgress 0.5367` |
| Timestamps in report | `cutInAt 2.883 s`, `abortObservedAt 4.983 s`, `contactDriveAt 9.003 s`, `contactObservedAt null` |
| Generation diagnostic (`report.diagnostic`) | `max 20`, `steps 1386`, `initialGeneration 2`, `finalGeneration 3`, `generationStable false`, `truncated false`, **1 event**: `{t 23.233, step 1386, oldGeneration 2, newGeneration 3, oldBodyAlive true, newBodyAlive false, bodyReplaced true, hasChange false, progress nil, cutInAt 2.883, abortObservedAt 4.983, contactDriveAt 9.003, contactObservedAt nil, contacts 0, touches 0, bodiesCount 0, bodyToken "nil", proxyToken "table: 0x…", source "step"}` |
| Distance series (from 177 samples, ego to proxy) | 75.8 studs at t=0.12 s → **45.8 at abort (4.98 s)** → **23.9 at contact-drive (9.00 s)** → monotonically rising → **first > 800 studs at t=22.283 s** → 862.1 at 23.12 s; ego speed max **17.1** studs/s vs proxy speed 10.3→**52.2** studs/s |
| Retirement timing vs the rule | Controller `TrafficController.RETIRE_RADIUS = 800`, rule `car.unobservedSince and now - car.unobservedSince > 1` → observed **0.95 s** after the first >800 studs sample (heartbeat quantisation), exactly the documented rule |
| Chunk transport (proven again) | collector phases `["prepared","runtime","cleanup"]`, rejections `[]`; runtime delivered via **6 chunks**, xor bytes **1,477,237**, sha256 **`f66e1b6d079076bd9abe8b68c6ff30692d26b43b6c4f4a877d8f1dce7f5c4c3e`**; `report.json` **2,437,349 B** sha256 `8c0ebac41d727823…`; `runtime-http.json` `{running:true,server:true,httpEnabled:true}` |
| Prepare / cleanup | prepared: `helpers 4`, `verifiedSources 101`, `bootstrapDisabled true`, `httpEnabledForRuntime true`; cleanup: `cleaned true`, `helpersAbsent true`, `httpRestored true`, `removedHelpers 4` |
| Host / crash | hostcheck `{stalls 0, worst_ms 0, label "clean"}`; no new crash dumps; `gpuJournal null` |
| Post-state | `verify-clean` rc **0**, `clean:true`, sources 101; Studio alive pid **388820**; plays **1/3** for this opening (`{"pid":388820,"plays":1}`); sealed release `84324d43…`, test/place `c6651793…` **unchanged** |
| Artifacts hashed | BEFORE and AFTER: `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce build/release.rbxlx`; `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98 build/test.rbxlx` and the same for the runner place |

## 3 · What the instrumentation revealed (mechanism, not verdict inflation)

1. **The transport and the instrumentation work in Play.** The runtime report (2.4 MB) arrived through the exact chunk
   transport (6×256 KiB, new sha `f66e1b6d…`), and the bounded recorder captured a single named event with the probe
   context — the first time this failure has been attributable rather than a bare "physical generation retired/replaced".
2. **The retirement is the controller's documented unobserved rule, not unexpected churn.** The event has
   `hasChange:false`, `contacts:0`, `touches:0`; the body was removed from `controller.bodies` (`bodiesCount:0`,
   `bodyToken:"nil"`), and the probe's own distance series crossed `RETIRE_RADIUS=800` at 22.283 s with retirement at
   23.233 s — matching `unobservedSince > 1`. The guard therefore cannot be relaxed for this event: it is not a
   contact-correlated replacement (batch 7 §6's "legit" branch), and it is not a product-defect pattern (no corrupt-body
   telemetry, no multiple/unexplained churn).
3. **The cell's real remaining defect is fixture geometry.** The abort-contact maneuver ends with the proxy faster
   (52 studs/s) than the ego can follow (17 studs/s): distance grows instead of closing, the intended native contact
   never occurs (`contacts:0`, `touches:0`, `contactObservedAt:null`), and the probe then holds a body outside the
   controller's observation contract until the controller retires it. The judgement per batch 7 §6 is therefore:
   **FAIL, not passable by relaxing the assertion** — the probe must stay engaged (keep an observer within
   `RETIRE_RADIUS`, or hold the probed car explicitly) before `traffic:abort-contact` can reach its contact leg. That is a
   scenario/fixture fix for a later ruling, not a change made here.
4. **What this Play does not prove:** it does not certify the transport path as a *cell pass* (the cell still fails),
   does not touch the verified-clean/skip-opening legs beyond what the fixture already asserts, and says nothing about
   `atlas:*`/`moving` (different fixtures; their evidence is the separate trace plans). `worldTrace` is an atlas-only
   field: this traffic runtime report has no `worldTrace` (the traffic report carries `diagnostic` instead).

## 4 · Staged sequence actually run (nothing more)

```bash
cd ~/Work/projects/untitled-street-game
# 1. read-only Studio state + sealed hashes BEFORE (pid/window/Edit/plays 0, place == build == c6651793…)
python3 tools/vinegar_run.py preflight            # one refusal (host memory pressure), nothing touched; retry rc 0
# 2. the one Play
python3 tools/vinegar_run.py run traffic:abort-contact   # exit 1, run dir 20261102T022940Z-traffic-abort-contact
# 3. close-out
python3 tools/vinegar_run.py verify-clean          # rc 0, clean:true
# 4. sealed hashes AFTER (unchanged) + plays (1/3) + Studio pid (388820 alive)
```

## 5 · Holds respected

No retry, no extra Play (1/3 used), no Studio input beyond the runner's own Play/Stop, no build, no upload/publication,
no credential value, no image reads, no writes outside the repo evidence dir / `board/`. $0 of $0.
