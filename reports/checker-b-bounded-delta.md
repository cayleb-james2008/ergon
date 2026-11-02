# CHECKER — B-bounded abort-contact delta (`e6ae134`) — verdict: **ACCEPT**

Fresh-context delta re-verify of the Nox run-9 B-bounded start geometry on top of my accepted option-B delta (`68c020b`).
I built none of it. No Play, no Studio input, no image reads, no host input, **$0**. Ran from a throwaway clone
`/tmp/checker-bb/repo` (2026-11-02T01:42:19-05:00) and a second clone for mutations; maker tree only read.

- **Subject tip:** `e6ae1344fd945ff9f9e7ee1d371ad1fc346919ed` (`e6ae134`), parent `68c020b`, tree
  `1c0bb6b8505e374f9e222ab7bb0a035bf09554c7`; tracked mods **0**; sealed `84324d43…`/`c6651793…` unchanged
  before/after and at end.
- **Delta over `68c020b`:** exactly **3 files, +85/−61** — `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` (+8/−7),
  `tests/tools/test_traffic_acceptance.py` (+61/−51), `tests/traffic/Server.luau` (+16/−3). No `src/`, `assets/`,
  `build/` change; no atlas/B2 file touched; `tools/vinegar_run.py`, `tests/atlas/*`, `tools/atlas_acceptance.py`,
  `tools/map_atlas.py` untouched → settle/retry/audit/45 s/assert-and-skip intact.
- **Maker report:** `board/reports/team-d-b-bounded-freeze-20261102.md` is **absent**; verified from the commit.

## Verdict in one line

**ACCEPT** — fixture-only: the ruled 12 s no-touch window, closing clamp `clamp(proxy+8,12,60)`, 40 studs/s² cap, both
guards and the reached-contact exemption are **byte-unchanged** (function-block sha256 identical at parent and tip),
while B-bounded adds `CONTACT_GAP=25`, a named bounded `PRELEG_TIMEOUT=4` pre-leg after `abortObservedAt` with the
named fail-closed phase message, the smaller initial ego offset (80→70) and a gap-triggered, latched contact start;
my independent kinematics from the Play-11 one-run profile reproduce the claim (gap 66.62 at abort → trigger at 24.87
studs 2.60 s after abort ≤4 s; contact 7.63 s <12 s with the rolling speed vs 17.65 s at the old 10 studs/s), and the
three scenario mutations each fail the pin.

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① Fixture-only; ruled values byte-unchanged | **PASS** | Delta paths are docs/tests only (`src/` empty). `Server.pursuitAcceleration` block sha256 `627e580d95c5…` and `Server.contactLegGuard` block sha256 `d372efb92290…` are **identical at `68c020b` and `e6ae134`**; `sinceContactDrive>=12`, `>700 of 800 studs`, `closing=math.clamp(proxySpeed+8,12,60)`, both `if acc.Magnitude>40 then acc=acc.Unit*40 end` caps and `touches<=0` exemption are present unchanged. The contact-trigger hunk replaces the fixed `abortAt` trigger with the bounded gap trigger; the fixture record only gains `contactGapTrigger=25,preLegTimeout=4,initialEgoOffset=70`; ego spawn `chosen+80`→`chosen+70`; the no-maneuver write contract (`initialBodyPlacements/initialVelocityWrites` writes, zero maneuver writes) is unchanged |
| ② B-bounded implementation | **PASS** | `tests/traffic/Server.luau:19-20`: `CONTACT_GAP=25`, `PRELEG_TIMEOUT=4`; `:76` fixture record; `:145` `Sim.frame(source.poly,chosen+70)`; `:205-218`: per-heartbeat `report.preLeg={t,gap,contactGap,timeout}`, `if probeGap<=CONTACT_GAP then contactAt=age; … report.contactLegTrigger={t=age,gap=probeGap}`, `elseif age-abortAt>=PRELEG_TIMEOUT then error("contact leg phase never converged: proxy stayed %.1f studs from the probe (needs <=%d) for %.1f s after abortObservedAt", …)`. Independent numbers (Play-11 profile, `trigger-numbers.txt`): gap at abort **66.62** (>25 → trigger needed), trigger **2.60 s** after abort (≤4 s), gap at trigger **24.87** (ruled 12–25), rolling speed 30.0, contact **7.63 s** (<12) vs **17.65 s** at the old 10 studs/s (>12); Play-9 phase gap at its abort 18.67, |Δ| 47.94 (>15) documents one-run phase sensitivity |
| ③ Tests + mutations | **PASS** | `test_b_bounded_leg_starts_on_the_gap_trigger_with_a_named_bounded_pre_leg` pins `CONTACT_GAP=25`, `PRELEG_TIMEOUT=4`, `chosen+70`, the fixture record (`preLegSpeed=30,contactGapTrigger=25,preLegTimeout=4,initialEgoOffset=70`), the rolling-speed conditional, `if probeGap<=CONTACT_GAP then`, `report.contactLegTrigger`, the `elseif age-abortAt>=PRELEG_TIMEOUT` branch + exact named message, the absence of the old fixed wait and `-18`, and the unchanged closing clamp / guards / 12 s text. `test_b_bounded_trigger_fires_inside_the_timeout_and_contact_fits_the_ruled_window` uses the Play-11 one-run profile (trigger inside 4 s, gap 12–25, contact <12 vs old 10 >12) and explicitly documents the Play-9 phase difference. Mutations in a throwaway clone, restored: **M26** initial offset back to 80 → **FAILED (1)**; **M27** named pre-leg timeout removed → **FAILED (1)**; **M28** gap trigger made unconditional → **FAILED (1)** (all on the scenario-pin test). Restored `tests/traffic/Server.luau` sha256 `e7839c737214…` (frozen), git clean |
| ④ Scope / regressions | **PASS** | Exactly the 3 claimed files; no game/asset/sealed path; no atlas/B2 file mixed in; atlas/settle/retry/audit/45 s untouched |
| ⑤ Suites (fresh clone) | **PASS** | runner `Ran 91 tests … OK` (12.520 s); traffic acceptance `Ran 9 … OK` (0.079 s); full Python `Ran 331 tests … OK` rc 0 (47.622 s); Lune traffic filter `93 passed, 0 failed (18 specs)` — covers all three Lune specs reading the changed file (`traffic-contact-pursuit`, `traffic-runtime`, `traffic-timing`). **Full Lune not re-run** (12 min; tests/docs/fixture-only delta) — relied on the accepted full-Lune `923/0 (127 specs)` at the parent |
| Host / no-Play | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed unchanged; maker HEAD `e6ae134`, tracked clean at end. The abort-contact Play is the live proof |

## Residuals (named, non-blocking)

- **R1 — live limit:** no Play; the gap trigger/timeout kinematics come from the Play-11 one-run proxy profile and are
  explicitly not a universal acceptance claim (the test says so); the abort-contact Play is the live proof.
- **R2 — maker report absent:** `team-d-b-bounded-freeze-20261102.md` is not present; verified from the commit.
- **R3 — phase sensitivity:** the proxy profile is one-run data (Play 9 vs Play 11 differ by ~48 studs at abort); the
  test documents this, so a future phase change may need a new profile — no hidden universal claim.
- **R4 — observation:** `report.preLeg` is overwritten each heartbeat (latest t/gap only), not a history; adequate for
  the named timeout evidence, but not a trace series.
- **R5 — cosmetic:** untracked evidence-dir count now 815 vs the "792 pre-existing" wording used in earlier reports.
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-b-bounded-delta/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + paths + sealed +
no-atlas-mix + function-block shas at parent/tip + ruled values + B-bounded constants/trigger/timeout/offset code +
numbers + maker-report absence), `trigger-numbers.txt` (gap 66.62 at abort / trigger 2.60 s / gap 24.87 / contact 7.63 s
vs 17.65 s / Play-9 phase Δ47.94), `suites.log`, `runner.log` (91 OK), `traffic.log` (9 OK), `python.log` (331 OK),
`lune-traffic.log` (93/0, 18 specs), `mutations.log` (M26/M27/M28 each 1 failure; restored hash), `run-started.txt`.
Throwaway copies `/tmp/checker-bb/{repo,mut}`; maker repo read-only, unchanged, still at `e6ae134`.
