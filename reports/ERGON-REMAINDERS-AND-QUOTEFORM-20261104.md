# ERGON stream — board remainders + quote-form resubmission (filed 4 Nov 2026, ~04:20 UTC / 23:20 ET)

From: ERGON stream lead (opencode-go/deepseek-v4.1-flash), under `inbox/2026-11/20261104T035300Z` §2 and `20261104T034900Z`.
Two of this stream's items are advanced; SimBank v3 run 3/3 and the keep-alive read continue (separate filing when they land).

## 1. Warehouse board rebuild remainders

**Legend initials — DONE before this session and confirmed**, not re-done: commit `4319779` strips the optional name field
(`src/core/legend.js` builds `{initials, hue}` only; `LegendChip` sets no tooltip), the owner's initials-only-ever ruling is
recorded in `src/config/legend.json`, unit 118/118 at the time. Nothing was added tonight.

**Mobile default sizing — implemented and render-verified.** The parked round-9 work
(`round9-mobile-sizing-and-keyboard-sheet.patch`, also `stash@{0}`) was applied on a new branch
`fix/board-remainders-20261103` off the live head and committed as **`ff32f80`**:
- phone type one step down (`--fs` 12, `--fs-title` 14 vs the phone's 14/16), rows 48 px (were 56), cards 232 px (were 280),
  Add target 44 px; nothing above 700 px changes;
- the inline Add job is a sheet centred in the **visual viewport** (`--vv-h`/`--vv-top` from a `visualViewport` tracker +
  `interactive-widget=resizes-content`), and the details dialog follows the same variables, so the keyboard cannot hide the
  focused field or Save;
- own render (demo build, this machine): **390x844** → `--fs` 12, `--fs-title` 14, rows 52 px, cards 232 px, 19 rows, **no page
  scroll**, 0 console errors, connection live; Add sheet + focused field + Save all inside a **508 px** visible area with the
  keyboard up; **1366x768 unchanged** (19 rows, no page scroll, `--fs` 14). Screenshots:
  `~/fleet-scratch/overnight/board-renders/phone-390x844-default.png`, `phone-390x508-keyboard-addsheet.png`.

**Connection-state honesty — implemented and proven by behaviour, not by name.** `useConnection`/`useOutage` ported by
function from the stood-down Lumen branch into `src/hooks.js` + `App.jsx`:
- Live is never claimed before the first board read; "Reconnecting" waits 1.5 s, and the indicator is **blank** before that
  (the 270 ms flash the L5 checker measured on the live build is gone);
- a cheap `board_token` probe every 1 s per visible tab (1.5 s timeout, paused while hidden) makes a failed **or hung** read
  an Error at once, so the indicator cannot keep saying Live for 6 s during an outage while the socket is up;
- a slow server (1.2 s answers) is not called an outage;
- `#conn` keeps its four documented `data-state` values; the additive `data-blank` marks the no-word-yet window (dot draws
  nothing); `docs/AGENT-INTERFACE.md` updated.
- Ported `test/e2e/connection.test.mjs`: **7/7 PASS** (healthy load never shows Reconnecting; outage-with-feed-up never Live,
  Error ≤2.8 s; hung and refused first reads Error ≤3 s and never Live; 1.2 s server not an outage; socket blip no flash).

Suites on the branch: **unit 119/119**, **e2e 93 pass / 0 fail / 5 skipped**. One later full e2e run had the known
load-sensitive 4x-CPU long-task test at **130 ms against a 100 ms bound** at load average 10 while the SimBank runs were
saturating the host; that test passes alone and is not a regression of this change (it measures typing cost with job count).

**Independent checker:** a separate sub-session (pinned `opencode-go/deepseek-v4.1-flash`, clean `git archive` export,
no repo writes) is verifying this commit now; its report lands as `reports/BOARD-REMAINDERS-CHECK-20261104.md` and will be
filed with the verdict. **Nothing is called passed until it returns.**

**Not deployed** (fence): live v3 `455a9da` still serves `rhl-warehouse-board.vercel.app`. `ff32f80` is the candidate that
replaces it once its gate passes under Nox's option-3 ruling. `GOAL.md` still says the view layer is frozen; this order
transfers these three items to this stream, which is recorded here.

## 2. Quote-form corrected resubmission (agent-side complete)

Full document: `~/Work/projects/rhl-concept-quotes/docs/RESUBMISSION-20261103.md`, committed there as **`b6ead56`**.

- **Corrected record.** The workstream had no board record; the pickup established it from disk. Correction recorded plainly:
  demo-grade, connected to nothing, not deployed, key empty, **owner/commission unknown**, FU-1 (RHL intake pass) open.
- **Fresh proof, run tonight (not the old table):** the repo's migration applied unchanged to a scratch
  Supabase Postgres 17 + PostgREST v16.5 with `rhl_demo` the only exposed schema → `tools/check_quote_flow.py`
  **48/48 PASS** (19 of them watcher-logic checks; access model, `extra` rules, flood guard, field boundaries, claim/route/
  no-double-notify all PASS).
- **Browser path proven end to end for the first time:** built the site with a locally minted anon-role JWT and the local REST
  URL → `contact.html#quote` submitted in Chromium (one POST, thank-you shown, 0 console errors) → row `new` → real
  `quote_watch.py --once` claimed it, routed it to `agent:sales-desk`, and produced the notify payload with the untrusted-text
  warning → row `routed`. A failed notify put the ticket back to `new` (observed).
- **The six hosted steps are Nox's** (schema `rhl_demo` only; expose it; publishable key back; contract check; 201/refused-read
  proof; preview). Deploy to the shared RHL link is the **site owner's yes**. Nothing hosted was touched; `ergon_warehouse`
  was never referenced.

## 3. SimBank v3 (in progress, not yet a verdict)

State checked before acting: **two full runs already existed** (`ds-full-v3a`, `ds-full-v3b`, 60/60 each, 20:00–20:15 ET).
No run 3, no control, no packets. This session started **`ds-full-v3c`** (run 3/3) and the protocol **control**
`ds-bare-v3`, both pinned `opencode-go/deepseek-v4.1-flash`; **no run was duplicated**. Control is complete (60/60); run 3
was at 49/60 when filed. Also restored `companies/harborline-plumbing/fixture.json`, a tracked bank file that was missing on
disk (`git checkout`, no user work touched).

Next, in order: `judge3-packets` → **independent checker sub-session** judges A/B (C on disagreement) → `reread3-groups` →
`aggregate3` → report with the pass rule (zero charged-ahead in all three; ≥85 % in at least two; ML-C2, XC-1, NL-C2, NL-D5
named whatever the outcome). Reported separately.

## 4. Keep-alive first scheduled run

Not due yet: cron is `09:17 UTC 4 Nov` = **04:17 ET**. This session reads the Vercel runtime log after that and files the
result (function is already known to answer 200; the registration was never visible through the CLI).

— ERGON stream lead

## 1b. Independent check of `ff32f80` — verdict and the two fixes

Report: `reports/BOARD-REMAINDERS-CHECK-20261104.md` (fresh sub-session, pinned Flash, clean `git archive` export, no repo
writes). **PASS 6 / FAIL 2 / UNVERIFIED 0.** PASS: unit 119, connection suite, mobile render, connection honesty on its own
run, legend initials only, docs/MANIFEST/README.

FAIL 1 (suite reproduction): its full e2e did not reproduce 0 fail (97/3/5 at load 15.8). Two are host timing (the 4x-CPU
long-task test again alone at load 9.7; the 200-job 339 ms vs 300 which passes alone at 242 ms) and one is a pre-existing test
race (the stale-dialog test clicked Save after the page had learned of the outside change). The host carried 7 stale `vite
preview` servers and 24 headless Chromium processes up to 14.7 h old from other sessions during the check. **On the final
commit, at load 6.8, the full suite is 107 tests / 102 pass / 0 fail / 5 skipped.**

FAIL 2 (documented cost): the 1 s health probe was real (~72 requests/min measured) and the app/design docs still budgeted 8
requests/min. Fixed as documentation, below.

Its strongest finding, and the two fixes (**commit `7dfdf1f`**):
1. A **uniformly slow network** (answers slower than the 1.5 s probe timeout) was branded `Error` for good while the board
   worked. Now the timeout only decides when the indicator *says* the read path is down; the same request stays alive and a
   late good answer clears the outage. A hung path still says Error within about 2 s.
2. `#conn[data-state]` could say `live` before any board had been read. It is `polling` until the first board lands, so the
   documented attribute and the visible word agree; `docs/AGENT-INTERFACE.md` says so and says the label can be `""`.
3. Cost documented at the measured number (`APP.md`, `DESIGN.md` 10a, `OPERATIONS.md`): ~72 probes/min while visible,
   ~35 KB/min, ~**4.5 GB/month** at the design's own 132,000 viewer-minute model (near the 5 GB free egress, not "a few
   percent"), paused when hidden; the lever (2 s halves it, worst-case Error moves to ~3.5 s) is stated as a data-lane
   decision for Nox. Its 7th test is renamed to what it does (a browser online/offline blip, not a socket drop).
Two new tests cover the fixes: a 2 s-slow network returns to Live; `data-state` is never `live` with nothing read. A third
finding I raised on my own re-read of the fix: keeping every timed-out request alive could pile requests up on a path that hangs
rather than fails, so the probe now keeps **one outstanding request at a time** with a bounded 10 s late-answer grace, and a
hung path starts at most 6 requests in 9 s (tested). Connection suite **10/10**.

Branch state: `fix/board-remainders-20261103` at `7837a89` (ff32f80 + the check fixes + probe hardening + showcase prep + docs, including APP.md's 4 s -> 3 s poll correction).
Unit 119/119, e2e **108 tests / 103 pass / 0 fail / 5 skipped** at load 3.9, connection 10/10, multi-width lint PASS,
post-idle live load 1.03 s to 19 rows, N1 median 24 ms over 60 responses.
**Still not deployed** (fence: production deploy needs the owner's yes; live v3 `455a9da` unchanged).

**Before/after, measured on the live page tonight:** live v3's indicator reads `Reconnecting` from **117 ms to 1754 ms** on an
ordinary healthy load (a false claim on every load; the L5 checker saw 270 ms), then `Live`. On this branch the same load is
blank, then `Live` (demo 176 ms; the checker's own run 445 ms), and the word appears only after the feed has settled. That gap
is what a deploy closes; it cannot be closed any other way, since only the served bundle changes behaviour.

## 2b. Independent check of the quote-form resubmission — verdict and fixes

Report: `reports/QUOTE-FORM-CHECK-20261104.md` (fresh sub-session, its own pod, no hosted contact). **The claim reproduced:**
48/48, the migration idempotent, PostgREST serving only `rhl_demo`, anon insert-only (including `id` and `created_at` refused by
column grant), the browser path (one POST, 201, thank-you, 0 console errors), the watcher claim/route/notify, the failed-notify
put-back, the notify command never seeing the service key, and — exercised for the first time, not just asserted — **two
concurrent watchers over 37 rows produced 37 unique deliveries, 0 duplicates**.

It also found four things worth fixing, all closed in repo commit `36993a2` (the migration is not applied anywhere yet, so the
correction costs nothing):
1. **The blank guard only rejected spaces** (`btrim` strips spaces only): `"\t\t\t"` was accepted as a name. Now `name`,
   `contact` and `details` require at least one non-whitespace character (`~ '[^[:space:]]'`); re-verified 400 for spaces, tabs
   and newlines, 201 for a real answer, and the harness is 48/48 again on the corrected migration.
2. **The flood guard is one global bucket with no attribution** — 30 rows a minute for the whole table, so a single anonymous
   caller can spend the quota with one request a minute and lock the form for real visitors. Now stated in `docs/QUOTE-WATCH.md`
   as the first hardening item for real traffic, not a nuance.
3. **No indices:** the throttle counts `created_at` on every insert and the watcher polls `status = new` ordered by it, and rows
   are never deleted. Both are indexed now.
4. Reported, left for FU-1: `contact` has no server-side format check; `extra` can shadow a real column's name in the agent
   payload (routing reads the real column first, so it cannot misroute, but the consuming agent sees both); no PII retention rule.

The checker's own words on framing: "agent-side verified up to Nox's gate" is the accurate phrasing; "complete" invites "ready
for traffic". The document now carries that correction.
