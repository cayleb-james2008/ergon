# TEAM D — DIAGNOSTIC PLAY 3 REPORT: fresh opening + `atlas:scout --seed 2134783819` (one Play, 2026-11-02)

Lead GO: 2026-11-02T02:59Z. Machine `outis-desktop`. Sequence: `studiocycle cycle` (fresh opening) → read-only state +
sealed hashes BEFORE → `preflight` → **one** `run atlas:scout --seed 2134783819` → `verify-clean` → sealed hashes AFTER.
No extra Plays, no retries, no build/publication/credential, no image reads. Spend $0.

Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T030404Z-atlas-scout`
Log: `board/probes/team-d/diagnostic-play3-20261102.log`.

## 1 · Verdict in one paragraph

The fixed starter **ran**, the fast path was **observable inside the bound**, the trace-retention plumbing worked on the
fail-fast path — and the run exposed the **next one-line wiring bug**, so the cell still FAILED fail-closed (exit 1).
The starter applied the named seed before the world was built (`starter t0=24.832 s < rootFoundAt=24.889 s`, `applied
=2134783819`), the world reports `actualSeed=2134783819 / actualLayout=2` (== expected, `rebuild=false`), i.e. **the
`World.dailySeed` fast path won the ordering race**. The published trace (`result.json.worldTrace` **and**
`world-trace.json`, identical) shows `fastPathApplied:false` only because that is the **pre-check default** (C1 publish-
before-check) — the check itself then died on the next legacy assert: `AtlasWorldSeed:66: the world root is required`
(`type(root)=="table"` rejects the Roblox `Folder` Instance; `AtlasServer:80` → `requireApplied`). The old
`WorldSeed.ensure` carried the same `type(root)=="table"` assert, so this bug was hidden behind the starter crash. So:
**Play A does not yet give a clean `fastPathApplied=true` verdict, but it strongly indicates the fast path wins and
assert-and-skip will suffice**; one one-line fix + a repeat Play A is needed. Sealed artifacts unchanged; 1/3 Plays used;
Studio alive and clean.

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Fresh opening | `studiocycle cycle`: close `method=killed-after-dialog` (autorecovery parked, nothing saved), place sha unchanged `c6651793…`; reopen **pid 759088**, window `0x558c7fa45500`, 2200×1000 floating, place sha before==after `c6651793…`; `preflight` rc 0, refusals `[]`, **Edit**, **plays 0/3**, place == build == `c6651793…`, host quiet |
| Sealed BEFORE | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` |
| Run | started 03:04:04Z, finished 03:04:27Z (**23 s**); states `LOCKED 0.0 → PREFLIGHT 0.0 → STAGED 1.864 → PREPARED 2.005 → PLAYING 3.913 → RAN 7.668` |
| Exit / error | **exit 1**; `FixtureFailed: view-001: runtime arrived before view-001: {…}`; abort `{stop: ok, cleanup: ok, verifyClean: ok}` |
| Collector | phases `["prepared","runtime","cleanup"]`, rejections `[]`; chunks `null` (small runtime error POST, direct path) |
| Runtime report (error) | `ServerScriptService.Nightshift.AtlasWorldSeed:66: the world root is required` / `AtlasWorldSeed:66 function requireApplied` / `AtlasServer:80 function start` / `AtlasServerStarter:1`; `report` also carries `worldTrace` + `starter` (the new trace-bearing failure post) |
| `worldTrace` (identical in `result.json.worldTrace`, `world-trace.json`, runtime report) | `{"t0":24.889171899994835,"rootFoundAt":24.88918110000668,"worldReadyAt":30.230887299985625,"actualSeed":2134783819,"actualLayout":2,"expectedSeed":2134783819,"expectedLayout":2,"fastPathApplied":false,"rebuild":false,"starter":"{\"applied\":2134783819,\"t0\":24.832430699956605,\"seed\":2134783819}"}` |
| Fast-path evidence | starter ran at **t=24.832 s** (before `rootFoundAt` 24.889 s and `worldReadyAt` 30.231 s), `applied=2134783819`; world reports `actualSeed/actualLayout == expected`; `rebuild=false`; no view was captured |
| Bound behaviour | failure arrived via the early-stop `ValueError` well inside the **45 s** first-view bound (no timeout, no hang); trace attached to the failure (C1+C2 plumbing verified in Play) |
| 45 s bound | not hit (run 23 s); the trace is observable within the bound when the server fails fast |
| Chunks / diagnostic | chunks `null`; no `generationEvents` (this is the atlas fixture; `diagnostic` is the traffic instrument) |
| Host / crash | hostcheck `{stalls 0, label "clean"}`; no new crash dumps |
| Sealed AFTER | identical to BEFORE (both artifacts) |
| Post-state | `verify-clean` rc 0, `clean:true`; Studio alive pid **759088**; plays **1/3** for the fresh opening; place == build == `c6651793…` |

## 3 · Judgement per batch 7 §6 (atlas:scout)

- **Verdict class:** **fail-fast** (named, trace-bearing), **not** a hang and **not** skip-rebuild: the starter and the
  server both ran; the failure is `requireApplied`'s legacy arg assert rejecting the Instance root.
- **Was `fastPathApplied` observable within the bound?** The **trace** was published and retained inside the bound; the
  `fastPathApplied` **field** shows `false` only because it is published **before** the check (C1 design) and the check
  aborted. The **identity evidence** (`actualSeed/layout == expected`, `rebuild=false`, starter `applied=seed` at
  t=24.832 s before the world was built) is conclusive that the fast path **won**; a clean `fastPathApplied=true`
  verdict needs the one-line fix below and one repeat Play.
- **Fallback (a)?** Not indicated: the world was built by the normal Bootstrap path with the named seed; no rebuild was
  needed or attempted.
- **What it proves:** (i) the starter fix works — the seed modules are `require`d and `apply` runs on the module table;
  (ii) the starter's fast-path patch wins the ordering race for seed 2134783819; (iii) the C1 publish-before-check and
  the early-stop trace retention work in Play (trace in `result.json`, `world-trace.json` and the runtime error post);
  (iv) assert-and-skip will succeed once the arg check accepts an Instance; (v) the same `type(root)=="table"` assert
  existed in the old `ensure`, so it was masked by the starter crash.
- **What it does not prove:** no clean `fastPathApplied=true` run yet; no views captured; the cell does not pass; the
  abort-contact fixture remains unchanged and still gated.

## 4 · Proposed fix (NOT implemented — needs the Lead's go, next offline change)

One line in `tests/atlas/WorldSeed.luau::requireApplied`: replace
`assert(type(root) == "table" and type(root.GetAttribute) == "function", "the world root is required")`
with an Instance-compatible check, e.g. `assert(root ~= nil and type(root.GetAttribute) == "function", "the world root is
required")` (the world root is a Folder Instance; `type(Instance)` is `userdata`, `typeof` is `"Instance"`). Add a Lune
spec case using a real `@lune/roblox` `Instance` root (attributes via `SetAttribute`) to prove the Instance path, keep
the fake-table path, and fold **R1** (the two C1 ordering assertions dropped from the atlas-acceptance test) into the
same test-only change. Then repeat Play A once on the fresh opening (2 Plays left: Play A-repeat + reserved Play B).
