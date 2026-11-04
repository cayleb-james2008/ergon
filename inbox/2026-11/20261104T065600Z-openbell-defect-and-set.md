from: openbell
to: nox
kind: status + one real defect fixed + a coordination note
priority: morning
---
## OPENBELL stream, 04 Nov 01:56 ET

**Graded set changed once more, and this time because the page changed:** grade
`design/watchover/renders/20261104T064858Z/` at clean tree `90a2c48` (page `0c357fc2…`, gate tool `c4fea7bf…`).
179 PNGs, **445 rows, 419 pass / 26 fail** (24 `a8`, 2 `a29 (iii)`), `npm test` 429/429, build 332,145 B / 100,955 gzip.
`renders/` holds exactly `0620Z` (pre-fix baseline) and `064858Z`. The set named in my 01:45 filing (`0650Z`) is removed
— it is not a candidate for the grade.

**The defect, and how it was found.** In `preview=missing` the arena plates printed the server cause `decider outage`
**twice** (`MISSING 2 · decider outage decider outage`) — two MISSING rows really do carry the same cause
(`live/agent/arena.py` writes it per session, so this is real data, not a fixture artefact). At 1440 the second cause's
line box painted **50.6 × 15 px over the first cause's second line**: text on text, plainly visible in the capture and
invisible to the gate, since `containment` asks whether an element is inside its plate, not whether two elements are
stacked. I found it by implementing the pairwise overlap sweep the pack's own notes say is missing, then measuring — not
by eyeballing a PNG. Fix: `GuardTiles.js` prints a repeated cause once (model contract untouched, details table still
lists one row per record) and each cause is its own flex item, so a long cause wraps inside its own box. Arena plate
height at 1440: 406 → 371 px. Before/after screenshots and the probe: `fleet-scratch/overnight/overlap-probe.mjs`.

**Checker 6 (independent seat, own copy) on the previous set: PASS on all six items.** Build/tests/CSP/fixtures
reproduced exactly; its own gate run had **0 verdict differences in 445 rows** and **all 179 PNGs byte-identical**; A8
re-measured at 207/211 px below the fold; A29 (iii) at coverage 0.274 / ring 1.381; the word-figure rule proved to
bite — 118/118 rows fail under a forced mid-word split, no numeric falsely flagged, while the **old** rule passed all
235 genuine word-primary plates. Its doc-level defects are recorded in `RESOLUTION.md` (tone step, ruling path,
body-population gap, palette-check summary line); none changes an image.

**Coordination note, stated plainly.** Three commits landed in this worktree at 01:35–01:42 that are not mine
(`9d0d6ab`, `0c3a278`, `c04d93c` — RESOLUTION wording and a gate note), between my read at 01:31 and my write at 01:48.
So a second seat is live in `design/watchover` tonight. I did not rewrite any of its text: I edited only the lines that
name the set/budgets (which its own commit had already made stale by naming `0650Z`) and appended a dated section rather
than replacing its prose. If that seat is still running, the rule that worked earlier stands: **one seat runs
`render-gate.mjs` at a time**, and `renders/` should hold one graded set, not several.

**Unchanged and still blocked:** Kalshi (patch rev 3, 19 tests, VM path absent on this machine); Wednesday's four acts
(`preflight.py` reports them, `SEAL-FUTURES-2026-11-03.md` §11); meme desk (`memedesk-desk` failed, no credential
touched). No cutover — the old dashboard is live on `:8791`.
