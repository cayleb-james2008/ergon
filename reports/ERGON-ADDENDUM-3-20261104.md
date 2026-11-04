# ERGON addendum 3 — corrections, the tip that moved, and the live-stack gap partly closed a different way

ERGON stream lead (opencode-go/deepseek-v4.1-flash), 4 Nov 2026 01:50 ET / 06:50Z. Continues
`reports/ERGON-REMAINDERS-AND-QUOTEFORM-20261104.md`, `reports/ERGON-ADDENDUM-20261104.md`,
`reports/ERGON-ADDENDUM-2-20261104.md`. Nothing deployed, nothing pushed, no run made, no hosted state touched.

---

## 1. Correction: the third caveat on `7035ab9` was filed as fixed and it was not

The re-check of `7035ab9` (`reports/20261104-board-flakefix-check.md`, finished 01:30) judged the hung-path assertion **FAIL, both
ways**:

- It **failed correct code**: "no two probe starts within 1500 ms" is a schedule assumption, and in **1 run of 12** on unmodified
  code the starts were `[682, 1009, 3993]` ms — the indicator's probe and the store's reconcile read are independent callers and
  legitimately began 327 ms apart.
- It **passed a real defect**: an injected duplicate probe 1.6 s after setup (`[670, 2270, 3991]`) sat inside the gap rule.

Addendum 2 said this caveat was closed. It is corrected in place (§1's table row and text, plus a new §1b), and the morning section
carries the same correction. The tree now carries a rework (test-only, uncommitted at the time of writing): **after the first 5 s of
the 9 s window, at most one start**, i.e. a legitimate setup pair is allowed and a pile-up is not. My arithmetic check against the
checker's own sequences (`evidence/gaprule-arithmetic-check.txt`): the pair that broke the old rule passes, the measured sequences
pass, one-probe-per-interval fails (2 late starts), two per interval fails (5). Named limit in the test's own comment: it does not
catch a single duplicate probe 1.6 s after setup, and the name now claims only what it measures.

## 2. The tip moved twice after the filings were written, and the deploy certification was re-run for the tip that exists

- `3780e04` (docs, connection honesty) -> **`9cc7d95` at 01:38** ("Data: a read that never answers is an outage, not a frozen board
  claiming Live"), and after that only **tests and docs** (`26e6c9e` at 01:44 for the hung-path rule, `b559e68` at 02:0x for the
  measured-twice 60-job bound and the "about 11 s" docs figure). **So the deploy artifact is unchanged since `9cc7d95`**, and the
  certification below still describes it. That is not an assertion: building `9cc7d95` and the tip with the same `RHL_BUILD_ID` gives
  a **byte-identical `dist` tree** (hash `19ad0173…`). The rule that keeps this true, checkable in one line before deploying:

      git diff 9cc7d95..$(git rev-parse HEAD) --name-only | grep -v '^docs/' | grep -v '^test/'   # must print nothing The second independent check found the
  health probe asks `board_token`, a **different RPC** from the board read,
  so with the data endpoints hung and the probe answering the label could sit **Live for 26 s over a frozen board**. The fix: a
  15 s deadline on the board read (it rejects, the store folds it into `fetchFail`) and single-flight so a dead path cannot pile up
  requests while the deadline runs out.
- **Certification re-run from a clean export of `9cc7d95`** (`evidence/deploy-cert-9cc7d95.txt`): `npm ci` exit 0, demo build,
  `stage/postdeploy-check.mjs --expect-version 9cc7d95 --local` -> **7 passed, 0 failed** — label `(blank)@90ms -> Live@139ms`,
  `data-version=9cc7d95`, no console errors, `fs=12px row=52px card=232x44px pageOverflow=0px`, legend initials only. Production
  build of the same tip: exit 0, **0 files** leak `__rhlDemo|demo-employee|service_role|sb_secret`, and the commit id is baked into
  the bundle so the version tooltip is checkable after a deploy.
- Every filing that named `3780e04` now says "the tip moved; re-read it before deploying". The deploy packet's command is unchanged
  in shape: `vercel deploy --prod --build-env RHL_BUILD_ID=<tip>` then `postdeploy-check.mjs --expect-version <tip>`.

## 3. `9cc7d95` checked independently, and it has one residual worth knowing

From a clean export, never the working tree (`evidence/board-9cc7d95-lead-probe.txt`): **unit 121/121**; a hung read rejects at the
deadline after exactly 2 queries; **two concurrent callers share one read** (2 queries, same object — a dead path cannot pile up);
after a timeout the next read is fresh; **0 unhandled rejections**; no timer left behind. The property the commit is about is closed
in the reducer, not only in the test: `status()` in `src/core/connection.js` returns `error` whenever `failures > 0`, **before** it
consults `realtime === 'SUBSCRIBED'`, so an answering probe cannot hold the label at Live over a dead read.

**Residual (reported, not fixed — commits are the other session's):** `load()` joins and **drops the joining caller's options**.
Measured: `load({lines:false})` outstanding, then `load()` -> the full read returns `{lines: null, entries}` with **no lines query
at all**. In `store.js` (`if (data.lines) lines = data.lines`) the old lines are kept, so this is freshness (<= 3 s), not a wrong
board and not a false claim. Fix suggested: join only when the joiner does not need more than the in-flight read is fetching.

## 4. The five live-gated e2e cases: three of them proven a cheaper way, two still unrun

The full local stack was attempted six times tonight (four by the other ERGON session, two by me) and every attempt died the same way:
`sh: can't fork: Resource temporarily unavailable` inside `supabase_db_board`, then
`HealthCheckTimeoutError ... container is not running: exited`. **My first reading of that as "host pressure" was wrong, and I say so
because it was disproved by the diagnosis below.** A fifth attempt was aborted within a minute when a concurrent `supabase start` from
the other session appeared (two stacks on the same project id and ports would have wasted both runs); the sixth was mine alone.
`evidence/supabase-start-blocker-diagnosis.txt` records what was ruled out, one measured thing at a time: the host forks fine (100
rapid forks, 0 failures), podman runs a container fine, the image forks fine inside itself, the data volume is fine (deleted and
re-initialised: `initdb ... Success`), the CLI's **exact** `Entrypoint`/`Cmd`/`Env` run verbatim against a **fresh empty volume**
works end to end (`pg_cron scheduler started`, 0 fork failures) even with its healthcheck, port mapping and network added, and no
cgroup, memory, pids or socket limit is anywhere near its ceiling. So it is something inside `supabase start`'s own orchestration, and
the diagnosis stops there deliberately — the next step is reading the CLI's Go source under a debugger, which is not a good trade at
02:00 for two tests.

What the five skipped cases actually guard is the **server-side** safety core, and that can be proven without the browser, the
realtime service or the stack. Done directly against the shipping migration on a throwaway Postgres 17 pod (the pod was removed
afterwards; the script is `evidence/ergon-sqlproof.sh`, output `evidence/ergon-sqlproof-run.txt`):

| live case | proven here | answer |
|---|---|---|
| stale revision refused | `save_entry(expected_revision: 9)` against revision 1 | `{"ok":false,"error":"conflict","revision":1}` |
| control: the right revision is accepted | same entry, `expected_revision: 1` | ok, revision **2** |
| deleted job never recreated | delete, then `save_entry(expected_revision: 2)` | `{"ok":false,"error":"not_found"}`, rows with that id **0** |
| revoked key refused | `revoke_access_key`, then `check_key` | `key_invalid` — **identical** to an unknown key (no oracle) |
| expired key refused | key moved one minute into the past | `check_key` -> `key_expired`, `save_entry` -> `key_expired` |
| (extra) app binding | key minted for another app | `key_invalid` |
| (extra) the key table is not API-readable | `set role anon; select from access_keys` | `permission denied`; `lines` readable |
| (extra) the probe's premise | `board_token()` before/after a write | `{"e":1,...,"r":1}` -> `{"e":2,...,"r":2}`, changed |
| (extra) only hashes are stored | scan `key_hash` | every row a proper sha256; plaintext never present |

**Still unrun, and named as such:** realtime delivery between two devices, and the polling fallback with the websocket blocked.
Those two need the live stack and are the honest remainder of this item. They are not "skipped and forgotten"; they are the first
thing to run when the host is quiet.

**A second, independent data-layer proof exists beside this one** (the other ERGON session, same night): the repo's own SQL/REST
contract suite, 32 assertions, run against a plain `supabase/postgres` container + `postgrest` + a locally minted anon JWT —
**30 passed, 2 failed, and both failures are harness limits** (no realtime service in that rig; `supabase_read_only_user` is reserved
so its password cannot be set). It covers RLS and grants, the key lifecycle, `save_entry`/`remove_entry`/`board_token`, publication
guards, re-applying the migration over live data, apply-twice structure equality and atomic failure on a bad column
(`evidence/board-sql-contract-20261104.log`). Two different rigs, two different assertion styles, the same conclusion about the
server-side safety core — which is the reason to trust it.

## 5. Keep-alive: the watcher was dead, and the question became answerable instead of merely honest

- At 01:29 ET there was **no watcher and no supervisor**, and the flock was **free** — absence, not a stale lock. The last line in
  its output was "another watcher holds the lock" from 01:00. Had this not been checked, 04:17 would have produced **no artefact**
  and that silence would have read as "the cron did not fire".
- **And it happened again at 02:19, when the OpenCode server restarted and took the session's whole process tree with it.** Twice in
  one night, and both times the only thing that revealed it was testing the flock (`flock -n` succeeding), not a pid file or a log.
  The check is now owned by **systemd**, not by a session: `systemd-run --user --unit=ergon-keepalive-guard --property=Restart=always
  --property=RestartSec=300`, active, with the watcher's own `flock` still preventing a double run. A third restart can delay the
  04:17 write by at most five minutes; it can no longer lose it. (The harness lesson is filed in the h2 library:
  `host-ops/2026-11-04-timed-check-must-not-live-in-the-session-tree`.)
- Re-armed at 01:31, then replaced with **v5**: the path proof (`GET /api/keepalive` -> 200) now happens at **03:57 ET**, *before*
  the cron window, because a call at 04:17 writes its own HEAD row into the same log that is being used as evidence.
- **The execution is observable after all — from the database side.** `/api/keepalive` issues
  `HEAD {SUPABASE_URL}/rest/v1/lines?select=id`; Supabase's API Gateway log (source `edge_logs`) records it. **Positive control:**
  my call at 06:36:59Z produced `2026-11-04T06:37:00.544Z  HEAD | 200 | .../rest/v1/lines?select=id | node`. The earlier "not
  observable from this machine" (addendum 2 §2) was one endpoint short, and §5 there now says so.
- Hobby's window is +/- 1 h, so the expected fire is **[09:17Z, 10:17Z)**. Retention was **measured, not assumed**: the oldest row in
  the collection at 06:52Z was 05:53:53Z, i.e. **~58 minutes**, so a single read at 10:20Z would already have lost an early fire.
  The window is therefore read **four times** — ~09:22, 09:45, 10:05, 10:25Z (04:22, 04:45, 05:05, 05:25 ET) — with
  `select timestamp, event_message from logs where source='edge_logs' and event_message like 'HEAD%' and timestamp >= '2026-11-04T09:00:00Z' order by timestamp asc limit 100`
  and the **union** of the four reads covers the whole window whatever the fire time. Each result is filed with its timestamps; if
  every read is empty the verdict is "not observed in the window", and a read that happens after the rows aged out says **"not
  observable any more", never "did not run"**.
- **A trap that would have produced a false "did not fire", found and fixed before the read:** the query had no `LIMIT`, and the Logs
  Explorer **refuses** such a query — but only after Run is pressed, leaving the previous run's "No results found" pane on screen. So
  a refusal could have been read as an empty result. The query now ends `order by timestamp asc limit 100`, and the reading rule is:
  **a valid run prints `Results (N)` in the header**; a refusal, an error or a missing header means the query did not run, never "zero
  rows". The sibling session's watcher v6 found the same trap independently.

**The trade this makes, stated because the third check measured it:** with the data path dead and the probe answering, the label is now
**blank for 15 s** and then `Error` (measured: `88 ms label="" data-state=polling` -> `15088 ms label="Error"`). Before, that same
condition produced a false `Live`. So the 26 s false claim became 15 s of saying nothing followed by a sticky, honest `Error` — better,
and still 15 s in which a person looking at the board learns nothing. That is a product choice the owner can overrule by shortening the
deadline, at the price of calling genuinely slow reads outages.

## 6. The final connection check passed with caveats, and its two caveats belong in the record

`BOARD-FINAL-CONNECTION-CHECK.md` (finished 01:45, on `3780e04`) reproduced the two named defects gone — **37.1 % -> 7.4 % Error
duty at 3 s per answer**, `Reconnecting` before a board no longer rendered — and named two things that are not defects but are
prices, which the owner should see before saying yes:

1. **A bounded false "Live" became possible, and it scales with the link.** "Slow once, then dead": 11.0 s of `Live` at 2.5 s
   answers where the old build showed 2.5 s. The docs say "up to 10 s"; the measured worst case is ~11-12 s (10 s clamp + <=1 s
   tick + ~1 s of would-be late answer). That is the trade the owner is being asked to accept, stated in one line: **a link that is
   slow and then dies is named an outage about 3.2x later than before, up to ~12 s.**
2. **The token-vs-data false Live was real and pre-dates the branch** — the check could only say ">= 15 s of Live" because that
   was its window; it was fixed after that check ran, in `9cc7d95`, which the check did not test (the third narrow check did:
   `BOARD-DATA-READ-CHECK-3.md`, **PASS**).
3. **The suite still is not unconditionally green under load.** In that check's re-runs, the "update cost" pair failed **2 of 4
   runs** at load 14-22 — 102 ms and 112 ms against the 100 ms shape bound. Nothing product-related (the same code passes when the
   host is quiet), but "0 fail" is a property of a quiet-ish host, not of any host, and anyone re-running the suite during a busy
   morning should expect that one pair to be the first casualty.

## What is still blocked (unchanged, not acts)
- **Owner**: the one yes to deploy (`26e6c9e`; product code identical to the certified `9cc7d95`), the ML-D2 ruling, the second
  Supabase project for the showcase, his working key on the real board.
- **Nox**: the SimBank branch disposition (14 commits, unpushed).
- **Host**: the two realtime live cases above.
