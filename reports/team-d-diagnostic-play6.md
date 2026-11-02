# TEAM D — DIAGNOSTIC PLAY 6 REPORT: capture-retry Play — persistent duplicate at view-021 + Studio crash (2026-11-02)

Lead GO: 2026-11-02T04:22Z (current opening, no cycle needed). Machine `outis-desktop`. Sequence: read-only state +
sealed hashes BEFORE → `preflight` → **one** `run atlas:scout --seed 2134783819` → `verify-clean` → sealed AFTER →
Studio crash recovery. **No extra Plays, no retries beyond the fixture's own bounded retry, no build/publication/
credential, no image reads.** Spend $0.
Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T042225Z-atlas-scout`
Log: `board/probes/team-d/diagnostic-play6-20261102.log`.

## 1 · Verdict in one paragraph

The bounded retry **worked exactly as designed and proved the duplicate is persistent, not transient**: at **view-021
`highway-overhead`** all **three** capture attempts (0.3 s apart) returned bytes **byte-identical to view-020
(`city-overhead`)** — i.e. the capture kept returning the *previous* overhead frame. The runner therefore failed
closed with the named **`… on all 3 attempts (stale capture; attempts recorded)`** (exit **5** because Studio then
**crashed** during/after the Play stop: new **106 MB minidump `2742f914-…dmp` at 04:22:48Z**, `abort.cleanup`/
`verifyClean` = MCP disconnected, `newCrashDumps:1`; no host GPU-journal lines, hostcheck clean). 20/24 views were
captured again (20 distinct sha256, all `viewportMatchesPng:true`); the full 21-entry `captureAttempts` audit trail is in
`result.json`. **Layer 5 = overhead-view render/capture stall (previous frame persists >0.6 s) and a Wine Studio crash
on Play shutdown** — a new named layer, not a retry-logic defect. Studio was recovered with `studiocycle open`
(sealed-hash guarded, place unchanged); fresh opening is **0/3 Plays**; sealed artifacts unchanged throughout.

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Pre-state | Studio pid **797879**, window `0x558c7fa15fe0`, 2200×1000 floating, **Edit**, plays **1/3**; place == build == `c6651793…`; `preflight` rc **0**, refusals `[]`, host quiet |
| Sealed BEFORE | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` |
| Run | started 04:22:25Z, ended 04:25:54Z (~209 s); states `LOCKED 0.0 → PREFLIGHT 0.0 → STAGED 1.791 → PREPARED 1.898 → PLAYING 3.791 → RAN 6.805`; exit **5** |
| Error | `FixtureFailed: view-021: screen_capture returned the same bytes as an earlier view on all 3 attempts (stale capture; attempts recorded)` |
| Captures | **20 sidecars** (001–020), 24 planned; 20 **distinct** sha256; `viewportMatchesPng:true` on all; `accepted:true`; `stationary`; JPEG; 1837×744 |
| `captureAttempts` (result.json, 21 entries) | views 001–020: one attempt each, `duplicate:false`; **view-021: 3 attempts, each `sha256=0ac9595d3372a880…` = view-020 `city-overhead`'s sha, `duplicate:true`** — the previous frame persisted across both 0.3 s waits |
| Collector | phases `prepared` + `view-001 … view-021` (21 acks), rejections `[]`; chunks null |
| `worldTrace` (`result.json` + `world-trace.json`) | `fastPathApplied=true`, `actualSeed=2134783819`, `actualLayout=2` == expected, `rebuild=false`, starter applied before `rootFoundAt` (world build ≈4.1 s) |
| Crash | `newCrashDumps:1`; minidump `2742f914-5057-4949-8115-e1c5c1074fd4.dmp` **106,116,768 B @ 2026-11-02T04:22:48.583Z**; Studio log ends 04:22:47.58Z right after `StopPlaySoloShutdownDataModels` starts; `abort.stop:"ok"`, `abort.cleanup:"Disconnected('no response to tools/call within 180s')"`, `verifyClean` likewise; `gpuJournal:null`; kernel journal grep (amdgpu/MES/ring/GPU reset/drm) since 04:22 — none; hostcheck `{stalls 0, label "clean"}` |
| Sealed AFTER | identical to BEFORE (both artifacts, before and after the crash/recovery) |
| Recovery | `studiocycle open` → sealed place sha before==after `c6651793…`; new **pid 814951**, window `0x558c7f831f20`, 2200×1000 floating; `verify-clean --clear-latch` on the reopened sealed place; `preflight` rc 0, **Edit**, **plays 0/3** (fresh process; the crashed opening had consumed 2/3), place == build == `c6651793…`, refusals `[]` |
| Residual nit | `capture-attempts.json` is not written on the failure path (the audit is in `result.json`); fold into the next offline change. |

## 3 · Judgement per batch 7 §6 (atlas:scout)

- **Retry logic: correct.** Three bounded attempts, per-attempt sha + camera delta, final named fail-closed, audit trail
  in `result.json`; no loop beyond the bound; no dishonest capture stored.
- **New named layer 5: overhead-view render/capture stall.** The duplicate is the **previous overhead view's exact
  bytes**, three times, 0.6 s apart — the capture path kept returning the last rendered frame instead of the new
  highway-overhead frame. The client had acknowledged view 21 (collector `view-021` ack), so the pose was set; the
  render/capture did not catch up. This is the first overhead→overhead transition after the (also overhead)
  `city-overhead` view; the highway-overhead fit renders the whole highway, the heaviest frame of the batch.
- **Crash:** Studio died during/after the fail-closed abort's Play stop (minidump, MCP gone; no kernel GPU fault; host
  quiet). Crash class is honest (`exit 5`, `newCrashDumps:1`), not a product failure; it prevented cleanup/verify-clean
  in that process, which the reopened sealed place + `verify-clean --clear-latch` resolved.
- **What it proves:** the retry safety net works and produces auditable evidence; 20/24 captures are repeatable;
  `fastPathApplied=true` again; sealed integrity held through a crash.
- **What it does not prove:** views 21–24 and the `runtime` phase were never captured, so there is **no validator
  verdict, no capture index/coverage and no look/B3 evidence**; the cell does not pass. The exact cause of the render
  stall / shutdown crash is not established (userland minidump only; no kernel GPU evidence; not reproducible on demand
  within one Play).

## 4 · Proposed named fix (NOT implemented — for the Lead's ruling; next separate freeze)

Layer-5 candidate: **overhead-specific bounded render settle + audit** (runner/fixture only):
- after the ack for a view whose client plan marks it overhead/`tile`/`cell-roof`, wait a bounded settle (e.g. **1.0 s +
  re-probe**) before the first capture, and use a **longer retry wait for overhead kinds (e.g. 1.0 s, still max 2
  retries)**, recording `kind` + `settledSeconds` in `captureAttempts`;
- write `capture-attempts.json` also on the **failure path** (the audit currently lives only in `result.json`);
- offline tests: overhead kind takes the settle path (sleep recorded), non-overhead does not; a still-duplicate capture
  still fails closed after the bound; the failure-path audit file is written.
If a Play shows the frame still frozen after the longer settle, the honest conclusion is a **renderer/Wine limitation
for whole-highway overhead frames** and the options become a different overhead capture strategy (e.g. a smaller
functional tile-like view per zone, which is a plan/validator change) — a Lead/Nox ruling, not a silent change.
Play budget: the crashed opening is gone; the recovered opening is at **0/3**. Next atlas Play after the layer-5 fix
would be Play 1 of it; Play B (abort-contact, fixture A+C) remains gated behind its own freeze; the reserved third Play
of the Lead's plan stays reserved.
