# R7 — Close-out: state record, teardown, second from-scratch pass (pass 1)

- Step: R7 — Close-out: state record, teardown, second from-scratch pass
- Pass: 1
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: record the state (company page, employee list, tickets, URL, host logs), destroy the company root, and run the whole pass a second time from scratch
- Expected: the state record names every entity the pass created; the root is emptied; a second full pass from a fresh root reproduces the same per-step verdict table
- Actual: 12/12 hard checks held; 0 named pending; 1 recorded finding(s)
- Word: **clean**
- Evidence files: `R7-close/state-record.md`, `R7-close/secret-leak-check.txt`, `R7-close/host-teardown.txt`
- Timestamp (UTC): 2026-11-01T22:31:48Z .. 2026-11-01T22:31:48Z
- Blocker / finding: none

## Checks
- PASS prior step R0 produced its data — R0
- PASS prior step R1 produced its data — R1
- PASS prior step R2 produced its data — R2
- PASS prior step R3 produced its data — R3
- PASS prior step R4 produced its data — R4
- PASS prior step R5 produced its data — R5
- PASS prior step R6 produced its data — R6
- PASS final canonical state dump succeeds before teardown — exit=0 sha256=79b7a0e25e142a9e4c87f254612200c2dd818c45bdacea6b4a80bf0d10dc203d
- PASS state record written (company / cast / tickets / order / digests) — R7-close/state-record.md
- PASS no fixture secret value appears in the evidence pack — 25 values checked, 0 hit(s)
- PASS teardown empties the fresh company root — exit=0 entries after=0
- PASS the root was a fresh company root before teardown (state existed to destroy) — entries before=2

## Findings recorded (not a failure of this step)
- state advanced by the pass — seed digest 558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1 -> final digest 79b7a0e25e142a9e4c87f254612200c2dd818c45bdacea6b4a80bf0d10dc203d (the pass created real state: tickets, onboarded assistants, a acted-on board task via the fleet order)

## Recorded metadata
```json
{
  "teardownExit": 0,
  "seedDumpSha256": "558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1",
  "finalDumpSha256": "79b7a0e25e142a9e4c87f254612200c2dd818c45bdacea6b4a80bf0d10dc203d",
  "ticketIds": [],
  "orderId": null,
  "firstPass": true
}
```
