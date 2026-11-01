# JOB 12 — SETTLE-OR-CLOSE + MNQ + ROUND 3 dispositions (evidenced)

trading-lead, 2026-11-01 02:5xZ (Sat 31 Oct 22:5x ET). Read-only probes; no live action, no orders, no sign-in, no
credentials, $0. Repo probed: `~/Work/projects/openbell` (its pre-existing `live/` WIP untouched).

## Item 4(a) — settle-or-close: MES VWAP proof (UNVERIFIED) and MES costs (UNREAD) — **CLOSED-WITH-REASON, both still unproven**
What the brief asks: either find a live capture route and close them, or write the evidenced closure note (the capture lane that
owned them is closed). The honest answer is the second: **no live capture route for either exists on this desktop**, and no
record here contains a proof or a realised cost read.

**MES VWAP proof — UNVERIFIED, and the local record says why.**
- Every VWAP hit in the repo is *strategy* research, not a VWAP proof: `reports/RESEARCH-2026-10-26.md` (VWAPTrend family) and
  `reports/PREREG-SCREENSHOT-TRADER-2026-10-27.md` line 87 ("VWAP from today's 09:30 open, weighted by proxy tick volume").
- The futures-adjacent VWAP computation on record is **proxy, not futures** — `reports/RESEARCH-2026-10-26.md` line 116,
  quoted verbatim: *"Dukascopy USA500 is a cash-index CFD with BID prices only. It has no futures basis, no roll, and
  tick-count volume instead of contracts, which affects the VWAP weights. The futures cross-check could not run because of
  HTTP 429. Intraday S&P paths usually track ES closely, but that was not verified here."*
- No MES bar/VWAP artifact exists elsewhere: the forward archive holds OHLCV bars only (`data/forward/2026-10-30/bars.jsonl`
  keys `[c,h,l,n,o,src,ts,v]`, `src: tv-ws`), no VWAP field, no proof file.

**MES costs — UNREAD for realised costs, and the desk's own record has no fills or fees to read.**
- `data/forward/2026-10-30/manifest.json`, `account` block: `"fill_rows": 0, "fee_rows": 0` (1,288 account rows: contract
  1,280 / account 2 / cashBalance 2 / accountRiskStatus 2 / userAccountAutoLiq 2).
- The broker-record mirror `live/ledger/account-2026-10-30.jsonl` has **no fill or fee rows at all**: kinds counted are
  `contract 1284, account 2, cashBalance 2, accountRiskStatus 2, userAccountAutoLiq 2, dom 1`.
- The only cost numbers on record are **modelled constants, not reads**: contract §17 constants "1 MES; $5 a point; tick 0.25;
  $2.50 a round trip" and `reports/FIRST-PASSAGE-MNQ-2026-10-28.md` "real = $2.50 round trip + 1 tick ($0.50) slippage a side
  = $3.50 a contract". They are quoted here **labelled modelled**, never as the unread item.

**Why the lane is closed / disposition.** The capture lane that owned the live MES VWAP and cost records is closed (brief);
on this desktop the only MES feed is the passive Tradovate tab over CDP (weekend-dead; `feed "retrying"` since Friday close)
and the archive is MES-hardcoded with zero fill/fee rows. Disposition: **VWAP proof stays UNVERIFIED; MES costs stay UNREAD
for realised costs — both closed with this evidence, not silently dropped, and not "closed as passed".** What reopens them: a
real trading day with fills (read the costs from the broker record, not a research lane) and a futures VWAP cross-check when
the data route returns. No substitute data, no relaxed bar.

## Item 4(b) — MNQ prospective test: **BLOCKED, and the blocker is verified from the local record**
- **No evidenced feed-lane failure report exists on this machine.** Searched the repo (`reports/`, `research/`), the board and
  `~/Work`: the only MNQ records are (i) `reports/PREREG-FLOWCLOCK-2026-10-27.md` — a *different* family (Topstep/ProjectX,
  MES+MNQ), (ii) `reports/FIRST-PASSAGE-MNQ-2026-10-28.md` + `research/scalp/first_passage_mnq*` — a **historical** Databento
  study whose verdict is FAIL (0 of 36 rows with a positive after-cost interval), **not** the frozen prospective shadow test,
  and (iii) `data/databento/raw/MNQ/` raw bars. No prospective-test preregistration, no first-bar record, no failure report.
- **The blocker is confirmed in the record format:** `data/forward/2026-10-30/bars.jsonl` bar keys are
  `[c,h,l,n,o,src,ts,v]` — **no symbol, no contract, no receipt timestamp**; `live/bars.py` `tick(t, price, vol, src="tv-ws")`
  carries no symbol; and the archive/reader is **MES-hardcoded** (`research/forward/archive.py:39` `TICK, PT_VALUE = 0.25, 5.0
  # MES`, `:48` `ENTRY_CARD = re.compile(r"OpenBell (BUY|SELL) (\d+) MES")`, `:346-348` contract ids filtered to names starting
  "MES" plus `config.contract_id`). A reader asked for MNQ therefore returns MES — the brief's two claims are exactly right.
- Disposition: **MNQ start stays BLOCKED** — no qualifying bars, no substitute data, no relaxed bar; the feed lane's evidenced
  failure report (or a working route with symbol/contract/receipt fields) is a prerequisite and does not exist here.
  `FIRST-PASSAGE-MNQ` is explicitly **not** this test and must not be cited as its result.

## Item 5 — Round 3 continuation: **BLOCKED — the frozen harness/data is not on this machine**
- Grep for `FRAGILE` / `FRAGILE_OR_FAIL` across `~/Work/` and the board: **0 hits**. `research/promptstrat/` holds the
  preregistration (`reports/PREREG-PROMPT-STRATEGIES-2026-10-27.md`), `build_log.json` (keys: chart_validator, codeseeds,
  day1_fixes, sim_core), `freeze_manifest.json`, and the sim/exits/bracket code — but **no 16-candidate Round-3 set, no frozen
  scoring run, no FRAGILE verdict record**.
- Disposition: **BLOCKED**, named honestly. Continuing "through the SAME frozen gates" requires the frozen Round-3 candidate
  files and their scoring harness; re-deriving the 16 candidates from the brief would be a new, unfrozen construction and is
  forbidden by the rule. Nothing was improvised. Successor: the lane that holds the Round-3 set must hand it over (or state it
  is lost), then the continuation runs unchanged.

## Fences
No `live/` file touched, no order, no wallet, no sign-in, no credential, no spend, nothing sent. Only the Kronos report was
written (via its own committed writer) and these two records were created under `board/`.
