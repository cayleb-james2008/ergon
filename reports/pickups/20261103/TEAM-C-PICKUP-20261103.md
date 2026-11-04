# TEAM C — PICKUP PACK (OpenBell futures + Kalshi)

**For:** the DeepSeek continuation after Claude's weekly limit (Nox order `20261103T231027Z`, 18:10 ET; the 18:02 handoff replies were cut off mid-write).
**Assembled:** 2026-11-03 19:35 ET, read-only from recorded state. No board push — the Lead files pointers.
**Owner:** Team C / `trading-lead`. The **ORB15 beta desk is Team C/Nox-owned**, not the Lead's runbook desk (Nox `20261103T144500Z` §2).

## Bottom line
- Futures desk is flat: **0 signals, 0 engine fills, 0 positions, day P&L $0.00**. The only P&L is the owner's manual trade (+$445.28 net, 07:20–07:22).
- The autonomous **ORB15 executor was armed 11:14:32–17:00:01 ET under Nox's enable and self-halted at expiry with zero intents** (`intents-2026-11-03.jsonl` holds only the owner-activity rows and the halt). "Observation-only" is true at the fill level, not at the arm level.
- The feed fault is fixed: `cdpfeed.find_target` now pins `127.0.0.1:9222` (was `localhost`, which resolved to a second `[::1]:9222` Chromium); engine restarted 09:28:59, feed resumed 11:12:17 after the re-login.
- ORB15 **HOLD** is the futures loop's own decision at 19:07: `{"cause": "FEED_SILENCE", "decision": "HOLD", "fills": {"fills": 0, "net_usd": 0.0}}`, then `R-SILENT: … 2026-11-03=FEED_SILENCE -> not fired`. Quote, not interpretation.
- Kalshi is **shadow-only and running clean** (3 units, 0 errors/1h). Pack 23 is delivered and accepted; Kalshi15 Stage A is still a **DRAFT**, blocked on the historical prints pull (not running) and open Nox rulings.

## What this is
- **Futures:** `openbell-live` MESZ6 signal engine (passive Tradovate feed over CDP, shadow ledger, ntfy cards, never places orders) + the separate **ORB15 executor** (`openbell-executor`) that can place 1 MES inside the rails when attested and armed. FundedNext Flex 50K eval; rails -$400/+$900, max 2 trades, 15-min cooldown, flat 16:05.
- **Kalshi:** 15-minute crypto shadow forecaster (public REST + 1 s exchange reference; paper ledger), the Pack 23 model-vs-market DEV read, and the Kalshi15 Stage A build whose live-order package ships OFF by construction.
- **Not in scope here:** the Lead's runbook desk (separate record), the meme desk, and any live Kalshi order.

## Current state (with evidence)

### Futures desk
- `live/heartbeat.json` read 19:29:43 ET: `status running`, `mode automated`, `feed ws`, `feed_age_s 0.0`, `last_price 7576.0`, `bars 6203`, `last_bar_et 19:28`, `pos null`, `trades 0`, `day_pnl 0.0`, `paused false`, `market_open true`.
- 11 cards today, all lifecycle (`tv-stuck`, `tv-login`, `start`, `stale`, `fresh`); **no signal card** (`live/logs/cards.jsonl`; desk record lists the same 11).
- Account state (`live/account_state.json`, read 19:35 ET): `status attached`, `tab_path /`, DOM `equity 50445.28`, `open_pnl 0.0`, `attached_t` 19:20:32; its accounts/positions block is older (`t` 17:33:51), and the executor's panel flat-proof is unreadable — treat flat as the engine+ledger claim, not a fresh broker proof.
- The owner's manual trade is **owner P&L, not engine/executor P&L**: buys 1+2 then sells 3, 07:20:08–07:22:11, net **+$445.28** (`openbell-executor/live/exec/state/day-2026-11-03.json`; owner described "2 ES" — the ledger shows 3 contracts).
- **Executor:** attestation `PASS` at 11:12:25 (five legs, `attestation-2026-11-03.json`), armed 11:14:32 (`notify.jsonl`), enable expiry 17:00:00 (`enabled.json`, go_id `NOX-ARMING-ORDER 2026-11-03 07:32 ET`), halt at 17:00:01 `leg_failed:armed` (`day-2026-11-03.json`). `trades 0`, `placed []`.
- Executor's own panel flat-proof reads **"not flat or unreadable"** at 19:31 (`flat_proof.json`); account record and engine both say flat. Treat the panel read as unavailable, not as evidence of a position.
- Feed fault: engine log 09:13–09:28 `RuntimeError('no trader.tradovate.com tab')` every 30 s; Nox root-caused it as a **localhost/`[::1]` debug-port collision, not the tab** (`20261103T144500Z` §2). The live tree's `live/cdpfeed.py` diff is exactly `localhost` → `127.0.0.1` (uncommitted, loaded by the 09:28:59 restart). The second `[::1]:9222` Chromium is documented read-only in `reports/team-e-n5-result-20261102.md` §2.
- Timeline: restart 09:28:59 → stale 09:31 → feed OK **11:12:17** (`NehBQx9UbI1r`) → 16:05 flat / 16:20 outcome both flat, 0 trades (`TUESDAY-20261103-DESK-RECORD.md`).
- `openbell-futures-loop` (hourly timer): 19:07 cycle `HOLD`, `FEED_SILENCE`, `0 fills`, R-FILLS 0/0, net $0.00; next run 20:07.

### Kalshi
- `openbell-kalshi-poll` / `-ref` / `-shadow` are **active** (started 2026-10-31 13:34 ET): poll 465,847 requests, **1.867 req/s**, `gaps_today 0`, `errors_1h 0`; ref **7/7 venues connected** (coinbase, kraken, bitstamp, gemini, cryptocom, bullish, binance); shadow `decisions_today 234`, `fills_today 434` (paper), `orders_open 180`, chain seq 39,276 decisions / 264,970 orders, `errors_1h 0`, `params_status PROVISIONAL (dry Stage A, theta unset)`.
- **Shadow-only rails:** the shadow unit says "paper only: reads local capture files, writes its ledger; no network, no orders"; the order path is `live/kalshi_exec/`, **OFF by construction** (six-check fail-closed guard; only `transport_live.py` may open a socket; guard tests assert refusal). No credentials, public REST only, no spend.
- **Pack 23 is delivered and accepted** (Nox ruling 6, `20261102T222300Z` §6): `predictions.csv` 558 rows, sha256 `5ab332b5…6254`, branch `pack23-inputs` @ `3f59b9b`; independent regeneration in `kalshi-pack23-wt` @ `cb320e2` is byte-identical (`PACK23-PREDICTIONS-REPORT.md`, 2026-11-02). Branch stays local/unpushed for the owner's sitting.
- Exclusion accounting filed: 2,008 windows, **42 excluded** (9 first-window `book_gap` + 33 DST-hour `gap_record`), last at `2026-11-01T06:51Z`; no exclusions since (`reports/team-c-kalshi-exclusion-accounting-20261102.md`).
- **Kalshi15 Stage A is DRAFT, not a verdict:** θ UNSET, P/L **NOT EVALUABLE** (live latency unmeasured), bars not met; the historical fit-split prints pull is **not running** (`pgrep -af "pull.py prints"` empty), resumable from `research/kalshi15/RESUME.md`. L* = 30 s was ruled and applied; the live gate must use the measured row, never the best row.
- The shadow heartbeat's `lag_s 68.7` is heartbeat granularity, **not** decision availability; measured availability is 16.1 s p50 after the decision tick (`results/availability_2026-10-31.json`, cited in `DECISIONS.md` Amendment 2).

## Running now
- `openbell-live.service` — active/running, PID 2401904, since 09:28:59; entries only 09:46–12:00, so it is logging overnight bars.
- `openbell-executor.service` — active/running, PID 189363, since 11:14:32, but **self-halted for the day**; enable expired 17:00.
- `openbell-dash` (dashboard + passive account watcher), `openbell-tabkeeper` (`tradovate ok / session ok` at 19:33), `openbell-sessionwatch` (`state ok`), all active.
- Kalshi poll/ref/shadow — active; the shadow keeps writing paper decisions.
- Timers: futures-loop next **20:07**, forward-archive next **2026-11-04 17:20**.
- **Stopped:** the Kalshi historical prints pull; `openbell-futures-loop.service` between timer runs; the Kalshi order path (by construction).

## Owed / decisions
- **Nox → the desk:** a fresh day enable (`go_id` + expiry) before any new session; the five-leg attestation and a **different-seat inspect** must pass; the R11 price-limit record must exist with `read_at` inside 09:00–09:15 ET (today's is provisional, desk-bar stopgap, `state/r11/cme_limits-2026-11-03.json`).
- **Nox/owner:** accept the **first real Send** risk — post-Send dialog, TP/SL attachment and flatten/cancel are unverified (`exec_config.json` `cdp_tradovate.note`; `ARMING.md` "What the first real signal will prove"). The executor fails safe before Send and halts after it on anything unexpected.
- **Nox → Kalshi:** amendments still marked FOR NOX in the resume pack — **Amendment 5** (order-path wording in the prereg) and **Amendment 6** (Kelly posture); no later ruling found. Also whether to resume the prints pull and when to run Stage A.
- **Team C owes:** the formal Kalshi15 **Stage A DEV report** (with the per-coin framing correction Nox accepted, not the pooled "43%") once data lands; the freeze still has 7 `<FILL AT FREEZE>` slots, and its 3–4 Nov target is not reachable while the prints pull is stopped.
- **Record gap:** I found the executor's 17:00 halt notice on the board (`inbox/2026-11/20261103T220001Z-claude-exec-to-nox-296.md`) but **no consolidated Team C day record** for 3 Nov. Whether one is owed is UNKNOWN.
- **Login:** a lapse is no longer a desk blocker (Nox). The **two-click routine** — "Log in" (autofill) then "Access Simulation", no typing, stop on 2FA/captcha/empty field — is recorded on Cayleb's desktop (Nox's act, desk record 10:43; `20261103T144500Z` §3), and the pickup order treats it as the **owner/Nox act**. The next lane routes it and verifies the resume; it does not perform it.

## Next steps to continue (ordered)
1. Read-only snapshot: `heartbeat.json`, `cards.jsonl`, `account_state.json`, `logs/tabkeeper.json`, `logs/session-state.json`, `openbell-executor/.../day-2026-11-03.json`, `intents-2026-11-03.jsonl`, `journalctl --user -u openbell-live -n 20`.
2. Confirm the executor is still halted and the enable is expired; **do not** delete/refresh `enabled.json`, do not re-arm, do not start a day.
3. If `tabkeeper`/`sessionwatch` says the session lapsed: record it, route the two-click routine to owner/Nox, then re-check heartbeat `feed ws` / `feed_age_s`; do not sign in and do not call it a desk blocker.
4. If Nox orders a new day: build the R11 record inside 09:00–09:15 → write the enable with Nox's exact `go_id`/expiry → `attest` → different-seat `inspect` → start only if all five legs PASS; otherwise stay observation-only.
5. Kalshi health only: read the three heartbeats + `ledger/counts.json`; no unit restart, no order, no credential.
6. Pack 23: no action; leave the branch local/unpushed unless the owner directs a push.
7. Only on Nox's word for Stage A: restart `pull.py prints` (≤5 req/s total, poller ~2), then RESUME §Pending 2 steps (a)–(f), including `second_scorer` AGREE and the matched-n note.
8. File the day's pointer/record through the Lead (no board push from this pack).

## Constraints / do-not
- **Read-only on `/home/cayleb/Work/projects/openbell`** — the units run from it, the tree is dirty on `research/kronos-stage1-verdict`; never write there.
- **No agent sign-in, no credential entry, no token file**; the two-click routine is the owner/Nox act.
- **No live trade or order**; no config/rail changes; no service restart/kill; no `enabled.json` edits.
- **Kalshi shadow-only:** no orders, no credentials, no spend, public REST only; `live/kalshi_exec/` stays OFF.
- Do not treat `account_state.json` as an order/position authority — its accounts block lags (17:33) and the panel flat-proof is unreadable; cross-check the tab/session.
- Do not use `localhost` for the desk CDP; pin `127.0.0.1:9222`.
- Do not call the ORB15 beta the Lead's desk; do not mix its state with the runbook desk's NO-DESK-DAY record.
- No board pushes; the Lead files pointers.

## Where things live
- Live checkout: `/home/cayleb/Work/projects/openbell` (`live/` state; `live/dash.py`; `live/engine.py`; `live/cdpfeed.py`).
- Executor: `/home/cayleb/Work/projects/openbell-executor` (`live/exec/`, `ARMING.md`, `state/r11/`).
- Kalshi: `/home/cayleb/Work/projects/openbell-kalshi` (`research/kalshi15/RESUME.md`, `live/kalshi/`, `live/kalshi_exec/`); `/home/cayleb/Work/projects/kalshi-shadow-data`; `/home/cayleb/Work/projects/kalshi-pack23-wt`.
- Board: `reports/team-c-*.md`, `reports/TUESDAY-20261103-DESK-RECORD.md`, `probes/team-c/pack23/`, `reports/team-e-n5-result-20261102.md`; inbox `20261103T144500Z-nox-to-delegator.md`, `20261103T220001Z-claude-exec-to-nox-296.md`.

## Open questions for Nox/owner
1. Is today's Team C/ORB15 day record filed anywhere beyond the 17:00 halt notice? (not found)
2. Did Duet's Pack 23 confirmation pass come back, and is the leg closed? (not found)
3. Did Nox rule Amendments 5/6, or do they still block the prereg freeze? (no later ruling found)
4. What are tomorrow's `go_id`/expiry, and is the unverified first real Send accepted?
5. Who performs the two-click routine if the session lapses — Nox or the owner? (record says Nox/Cayleb's desktop; pickup order says owner/Nox)
6. Why does the futures loop classify 3 Nov as `FEED_SILENCE` when the engine had live feed from 11:12? (its own classification; not investigated)
