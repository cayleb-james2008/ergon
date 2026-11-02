# TEAM D — BATCH 7 REPORT (Jobs 16-17): Plan 2 frozen, doc gap closed, diagnostic Play staged — 2026-11-01/02

Machine: `outis-desktop` (Linux 7.2.8-9-omarchy, Hyprland/Wayland), user `cayleb`. Lead: `roblox-lead` (Team D).
Tier A. **Spend $0 of $0.** Publication not authorised and not attempted. No credential value asked for, printed,
copied, logged or committed. Maker never checker: **nothing here is a DONE**; the fresh checker certifies Plan 2.
Batch window: **2026-11-01 23:34Z – 2026-11-02 00:30Z** (19:34–20:30 ET).

Repo: `~/Work/projects/untitled-street-game`, branch `prep/b1-capture-format-20261101`, base `271b601`.
Frozen tree at HEAD: `29a626b2b8a13556ac5ee07db6fdb60401ecc6e8` (tree `9dba457b070564c042df82dae41092e75cec610e`).

## 0 · Verdict in one paragraph

Plan 2 (**bounded world-ready gate for `moving`/`first-fare`**) is implemented, tested, mutation-checked and **frozen
at `29a626b`**, parent `a6ac63a` (doc-only; the code/test delta is vs `d2a94db`); tracked tree clean, sealed artifacts unchanged, suites green (runner 81/81, full
Python 298 OK, Lune 914/914). The D3 doc gap is closed in a **doc-only follow-up commit `a6ac63a`**. Studio is idle in
Edit at **0/3 Plays**, place == build == `c6651793…`, window `0x558c7f6b0160` 2200×1000 floating; **no Play, no build,
no Studio input** in this batch. One new finding changes the diagnostic block: **`Lighting.Technology` cannot be read
from any `execute_luau` context** ("lacking capability RobloxScript", probed 23:45:08Z), so §1 of `docs/art/SIDECAR.md`
(`report.lighting.technology`) is **unsatisfiable by the probe** — every capture fixture (moving + all atlas views) will
still fail its sidecar after Plans 3/4 unless the Lead rules a contract amendment. Options + recommendation in §2.
Plans 3 and 4 were **not started**: the order is freeze → fresh checker → next, and Plan 2 is ready for that checker.

## 1 · D1 — Plan 2 implemented and frozen: `29a626b`

**Change (fixture timing only; no game code, no relaxed sidecar contract).** `tools/vinegar_run.py`:
`FirstFare.run()` (shared by `moving` and `first-fare`) now calls `await_world_ready(ctx)` before the first
`probe()`/capture. The gate is a read-only **Server** exec (`WORLD_READY` Luau, new constant) that reads the
Workspace `Nightshift` root's `WorldReady`, `WorldSeed`, `WorldLayoutVersion` plus the CityBuilder-created
`Lighting` `Sky`/`Atmosphere` presence; a not-ready answer re-polls every 0.25 s **inside a 30 s bound**
(`WORLD_READY_TIMEOUT`, never past the run budget). On expiry it raises `WorldNotReady` → exit 1 with the named
message `world not ready within 30s (Nightshift/WorldReady/WorldSeed/WorldLayoutVersion/Sky/Atmosphere): {last state}`;
no capture and no sidecar are written. On success the trace goes into `result.json` as
`worldReady: {waitedSeconds, polls, seed, layoutVersion, lighting:{technology, readable, skyPresent, atmospherePresent}}`
(new `Fixture.result_trace()` hook, merged without overwriting any result field). The frame sidecar contract is untouched.

Why the gate matches the real race: `src/server/World.luau:350` sets `WorldReady=false` first and only at the end
(`:393-395`) sets `WorldSeed`, `WorldLayoutVersion` and `WorldReady=true`; CityBuilder creates `NightshiftSky`/
`NightshiftAtmosphere` inside that build. The 2026-11-01 `moving` Play wrote frame-001.jpg but its after-probe had no
`seed`.

**Deviation from the plan text (deliberate, evidence-backed, for the Lead to accept or overrule):** the approved plan's
pseudo-gate also required `lightingTechnology`. **That field cannot ever be non-nil** in this MCP context (§2), so
requiring it would make the gate time out on every run. The gate therefore requires the *readable* lighting legs
(Sky/Atmosphere) and **records** `lightingTechnology`/`lightingReadable` as observed. Everything else matches the plan.
No assertion/judge change was made; `docs/art/SIDECAR.md` is unmodified, so the capture contract question is open.

**Files/diff (frozen commit):** 3 files, +203/−2 —
`tools/vinegar_run.py` +81 (constants `WORLD_READY`/`WORLD_READY_TIMEOUT`/`WORLD_READY_POLL`, `WorldNotReady`,
`world_ready_state`, `_world_ready`, `await_world_ready`, `Fixture.result_trace`, `FirstFare` gate + trace, `run_fixture`
trace merge); `tests/tools/test_vinegar_run.py` +122; `docs/verify/2026-10-19/VINEGAR_RUNBOOK.md` +2 (the gate/trace
paragraph).

**Tests (offline; all pass):** 5 new gate tests + 1 Lune mock run of the real Luau:
`test_world_ready_gate_reads_before_the_first_capture_and_records_its_trace` (ready on the first read: `polls=1`,
trace seed 2134783819 / layout 2 / lighting technology null+readable false / Sky+Atmosphere true, readiness read
happened when 0 captures existed), `test_world_ready_gate_waits_through_not_ready_reads_then_proceeds` (3 not-ready reads
then ready → `polls=4`, no capture while not ready), `test_first_fare_takes_the_same_bounded_world_ready_gate`,
`test_world_ready_gate_fails_closed_with_the_named_message_and_no_capture` (patched 0.05 s bound: exit 1, named message,
`worldReady.timeout=true`, 0 `screen_capture`, no `frames/*.json`, abort clean), and
`test_world_ready_gate_keeps_requiring_the_readable_lighting_contract` (no Sky/Atmosphere → timeout; no seed → timeout).
`test_world_ready_luau_reports_the_gate_contract_against_a_mock_server` runs the real `WORLD_READY` under Lune against a
mock Server (ready + not-ready cases; unreadable `Technology` swallowed).

**Maker mutation proofs** (file restored after each; final suite re-run OK):
(a) remove the gate call from `FirstFare.run` → runner suite FAILED (3 failures + 3 errors);
(b) `_world_ready` without the Sky/Atmosphere requirement → FAILED (1 failure);
(c) timeout returns as-if-ready instead of raising → FAILED (3 failures);
(c2) timeout raise deleted (gate loops) → suite does not terminate (the bound is the only exit) — caught.
Restored tree: 81/81 OK.

**Freeze evidence (commands + results + timestamps; full log `board/probes/team-d/plan2-world-ready-gate-20261101.log`):**

| # | Command | Result |
|---|---|---|
| 1 | `git -C ~/Work/projects/untitled-street-game status --short` | tracked tree clean (792 untracked evidence-dir lines, pre-existing) |
| 2 | `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` | **Ran 81 tests … OK** (10.6 s) — 00:22Z |
| 3 | `python3 -m unittest discover -s tests/tools` | **Ran 298 tests … OK** (41.2 s, rc 0) — 00:22Z; log `board/probes/team-d/python-suite-plan2-20261101.log` |
| 4 | `./.tools/bin/lune run tests/unit/run.luau` | **unit: 914 passed, 0 failed (125 specs)**, rc 0 (~12 min) — log `board/probes/team-d/lune-unit-plan2-20261101.log` |
| 5 | `sha256sum build/release.rbxlx build/test.rbxlx ~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` | `84324d43…` (1,990,548 B), `c6651793…` (2,403,691 B), `c6651793…` — unchanged before/after every run |
| 6 | `python3 -m py_compile tools/vinegar_run.py tests/tools/test_vinegar_run.py` | rc 0 |

**What the fresh checker must reproduce on `29a626b2b8a13556ac5ee07db6fdb60401ecc6e8`** (parent `a6ac63a`; branch
tip, tracked tree clean; `git checkout 29a626b` in a scratch worktree and `git status` must be empty):
1. `python3 -m unittest discover -s tests/tools -p 'test_vinegar_run.py'` → `Ran 81 tests … OK`.
2. `python3 -m unittest discover -s tests/tools` → `Ran 298 tests … OK`, rc 0.
3. `./.tools/bin/lune run tests/unit/run.luau` → `unit: 914 passed, 0 failed (125 specs)`, rc 0 (bounded ~15 min).
4. Sealed hashes as in row 5 (do **not** run `tools/usg build`; it rewrites `build/` though deterministically — the
   clean way is the hash check only).
5. Mutation checks: (a) delete the `await_world_ready` block from `FirstFare.run` → the 5 gate tests fail;
   (b) drop the `skyPresent/atmospherePresent` legs from `_world_ready` → 1 failure; (c) make the timeout return a trace
   instead of raising → 3 failures. Restore each time.
6. Gate contract spot-check: `V.WORLD_READY` contains `WorldReady`/`WorldSeed`/`WorldLayoutVersion` and does **not**
   gate on `lightingTechnology`; `result.json` carries `worldReady` on both pass and failure paths
   (`run_fixture` merge; `FirstFare.result_trace`).
No Play is needed to check any of the above.

**Worktree note for the checker:** commit order on the branch is `d2a94db` → `a6ac63a` (D3, doc-only, parent
`d2a94db`) → `29a626b` (Plan 2, tip). `git show --stat 29a626b` lists exactly the 3 Plan 2 files above (its +2 for
`docs/verify/…/VINEGAR_RUNBOOK.md` is the gate/trace paragraph); the queue-rules rewrite belongs to `a6ac63a` and is
visible only when diffing the range (`git diff d2a94db 29a626b -- docs/verify/2026-10-19/VINEGAR_RUNBOOK.md`). Neither
commit touches game code, the sealed build, or the sidecar contract.

## 2 · NEW FINDING — `report.lighting.technology` is unsatisfiable; needs a Lead ruling before any capture cell can pass

**Probe (read-only, no Play):** `/tmp`-style script saved as `board/probes/team-d/technology-capability-probe-20261101.py`;
MCP `execute_luau`, Edit mode, **2026-11-01T23:45:08Z**, Studio id `cdefa06f-…`, result
(`board/probes/team-d/lighting-technology-capability-probe-20261101.json`):

```json
{"directOk": false, "nameOk": false, "typeOk": false,
 "nameError": "The current thread cannot read 'Technology' (lacking capability RobloxScript)", "studioMode": true}
```

Corroboration: (i) the 2026-11-01 `moving` Play probe *also* lacked `lighting.technology` (the B1 error listed it beside
`seed` — `board/probes/team-d/b1-opening3-atlas-moving-findings-20261101.md:46-50`); (ii) the repo already encodes this
as a real condition: `tests/tools/test_vinegar_run.py:1618-1628` ("Lighting.Technology unreadable from the Client exec")
asserts the capture fails closed on it; (iii) the sealed place **does** have `Technology=4` (`Enum.Technology.Future`,
the value `docs/art/SIDECAR.md` expects) in `build/test.rbxlx` XML, so the truth is knowable **offline**, just not via
`execute_luau`.

**Consequence.** `frames.look_report()` requires `report.lighting.technology` to be a string; `frames.sidecar()` is used
by `moving` (line 728) **and every atlas view** (line 643), and `tools/look_metrics.py` requires `"Future"`. So after
Plans 2/3/4 every capture fixture still fails closed on this one field. The B1 cells `atlas:scout` /
`atlas:highway-extension` (and the whole B3 look ladder) cannot pass without a contract decision. No judge change was
made by me.

**Options (for the Lead; recommended first):**
1. **Run-level offline assertion (recommended, honest, no probe capability needed):** the runner reads the sealed
   place's XML `Lighting.Technology` (deterministic, hash-pinned place `c6651793…`/build `test.rbxlx`) and records it in
   each sidecar as a run field (e.g. `report.lighting.technology` + `source:"place-file"`), `look_metrics` compares it
   with the same build. The probe keeps recording `technologyReadable:false`. Requires a small `frames.py`/`look_metrics`
   change and a SIDECAR.md note — a **contract amendment**, so it needs the Lead's ruling and a fresh checker.
2. **Game-side attribute:** `CityBuilder`/`Bootstrap` sets e.g. `Lighting:SetAttribute("NightshiftTechnology","Future")`
   once (a constant of the code under test) and the probe records the attribute. Weaker (self-reported) and needs a
   production-code change + rebuild of the sealed candidate — **not recommended** for a sealed candidate.
3. **Drop/relax the per-capture field:** record it as observability-only (`technology:null`, `readable:false`) and make
   `look_metrics` accept it when the place-file evidence is supplied out of band. Weakest evidence, cheapest.
   (Also possible: keep the field mandatory only for captures taken in Edit — but captures are Play-only.)

## 3 · D2 — Plans 3 and 4: not started (by order), ready to start on Plan 2 ACCEPT

Per the order "each freeze → fresh checker → next", Plan 2 is frozen and **ready for the fresh checker now**. On ACCEPT
I implement, in this order, with one freeze each:
- **Plan 3 — traffic generation instrumentation** (`tests/traffic/Server.luau` + `traffic_acceptance` test shape):
  bounded `report.generationEvents` ≤20 with `{t, step, oldGeneration, newGeneration, oldBodyAlive, newBodyAlive,
  hasChange, progress, cutInAt, abortObservedAt, contactDriveAt, contacts, touches, source}` + `report.diagnostic`
  (`generationEvents final, finalGeneration, generationStable`), assertion stays fail-closed; offline Lune + Python tests;
  no production code.
- **Plan 4 — atlas named-seed assert-and-skip** (`tests/atlas/Server.luau` + starter): verify world seed/layout ==
  **2134783819 / layoutVersion 2** → `rebuild=false`, skip; mismatch → named fail-fast, **never** `World.generateSafe`
  on a populated world; starter/Play trace `{t0, rootFoundAt, worldReadyAt, actualSeed, actualLayout, expectedSeed,
  expectedLayout, fastPathApplied}`; runner-side 45 s bound for the diagnostic only; fallback (a) = prepare disables
  Bootstrap and a single Atlas starter builds the named world from scratch. Tests for both branches; instrumentation
  trace dumped into the failure report.
Both freezes will use the same evidence discipline as Plan 2 (suites, mutations, sealed hashes, no Play).

## 4 · D3 — doc gap fixed: `a6ac63a` (doc-only, parent `d2a94db`)

`docs/verify/2026-10-19/VINEGAR_RUNBOOK.md:62-74` no longer describes the old retry semantics. It now matches the
accepted Plan 1 help/code and board runbook §1e: queue lines carry `run`'s options (shlex.split, same validation; smoke
takes none); `done` = **exit 0 only**; a non-zero entry is recorded in `failed` with class `cell`/`harness`/
`smoke-failure` and the queue **stops**; `--resume --rerun` retries exactly once per invocation (attempt history kept,
no automatic retry loop, plain `--resume` refuses while `failed` is set); an interrupted entry stays undone and a plain
`--resume` retries it; a changed queue text (sha256) invalidates the checkpoint; a stopped queue exits with that entry's
code, a paused/complete queue exits 0. The example now shows the frozen atlas seed `--seed 2134783819`.
Commit `a6ac63a` = 1 file, +6/−6, no code/test change (`git show --stat a6ac63a`). This is a named follow-up commit,
not part of the Plan 2 freeze target.

## 5 · D4 — Studio state (read-only re-verify, 2026-11-02T00:22:53Z)

- Studio **pid 388820** (RobloxStudioBeta.exe under vinegar), **window `0x558c7f6b0160`**, **2200×1000 floating** at
  621,233 (`winctl get --target studio`), title `…usg-desktop.rbxlx - Roblox Studio`.
- **Edit** mode, **plays 0/3** (fresh opening), MCP preflight **rc 0, refusals []**; `list_roblox_studios` →
  `usg-desktop.rbxlx`; tools list complete. Place file = build test place =
  `c6651793ace089a093f50f51cc2a649d0a1e3cc9e790660baa8445ccf418fe98` (matchesBuild true).
- Sealed release `84324d4342667dd163ab8e64adacbf8ef44ab786aeda5c7ab1403e76d52ef6ce`.
- `studiocycle snapshot`: place sha `c6651793…` (backup recorded). **No Studio input injected, no window/lifecycle change,
  no Play**; the only Studio interaction was the read-only MCP probe in §2 (list + one read-only execute_luau in Edit).

## 6 · D5 — the single diagnostic Play command sequence (for the Lead's approval; nothing run)

Preconditions (all re-verified at run time): branch tip == accepted freeze, tracked tree clean; Studio in Edit, 0/3
Plays, place==build==`c6651793…`; `vinegar_run.py preflight` rc 0; no Play since the freeze; `studiocycle snapshot` taken.
Then **one** Play (the only Play of the block):

```bash
cd ~/Work/projects/untitled-street-game
python3 tools/vinegar_run.py preflight                      # expect rc 0, plays 0, refusals []
python3 tools/vinegar_run.py run traffic:abort-contact      # Play 1 — the instrumented diagnostic
python3 tools/vinegar_run.py verify-clean                   # expect exit 0 (clean:true, latch none)
```

Expected bounded traces: prepare ok (`helpers 3`, `chunks True`, verified sources 101); Play server+client ready; collector
phases `envelope → prepared → runtime → cleanup`; runtime delivered as 6×256 KiB chunks (`bytes 1,393,473`,
`sha256 94f937af…`), report ~2.3 MB; `result.json` exit 0 or 1 (Plan 3 keeps the assertion fail-closed, so exit 1 with
`generationEvents` is the expected diagnostic outcome), `diagnostic.generationEvents` ≤20, `worldReady` trace present
(moving-only in practice, but the shared run_fixture merge is harmless); Play window < 155 s; abort path clean.
Stop rule: one Play only, no retry, then `verify-clean`, `studiocycle snapshot` compare, play counter 1/3, sealed hash
before/after.

How the three failing cells are judged (after the block is frozen+accepted, not in this Play):
1. **`traffic:abort-contact`** — from `generationEvents`: was the body replaced **contact-correlated** (a `_retire` at
   the cut-in/contact moment: `hasChange`/`progress`/`cutInAt`/`contactDriveAt` aligned with `contacts`/`touches`)? If
   yes → ruling to let the probe allow a **single, contact-correlated** replacement (new generation recorded in the
   report and validated); if unrelated churn (no correlation, multiple events) → product finding against
   `TrafficController._retire` call sites with the event list. No assertion change before that evidence.
2. **`atlas:scout` / `atlas:highway-extension`** — from Plan 4's trace: `fastPathApplied` true → assert-and-skip
   (`rebuild=false`) then the views; false → named fail-fast (`named seed not applied before Play … refusing in-Play
   rebuild`) and apply fallback (a) (prepare disables Bootstrap; one Atlas starter builds the named day world from
   scratch). A 45 s bound catches a hang in tens of seconds; a City+Highway rebuild inside a running Play is never the
   route.
3. **`moving`** — from Plan 2's `worldReady` trace (expected: gate passes in well under 30 s with seed/layout) but the
   capture sidecar will then fail on `lighting.technology` until the §2 ruling lands: the gate fixes the race, **not**
   this contract field. `viewportMatchesPng:true` is still not observed anywhere; the atlas cells are expected to supply
   it once they can capture.

## 7 · Holds respected / not done

No Play; no Studio launch/close/resize; no host input injection; no image reads/screenshots in DSH; no build
(`tools/usg build` never run — sealed build untouched); no asset upload/publication/purchase; no credential value
anywhere; no writes outside `board/reports/team-d-*`, `board/probes/team-d/**`, and the authorised repo branch
(commits `a6ac63a`, `29a626b` only). Only Studio interaction: read-only MCP (`list_roblox_studios`, one read-only
`execute_luau` in Edit) and `winctl get`/`studiocycle snapshot` (read-only). $0 of $0.

## 8 · Blocked on / next step + successor

- **Plan 2 is frozen and ready for the Lead's fresh checker** (`29a626b2b8a13556ac5ee07db6fdb60401ecc6e8`; reproduce
  block §1). On ACCEPT: Plan 3, freeze, fresh checker; then Plan 4, freeze, fresh checker; then D5's one diagnostic
  Play (Leads's approval).
- **Blocked on the Lead's ruling:** §2 `report.lighting.technology` contract (options 1–3, option 1 recommended).
  This is on the critical path for `moving`, `atlas:*` and the B3 look ladder — the diagnostic Play will not make
  those cells pass without it.
- No other blockers; Studio idle at 0/3 Plays and sealed unchanged.

**Evidence files:** `board/probes/team-d/plan2-world-ready-gate-20261101.log` (freeze log + mutations + Studio state),
`lune-unit-plan2-20261101.log`, `python-suite-plan2-20261101.log`,
`lighting-technology-capability-probe-20261101.{json,py}`, `batch7-studio-preflight-20261101.json`; plans
`board/plans/team-d/{moving-world-ready-gate,traffic-generation-instrumentation,atlas-named-seed}-plan-20261101.md`.
