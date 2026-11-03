# TEAM D (re-homed to Claude) — ITEM 1: abort-contact, geometry fix D built and frozen; the single Play was NOT taken because the cell cannot score; RULING NEEDED (2026-11-02)

## 0 · What this report corrects, in one paragraph
The order read as if B (the ego's physics-consistent authority) was still to be built. It was already built and frozen (`b73f6cd`) and
already Played once (Play 21, fresh opening, 1/3: authority proven, gap opened 72 -> 168, 12 s guard fired). Nox's 17:22 ET ruling
(Option 1: fix the leg-start / target-side geometry, fixture only, window / 40 cap / closing clamp / both guards untouched) is the
live order, and Nox accepted this correction. Everything below is Option 1.

## 1 · What was built (offline, no Play, no Studio, sealed place untouched)
Branch `prep/b1-capture-format-20261101`, two commits on top of `b73f6cd`:
- `174ba2c` **fix D**. `tests/traffic/Server.luau`: after `abortObservedAt` the obstacle holds the TARGET lane at 8 studs/s until the
  proxy is back in its source lane with no manoeuvre (or 10 s), then the leg starts (`contactDriveAt` strictly after the abort); the
  chase direction points toward the proxy (`Server.contactDirection`: against the lane while the obstacle is ahead, along it once the
  proxy has passed). Assist `LineDirection = along`, `MaxForce = mass*sqrt(40^2 - lateral^2)`, the VectorForce keeps ONLY its lateral
  part (Play 21 lost lateral control: the obstacle drifted 28 studs off the lane). `contactLeg` also records `egoS`, `signedGap`,
  `direction`. Ruled values untouched (12 s clause, >700 radius, `touches>0` exemption, `clamp(proxy+8,12,60)`, 40 cap, -38/40, lateral term).
- `cbc9c02` pins the checker's finding (section 3) against the production scorer, plus the checker's MED/LOW findings.
Why Play 21 failed, from its own samples: the obstacle was 72-76 studs AHEAD of the proxy the whole time; a forward chase at proxy+8
(60 vs 52) opens the gap at 7.9 studs/s; leaving the target lane let the proxy commit it (rows with lane == target, which the host
validator rejects anyway). The earlier "target behind the probe / sign" reading is the same fact.

## 2 · Evidence for fix D
- Offline judge: `tests/unit/traffic-contact-pursuit.spec.luau` runs the real `TrafficSim` proxy and the fixture helpers EXTRACTED from
  the source. It reproduces Play 21 (abort seen, lane committed, no contact, a named guard fires) and shows fix D: abort genuine, target lane
  never committed, leg starts 5.3 s after the abort, box contact 4.7 s into the 12 s window. Wait speeds 0-16 all work; 20+ lets the
  proxy commit. (Lune's `lookAt` faces the opposite way to Roblox's; the spec re-orients the frames and asserts it, found by checking the first run against Play 21.)
- Suites: full Lune 929 passed / 0 failed (127 specs; 13 min 26 s, finished cleanly); Python 336 OK; `test_vinegar_run` 91 OK; sealed
  `build/test.rbxlx` c6651793... and `build/release.rbxlx` unchanged. Mutants caught: direction sign, leg-at-abort, wait speed 30.

## 3 · The fresh checker: FAIL, 1 HIGH — and it is right (I re-verified it)
Independent checker on `174ba2c`: `Events.contactWithTargets` (the production scorer the cell must fire; `Events.luau:71,1011,1040`) needs
**>= 12 studs/s of relative closing** (`CONTACT_CLOSING_SPEED`). After the abort resolves, the proxy has passed the obstacle, so the
leg is a chase from behind at the ruled `clamp(proxy+8,12,60)` = **8 studs/s relative**. `Part.Touched` would fire but the scorer never
would, so the run would die at the 60 s bound or in the validator ("touch and scored contact clocks disagree"): the single Play wasted by
construction. Worse, Play 21's proxy was commanded at ~52 studs/s, where the ruled upper clamp 60 gives only 8 relative at ANY margin.
Measured with the real scorer in the what-if (kinematic, not engine):

| proxy speed | ruled `+8`, cap 60 | `+16`, cap 60 | `+16`, cap 72 |
|---|---|---|---|
| 45 | box contact, NOT scored (8) | scored (15) | scored (16) |
| 52 | box contact, NOT scored (8) | NOT scored (8) | scored (16) |

So the ruled clamp (written for a chase, run-6) can never score in this geometry; it also could not have scored in the original run-6 design
unless head-on. Other findings applied in `cbc9c02`: pinned the `egoS` glue line (a mutant that projected the PROXY position survived),
contact-during-leg assertion, stale runbook paragraphs marked superseded, stale test renamed.

## 4 · RULING NEEDED (Nox) — two ways, my recommendation first
**(a) RECOMMENDED — lift the closing clamp to `clamp(proxy+16,12,72)`** (two ruled values: `+8 -> +16`, `60 -> 72`; window, 40 cap and both guards stay).
Ready to apply, offline-tested, NOT merged: branch `prep/d1-closing-16-72-20261102` @ `956cf99` (one fixture line, pins, spec, runbook).
Suites on it: traffic Lune 99/0, traffic acceptance OK, runner 91 OK. Expected in the model: leg ~5 s after the abort, scored contact ~3 s into the window, relative 16.
**(b)** Keep every ruled value and make the contact head-on (leg started while the proxy is still returning, obstacle ahead, reversing toward it: relative 39-68
in the model). I tried time and proxy-speed triggers: it only works in a narrow timing band (best trigger 6/9 perturbed starts), so I do not recommend spending the one Play on it.
**(c)** Option 2 of the earlier ruling (accept the cell's honest limit) is still open if neither is wanted.

## 5 · State and fences
No Play taken (opening 1/3, pid 3953752 unchanged, Edit); nothing published, merged, uploaded or pushed; `tools/usg` never run; no Studio call; no Cayleb-personal
session touched. After a ruling: merge the chosen change, fresh-checker delta again (a checker that reads the production scorer), `studiocycle cycle`, ONE `traffic:abort-contact` Play, judged in the ruled order
(gap direction and achieved acceleration first, scored contact second, guards third).

## 6 · What I learned about the harness (for Harness²)
A checker that runs the PRODUCTION predicate on the what-if's last frame found what my own box-overlap test could not. An offline what-if must call the real scorer, not a lookalike.
