# TEAM D — DIAGNOSTIC PLAY 9 REPORT: traffic:abort-contact (fixture A+C) — pursuit closes but the ruled 12 s window is too short (2026-11-02)

Lead GO: 2026-11-02T05:55Z (fresh opening prepared for Play B; fixture A+C frozen `1e8b5b7`, delta ACCEPTED). Sequence:
read-only state + sealed BEFORE → `preflight` → **one** `run traffic:abort-contact` → `verify-clean` → sealed AFTER.
No extra Plays, no build/publication/credential, no image reads. Spend $0.
Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T055035Z-traffic-abort-contact`
Log: `board/probes/team-d/diagnostic-play9-20261102.log`.

## 1 · Verdict in one paragraph

The ruled **A+C fixture behaves exactly as designed, and the cell fails honestly**: the closed-loop pursuit engaged the
proxy's current lane position (contact leg started at `contactDriveAt 9.001 s`), ego accelerated from ~9 to **59.06
studs/s** against the proxy's **52.17 studs/s**, and the distance **closed from a peak 121.5 studs (t=14.33) back to
79.6 studs at the window end** (~6.7 studs/s closing, still shrinking). But the ruled **12 s no-touch timer fired at
t≈21.01 s** (`HandlingServer:215: contact leg never reached within 12 s of contactDriveAt`) before the gap could be
closed, so **touches=0, contacts=0, cell FAIL**. The **observation-radius guard did not trip** (peak 121.5 << 700) and
the **generation guard stayed stable** (generation 2→2, 0 events, 1252 steps — the Play-1 generation-retirement churn did
not repeat). **No crash**, sealed artifacts unchanged, `verify-clean` clean, plays **1/3** in the fresh opening
(pid 881911). Root: the pursuit inherits the shared law's **+10 studs/s² longitudinal acceleration clamp**, so the ramp
from ~9 to ~60 studs/s costs ~8 s; the proxy pulls ~120 studs ahead before the chase reaches speed, and the 12 s window
then expires while the gap is still ~80.

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Pre-state | Studio pid **881911**, window `0x558c7f892380`, 2200×1000 floating, **Edit**, plays **0/3** (fresh opening); place == build == `c6651793…`; `preflight` rc **0**, refusals `[]`, host quiet |
| Sealed BEFORE | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` |
| Run | started 05:50:35Z, ended 05:51:16Z (~41 s wall; runtime `elapsed` **21.0147 s**); states `LOCKED 0.0 → PREFLIGHT 0.0 → STAGED 3.718 → PREPARED 3.726 → PLAYING 5.535 → RAN 8.834`; exit **1** |
| Failure reason | `HandlingServer:215: contact leg never reached within 12 s of contactDriveAt` (stack `HandlingServer:215` ← `HandlingServer:174`); runtime `state:"failed"` |
| Cell verdict | **FAIL** (touches **0**, contacts **0**); `maximumProgress 0.5356710547038487`; `signalled:true`; `cutInAt 2.882 s`; `abortObservedAt 4.982 s`; `contactDriveAt 9.0014 s`; `contactObservedAt` absent |
| `contactLeg` (latched at the failure) | `{t:20.998, distance:79.518, proxyS:3465.644, targetS:3473.469, proxySpeed:52.166, speed:60, lead:0.15}` |
| Pursuit profile (161 heartbeat samples; distance = |egoFrame − proxy frame|) | 0.12 s **75.6** → 4.33 s **46.4** → 8.47 s **12.5** (pre-contact-leg closest) → 9.57 s 33.6 → **peak 121.5 at 14.33 s** → 16.98 s 107.4 → 19.03 s 93.2 → **20.98 s 79.6** (closing ~6.7 studs/s at window end) |
| Speeds | ego ramps 13.5 → 22.1 → 30.4 → 39.1 → 47.2 → 55.8 → **59.06 studs/s** (15.9 s onward); proxy steady **52.1–52.2 studs/s**; commanded ego speed = 60 (closing clamp) once the leg is active |
| Guards | **12 s no-touch timer: TRIPPED as designed** (named message, at the window end); **observation-radius guard: NOT tripped** (max 121.5 << 700); reached-contact exemption untested (no contact) |
| Generation guard | `generationStable:true`, `initialGeneration:2`, `finalGeneration:2`, `generationEvents:[]`, steps 1252, truncated false — **no retirement churn** (differs from Play 1) |
| Runtime cost/telemetry | 1252 timing rows, not truncated; physicsFPS 60.04, physicsThrottle 100; 2 awake parts; samples 161 |
| Chunks | `null` (actual for this run — the expected 6×256 KiB chunk transport did not trigger; report fitted the direct path) |
| Crash state | **no crash**: `newCrashDumps:null`, `gpuJournal:null`, abort `{stop:"ok", cleanup:"ok", verifyClean:"ok"}`, hostcheck clean |
| Sealed AFTER | identical to BEFORE (both artifacts) |
| Post-state | `verify-clean` rc 0, `clean:true`; Studio alive pid **881911**; plays **1/3** in the fresh opening |

## 3 · Judgement per batch 7 §6 (abort-contact)

- **Did the closed-loop pursuit reach contact inside the bounded window?** **No** — it was working (gap shrinking at
  ~6.7 studs/s at the end; extrapolation ~12 more seconds would touch) but the ruled 12 s window expired first.
- **Do the guards behave?** **Yes**: both named guards are wired and the correct one fired (12 s timer); the radius guard
  correctly did not fire (distance never near 700); the reached-contact exemption is not contradicted. The failure is
  named, bounded and evidence-preserving — no dishonest capture, no relaxed assertion.
- **What it proves:** fixture A (pursuit of the proxy's current lane position with `clamp(proxy+8,12,60)`, 40 studs/s²
  cap) works directionally and closes; guard C is correctly wired; the generation-guard instrument shows the car was
  **not retired** this time; sealed integrity and Studio lifecycle hold.
- **What it does not prove:** the cell does not pass; contact/touch was never reached; the pursuit's acceleration ramp is
  the limiting factor (the shared law's **+10 studs/s² forward clamp**, inherited by the pursuit helper).

## 4 · Options for the Nox ruling (filed separately, NOT implemented)

See `board/probes/team-d/abort-contact-window-options-20261102.md`: **(A)** extend the no-touch window (~20–25 s; ruled
guard value change, needs Nox); **(B)** adjust the fixture start geometry so contact is reachable inside the ruled 12 s
(fixture-only); **(C)** raise the closing clamp/top speed or the pursuit's longitudinal clamp up to the unchanged
40 studs/s² cap (ruled-value change, needs Nox). Recommendation in the note: **(C) first** (pursuit forward clamp
10 → 40, which is *inside* the already-ruled 40 studs/s² cap; estimated ramp 9→60 in ~1.3 s instead of ~8 s), then (A) if
still short, (B) only if the start geometry is deemed changeable. **Nothing implemented.**

## 5 · Next

Per the Lead's sequence: the final atlas Play runs in the current opening (see `team-d-diagnostic-play10.md`); no further
traffic Play is run until the Nox ruling on the options. Sealed unchanged; Studio idle after the Play; reserved Plays
reserved.
