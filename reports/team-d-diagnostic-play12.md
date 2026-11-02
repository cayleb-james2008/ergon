# TEAM D — DIAGNOSTIC PLAY 12 REPORT: B-bounded abort-contact — named phase fail (gap never ≤25), Studio clean (2026-11-02)

Lead GO: 2026-11-02T06:52Z (after B-bounded delta ACCEPT on `e6ae134`). Sequence: state + sealed BEFORE → `studiocycle
cycle` → fresh opening verified → `preflight` → **one** `run traffic:abort-contact` → `verify-clean` → sealed AFTER.
No extra Plays, no build/publication/credential, no image reads. Spend $0.
Run: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T065310Z-traffic-abort-contact`
Log: `board/probes/team-d/diagnostic-play12-20261102.log`.

## 1 · Verdict in one paragraph

**The B-bounded fixture fired its named bounded pre-leg fail exactly as designed, and the cell fails honestly.** The
proxy never came within the 25-stud trigger in this run's phase: at `abortObservedAt 3.762 s` the gap was **73.55 studs**
and it **grew to 123.97** over the 4 s pre-leg (the traffic car accelerated away/its lane-change phase differed), so at
`t=7.762 s` the fixture raised `contact leg phase never converged: proxy stayed 123.9 studs from the probe (needs <=25)
for 4.0 s after abortObservedAt` (`HandlingServer:216`). No contact leg → no `contactLegTrigger`, no `contactLeg`; the
12 s timer was never reached (the named pre-leg bound fired first), the radius guard was never engaged (gaps ≪ 700), no
crash, sealed artifacts unchanged, `verify-clean` clean, plays **1/3** in the fresh opening (pid 923305). Per the Nox
ruling, the named phase fail sends option **A (window 12 → 20 s)** back to Nox with this evidence — **no silent
extension**; C is **not** triggered (the leg never started, so the "starts inside the bounded gap and still misses"
predicate is false).

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Pre-state | Studio pid **905535** (old opening, 1/3) → `studiocycle cycle` 06:52:42Z: close `killed-after-dialog` (place sha unchanged, nothing saved), reopen **pid 923305**, window `0x558c7f831f20`, 2200×1000; sealed place before==after `c6651793…`; `preflight` rc **0**, refusals `[]`, **Edit, 0/3**, host quiet |
| Sealed BEFORE / AFTER | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` — unchanged |
| Run | 06:53:10Z → 06:53:31Z (~21 s); exit **1**; states `LOCKED → PREFLIGHT → STAGED 2.734 → PREPARED 2.741 → PLAYING 4.551 → RAN 7.118` |
| Failure reason | `HandlingServer:216: contact leg phase never converged: proxy stayed 123.9 studs from the probe (needs <=25) for 4.0 s after abortObservedAt` (stack `:216` ← `:179`); runtime `state:"failed"` |
| B-bounded wiring in the run | `cutInAt 2.762`; `abortObservedAt 3.7621`; `preLeg` latched `{t:7.7783, gap:123.93, contactGap:25, timeout:4}`; `contactLegTrigger:null`; `contactLeg:null`; fixture `contactGapTrigger:25`, `preLegTimeout:4`, `initialEgoOffset:70`, `preLegSpeed:30`; no maneuver writes (`initialBodyPlacements/VelocityWrites=2`, `maneuverPoseWrites/VelocityWrites=0`) |
| Gap series (60 samples) | gap **73.55** at abort (t=3.76) → **never below 73.55** in the pre-leg → **123.97** max at t≈7.68 → named fail at t=7.762 |
| Guards | **named bounded pre-leg fail fired** (contact leg never started); the ruled 12 s no-touch timer was never reached; the >700/800 radius guard was never engaged (max gap 124 << 700); reached-contact exemption untested |
| Contacts/touches | **0 / 0**; `maximumProgress 0.9948`; `completedAt 6.833` |
| Generation/diagnostics | stable 2→2, 0 events, 468 steps; chunks `null` (actual) |
| Crash | **none**: `newCrashDumps:null`, abort `{stop:"ok", cleanup:"ok", verifyClean:"ok"}`, hostcheck clean |
| Post-state | `verify-clean` rc 0, `clean:true`; Studio alive pid **923305**; plays **1/3** in the fresh opening |

## 3 · Judgement (batch 7 §6 abort-contact)

- **Did the gap trigger fire inside the bounded pre-leg?** **No** — the phase never converged (min gap 73.55 > 25) and
  the **named bounded fail** fired at the 4 s bound, which is the designed fail-closed behaviour for an unreachable phase.
- **Did contact occur inside the ruled window / does the runtime complete / does the cell pass?** **No** — no leg, no
  contact, runtime failed with the named message, **no cell pass**. The runtime completed its fail path and cleaned up.
- **What it proves:** the B-bounded trigger + named timeout + fixture audit fields work exactly as accepted (no
  silent extension, no fabricated leg start, no dishonest contact); the guards/law are untouched; sealed integrity and
  Studio lifecycle hold; the phase variability observed in Play 11 is real and can make the 25-stud trigger unreachable.
- **What it does not prove:** why the proxy phase was this far away (product traffic phase/timing — not a product defect
  implied); nothing about option A/C (untested); no cell pass.

## 4 · Ruling path (per Nox)

Per the run-9 ruling: **B-bounded fired its named phase fail ⇒ option A (extend the no-touch window 12 → 20 s) goes back
to Nox with this evidence; do not extend silently.** C is **not** triggered (the pre-approval predicate requires the leg
to start inside the bounded gap). The current fresh opening is at **1/3**; remaining Plays reserved; the atlas B2+A2
freeze is a separate offline item (no Play).
