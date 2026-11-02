# CHECKER — fixture A+C delta (`1e8b5b7`) — verdict: **ACCEPT** (for `1e8b5b7`; see tip-movement note)

Fresh-context delta re-verify of the Nox run-6 A+C abort-contact fixture on top of my accepted layer-5 delta fix
(`fc546e5`). I built none of it. No Play, no Studio input, no image reads, no host input, **$0**. Ran from a throwaway
clone `/tmp/checker-ac/repo` (2026-11-02T00:19:17-05:00) and a second clone for mutations; maker tree only read.

- **Freeze commit:** `1e8b5b774832bbaac2de9bbb0487fbd3baba5ae7` (`1e8b5b7`), parent `fc546e5` (my last accepted delta),
  tree `4d6dcdb47682581528f9f0e3ccbe0ae198d9c4e3`; tracked mods **0** at verification start; sealed
  `84324d43…`/`c6651793…` unchanged before/after and at end.
- **Delta:** exactly **3 files, +105/−14** — `tests/tools/test_traffic_acceptance.py` (+15/−12),
  `tests/traffic/Server.luau` (+35/−2), `tests/unit/traffic-contact-pursuit.spec.luau` (+55/−0, new). No `src/`,
  `assets/`, `build/` change; `tools/vinegar_run.py` and `tests/atlas/*` untouched → overhead settle/retry/audit/45 s
  bound/assert-and-skip intact.

## Verdict in one line

**ACCEPT** — fixture A is implemented exactly as ruled (closed-loop pursuit of the proxy's current lane position,
`proxyS=Sim.project(...)`, 0.15 s lead, `closing=clamp(proxySpeed+8,12,60)`, pure `Server.pursuitAcceleration` with the
unchanged 40 studs/s² cap, `report.contactLeg{…}` recorded, old blind `wantedSpeed=-18` reverse gone), C has both named
fail-fast guards (observation radius >700 of 800; 12 s no-touch timer) run before the next acceleration write with a
reached contact never overridden, B is not used and D is absent, the tests execute the real helpers and pin the wiring,
and all four maker mutations reproduce (1 / 2 / 1 / 1 failures). Suites are green including full Lune (923/0, 127 specs).

**Tip-movement note (for the Lead):** the maker branch advanced during my verification — HEAD is now `d83efa8` with
`701f70c` ("atlas: per-zone functional overhead views replace the whole-highway framing (Nox run-7 B+A)") and `d83efa8`
(tests) on top of the subject commit; the 5 tracked files I saw dirty at 00:20–00:21 became those commits. `1e8b5b7` is
an ancestor of the current tip and its tree is unchanged, and my fresh clone of `1e8b5b7` was clean — so this ACCEPT is
for the frozen `1e8b5b7` bytes only. The later commits (`701f70c`, `d83efa8`) are **not covered by this verdict**; if
Play B is to run on "the frozen commit", confirm which commit the working tree should be on.

## Per-item evidence

| Item | Verdict | Evidence (bytes / runs) |
|---|---|---|
| ① A — closed-loop pursuit as ruled | **PASS** | `tests/traffic/Server.luau:48-60`: pure `Server.pursuitAcceleration(loop,targetS,position,velocity,speed)` (lane frame, longitudinal/lateral clamps, `if acc.Magnitude>40 then acc=acc.Unit*40 end`); wiring `:207-218`: `local proxyS=Sim.project(source.poly,proxy.body.Position)`, `local leadS=proxyS+math.max(0,proxySpeed)*0.15`, `local closing=math.clamp(proxySpeed+8,12,60)`, `report.contactLeg={t=age,distance=distance,proxyS=proxyS,targetS=leadS,proxySpeed=proxySpeed,speed=closing,lead=0.15}`, `acc,lane=Server.pursuitAcceleration(source,leadS,ego.Position,ego.AssemblyLinearVelocity,closing)`; `wantedSpeed=-18` absent; force-cap occurrences = 2 (both helpers). Delta touches only `tests/` (no `src/`, assets, sealed) |
| ② C — both named guards, ordering, reached-contact exemption | **PASS** | `tests/traffic/Server.luau:61-66`: `Server.contactLegGuard(distance,touches,sinceContactDrive)` returns exactly `probe left the controller's observation radius (>700 of 800 studs): contact leg unreachable` when `touches<=0 and distance>700`, and `contact leg never reached within 12 s of contactDriveAt` when `touches<=0 and sinceContactDrive>=12`; `touches<=0` means a reached contact is never overridden (spec asserts `guard(900,1,20.0)==nil`). Wiring `:214-217`: `local guardError=Server.contactLegGuard(distance,#report.touches,age-contactAt); if guardError then error(guardError) end` **before** `report.contactLeg` and the `pursuitAcceleration`/force write |
| ③ B not used / D absent | **PASS** | No bounded proxy-speed change (no `proxySpeedMax`/`proxySpeed=math.clamp`), no pinning/holding construct in `tests/traffic/Server.luau`; the only proxy-speed use is the read + lead/closing formula |
| ④ Tests execute the real helpers + source-shape | **PASS** | `tests/unit/traffic-contact-pursuit.spec.luau` (new): extracts the real `pursuitAcceleration`/`contactLegGuard` blocks from `tests/traffic/Server.luau`, loads them under Lune with fake `Sim`/`Vector3`/`CFrame`, and asserts direction/cap, higher closing target accelerates harder, both named guards, reached-contact exemption, and the synthetic Play-1 series (escaping `{23.9,60,180,420,720,862}` trips the radius guard; closing `{23.9,18,9,3,0}` does not). Python `test_abort_contact_uses_closed_loop_pursuit_and_both_named_guards` pins helper/wiring/lead/clamp strings, `report.contactLeg` fields, absence of `wantedSpeed=-18`, and force-cap count 2. Runs: traffic acceptance `Ran 7 … OK`; Lune traffic filter `unit: 93 passed, 0 failed (18 specs)`; full Lune `unit: 923 passed, 0 failed (127 specs)` rc 0 |
| ⑤ Scope / earlier layers | **PASS** | Exactly the 3 claimed files; no game/asset/sealed path; `tools/vinegar_run.py` and `tests/atlas/*` untouched (`ATLAS_FIRST_VIEW_TIMEOUT = 45`, `capture_attempts_max = 3`, `overhead_settle = 1.0` intact) |
| ⑥ Suites + mutations | **PASS** | Fresh clone: runner `Ran 91 tests … OK` (13.407 s); traffic acceptance `Ran 7 … OK` (0.102 s); full Python `Ran 325 tests … OK` rc 0 (54.814 s); Lune traffic filter `93/0 (18 specs)`; `atlas-paths` `1/0`; full Lune `923 passed, 0 failed (127 specs)` rc 0 (00:19:28–00:31:28). Mutations in a throwaway clone, restored: **M14** blind `wantedSpeed=-18` restored → traffic acceptance **FAILED (failures=1)**; **M15** observation-radius guard removed → Lune pursuit spec **2 failed** (named radius guard + Play-1 escaping series); **M16** 12 s timer removed → Lune **1 failed** (named timer); **M17** closing clamp widened to 80 → traffic acceptance **FAILED (failures=1)**. Restored `tests/traffic/Server.luau` sha256 `bf29900eee5e…`, git clean |
| Host / no-Play | **PASS (as checked)** | I ran no Play/Studio; no latch; sealed unchanged. Play 1 attribution (escape from 23.9 to >800 studs, retired car) is the rationale; Play B is the live proof |

## Residuals (named, non-blocking)

- **R1 — live limit:** no Play; the pursuit's closing behaviour and both guards are proven offline (pure helpers +
  synthetic series), not on the engine. Play B (authorized on ACCEPT) is the live proof.
- **R2 — tip moved during verification (process):** current HEAD is `d83efa8` (`701f70c` B+A zone-overhead code +
  `d83efa8` tests) on top of `1e8b5b7`; those two commits are **outside this verdict** and must be checker-verified before
  a Play runs on them. `1e8b5b7` tree unchanged and my clone clean.
- **R3 — coverage note:** the delta replaced the old Python source-shape test, dropping the direct string pins for the
  `GenerationGuard` wiring (`guard=GenerationGuard.new`, `guard:observe`, violation→error) in `tests/traffic/Server.luau`.
  The wiring is unchanged in the frozen commit and the guard module is still covered by
  `tests/unit/traffic-generation-guard.spec.luau` (+ traffic-runtime spec loading), so this is a coverage reduction, not
  a behaviour change.
- **R4 — cosmetic:** report says "792 pre-existing untracked evidence-dir lines"; tree now has 807. No commit impact.
- **R5 — spec brittleness:** the Lune spec extracts helper blocks by regex from the source; it will fail loudly if
  formatting changes (acceptable for a fixture spec).
- No game code, assets, sealed artifacts, Studio input, image reads or host input; $0.

## Evidence paths

`board/probes/checker-fixture-ac-delta/` — `delta-verification.log` (HEAD/parent/tree/delta numstat + paths + sealed +
A/C code + no B/D + force cap), `tip-movement.log` (branch advanced to `d83efa8`; `1e8b5b7` ancestor, tree unchanged),
`suites.log`, `runner.log` (91 OK), `traffic.log` (7 OK), `python.log` (325 OK), `lune-traffic.log` (93/0, 18 specs),
`lune.log` (923/0, 127 specs), `atlas-paths.log` (1/0), `mutations.log` (M14 1, M15 2, M16 1, M17 1; restored hash),
`run-started.txt`. Throwaway copies `/tmp/checker-ac/{repo,mut}`; maker repo read-only (now at `d83efa8`), frozen commit
`1e8b5b7` unchanged.
