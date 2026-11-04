# DESIGN / REITERATION REMAINDER — PICKUP PACK (Nox order 20261103T231027Z)

Assembled 3 Nov 2026 ~19:30 ET, read-only. Engine for the continuation: DeepSeek V4.1 Flash only.

## Bottom line
- **Do NOT restart the Watchover workstream — it is already live on DeepSeek.** P0, P1 and P2 have passed (gate plus independent
  checkers). The next work is **P3–P5**: grey-tone renders + measurements, styleguide + A1–A30 acceptance, cold review +
  `RESOLUTION.md`/README for Team C.
- **P3 was started and its maker died** in the 2 h 37 m dormancy: only untracked `tools/render-lib.mjs` (18 KB) and
  `tools/fixtures/` (8 KB) exist; no `shots/` directory, no measurements. **P3 must be re-run**, not resumed from a claim.
- **One owner ruling blocks live 3D**: Phase 3 must serve one `GET /scene-<hash>.js` route, and SPEC §18.1 ruled against a
  serving-path change. Inline would be 123,767 gzip > 112,640. Until ruled, the live page degrades honestly to
  `3D off: import failed`.
- The Lumen/warehouse view-layer remainder is **archived, not running**: `design/board-v2` stands down at `d620299`, and the
  RHL front-end reiteration exists as a mission order + diagnosis only (docs branch), owned by a session whose view layer is
  frozen and who has not started building.

## What this is
Two related but separate design remainders:
1. **OpenBell watch-over ("Dry Dock", the name never appears on screen)** — a replacement for the OpenBell dashboard
   (`live/dash.html`, served by `live/dash.py`): one live 3D machine body per account on an inspection pad in one concrete bay
   (React + Three.js + Tailwind), plus a DOM "twin" plate per account that carries every state in words, numbers and ages.
   It is a watch-over, not a game. Read-only, GET-only against the existing dashboard; body cap 6 on desktop via the shell's
   `splitFront()`; flat colour, no neon/gradients/glow/blur/emissive/translucency; motion is a signal (zero idle frames); real
   data only, synthetic labelled.
2. **RHL warehouse board view-layer reiteration** — rebuild the board's front end from zero on the verified functional core
   (owner: "real components need to be implemented"; the previous front end is baseline evidence to beat, not a base to edit),
   plus the **Lumen archive** and the earlier `openbell-game` archive.

## Current state (evidence) — Watchover
| Item | State |
|---|---|
| Worktree | `/home/cayleb/workspace/openbell-watchover`, branch `design/openbell-watchover`, cut from unified head `6687b79` |
| Tip | `2f281b4` (`p2fix`), tree clean **except** `?? design/watchover/tools/fixtures/` and `?? design/watchover/tools/render-lib.mjs` (the dead P3 partial) |
| Commits | `1c9b117` WIP Phase 2 (unreviewed) → `6a1dc93` P0 → `1ff71f0` P1a → `09f94ec` P1b → `52600dc` P2 → `2f281b4` P2 fix |
| P0 (build unblock) | `tools/flat-lint.mjs` declaration-level shadow/filter checks + 12 MUST-FAIL/MUST-PASS tests; smoke made phase-honest. Build exit 0: `dist/dash.html` 330,003 B, **gzip 100,137 ≤ 112,640**; CSP 2 blocks; smoke 13/0; `npm test` 363/356/7/0. **PASS** (fresh checker; one LOW: uppercase `BOX-SHADOW:` passes both lints) |
| P1 (six bodies) | Sentinel + Orbiter `FOOT_FRONT_Z` fix + Assembler test (`1ff71f0`, 395/388/7/0); Low Walker 716 tris + Pedestal Brute 748 tris + six-body barrel `registerAll` (`09f94ec`, 416/409/7/0). **PASS** (fresh check): sentinel 720/22 parts, sentry 320/8, assembler 1144/16, orbiter 708/22, brute 748/11, walker 716/10 — all ≤1,500 tris / 3 draws; 105 pose calls per body, no throw; honesty proven (45 signal variants × 3 bodies, 0 JSON diffs); 3/3 mutations caught |
| P2 (bay wiring) | `52600dc`: separate same-origin ESM asset `dist/scene-<12hex>.js` (64,429 B / 23,229 gzip), no scene bytes in HTML, deferral gate, `registerAll()` before apply, **cap authority = shell `splitFront`** (6 drawn, overflow plates only), pads/tiles/shelf, walker lifted 0.18 on the live notched slab. `dash.html` 330,871 / **gzip 100,538**; served mount 7,716 tris / 19 draws |
| P2 verdict | First fresh check **FAIL**: an eager `import './index.js'` leaked the scene into the page (108,800 gzip) and no test caught it. Fix `2f281b4`: `test/scene/loader.lazy.test.js` (fresh-child probe + static-source-graph guard) and a build-level app-block assertion. **Re-check PASS**: original mutant + static-import, top-level-`await import`, body-only mutants all caught. Final build 330,921 B / **gzip 100,590 ≤ 110 KiB**; `npm test` **429/429/0/0 up**, 420/9/0 down; smoke 33 |
| Owner ruling open | **Phase 3 needs one `GET /scene-<hash>.js` route**; SPEC §18.1 ruled against serving-path changes; inline = 123,767 gzip > 112,640, so the route is genuinely required. Filed to Nox 16:39 (`inbox/2026-11/20261103T213904Z-delegator-to-nox.md`); **not ruled** as of 19:30 |
| Rulings already settled | cap authority = `splitFront` (SPEC §4 + F18); palette notes accepted (crit-on-panel 6.61 as a placement rule, red 2.68, pad-live 1.14); honesty (green smoke tests use six **stub** bodies — never quote as evidence about real bodies); A1/A6/A19 network baselines re-taken after lazy wiring; §20 fixture roots stay in-pack |
| Handoff pack | `/home/cayleb/Work/shared-workflows/design/openbell-watchover-handoff/` — `README.md` (11 KB), `SPEC.md` (102 KB, 584+ lines, §18 all ruled, §19 48 findings, §20 unified-base deltas), `SALVAGE.md` (14 KB), `phase2-stage-results.json` (82 KB), `reviews/` |
| Workstream record | `board-push/reports/WATCHOVER-WORKSTREAM-20261103.md` (18,240 B) — identical copy at `lanes/board/reports/WATCHOVER-WORKSTREAM-20261103.md`; status log ends at 16:52 (P3 maker spawned) |
| Stand-down filing | `lanes/board/inbox/2026-11/20261103T194100Z-design-to-nox.md` — Claude lane stopped; Fable was an explicit pin (36 agents; ~2.6 M subagent tokens for the Phase 0–1 spec run); `wf_6e0ea722` and `wf_52217147` stopped 14:33–14:35, follow-on Opus run `wf_a479538f` killed; no transcript written after 14:36. Live OpenBell and RHL/Ergon were never modified |

## Current state (evidence) — reiteration / archive
| Item | State |
|---|---|
| Reiteration prep | `/home/cayleb/workspace/frontend-reiteration` (Ergon clone), branch `prep/frontend-reiteration` @ `95d092ca` (2 Nov 17:04) — **docs only**: `docs/frontend-reiteration/{README.md, DIAGNOSIS.md 22 KB, REITERATION-MISSION-ORDER.md 36 KB, checker-instruments/, workflow-amendments/}` |
| Mission | Rebuild the RHL board view layer from zero on the verified core; own `src/components/**`, `App.jsx`, `main.jsx`, styles, `design/**`, `docs/DESIGN.md`, e2e tests; data/ops/hosting stay read-only to it; branch `design/board-v2` cut from `build/rhl-board-rebuild` @ `7817250`; no merges, pushes or deploys; N7 stands (19 lines, no page scroll at 1920/1366/1280; 390 usable) |
| Contract | dark default and primary graded mode; solid surfaces, translucency only in menus/popovers/mobile sheet ("if the first thing anyone notices is the blur, it is too much"); no visible board heading; initials only; no passcode; key in fragment; machine hooks exact; `npm test` 117/117 untouched |
| `design/board-v2` | `/home/cayleb/workspace/board-v2`, branch `design/board-v2`, tip `d620299` **stand-down**: "game-style restart of the warehouse board aborted (owner scope fix); Phase 0 salvage kept"; parent `0dfd2d0` Lumen-scrapped archive note; before that hero rounds 4–6 (`58efd45`, `42c85a0`, `5969562`) with connection honesty, mobile sizing, key sweep, wall-band type. Clean. Tag `archive/lumen-board-v2-round6` |
| `lumen-polish` | `/home/cayleb/workspace/lumen-polish`, branch `lumen-v2-polish-r2` @ `684110bd` (2 Nov 13:38), `...origin/team` **behind 1002**, untracked `lumen-polish-r2/` |
| Scope ruling | Nox 3 Nov 11:22: game-style/3D is **OpenBell only**; Ergon/RHL warehouse board and the DSH face keep the clean professional library look (shadcn/ui + MUI + Radix, flat, no gradients); any wider order rescinded. The Lumen visual pack is **scrapped** for the DSH face (PENDING VISUAL ITEM: restyle to professional library; mission-or-successor question open) |

## Running now
- **The Watchover workstream is running on DeepSeek** — but not on this seat: it is the Lead/delegator session's workstream, with
  every agent pinned `opencode-go/deepseek-v4.1-flash`. **Nothing of it is executing as of 19:30**: the P3 maker died; no render
  harness, no Chromium, no build process for `design/watchover` is up. There is no `shots/` output.
- Nothing of the reiteration is running: no `design/board-v2` dev server, no build. The board's `vite preview` processes belong to
  other sessions and are not this workstream.
- Live OpenBell dashboard keeps running untouched on `127.0.0.1:8791` (its own units; read-only to this work).

## Owed / decisions
1. **Owner ruling — Phase 3 serving route.** One `GET /scene-<hash>.js` (static, same-origin, hashed) vs SPEC §18.1. Without it,
   live 3D cannot load and the page must keep saying `3D off: import failed`. This is the single blocking decision.
2. **P3 re-run** — grey-tone + colour renders of each body in every §6.5 pose, six-body lineup, shelf rank, 64×64 A23 silhouette
   sheet (pairwise Hamming ≥20), all-`#3A3E45` grey batch, full bay at 1920×1080 and 1440×900, A29 wall-distance frames; measure
   tris/draws, A15 zero idle frames over ≥120 s, scene gzip, first frame, tone contrast, A20 saturation ≤44 %/no lightness above
   `#D8DCE2`, Hamming, A29 coverage with the tone-lift rule. Must not change shipped bytes.
3. **P4** — styleguide page (every component × state at 1920/1440/1264/390) + A1–A30 acceptance harness over the eight §20 authored
   fixture roots (copy lint A18, flat-only A20 on compiled CSS **and** computed styles **and** scene materials, A2 scan, A9 axe via
   CDP, A5/A6 perf, A12 crossing, A13 frozen, A24 rungs, A27 hygiene).
4. **P5** — cold review (honesty/security + craft/use), fixes, `design/watchover/RESOLUTION.md`, `README.md` for Team C. **Then stop:**
   Phase 3 is Team C's, Phase 4 the fresh checker's, and the owner grades **renders only, never source**.
5. **Reiteration start** — no one has started. It needs Nox's go, a fresh base decision (`7817250` in the order vs `d1f9ba9`/
   `455a9da` live now vs `d620299` stand-down salvage), and the parked mobile patch (`round9-mobile-sizing-and-keyboard-sheet.patch`)
   either adopted or formally dropped.
6. **Lumen archive** — preserved and tagged; do not revive it for the DSH face or the warehouse board.
7. **Loose ends**: uppercase `BOX-SHADOW:` lint hole (pre-existing, LOW); sentry bbox vs nominal; assembler `poses(null)` nit.

## Next steps to continue
1. **Read `WATCHOVER-WORKSTREAM-20261103.md` and the handoff README first**, then `SPEC.md` §2, §14, §16–§18. Do not re-derive the
   plan; do not restart the assessment.
2. Take the P3 decision out of the critical path: ask Nox/owner for the one-route ruling (one line, with the 123,767 gzip number).
   In parallel, re-run P3 from a clean tree — the untracked `render-lib.mjs` is a partial harness, not a result; inspect it, then
   either finish it or delete it, and write the harness under `design/watchover/tools/`.
3. Re-run the gates on the current tip before measuring (`npm test`, `npm run check`, `npm run build`, `npm run csp`, `npm run smoke`),
   serving **your own build on your own port** — a stale server on a fixed port silently measures an old build (trap from the pack).
4. P3 → P4 → P5 in that order; commit each with its evidence; then hand off to Team C and stop.
5. Reiteration: do not start until Nox's go; when started, one writer per file, and pick the base deliberately (record which).

## Constraints / do-not
- **Do not restart the Watchover workstream** and do not spawn Claude (weekly limit hit; Fable banned fleet-wide). Every agent
  pinned `opencode-go/deepseek-v4.1-flash`; no un-pinned spawns; no model fan-out.
- **Live OpenBell checkout `/home/cayleb/Work/projects/openbell` is READ-ONLY** — units run from it and nightly jobs commit into it.
  Never write `live/`, `meme/`, `bt/`, `research/`, `tools/`. No serving-path or CSP change without the owner.
- Flat colour only: no gradients, neon, glow, bloom, blur, emissive, translucency or colour ramps in CSS, SVG, canvas, shaders or
  textures. No game mechanics, no "Ultron" on screen, no flavour copy — labels, numbers, units and server text only. Real data
  only; synthetic is labelled `SYNTHETIC` on every surface.
- Owner grades rendered pages, never source. Do not report the face/look visually done off a lint pass.
- Never `pkill -f` / `pgrep -f` by pattern (it matches its own shell); kill by PID. No deploys, no kills for this pickup.
- Do not touch the frozen view layer's files from the board lane; one writer per file. Do not revive Lumen.

## Where things live
- Code/worktree: `/home/cayleb/workspace/openbell-watchover` (`design/watchover/`, branch `design/openbell-watchover`).
- Spec/pack: `/home/cayleb/Work/shared-workflows/design/openbell-watchover-handoff/` (`README.md`, `SPEC.md`, `SALVAGE.md`,
  `phase2-stage-results.json`, `reviews/`); live copy `openbell-watchover/docs/game/SPEC.md`.
- Workstream record: `board-push/reports/WATCHOVER-WORKSTREAM-20261103.md` = `lanes/board/reports/WATCHOVER-WORKSTREAM-20261103.md`.
- Stand-down filing: `lanes/board/inbox/2026-11/20261103T194100Z-design-to-nox.md`; ruling request (route): `.../20261103T213904Z-delegator-to-nox.md`.
- Reiteration prep: `/home/cayleb/workspace/frontend-reiteration/docs/frontend-reiteration/` (mission order, diagnosis);
  reference pack under `docs/design/reference/`.
- Archive: `/home/cayleb/workspace/board-v2` (`design/board-v2`, tag `archive/lumen-board-v2-round6`); `/home/cayleb/workspace/lumen-polish`;
  `/home/cayleb/workspace/openbell-game` (`design/openbell-game-ui`, earlier docs-only worktree).

## Open questions
- Who owns the P3 re-run now — the Lead's session continues it, or this continuation starts it? (Do not run two makers on one tree.)
- The one-route ruling: granted, refused (accept honest 3D-off live), or deferred to Phase 3 as a Team C precondition?
- Is the untracked `render-lib.mjs` salvageable, and does it match the pack's measurement definitions (A15/A22/A23/A29)?
- Reiteration base: `7817250` as the order says, the live `455a9da`, or the `d620299` salvage? The order predates rounds 7–9.
- Does the parked mobile patch belong to the reiteration (owner critique) or to this board lane (it was written here, uncommitted)?
- DSH face restyle to the professional library look: mission or successor? (Open since 3 Nov; the Lumen pack is scrapped.)
