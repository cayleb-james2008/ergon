# ERGON — correction: the keep-alive check's evidence channel is blind, and what replaces it

Filed 4 Nov 2026, 01:05 ET by the ERGON stream lead (opencode-go/deepseek-v4.1-flash).
Corrects one claim in `reports/ERGON-REMAINDERS-AND-QUOTEFORM-20261104.md` and in the earlier addendum.

## What I said before, and why it was wrong

The earlier filing said the armed watcher's log command was tested and that "runtime logs are function-invocation only, so an
empty pre-cron window is expected and the watcher states that a miss is not proof the cron did not run." The second half was
right by luck; the first half is false. The channel does not show function invocations at all. A watcher built on it can only
ever report "not seen", whatever the cron does.

## The measurement (all commands re-runnable; full transcript in `evidence/keepalive-channel-proof.txt`)

1. **A known invocation.** `GET /api/keepalive` → `200 {"ok":true,"status":200}` in 1.26 s; response headers
   `x-vercel-cache: MISS`, `age: 0` — the function ran, this was not a cached answer.
2. **The window that must contain it.** `vercel logs --scope … --project … --since 10m --json` →
   `{"type":"summary","recordsReturned":0, …}`. Six polls over 90 s, a 30-minute window, and a deployment-scoped window all
   returned the same empty summary.
3. **`--follow` does not stream either**: the CLI answers `{"type":"warning","reason":"deprecated","message":"--follow is
   deprecated and no longer streams logs. Showing a bounded result; remove --follow."}`
4. **Retention is 30 minutes**, so the watcher's own `--since 60m` was an error that the script read as an empty window:
   `--since 1h|12h|7d` → `"The hobby plan does not retain runtime logs for the requested time range."`
5. **The events API is build-only**: `/v3/deployments/<id>/events` holds 52 records — 45 stdout, 7 stderr — newest
   `2026-11-03T08:02:36Z "Build cache uploaded: 415.000ms"`. `/v1/deployments/<id>/runtime-logs` → `not_found`.
6. **No cron-run history is exposed**: the project object's `crons` carries `enabledAt`, `updatedAt`, `deploymentId` and
   `definitions`; `updatedAt` equals the deployment creation time, so it tracks deploys, not runs. `/v1/projects/<id>/crons`,
   `/v9/…`, `/v1/crons/<projectId>`, `/v1/projects/<id>/crons/executions` → `not_found`.

## What replaces it

**Watcher v4** (`stage/keepalive-watch4.sh`, supervised by `stage/keepalive-supervise4.sh`, both running; one instance, flocked,
polls within retention). It reports exactly one of three outcomes:

1. **CRON SEEN** — a runtime-log record names a keepalive invocation or the `vercel-cron` agent inside a window that began
   before the check. (Possible only if Vercel's log channel comes back; it is polled anyway, cheaply.)
2. **CHANNEL WORKS, NO CRON RECORD** — the channel demonstrably returns records, but none names the cron. Stated as such.
3. **UNPROVABLE FROM THIS SEAT** — after proving the channel blind with a known invocation (the guard in §3), it says so and
   files what *is* proved instead:
   - **Registration**, re-read at check time from the project object: `crons.enabledAt` 2026-11-02T10:42:37 ET, one definition
     `{schedule: "17 9 * * *", path: "/api/keepalive", host: rhl-warehouse-board-2ttt1y84e-caylebjamesportfolio.vercel.app}`,
     and `vercel inspect https://rhl-warehouse-board.vercel.app` resolving to that same `dpl_busDKSQZ6wZKggot9EvwyWBemjpR`. So
     the schedule is bound to the build that serves production, not to a branch.
   - **The endpoint answers after the scheduled minute** — `200 {"ok":true,"status":200}` with a cache MISS, i.e. the function
     ran and Supabase answered its count-only HEAD (project not paused, key works).

The honest gap that remains, named rather than papered over: **nothing reachable from this seat records that the scheduler's
timer fired.** The one act that would exercise Vercel's cron runner end to end is `vercel crons run /api/keepalive`
(a manual trigger — same read-only count HEAD, no data change). It is staged, not run, because it would not be the scheduled
run and should not be recorded as one.

**Recommendation (one line, not an act):** the board's availability now rests on a single scheduler whose firing this seat
cannot observe. If that is unacceptable, the cheapest second path is a daily `GET /api/keepalive` from any other
always-on host — the endpoint is a count-only HEAD, safe to call from anywhere, and it would make the pause clock's upkeep
independent of one unproven timer. No such caller was added: it is the owner's/Nox's call where it lives.

## Also on the record
- **Two ERGON-identity sessions are awake.** This session's first repo read at 00:40 ET found six modified test files; at
  00:48:34 they were committed by another ERGON session as `d9c782b` (not by a command here). No work was lost — that commit is
  the same determinism work this session had already read and was verifying — but the split is now written down in
  `~/fleet-scratch/overnight/CLAIM-ergon-20261104.md`: that session owns the repo commits; this one owns the keep-alive
  correction, the independent check of the settled tip, and the morning record. Nothing was edited in the repo by this session
  after that.
- **Host hygiene, because it was the cause of the flapping tests**: 15 orphan `vite preview` servers (19–37 h old, reparented
  to systemd) and 24 stale headless chromium/playwright processes were stopped. The full e2e then ran **108 tests, 103 pass,
  0 fail, 5 skipped, exit 0** at 1-minute load ~40 with the suite's own host reference at 103 ms (41 ms on a quiet host) —
  the same conditions that produced the checker's three failures earlier.
