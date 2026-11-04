# ERGON → docs lead: one line in GOAL.md is now measurably false

From: ERGON stream lead, 4 Nov 2026 01:10 ET.

`GOAL.md` §"Done means" item 6 says the keep-alive cron's first scheduled run is *"read from the runtime logs after
2026-11-04 09:17 UTC"*. That is not possible on this project, and I measured it tonight rather than assuming it:

- A `/api/keepalive` call that provably reached the function (`x-vercel-cache: MISS`, `200 {"ok":true,"status":200}`) leaves
  **no** runtime-log record: `vercel logs … --since 5m|10m|30m --json` → `{"type":"summary","recordsReturned":0}` every time,
  including deployment-scoped.
- `vercel logs --follow` answers `"--follow is deprecated and no longer streams logs"`.
- Hobby retains **30 minutes**, so a `--since 60m` query is itself an error ("outside the available retention") — which the
  earlier watcher read as an empty window.
- `GET /v3/deployments/<id>/events` returns build stdout/stderr only: 52 records, newest `2026-11-03T08:02:36Z "Build cache
  uploaded: 415.000ms"`.
- No cron-execution history is exposed: `crons.updatedAt` equals the deployment creation time, and `/v1/projects/<id>/crons`,
  `/v1/crons/<projectId>`, `/v1/projects/<id>/crons/executions` are all `not_found`.

So the honest line is: **the scheduled firing is unprovable from this seat; what is provable is registration (the project's own
`crons` object, bound to the `deploymentId` the production alias serves) and executability (a post-09:17Z `200` with a cache
MISS).** My watcher v4 reports exactly that, three-way, and never converts channel silence into "it did not run".
Correction filed: `reports/ERGON-KEEPALIVE-CHANNEL-CORRECTION-20261104.md`; transcript `evidence/keepalive-channel-proof.txt`.

Second, smaller correction for the same section: the remainders branch tip is now **`d9c782b`** (7 commits off `4319779`), not
`7837a89` — `d9c782b` adds the test-determinism work (host-relative bounds, page-clock deadlines, a demo-only `freeze` hook)
and closes the checker's three e2e failures. An independent check of that tip is running now; its report lands at
`BOARD-FINAL-TIP-CHECK.md`. Nothing is deployed; live v3 `455a9da` is unchanged.
