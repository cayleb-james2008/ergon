# BOARD-REMAINDERS-CHECK — `ff32f80` (fix/board-remainders-20261103)

Independent check. Clean export at `/tmp/opencode/chk-board` (`git archive ff32f80`, `npm ci`), no repo write, no
git write in the source repo, no network beyond `127.0.0.1`, no real project touched. Own render, own fakes, own
timeline sampler. Host: 32 cores, load average **6.70 → 16.96** during the runs (quoted beside each timing).

Test-tree reconciliation: the tree has **105 e2e tests** = 100 runnable + 5 skipped (`supabase-live`, "set
E2E_SUPABASE=1"). The commit's "93 pass" excludes the 7 new `connection.test.mjs` tests, which it reports
separately: 93 + 7 = 100. That matches.

## Counts

| # | Check | Verdict |
|---|---|---|
| 1a | `npm test` = 119 | **PASS** |
| 1b | `npm run test:e2e` = ~93 pass / 0 fail / 5 skip | **FAIL** (97 pass, 3 fail, 5 skip; one failure reproduces alone) |
| 2 | `connection.test.mjs` 7/7 | **PASS** |
| 2b | Is the 1 s probe a real cost the docs should state? | **FAIL** (real, ~8× the documented rate, stated nowhere) |
| 3 | Mobile render at 390×844 + Add sheet at 508 | **PASS** |
| 4 | Connection honesty on my own run | **PASS** (two named caveats) |
| 5 | Legend initials only | **PASS** |
| 6 | Docs: `#conn`/`data-blank`, 4 states, MANIFEST, README | **PASS** |

**PASS 6 · FAIL 2 · UNVERIFIED 0**

## FAIL detail

### 1b. `npm run test:e2e` is not "0 fail", and one failure reproduces alone
Full run (start load 15.84, end 11.93): **105 tests, 97 pass, 3 fail, 5 skipped, 111 s**.

1. `test/e2e/round3.test.mjs` "update cost does not grow … 4x CPU" — printed
   `typing worst key event 1376 ms at 12 jobs, 1720 ms at 60 jobs; long tasks [] and [250]`
   at load **15.84**. Fails on the long-task bound (100 ms: the 12-job run saw no long task at all, so the
   bound falls to the 50 ms floor × 2).
   **Re-run alone**: `typing worst key event 1360 ms at 12 jobs, 1728 ms at 60 jobs; long tasks [] and [325]`
   at load **9.68** → **FAIL again**. The commit says "that test passes alone"; at this load it does not. The
   1360 ms/keystroke at 12 jobs says more about host contention than about the board (event duration includes
   waiting for a presented frame), but the verdict is what it is.
2. `round4` "density: update cost at 200 jobs" — `longest task 339 ms` against a 300 ms bound at load 13.36.
   Re-run alone: `longest task 242 ms`, **PASS** at load 11.36. The test's own comment records 209–293 ms on
   earlier hosts and a 250 ms bound that "flapped", so the 300 ms bound keeps ~7 ms of headroom.
3. `round4` "a job deleted or changed elsewhere while its dialog is open" — `TimeoutError` clicking `#job-save`
   ("element is not enabled") at 33.9 s, load 13.36: the page learned about the outside change before the click.
   Re-run alone: **PASS** (16.5 s) at load 10.78.

So the full-suite instability is **3/100, not the 1 the commit admits**; two are pure timing, one is a real race
between the test's "this page has not been told" precondition and the demo adapter's poll. No product regression
found in any of the three — but "0 fail" did not reproduce, and the one failure the commit called load-only also
fails at load 9.7. Note `node --test` runs the 8 e2e files concurrently, so the suite itself is what lifts load
from ~7 to ~16; that is a harness property, not a defect of this commit. The host also carries **7 stale
`vite preview` servers and 24 headless Chromium processes up to 14.7 h old** from other sessions (not mine; mine
all exited and their temp builds are gone), which is a large part of the ambient load — worth knowing before
calling any timing number a regression.

### 2b. The 1 s probe is a real, undocumented cost (about 8× the documented rate)
Measured on my own run, healthy and Live, one visible tab: **12 `/rpc/board_token` requests in 10 s**
(gaps `660, 999, 1001, 999, 335, 666, 999, 1001, 999, 1000, 1001` ms) = **~72/min**, plus the store's own
reconcile. `docs/APP.md` ("Realtime and polling cost") and `docs/DESIGN.md` §10a still say **8 requests a minute**
while Live/idle and do the whole egress budget on that figure. The hook's own 1 s loop is mentioned in no doc and
in no line of the commit message. Using the docs' own per-probe wire figure (~2.3 KB/min ÷ 4 probes ≈ 0.57 KB),
68 probes/min is **≈39 KB/min**, i.e. **≈5 GB/month at the 132,000 viewer-minutes the docs assume** — around the
5 GB free allowance the same paragraph cites, against the 305 MB it claims. Requests scale the same way
(~1.6 M/month vs 1.06 M). This is real money/quotas on a board that is left open on a wall all day, and it is
the item most worth documenting. (Pausing while hidden is correct: 0 probes in 5 s hidden, measured.)

## UNVERIFIED
None. Everything above was run here.

## Item 2 — what each of the 7 connection tests proves (my reading of the code, then my own runs)

1. *Healthy load*: the word is blank until the feed confirms, then `Live`; `Reconnecting` is never in the page and
   `data-state` ends at `live`; a real (non-TEST) board hides `#test-marker`. Proves the 1.5 s settle hold works
   for the ordinary load. Mine: words `["", "Live"]`, blank 131→445 ms.
2. *REST outage with the feed up*: `Error` within 2.8 s, and no `Live` after the first `Error`; recovery returns
   to `Live`. Proves the probe, not the store, is what catches a dead read path while the socket is healthy.
   Mine: `Error` at t=2600 ms with the outage starting ~600 ms (≈2.0 s), 0 `Live` after.
3. *REST hangs instead of failing*: same, ≤3 s. Proves the 1.5 s probe timeout converts a hang into `Error`.
4. *The first read hangs*: never `Live`, `Error` ≤3 s. Proves the blank window does not turn into a false Live
   when nothing has ever loaded.
5. *The first read is refused* (aborted): never `Live`, `Error` ≤4 s including page load.
6. *A 1.2 s server*: still `Live` after 7 s. Proves the timeout is not so tight that a merely slow answer is an
   outage — but see the 2 s finding below; the guarantee is 1.5 s wide, not more.
7. *"The socket drops and returns"*: **the test never touches the socket.** It dispatches `window offline` /
   `window online`, i.e. it fakes `navigator.onLine`. What it actually proves is narrower than its name: a 400 ms
   browser-journal offline blip does not produce the word `Reconnecting`. It would pass even if the indicator
   screamed `Offline` for the whole blip, and mine does exactly that (`Offline` 425→840 ms, then `Live`). The name
   should say "navigator-onLine blip"; a real socket drop (adapter status → `polling` → `Reconnecting` after the
   settle) is untested here.

Probe cadence: yes, it is a real cost and yes the docs should state it (see 2b).

## Item 3 — mobile render (my build, `VITE_BACKEND=demo`, 390×844, `isMobile`, `hasTouch`, DPR 2)

Measured, load 10.87: `--fs` **12px**, `--fs-title` **14px**, body 14px / board 12px, `.row` **52px** with jobs
and **48px** empty (both exist — the commit bullet says 48, its evidence says 52; each is half the truth),
`.job` **232×44**, first Add control **44 px high / 90 wide**, page **no** horizontal scroll (390/390) and no
vertical scroll (844/844), board scrolls inside itself, **19 rows**, 0 console errors. Add sheet at `--vv-h: 508px`:
sheet 21→487, focused `add-A-title` 103→147, Save (`add-A-save`, enabled) 426→470 — **all inside 0–508**, sheet
does not scroll, scrim present, popper wrapper `position: fixed; height: 508px`. Screenshots:
`/tmp/opencode/chk-render/phone-390x844.png`, `phone-add-sheet-508.png`.

**Opinion:** legible and proportionate now — 12/14 px with 52 px rows and 232 px cards reads like a phone app, not
a shrunk wall screen; the second card peeking at the right edge is a clear swipe affordance; the Add job sheet
with the keyboard up is genuinely usable (the field and Save both visible, nothing clipped). Two honest nits:
(1) 14 customer names on the visible cards are ellipsised (`TEST Harbour …`, `TEST Millbank…`) — full text is on
the card `title`, and the demo's `TEST ` prefix eats the width, so this is the swipe design working, not a
regression; (2) half of the 844 px screen is empty-line rows (D, F, I, J, L) each with a 44 px "Add job" — correct
per "19 bays always drawn", but it is why the board has to scroll on a phone.

## Item 4 — connection honesty, my own run

* Healthy load: label words were only `""` then `Live`; **no `Reconnecting`**. `Live` first appeared at 445 ms
  with rows already rendered; in the slow-load case below the label stayed blank with `jobs: 0`, so **the label is
  never `Live` before rows**. PASS.
* REST aborted while the WebSocket stayed up: `Error` ≈2.0 s after the failure began, and **0 samples of `Live`
  after the first `Error`** over the next 18 s. PASS.
* 1.2 s answers: stays `Live`, never `Error` (also true at 1.4 s). PASS.
* `data-blank` really hides the dot: computed `.dot` background while blank is `rgba(0, 0, 0, 0)`, 8 px, and the
  `Live` dot is `rgb(163, 201, 93)`. PASS.

Caveats, with evidence: the 1.5 s ceiling (below), and the `Offline` flash (below).

## Items 5 and 6

* **Legend:** rendered `.legend .initials` = `JA/JS/JG/AK/LB` with `data-hue` blue/pink/orange/green/purple; **no
  `title`, no `aria-label`, no extra text** on any chip (`legendHtmlLen` 279, no name attributes); the `<ul>` label
  is `Initials colours`. `src/config/legend.json` entries hold only `{"hue": …}` keyed by initials (plus an
  `aliases` map), and the `$comment` records the ruling verbatim: "Person names are not shown, by owner ruling
  (Cayleb, 3 Nov 2026) … Do not add a name to an entry." PASS.
* **`docs/AGENT-INTERFACE.md`** line 165 documents `#conn` `data-state` = `live / polling / offline / error` (four,
  unchanged, and no fifth value exists in code) **plus** the additive `data-blank` ("label empty, dot draws
  nothing"). PASS.
* **`node scripts/manifest.mjs --check`** → `MANIFEST ok: 163 files`. It shells out to `git ls-files`, so it cannot
  run in a bare tarball; I ran it from the export after `git init && git add -A` (tmp copy only, source untouched).
  PASS.
* **README** (new in this commit): React 19 / Vite / Tailwind 4 / Radix is true, `../GOAL.md`,
  `../CONTRACT-AND-LEDGER.md`, `../L5-CHECK-BRIEF.md` all exist, "build `455a9da`" is the deployed one, the
  real-data warning is right. One stale line: `npm run test:e2e # demo backend + board flows + pack-check` omits
  the new `connection.test.mjs` (which runs the supabase adapter against faked network). Cosmetic; PASS with that
  note.

## Item 7 — what the commit does not admit

1. **A uniformly slow network (>1.5 s per request) is labelled `Error`, permanently, while the board works.**
   Measured: every REST answer delayed 2000 ms → `Live` → `Error` and it stays `Error`; with a 5 s first read the
   timeline was blank (0–2.5 s) → `Error` at 2.6 s → **board `ready` with its job at 5.0 s and the label still
   `Error`**. Editing is then disabled (`canEdit` false) on a board that is fine. The hook's comment claims "the
   timeout is longer than a slow but working server needs"; the true ceiling is `PROBE_TIMEOUT_MS = 1500`, and the
   test only pins 1.2 s — 300 ms of headroom. On a bad phone connection this makes the honest indicator lie in the
   other direction. This is the strongest finding in the commit.
2. **`data-state="live"` while nothing has been read.** The label is blank, but the attribute says `live` for the
   whole slow read: measured `state: live, blank: true, jobs: 0, board: loading` from t=400 to t=2400 ms. The code
   comment says "Live is never claimed before the first board is read" — true of the words, false of the documented
   agent-facing attribute. An agent that reads only the four documented `data-state` values is told the board is
   live before any data exists. That is the real contract wrinkle, not `data-blank` itself (which is additive and
   documented). A monitor that switches on `#conn-label` **text** now must handle `""` as a possible value.
3. **The word hold does not create a long blank.** Answering the question directly: no. The probe bounds it — blank
   lasted at most **~2.5 s** in every case I could construct (hung first read: blank 89→2480 ms, `Error` at 2600;
   healthy: 314 ms; slow first read: blank ends at the probe verdict, 2.6 s). It never ran to "settled", because
   `waiting` ends with the probe's own failure. The cost of the design is item 2 above, not a long silence.
4. **`data-blank` is not a break in the four `data-state` values, but it is a semantic drift**: `data-state` can be
   `live` or `polling` while the human sees nothing at all, so "polling = Reconnecting" in the docs (and in
   `docs/FUNCTIONALITY-DIFF.md`, `docs/APP.md`) is no longer literally true for up to 1.5 s (2.5 s on a hang).
5. **A transient browser offline flag flashes `Offline` for its whole duration** (`Offline` 425→840 ms on a 400 ms
   blip) — the same class of unconfirmed transient the commit set out to remove, left in place and explicitly
   allowed by the 7th test, whose name ("the socket drops") describes something it never does.
6. **Docs left stale by the change**: `docs/APP.md` and `docs/DESIGN.md` §10a still state the 8 requests/minute
   cost and give no mention of the 1 s probe or the blank window (only `AGENT-INTERFACE.md` was updated). This is
   the same 2b item: on the docs' own traffic model the new probe is where the free-tier egress gets spent.
7. Minor: the commit bullet says rows 48 px, its render evidence says 52 px; both are real (empty rows 48, rows
   with jobs 52) — the wording is just inconsistent. Pre-existing, not this commit: `docs/APP.md` says
   "Reconnecting (4 s poll)" while `POLL_MS` is 3000.

## Evidence paths (mine, outside the repo)
`/tmp/opencode/chk-render/render.mjs`, `conn.mjs`, `offline.mjs`, `phone-390x844.png`,
`phone-add-sheet-508.png`, `wide-1366x768.png`; logs `/tmp/opencode/chk-board-e2e.log`,
`/tmp/opencode/chk-board-rerun.log`.

No deploy, no push, no key, no spend, no write to the real project. Counts and structure only; all fixtures above
are my own fake rows (`e1`, `X`) or the demo seed.
