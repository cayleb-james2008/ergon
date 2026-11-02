# TEAM D — DIAGNOSTIC PLAY 2 REPORT: `atlas:scout --seed 2134783819` (one Play, 2026-11-02)

Lead GO: 2026-11-02T02:35Z. Machine `outis-desktop`. Run: **one** Play of
`python3 tools/vinegar_run.py run atlas:scout --seed 2134783819` on the accepted instrumented block (branch tip
`659779ce89fa18c8ab2dc13a29906f7ab05e1499`), staged `preflight` before and `verify-clean` after. No retries, no extra
Plays, no image reads, no build, no publication, no credential values. Spend $0.

Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T023321Z-atlas-scout`
Log: `board/probes/team-d/diagnostic-play2-20261102.log` · console capture:
`board/probes/team-d/console-after-atlas-play-20261102.json`.

## 1 · Verdict in one paragraph

The Play **FAILED fail-closed (exit 1)**: `TimeoutError: collector did not receive view-001 within 45s` — the new
**45 s first-view bound fired** (previously a 600 s silence). The named-seed path did **not** hang in a rebuild and did
**not** reach assert-and-skip: the read-only Studio console proves the **seeded starter died on line 1** —
`apply is not a valid member of ModuleScript "ServerScriptService.Nightshift.AtlasWorldSeed"` — because `SEED_PATCH`
calls `S.apply(W, seed)` where `S` is the **ModuleScript Instance**, never `require`d. That error sits **outside** the
starter's `xpcall`, so no runtime phase is posted, the server never starts, no `AtlasWorldTrace` is published, and the
runner can only time out. **`fastPathApplied` was therefore NOT observable** (trace nil, no `world-trace.json`), and this
is not the fallback-(a) case: the assert-and-skip code was never executed. The same bug exists in the 2026-11-01
`SEED_PATCH` (verified via `git show 9022c53^`), so it **supersedes the earlier "in-Play City+Highway rebuild hang"
inference** — the rebuild never ran. The real named-seed blocker is this one-line starter wiring bug; the 45 s bound,
sealed-hash guard, clean abort and trace-retention plumbing all behaved exactly as designed.

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Pre-state | Studio pid **388820**, window `0x558c7f6b0160`, 2200×1000 floating, Edit; plays **1/3**; place == build == `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98`; `preflight` rc **0**, refusals `[]`, host quiet |
| Sealed BEFORE | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`, test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` |
| Run | `run atlas:scout --seed 2134783819`, started 02:33:21Z, finished 02:34:21Z (**60 s** total; Play window ~45 s) |
| Exit / error | **exit 1**; `TimeoutError: collector did not receive view-001 within 45s`; abort `{stop: ok, cleanup: ok, verifyClean: ok}` |
| States | `LOCKED 0.0 → PREFLIGHT 0.0 → STAGED 3.134 → PREPARED 3.245 → PLAYING 5.134 → RAN 7.951` (then the 45 s first-view bound) |
| Collector | phases `["prepared","cleanup"]` — **no `runtime` and no views**; rejections `[]`; chunks `null` |
| Prepared | ok: `verifiedSources 101`, `namedSeed 2134783819`, `localPlace 0`, `skipOpeningFares 2`, `productionSourcesUntouched true`; helpers installed: 7 (`AtlasRoadGround, AtlasExactReportTransport, AtlasPaths, AtlasServer, AtlasClient, AtlasCameraState, AtlasWorldSeed`) |
| Trace | `result.json.worldTrace` **null**; **no `world-trace.json`**; `report.json` 988 B (prepared+cleanup only) |
| Studio console (read-only MCP, after the run) | `apply is not a valid member of ModuleScript "ServerScriptService.Nightshift.AtlasWorldSeed"` / `Stack Begin` / `Script 'ServerScriptService.Nightshift.AtlasServerStarter', Line 1` / `Stack End` — the only atlas-side error; the game itself loaded normally (art 24 models, preview boards). Full text: `board/probes/team-d/console-after-atlas-play-20261102.json` |
| Root cause | `tools/atlas_acceptance.py:SEED_PATCH` (current and pre-Plan-4): `local S=assert(script.Parent:WaitForChild("AtlasWorldSeed",20)); S.apply(W,seed)` — `S` is the **Instance**, so `apply` is not a member; the `require` is missing. The error is outside `STARTER_SOURCE`'s `xpcall`, so it is silent (no runtime error post) → no phase → runner timeout |
| Host / crash | hostcheck `{stalls 0, label "clean"}`; no new crash dumps; `gpuJournal null` |
| Sealed AFTER | identical to BEFORE (both artifacts) |
| Post-state | `verify-clean` rc **0**, `clean:true`, sources 101; Studio alive pid **388820**; plays **2/3** for this opening (`{"pid":388820,"plays":2}`) |

## 3 · Judgement per batch 7 §6 (atlas:scout)

- **Fail-fast vs skip-rebuild vs hang:** observed = **silent starter crash, then the bounded timeout** — *not* the
  named-seed fail-fast (no trace, no runtime error) and *not* a rebuild. The assert-and-skip code was never reached.
- **Was `fastPathApplied` observable within 45 s?** **No.** The server never started, so `AtlasWorldTrace`/`AtlasStarterTrace`
  were never written; trace is nil and there is no `world-trace.json`. The 45 s bound did, however, make the failure
  observable in 45 s instead of 600 s, and the console pinpointed the cause.
- **Fallback (a) (Bootstrap-disabled named-world rebuild)?** **Not applicable / not indicated:** we did not observe
  `fastPathApplied=false`; we observed the starter never running. Fallback (a) stays conditional on a real evaluation.
- **What the Play proves:** the named-seed blocker is a starter wiring bug (`S.apply` on the Instance), present in both
  the old and current `SEED_PATCH`; the 2026-11-01 "named-seed in-Play rebuild hang" reading is wrong (the rebuild was
  never invoked); the 45 s bound, fail-closed abort, sealed-hash guard and clean verify all work.
- **What it does not prove:** nothing yet about whether the `World.dailySeed` fast path wins the ordering race, about
  assert-and-skip behaviour in Play, or about fallback (a) being needed. One Play after the starter fix will answer that
  (`worldTrace.fastPathApplied`, now reachable).

## 4 · Proposed named fix (NOT implemented in this Play — for the Lead's ruling)

Two small test/starter-side changes (no production code), each testable offline:
1. **The one-line wiring fix** in `tools/atlas_acceptance.py` `SEED_PATCH`:
   `local S=require(assert(script.Parent:WaitForChild("AtlasWorldSeed",20)))`, and likewise
   `local W=require(assert(script.Parent:WaitForChild("World",20)))` (used as the module table); keep
   `S.apply(W,seed)` / `S.applied(W)` on the required tables.
2. **Make any starter failure loud:** wrap the entire `SEED_PATCH` in the same `xpcall` that `STARTER_SOURCE` uses (or
   move the seed patch inside it) so a starter error always posts a `runtime` error report carrying `AtlasWorldTrace` —
   the silent-crash class becomes visible instead of a timeout.
3. Offline regression tests: execute the real `SEED_PATCH` text under Lune against a fake Instance tree + `require`
   (asserts `require` is called on the AtlasWorldSeed instance, `apply` receives the required World table, and the
   `AtlasStarterTrace` attribute is written), plus a Python source-shape assertion that the patch `require`s both
   modules and sits inside an error-reporting wrapper.
Play budget: **1 Play left in this opening (2/3 used)**. The starter-fix Play (to observe `fastPathApplied`) and the
abort-contact fixture Play (design note `board/probes/team-d/abort-contact-fixture-geometry-fix-20261102.md`) compete for
it; a `studiocycle` close/reopen gives a fresh 3-Play opening when the Lead sequences them.

## 5 · Staged sequence and holds

```bash
# 1. read-only state: pid/window/Edit, plays 1/3, place == build == c6651793…, sealed BEFORE
python3 tools/vinegar_run.py preflight            # rc 0, refusals []
# 2. the one Play
python3 tools/vinegar_run.py run atlas:scout --seed 2134783819   # exit 1, TimeoutError view-001 within 45s
# 3. close-out
python3 tools/vinegar_run.py verify-clean         # rc 0, clean:true
# 4. sealed AFTER (unchanged), plays 2/3, Studio alive; read-only MCP get_console_output for the root cause
```
No retry, no extra Play, no build/publication/credential, no image reads, no Studio input beyond the runner's Play/Stop.
$0 of $0.
