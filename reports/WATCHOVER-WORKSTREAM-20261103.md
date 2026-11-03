# WATCHOVER WORKSTREAM — OpenBell dashboard design on DeepSeek V4.1 Flash (3 Nov 2026, from 14:38 ET)

Owner order (verbatim substance, via Nox 3 Nov ~14:38 ET, `inbox/2026-11/20261103T193800Z-nox-to-dsh-delegator.md`):
OpenBell stops on Claude; the dashboard design work moves to DeepSeek V4.1 Flash on the harness. New workstream beside the FINISH mission
(FINISH keeps first claim on fleet capacity). Every agent pinned DeepSeek V4.1 Flash, no un-pinned spawns. Report progress by board filing.
Claude design lane stood down (it filed the handoff pack at `inbox/2026-11/20261103T194100Z-design-to-nox.md`).

**Pin used for every agent so far:** subagent provider `opencode-go`, model `deepseek-v4.1-flash` (explicit on every spawn).

## Pack
- Handoff: `/home/cayleb/Work/shared-workflows/design/openbell-watchover-handoff/` (`README.md`, `SPEC.md`, `SALVAGE.md`, `phase2-stage-results.json`, `reviews/`).
- Code: worktree `/home/cayleb/workspace/openbell-watchover`, branch `design/openbell-watchover` (cut from unified head `6687b79`; commits `86928bc`, `1c9b117`); pack `design/watchover/`.
- Live OpenBell checkout `/home/cayleb/Work/projects/openbell` is READ-ONLY and never written by this workstream.

## Cold-start assessment (first DeepSeek agent, 14:45 ET; read-only)
- `npm test`: **PASS** 351 tests / 344 pass / 7 self-skip (`test/shell/layout.browser.test.js`, "# server not up on 8792") / 0 fail.
- `npm run check`: **PASS** (tokens drift ok; palette "every rule holds"; 3 non-failing under-target notes: crit word on panel 6.61, red #8E473A on base 2.68, pad-live on slab 1.14).
- `npm run build`: **FAIL** — A20 flat-lint false positive: compiled CSS contains `.transition{transition-property: … box-shadow … backdrop-filter}` emitted because the Tailwind content glob scans plain-JS identifiers (`transition`, `!transition`) in scene modules; `lintCss()` matches bare substrings. dist/ empty. Real build blocker, not a missing fixture.
- Bodies: **3/6 partial**. Parked Sentry done+tested (320 tris); Assembler partial (1144 tris, 16 parts, no test); Orbiter partial + real bug (`FOOT_FRONT_Z` undefined → ReferenceError in `pose.dark && dead`, no test); **Sentinel, Pedestal Brute, Low Walker missing**.
- Bay: core exists (`core/bay.js`, `scene.js`, `camera.js`, `registry.js`, `index.js`); `src/scene/bay/` missing; **app never mounts the scene** (`main.jsx` renders `loader={null}`); latent cap mismatch (scene filters `drawn` by front index; shell `splitFront` caps versions and keeps fixed bodies).
- Renders/measurements missing; styleguide + A1-A30 acceptance missing; RESOLUTION.md/README missing; §20 authored fixture roots missing (only dash-demo + 15 preview variants).
- Solid baseline: tokens + 3 tools, 29 component files (~1,700 L tests), model (derive/freshness/figures/connection/hook, 16 fixtures), scene core 12 files (~1,400 L, 9 test files).

## Plan (P0 first: nothing is measurable until the build emits)
1. **P0 — unblock build (0.5 d):** declaration-level flat-lint (`/box-shadow\s*:/`, `/text-shadow\s*:/`, `/backdrop-filter\s*:/`; keep every other check strong), single-source lint module + MUST-FAIL/MUST-PASS tests; gates: build exit 0, `dist/dash.html` + CSP txt, HTML gzip ≤ 110 KB, `npm run csp`, `smoke.mjs`, `npm test` green. **Maker spawned 14:47 ET (pinned DeepSeek V4.1 Flash).**
2. **P1 — six bodies to SPEC §3/§6.5, test-first per body** (order: Sentinel [critical path], Low Walker, Pedestal Brute, then fix Orbiter bug + test, add Assembler test). Gates: `npm test` green with 6 body tests; each ≤1,500 tris / ≤10 draws; A21 clean; no result number reaches a body; silhouette bounds.
3. **P2 — bay wiring:** import/register six bodies; create `src/scene/bay/` (or rename `core/bay.js`); `main.jsx` lazy loader; one cap authority = shell `splitFront` (spec §4 + F18 RULED: 6 drawn bodies, extra versions keep plates, no body); tiles/rowRef/focus rim. Gates: 6 front + shelf rank at 1920×1080/1440×900; `__bay` tris ≤12k, draws ≤60; pads line up with cells; `?gl=off` DOM text identical (A11).
4. **P3 — grey-tone renders + measurements** into `shots/bodies/` + `renders/<run>/` (all poses, six-body lineup, 64×64 silhouette sheet A23 pairwise Hamming ≥20, all-#3A3E45 batch, full bay at both desktop sizes); measure tris/draws/zero-idle-frames (A15)/scene JS gz/first frame/tone contrast/A29 wall-distance row with the documented tone-lift rule.
5. **P4 — styleguide + A1-A30 acceptance harness** (8 §20 authored fixture roots; every component × state at 1920/1440/1264/390; copy lint A18, flat-only A20 extended to computed styles + scene materials, A2 scan, A9 axe via CDP, A5/A6 perf, A12 crossing, A13 frozen, A24 rungs, A27 hygiene).
6. **P5 — cold review, fixes, `RESOLUTION.md`, README for Team C.** Then stop: Phase 3 is Team C's, Phase 4 the fresh checker, owner grades renders only.

## Decisions
- **Cap authority (assessment D2): resolved from the spec, no Nox ruling needed.** SPEC §4 + F18 (RULED): 6 drawn bodies on desktop; front slot count = 4 + `arena.versions.length` capped at 6; versions past the cap keep a full plate and no body. The shell's `splitFront()` already implements the ruling → it is the one cap authority; the bay must be fed `drawn` + `overflow` from it, and the model's front-index `drawn` flag must not be used as a second cap.
- **Palette notes (assessment D5): accepted as recorded, Nox ack not owner** — keep crit-on-panel 6.61 as a placement rule (crit word never on a card; AlertsBand on base = 7.24 ok); record red fill 2.68 and pad-live 1.14 as accepted spec decisions so the fresh checker does not re-litigate.
- **Honesty (assessment D6):** the green scene smoke tests use six *stub* bodies — never quote them as evidence about the real bodies.
- **Network baselines (D3):** A1/A6/A19 must be re-taken after the scene lazy-import wiring, not before.
- **§20 roots (D4):** stay in-pack under `design/watchover/fixtures/`; no ruling needed.

## Status log
| ET | event |
|---|---|
| 14:38 | Nox move order received by the delegator; workstream opened |
| 14:45 | cold-start assessment returned (above) |
| 14:47 | P0 maker spawned (pinned `opencode-go`/`deepseek-v4.1-flash`), build-lint unblock |
