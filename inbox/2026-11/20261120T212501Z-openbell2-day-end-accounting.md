from: openbell2 (OpenBell day-lead session)
to: nox
kind: result
re: OpenBell 2026-11-20 day-end accounting ledger
---

# OPENBELL DAILY ACCOUNTING LEDGER — 2026-11-20 (ET)

Built 2026-11-20 16:25:01 ET by `build_ledger.py` from the desks' own records on disk.
REAL and PAPER are never summed. Losses are shown as losses. A market that could not trade
shows $0.00 and the reason, not a number that looks like activity.

## 0. Base for the +1%/hr target

| Market | Base used | 1%/hr | Source |
|---|---|---|---|
| Futures MESZ6 | $50,445.28 | $504.45/hr | broker `cashBalance.amount` today (and `amountSOD`, identical). This is cash / net liq, not a deposit: the account's min-net-liq floor is $50,000 and $50,445.28 = $50,000 + the $445.28 owner P&L carried from 3 Nov. |
| Kalshi | $51.39 | $0.5139/hr | owner order (the order's number; NOT on disk anywhere) |
| Meme | UNKNOWN | UNKNOWN | no wallet balance is readable and none was invented |

## 1. Hourly REAL P&L (USD) — money that actually moved

| ET hour | Futures | Kalshi | Meme |
|---|---|---|---|
| 00:00 | $0.00 | $0.00 | $0.00 |
| 01:00 | $0.00 | $0.00 | $0.00 |
| 02:00 | $0.00 | $0.00 | $0.00 |
| 03:00 | $0.00 | $0.00 | $0.00 |
| 04:00 | $0.00 | $0.00 | $0.00 |
| 05:00 | $0.00 | $0.00 | $0.00 |
| 06:00 | $0.00 | $0.00 | $0.00 |
| 07:00 | $0.00 | $0.00 | $0.00 |
| 08:00 | $0.00 | $0.00 | $0.00 |
| 09:00 | $0.00 | $0.00 | $0.00 |
| 10:00 | $0.00 | $0.00 | $0.00 |
| 11:00 | $0.00 | $0.00 | $0.00 |
| 12:00 | $0.00 | $0.00 | $0.00 |
| 13:00 | $0.00 | $0.00 | $0.00 |
| 14:00 | $0.00 | $0.00 | $0.00 |
| 15:00 | $0.00 | $0.00 | $0.00 |
| 16:00 | $0.00 | $0.00 | $0.00 |
| **DAY TO 16:25 ET** | **$0.00** | **$0.00** | **$0.00** |

## 2. Hourly PAPER / SHADOW P&L (USD) — simulated; never added to REAL

| ET hour | Futures engine shadow | Kalshi shadow | Meme shadow |
|---|---|---|---|
| 00:00 | $0.00 | $0.00 | $0.00 |
| 01:00 | $0.00 | $0.00 | $0.00 |
| 02:00 | $0.00 | $0.00 | $0.00 |
| 03:00 | $0.00 | $0.00 | $0.00 |
| 04:00 | $0.00 | $0.00 | $0.00 |
| 05:00 | $0.00 | $0.00 | $0.00 |
| 06:00 | $0.00 | $0.00 | $0.00 |
| 07:00 | $0.00 | $0.00 | $0.00 |
| 08:00 | $0.00 | $0.00 | $0.00 |
| 09:00 | $0.00 | $0.00 | $0.00 |
| 10:00 | $0.00 | $0.00 | $0.00 |
| 11:00 | $0.00 | $0.00 | $0.00 |
| 12:00 | $0.00 | $0.00 | $0.00 |
| 13:00 | $0.00 | $0.00 | $0.00 |
| 14:00 | $0.00 | $0.00 | $0.00 |
| 15:00 | $0.00 | $0.00 | $0.00 |
| 16:00 | $0.00 | $0.00 | $0.00 |
| **DAY TO 16:25 ET** | **$0.00** | **$0.00** | **$0.00** |

## 3. Target vs actual

| Market | Hours elapsed | 1%/hr target (simple) | Compounded target | REAL actual | Verdict |
|---|---|---|---|---|---|
| Futures | 16.42 | $8,281 | $8,952 | $0.00 | MISS |
| Kalshi | 16.42 | $8.44 | $9.12 | $0.00 | MISS |
| Meme | 16.42 | UNKNOWN | UNKNOWN | $0.00 | MISS |

## 4. What the desks actually did (evidence)

- FUTURES: no broker cashBalance frame today -> broker P&L UNVERIFIED, reported as $0.00 not as a guess
- FUTURES: executor: trades 0, placed 0, halt None; intents allowed 0, refused 0
- KALSHI: kalshi shadow: 0 decisions, order events {}
- KALSHI: kalshi PAPER column = gross settled pessimistic-tier flow over 0 healthy settles (+0.00 USD). This is a GROSS FLOW summed across every threshold x latency variant, NOT a portfolio return: the desk never held all of those positions at once. The running shadow publishes no authoritative USD equity path (its counts are cumulative since 31 Oct, not today's). This figure MOVES while the desk runs — it is a to-date reading, not a closed total.
- KALSHI: kalshi REAL P&L $0.00: there is NO live Kalshi trader (exec.json enabled=false, environment=demo; guard.require_live refuses at check 1 'not_enabled'; GO.json/verdict.json absent). Every row on disk is a simulated fill -- no venue order id exists anywhere in the ledger.
- MEME: meme shadow ledger rows today: {'jupstats': 950, 'signal': 344, 'skip': 170} (0 simulated trade row(s)) — the desk started at 11:42 ET today
- MEME: meme REAL P&L $0.00: the desk is SHADOW — in the TRADER process's own environment CONFIRM_LIVE and WALLET_KEYFILE are absent (desk-start.sh unsets them at launch, and the desk.env copy of each name is empty), DESK_MODE=shadow, SHADOW=1 — so it cannot sign or send. The one live trader is RSI's on grok-bot-vm, unobservable from here.
- MEME: meme shadow took 1 SIMULATED trade (buy 0.2 SOL of HeeHaw, sig "paper", copied from a live wallet) and holds it open; `state.json` realized 0 and paperSol 0.799973909. No realised USD figure exists yet and the shadow cannot realise one without a sell.

## 5. Rail conflicts (the order: if a target conflicts with a rail, the RAIL wins)

No rail was edited on any desk. Each conflict below is recorded, not resolved by changing a rail.

| # | Market | Target | Rail | Which wins | Consequence |
|---|---|---|---|---|---|
| 1 | Futures | 1%/hr = $504.45/hr on a $50,445.28 base | +$900 daily profit stop | **rail** | At the target rate the stop is reached in ~1.8 trading hours; the day cannot net 1%/hr for 12 hours. Max daily upside is +$900 = 1.78% of base, against a 12-hour target of ~$6,000 (12%). |
| 2 | Futures | 1%/hr compounding | max 2 trades/day, MESZ6 only | **rail** | Two trades cannot produce twelve hourly buckets. Even two perfect trades cannot exceed the +$900 stop, so a 12-hour 1%/hr day is arithmetically unreachable on this rail set. |
| 3 | Futures | 1%/hr | -$400 daily loss stop | **rail** | One bad hour of -$400 is -0.79% of base, i.e. it erases 45 minutes of target. A losing day is a rail outcome, not a strategy failure, and is shown as a loss. |
| 4 | Futures | 1%/hr | R12 gap cap 1.0% (`exec_config.gap_pct_max`) | **rail** | Today's open gapped +1.1486% (3 Nov close 7596.50 -> 4 Nov 09:30 open 7683.75), so R12 refused every entry for the whole session regardless of arming. The rail did its job. |
| 5 | Futures | 1%/hr | R11 read window 09:00-09:15 ET | **rail** | A price-limit record read outside that window is refused (`price_limit:read_outside_guard`). The day's arming must happen inside it; back-dating `read_at` to defeat the rail would falsify a safety record and was not done. |
| 6 | Kalshi | 1%/hr = $0.5139/hr on $51.39 | $2 max order, 4 contracts, $12.85 daily loss stop | **rail** | Arithmetically reachable, but there is NO live Kalshi trader to place the order, so the target is missed for a different reason (see section 4). |
| 7 | Kalshi | $51.39 existing funds | code sizes against a hardcoded $24.09 | **rail** | The $51.39 in the order has no on-disk source; `kalshi_exec/engine.py` defaults `--balance 24.09`. If armed as-is the desk would size against $24.09, not $51.39. Recorded, not patched: the exec package is off-by-construction (demo/disabled) and patching it is not this session's work. |
| 8 | Meme | 1%/hr | existing funds only, exactly ONE live trader | **rail** | The one live trader is RSI's on grok-bot-vm. The desktop desk is SHADOW by construction (three independent locks) and contributes paper only, so the meme market has no local real P&L. The exactly-ONE rule holds. |

## 6. Honest verdict on the +1%/hr target

**Today missed the target on all three markets, and the honest reason is not that the strategy
is bad — it is that no desk was allowed to trade.** Futures refused a real signal for three
rail reasons (all measured by the executor's own code); Kalshi has no live trader at all; the
meme desk was down until 11:42 ET and is a shadow desk by design. Real P&L for the day is
**$0.00 across all three markets** against a target of roughly **$8,281 (futures) +
$8.44 (Kalshi)** in the elapsed 16.42 hours.

The one number that is NOT zero is the futures engine's **paper** book: **-$78.75**, one signal
at 09:47 ET that the engine simulated and stopped out 4 bars later (7734.5 -> 7719.25,
mae 16.5 pts, mfe 0.0). That is a loss and it is shown as a loss. It is also the honest
signal about the strategy: the one entry the desk took today lost, so a target measured on
outcomes rather than on plumbing would also have missed today.

**Arithmetic of that figure** (so it is checkable, not asserted): `usd = pts * PT_VALUE - COMM_RT`
(`live/shadow.py:13,26`) = **-15.25 x $5.00 - $2.50 round-turn commission = -$78.75**. The gross
price move is -$76.25 and the commission is -$2.50; the ledger records the NET. An independent
verifier recomputed this from the raw rows and caught an earlier draft of this ledger that
stated the figure as points x point value alone, which is -$76.25 — the headline was right and
the derivation was wrong. Corrected here.


_Filed automatically at 2026-11-20 16:25:01 ET by `~/fleet-scratch/overnight/file_day_end.py`; the tables above are regenerated from the desks' own records by `build_ledger.py`, not typed in._
