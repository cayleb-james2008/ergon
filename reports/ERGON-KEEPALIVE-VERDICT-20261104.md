# Keep-alive: the cron FIRED, at 09:50:42Z — three-channel evidence

**ERGON stream · 4 Nov 2026, 05:0x ET (10:0xZ) · lead model `opencode-go/deepseek-v4.1-flash`**

The question the night circled — *does the daily keep-alive actually run?* — is answered. `17 9 * * *`, Vercel
(`caylebjamesportfolio/rhl-warehouse-board`), invoked `/api/keepalive` once, and the invocation is on the record three independent ways.
The function did its job: it reached Supabase and answered the count.

## The verdict

| channel | what it shows | timestamp |
|---|---|---|
| **A — Vercel Request Logs** (names the caller) | `GET /api/keepalive` **200**, **User Agent `vercel-cron/1.0`**, **no search params**, host `rhl-warehouse-board-2ttt1y84e-caylebjamesportfolio.vercel.app` (the host `vercel crons list` binds the job to), region iad1, **execution 916 ms**, response 1.2 s, **External API `HEAD …/rest/v1/lines`**, deployment `dpl_busDKSQZ6wZKggot9EvwyWBemjpR` production. Request ID `8l7rn-1793785842074-582ac4002ec5`. | **09:50:42.07Z** |
| **B — Supabase API Gateway (`edge_logs`)** | `HEAD \| 200 \| https://yhpjpjeedpuvcgnvfpka.supabase.co/rest/v1/lines?select=id \| node` — **623 ms** after A, so the database was reached and answered | **09:50:42.693Z** |
| **C — Supabase Data API report** | Total requests in 09:01–10:01Z: **1** — `HEAD 200 /rest/v1/lines?select=id` at **09:50**, 559 ms avg, **0 response errors** | 09:50Z |

**Inside the documented hour.** Vercel's *Cron jobs accuracy* page says a job "may be invoked at any point within the specified hour"
(`0 8 * * *` → "anytime between 08:00:00 and 08:59:59"), so the window for `17 9 * * *` is **[09:00:00Z, 09:59:59Z)**. 09:50:42 is
33m42s after the nominal minute: a normal late fire on Hobby, not a miss and not a defect. The earlier "Hobby fires up to an hour
late" model (window [09:17,10:17)) was corrected at 03:07 ET from the vendor's own text; under that wrong model this same fire would
have been mislabelled.

**What it does not settle.** One observed invocation is not a guarantee. Vercel documents cron delivery as *best effort* with **no retry
on a failed invocation**, so a future missed day is documented behaviour, not a broken job. Nothing here proves tomorrow. It proves the
registration, the mechanism, and that the evidence chain can answer the question.

## The trap that nearly inverted the answer (worth more than the verdict)

**Vercel Request Logs ingest with ~10 minutes of lag on this project.** Measured, same view, same filter, 13 seconds apart:

- refresh at **10:01:04Z** (`refreshedAt=1793786464005`), axis 09:32–10:02Z → **"No Request Logs Match Your Filters"**;
- refresh at **10:01:17Z** (`refreshedAt=1793786477891`), same axis → **one row, 04:50:42.07**.

The request was at 09:50:42 — so the row arrived ~10.4 minutes later. A single "last 30 minutes" read taken at the end of the hour
(the plan's slot 3 at 10:05Z) would have been **empty**, and the honest-looking verdict would have been *NO FIRE OBSERVED* — false, and
false in the one document written to end the argument. Channel B has no such lag: it showed the row **623 ms** after the call.

Everything else in the reading procedure was already hardened against its own traps and each one was met for real tonight: the
mandatory `LIMIT` (refusal shown only after Run), the pane that keeps the previous run's words (so `Results (N)` or the query's own HTTP
response is the evidence, never the pane), the bookmark that **seeds the editor truncated to `select tim`** (and a `fill` on the hidden
Monaco textarea changes the DOM value but not the editor model), and a request log whose rendered row list can **lag one refresh**
(09:35Z: axis moved, stale row still listed).

## How the evidence was taken

- **The artefact is real and complete.** `keepalive-first-run.txt` was written at **09:17:01Z** by watcher v6.3 under a persistent
  systemd user unit: 108 lines, ends `--- end ---`, carries `DISCOUNT RULE (v6.3)` and `TRAP 3`. Section 1 re-read the registration at
  check time (`/api/keepalive`, `17 9 * * *`, `enabled: true`, `undeployed: []`); `vercel inspect` of that host returns
  `dpl_busDKSQZ6wZKggot9EvwyWBemjpR`, target production, Ready — so the job is bound to the deployment production serves.
- **The path was proved before the window, not after.** One call at **08:57:01Z** → `200 {"ok":true,"status":200}` in 1.27 s →
  its HEAD row at **08:57:00.758Z** in channel B. That row is the control every read is measured against, and the discount rule
  (attribute by the call, never by a count) is possible only because `keepalive-launches.log` records every launch and every call.
- **Reads at 09:12, 09:22, 09:35, 09:44, 09:47, 10:01 and 10:01:17Z**, with the union as the answer. The database channel was queried
  through the dashboard's **own analytics endpoint** with its own SQL and bounds once the Explorer's Run button stopped issuing
  queries; the response is timestamped and carries HTTP status, so a stale pane is impossible.
- Raw reads: `~/fleet-scratch/overnight/evidence/keepalive-reads-20261104.txt`. Read log and the single verdict:
  `~/fleet-scratch/overnight/KEEPALIVE-READ-20261104.md`. Both are also on disk with the artefact itself.

## What this changes for the owner

- **Nothing needs doing about the keep-alive.** It works, and it is now proved rather than assumed. The board's newest write was 29 Oct
  and the Free-plan rule is a week of inactivity: the mechanism that keeps it from pausing is live, and Supabase's own pausing guide
  names a daily API request as the recommended remedy (while warning that "it's possible that usage is not enough" — the residual risk
  is quoted, not hidden).
- **One thing to decide later, not tonight:** the check was instrumented for *one* run. Whether to leave a permanent, automatic record
  of each daily run (and how much log retention that needs, given Vercel's ~10-minute request-log lag and Supabase's ~1-hour `edge_logs`
  retention) is a small design call for the morning, not a fence to cross at 05:00.

## Fences

No deploy, no spend, no hosted row content read (counts and log statuses only), no cron call made inside the window by hand, no SimBank
run duplicated (3 runs + 1 control, unchanged), no repository changed for this work.
