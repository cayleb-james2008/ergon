# ERGON → delegator: final state for the 08:00 record (verification half)

From the ERGON stream lead, 4 Nov 2026, 02:00 ET. Companion to the other ERGON session's filing — this one is the *check* record:
three independent passes, what each caught, and what is still not proven.

## The board branch is verified three times, and each pass changed something
- **Pass 1 `d9c782b` → FAIL.** Item 3 was a product defect: at 3 s per answer the label cycled Error/Live every 4 s, 36 % of a run
  calling a working server down, ending on Error. Item 7: the host-relative bound (`4 × hostRef`, and `4 × 41 > 100`) had retired
  the "cost does not grow with jobs" rule; `round4` asserted the best of three. Unbriefed finding: "Reconnecting" at 1577 ms on a
  first load that never connected.
- **Pass 2 `3780e04` → PASS**, with a required negative control: a deliberately injected ~204 ms per-job regression made the
  long-task test fail (210–300 ms against a 150 ms bound) where the clean build showed 75 ms against 178 — so green means
  something. It also found the last false claim: with the data endpoints hung and the probe answering, the label said **Live over a
  frozen board for 26 s**.
- **Pass 3 `9cc7d95` → PASS** on the fix for that: blank → **Error at 15 088 ms**, never Live, 0 rows, one retry beyond the first
  read's pair; a 14 s answer still resolves under the 15 s deadline; no timer left behind.

## Figures on the final tip `26e6c9e` (12 commits off `4319779`; live is `455a9da`)
- unit **121/121**; e2e **108 tests / 103 pass / 0 fail / 5 skipped, exit 0** (`evidence/board-e2e-tip26e6c9e.log`);
  connection **10/10**; agent suite **22 pass / 0 fail / 12 skipped**.
- **Deploy precondition from zero, on this exact commit**: `git archive 26e6c9e` → `npm ci` → `npm run build` (652 KB dist) →
  unit 121/121; leak scan on that build: **0** demo keys, **0** `service_role`/`sb_secret`, **0** baked Supabase host,
  **0** `__rhlDemo`.
- **Not deployed.** One owner yes; `vercel deploy --prod --build-env RHL_BUILD_ID=26e6c9e`; rollback is `vercel rollback`.

## The one gap, named rather than smoothed over
Five e2e tests (`supabase-live.test.mjs`) and twelve agent tests are skipped unless a local Supabase stack runs. I tried twice; the
CLI's postgres container dies at startup with `sh: can't fork: Resource temporarily unavailable` (same image runs a trivial
container, `Memory=0`/`PidsLimit=0`, and podman ran postgres cleanly an hour earlier for the quote proof). Transcript and diagnosis:
`evidence/board-live-stack-20261104.log`. **So: demo and mocked paths are verified on this branch; real-backend paths are not,
tonight.** The live v3 has been serving the same RPCs since 08:02 ET yesterday — that is not a test and is not counted as one.

## Other items, one line each (all with evidence on disk or the board)
- **Quote form** (`rhl/quote-requests` `0c156d4`): 48/48 contract checks on a real PostgREST; browser path end to end; blank guards
  fixed and proved; and the migration **applied three times** on a fresh Supabase Postgres 17 pod — exit 0, zero errors each time,
  and afterwards exactly 1 trigger, 1 policy, 7 checks, 3 indexes, so Nox can retry his hosted step safely. Two traps documented
  (bare Postgres lacks the roles the revoke names; `return=representation` is refused for anon by design).
- **SimBank v3**: verdict filed (`NOT PASSED`, ML-D2, lead-recomputed from the raw verdicts); the run-3 evidence — 1220 files, 11 MB
  — was **untracked** and is now committed (`0e6f7c94`) so the verdict is reproducible off this machine. Branch disposition is
  still Nox's.
- **Keep-alive**: the Vercel runtime-log channel is **blind** on this plan (a known 200 with a cache MISS leaves
  `recordsReturned: 0`; `--follow` is deprecated; Hobby retains 30 minutes; the events API is build-only). The armed watcher therefore
  records what is provable — registration (`17 9 * * *`, `/api/keepalive`, bound to the live deployment), a post-04:17 `200` with a
  cache MISS — and names where the *execution* can actually be seen: the Supabase API-gateway logs, which do record the keepalive's
  `HEAD /rest/v1/lines?select=id` (positive control from a known call at 06:36:59Z), with a discount rule so any HEAD row in the
  09:17–10:17Z window is the scheduler. That query is for Nox (or the owner's console); it cannot be run from here.
- **Host**: 15 orphan `vite preview` servers and 24 stale headless browsers were stopped; they were the real cause of the timing
  failures that started this.

## Blocked on the owner, smallest act each
1. Deploy the board remainders — one yes (packet ready, build id `26e6c9e`).
2. Rule whether the SimBank ML-D2 re-read override stands (default: it stands; overruling needs the staged hash-bound input).
3. Second free Supabase project for the showcase.
4. His working key on the real board (minted through the app flow, his hands).
5. Quote form's production link changes only on the site owner's yes.
6. The 1 s health probe's egress (~4.5 GB/month at the 10-viewer model) is a design choice worth his eye, per the checker.
