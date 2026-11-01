# D-4 — Job 16 test matrix, made runnable (2026-11-01)

Companion machine files: `board/probes/team-d/d4-play-plan-20261101.json` (all **51 Plays / 19 Studio openings**
parsed from `docs/verify/2026-10-19/vinegar/SPRINTS.md` and validated against the runner's own `FIXTURES` +
`DESIGN_ORDER`: **0 problems**), `docs/verify/2026-10-19/vinegar/v1.queue` (the 7-entry first queue, unchanged),
`docs/verify/2026-10-19/vinegar/SPRINTS.md` (per-Play commands, skip rules, visual flags).
Pinned inputs: street candidate **`b67abdd`**, test place **`c6651793…`/2,403,691 B**, runner place copy refreshed to
it (`~/.var/…/data/usg/usg-desktop.rbxlx`, 08:59 ET). Execution tooling: `tools/vinegar_run.py` (queue/run/smoke/
preflight/verify-clean) — **never Studio-launching**, one Play per invocation, 3 Plays per Studio process.

## 1 · The matrix — what each cell is, what it must produce, what blocks it now

Cell classes: **A** offline/in-Studio behaviour · **B** native fixture Plays · **C** device · **D** two clients ·
**E** persistence across rejoins · **F** receipt flows · **G** audio/performance · **H** packaging.
"Evidence" is the acceptance object — if it cannot be produced, the cell is **FAILED**, not "probably fine".

| ID | Test | Exact invocation | Evidence the cell must produce | Status now / blocker |
|---|---|---|---|---|
| A0 | Offline baseline on the exact candidate | `TMPDIR=~/.cache/usg-tmp python3 tools/usg test`; `python3 -m unittest discover -s tests/tools`; `python3 tools/store_check.py` | quoted totals (specs/run counts), exit 0s, and `sha256sum build/release.rbxlx build/test.rbxlx` = `84324d43…` / `c6651793…` | **PASSED 2026-11-01 09:23–09:36 ET on the b67abdd tree**: Lune **914 passed / 0 failed (125 specs)**, Python suite **273 tests OK** (39.2 s, exit 0), store_check **3/3 PASS (0 problems)**, three exit 0s, build hashes after the suite still `84324d43…` / `c6651793…` (`a0-offline-baseline-20261101.log`). Note: docs' 926/128 figures belong to a later tree (`1bef02f`/`HEAD`), not this candidate; quote 914/125. |
| A1 | In-Studio behaviour: the studio specs | **`tools/studio.py sync --test` then `tools/studio.py tests`** from the pinned tree (NOT `usg test`, which rebuilds) | per-spec JSON report; exit 0; place hash `c6651793…` | **GREEN (evidence complete 2026-11-01 13:01 ET; awaiting Lead declaration).** Spec checker PASS (`board/reports/team-d-a1-highway-spec-checker-verdict.md` on prep `1abe303`). Run 1 candidate sources (prep branch @1abe303, base b67abdd): sync `{"synced":101}` rc0; tests **rc0, 11/11 modules PASS**; `highway.spec {passed:2, seedsPassed:100, expectedTrafficCount:54, minimumTrafficSpacingSlots:0.6694870414212347, roadSamples:3142, groundSamples:5}`; sealed hashes identical before/after (`84324d43…`, `c6651793…`). Run 2 post-merge main `de361ba` (merge of 1abe303; only the spec changed vs 8dfb204; no remote/no push): same clean result. Logs: `a1-in-studio-final-20261101.log`, `a1-in-studio-postmerge-main-20261101.log`; summary `a1-official-runs-20261101.md`. **B1 unblocks on Lead's green declaration.** |
| A2 | First five minutes in a real client | `python3 tools/vinegar_run.py run first-fare` / `run moving` | runner JSON + HUD/hint/result/phone-layout screenshots; note it is not a human judgement | **BLOCKED-ON-OWNER-WINDOW** (needs refreshed place open) |
| B1 | V1 baseline + cadence (10 Plays, 4 openings) | `queue … v1.queue` / direct runs; 3 Plays per open | per-Play exit 0/1/2/4/5; JSON per Play; verify-clean; hostcheck quiet | **RUNNING — chunk transport PROVEN (`traffic:abort-contact` delivered runtime via 6 chunks, 1,393,473 B, sha 94f937af…; report 2,295,405 B); Play 1 failed on scenario finding `HandlingServer:165 physical generation retired/replaced` (fail-closed, abort clean).** Play 2 = direct `run atlas:scout --seed 2134783819`: **exit 1, `TimeoutError: collector did not receive view-001 within 600s`** — prepared ok with `namedSeed=2134783819`, Play started, then hung with no runtime/views (collector phases prepared+cleanup only); likely the named-seed rebuild path `WorldSeed.ensure → World.generateSafe` on an already-built world (atlas prepare leaves Bootstrap enabled) never returns; fixture/tooling finding, instrumentation needed. Play 3 not attempted (stop rule). Plays 2/3 used, Studio alive, verify-clean clean, sealed `84324d43…`/`c6651793…` unchanged; `viewportMatchesPng` still not observed. Records: `b1-traffic-play1-generation-retired-20261101.md`, `b1-atlas-seeded-timeout-20261101.md`, logs `b1-direct-opening-20261101.log`, `b1-atlas-seeded-opening-20261101.log`. |
| B3 | Look ladder (12 Plays, 4 openings) | SPRINTS B3; every entry a **single run** with `--visual-flags` | look-metric thresholds + independent critic ≥4 on every rubric category against `docs/ui-research/anchors/anchors_v1.png`; critic name/value recorded, never self-graded | **BLOCKED-ON-OWNER-WINDOW** + independent critic seat (Lead spawns) |
| B4 | Feel & performance (6 Plays, 2 openings) | SPRINTS B4 | frame time/memory/part counts measured on the real GPU; phone/tablet budget estimates | **BLOCKED-ON-OWNER-WINDOW** |
| C1 | Physical phone play (incl. leave/rejoin with progress) | owner's score sheet `docs/launch/OWNER_PLAYTEST.md`; join the game's private place on the phone | **his filled sheet** (1–5 per axis, device model, screenshots); Studio emulation does not count | **BLOCKED**: the existing private place is **version 7 / old hash** (`5014db1f…`), not the candidate; testing the candidate on a phone needs the candidate uploaded as a new version first — that is the owner's upload yes (checklist D1/D2) plus the Open Cloud key. Then ~30 min of his time |
| C2 | Physical tablet play | as C1 | as C1 | same as C1 |
| D1 | Two clients at once | local server + two clients under Vinegar/wine (SPRINTS D2 sprint: `run` list) | two client logs with **distinct userIds**, one server log, a synced moment both see; if wine cannot, the run is labelled **SIMULATED** with the attempt's exit evidence (the plan allows it) | **BLOCKED-ON-OWNER-WINDOW** (needs a Studio opening; 12 Plays / 4 openings = the V5 GPU sprint) |
| E1 | Live persistence across rejoins | `tools/usg luau tests/live/persistence.luau --version N` | script output + DataStore round-trip + `PROFILE_VERSION` unchanged (6) + second join shows same profile | **BLOCKED**: needs a **Saved version N of the candidate** + Open Cloud key (owner upload yes, P9 key) |
| E2 | Rejoin simulation (offline twin) | `./.tools/bin/lune run tests/playtest/rejoin.luau` (pinned b67abdd tree) | run output, labelled **simulated**, not live | **PASSED 2026-11-01 12:14 ET**: exit 0; "saved: CR 4918 L3 …" / "rejoined: CR 4918 L3 …" / **OK: nothing lost across a mid-shift leave** (`offline-twins-20261101.log`). Simulated, not a live rejoin. |
| F0 | Locate/record the purchase-prompt switch state | grep the prompt path (`Monetization.luau:471,480,446`) + the place's own state | the exact constant/property that disables prompts, quoted; both states proven by a toggle run | partly offline (code quote); toggle proof needs Studio |
| F1 | Receipt flow with prompts in intended final state | `tools/usg luau tests/live/receipts.luau --version N` | receipt log per purchase: `PurchaseId` granted **once**, retry does not double-grant, no grant before profile load, rollback recorded; then prompts restored to the state Cayleb approved **and that restoration proven** | **BLOCKED**: saved version + key (as E1) **and** a real 39 R$ purchase needs his **separate Robux yes** (`PUBLIC_FLIP_CHECKLIST` D5) |
| F2 | Store/receipt offline twin | `./.tools/bin/lune run tests/playtest/store.luau` (pinned b67abdd tree) | run output, simulated label | **PASSED 2026-11-01 12:14 ET**: exit 0; 6 prompt calls recorded; stale key "CreditStack" correctly refused ("That item is not available yet."); monetization extras printed (`offline-twins-20261101.log`). Simulated — no prompt UI exists offline. |
| G1 | Audio reel: 27 runtime files listened | `tools/audio_review.py` reel + owner's listening | reel list with hashes + **listening actually performed** (owner and/or independent critic); "listening NOT PERFORMED" stays in the label if not done | reel can be generated offline; listening needs owner (~10 min, attach to the Studio window) |
| G2 | Performance budgets inside Studio | measurements during A2/B4 | measured numbers vs documented caps, host named | **BLOCKED-ON-OWNER-WINDOW** |
| H | Packaging + his note | — | release hash + version bump + one plain paragraph (what he sees / what to poke / known-rough) + prompts-state note + rollback line | after the cells; not startable |

## 2 · The runnable native plan (51 Plays / 19 openings) — verified

Parsed from SPRINTS.md and checked against the runner: B1 10 · B2 4 · B3 12 · B4 6 · C1-sprint 7 · D2-sprint 12 =
**51 Plays**, in **19 Studio openings** (B1 4, B2 2, B3 4, B4 2, C1-sprint 3, D2-sprint 4). The machine plan has, per
Play: sprint, opening, kind (queue/smoke/run), fixture, exact command. Checks run: every fixture in the plan exists
in `vinegar_run.py` `FIXTURES`; every `run` fixture is in `DESIGN_ORDER`; **0 problems**. Queue plays 1–7 are
covered by `v1.queue` (7 entries; runner pauses at 3 Plays and continues with `--resume`).

Execution shape for the window: one queue invocation per opening (or one `run`), then **owner closes Studio
(Don't Save), reopens on the runner place** — 3 Plays per Studio process, hard ceiling; `verify-clean` at the end
of every sprint. The window cannot be one continuous sitting for the *whole* 51 Plays (19 reopen cycles ≈ an
afternoon); the first sitting should take **opening 1 (A1/A2 + queue Plays 1–3)** — see D-5.

## 3 · Blockers, named (each with the owner's smallest act)

| # | Blocker | What it blocks | Smallest act |
|---|---|---|---|
| W1 | Studio must be opened on the runner place (`usg-desktop.rbxlx`) — currently a baseplate is loaded; the team may not drive the open Studio until Cayleb confirms it is his | A1, A2, all B/G cells, F0 toggle | D-5 one-step (File → Open the named file, Edit, say go) |
| W2 | **Candidate version upload** (Owner yes per `PUBLIC_FLIP_CHECKLIST` D1/D2 + Open Cloud key) | E1, F1, C1, C2 (physical device can only join an uploaded version; the live private place is version 7, not the candidate) | one-line "yes, upload the candidate as a saved version" + key via secure route — **not yet in the ASK**, Lead to route |
| W3 | **39 R$ real purchase** for the receipt cell | the real-money leg of F1 (the rest of F1 can run simulated/offline) | his separate Robux yes at that step |
| W4 | Owner device time + listening time | C1, C2 (phone/tablet), G1 (audio reel) | ~30 min phone/tablet + ~10 min audio reel, ideally attached to the Studio window |
| W5 | Independent critic seat (B3 look gate) | B3 certification | Lead spawns a fresh checker/critic at the gate; never self-graded |
| W6 | E2/F2 offline twins | runnable now, no owner | agent (queued behind A0 this batch) |

## 4 · What this batch actually executed vs prepared

- **Executed:** candidate rebuild + pin (D-2); runner place refreshed; hostcheck/status/preflight (D-1); A0 offline
  baseline launched on the pinned tree. No Play, no Studio change, no upload.
- **Prepared:** the 51-Play runnable plan + per-cell acceptance objects above; D-5 window runbook; the exact owner
  asks (W1/W2) and the routing they need.
- **Not started:** every cell that needs the window or an owner act. Nothing here is reported as passed.
