# DESIGN NOTE — OPTION C (abort-contact pursuit forward clamp 10 → 40) — implementation-ready, NOT implemented

Status: awaiting the Nox ruling (Lead filed C as the recommendation). Offline only; **no Play, no code change yet**.
Fixture-only: **no production file is touched**. Evidence base: `board/reports/team-d-diagnostic-play9.md`,
`board/probes/team-d/abort-contact-window-options-20261102.md`, run `20261102T055035Z-traffic-abort-contact`.

## 1 · The one-line change

`tests/traffic/Server.luau` → `Server.pursuitAcceleration` (the pursuit-only helper added by fixture A+C):

```lua
-- now
local acc=forward*math.clamp((speed-velocity:Dot(forward))*2,-38,10)+right*math.clamp((lane.Position-position):Dot(right)*1.5-velocity:Dot(right)*2,-12,12)
if acc.Magnitude>40 then acc=acc.Unit*40 end
-- after (only the forward upper clamp changes; lateral term and force cap untouched)
local acc=forward*math.clamp((speed-velocity:Dot(forward))*2,-38,40)+right*math.clamp((lane.Position-position):Dot(right)*1.5-velocity:Dot(right)*2,-12,12)
if acc.Magnitude>40 then acc=acc.Unit*40 end
```

`Server.obstacleAcceleration` (the shared obstacle law) **keeps its `10`** — unchanged. The ruled closing speed
`clamp(proxySpeed+8,12,60)`, the 40 studs/s² total cap, the 0.15 s lead and both guards are unchanged.

## 2 · Why it is estimated to close inside the ruled 12 s (model cross-checked with Play 9)

Kinematic model, v0 = 9 studs/s (ego entering the contact leg), target 60, proxy 52.17, initial gap 23.7 (Play-9 8.47 s
sample):
- clamp **40**: reach 60 in **1.28 s**; gap at speed ≈ **46** studs; then close at 8 studs/s → **~7 s total**.
- clamp **10** (current): reach 60 in **5.1 s**; gap at speed ≈ **113**; then **~19 s total** → the observed Play-9
  failure (`>12 s`, peak 121.5, 79.6 at window end, ~6.7 studs/s closing).
So C targets exactly the limiting factor and stays inside the already-ruled 40 cap; no guard/ruled value changes.

## 3 · Files touched (fixture/test only)

| File | Change |
|---|---|
| `tests/traffic/Server.luau` | one clamp value in `Server.pursuitAcceleration` (`10`→`40`); nothing else |
| `tests/unit/traffic-contact-pursuit.spec.luau` | new assertions/tests (below) |
| `tests/tools/test_traffic_acceptance.py` | source-shape additions + a pure-Python kinematic regression (below) |
| `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` | one line in the fixture section: pursuit forward clamp uses the ruled 40 cap (obstacle law unchanged) |
| `board/probes/team-d/abort-contact-window-options-20261102.md` | status line: option C implemented (if ruled) |

## 4 · Exact tests to add

1. **Lune spec** (extend `traffic-contact-pursuit.spec.luau`): straight-line v=0, speed=60 → `|acc| == 40` (±1e-3)
   (with the old clamp it is 10 — the mutation catches this); lateral offset 500 with v=0 → `|acc| <= 40.001` (total cap);
   the pursuit's right/lateral term text is identical to `Server.obstacleAcceleration`'s (unchanged), and the force-cap
   line still appears in both helpers (count 2).
2. **Python source-shape** (`test_traffic_acceptance.py`): pursuit helper contains
   `math.clamp((speed-velocity:Dot(forward))*2,-38,40)`; obstacle helper still contains `...,-38,10)`; `acc.Magnitude>40`
   cap count == 2; closing clamp and guards texts unchanged.
3. **Pure-Python kinematic regression** (no Play): integrate the two clamps from (v0=9, gap=23.7, proxy=52, target=60,
   dt=1/60); assert clamp 40 touches in <12 s (≈7 s) and clamp 10 does not (≈19 s) — the same bounded numbers as §2.
4. **Mutation proof M22**: revert the pursuit clamp to `10` → the Lune spec (item 1) and the Python kinematic/source tests
   (items 2–3) fail; restored, tracked clean.

## 5 · Freeze → checker → Play evidence chain

Suites on the fix tree: runner 91, traffic acceptance 7+, Python suite (current 329 + new test(s)), Lune traffic filter +
new pursuit spec (3+ tests), full Lune re-run; sealed unchanged (`84324d43…`/`c6651793…`); commit + freeze block.
Play (fresh opening, Lead sequences): one `run traffic:abort-contact`; expected **contact inside 12 s** — judge from
`contactLeg` (per-heartbeat latest row is in the report; the full **series** is derived from the bounded `samples`:
distance = |egoFrame − proxy frame|, ego/proxy speeds from the velocity vectors), ramp time to ~60 (~1.3 s modeled),
guards (radius guard not tripped; the 12 s timer must **not** trip because touch occurs first), generation stable, sealed
before/after, plays counter. If contact is still missed, the evidence decides between options A (20–25 s window) and B
(start geometry) — no silent extension.

## 6 · Risks / non-changes

No production file; no ruled value changes (closing clamp, 40 cap, lead, guards all as ruled); lateral term and the shared
obstacle law unchanged; the fixture's acceleration cap can now use the full ruled 40 (fixture honesty note kept in the
report). Play budget: the Play after the delta needs the Lead's sequencing; the current opening's last Play and all
reserved Plays stay untouched.

## 8 · Batch record

Written 2026-11-02T06:10Z as the offline implementation-ready note; repo tip stays `8bda598` (tracked tree clean),
no code/plan change, no Play, Studio untouched (pid 881911, opening 2/3, last Play reserved).
