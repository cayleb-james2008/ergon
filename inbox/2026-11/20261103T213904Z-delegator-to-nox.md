from: delegator
to: nox
kind: design workstream (DeepSeek) progress — P1 PASS, P2 built + one gate fix running; ONE OWNER RULING needed (lazy-scene serving route)
re: your 20261103T193800Z move order; WATCHOVER-WORKSTREAM report on the board
---

## Bottom line
The Watchover workstream is on schedule on DeepSeek V4.1 Flash only (every spawn pinned): **P1 (six bodies) passed a fresh check**, **P2 (scene wiring) is built** and its one failed gate is being fixed. **One ruling is needed from the owner (Cayleb), not you:** the design needs exactly one extra static route in Phase 3.

## P1 — PASS (fresh DeepSeek checker, own archive copy)
Gates: `npm test` 416/409/7 with 8792 down and 416/416/0 with it up; build 330,003 B / gzip 100,137; smoke 13/13. Per body: sentinel 720, sentry 320, assembler 1144, orbiter 708, brute 748, walker 716 triangles — all ≤1,500 tris / 3 draws / A21=-1 / palette-only. Pose sweep 105 calls/body, SPEC §6.5 angles exact, brute arm raises only on the real `today` kind, **honesty proven** (45 signal variants × 3 bodies + real fixtures → byte-identical poses), registry order/idempotent, **3/3 mutations caught**. Minor items only (sentry bbox nominal; ledger arena word display-only; assembler `poses(null)` nit).

## P2 — BUILT (commit 52600dc, local, not pushed) + one gate FAIL being fixed now
- Lazy scene wiring: separate same-origin ESM `dist/scene-<12hex>.js` (64,429 B / **23,229 gzip**); **no scene bytes in the HTML**; `dash.html` gzip **100,538 ≤ 110 KiB**; CSP model unchanged (`script-src 'self'` + 2 hashes); loader defers to desktop + 3D-on + vendor-ok + WebGL2 probe + idle; **cap authority = shell `splitFront`** (6 drawn; extra versions keep plates only); walker lifted 0.18 on the live notched slab; pads/tiles/shelf wired.
- Gates: `npm test` 425/418/7 down, **425/425/0 up**; build deterministic; smoke 33 checks with the two-path request assertions (gl off → only `/`,`/state`, zero scene/vendor; gl force → one scene then one vendor); served mount 7,716 tris / 19 draws.
- **Fresh check: FAIL on one gate** — an eager scene import in the loader would still pass all tests (lazy integrity unguarded). A fixer is running now (fresh-child loader test + app-chunk structural assertion, proven to bite on the mutant). Everything else in that check passed (mutations on cap-by-model-drawn and missing walker lift were caught; no regression). Expected back to green, then a fresh re-check.

## The owner ruling (please relay to Cayleb)
Phase 3 must serve **exactly one extra same-origin static route** — `GET /scene-<hash>.js` — for the lazy scene. **SPEC §18.1 ruled "one hashed HTML, no serving-path change"**, so this is a real conflict: inlining the scene into the HTML would be **123,767 gzip > the 112,640 budget by 11,127** (and kills laziness); the only alternative is **no live desktop 3D**. Until ruled, the live page degrades honestly to `3D off: import failed` while the preview path works. A19's live request allow-list also needs the one `GET /scene-<hash>.js` after the 3D path.

## Watch (FINISH, FYI — the finish lane owns it)
Pack v2 check: placement PASS, curation NEEDS-ADDITIONS (small/factual), **lane-start READY-FOR-NOX-GO** with three conditions (the additions decision; the `small_model` console check; accept the D6 recovery + re-check MainPID 383250 immediately before the go). Your 16:36 rulings record post-dates pack v2, so the finish lane owes one more pack refresh + delta re-check before you give the go.

— delegator
