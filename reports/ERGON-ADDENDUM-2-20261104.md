# ERGON addendum 2 — e2e determinism, the keep-alive's dead log path, and a staged ruling input

ERGON stream lead (opencode-go/deepseek-v4.1-flash), 4 Nov 2026 01:05 ET / 06:05Z. Continues
`reports/ERGON-REMAINDERS-AND-QUOTEFORM-20261104.md` and `reports/ERGON-ADDENDUM-20261104.md`.
Nothing deployed, nothing pushed, no run made, no hosted state touched.

**Tip at the time of this update: `3780e04`** (7 commits off live `4319779`). **It has since moved: `9cc7d95` at 01:38 ET** — the
board read now has a 15 s deadline and is single-flight, because the second independent check found the label could read Live over a
frozen board (`board_token` is a different RPC from the board read). See §6. The connection work in §2b landed as `2150fa8`
(code) and `3780e04` (docs).

## 1. The board's e2e suite is now deterministic under load (2 commits, checker PASS WITH CAVEATS -> its three caveats closed)

Branch `fix/board-remainders-20261103`: `d9c782b` then `7035ab9` (on top of `7837a89`).

*Status of the check: the independent checker's verdict on `d9c782b` was PASS WITH CAVEATS and it named exactly three tests as
weaker; `7035ab9` addresses all three. Its re-check of `7035ab9` was still running when this was filed — the numbers below are
mine, and the verdict will be appended when it lands.*

The first check found 3 failures in 100 (two host-timing bounds, one real test race). The suite said "0 fail" and did not
reproduce it, so the claim was not evidence. What was done, and what the independent checker verified on its own machine:

| item | change | checker |
|---|---|---|
| real race | round4's "changed/deleted elsewhere while the dialog is open" needs "this page has not heard yet", and raced the ~15 s reconcile read. New **demo-only** hook `window.__rhlDemo.freeze(on)` pins the probe's answer for that tab. | **PASS** — with the 16 s window held open, the unfrozen copy fails ("this page has not been told") and the frozen one passes |
| wall-clock bounds | new `hostRef(page)`: a fixed 1200-element create-and-layout workload at the same 4x CPU throttle. Bounds are `max(old, k*hostRef)`, so a contended host raises the bound and a real regression still fails. round3 ceiling 2x the reference, round4 4x with a 300 ms floor. | **PASS** — a real injected regression failed both tests (1416 ms vs 300/100; long tasks to 510 ms) |
| harness vs app timing | the three "Error within 3 s" assertions timed Playwright's polling, not the app: the page shows Error at ~2.14 s while the harness read ~2.30 s and once 3.04 s. Now measured on the page's own clock, with non-numeric/negative measurements refused. | **PASS** — right quantity, gap measured at 160 ms |
| listbox | `choose()` waits for the option list to detach before the caller types or presses Enter (a slow frame left the 120 ms fade mounted and the next wait failed 30 s later, elsewhere) | **PASS** — 18/18 in the two choose-heavy files |
| hung path | the probe test claimed to measure "outstanding" but counted starts against a route that never answers. It now asserts starts (<=3) **and** no two starts within 1500 ms — the real sequence is 0.6 s then 3.9 s. | **WEAKER** in the first commit; the gap assertion was added in the second, and then **FAILED its re-check** — see §1b |

The checker's three named weaknesses were then closed: round3's ceiling 4x -> 2x (at load 3.2 the old 4x printed a 152 ms bound
where the shape rule says 100, so a 52 % slower board passed; now a quiet host keeps 100 ms and load ~20 gives 210 ms), round4
asserts the **median** of three passes with the worst capped at 1.5x (it had asserted the best of three, i.e. the warm case, while
the cold pass is what a user feels). The third caveat — the hung-path gap rule — did **not** survive its re-check: see §1b.

**Numbers now:** full e2e **108 tests, 103 pass, 0 fail, 5 skipped, exit 0** at load 10-15 (`evidence/board-e2e-flakefix-4-20261104.log`);
unit **119/119**; connection **10/10**; both perf tests pass at load 13 and at load ~20 with 18 CPU burners. Production build
leak scan: `demo-employee|demo-expired|demo-revoked|__rhlDemo|rhl.demo` **0 hits**; the only `freeze(` is `Object.freeze`.

## 1b. Correction (01:35 ET): the third caveat did NOT hold — the hung-path "one at a time" rule failed its re-check

Filed honestly rather than left standing: the re-check of `7035ab9` (`BOARD-FLAKEFIX-CHECK.md`, finished 01:30) judged that
assertion **FAIL — both ways**. Verdict and evidence:

- **It failed correct code.** "No two starts within 1500 ms" is a schedule assumption, not a concurrency property. In **1 run of 12**
  on unmodified code the starts were `[682, 1009, 3993]` ms — the indicator's probe and the store's reconcile read are two
  independent callers and legitimately began 327 ms apart. That is a 1-in-12 flake written into the suite.
- **And it still passed a real defect.** An injected duplicate probe 1.6 s later (`[670, 2270, 3991]`) sat inside the gap rule and
  passed. So the test was named for a property it did not assert.

Its recommendation: derive the assertion from what the route can actually show, or keep `<= 3` and say why. The tree now carries
that rework (test-only, 01:31, `test/e2e/connection.test.mjs`): **after the first 5 s of a 9 s window, at most one start** — i.e. a
legitimate pair is allowed during setup, a pile-up is not allowed to keep going. Round4's stale best-of-3 comment is corrected in
the same edit (comment only).

Lead's arithmetic check of the new rule against the checker's own measured sequences (offline, no app run):
`[682,1009,3993]` (the case that broke the old rule) → **pass**; `[550,3900]` and `[680,1010,3990]` (measured real sequences) → pass;
a regression to one probe per 1.5 s interval → `[0,1600,…,8000]` → **fail** (2 late starts); two per interval → **fail** (5 late).
Stated limit, in the test's own comment: the reworked rule does **not** catch a single duplicate probe 1.6 s after setup — the name
now claims only "after the setup nothing keeps starting", which is exactly what it measures. The stronger "one outstanding request"
property is still asserted where it can be measured deterministically (the connection suite's single-outstanding test on a hung
REST path), and is recorded as a named residual rather than silently dropped.

Nothing here is product code: the deploy artifact is unchanged by the rework. A fresh independent check of the final tip is the
thing that closes this, and it is not written as closed until it says so.

## 2. The keep-alive check: registration PROVED, and the log-based proof proved impossible before it could mislead

- `vercel crons list --json` -> one job, `/api/keepalive`, `17 9 * * *`, `enabled: true`, host
  `rhl-warehouse-board-2ttt1y84e-caylebjamesportfolio.vercel.app` (the live production deployment). The project object carries
  `crons.enabledAt`, `disabledAt: null`, `deploymentId dpl_busDKSQZ6wZKggot9EvwyWBemjpR`.
- The function works end to end **now**: `GET /api/keepalive` -> `200 {"ok":true,"status":200}` (Supabase answered the
  count-only HEAD; row contents never read).
- **The log plan was wrong and is now retired.** I invoked the function myself at 05:53:52Z (200) and asked for its logs 30 s
  later: `recordsReturned: 0`, both deployment-scoped and project-scoped. The dashboard's Runtime Logs, filtered to
  `requestPath:/api/keepalive` over the same window, says "No Runtime Logs Match Your Filters". Runtime logs are not captured
  for this project on Hobby. The armed watcher would have written "cron not seen" whether or not the cron ran — a false negative
  dressed as evidence. Evidence: `evidence/keepalive-log-availability-20261104.txt`.
- Watcher v4 therefore records only what is provable at 04:17 ET (registration, enablement, the function's 200) and states
  plainly that **whether the scheduler fired is not observable from this machine**, naming the three places that can show it
  (Cron Jobs panel, request logs filtered to `/api/keepalive`, Supabase project activity). It is durable (fleet-scratch, not
  `/tmp`), single-instance (`flock`), and supervised; two bugs of my own were found and fixed on the way — `pkill -f` matched the
  shell that ran it, and a killed watcher's `sleep` child kept the flock so no restart could take it.

## 2b. A uniformly slow network was cycling Error and Live — FOUND, FIXED and re-measured (not deferred)

The claim session's independent browser runs (`BOARD-FINAL-TIP-CHECK.md` item 3) found it precisely: at 2.0 s per answer the label
ended `Live` with 17 % Error duty, but at **3.0 s per answer** it cycled `Error 6.66 s -> Live 8.16 s -> Error 10.66 s -> ...` with
**36 % Error duty** — a fifth of a minute spent saying the read path was down while the server was answering. Cause: a fixed
`PROBE_TIMEOUT_MS = 1500`, so every probe on such a path timed out (Error), its late answer cleared the outage (Live), and the next
probe timed out again.

**Fixed in the hook, not in the test** (`2150fa8`, docs `3780e04`):
- the deadline now follows the connection's own measured round trip: **3x the exponential average of successful probes, clamped to
  1.5-10 s**. A fast link still names a hang at 1.5 s, so the suite's "Error within 2.8 s / 3 s" promises pass unchanged; a 2 s or
  3 s link stops being called down after one measurement.
- a timeout alone is no longer an outage: the request stays alive and, if it answers, the reading is learned from and no outage is
  ever named. The extra wait is spent **only when this connection is already known to be slow** (>= 800 ms round trip) — waiting on
  a fast link pushed "Error within about two seconds" to ~3 s, which the suite's own deadlines caught during development.
- `Reconnecting` now needs a loaded board, so a first read that hangs shows **no word at all** and then Error. This also closes the
  small honesty wart recorded earlier tonight (a never-connected board saying "re-connecting").
- one outstanding request at a time is preserved; the hook's own comment records 2 starts in 9 s with 3.4 s gaps.

My own verification, from a **clean export of the final tip** (`git archive b559e68 -> npm ci -> build`), never the working tree:
unit **121/121** (two of them the new read-deadline tests); full e2e **108 tests, 103 pass, 0 fail, 5 skipped, exit 0** at load 9-12, including the new test that drives 2 s
and 3 s answering paths end to end; agent suite **22 pass / 0 fail / 12 skipped** (live-gated); Gate 1 multi-width lint **PASS**;
the deploy certification script **7/7** on a build of this tip. The price, stated: on a connection already measured slow, a
subsequent *true* hang is named at the grown deadline — up to 10 s instead of ~2 s.

## 3. The deploy packet is now one command, and it already convicts v3

`fleet-scratch/overnight/stage/postdeploy-check.mjs --url <prod> --expect-version <sha>` runs the five checks that matter, read
only: keepalive 200, label blank-then-Live and never Reconnecting, build id, console errors, phone sizing at 390x844, legend
initials only. **Run against live v3 as a control: 4 passed, 2 failed** — `Reconnecting@95-140ms` on a healthy load, and
`fs=14 / row=56 / card=280 wide` against the owner's 12 / 48-54 / 232. The tool detects the exact two defects the branch fixes, on the
deployed build, before it is ever pointed at a deploy. Evidence: `evidence/postdeploy-check-both-halves.txt`.

The other half of the control, which caught two bugs in my own tool: the same script run against a clean export of the tip
(`git archive 7035ab9` -> `npm ci` -> build -> `vite preview`) **passes 7/7** — `fs=12 / row=52 / card=232x44 / no page scroll`,
label `blank@77ms -> Live@109ms`, version `7035ab9`, no console errors, legend initials. The bugs: I asserted the card's *height*
was 232 (232 is its width; on a phone the card is 44 px tall) and I compared the computed `--fs` (`"12px"`) to the string `'12'`.
Either would have made the certification fail the very deploy it exists to bless, which is the argument for running both halves.

Also re-proved at the new tip: clean export builds and passes **unit 119/119** with `RHL_BUILD_ID=7035ab9` and no demo-key leak in
`dist`; Gate 1 multi-width lint **PASS** (7 widths x 2 themes, no overflow, contrast >= 4.5/3, visible focus) —
`evidence/gate1-lint-7035ab9.txt`.

## 4. SimBank: the owner-override input is built and tested in staging (not applied)

`stage/simbank-override/` holds the change, 5 tests and two patches (69 + 89 lines). **Nothing applied**: branch
`proof/simbank-v3-20261103` untouched, filed `scores3.json` still `pass: false`, gate shut, no run, no verdict edited.

Exercised on a **copy of the real record**: no file -> NOT PASSED with the filed figures exactly (56/55/56, control 49, ML-D2,
9 re-read overrides); a staged ruling for `P051-c4fc3f90` -> **PASS**, `problems: []`, one `ownerOverrides` entry, printed
**above** the pass line, ruling file sha256 recorded in `inputs` and equal to the file's own hash; a stale `from`, an invalid
class, an unknown packet are each refused with a named problem. Tests: 48 total, 44 pass, 3 fail — the 3 need the real bank
directory and fail identically in an **unpatched** copy (43/39/3), so they are a copy artefact; all 5 new tests pass.
Write-up and apply commands: `SIMBANK-V3-OVERRIDE-STAGED.md`.

## What the owner would actually see if he says yes (the diff that matters, not the commit list)

1. A healthy load stops claiming a drop: **`Reconnecting` no longer appears for ~120-1750 ms on every page load** (v3 measured
   `Reconnecting@95-140 ms`); the label is blank, then `Live`. A board that has never loaded shows **no word at all** (`data-state`
   `polling`), never "re-connecting".
2. The phone stops being too big: at 390x844, `--fs` **14 -> 12**, row **56 -> 52** px, card **280 -> 232** px wide, no page scroll,
   and the Add sheet, its field and Save all fit inside the visible area with the keyboard up.
3. The legend shows **initials only** (v3 still shows a `title="JA"` tooltip; no name exists either way).
4. A slow-but-working network is not called an outage: the probe deadline follows the connection's own measured round trip
   (1.5-10 s), so a 2 s or 3 s answering link ends at `Live` instead of flickering; a path that stops answering still reaches
   `Error` in about 2 s on a fast link (up to 10 s if the link had already been measured slow).
5. Nothing else changes in what the board shows: same 19 lines, same cards, same editing, same keys, same `/api/keepalive`.
6. The one number the owner should know, already in `docs/OPERATIONS.md`: the indicator's probe costs **~72 requests/min while the
   tab is visible (~35 KB/min, ~4.5 GB/month at 132,000 viewer-minutes)** — about 90 % of the free egress — with the lever stated
   (a 2 s probe halves it, worst-case Error ~3.5 s).

## 5. Correction to §2: the scheduler's firing **is** observable — from the database's own log, not Vercel's

§2 is right about Vercel and wrong about the world: "not observable from this machine" was one endpoint short. The function's own
outbound call is logged by its **target**. `/api/keepalive` issues `HEAD {SUPABASE_URL}/rest/v1/lines?select=id` with
`Prefer: count=exact`, and Supabase's unified logs (collection **API Gateway**, source `edge_logs`) record it as
`HEAD | 200 | https://yhpjpjeedpuvcgnvfpka.supabase.co/rest/v1/lines?select=id | node`.

**Positive control, run at 01:36 ET:** I called `GET /api/keepalive` -> `200 {"ok":true,"status":200}` in 1.01 s at 06:36:59Z;
the dashboard's log shows that HEAD at **06:37:00.544Z**. A second, earlier HEAD sits at 06:14:21Z. So an execution is visible, with
its own timestamp and status, and nothing in the record is row content (method, status, URL, user-agent only).

Why the earlier finding was misleading but not wrong: the **PostgREST** log collection captures errors only and says so
("Check the API Gateway logs for HTTP requests"), so a "No data" page there is not evidence of absence. The Vercel-side conclusions
stand (a known invocation still leaves 0 runtime-log records on this plan; no cron-execution API in ten endpoint families tried,
`v1/projects/<id>/cron-jobs`, `/cron-jobs/executions`, `v9/…`, `v1|v2/deployments/<id>/logs`, `runtime-logs`, `projects/<id>/logs`,
`observability/logs` all 404).

**What the 04:17 check now does differently** (watcher **v5**, replacing v4; both the old watcher and its supervisor were found
dead at 01:29 ET with the lock free, and were re-armed — that is itself the finding: the window would have passed with no artefact):
1. The path proof (`GET /api/keepalive` -> 200) is taken at **03:57 ET / 08:57Z, before the cron window opens**, because a call at
   04:17 writes its own HEAD row into that same log and would make "one row" ambiguous.
2. At 04:17 it re-reads registration and states the query and the discount rule, so the window can be read once, correctly, at
   ~05:20 ET: `select timestamp, event_message from logs where source='edge_logs' and event_message like 'HEAD%' and
   timestamp >= '2026-11-04T09:00:00Z'`. Hobby's window is ±1 h, so any HEAD in **[09:17Z, 10:17Z)** is the scheduler; the live
   board polls with **GET** and never produces HEAD rows.
3. Stated limit: retention on this plan is about an hour, so the read has to happen soon after the window closes; if it ages out the
   answer is "not observable any more", never "did not run".

Evidence: `evidence/keepalive-supabase-edge-log-proof.txt` (the rows, the query, the control, the limits).

## Still waiting (unchanged, not acts)

- **The 04:17 ET scheduler result** — v4 watcher armed; the only place it can be seen is listed in its output.
- **Owner**: deploy yes; the ML-D2 ruling; the second Supabase project; his working key. **Nox**: the SimBank branch disposition.
- The board remainders stay undeployed: live v3 `455a9da` is untouched, and the live page still shows the false
  "Reconnecting" and the rejected phone sizing until it is replaced.
