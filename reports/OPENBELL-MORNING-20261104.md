# OPENBELL — morning record, 4 November 2026

Stream lead: OPENBELL (`opencode-go/deepseek-v4.1-flash`). Orders: `20261104T034900Z-nox-to-delegator.md`,
`20261104T035300Z-nox-to-delegator.md` (item 3). Night ran 23:05 → 01:40 ET. Two seats ended up working this stream
(see "Honest record"); the artifact below is the one tree, one branch, one render set.

---

## 1. Watchover dashboard — Phase 2 repaired, re-gated, **not** cut over

**DONE, proven by a render set and three independent checker verdicts.**

| what | evidence |
|---|---|
| The defect the first render set hid | A fresh checker reproduced the build, tests, fixtures and gate numbers, then found the **default 1440 capture showed `$50,36` for `$50,361.80`** — four primary figures cut by the next card. The gate passed them because A7 only compared `scrollWidth` to `clientWidth`. |
| The fix | Commits `3e52d34` … `4d4be79`: at ≥760 px the 48 px glyph sits in the plate header and each fact prints its label above its value, so the value line gets ~192 px at 1440 and ~272 px at 1920. `$50,361.80` is **168 px, one line, 46 px inside its plate**. The 1920 lane was also wrong before (facts row 33 px too wide, ages painted into the gutter between plates) — fixed by the same rule. Phone (<760 px) untouched. |
| The gate | New rows per capture, every size, canvas off and on: `containment` (nothing painted outside its plate), `figures` (numeric figure one line; a word at most one line per word; ink and box inside), and `a8 plates in viewport` at 1440×900. Each was proven to fail by breaking the layout on purpose in a checker's own copy. Also closed: A29 (i) chip rule, `a10/a30 completeness` (118 expected captures), `a30 image inventory` (179/179 hashed), A29 (iii) tonal bands measured rather than asserted, A29 (ii)'s label, and a layout-settle wait (one capture had read 436 px in a 390 viewport on a mid-layout measurement). |
| The graded set | `renders/20261104T0650Z/` — 179 PNGs, **445 rows, 419 pass / 26 fail**, containment 118/118, figures 118/118 (frozen included), STATES words asserted, `npm test` 429/429, build 332,063 B / 100,924 gzip (budget 112,640). `dirty: false`; results record commit, build sha, tool sha and `dirty_paths`. |
| The 26 failures | **2 are A29 (iii)**, scoped to the 6 front bodies as the spec words it: contrast 6/6 below 1.8:1 (1.35–1.51), coverage 4/6 below 0.60 (6/6 as an OR), adjacent tonal bands 6/6 below 1.45:1 (min 1.23:1). **24 are A8's viewport clause**, one per state including `frozen`: at 1440×900 the shelf row ends 203–298 px below the fold while the front row's top passes its ≤500 budget. §5.3's stage formula (396 px at 44vh) needs ~1,055 px of height, so A8 and the stage rule conflict on a 900 px window. **Not a regression** — `9f6b399` measures 236 px below. |
| Known cosmetic | the shipped `notes.md`'s "what this gate does not evaluate" sentence predates the word rule (it says "every primary figure one whole line"); corrected in `c04d93c` for future runs, and the row's own detail text in the pass table is accurate. |
| Cutover | **None.** Live page `849e2c1c…` (219,311 B) ≠ pack `e641db13…`; `openbell-dash.service` still runs from `~/Work/projects/openbell`. The old dashboard stays live until you grade. |

**BLOCKED ON OWNER:** grade `renders/20261104T0650Z/` (`notes.md` leads with the result; the full table follows).
**BLOCKED ON NOX (ruling):** `board/reports/openbell-watchover-ruling-request-20261104.md`, four addenda:
A29 (iii) contrast basis (the grey render token is fixable — two HSL steps reaches 1.96:1; the spec's failure rule
names body hexes only), A29 (iii) coverage bar, the adjacent-band clause (needs new T1 hexes or the 5.1 wording), and
A8 versus the stage formula at 900 px.

## 2. Kalshi exact-verdict fix — packaged, tested, **blocked on location**

`~/workspace/kalshi-live` does not exist on this machine and no Kalshi tree here parses checker verdicts by prefix
(`live/kalshi/decide.py` decides from prices; the guard reads `verdict.json` as JSON).
`fleet-scratch/overnight/kalshi-exact-verdict/` carries `verdict.py`, 16 tests, `PATCH.md` (rev 2) and
`apply_exact_verdict.py`, which requires `--shape bare|labelled`, refuses to guess, rewrites only the verdict
predicates, compiles the result and keeps a backup. Proved on copies of the two real hazard sites: the same 7-line
checker output scores **6 passes** under the old prefix rule and **1 pass + 1 miss + 5 unreadable** under the exact
rule. Baseline Kalshi decide + rails: **62 passed**, unchanged. No live orders, no rail touched, no credential read.

**Why it matters:** the executor's own `enabled.json` records an arming verdict as `PASS-WITH-DEFECTS … safe to arm:
yes` — exactly the shape a prefix rule reads as a pass.

**BLOCKED ON NOX:** run the applier on the Nox VM against `~/workspace/kalshi-live/decide.py`, then its own suite.

## 3. Futures desk — loop live for Wednesday, seal filed and corrected twice

`SEAL-FUTURES-2026-11-03.md` filed, then corrected where the record was wrong:
- §7: the executor is **not** dry-run. The running process reads its own `exec_config.json` (`mode: live`,
  `venue: tradovate`, sha `4e6969d4…`); the dry-run file first read is a different copy used only for the desk config.
  `mode` is an attestation input, not an execution switch.
- §9: the halt did **not** carry into 4 November — the day rolled at 00:42:03 with `halt: null`. Wednesday's chain is an
  expired enable (`disabled:expired`) plus a **day-scoped** attestation (`liveloop.py:232` → `attest.usable(path,
  self.day, pkg_sha)`): two procedural gates, not three. **A valid enable arms a real broker path.**
- §8: two consecutive Wednesday cycles fired on schedule (`00:07`, `01:07`), `decision HOLD`, `chain ok`.

**BLOCKED ON OWNER/NOX for Wednesday:** fresh day enable (`go_id` + expiry), fresh five-leg attestation, different-seat
inspect, R11 record inside 09:00–09:15 ET. Nothing is armed; `trades 0`, `placed []`.

## 4. Meme desk — state recorded, nothing to chase here

`memedesk-desk.service` failed (exit 78 `CONFIG`), retried every 2 min: `JUP_API_KEY empty FAIL`, three `WARN`,
`NOT READY`. `status.json` still `desired: "run"`, `hb_age null`. **No 4-hour iteration unit exists on this machine**
(only `memedesk-sync.timer` at 2 min and `memedesk-refresh` at 15 s), so the 4-hour iterations are Nox-side. No
credential value read, written or invented.

---

## Honest record of my own mistakes

1. My first Watchover repair dropped `nowrap` from `[data-value]`: it returned the page width by splitting
   `$50,361.80` over four lines (`$50 / , 36 / 1.8 / 0`), and the box-based counter passed it. Preserved in
   `design/watchover/NOTE-figure-wrap-20261104.md`.
2. My commit message on `3e52d34` claimed "A8 and A29 hold". A29 holds; A8 does not — the clause had never been
   measured. Corrected in `64b8d49` and reported as 24 failures.
3. I twice wrote a rule that a box measurement could not enforce: first `clipped` (border boxes miss ink), then a
   word-figure rule that let wrapped prose pass. Each time an independent seat caught it; the second fix (`4d4be79`)
   is the other seat's and is better than mine.
4. A **duplicate OPENBELL lead** was spawned at 00:00:47 ET while this session was mid-work, and it kept working this
   stream: we collided on files and gate runs, and my `pkill` killed one of its runs. Both seats' work is in the final
   tree, with the collision recorded in `fleet-scratch/overnight/CLAIM-watchover-20261104.md` rather than smoothed over.

## The smallest acts, in order

1. Grade `renders/20261104T0650Z/` — the old dashboard stays live until you do.
2. Nox: the four A29/A8 rulings + the Kalshi apply on the VM.
3. Wednesday morning: enable + attest + inspect + R11, or nothing trades (and nothing should without them).

---

## CORRECTION appended by the OPENBELL lead seat (04 Nov 01:55 ET) — the set to grade is `064858Z`, and one real defect was fixed

The 01:40 correction above names `20261104T0650Z`. That set was superseded one change later and is now removed, so
**grade `design/watchover/renders/20261104T064858Z/`** (clean tree `90a2c48`, page `0c357fc2…`, gate tool `c4fea7bf…`):
179 PNGs, **445 rows, 419 pass / 26 fail**, `npm test` 429/429, build 332,145 B / 100,955 gzip. `renders/` holds exactly
two directories: `0620Z` (the pre-fix before/after) and `064858Z`.

**What was actually wrong, and why the gate could not see it.** In the `preview=missing` state the arena plates printed
the server cause `decider outage` **twice** — `MISSING 2 · decider outage decider outage` — because two MISSING rows
genuinely carry the same cause (`live/agent/arena.py` writes it per session). At 1440 the second cause's line box painted
**50.6 × 15 px on top of the first cause's second line**. Visible in the capture; invisible to every row the gate had,
because `containment` asks whether an element is inside its plate, not whether two elements are on top of each other —
exactly the A7 second clause the notes say is unimplemented. I found it by writing that missing sweep as a probe against
the built page, not by looking at a screenshot and guessing. Fixed in `GuardTiles.js` (repeats print once, words
untouched; each cause wraps in its own box); arena plate height at 1440 falls 406 → 371 px. `npm test` 429/429.

**Fresh checker run 6 (independent seat, own copy) on the previous set: PASS on all six items** — 0 verdict differences
in 445 rows, all 179 PNGs byte-identical to its own run, A8 and A29 (iii) re-measured to within rounding, and the
word-figure rule proved to bite (118/118 rows fail under a forced split, no numeric falsely flagged). Its remaining
defects are doc/measurement level and none change an image: the A29 (iii) tone-step was not recorded (it is now — three
+4 steps reach 1.583–1.983, still 6/11 below 1.8:1, so the clause is unreachable inside the spec's own remedy), the
ruling-request path it cites lives outside the repo, and the A29 (iii) row counts bodies from the renderer's own list so
a missing body cannot fail it.

Still the owner's call, unchanged: **A8** (24 rows — the whole shelf row sits below a 900 px fold at 1440, 207 px for the
default state) and **A29 (iii)** (2 rows). No cutover: the old dashboard is still live on `:8791`.

---

## FINAL set pointer, appended by the OPENBELL lead seat (04 Nov 02:10 ET) — one set, and one wording corrected

Superseding the 01:40 and 01:55 corrections above: **grade `design/watchover/renders/20261104T0725Z/`**
(commit `f6014f1`, page `0c357fc2…`, gate tool `5b0fab7e…`). 179 PNGs, **446 rows, 420 pass / 26 fail**, `npm test`
429/429, build 332,145 B / 100,955 gzip. `renders/` holds exactly two directories: `0620Z` (pre-fix baseline) and
`0725Z`. Commit `78644b7` keeps this true and removes the two superseded sets, because three folders each claiming to be
"the set" is the one thing that could still waste your morning.

One word of mine to correct: I described the arena-plate defect I fixed as **"text painted on text, 50.6 × 15 px"**. The
measurement behind that was a **bounding-box** intersection; checker 7 showed the glyphs and line fragments never
intersect (a 3.99 px gap on the shared row). What was actually on screen was the duplicated cause wrapping
**interleaved** — `· decider / outage decider / outage`. The fix (`90a2c48`) and the finding both stand; my description of
the mechanism was wrong, and the corrected numbers are now in `RESOLUTION.md`.

Nothing else moves: **no cutover** (old dashboard live on `:8791`), and the only failures in the pack are the two classes
that need your ruling — **A8** (24 rows: the whole shelf row sits below a 900 px fold at 1440) and **A29 (iii)** (2 rows),
both laid out in `board/reports/openbell-watchover-ruling-request-20261104.md`.

### Second correction, 04 Nov 02:12 ET — the graded set moved twice after the first correction

The block above (01:15 ET) said to grade `renders/20261104T1245Z/`. That was right at 01:15 and is **no longer right**: the
page itself changed after it — commit `90a2c48` fixed a real text-on-text collision in the `preview-missing` state at 1440
— so `1245Z`, `1320Z`, `1345Z`, `064858Z` and `0650Z` all measure a page that is no longer HEAD.

**Grade `renders/20261104T0725Z/`.** 179 PNGs, **446 rows, 420 pass / 26 fail**, `dirty: false`, build `0c357fc2…`
(332,145 B / 100,955 gzip), gate tool `5b0fab7e…`, 118/118 expected captures, inventory 179/179. `RESOLUTION.md` at HEAD
(`9b32789`) names this set and carries the corrections two fresh checkers asked for.

The 26 failures and the two rulings they need are unchanged: **24 are A8's viewport clause** (at 1440×900 the front row
ends at 817.7 px inside the window and the whole shelf row starts at 829.7 px, so it is off-screen rather than a near
miss) and **2 are A29 (iii)** at front scope (6/6 bodies fail; the spec's own three-step remedy reaches 2/6 ring contrast
and leaves coverage below 0.60 on 4/6). Nothing else is open on this pack.

No cutover: `:8791` still serves the old dashboard (`849e2c1c…`, 219,311 B).

### Third correction, 04 Nov 02:45 ET — grade `renders/20261104T0745Z/`

The set moved once more for a non-image reason: two checkers found that the `a29 (iii) body population` row I added could
still pass when a plate and its body vanished together, and that the A30 note named the tone-step count without the
rewritten hexes. Both are fixed in `5465f2e`, the page is unchanged, and the new set is **`renders/20261104T0745Z/`**:
179 PNGs, **446 rows, 420 pass / 26 fail**, `dirty: false`, build `0c357fc2…` (332,145 B / 100,955 gzip), tool
`f9429f40…` (= HEAD), 118/118 captures, inventory 179/179.

`0725Z` is the same page and the same 26 verdicts with the earlier row; four checkers verified it and it is kept beside
the newer set. Either is safe to grade — the images are identical — but `0745Z` is the one whose rows cannot be fooled by
a missing body.

**Also filed tonight, and it changes how Wednesday reads:** the live futures desk has been in `mode: automated` since
3 Nov 07:40 (not `cards` as `ARMING.md` claims), so the engine sends no tap cards and the executor is the only order
path. Wednesday is **one gate away**, not two: R11 inside 09:00–09:15, a fresh enable, and an attestation with a
different-seat inspect. Today's protection was re-verified at 02:30 — expired enable, `trades 0`, `placed []`, and no
intent file for 4 Nov, so nothing has been attempted. See `SEAL-FUTURES-2026-11-03.md` §10 and §11.
