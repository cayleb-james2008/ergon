from: delegator
to: nox
kind: **RULING NEEDED — abort-contact layer 8: proxy-phase variability** (option-B wiring ran exactly as ruled, but the leg-start gap varied 24→71 studs → 12 s guard fired honestly); options A / B-bounded / A+B-bounded, recommendation B-bounded; world-overhead ruling still pending from 060800Z
re: 060541Z (Play B = option B); Play 11 evidence
---

## PLAY 11 — OPTION-B WIRING EXACTLY AS RULED; THE VARIABLE IS THE PROXY PHASE (layer 8)
Fresh opening (studiocycle; sealed place unchanged through close+open; pid 905535, 0/3): one `traffic:abort-contact`, exit 1, no crash,
sealed BEFORE==AFTER, plays 1/3, verify-clean.
- **Option-B wiring verified in the run:** `fixture.preLegSpeed=30`; `abortObservedAt=3.7826` → `contactDriveAt=3.7826` (no 4 s wait);
  `-18` absent; maneuver writes 0; the ruled law/guards untouched (checker had confirmed byte-identity on `68c020b`).
- **But the leg-start gap was 71.4 studs, not the modelled 24.3:** this run's proxy phase differed from Play 9 (abort at 3.78 s with proxy
  ~27 studs/s vs Play 9's 4.98 s/~11 studs/s), so **a fixed pre-leg speed cannot bound the start gap — the variability is the proxy
  phase (layer 8)**.
- **Pursuit/guards behaved correctly:** ego ramped 30→59.06, proxy 52.17; distance 71.4 → peak 141.75 → **142.44 at the guard**;
  **12 s timer fired exactly as designed**, radius guard not tripped (142 ≪ 700); contacts/touches 0; generation stable; no fabricated
  contact. **No cell pass claimed.**
**RULING NEEDED — options (no implementation until you rule):**
- **(A) extend the no-touch window 12 → 20 s** — now evidence-backed: Play 11 needed ~14.5 s from its start; phase-robust across the
  observed range. (A was rejected earlier when the failure was start geometry; the evidence class has changed to phase variability.)
- **(B-bounded) fixture gap-trigger** — the contact leg starts only when the proxy is within ≤25 studs (bounded by a named pre-leg
  timeout that fails closed if the phase never converges) plus a smaller initial ego offset. Fixture-only; **keeps your ruled 12 s window
  and all ruled values untouched**; guaranteed-intended geometry or a named fail.
- **(A+B-bounded)** if both bounds are acceptable.
**Team D's implicit recommendation: B-bounded first** (fixture-only, preserves the ruled guard), with A as the fallback if the phase
cannot be bounded honestly. **Please rule.**

## STILL PENDING FROM 060800Z — ATLAS `world-overhead`
The final atlas Play (Play 10) isolated the limitation to **whole-map overhead framings (highway AND world)**; `city-overhead` captures
fresh; zone views 22–25 unexercised because `world-overhead` precedes them. **Ruling requested: extend the B substitution to
`world-overhead` (B2 per-region/world-context views + A2 named limitation with the same retest rule), optionally folding the plan
ordering so the zone views precede world-overhead.** Team D has an implementation-ready note (`design-b2-world-regions-20261102.md`);
nothing implemented.

## STATE
Layer-6 ACCEPT noted; option-B freeze `68c020b` checker-ACCEPTED (ruled law/guards byte-identical, new-start kinematics 24.31 studs/30
studs/s → 7.56 s, M23-M25 fail; suites 91/331/Lune 923). FL1 unchanged (owner key entry only, held). FL2 NOT met. Remaining Plays
reserved; Studio idle; no extra Plays/build/publication/image reads; $0. Wake-up refresh will fold in the latest before 08:00–09:30 ET.

— delegator
