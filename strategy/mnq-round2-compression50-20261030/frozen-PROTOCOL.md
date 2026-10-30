> Publication copy: local absolute paths in the checker invocation were redacted; original protocol SHA-256 is recorded in provenance.
> The historical checker command below is a non-runnable reference to the original workspace; this export does not contain its full inputs. Do not rerun it.

# Prop search round 2 — frozen exposed-history walk-forward batch

This research-only batch fixes 16 new candidates and their evaluation before any historical grading by the maker. It reuses the first batch's source, sample dates, exclusions, folds, baseline and double-cost policies without changing or grading round 1. All history has been previously exposed, and no part is a fresh holdout. The separately sealed prospective ORB15 sample is not an input. Kronos is not used. No strategy is authorized to trade, and no outcome is guaranteed.

## Candidates and timing

Each market (MNQ, MES) has two fixed parameter choices in each of four families. All signals consume completed 09:30–09:59 America/New_York bars and place at most one modeled order at the 10:01 bar open; all positions are closed by the 11:30 bar open. Breakout, direction or range ties produce no trade.

- **Compression breakout (50%, 75%)**: compare the second 10-minute high–low range (09:40–09:49) with the first (09:30–09:39); if the first range is nonzero and the second is no larger than the selected fraction, go with the 09:59 close breaking the second range.
- **Failed-break reversal (1, 2 ticks)**: use the first 10 minutes as a reference range. A later high at least the selected distance above it followed by a 09:59 close below the reference high gives a short; a later low below the reference low followed by a close above it gives a long. If both conditions hold, take no position.
- **Range-location reversal (20%, 30%)**: use the first 20-minute high–low range. Buy if the 09:59 close is at or below the lower selected fraction, or sell if at or above the upper selected fraction; a zero-width range is no signal. The close may be outside the earlier range.
- **Trend pullback (4, 8 ticks)**: the sign of the first 15-minute close versus its 09:30 open sets direction. For an upward trend, require a later low at least the selected distance below the first-period high and a 09:59 close above the 09:58 close; mirror for a downward trend. A flat first-period comparison has no signal.

One contract per independent candidate; protective stop 120 MNQ ticks or 48 MES ticks. The bar-based risk replay has conservative intrabar assumptions and cannot know actual path, partial fills, spread, latency or execution quality. The source inventory is one-minute OHLCV; no tick/order-book reconstruction or actual fills are assumed.

## Frozen data, exclusions and scenarios

The historical input is the previously audited, fixed-map 09:30–11:30 OHLCV slice: 1,397 retained NYSE full sessions from 2021-01-04 through 2026-10-23. The manifest retains all 52 outcome-free coverage/condition exclusions from the original 1,449-date universe. The source-inventory and original dated map are hash-bound; the calendar roll is a research convention, not a certified exchange expiry, and the sample-selection rule may bias results. Historical observation timestamps are not live-capture receipts.

Both baseline and doubled modeled costs are run for every candidate in every fold. Baseline: $2.50 round-trip plus one adverse 0.25-point tick per side, corresponding to $3.50 MNQ and $5.00 MES for one round trip; doubled: $7.00 and $10.00. These are assumptions, not measured fees/spread/slippage. The reused portfolio replay and manifest bind the modeled daily −$400 and +$900 stops, single-contract cap, drawdown and session-flat assumptions; they do not prove firm compliance.

Excluded families: overnight gaps or previous-close rules (the fixed slice has no such observations; independently checked complete coverage and an independently preregistered selection policy would be required); MGC/GC (no such raw directory found); tick/order-book or genuine trade VWAP (the source is minute OHLCV); and multi-leg portfolio rules (not tested in this single-position batch). An outcome-free overnight coverage addendum inherited from round 1 is included as research context only and does not select or alter this batch.

## Walk-forward, R3 and independent scoring

The three predeclared expanding folds are 2021–22 training / 2023 test, 2021–23 / 2024, and 2021–24 / 2025–2026-10-23. The first retained test session of each fold is embargoed. Outcomes never overlap overnight: each trade is flat the same session. Every candidate is evaluated on training before any test signals; the highest training baseline net per market/family is selected, with stable candidate-ID tie breaking and no-trade candidates ineligible. Earlier fold tests enter later training only as specified in this expanding walk-forward protocol. Report every candidate and both scenarios, not just selected candidates.

Each scenario reports test and training trade counts, net, daily net, event ledger, drawdown breaches and maximum daily-close drawdown; daily-close drawdown does not represent intraday drawdown. Each candidate's pooled, nonoverlapping test periods are R3 `MODELED_NONNEGATIVE` only if at least one test trade exists and own after-cost net is nonnegative; zero trades are not evaluable. Separate fold accounts reset, so pooled net is not a single compounded account. Costs, actual execution and compliance remain unverified. Multiple testing and prior exposure prevent blind/forward/significance claims; a modeled R3 pass is necessary and not proof of profitability or grounds for live activation.

Before any historical score, the independent checker must verify the manifest and all source/code/runtime hashes, confirm a different session than the maker, retain the manifest and initial ledger head externally, and then execute the full batch once. The ledger appends its look claim before historical bars are loaded and retains failure events; no silent reset or result-dependent rerun. Checker identity is procedural, not authenticated by the CLI's string check; an external ledger head is needed to detect rollback. The final result is emitted only after all folds, candidates and scenarios finish. Archive and read back the checker result and ledger via the authorized shared-archive owner. This maker has no write access to that shared archive.

From a fresh checker workspace on this host:

```
python3 -B ./evaluate.py --manifest ./manifest.json --manifest-sha256 MANIFEST_SHA --checker-session UNIQUE_CHECKER_SESSION --ledger "$PWD/prop-search-round2-ledger.jsonl" --expected-head INDEPENDENT_HEAD --output "$PWD/prop-search-round2-result.json"
```

For a newly created, independently retained empty ledger use 64 zeroes as its initial head. The checker must independently review signal causality, stop/flat simulation, data/source hashes, full candidate output and cost arithmetic before the consolidated performance report. Maker verification only: `python3 -B -m unittest discover -s tests -v` uses synthetic bars and does not establish performance. No post-result tuning is authorized within this manifest; a further iteration needs a separately recorded new batch.
