# R1 — Company setup and canonical state dump (pass 2)

- Step: R1 — Company setup and canonical state dump
- Pass: 2
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: operator opens the install link and runs setup as a first-time user; on this platform-install rehearsal the fixture `setup.mjs` performs the same stand-up deterministically (named stand-in for the installer UI, which is PACKAGE-PENDING)
- Expected: setup exits 0 with 6 cast profiles, 9 enrolled/pending devices, the six-week seeded backlog, 6 ReviewDesk ledgers, the scripted board and the stack artefacts; `dump-state.mjs` reproduces the pinned canonical digest.
- Actual: 16/16 hard checks held; 0 named pending; 1 recorded finding(s)
- Word: **clean**
- Evidence files: `R1/setup.log`, `R1/setup-summary.json`, `R1/company-cast.json`, `R1/state-dump.log`, `R1/state-dump-summary.json`
- Timestamp (UTC): 2026-11-01T22:07:36Z .. 2026-11-01T22:07:37Z
- Blocker / finding: none

## Checks
- PASS fresh rehearsal root is empty before setup — root=/tmp/ergon-rehearsal-20261101T220736Z-ZUb78e entries=0
- PASS setup.mjs exits 0 — exit=0
- PASS setup summary parses as JSON — keys=tickets,reviewDesk,library,profiles,stackArtefacts,devices,root,seed
- PASS setup reports one profile per cast member (6) — profiles=6 ids=dana-whitcomb,priya-lindqvist,marcus-oyelaran,teo-brandvold,rosa-quintero-hale,sam-akintola
- PASS setup reports the enrolled device set (6 cast devices + 4 background, incl. the pending sales laptop) — devices=10
- PASS setup reports the open-source stack artefacts — stackArtefacts=9
- PASS setup reports the seeded backlog + ReviewDesk + library summaries — tickets={"raised":104,"outcomes":{"resolved":79,"waiting":13,"escalated":3,"abandoned":1,"queued":8},"replies":133,"firstAt":"20 reviewDesk={"dana-whitcomb":{"requests":5,"later":2,"approve":1,"pending":1,"reject":1},"pr library={"tools":[{"name":"crew-sheet-builder","latestPublished":"1.0.0","versions":["1.
- PASS setup root is this pass's fresh root — summary.root=/tmp/ergon-rehearsal-20261101T220736Z-ZUb78e
- PASS company record matches the demo company — Lanternfield Moving & Storage / America/New_York
- PASS cast record holds six distinct named employees, each with a personalised agent profile — dana-whitcomb,priya-lindqvist,marcus-oyelaran,teo-brandvold,rosa-quintero-hale,sam-akintola
- PASS dump-state.mjs exits 0 — exit=0
- PASS canonical dump digest matches the pinned fresh-stand-up digest — sha256=558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1 pinned=558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1
- PASS canonical dump size matches the pinned size — bytes=315258 pinned=315258
- PASS canonical dump normalisation counters present — {"randomIdsAliased":1050,"wallClockTimestamps":73,"secretValuesHashed":3,"rootPathsReplaced":4}
- PASS board is recorded as scripted fixture state (board app PACKAGE-PENDING) — scriptedState=true day=pre-week status=PACKAGE-PENDING
- PASS onboarding sessions exist for the whole cast (fresh, stage installed) — session files=6

## Findings recorded (not a failure of this step)
- installer UI — R1 on a real host is the operator opening the install link and running the product installer. That package is not in the pinned tree (PACKAGE-PENDING); this rehearsal drives the fixture `setup.mjs`, which stands the company up through the product's own modules. Named stand-in: the first-run UI, not the company state.

## Notes
- Seeded past as stored: 104 tickets, 133 replies, 428 audit rows, 9 fleet devices, 6 ReviewDesk ledgers, 3 stack tool trees.
- The canonical dump is written under this step dir as `state-dump.json` (315 KB) with its digest in `state-dump-summary.json`; both passes must reproduce it byte-identically, which the R7 pass diff re-checks.

## Recorded metadata
```json
{
  "seedDumpSha256": "558df9edb29c8c5b8e449b53e6a68ae6e02f1259d257fabe3017d943a1b593c1",
  "seedDumpBytes": 315258,
  "ticketCount": 104,
  "deviceCount": 9,
  "reviewDeskLedgers": 6
}
```
