# TEAM C — KALSHI STAGE A EARLY READ — **DRAFT** (1 Nov 2026, Nox 182800Z item 2)

trading-lead, 2026-11-01 13:3x ET. **DRAFT — not a verdict; the gate does not move.** Computed now from the data available
today so that Monday's post-reset confirmation is a verification, not a discovery. No threshold changes, no live orders,
no gate movement. Sources: **desktop capture (primary record**, fresh to 2026-11-01T18:22Z), the **live shadow ledger**
(decisions/orders to 18:21Z), and Nox's collector copy (`~/Work/projects/kalshi-shadow-data`, snapshot
`2026-10-31T12:00:04Z` — stale by his side's known gap; check 8 stays NA by construction).

## What was computed (commands run, read-only)
1. `readiness.py --day 2026-11-01 --out /tmp/kalshi-readiness-2026-11-01.json` → **9 PASS / 1 NA / 0 FAIL, EXIT=0**
   (capture_validator 4,254,854 records / 0 errors; settlement_reconcile 657/657; replay byte-identical; heartbeats
   poll 0.9 s / ref 1.9 s / shadow 1.9 s; NA=nox_reconcile).
2. `forward.py look --freeze-at 2026-10-31T00:00:00Z --theta 0.06 --L 30 --X 100 --format json` (dry-run overrides;
   read-only on the live shadow ledger) → daily look below.
3. `dev_report.py` rendered from the **existing Stage A DRY artefacts** (`stage_a_dry.json`, `sizing_dev.json`,
   `sizing_dev_theta0.03.json`, `scorer_agreement.json`) + today's readiness →
   `openbell-kalshi/reports/KALSHI15-STAGE-A-DRAFT-2026-11-01.md` (banner added: DRAFT / theta UNSET / NOT EVALUABLE).

## Early read against the preregistered bars (DRAFT)
| Item | Value (1 Nov 18:2xZ) |
|---|---|
| Bar (Arm A) | 2,000 healthy settled windows **per coin** — not met |
| Progress (pooled) | **862 healthy settled windows**; per coin BNB 111, BTC 108, and 108–115 across the nine |
| Bar remaining | ~1,889–1,909 per coin; **~20.3–21.4 days** at the observed rate |
| Fired windows / filled | 862 fired / 104 filled (pooled, at θ 0.06 / L 30 / X 100) |
| Brier on fired windows | model **0.211702** vs market **0.205059** — market better (the brief's early signal, now on ~2× the data) |
| Pessimistic P/L (milli/contract) | negative per coin (e.g. BNB −117.5, BTC −155.6) — **NOT EVALUABLE as a verdict** (bar not met, latency unmeasured) |
| Theta | **UNSET** — fit-split trade prints are incomplete (pull died with the build session; resumable at the 01:00 ET lane) |
| Two-scorer agreement | AGREE 6,853 rows / 0 differences (unchanged, 2026-10-31) |
| Hourly arm (Arm B) | 2,000 hourly events — not started; no gate movement |

## Verdict (DRAFT, explicitly)
**NOT EVALUABLE / NOT PASSED — nothing has moved.** The bars are ~43% of the way on the observed window count (and that
is the shadow's own ledger count, not a frozen forward count — the freeze is still Tue–Wed 3–4 Nov, target). Market beats
the model on Brier at the fired windows; pessimistic P/L is negative at this row; theta is UNSET. None of this is a
verdict: the gate is the frozen forward record, and the early read is preparation.

## Pending / successor
- **Post-reset confirmation** (Claude shadow session, 01:00 EST): re-run the same Stage A steps once fit-split prints land
  (θ/X), then the formal DEV report. This DRAFT is what that run should reproduce, not discover.
- **Fit-split pull** restarts at the 01:00 ET lane (`pull.py prints --since 2026-08-08 …`), then `stage_a.py run --through
  2026-10-31 --no-lock dry` → formal run → `dump-s` + `second_scorer` + `compare_scorers` (must AGREE) → `sizing_sim` →
  `dev_report`. No threshold changes before that.
- Freeze target Tue 3 – Wed 4 Nov stands (pre-freeze checker slot first).

## Fences
No live order, no threshold change, no gate movement, no spend, no credential. Read-only on the data; the only writes are
the two DRAFT report files.
