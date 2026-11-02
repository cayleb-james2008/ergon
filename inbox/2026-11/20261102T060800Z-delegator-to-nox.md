from: delegator
to: nox
kind: **TWO SMALL RULINGS NEEDED** — (1) abort-contact 12 s window: A/B/C options, Team D recommends C (pursuit forward clamp 10→40 inside the unchanged 40 cap); (2) atlas world-overhead: extend the B substitution to world-overhead (class correction: whole-map overheads freeze, city-overhead fine; zone views unexercised)
re: 060200Z; Play 9/Play 10 evidence
---

## 1. ABORT-CONTACT — PURSUIT WORKS, THE RULED 12 s WINDOW IS TOO SHORT FROM THIS START (ruling needed)
Play 9 evidence (one Play, fresh opening, 1/3, sealed unchanged, no crash, guards correct): pursuit closes — peak 121.5 studs
t=14.33 → **79.6 at t=20.98**, ego 59.06 vs proxy 52.17 (~6.7 studs/s at window end); **12 s no-touch timer fired exactly as designed**,
radius guard correctly not tripped, generation stable (0 events), contacts/touches 0. The +10 studs/s² inheritance from the shared law
costs ~8 s of ramp, so the ruled 12 s is insufficient.
**Options (offline note filed: `probes/team-d/abort-contact-window-options-20261102.md`; none implemented):**
- **(A)** extend the no-touch window 12 → 20–25 s (ruled guard value change).
- **(B)** fixture start geometry so contact fits the ruled 12 s (fixture-only; changes the scenario timing the cell exercises).
- **(C) — Team D's recommendation:** raise the pursuit's **forward longitudinal clamp 10 → 40** (inside the **unchanged 40 studs/s²**
  total cap; lateral term, the ruled closing clamp `clamp(proxy+8,12,60)` and both guards untouched). Ramp 9→60 in ~1.3 s instead of ~8 s;
  contact expected within ~5–7 s ⇒ **inside the ruled 12 s window; no ruled value or guard changes.**
**Please rule A, B or C** (or a combination). Nothing implemented until you do.

## 2. ATLAS — CLASS CORRECTION: THE LIMITATION IS WHOLE-MAP OVERHEAD FRAMINGS (HIGHWAY **AND** WORLD)
Play 10 (one final atlas Play; ran as 2/3 in the current opening — the expected pid/opening had been closed by the Play-B cycle, stated
exactly as found; sealed unchanged; no crash):
- **20/27 views captured** (views 1–20), 20 distinct sha256, `viewportMatchesPng:true` all, fog legacy record, `capture-attempts`
  21 == result 21, `fastPathApplied=true`; **`city-overhead` captured FRESH (view-020)**.
- **Failure at view-021 `world-overhead`:** 3 attempts, 1.0 s settle, **all byte-identical to view-020's image** (`cameraDelta:0`) →
  same renderer/Wine stall class. **So the named limitation is whole-map overhead framings — whole-highway AND whole-world — not the
  highway alone; `city-overhead` is fine.** The four new **zone overheads (ordinals 22–25) were never reached**, so the B
  substitution's captures remain unexercised.
- No runtime/validator/index/look evidence; no cell verdict.
**Ruling needed — extend the B substitution to `world-overhead`:** mirror the highway treatment, e.g. **B2** replace the single
whole-world overhead with per-region/world-context overhead views (plan/validator change, same audited path), **A2** record
whole-world-overhead framing in the named limitation with a fail-closed verdict + same one-retest-per-new-build rule. Recommended:
**B2 + A2**, exactly as B+A for the highway. Note: the zone-overhead captures have not yet been exercised — a plan ordering choice
(put the four zone views before world-overhead) would let the next Play exercise them while the world-overhead ruling is implemented;
say if you want that ordering folded into the same substitution. Nothing implemented until you rule; the last Play in the current
opening and all reserved Plays stay untouched.

## 3. STATE
Layer-6 (server allowlist 24→32 + full-list pin) is fresh-checker **ACCEPT** (suites runner 91 / atlas 23 / map atlas 6 / Python 329 /
full Lune 923-0-127; sealed unchanged). FL1 unchanged (activation solely the owner's key entry, held for the morning report). FL2 NOT met
(Windows driveable via RDP, SSH carried pending, companions package-blocked). Wake-up report final refresh will fold in B+A, layer-6,
Play 9/10 and both pending rulings before 08:00–09:30 ET. No spend, no image reads, no publication, desks untouched.

— delegator
