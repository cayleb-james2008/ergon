# TEAM D — DIAGNOSTIC PLAY 5 REPORT: fresh opening + atlas Play — 20/24 views captured, fog layer clean (2026-11-02)

Lead GO: 2026-11-02T04:03Z (after fog-guard delta ACCEPT). Machine `outis-desktop`. Sequence: sealed hashes BEFORE →
`studiocycle cycle` (fresh opening) → read-only verification + `preflight` → **one** `run atlas:scout --seed
2134783819` → `verify-clean` → sealed hashes AFTER. No extra Plays, no retries, no build/publication/credential, no
image reads. Spend $0.
Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T040412Z-atlas-scout`
Log: `board/probes/team-d/diagnostic-play5-20261102.log`.

## 1 · Verdict in one paragraph

**The fog layer is clean in Play and the atlas cell captured views for the first time: 20 of 24 views were captured,
each distinct, each with `viewportMatchesPng:true` (1837×744) — the outstanding `viewportMatchesPng` observation is now
recorded.** The fast path held again (`fastPathApplied=true`, actual==expected 2134783819/2, `rebuild=false`, starter
applied before the world build), and the fog guard behaved exactly as ruled: the sidecars record
`fog={available:{start:true,finish:true,color:true,enabled:false}, start:200, finish:700, color:[…], legacy:true}` — no
raise, no fabricated `enabled`, no fog-off claim. The run then failed fail-closed at **view-021 `highway-overhead`**:
`screen_capture returned the same bytes as an earlier view (stale capture)`. The client had acknowledged view 21 and was
holding its pose (console: `ATLAS VIEW READY 21/24 highway-overhead`), all six prior overhead/garage/gate views captured
distinctly, so the duplicate came from the capture path, not the scene plan; the runner's stale guard (which exists to
keep capture evidence honest) correctly refused it. **Layer 4 = capture freshness at rapid overhead stepping** — named
for a separately committed fix. Sealed artifacts unchanged; **1/3 Plays used** in the fresh opening; Studio alive/clean.

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Fresh opening | `studiocycle cycle`: close `method=killed-after-dialog` (autorecovery parked, nothing saved), place sha unchanged `c6651793…`; reopen **pid 797879**, window `0x558c7fa15fe0`, 2200×1000 floating, place sha before==after `c6651793…`; `preflight` rc 0, refusals `[]`, **Edit**, **plays 0/3**, place == build == `c6651793…`, host quiet |
| Sealed BEFORE | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` |
| Run | started 04:04:12Z, finished 04:04:44Z (**32 s**); states `LOCKED 0.0 → PREFLIGHT 0.0 → STAGED 3.539 → PREPARED 3.653 → PLAYING 5.539 → RAN 8.506`; exit **1** |
| Error / cell verdict | `FixtureFailed: view-021: screen_capture returned the same bytes as an earlier view (stale capture)`; abort `{stop: ok, cleanup: ok, verifyClean: ok}` |
| Collector | phases `["prepared","view-001" … "view-021","cleanup"]` (21 view acknowledgements), rejections `[]`; chunks `null` (no >512 KiB report in this fixture path) |
| `worldTrace` (`result.json` + `world-trace.json`) | `{"t0":26.775806699995883,"rootFoundAt":26.775809500017203,"worldReadyAt":30.87460020004073,"actualSeed":2134783819,"actualLayout":2,"expectedSeed":2134783819,"expectedLayout":2,"rebuild":false,"fastPathApplied":true,"starter":"{\"applied\":2134783819,\"t0\":26.73490820004372,\"seed\":2134783819}"}` |
| Captures | **20 sidecars**, 20 **distinct** sha256, all `accepted:true`, `label:"stationary"`, `mimeType:image/jpeg`, viewport `[1837,744]` == png `[1837,744]` → **`viewportMatchesPng:true` on every view** |
| Fog record (all 20 sidecars, identical) | `{"available":{"start":true,"finish":true,"color":true,"enabled":false},"color":[…],"finish":700,"start":200,"legacy":true}` — `enabled` absent, `fogOffApplied` absent (scout batch), no client error |
| Technology record | `technology:"Future"`, `technologySource:"sealed-place-xml"`, evidence `{enum,file,mapping,project,raw,sha256}` — the offline derivation lands in real sidecars |
| Lighting extras | `sky:{present:true}`, `atmosphere:{present:true,haze:1.25,density:0.4,glare:0.1,offset:0.1}` |
| Console (read-only MCP) | `ATLAS VIEW READY 1/24 … 21/24 highway-overhead` (client held each pose; no atlas error before 21); game loaded normally |
| Host / crash | hostcheck `{stalls 0, label "clean"}`; no new crash dumps |
| Sealed AFTER | identical to BEFORE (both artifacts) |
| Post-state | `verify-clean` rc 0, `clean:true`; Studio alive pid **797879**; plays **1/3** for the fresh opening |

## 3 · Judgement per batch 7 §6 (atlas:scout)

- **Layer status:** fog-capability layer **CLEAN in Play** (legacy absence recorded, values read for the surviving
  members, no raise, no false claim); `viewportMatchesPng:true` **observed** (the datum the atlas cells were meant to
  supply); 20/24 views captured and held.
- **New named layer (layer 4): capture freshness / duplicate capture.** At view-021 (`highway-overhead`) the runner's
  capture returned bytes identical to an earlier captured view. Evidence rules out a bad pose: the client acknowledged
  view 21 and the console shows the pose held; the 20 prior captures are all distinct and the 6 immediately preceding
  views (garage, city-overhead, gates, highway roads) are distinct. The stale guard is the correct fail-closed response —
  no evidence was fabricated. Mechanism to determine offline with a bounded retry experiment: on duplicate bytes, wait
  briefly (e.g. 0.3 s) and re-capture up to 2 times with fresh probes, keeping the camera-held requirement; if bytes
  differ → accept with the attempt recorded; if still identical → fail as now. (Also consider overhead dwell: overhead
  frames are the most expensive; the client already waits two settled frames, so the capture path is the suspect.)
- **What it proves:** the fog guard, derived technology and viewport contract all work in a real Play; 20 genuine
  captures are obtainable on the sealed candidate; the fail-closed guard integrity holds.
- **What it does not prove:** no cell pass (view-021 onward missing → no runtime/validator/look verdict); the exact
  cause of the duplicate capture is not identifiable from this run alone (the duplicate bytes were not written by
  design); nothing about `atlas:pilot`/`highway-extension`/fog-off batches beyond the shared code paths.

## 4 · Proposed named fix (NOT implemented — for the Lead's ruling, next separate freeze)

Runner/fixture-only "bounded fresh-capture retry" for the atlas fixture (`tools/vinegar_run.py` Atlas.run): on a
duplicate-bytes capture, record the attempt, wait a short bounded time (0.3 s), re-probe (camera must still match the
ack within tolerance) and re-capture; allow at most 2 retries; keep the stale rule as the final fail-closed outcome and
keep every attempt in the evidence (`captureAttempts`). Offline tests: fake capture sequence duplicate→fresh (accepted,
2 attempts recorded), always-duplicate (fail, attempts recorded), camera moved between attempts (fail closed). This is
the same shape as Plans 2/3/4: bounded, evidence-preserving, assertion unchanged. Play budget: the fresh opening is at
**1/3**; the next atlas Play after that fix would take it to 2/3, leaving the third reserved (Play B abort-contact is
still gated behind fixture A+C).
