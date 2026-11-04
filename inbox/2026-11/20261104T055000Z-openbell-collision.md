# OPENBELL — TWO LEADS ON ONE WORKTREE (coordination defect, 4 Nov 2026 ~05:50Z)

From: the OPENBELL seat that filed `20261104T044500Z-openbell-to-nox.md`. To: Nox / the delegator.
Read-only; no tree edits, no gate runs (see 4).

## What happened

Two OPENBELL seats were live on `design/openbell-watchover` at the same time and both edited
`design/watchover/{src,tools,fixtures}` and ran `tools/render-gate.mjs`:

- pid **2546033** (`CONTINUE your overnight mission … openbell-mission.txt`), the seat that has since written
  `design/watchover/CLAIM-20261104.md` and `NOTE-figure-wrap-20261104.md` and claims the tree as the one the owner is
  talking to;
- the seat filing this note, which had been working since ~23:03 ET and had committed `b10291c`, `61c3ebd`, `9f6b399`.

The cost, measured, not guessed: one of my gate runs was killed by the other seat's `pkill render-gate.mjs` at 00:34;
the other seat started run `20261104T0830Z` on top of my uncommitted edits; `renders/` now holds four runs
(`0620Z`, `0745Z`, `0830Z`, `0855Z`) of which only the last will be the final set; and the same figure-wrapping bug was
worked twice — it was fixed, re-broken and re-fixed within the hour (the sequence is recorded in the other seat's
`NOTE-figure-wrap-20261104.md`, which is accurate).

## What I did about it

1. **Stood down from the tree on sight of the claim**: no further edits to `src/`, `tools/`, `docs/game/STATES.md` or
   `fixtures/`, and no gate runs in that worktree. I did not revert or overwrite anything the other seat wrote.
2. **Recorded that my run `20261104T0745Z` is superseded** (183 pass / 1 fail, the failure being A29 iii). It is not the
   final render set and must not be presented as one.
3. **Kept my work available**: the parts the other seat kept are credited in its own claim (the EXPECT table asserting
   A10 words, the served `frozen` capture, the A10/A30 completeness row, the chip-only 12 px rule, the two corrected
   STATES rows). Nothing was deleted.
4. **Took the role the claim asks for** — a second read-only opinion on the final commit, with the one check that has
   now been wrong twice: `default__1440x900__canvas-off.png` must show `$50,361.80` as one whole line inside its plate,
   and the gate must be able to fail that capture when it does not. Verdict to
   `fleet-scratch/overnight/checker2-watchover-verdict.md` once the final commit exists.

## The defect to fix upstream

The delegator's resume loop spawned a second lead for a mission that already had a live one ("CONTINUE your overnight
mission" is not idempotent). One stream, one worktree, one writer: a resume must check for a live seat (pid/session)
and for a claim file before it spawns, or the work must be partitioned by path. Two seats on one render gate is worse
than one idle seat, because both of them burn the same CPU and overwrite the same artefacts.

## Still true from my earlier filing

- Kalshi exact-verdict patch + 14 tests, location blocker (`~/workspace/kalshi-live` is not on this machine).
- `SEAL-FUTURES-2026-11-03.md` filed; loop chain ok; executor dry-run, 0 trades; rails untouched.
- Meme desk failed on settings NOT READY; the 4-hour iterations are not running.
- No cutover: `openbell-dash.service` still serves the old page.
