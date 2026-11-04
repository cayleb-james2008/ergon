# ERGON → delegator: the keep-alive result is in — the cron FIRED (and the rest of the stream's state)

From the ERGON stream lead, 4 Nov 2026, 05:10 ET. Replaces the "unprovable" wording in my 02:00 ET note.

## Headline: the keep-alive cron fired, proved three ways by two readers
Scheduled `17 9 * * *` = 09:17Z. **It fired at 09:50:42Z** — 33 m 42 s late, inside Hobby's documented hour.

| channel | what it shows |
|---|---|
| Vercel Request Logs | `GET /api/keepalive 200`, **04:50:42.07 ET**, **User Agent `vercel-cron/1.0`**, no search params (ours carry `before-window=`), Request ID `8l7rn-1793785842074-582ac4002ec5`, deployment `dpl_busDKSQZ6wZKggot9EvwyWBemjpR`, production, 1.2 s |
| Supabase edge_logs | `2026-11-04T09:50:42.693Z  HEAD | 200 | …/rest/v1/lines?select=id | node` — 0.62 s after the request, so the database was actually touched |
| Supabase Data API | exactly **1** request in 09:01–10:01Z: `HEAD 200 /rest/v1/lines?select=id` at 09:50 |

The question that was open all night is closed. Registration was already proved; what was unproved was that the timer fires — it does.

**Two measured traps that would each have produced a false "did not run", and are now written down:** Vercel Request Logs
**ingest with ~10 minutes of lag** on this project (the 09:50:42 row was missing at 10:01:04Z and present 13 s later), and the
Supabase Logs Explorer keeps the previous render until Run finishes, so "No results found" is not a zero. An early read must never
be filed as "no fire". Verdict and every read: `KEEPALIVE-READ-20261104.md`.

## The rest of the stream, one line each
- **Board branch** `fix/board-remainders-20261103`, tip **`b559e68`**, tree clean, **not deployed** (live is `455a9da`).
  Three independent checks: `d9c782b` **FAIL** (a real product defect at 3 s latency + bounds that no longer described the app),
  `3780e04` **PASS** with a firing negative control, `9cc7d95` **PASS** on the 26 s false-Live fix. Unit **121/121**, e2e
  **108/103 pass/0 fail/5 skipped**, connection 10/10, agent 22/0/12. Certified from zero at `b559e68`
  (`git archive` → `npm ci` → build 652 KB → unit; leak scan clean).
- **Data layer** verified independently of the missing stack: the repo's SQL/REST contract suite **30/32** against a plain
  `supabase/postgres` + `postgrest` + locally minted anon JWT; the two failures are the harness (no realtime service; a reserved
  role's password), not the schema. **Not verified: realtime delivery and browser-to-live-backend.**
- **Quote form** `0c156d4`: 48/48 contract checks, browser path end to end, and the migration applied **three times** on a fresh pod
  (exit 0 each time; 1 trigger, 1 policy, 7 checks, 3 indexes after) so Nox can retry his hosted step safely.
- **SimBank v3**: verdict `NOT PASSED` (ML-D2) filed and lead-recomputed; the run-3 evidence (1220 files, 11 MB) was untracked and is
  now committed (`0e6f7c94`). Branch disposition is Nox's.
- **Host**: 15 orphan preview servers and 24 stale headless browsers stopped; these were the real cause of the timing failures.

## Blocked on the owner, smallest act each
1. Deploy the board remainders — one yes (`vercel deploy --prod --build-env RHL_BUILD_ID=b559e68`; rollback `vercel rollback`).
2. Rule whether the SimBank ML-D2 re-read override stands.
3. Second free Supabase project for the showcase.
4. His working key on the real board.
5. Quote form's production link — site owner's yes.
6. The 1 s health probe's egress (~4.5 GB/month at the 10-viewer model) and the measured price of the connection fix (a
   slow-then-dead link now shows Live ~11 s instead of ~2.5 s) are design calls worth his eye, not checker vetoes.
