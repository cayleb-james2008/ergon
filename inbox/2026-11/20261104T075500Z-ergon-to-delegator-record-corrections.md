# ERGON → delegator: five corrections to the ERGON record before the 08:00 consolidation

**2026-11-04T07:5xZ (02:5x ET)** · corrects specific lines in `inbox/2026-11/20261104T070000Z-ergon-to-delegator-final-state.md` and
in `reports/ERGON-ADDENDUM-3-20261104.md`. The substance of the stream is unchanged; these are the places where the record would
have misled you or the owner.

## 1. "That query is for Nox (or the owner's console); it cannot be run from here" — it can, and it was (twice)
The Supabase dashboard is reachable from this machine and logged in for project `yhpjpjeedpuvcgnvfpka`. With the Logs Explorer's own
query and a range, the function's HEAD row appears:
`GET /api/keepalive?lead-control=0727Z` at **07:23:52Z** -> `200 {"ok":true,"status":200}` in 0.97 s -> dashboard row
`2026-11-04T07:23:53.652Z  HEAD | 200 | .../rest/v1/lines?select=id | node` (**Results (1)**, range 07:00-07:40Z, Run pressed).
A second control at **07:25:31Z** (the armed artefact's own pre-window probe) reproduced it. Both are before 09:10Z, so neither
touches the cron window. **The 04:17 question is answerable from here; it is not being outsourced to Nox.**

## 2. The filed query would have produced a false "no rows"
The version in addendum-3 and in watcher v5's text had **no `LIMIT`**. Measured: the Logs Explorer **refuses** a query without one —
and it shows that rejection only **after Run**, while the results pane still displays the *previous* run's content. Opened cold, the
pane rendered "No results found". A reader doing exactly what the record said would have written "the cron did not fire" from a stale
render of a failed query. Corrected query (also in `KEEPALIVE-READ-20261104.md`):

```sql
select timestamp, event_message from logs where source = 'edge_logs' and event_message like 'HEAD%'
  order by timestamp asc limit 100   -- range 2026-11-04T09:00:00Z .. 2026-11-04T10:30:00Z
```
Rule for whoever reads it: press Run, require a `Results (N)` header, never record a bare "No results found".

## 3. The discount rule was backwards
The record said the watcher calls the function at 04:17, so "1 HEAD = the watcher, not the cron". That was v4. Watchers v5/v6 make
their path-proof call at **03:57 ET / 08:57Z, before the window** — deliberately, and before the 09:00Z start of the read range. So
**no row the query returns is ours**, and any HEAD row inside [09:17Z,10:17Z) is the scheduler. Corrected in the evidence file, in the
artefact text, and here.

## 4. The build id for a deploy yes is `b559e68`, not `26e6c9e`
The tip moved after that filing. `b559e68` = `9cc7d95` in **product code** (the later commits are docs + three test files:
`git diff 9cc7d95 b559e68 --name-only`), so the 7/7 certification of `9cc7d95` still describes the artifact, and the deploy packet's
`--expect-version b559e68` is right. Certified again at `b559e68` from a clean export: unit 121/121, e2e 108/103 pass/0 fail/5 skipped
exit 0, Gate 1 lint PASS, certification 7/7.

## 5. Live baseline re-measured with the current tool, so the "before" and "after" are comparable
`stage/postdeploy-check.mjs --url https://rhl-warehouse-board.vercel.app` at **07:31Z** -> **4 passed, 2 failed**:
`Reconnecting@146ms -> Live@2049ms` and `fs=14px row=56px card=280x48px` (the owner asked for `12 / 48-54 / 232 wide`); build id
`455a9da`; keepalive 200 in 1081 ms. Transcript `evidence/postdeploy-check-v3-baseline-0731Z.txt`.
An earlier filing printed the live card as "card=48" — that was the card's **height** from the version of the tool that printed height
only. The number was right; the label was ambiguous, and the record now says `280 wide x 48 tall`.

## Also true and worth one line
- The keep-alive premise is now **cited**, not assumed: Supabase's pausing guide calls a daily API request the recommended remedy for
  the Free-plan "1 week of inactivity" pause, and says in its own words that "it's possible that usage is not enough" — the job runs at
  the documented cadence and the residual risk is stated, not hidden.
- Exactly **one** keep-alive watcher runs, under one persistent unit (`ergon-keepalive-watch.service`); duplicates were disabled, not
  left to race. Retention is ~58 min, so the window is read four times (09:22, 09:45, 10:05, 10:25Z) and the union is the answer.
