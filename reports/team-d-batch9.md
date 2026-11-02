# TEAM D — BATCH 9 REPORT (Jobs 16-17): Plan 4 frozen — atlas named-seed assert-and-skip + bounded hang detection — 2026-11-02

Machine: `outis-desktop` (Linux 7.2.8-9-omarchy, Hyprland/Wayland), user `cayleb`. Lead: `roblox-lead` (Team D).
Tier A. **Spend $0 of $0.** Publication not authorised and not attempted. No credential value asked, printed, copied,
logged or committed. Maker never checker: **nothing here is a DONE**; the fresh checker certifies Plan 4.
Batch window: **2026-11-02 01:20Z – 02:00Z** (20:20–21:00 ET). Plan 3 verdict observed: **ACCEPT** (`checker-trading`).

Repo: `~/Work/projects/untitled-street-game`, branch `prep/b1-capture-format-20261101`.
**Plan 4 frozen commit (after the checker fix round): `659779ce89fa18c8ab2dc13a29906f7ab05e1499`** (`659779c`; parent
`9022c53` = first Plan 4 freeze; branch tip; tree `844523c3fcbd97dcca0caf2db8fbb25d9217a34c`; tracked tree clean; 792
untracked evidence-dir lines, pre-existing). Fix round = 4 files, +33/−2 (C1/C2 + regression tests). The original Plan 4
commit `9022c53` is 9 files, **+185/−71** — the batch-9 first report said +186/−71; **C3 corrects it here**. Includes the
checker's docs nit: `docs/art/SIDECAR.md` header/table newline fixed.

## 0 · Verdict in one paragraph

Plan 4 is implemented, tested, mutation-checked and **frozen at `659779c`** (fix round over `9022c53`). The seeded atlas fixture now **asserts**
the already-built world instead of rebuilding it inside a Play: matching seed+layout → `rebuild=false` and the run
proceeds; mismatch → **named fail-fast** (`named seed not applied before Play: world reports seed X layout Y, expected
2134783819/2; refusing in-Play rebuild`). **No in-Play City+Highway rebuild path exists any more** (`WorldSeed.ensure`
is gone; the module contains no `generateSafe` call). A Plan 4 instrumentation trace (`t0, rootFoundAt, worldReadyAt,
actualSeed, actualLayout, expectedSeed, expectedLayout, fastPathApplied, rebuild, starter`) is published as the Workspace
attribute `AtlasWorldTrace` **before any capture**, added to the runtime inventory and to the error report, and the
runner bounds the `view-001` wait to **45 s**, keeping the trace as `world-trace.json` + `result.json.worldTrace` on
success and on failure. Evidence: runner **84 OK**, full Python **313 OK**, full Lune **919 passed / 0 failed
(126 specs)** rc 0, atlas acceptance 15 OK, world-seed spec 10/0 (2 specs), six mutation proofs each caught (one as a
non-terminating hang), sealed `84324d43…`/`c6651793…` unchanged, tracked tree clean. No Play, no Studio input, $0.

## 1 · Plan 4 — named-seed assert-and-skip + bounded hang detection

**Why.** The 2026-11-01 seeded `atlas:scout --seed 2134783819` prepared cleanly, started Play, then hung **>600 s**
before `view-001` (collector only `prepared`+`cleanup`). Offline reading: `WorldSeed.ensure` called
`World.generateSafe` on the already-populated world to "apply" the named day; that in-Play City+Highway rebuild never
returned. Lead ruling: assert-and-skip — an in-Play rebuild must not be the route.

- `tests/atlas/WorldSeed.luau`: `WorldSeed.ensure` (rebuild-on-mismatch) **replaced** by
  `WorldSeed.requireApplied(root, expectedSeed, expectedLayout)`. Matching → `{fastPath=true, rebuilt=false}`; mismatch
  → named fail-fast with observed **and** expected identity. No `generateSafe` call exists in the module (a rebuild is
  only ever the world's own Bootstrap build).
- `tests/atlas/Server.luau`: assert-and-skip replaces `ensure`; the pre-rebuild seed locals are re-read after the no-op
  check; the trace is built from Play-time clocks (`t0`, `rootFoundAt`, `worldReadyAt`) and the world identity, published
  as `workspace:SetAttribute("AtlasWorldTrace", Http:JSONEncode(trace))` **before the view loop** and added to the
  runtime inventory as `worldTrace`; `fastPathApplied` falls back to "world already reports the plan".
- `tools/atlas_acceptance.py`: `SEED_PATCH` (the starter's pre-Bootstrap patch) records `AtlasStarterTrace
  {t0, applied, seed, rootSeed}` at Play start — the ordering datum (`rootSeed` is read when the root already exists);
  a failing starter start now posts `worldTrace=workspace:GetAttribute("AtlasWorldTrace")` with its error, so a
  fail-fast keeps the evidence.
- `tools/vinegar_run.py`: `Atlas.first_view_timeout = 45` — the `view-001` wait is bounded to tens of seconds instead
  of 600. `ATLAS_WORLD_TRACE` read-only Server exec fetches the trace; it is written to `world-trace.json` and merged into
  `result.json` as `worldTrace` on success, on first-view timeout and on runtime failure.
- `docs/art/SIDECAR.md` (checker nit): the atlas-acknowledgement table header newline is fixed (docs-only).
  `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md`: records the frozen plan seed **2134783819 / layoutVersion 2** and the
  assert-and-skip rule (`rebuild=false` on match; named fail-fast on mismatch; never rebuild City+Highway in Play; 45 s
  first-view bound; trace fields and file).

**Fallback (a) (disable Bootstrap in prepare; one Atlas starter builds the named world from scratch) is NOT implemented**
on purpose: it is conditional on the diagnostic Play's evidence. If `fastPathApplied=false` (starter patched
`World.dailySeed` after Bootstrap already built the day), the cell now fails fast in ≤45 s with the trace, and fallback
(a) is the ruled next step. If `fastPathApplied=true`, assert-and-skip is sufficient and the atlas cells proceed.

## 1b · Checker fix round (C1/C2/C3) — commit `659779c`

- **C1 (functional, fixed):** `AtlasWorldTrace` was published only **after** `WorldSeed.requireApplied`, so the mismatch
  fail-fast — the exact case the diagnostic Play exists for — left the trace nil. `tests/atlas/Server.luau` now publishes
  the trace **before** the named-seed check (`actualSeed`, `actualLayout`, `expectedSeed`, `expectedLayout` already in it)
  and **refreshes** it with the decision (`fastPathApplied`, `rebuild`) after the check, still before any view. So a
  fail-fast keeps the trace the Play needs.
- **C2 (same path, fixed):** the runner's first-view handler caught `TimeoutError`/`FixtureFailed` but not
  `collector.wait`'s **early-stop `ValueError`** (a fail-fast runtime arriving before `view-001` — the fail-fast path).
  `tools/vinegar_run.py` now catches `ValueError` too, reads `AtlasWorldTrace`, and raises the named failure with the trace
  attached (`world-trace.json` + `result.json.worldTrace` kept).
- **C3 (bookkeeping, fixed):** `9022c53` is **+185/−71** (the first report said +186/−71); corrected above.
- Regression tests: atlas acceptance asserts the first publication **precedes** `WorldSeed.requireApplied` and that the
  trace is published twice (before the check + refreshed); the runner gains the **early-stop fail-fast test** (`ok=false`
  runtime before `view-001` → exit 1 with `worldTrace` in the error, `world-trace.json` on disk, 0 captures, clean abort).
- Fix-round mutation proofs (both caught, restored): publication moved back after the check → atlas acceptance FAILED (1);
  `ValueError` branch removed → runner suite FAILED (1).

## 2 · Freeze evidence (commands + results; full log `board/probes/team-d/plan4-atlas-named-seed-20261102.log`)

| # | Command (fresh checkout of `9022c53`) | Result |
|---|---|---|
| 1 | `git rev-parse HEAD; git status --short` | `659779ce…` (fix-round tip; original freeze `9022c53`); tracked tree clean (792 untracked evidence-dir lines) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 85 tests … OK** |
| 3 | `python3 -m unittest discover -s tests/tools` | **Ran 314 tests … OK** rc 0 |
| 4 | `./.tools/bin/lune run tests/unit/run.luau` | **unit: 919 passed, 0 failed (126 specs)** rc 0 (~12 min; green at `9022c53`; the fix round touches no Lune-read file, re-run on the fix commit in the same green state — log `board/probes/team-d/lune-unit-plan4-20261102.log`) |
| 5 | `python3 -m unittest discover -s tests/tools -p 'test_atlas_acceptance.py'` / `./.tools/bin/lune run tests/unit/run.luau world-seed` | 15 OK / 10 passed, 0 failed (2 specs) |
| 6 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…`, `c6651793…`, `c6651793…` — unchanged before/after |
| 7 | `git show --stat 9022c53; git show --stat 659779c` | 9 files +185/−71 (corrected per C3) + fix round 4 files +33/−2 |

**Mutation proofs (maker self-check, each caught, files restored, tree clean):** (A) mismatch returns instead of
failing fast → world-seed spec 9 passed/1 failed; (B) layout leg dropped (seed-only check) → 9/1; (C) 45 s first-view
bound removed → the bounded-first-view test does not terminate (killed at 60 s = the old hang caught); (D) trace dropped
from the timeout failure → runner suite FAILED (1); (E) `AtlasWorldTrace` publication removed → atlas acceptance FAILED
(1); (F) starter `applied`/`rootSeed` evidence removed → atlas acceptance FAILED (1); fix round: (G) publication moved
back after the check → atlas acceptance FAILED (1), (H) `ValueError` early-stop branch removed → runner FAILED (1).

**Exact checker reproduction (delta):** `git checkout 659779ce89fa18c8ab2dc13a29906f7ab05e1499` in a fresh worktree → `git
status` empty; run rows 2–5 (Lune ~12 min; no Play or Studio needed for any of them); sealed hashes row 6. Contract spot
checks: `WorldSeed.requireApplied` exists and `WorldSeed.ensure`/`generateSafe(` are absent from `tests/atlas/WorldSeed
.luau`; `Server.luau` publishes `AtlasWorldTrace` before the view loop, has `worldTrace=trace` in the inventory, and
never calls `world.generateSafe`; `Atlas.first_view_timeout == 45`; a silent server yields exit 1 with `worldTrace` in
the error, `world-trace.json` on disk and **0 captures**; `SEED_PATCH` records `AtlasStarterTrace {applied, rootSeed}`
before `require(h).start(n)` and the error post carries `worldTrace`. Mutation list §2.

## 3 · Studio / holds / spend (read-only check 01:55Z)

Studio **pid 388820**, window `0x558c7f6b0160`, 2200×1000 floating, Edit, current opening **0/3 Plays** (play record
belongs to the previous pid 382409), place == build == `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98`,
sealed release `84324d43…`. No Play, no Studio launch/close/resize/input, no build, no upload/publication/purchase, no
credential value, no image reads. $0 of $0.

## 4 · Next step + successor

**Plan 4 fix round is frozen and ready for the checker's delta re-verify** (`659779ce89fa18c8ab2dc13a29906f7ab05e1499`; reproduce block §2; delta over the accepted `9022c53`).
On ACCEPT the instrumented diagnostic block is complete (Plans 2+3+4 frozen and accepted) and the single diagnostic
Play runs **only on the Lead's explicit go** — the staged sequence is batch 7 §6 (`preflight` → one
`run traffic:abort-contact` → `verify-clean`), which after Plan 3 adds bounded `generationEvents`/`diagnostic` to the
runtime report. The atlas cells' next Play (after the traffic diagnostic) will now show `worldTrace.fastPathApplied` in
≤45 s; if false, apply fallback (a) as a named follow-up.

**Evidence files:** `board/probes/team-d/plan4-atlas-named-seed-20261102.log`,
`lune-unit-plan4-20261102.log`; prior: `plan3-generation-instrumentation-20261102.log`, `sidecar-technology-ruling-
20261102.md`; plans under `board/plans/team-d/`. Reports: `board/reports/team-d-batch7.md` (Play sequence),
`team-d-batch8.md` (Plan 3), this file.
