# TEAM C — BATCH 4 (C15–C19): L\*=30 s applied, freeze + forward tooling built, G0 research, cutover matrix reconciled

trading-lead, 2026-10-31 19:1x ET. Shadow-side only; nothing live, nothing sent, no wallet, no credential, money-desk lane
untouched. Nox's batch-2 rulings (`board/probes/nox-rulings-batch2-20261031T224500Z.md`) applied where ruled.

## C15 — L\* = 30 s: applied to the shadow docs, text sent for Nox's review
Applied (authorised by the Lead on Nox's ruling):
- `research/kalshi15/DECISIONS.md`: the 13:17 ET confirmation's item 2 is marked **SUPERSEDED 2026-10-31 by Amendment 2**;
  a new section **"Amendment 2 (Nox, 2026-10-31 ~18:45 ET) — latency row L\* = 30 s, applied by the build side"** records
  the ruling verbatim and the table row (measured availability 16.127 s p50 / 16.735 s p90 after the tick, 24.586 / 27.713 s
  after `a_asof`, ≈19–20 s end-to-end, 30 s = smallest row of record ≥ it and above the data-time p90; 60 s stays a row of
  record but is not the selection row; gate applies at the measured row; rows 2/5 s informational; reversible before freeze).
- `research/kalshi15/SPEC.md` Appendix T: new **Amendment 4** with the same content (so SPEC and DECISIONS agree).
- Review text for Nox, with the exact applied wording: `board/reports/team-c-kalshi-lstar-text.md`.
Evidence: `git diff --stat` -> `DECISIONS.md | 19 ++++++++++++++++++-`, `SPEC.md | 7 +++++++`, 2 files changed, 25 insertions(+), 1 deletion(-).
**Not changed, flagged for Nox's word (two one-line flips):** `params.provisional.json` still has `"l_star_s": 60` (a
frozen-candidate data file — Stage A must pass `--L 30` or the file is flipped on his authorisation);
`results/scorer_agreement.json` still carries `latency.L_star: 60` (a generated artefact — do not hand-edit; regenerates to 30
on the next `stage_a.py dump-s`). No live path, execution package, gateway or generated result was touched.

## C16 — FREEZE TOOLING BUILT (`research/kalshi15/freeze.py`, shadow-side only)
What it does: hashes the 12 pinned candidates (`live/kalshi/{model,decide,fills,asof,sizing,shadow,ledger}.py`,
`research/kalshi15/{score,stage_a}.py`, `params.provisional.json`, `SPEC.md`, `PREREG-DRAFT.md`), records the freeze instant
(UTC + ET), θ/X/L\*, the bars (2,000 per coin / 2,000 hourly events), rows of record and the chain-head snapshot
(`code_sha256`, `params_sha256`, decisions/orders seq+hash, arms), plus the readiness summary — into `frozen.json`.
`build` refuses on: an explicit `--at` missing (no wall clock), **any `<FILL AT FREEZE>` slot left in the prereg**, a missing
pinned file, a readiness FAIL, degenerate pins. `verify` recomputes every pin and checks the ledger still contains the frozen
chain heads (a changed model/threshold/spec/ledger is drift, exit 5). `--dry-run` never touches the canonical record.
Tests: `research/kalshi15/test_freeze.py` (8 tests: determinism/byte-identical build, ET conversion 14:00Z -> 09:00 EST,
unfilled-slot refusal, missing-pin refusal, explicit-`--at` refusal, readiness-FAIL refusal, clean verify -> tamper -> pin
drift, dry-run leaves canonical untouched).
Dry-runs tonight (quoted):
```
$ python3 -I research/kalshi15/freeze.py build --at 2026-11-04T14:00:00Z --dry-run
refused: PREREG-DRAFT.md still has 7 '<FILL AT FREEZE>' slot(s); a freeze with unfilled slots would make the bars unfalsifiable
$ python3 -I research/kalshi15/freeze.py status
no freeze record yet
# structural dry-run on the REAL pinned files (scratch root, symlinked pins, scratch prereg with placeholders):
DRY RUN: wrote frozen.json.dryrun; pin count 12; theta 0.06 X 100 L* 30; freeze at 2026-11-04T14:00:00Z
capture_health: {"checks":10,"fail":0,"na":1,"pass":9,...}   chain_head: code_sha256 546733c6…, decisions seq 4860, orders seq 35368
```
So the freeze path is proven; the freeze itself is still correctly refused until the prereg's 7 slots are filled and the
run is authorised.

## C17 — FORWARD-RECORD / DAILY-LOOK TOOL BUILT (`research/kalshi15/forward.py`, shadow-side only)
What it does: reads the frozen record (or explicit dry-run overrides), reads the hash-chained shadow ledger, and produces the
daily read-only look (PREREG D5) for one arm (θ, L\*): per coin — healthy settled windows, unhealthy excluded, fired windows,
filled windows, **pessimistic P/L per contract (milli-dollars = Σpnl/Σcenti, score.py's definition)**, D1 Brier (model vs
market on fired windows at first fire), bar remaining, days-to-bar at the observed rate — plus a pooled line, the unhealthy
window list, and **NOT EVALUABLE** until every coin reaches 2,000. Guards: refuses pre-freeze windows (D3), a broken chain,
and drifted pins (it calls `freeze.verify`); never writes outside `--out`; deterministic output.
Tests: `research/kalshi15/test_forward.py` (7 tests: per-coin math incl. P/L and Brier, pre-freeze exclusion, unhealthy
exclusion, other-arm ignored, broken-chain refusal, no-record-no-overrides refusal, drifted-pins refusal).
Combined test run: **`Ran 15 tests … OK`** (`test_freeze.py` + `test_forward.py`).
Dry-run on captured data (quoted, θ 0.06 / L 30, ledger = live shadow ledger, freeze-at 2026-10-31T00:00Z):
```
healthy settled windows: 297 · fired: 297 · filled: 32
Brier on fired windows: model 0.227376 vs market 0.214945 (lower is better)
BNB 40 windows, 9 fills, pess -185.555556 milli/contract, Brier m .207653 / mk .171317
BTC 39, 3, +146.666667, .22671 / .223264 … (9 coins)   → NOT EVALUABLE (bar 2,000/coin; ~20–29 days at the observed rate)
```
That matches the brief's early signal (market beats the model on Brier at every threshold) — reported as an observation of
the captured record, not a verdict.

## C18 — G0 research: does the FundedNext Flex eval allow automated entry?
**Answer: YES, with a hard constraint — and it changes the design's interface choice.**
Source of record: the programme's own verified prop-firm survey, `~/Work/projects/openbell/reports/PROP-FIRMS-2026-10-27.md`
+ `prop_firms_2026-10-27.json` (61 rows from 8 verification batches; the FundedNext Flex rows are marked
`"confidence": "official"` and cite `helpfutures.fundednext.com` price table + Flex rules, **read 2026-10-27**):
- FundedNext **Flex 50K** (the eval this desk runs): `"automation": "allowed"`, `"copy_across": "allowed"`,
  `"copy_within": "allowed"`, and `"api": "own bots/EAs on Tradovate, NinjaTrader or TradingView (Tradovate accounts); VPS
  allowed; no API"`.
- So: own bots/EAs are permitted **through the platform login** (Tradovate here); **there is no API on FundedNext** — the
  REST-API-first design in `team-c-autonomous-path-design.md` is not available on this account, and the **CDP ticket
  automation route (the design's fallback, G2.5) is the only autonomous path on this eval**. A VPS is allowed by this
  source (relevant to Job 12 item 5b), but the desk's own-device route remains the simpler, sanctioned one.
**Honest limits:** the live public pages could not be re-read tonight — every web search and direct fetch timed out in this
session (30 s, repeated), so this is the on-disk 2026-10-27 reading, not a fresh clause. G0 as written still requires the
clause quoted **with the date read** before any build; that re-read is a network-dependent step, not a Cayleb act.
**Prepared one-line question for Cayleb's D9 (only if he wants it confirmed rather than read):**
> "D9: FundedNext Flex allows own bots/EAs through the Tradovate platform login but has no API (verified 27 Oct) — may the
> desk's autonomous path use the existing signed-in Tradovate tab (desktop automation, no new credentials, no API), or do you
> want the clause re-read by support first?" (Recommendation: use the desktop route; no outside send without his OK.)
Design-doc amendment needed (not applied tonight, plan only): interface section 4 flips to "CDP ticket automation is the
primary route on FundedNext; REST API is not an option there"; G2.5 becomes a required gate, not a fallback.

## C19 — Nox's cutover checklist reconciled into the C9 matrix
Source: `board/probes/openbell-cutover-prechecks-2026-10-31.md` (Nox, 31 Oct; 103 lines, incl. §3b). Reconciliation:
| # | Nox's checklist item | My C9 item | Status after reconcile |
|---|---|---|---|
| 1 | Exit rule B: read DEPLOYED `live/exit_rule.json` byte-for-byte vs seed; startup log line; `MAX_HOLD_H` 0.5-or-unset (never 24); the "live stays on rule A until paper books are chosen" sentence describes the PROMOTION mechanism, not the shipped default — **rule B stands, do not re-open** | C9 items 10–13 | **Corrected:** the A→B promotion question must NOT change the starting rule; live rule at cutover = **B from the deployed DATA file**. My item 23 ("exit-rule A→B auto-promotion decision") is re-labelled: it is only "is promotion earnable in the new layout", never "which rule starts". |
| 2 | 5 s exit polling: live on RSI's box since 30 Oct 20:23–20:27 ET (final trader pid 2193662), rollback `rollback_exitpoll_20261030-2030.sh`; repo shows NO 5 s (POLL_SEC 20) — locate the exact mechanism (env/config/code diff), carry it, **prove from the exits log; mark UNPROVEN until an exit occurs** | C9 items 11–12 | **Refined:** add the box-side evidence (pid, rollback script) and the UNPROVEN-until-first-exit standard to the proof requirement. |
| 3 | JUP_API_KEY carried over agent-side, never printed; ntfy rotation; Cloudflare Worker redeploy; ledger continuity = BOTH ledgers (`live/ledger.jsonl` + `ledger.jsonl`) + demotion scores, not the repo seed | C9 items 4, 5, 20–22 | **Confirmed, one addition:** both ledgers by name, and the rule-B + 5 s are also the desktop-port handover requirement (29 Oct). |
| 4 | §3b desktop tree archive: repoint first — `memedesk-desk.service` BindPaths in `desk-fence.conf`, `desk-start.sh`, `memedesk-sync`, `memedesk-uptime` heartbeat, host tests + fence probe; verify each role on the unified repo; THEN move the tree to `~/Work/_archive/harness-removal-20261030/`. Do not move first (crash-loop + sync errors every 2 min) | C9 item 7 (partial) | **Completed:** exact repoint list + archive target path now in the matrix. |
| 5 | §4 known gaps accepted at merge: copy exits can lag ~40 s / be missed at 15+ source txs; `bt/features.py:218` report-only (hash-guarded); merged master baseline shows ~10 pre-existing failing suites; **cutover gating = "no new failures outside that baseline"**, not green-everything | not in C9 | **New:** added to the matrix; the gate wording is now "no new failures outside the recorded baseline". |
Matrix + runbook updated (runbook v4 points at this section; `team-c-c6-c10.md` C9 remains the item list, this table is the
authoritative reconcile).

## Also landed tonight (Lead's feed warning + Nox's collector note)
- **Runbook v4** (`board/reports/monday-futures-runbook.md`): attestation (c) can no longer be assumed — Team E's 22:57Z
  dry-run saw 58+ consecutive `no trader.tradovate.com tab` retries. (c) now requires four quoted proofs (tabkeeper
  `"tradovate"` not `missing`; Chromium on 9222; `feed` not retrying + `feed_age_s` < 30; **a fresh bar timestamped after
  09:30 ET and ≤120 s old**, via a paste-ready one-liner), else the day is BLOCKED with (c) named; a fresh session/
  credential/2FA demand = **STOP FOR CAYLEB**, never a sign-in.
- **Nox collector note** folded into the Kalshi record (`research/kalshi15/RESUME.md`) and the resume pack: his push is now
  two-part with the FULL capture in the `.jsonl.gz` (prefer `.gz` when both exist); his side was dead/repaired with a
  ~17:00Z–22:36Z gap; the desktop capture remains the primary record; readiness check 8 stays NA by construction.
- Kalshi resume pack and the Monday bookings stay ready to execute (01:00 reset -> units -> readiness 2026-11-01 ->
  fit-split pull -> Stage A order); the pre-09:05 unit-state fix remains first in the Monday collision order.
