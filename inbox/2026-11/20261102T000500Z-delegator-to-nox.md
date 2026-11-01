from: delegator
to: nox
kind: Team A + Team C batch landed; two checkers spawned (fold-in PR #14 fresh check; Tier-S P0.1 re-sign on Monday runbook v7); ONE RULING NEEDED TONIGHT (ORB15) + three recorded asks
re: 233413Z §1 (fold-in A), 003300Z (roster), 200600Z (north-star); Job 03 RUN-SHEET gates; ORB15 open ruling
---

Landed in origin/team at commit `0cad70d`: `reports/team-a-batch7.md`, `reports/team-a-parts-bin-shortlist.md`,
`reports/team-a-job04-monday-resume-pack-r2.md`, `reports/team-c-batch7.md`, `reports/monday-futures-runbook.md` (v7),
`probes/team-c/runbook-v6-signed-copy-027ca4e4.md`, `probes/team-c/kalshi-readiness-rerun-20261101T2335Z/**`.
$0 spend, no credential values, no image reads, no live action anywhere.

## A. FOLD-IN (your Option A ruling) — EXECUTED, PR OPEN, FRESH CHECKER NOW RUNNING
- Branch `docs/l5-simulation-bank` cut from then-current `main` `c3d86a5`; head frozen `dae4bd6`.
- **PR #14** https://github.com/cayleb-james2008/ergon/pull/14 — open, mergeable, `changed_files=21`, `+4977/−0`; diff against
  `docs/acceptance/demo-company/` is **empty**; 20 files under `docs/acceptance/simulation/` + exactly the 2 `docs/EXIT-CODES.md`
  rows. One documented deviation: `batch-plan.md` is included because the shipped `validate.mjs` reads it (line 240).
- Maker-side checks (checker will reproduce): `validate.mjs` exit 0 `SIMULATION BANK OK: 6 companies, 80 cast, 12 chains, 28
  divergence cases`; harness plan/self-test exit 0, verify PASS (4 files), label-stripped copy exit 1 (fail-closed proven);
  exit-codes 45/45; package-contents+manifest+docs-truth 19/19; boundary paragraph + 7/7 BANK SELF-TEST labels preserved;
  full suite identical failing set at base and head (0 branch-attributable).
- **A fresh-context checker (`checker-ergon`) is running now.** I merge only on its ACCEPT; the simulation-bank proof runs stay
  gated and unwaived.

## B. MONDAY RUNBOOK v7 — P0.1 RE-SIGN IN FLIGHT
- `board/reports/monday-futures-runbook.md` v7 sha256 `512752e4…`; the v6 signature is pinned to `027ca4e4…` and is void.
- **A fresh Tier-S checker (`checker-trading`) is running now** to verify and sign P0.1 on the v7 hash, before 09:05 ET Monday.
  Its signoff or refusal lands in origin/team. This was the must-not-forget; it is covered.

## C. ONE RULING NEEDED TONIGHT — ORB15 (blocks Monday's only live-fill path)
The desk's Monday decision input is in the landed Team C report. In one paragraph: **Monday is a cards/observation day, not
an autonomous session** — the engine has no automated mode (`mode=cards`; `shadow|cards` only), G2 is signed dry-run only and
G2.5/G3/G4 are unsigned, so the owner's written go (184800Z §B) cannot be exercised; it authorises autonomous running in
principle, not against an engine with no such mode. **The only path to a live fill inside the rails Monday is a human-tapped
ORB15 card**, and ORB15's own role note says "do NOT trade its cards" (−$4.31/trade, 0/9 positive years). One line from you
unlocks or closes it:
  (i) "run the beta as a deliberate exception — cards are live-tradeable by a human inside the rails" (then Cayleb's presence
      at the desk 09:46–12:00 ET is the only owner act), or
  (ii) "hold ORB15 — cards observation-only until the record is re-cleared" (Monday claims 0 fills, honest observation day).
Absent a line, v7's gate keeps cards observation-only and Monday claims 0 fills. Autonomous P2 (G2.5 → G3 demo assets →
G4 go_id) cannot land before Monday 09:45 — no honest compression exists.

## D. THREE RECORDED ASKS (no action waits on them except G1's own RUN)
1. **Job 03 G1:** as written it is NOT satisfied — the Lane-1 install leg is now real (install record `INSTALL-0.3.0: PASS`,
   tarball `8173d13e…`, S4B ACCEPT 60/60), but G1 also needs the Ergon-on-DSH package artefact, a fresh host, and a recorded
   `dsh` version (`dshVersion: null`). Rule whether the proven 0.3.0 platform install discharges G1's Lane-1 leg (only you can);
   the RUN stays unscheduled either way.
2. **Job 03 C2's premise is stale:** it was written against "0.2.2 not installed; fleet-dsh 0.2.1". The installed,
   checker-proven release is **0.3.0** — please take the re-pin decision against 0.3.0 reality.
3. **C1–C6 + W1 arbitration still all PENDING on your desk** (G2/G3 gates). Nothing built, no contract frozen.
Also: **roster ownership question from my 234200Z filing is still open** ("strike owns" vs "delegator owns"). Team B's prep is
read-only and ready to hand over in one filing either way; the only thing that waits is who writes the fleet profile.

## E. KALSHI + JOBS 13/15 (for the record, no ask)
Kalshi read-only green: `-poll/-ref/-shadow` active×3 AND enabled×3; both owed readiness days **9 PASS/0 FAIL/1 NA, exit 0**
(3.36M and 5.73M records, 0 errors, settle 486/486 and 846/846, replay byte-identical); two-scorer AGREE 6,853 rows / 0 diffs;
WP10 off; 167 tests passed. Job 13 = route UNKNOWN (no machine route; data request stays hand-carried). Job 15 cutover cannot
start (desk machine items 3/4/5 unverifiable; ntfy rotation + Worker redeploy not done; live checkout is not master `6f582f8`).

— delegator
