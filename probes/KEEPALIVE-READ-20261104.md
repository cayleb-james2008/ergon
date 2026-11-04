# KEEP-ALIVE READ LOG — 4 Nov 2026, window [09:00Z, 10:00Z)

Filled in by whichever ERGON session is awake at each time. **Do not write a second verdict**: add rows to the slots below and let one
closing entry state the answer. The artefact `keepalive-first-run.txt` (watcher v6.3) carries the registration snapshot, the path proof
taken at ~08:57Z, the list of **every call the watcher made**, and both channels with their controls.

**Precondition:** the artefact is v6.3 text only if it contains `DISCOUNT RULE (v6.3)` and `TRAP 3`. If it does not, an older process
wrote it — use the rule below anyway and say so in the verdict.

Two independent channels. A fire should appear in **both**; a row in one and not the other is a finding to name, not a pass.

## Channel A — Vercel Request Logs (read this first: it names the caller)

`https://vercel.com/caylebjamesportfolio/rhl-warehouse-board/logs?requestPaths=%2Fapi%2Fkeepalive`

- **Select view → Request Logs.** The default, Runtime Logs, shows nothing for this function because it writes no stdout (measured: four
  known calls appear in Request Logs to the second while Runtime Logs for the same window says "No Runtime Logs Match Your Filters").
- **Range: the default 30-minute view is the one that is proven.** Read it at each of the four read times; four 30-minute windows at
  09:22 / 09:45 / 10:05 / 10:25Z cover the window continuously. Wider ranges are offered (12 h, day, week, 2 weeks) but no row older
  than 30 minutes has ever been observed here, so do not rely on one late read; the custom range is not verified at all.
- A fire looks like `NOV 04 HH:MM:SS  GET 200  rhl-warehouse-board.vercel.app  /api/keepalive`.
- **Click the row**: it shows User Agent, Search Params, Request ID, region and the Deployment ID that answered. My controls read
  `curl/8.23.0` with `lead-control=`/`before-window=` in Search Params, on deployment `dpl_busDKSQZ6wZKggot9EvwyWBemjpR`, production.
  **Vercel documents that a cron-triggered function always receives `user-agent: vercel-cron/1.0`** (and an
  `x-vercel-cron-schedule` header), so the scheduler row is the one whose **User Agent is `vercel-cron/1.0`** — read it, do not infer
  it. Channel A prints times in **local (GMT-5)** — add 5 h before comparing with channel B, which is UTC.

## Channel B — Supabase edge_logs (corroborates, and proves the database answered)

Open the bookmark and **press Run**. Record only what the table shows after Run: it must contain the 08:57Z control row. The range starts at **08:45Z** on
purpose: the 08:57Z control row must be visible, and a pane that cannot show a row it should show is not evidence of absence.

```
https://supabase.com/dashboard/project/yhpjpjeedpuvcgnvfpka/logs/explorer?q=select%20timestamp%2C%20event_message%20from%20logs%20where%20source%20%3D%20'edge_logs'%20and%20event_message%20like%20'HEAD%25'%20order%20by%20timestamp%20asc%20limit%20100&its=2026-11-04T08%3A45%3A00Z&ite=2026-11-04T10%3A30%3A00Z
```

```sql
select timestamp, event_message from logs where source = 'edge_logs' and event_message like 'HEAD%'
  order by timestamp asc limit 100   -- range 2026-11-04T08:45:00Z .. 2026-11-04T10:30:00Z
```

Discount rule (v6.3) — **attribute by the call, never by a count**: the artefact's section 2 lists every `launch` and every `call` line
with the exact UTC second. A HEAD row within 30 s of a `call` line is ours. A HEAD row further than 30 s from any logged call is the
scheduler. (A `launch` with no following `call` made no request — do not invent one.) A relaunch before 04:16 makes another call, and it
will appear in that list.

Traps, measured 07:2xZ and again at 03:05 ET:
- a query without `limit` is refused, and the refusal is shown only after Run;
- **the pane keeps the previous run's content until Run finishes, and an executed-but-empty query shows exactly the same words —
  "No results found" — as a stale pane.** So the header is not the proof. This query always contains the 08:57Z control row, so a
  correct run must show a table with at least that row; a bare "No results found" means the query did not run, or ran with a
  different range. **Do not record zero from it.**
- a range that starts at 09:00Z excludes the control row (hence 08:45Z).
Channel B's visible span is about an hour, which is why it is read four times.

## Channel C — Supabase Observability → Data API (corroboration, counts only)

`https://supabase.com/dashboard/project/yhpjpjeedpuvcgnvfpka/observability/postgrest`, window **Last 60 minutes**. The board's reads are
`GET /rest/v1/lines` and `GET /rest/v1/entries`; the keep-alive is the only `HEAD /rest/v1/lines?select=id`, so the HEAD count is a
keep-alive-specific counter (it read 6 at ~07:40Z for the known calls of that hour). Count-based, no timestamps: it corroborates, it
never decides.

## Slot 0 — 09:12-09:14Z (04:12-04:14 ET): rehearsal read, INSIDE the hour, before the artefact

Purpose: prove the whole read path on the known 08:57Z event **before** the scheduled minute, so a later empty pane cannot be read as
"the query did not run". Both channels answered on the first attempt.

- **A** read at **09:12Z** (Request Logs view confirmed in the "Select view" control; Timeline `Last 30 minutes`, so the visible range is
  ~08:42-09:12Z after Refresh Logs). **Exactly one row**: `NOV 04 03:57:00 .16 GMT-5` = **08:57:00.16Z**, `GET 200`
  `rhl-warehouse-board.vercel.app` `/api/keepalive`. Row opened: **User Agent `curl/8.23.0`**, **Search Params `before-window=1793782620`**,
  Request ID `m4hhd-1793782620165-83dc6f985503`, region iad1, Function Invocation 947 ms, `dpl_busDKSQZ6wZKggot9EvwyWBemjpR`, production.
  → the one row in range is **ours** (the 03:57/08:57 path proof). No scheduler row up to 09:12Z.
- **B** Run pressed **09:14Z**, range `04 Nov, 03:45 - 05:30` (picker shows ET = 08:45-10:30Z): **`Results (1)`** —
  `2026-11-04T08:57:00.758000` · `HEAD | 200 | https://yhpjpjeedpuvcgnvfpka.supabase.co/rest/v1/lines?select=id | node`.
  → control row visible, 758 ms after the call line at 08:57:01Z. No other HEAD row in range up to 09:14Z.
- **C** not read at slot 0 (count-only, corroborative; read from slot 1 on).
- note: **two operational traps met and solved, both of which would have produced a false "no results"**:
  1. loading the bookmark URL seeds the editor **truncated to `select tim`** (10 chars) — the query must be typed in: click the editor,
     `Control+A`, `Backspace`, then type the full query (`Meta+A` does nothing on this host). A `fill` on the hidden Monaco textarea sets
     the DOM value but not the editor model, so it does not fix it either.
  2. after `Run` the pane keeps "No results found" for **10-20 s** and the `Results (N)` header appears late. The verdict must be read
     from the header **`Results (N)`**, never from the pane's wording — exactly TRAP 2, now measured with a duration.

## Slot 1 — 09:22Z (04:22 ET)
- A: read at **09:18Z** (corrected: the clock was checked afterwards; this entry first said 09:23Z, which was wrong), Request Logs view, default 30-min window (03:49–04:19 ET), filter `requestPath:/api/keepalive`.
  Rows in the window: **1** — `NOV 04 03:57:00.16  GET 200  rhl-warehouse-board.vercel.app  /api/keepalive`, which is the artefact's
  own 08:57:01Z probe (0.84 s earlier in the row than the log line, same second). The window ended at 04:19 ET and showed no row
  at/after 04:17 — but with one minute of ingestion lag that is **not** evidence of absence; the later slots decide. UA not opened — the row is outside the window either way, and any row inside it would be the scheduler's by construction
  (the only call this watcher made was 08:57:01Z, which is 20 minutes before the window opens).
- B: read at **09:20Z** (corrected from 09:26Z), bookmark query run (pressed Run; header went `Results (1)`, so the query executed). Control row **visible**:
  `2026-11-04T08:57:00.758000  HEAD | 200 | .../rest/v1/lines?select=id | node`. **No other HEAD row** up to 09:20Z.
- C: not read (B already at `Results (1)` with the control visible; C is corroboration only).
- note: this is **1–3 minutes** into the scheduled hour (correction: an earlier draft of this entry said 4–9), so it is only the
  baseline: it proves both channels can see a known row. It says nothing yet about the fire.
  Hobby may fire anywhere inside the hour, so the later slots decide.

### Slot 1 corroboration — 09:22-09:23Z, second session, by a stronger method (not a second verdict)

The other ERGON session read slot 1 at 09:18-09:20Z. This session read the same window three minutes later **a different way**, and it
agrees; the difference is worth keeping because it removes two of the traps permanently.

- **A: 09:22Z**, Request Logs, Refresh Logs, graph axis after refresh **03:53-04:23 GMT-5 = 08:53-09:23Z**: **1 row**, the same
  `03:57:00.16` = 08:57:00.16Z control (its detail was already opened at slot 0: UA `curl/8.23.0`, `before-window=1793782620`).
  **No scheduler row in [08:53,09:23)**.
- **B: Run pressed 09:22:30Z** — then the **query's own HTTP response** was read instead of the rendered table:
  `GET api.supabase.com/platform/projects/yhpjpjeedpuvcgnvfpka/analytics/endpoints/logs.all.otel?sql=…source='edge_logs'…HEAD%…limit 100`
  `&iso_timestamp_start=2026-11-04T08:45:00Z&iso_timestamp_end=2026-11-04T10:30:00Z` → **HTTP 200**, body
  `{"result":[{"event_message":"HEAD | 200 | …/rest/v1/lines?select=id | node","timestamp":"2026-11-04T08:57:00.758000"}]}`
  → **one row, the control**. The URL itself carries the SQL and both bounds, so a stale pane and a wrong range are both impossible,
  and the HTTP status is part of the record. **Adopted for slots 2-4.**
- **C: 09:23Z**, Data API, window **08:22-09:22Z**: Total Requests **1** (`HEAD 200 /rest/v1/lines?select=id`, 08:57, avg 581 ms),
  Response Errors **No data**.
- Read: **no fire in [09:00,09:23)Z** on any of the three channels.

## Slot 2 — 09:45Z (04:45 ET)
- A: read at **09:46Z**, Request Logs view, default 30-min window **04:16–04:46 ET (09:16–09:46Z)**, filter
  `requestPath:/api/keepalive`: **"No Request Logs Match Your Filters" — 0 rows.** The 08:57Z probe is outside this window, so this
  is a clean zero for the 29 minutes after the scheduled minute.
- B: read at **09:45Z**, Run pressed, header **`Results (1)`** (query executed): the only row is the control
  `2026-11-04T08:57:00.758Z HEAD | 200 | …/lines?select=id | node`. **No scheduler HEAD row** anywhere in [08:45Z,09:45Z].
- C: not read.
- note: both channels agree — no request in the first 29 minutes of the hour. Hobby may still fire up to 10:17Z, so this is not the
  verdict; slots 3 and 4 cover the rest of the hour.
  **CORRECTION to that note (second session, 09:48Z): "up to 10:17Z" is the superseded model.** Vercel's *Managing Cron Jobs ->
  Cron jobs accuracy* (quoted in the artefact's header and in watcher v6.3): "Vercel may invoke these cron jobs at any point within the
  specified hour ... an example expression like `0 8 * * *` could trigger an invocation anytime between `08:00:00` and `08:59:59`".
  For `17 9 * * *` the hour is **[09:00:00Z, 09:59:59Z)**, not [09:17,10:17). So a request at 10:05Z would be **FIRED OUTSIDE THE
  DOCUMENTED HOUR** — a finding about the scheduler, not a late-but-normal fire — while a request at 09:05Z would be normal, not early.
  Slots 3 and 4 still cover the tail (10:25Z's 30-minute window reaches back to 09:55Z), so the reading is unchanged; only the label
  would have been wrong.

### Slot 2 corroboration — 09:44-09:47Z, second session, independent instruments (not a second verdict)

Agrees with the slot-2 read above, by different means, and adds the two facts that make an empty pane readable:

- **A, 09:44:33Z** (the refresh stamps `refreshedAt=1793785473103`): axis **04:15-04:45 GMT-5 = 09:15-09:45Z** and the panel says
  **"No Request Logs Match Your Filters"** — an explicit empty state under a *current* axis, which is a different thing from a stale
  pane. The 08:57Z control is 48 minutes old at this read and therefore outside the 30-minute window by arithmetic; the view's ability
  to show a row was proved three times in this session (09:12, 09:22, 09:35).
  **New trap, measured:** at **09:35Z** the refresh moved the axis to 09:05-09:35 but the row list **still displayed the 08:57 row**,
  i.e. the list can lag one refresh. So the recorded fact is "empty state + current axis", never a row count taken mid-refresh.
- **B, 09:47:24.930Z**, HTTP **200** in 1.76 s, sent from the page to the dashboard's own analytics endpoint with the page's session
  (same SQL, `iso_timestamp_start=2026-11-04T08:45:00Z`, `iso_timestamp_end=2026-11-04T10:30:00Z`):
  `{"result":[{"event_message":"HEAD | 200 | …/rest/v1/lines?select=id | node","timestamp":"2026-11-04T08:57:00.758000"}]}`
  → **only the control**. Reason for the change: the Explorer's Run button stopped issuing queries after its first successful run
  (measured: two clicks and a Ctrl+Enter produced no new request), so the query is now sent directly, which is the same call the
  dashboard makes and is timestamped rather than cached.
- **C, 09:47Z**, Data API window **08:47-09:47Z**: Total Requests **1** (`HEAD 200 /rest/v1/lines?select=id`, 08:57), Response
  Errors **No data**.
- Read: **no fire in [09:15,09:47)Z.** With slot 1, the hour is clean from 09:00 to 09:47.

## Slot 3 — 10:01-10:03Z (05:01-05:03 ET) — **FOUND IT: THE CRON FIRED AT 09:50:42Z**

Read at **10:01:04Z** (channel A refresh, `refreshedAt=1793786464005`): axis 09:32-10:02Z, **"No Request Logs Match Your Filters"**.
Thirteen seconds later a second refresh (`refreshedAt=1793786477891`) returned **one row**. The row had been ingested between the two
refreshes — **~10.4 minutes after the request itself** (see the lag note below). If I had stopped at the first empty read I would have
written NO FIRE from a true row that simply had not arrived.

- **A: `GET /api/keepalive` at `NOV 04 04:50:42.07 GMT-5` = 2026-11-04T09:50:42.07Z, status 200.** Row opened:
  - **User Agent: `vercel-cron/1.0`** — the signature Vercel documents for a cron-triggered invocation.
  - **No Search Params** (our calls always carry `before-window=`/`lead-control=`).
  - Host **`rhl-warehouse-board-2ttt1y84e-caylebjamesportfolio.vercel.app`** — the exact host `vercel crons list --json` reports for
    this job; our own calls went to the production alias `rhl-warehouse-board.vercel.app`.
  - Request ID `8l7rn-1793785842074-582ac4002ec5`, region iad1, Firewall Allowed, **Function Invocation: Execution Duration 916 ms**,
    Response finished in 1.2 s, **External APIs: `HEAD yhpjpjeedpuvcgnvfpka.supabase.co/rest/v1/lines`**,
    Deployment `dpl_busDKSQZ6wZKggot9EvwyWBemjpR`, environment production. Screenshot: `evidence/keepalive-fire-0950Z-vercel.png`.
- **B: `HEAD 200 https://yhpjpjeedpuvcgnvfpka.supabase.co/rest/v1/lines?select=id` at `2026-11-04T09:50:42.693000`** — **623 ms** after
  the Vercel request started, i.e. the function did reach the database and the count was answered. Read at **10:01:12.741Z**, HTTP 200,
  body = exactly two rows: the 08:57:00.758Z control and this one.
- **C: Data API, window 09:01-10:01Z (Nov 4 4:01am-5:01am ET): Total Requests `1`** — `HEAD 200 /rest/v1/lines?select=id` at
  **09:50**, avg 559 ms; Response Errors none. The count is keep-alive-specific (the board's own reads are GET, and nobody had it open).
  So exactly **one** keep-alive request in the whole hour, and it is the cron's.
- **Reading:** **FIRED at 2026-11-04T09:50:42Z**, inside the documented window **[09:00:00Z, 09:59:59Z)**. All three channels agree, and
  they agree on the *same second* (A 09:50:42.07, B 09:50:42.693). The nominal minute is 09:17; 09:50:42 is 33m42s later, which is
  what Vercel's "any point within the specified hour" means on Hobby — a normal late fire, not a miss and not a defect.
  Because B logged a `HEAD … 200`, the function's body was `{"ok":true,"status":200}`, not the `{"ok":true,"skipped":…}` shape: the
  database was actually touched.
- **NEW LIMIT, and it invalidates a strict reading of the four-read plan: Vercel Request Logs ingest with a lag of ~10 minutes here.**
  Measured: request at 09:50:42, absent from the "Last 30 minutes" view at 10:01:04, present at 10:01:17. So an A read is only
  evidence about the period that ended ~10 minutes before the read; the last read of the hour must happen **after** 10:10Z to be able
  to show a late fire. My slots 1 and 2 are unaffected (they reported windows that ended at 09:23 and 09:45, and the fire is at
  09:50:42), but "A was empty at 10:01" would have been a false NO FIRE. Channel B has no such lag: it showed the new row 623 ms after
  the call, in the very first read after it.
- Slot 4 (10:25Z) still runs, to close the tail and to confirm nothing fired after 09:50:42.

## Slot 3 CORROBORATION — independent second reader, 10:06-10:08Z

The other ERGON session read A and B at 10:01Z and found the fire; a second reader reproduced it six minutes later without seeing
that entry first, and the two agree to the second:

- **A** (fresh Request Logs page, default 30-minute window 04:36–05:06 ET = 09:36–10:06Z, filter `requestPath:/api/keepalive`):
  **1 row**, `NOV 04 04:50:42.07  GET 200  rhl-warehouse-board-2ttt1y84e-…`. Row opened: **User Agent `vercel-cron/1.0`**,
  Request ID `8l7rn-1793785842074-582ac4002ec5`, Deployment `dpl_busDKSQZ6wZKggot9EvwyWBemjpR`, production, response 1.2 s.
- **B** (bookmark, Run pressed at 10:08Z): **`Results (2)`** — control `2026-11-04T08:57:00.758Z` **plus**
  `2026-11-04T09:50:42.693Z  HEAD | 200 | …/rest/v1/lines?select=id | node`.
- Agreement: A 09:50:42.07Z and B 09:50:42.693Z — 0.623 s apart, the same request. This is the corroboration the plan asked for, from
  a different reader, a different tab and a different minute, and it changes nothing: **FIRED at 09:50:42Z**.
- I also hit the ingestion lag the slot-3 note describes, from the other side: my 09:46Z A read showed **0 rows** over
  09:16–09:46Z, which was true of the data at that moment but would have been a false "no fire" if the fire had been at 09:40.

## Slot 3 NOTE — why this slot is stamped 10:01-10:03Z and not 10:05Z

The scheduled read time was 10:05Z. I read at 10:01:04 and again at 10:01:17 because the hour closes at 09:59:59Z and a read as soon
after it as the instrument allows is the one that covers the whole hour. Both reads, plus the 10:01:12Z channel-B query and the 10:02Z
channel-C report, are recorded above. The 10:05Z read is therefore the same evidence, taken four minutes earlier.

### Slot 3 confirmation — 10:09:05Z (lag hypothesis tested, not assumed)

Refresh at **10:09:05Z** (`refreshedAt=1793786945686`), axis **04:40-05:10 GMT-5 = 09:40-10:10Z**: **exactly one row**,
`04:50:42.07` = 09:50:42Z, detail panel still showing **User Agent `vercel-cron/1.0`** and the External API `HEAD …/rest/v1/lines`.
So the row that was missing at 10:01:04Z is stably present eight minutes later, which is what a ~10-minute ingestion lag predicts and
what a "wrong filter/range" explanation would not. It also shows **nothing fired between 09:50:42Z and 10:09Z** — the tail of the hour
is clean.

## Slot 4 — 10:24Z (05:24 ET) — tail check: nothing fired after 09:50:42Z
- A: refresh at **10:24:19Z** (`refreshedAt=1793787859625`), axis advanced to **04:55-05:25 GMT-5 = 09:55-10:25Z**:
  **"No Request Logs Match Your Filters" — 0 rows.** The fire (09:50:42) is outside this window by five minutes, so a zero here is
  the expected reading, and it means **no second request in [09:55,10:15)Z** (the last ten minutes of the window are inside the
  ~10-minute ingestion lag and are not claimed).
- B: **10:24:32.543Z**, HTTP **200** in 737 ms, range 08:45-10:30Z: body = **exactly two rows** —
  `2026-11-04T08:57:00.758000` (control) and `2026-11-04T09:50:42.693000` (the fire). **No third HEAD row**, so no duplicate
  invocation reached the database up to 10:24:32Z. B has no ingestion lag worth naming (623 ms on the fired request).
- C: **10:24:55Z**, Data API window **09:24-10:24Z**: Total Requests **1** — `HEAD 200 /rest/v1/lines?select=id` at 09:50; Response
  Errors none.
- note: the hour closes with **one invocation, one database touch, one counted request**. Nothing fired before 09:50:42Z (slots 1-3) and
  nothing after it (slot 3 confirmation at 10:09Z over 09:40-10:10Z, and this slot over 09:55-10:24Z on channels B and C).

## Verdict (one closing entry)

# **FIRED — 2026-11-04T09:50:42Z (04:50:42 ET), inside the documented hour.**

The Vercel cron `17 9 * * *` invoked `/api/keepalive` once, 33m42s after its nominal minute, and the function did its job: it reached
Supabase and answered the count. Both independent channels name the same second, and the third channel counts exactly one request in
the hour.

| channel | what it shows | timestamp |
|---|---|---|
| **A — Vercel Request Logs** | `GET /api/keepalive` **200**, **User Agent `vercel-cron/1.0`**, **no search params**, host `rhl-warehouse-board-2ttt1y84e-…` (the deployment the job is registered against), exec 916 ms, response 1.2 s, External API `HEAD …/rest/v1/lines`, `dpl_busDKSQZ6wZKggot9EvwyWBemjpR` production. Request ID `8l7rn-1793785842074-582ac4002ec5`. | **09:50:42.07Z** |
| **B — Supabase API Gateway (`edge_logs`)** | `HEAD \| 200 \| https://yhpjpjeedpuvcgnvfpka.supabase.co/rest/v1/lines?select=id \| node` — 623 ms after A, so the database was actually touched and answered | **09:50:42.693Z** |
| **C — Supabase Data API report** | Total Requests **1** in 09:01-10:01Z: `HEAD 200 /rest/v1/lines?select=id` at **09:50**, 559 ms avg, **0 response errors** | 09:50Z |

**What this settles.** The question the whole night circled was whether the scheduler's firing was observable at all. It is, on two
independent channels, to the second, with the caller named. The keep-alive does what it exists for: the board's newest write was 29 Oct
and a daily API request is Supabase's own documented remedy for the Free-plan inactivity pause — and this one reached the database.

**What it does not settle.** One observed invocation is not a guarantee. Vercel documents cron delivery on this plan as **best effort,
with no retry on failure**, so a future missed day is documented behaviour, not a broken job. Nothing here proves tomorrow's run; it
proves the mechanism, the registration and the evidence chain.

**One limit found while reading, which matters more than the verdict itself:** Vercel **Request Logs ingest with ~10 minutes of lag on
this project** (request 09:50:42; absent at 10:01:04; present at 10:01:17). An "empty" request-log read is therefore only evidence
about the period that ended ~10 minutes before the read. Channel B has no such lag (623 ms). The four-read plan survives this — but a
single late A read must be taken **after** the lag, and the rule "empty pane = no fire" is wrong without it.

Superseded vocabulary, kept so the file still explains itself:
- **FIRED at** ______ — a scheduler row in channel A *and* a matching HEAD row in channel B inside the scheduled hour
  **[09:00Z,10:00Z)**. (Vercel docs: on Hobby the job may run at any point inside the scheduled hour.) → **this is the entry that fired.**
- **FIRED EARLY at** ______ — the same, before 09:17Z. Not this run: 09:50:42 is inside the hour and after the nominal minute.
- **NO FIRE OBSERVED** — would have required a channel A read taken after the ingestion lag, plus a fresh B `Results (0)`. Neither
  applies; the fire was found.
- **INCONCLUSIVE** — a read whose query/range/view did not execute, or that cannot see its control. Not used: every read in this file
  states which trap it could have hit and how it avoided it.

- **FIRED at** ______ — a scheduler row in channel A *and* a matching HEAD row in channel B inside the scheduled hour
  **[09:00Z,10:00Z)**; paste both. (Vercel docs: on Hobby the job may run at any point inside the scheduled hour.)
- **FIRED EARLY at** ______ — same, before 09:17Z. The "Hobby fires late" assumption is unmeasured for this project's first run, so
  this is a result and the assumption was wrong, not an error.
- **NO FIRE OBSERVED** — channel A covering the window **with the 08:57Z control visible** and no scheduler row, plus either a fresh
  channel B `Results (0)` or a B range whose rows have aged out (say which).
- **INCONCLUSIVE** — a read whose query/range/view did not execute, or that cannot see its control. Name the trap.
- A channel-A read that identifies the caller can decide alone. **A channel-B-only read cannot** (it has no caller column) — if only B
  is readable, write INCONCLUSIVE.
