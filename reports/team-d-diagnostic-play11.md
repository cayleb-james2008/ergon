# TEAM D — DIAGNOSTIC PLAY 11 REPORT: option-B abort-contact Play — proxy phase left the leg-start gap at 71.4 studs, guard fired (2026-11-02)

Lead GO: 2026-11-02T06:26Z (after option-B delta ACCEPT on `68c020b`). Sequence: state + sealed BEFORE → `studiocycle
cycle` → fresh opening verified → `preflight` → **one** `run traffic:abort-contact` → `verify-clean` → sealed AFTER.
No extra Plays, no build/publication/credential, no image reads. Spend $0.
Run dir: `~/Work/projects/untitled-street-game/docs/verify/2026-10-19/vinegar/20261102T062704Z-traffic-abort-contact`
Log: `board/probes/team-d/diagnostic-play11-20261102.log`.

## 1 · Verdict in one paragraph

The option-B fixture is **wired exactly as ruled and the Play exercises it** (`fixture.preLegSpeed=30`, contact leg
starts at `abortObservedAt+0 = 3.7826 s`, `-18` absent, ruled pursuit law/guards byte-identical from the accepted delta),
but the cell **fails honestly in a new way**: because this run's traffic car was in a different phase at the abort
(`abortObservedAt 3.7826 s`, proxy ~27 studs/s there), the **leg-start gap was 71.4 studs**, not the 24.3 the offline
kinematic model predicted from Play 9's proxy profile. With ego rolling at 30 studs/s the pursuit closed as designed
(ego 30→59, distance 71→~104→142 at the window end; the 12 s window is too short for that start), and the **12 s
no-touch guard fired at t=15.765 s** (`HandlingServer:218`), the radius guard correctly not tripped (max 142 << 700).
**contacts=0, touches=0, cell FAIL**; no crash; sealed unchanged; plays **1/3** in the fresh opening (pid 905535).

## 2 · Evidence table (verbatim)

| Fact | Value |
|---|---|
| Pre-state | Studio pid **881911** (old opening, 2/3) → `studiocycle cycle` 06:26:45Z: close `killed-after-dialog` (place sha unchanged, nothing saved), reopen **pid 905535**, window `0x558c7f831f20`, 2200×1000; sealed place before==after `c6651793…` |
| Sealed BEFORE / AFTER | release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`; test/place `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` — **unchanged** |
| Preflight | rc **0**, refusals `[]`, **Edit, plays 0/3**, place == build == `c6651793…`, host quiet |
| Run | 06:27:04Z → 06:27:38Z (~34 s); exit **1**; states `LOCKED → PREFLIGHT → STAGED 2.567 → PREPARED 2.575 → PLAYING 4.383 → RAN 6.982` |
| Failure reason | `HandlingServer:218: contact leg never reached within 12 s of contactDriveAt` (stack `:218` ← `:177`); runtime `state:"failed"` |
| Option-B wiring in the run | `fixture.preLegSpeed=30`; `cutInAt 2.783`; `abortObservedAt 3.7826`; **`contactDriveAt 3.7826`** (abort+0, no 4 s wait); `maneuverPoseWrites/VelocityWrites=0` (no-maneuver contract intact, `initialBodyPlacements/VelocityWrites=2`) |
| Cell verdict | **FAIL** — **touches 0, contacts 0**; `maximumProgress 0.9938`; `completedAt 6.833` |
| Leg-start geometry (real) | gap **71.4 studs**, ego **30.0 studs/s** (offline model said 24.3/30 using Play-9's proxy profile; this run's proxy phase differed: abort 3.78 s vs 4.98 s, proxy 27 vs 11 studs/s) |
| Pursuit | ego ramps 30 → **59.06** studs/s; proxy steady **52.17**; distance 71.4 → peak **141.75** (t=15.68) → latched **142.44** at the guard; `contactLeg={t:15.765,distance:142.444,proxyS:3216.121,targetS:3223.947,proxySpeed:52.170,speed:60,lead:0.15}` |
| Guards | **12 s no-touch timer: TRIPPED as designed** (named message); **observation-radius guard: NOT tripped** (max 142 << 700); reached-contact exemption untested |
| Generation / diagnostics | `generationStable:true`, `initialGeneration:2`, `finalGeneration:2`, `generationEvents:[]`, steps 939; chunks `null` (actual) |
| Crash | **none**: `newCrashDumps:null`, abort `{stop:"ok", cleanup:"ok", verifyClean:"ok"}`, hostcheck clean |
| Post-state | `verify-clean` rc 0, `clean:true`; Studio alive pid **905535**; plays **1/3** in the fresh opening |

## 3 · Judgement (batch 7 §6 abort-contact)

- **Did contact occur inside the ruled window with the ruled law untouched and guards correct?** **No contact.** The law
  and guards behaved exactly as accepted; the failure is the **start gap** (71.4) against a **correct** 12 s window. The
  guard fired honestly; no fabricated contact; the audit reflects reality.
- **Why the modelled 24.3 was wrong:** the offline kinematic expectation used **Play 9's proxy speed profile** (a single
  run) as a fixed proxy trajectory. The actual traffic car's phase at `abortObservedAt` varies run-to-run — Play 9 aborted
  at 4.98 s with the proxy at ~11 studs/s (gap 20.2); Play 11 aborted at 3.78 s with the proxy at ~27 studs/s and the ego
  still 71 studs behind (it had been braking behind the cut-in car). **A fixed pre-leg speed cannot bound the leg-start
  gap** — this is a new named layer: **layer 8 = proxy-phase variability / unbounded leg-start gap**.
- **What it proves:** the option-B wiring works (leg starts at abort+0, rolling pre-leg recorded, no maneuver writes, the
  ruled pursuit/guard code is untouched and fails closed correctly); the guards remain correct under a large gap; sealed
  integrity and Studio lifecycle hold; the offline model is not a substitute for the Play.
- **What it does not prove:** no cell pass; no contact; the 12 s feasibility for an arbitrary run phase; the model's
  proxy profile is one-run data (a residual: the kinematic test should be marked as Play-9-profile-specific).

## 4 · Options for the Lead/Nox ruling (no implementation)

1. **A (extend the no-touch window 12 → 20 s; ruled guard-value change).** From the observed Play-11 start (gap 71.4,
  ego 30), the model needs ~14.5 s (ramp 3 s + closing from ~92 at 8 studs/s ≈ 11.5 s) → a 20 s window would pass with
  margin. It is phase-robust for the observed range; costs a longer fixture runtime and weakens the "fails fast" property.
2. **B-bounded (fixture-only): start the leg at a bounded gap trigger, not a fixed time** — e.g. leg starts at the first
  heartbeat after abort when `distance <= 25`; if the gap never enters 12–25 within a bounded pre-leg (e.g. abort+4 s),
  **fail with a named message** ("phase made the contact leg unreachable") instead of the generic timer. This keeps the
  12 s guard but makes the scenario conditional; Play 11's run would then fail fast with the named phase message rather
  than after 12 s. Combined with a **smaller initial ego offset** (e.g. 40–50 instead of 80) it could bound the closest
  approach; both are fixture-only but change the scenario description.
3. **A+B-bounded together** would be the robust choice if Nox accepts a 20 s bound plus a named phase fail.
Recommendation: **A (20 s)** as the smallest phase-robust change; B-bounded as the stricter alternative if the 12 s
property must stay. **Nothing implemented; the fixture/law/guards stay exactly as accepted.**

## 5 · Next

Awaiting the ruling. The current fresh opening is at **1/3**; the remaining Plays are reserved. The world-overhead
B2+A2 ruling is still with Nox (implementation-ready note filed; not implemented). No Play consumed by this report.
