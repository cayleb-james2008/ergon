# TEAM D — FIXTURE A+C FREEZE: abort-contact pursuit + named guards (Nox run-6 ruling), 2026-11-02

Nox run-6 ruling (20261102T033600Z, item 3): implement **A** (closed-loop pursuit for the contact leg — target the
proxy's current lane position, closing speed `clamp(proxy+8,12,60)`, keep the existing 40 studs/s² cap) + **C** (both
named fail-fast guards: >700-of-800 observation-radius guard and the 12 s no-touch timer). **B** only as fallback with
exact bounded values if A demonstrably cannot close; **D** rejected. Fixture/test files only, no production code.
**Offline only — no Play.** Machine `outis-desktop`, spend $0.

**Freeze commit: `1e8b5b774832bbaac2de9bbb0487fbd3baba5ae7`** (`1e8b5b7`; parent `fc546e5` = accepted layer-5 delta fix;
branch tip; tree `4d6dcdb47682581528f9f0e3ccbe0ae198d9c4e3`; tracked tree clean; **3 files +105/−14**). Sealed artifacts
unchanged (`84324d43…` release, `c6651793…` test/place). Log: `board/probes/team-d/fixture-ac-freeze-20261102.log`.

## 1 · Why (Play 1 attribution)

The 2026-11-02T02:29:40Z Play failed the abort-contact cell with one bounded `generationEvent` (step 1386, generation
2→3, `bodyReplaced`, `bodyAlive` false, `hasChange:false`, contacts/touches 0) — and the probe's own distance series
explained it: after the abort (4.983 s) the contact drive drove the **sole observer away** from the proxy (23.9 studs at
`contactDriveAt 9.003 s` → **>800 studs (`RETIRE_RADIUS`) at t=22.283 s**), so `TrafficController`'s documented
`unobservedSince > 1` rule retired the car 0.95 s later. Not contact-correlated churn, not a product defect — a fixture
geometry defect (the cell's contact leg was unreachable). Nox ruled A+C.

## 2 · What changed (fixture only — `tests/traffic/Server.luau`)

**A — closed-loop pursuit.** During the contact leg the fixture now chases the proxy's **current lane position**:
`proxyS = Sim.project(source.poly, proxy.body.Position)`, `leadS = proxyS + max(0, proxySpeed)*0.15`,
`closing = clamp(proxySpeed+8, 12, 60)`; a new pure `Server.pursuitAcceleration(loop,targetS,position,velocity,speed)`
mirrors the existing acceleration law against the target lane frame with the **unchanged 40 studs/s² cap**. The latest
pursuit geometry is recorded in the runtime report as
`contactLeg = {t, distance, proxyS, targetS, proxySpeed, speed, lead}`. The old blind `wantedSpeed=-18` reverse is
**removed** (it is what drove the observer away).

**C — both named fail-fast guards.** A pure `Server.contactLegGuard(distance,touches,sinceContactDrive)` returns:
- `probe left the controller's observation radius (>700 of 800 studs): contact leg unreachable` when `touches<=0 and
  distance>700`;
- `contact leg never reached within 12 s of contactDriveAt` when `touches<=0 and sinceContactDrive>=12`;
and **never overrides a reached contact** (`touches>0` → nil). Both run before the next acceleration write.

**B is not used** (no bounded proxy-speed change is present — the ruled fallback stays unused unless a Play shows A
cannot close), and **D (pinning/holding the probed car) is absent** (rejected). No production file is touched.

## 3 · Tests (the full §5 offline list)

1. **Lune helper spec** — new `tests/unit/traffic-contact-pursuit.spec.luau` (3/3) executes the **real helpers** extracted
   from `Server.luau`: pursuit pushes toward the proxy's lane position, force cap holds (including lateral), a higher
   closing target accelerates harder; both named guard messages appear; a reached contact is exempt; **synthetic Play-1
   series regression** — the escaping post-contactDrive series `{23.9, 60, 180, 420, 720, 862}` trips the radius guard,
   while the closing series `{23.9, 18, 9, 3, 0}` never does.
2. **Python source-shape test** (`tests/tools/test_traffic_acceptance.py`): pins the pursuit helper + call, the proxy
   lane target/lead/closing clamp, both named messages, the `contactLeg` evidence row, the absence of the blind reverse,
   and that the force cap appears unchanged in both helpers (count 2).
3. **Mutation proofs (maker self-check, each caught, restored, tracked clean):** M14 blind reverse restored (no pursuit)
   → traffic acceptance FAILED (1); M15 observation-radius guard removed → Lune spec FAILED (2); M16 12 s no-touch timer
   removed → Lune spec FAILED (1); M17 closing clamp widened to 80 → traffic acceptance FAILED (1).

## 4 · Freeze evidence (fresh checkout of `1e8b5b7`)

| # | Command | Result |
|---|---|---|
| 1 | `git rev-parse HEAD; git status --short` | `1e8b5b7…`; tracked tree clean (792 pre-existing untracked evidence-dir lines, R2 cosmetic) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 91 tests … OK** |
| 3 | `python3 -m unittest discover -s tests/tools -p 'test_traffic_acceptance.py'` | **Ran 7 tests … OK** |
| 4 | `python3 -m unittest discover -s tests/tools` | **Ran 325 tests … OK** rc 0 |
| 5 | `./.tools/bin/lune run tests/unit/run.luau traffic` | **unit: 93 passed, 0 failed (18 specs)** — includes the new pursuit spec |
| 6 | `./.tools/bin/lune run tests/unit/run.luau` | **unit: 923 passed, 0 failed (127 specs)** rc 0 (~12 min); log `board/probes/team-d/lune-unit-ac-20261102.log` |
| 7 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…`, `c6651793…`, `c6651793…` — unchanged |
| 8 | `git show --stat 1e8b5b7` | 3 files, +105/−14 (`tests/traffic/Server.luau`, the new Lune spec, the source-shape test) |

**Exact checker reproduction:** `git checkout 1e8b5b774832bbaac2de9bbb0487fbd3baba5ae7` fresh worktree → `git status`
empty; run rows 2–6 (no Play/Studio needed); sealed row 7. Contract spot-checks: `Server.pursuitAcceleration` exists and
targets `targetS`; the heartbeat computes `proxyS/leadS/closing` exactly as ruled and calls the guard **before** the
acceleration write; `contactLeg` evidence is recorded; both named messages are present verbatim; `wantedSpeed=-18` is
absent; the force cap string appears twice. Mutations M14–M17 fail as above.

## 5 · Next (per Nox's standing sequence)

On checker delta ACCEPT: **Play B** runs in a **fresh opening** (`studiocycle` first; the current opening stays untouched
at 1/3 until the ruling/delta decide the next Play), one Play, sealed hashes before/after, `contactLeg`/guard evidence
judged per the batch-7 §6 abort-contact criteria; the reserved Plays stay reserved. The overhead-render limitation
ruling (options A/B/C/D) is **not implemented** and is pending Nox; no plan/validator change. Studio stays idle.
