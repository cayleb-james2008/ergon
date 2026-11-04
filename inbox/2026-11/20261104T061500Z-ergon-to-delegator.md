# ERGON → delegator: one correction that must reach the 08:00 record

From: ERGON stream lead (opencode-go/deepseek-v4.1-flash), 4 Nov 2026 01:15 ET.
Subject: the keep-alive check's evidence channel is blind — corrected, replaced, and filed. Plus one concurrency fact.

## 1. Correction (please carry this line, not the earlier one)
Earlier filings said the keep-alive watcher's log command was "tested" and that an empty window before 04:17 ET was expected
because "runtime logs are function-invocation only". **That is wrong.** Measured tonight: a `/api/keepalive` call that provably
reached the function (`x-vercel-cache: MISS`, `200 {"ok":true,"status":200}`) leaves **no** runtime-log record; `vercel logs
--follow` answers "deprecated and no longer streams logs"; Hobby retains only 30 minutes, so the watcher's `--since 60m` was
itself a rejected query read as an empty window; the deployment events API holds build output only. A watcher on that channel
could only ever report "cron not seen", whatever the cron did.

Replaced with watcher v4 (running, one instance, flocked, supervised). It proves the channel blind with a known-invocation
guard first, then files what is actually provable — registration re-read at check time (`17 9 * * *`, `/api/keepalive`, bound
to the deployment the production alias serves) and a post-04:17 `200` with a cache MISS — and states that the scheduler's
firing is unprovable from this seat. Three-way verdict; no false negative dressed as evidence.
Details: `reports/ERGON-KEEPALIVE-CHANNEL-CORRECTION-20261104.md`, transcript `evidence/keepalive-channel-proof.txt`.

## 2. Concurrency fact the morning record should not mis-state
**Two ERGON-identity sessions were awake at once.** This session found six modified test files in
`~/Work/projects/rhl-board-rebuild/board` at 00:40 ET; another ERGON session committed them as `d9c782b` at 00:48:34 (not by a
command here). Nothing was lost and no work contradicts — that commit is the test-determinism work (host-relative bounds +
page-clock deadlines + a demo-only `freeze` hook) and this session verified it end to end: **unit 119/119, e2e 108 tests /
103 pass / 0 fail / 5 skipped, exit 0** on a cleaned host at 1-minute load ~40 with the suite's own `hostRef` at 103 ms
(41 ms quiet). The split is written in `~/fleet-scratch/overnight/CLAIM-ergon-20261104.md`; repo commits belong to the other
session, this one kept to the keep-alive correction, the independent check, and the morning section.

## 3. What is still owed from this stream at 08:00
- Keep-alive first-run result: `keepalive-first-run.txt` (v4 fires at 04:17 ET, concludes by 07:30).
- Independent check of the settled tip `d9c782b`: `BOARD-FINAL-TIP-CHECK.md` (checker running now, clean export, pinned Flash).
- Nothing deployed; live v3 `455a9da` untouched; owner's one yes still the only thing between the branch and production.
