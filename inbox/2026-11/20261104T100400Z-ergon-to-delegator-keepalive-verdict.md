# ERGON → delegator: morning section is final, and the keep-alive question is answered

**2026-11-04T10:04Z (05:04 ET)**

`~/fleet-scratch/overnight/ERGON-MORNING-SECTION.md` is **final as of 10:03Z**. This supersedes my 03:10Z pointer
(`20261104T081000Z-ergon-to-delegator-morning-section-pointer.md`); the section's structure and every other paragraph are unchanged, so
only the two keep-alive passages moved.

## The one thing that changed since the pointer: the cron FIRED, and it is on the record three ways

`17 9 * * *` → `GET /api/keepalive` at **2026-11-04T09:50:42Z**:

- **Vercel Request Logs**: `200`, **User Agent `vercel-cron/1.0`**, **no search params**, host `rhl-warehouse-board-2ttt1y84e-…` (the
  host the job is registered against, not the alias our own curl calls use), execution 916 ms, and an External API `HEAD
  …/rest/v1/lines` listed in the same row — Request ID `8l7rn-1793785842074-582ac4002ec5`.
- **Supabase gateway log**: `HEAD 200 …/rest/v1/lines?select=id` at **09:50:42.693Z** — 653 ms later, so the database answered.
- **Supabase Data API report**: exactly **1** request in 09:01–10:01Z, at 09:50, zero errors.

09:50:42 is inside the documented hour [09:00:00Z, 09:59:59Z): a normal late Hobby fire, not a miss. **One observed invocation is not a
guarantee** — Vercel documents cron delivery as best effort with no retry — and the section says so in the same breath as the result.

## The part I would not drop from the consolidated record

The result was nearly the opposite. **Vercel Request Logs ingest with ~10 minutes of lag on this project**: the 09:50:42 request was
absent from the "last 30 minutes" view at **10:01:04Z** and present at **10:01:17Z**. A single empty read at the end of the hour — which
was the plan's slot 3 — would have produced a confident, wrong **"NO FIRE OBSERVED"**. It was caught by re-reading 13 seconds later,
not by being clever. Channel B (Supabase) showed the row 653 ms after the call and has no such lag.

That is the fourth trap this reading procedure hit tonight, and the only one that would have inverted the answer. The morning section
names all four. Report: `reports/ERGON-KEEPALIVE-VERDICT-20261104.md`; raw reads
`~/fleet-scratch/overnight/evidence/keepalive-reads-20261104.txt`; read log and single verdict
`~/fleet-scratch/overnight/KEEPALIVE-READ-20261104.md`.

## State of the rest of my scope (unchanged since 03:10Z, re-verified at 09:2xZ)

- **Warehouse board remainders** — `fix/board-remainders-20261103` tip **`b559e68`**, tree clean, **not deployed**. The deploy invariant
  prints nothing (`git diff 9cc7d95..b559e68` touches only `docs/` and `test/`), so the 7/7 certification still describes the artifact.
  One owner yes ships it.
- **Quote form** — `rhl/quote-requests` tip **`cc42461`**, tree clean; agent-side complete, the six hosted steps are Nox's.
- **SimBank v3** — **NOT PASSED** (ML-D2 charged-ahead in run b; 93.3 / 91.7 / 93.3 % against an 85 % floor; control 81.7 %). Branch
  `proof/simbank-v3-20261103` clean at **`0e6f7c94`**; exactly 3 full runs + 1 control, re-checked tonight, nothing duplicated. The
  owner-override patch is staged and proven on a copy and in situ, nothing applied. Ruling is Nox's/owner's.
- **Fences held**: nothing deployed, nothing spent, no hosted row content read, no hand-made keep-alive call inside the window, no
  repository changed for the keep-alive work.

## Blocks that are decisions, not acts

1. The board deploy — one yes (tip `b559e68`; certify with `stage/postdeploy-check.mjs --expect-version b559e68`).
2. Whether the SimBank ML-D2 re-read override stands.
3. A second free Supabase project for the showcase.
4. The quote form's production link changes only on the site owner's yes.
5. The connection fix's measured price: a slow-once-then-dead link reads `Live` for ~11 s (was 2.5 s).
