# G0 — FIRM-AUTOMATION VERDICT: may this eval be traded automatically? (Job 12 successor gate)

trading-lead, 2026-11-01 ~09:0x ET / 14:0xZ. **Verdict: YES — automation is permitted on this eval, with conditions.**
Read-only web research; no send, no account, no support ticket, no credential, $0.

**G0 STATUS: PASS — SIGNED by the independent Tier S checker (G0-SIG-2026-11-01-TS-01), verdict file
`board/reports/team-c-g0-g1-checker-verdict.md` (sha256 `afc44f85…`).** All four cited pages were independently re-read and
matched verbatim; the account facts were confirmed from the firm's own Flex pages; the false "$50K EA ban" conflict was
confirmed killed. The checker's conditions are folded in below (marked **G0-C1…G0-C5**); a fresh re-read is required again
before G4. G1 is **HOLD** against design v2 — see `board/reports/team-c-autonomous-path-design-v3.md`.

## 1. Verdict in one paragraph
FundedNext Futures **explicitly allows automated trading systems (EAs and bots) during both the Challenge Phase and on
FundedNext Accounts**, on its three supported platforms (Tradovate, NinjaTrader, TradingView). It offers **no support** for
them and disclaims technical issues. It **does not offer a REST/API trading surface** in the pages reviewed — trading is
through the platform login — so on this eval the **CDP ticket route (driving the already signed-in Tradovate web UI) is the
viable autonomous path**, and it must be self-contained (no third-party app such as Telegram/WhatsApp inside the bot) and
must not fall into the prohibited-strategy classes (latency abuse, order flooding, spoofing, layering, grid, wash,
hedging, bracket-strategy exploitation, micro-scalping, 2%-of-price-limit trading). The ORB15 beta's shape (1 MES,
≤2 trades/day, minutes-long holds, one-sided bracket) sits outside every prohibited class on the evidence below, **subject to
the guards in §4** being implemented in `rails.py`.

## 2. Quoted clauses (verbatim; pages read 2026-11-01)
Source A — "Is the usage of Automated Trading Systems (EAs and Bots) allowed in FundedNext Futures?" (page dated
**April 9, 2026**), https://helpfutures.fundednext.com/en/articles/14298560-is-the-usage-of-automated-trading-systems-eas-and-bots-allowed-in-fundednext-futures :
> **TL;DR**: **FundedNext Futures allows the use of automated trading systems, including Expert Advisors (EAs) and trading
> bots**, during both the **Challenge Phase** and on **FundedNext Accounts**. However, **no support is provided** for setup
> or troubleshooting, and **FundedNext is not responsible for technical issues** or their impact. Any strategies that
> exploit **system vulnerabilities**—such as **latency abuse** or **order flooding**—are strictly prohibited.

Source B — "FundedNext Futures — Most Frequently Asked Questions" (page shows "Updated this week"; read 2026-11-01),
https://helpfutures.fundednext.com/en/articles/17225788-fundednext-futures-most-frequently-asked-questions .
**G0-C1 — the earlier version spliced two separate FAQ answers into one quote; the two answers are now separated:**
> **B(i) — "Can I use a VPN/VPS to trade?"** *"**Yes.** You may use a **VPS (Virtual Private Server)** or **VPN (Virtual
> Private Network)** while trading on both your **Challenge Account** and your **FundedNext Account**."*
>
> **B(ii) — "Is the use of Automated Trading Systems (EAs and Bots) allowed?"** *"At FundedNext Futures, you are allowed to use
> automated trading systems such as Expert Advisors (EAs) and trading bots during both the Challenge and on FundedNext
> Accounts. However, we do not offer any support for setting up, configuring, or troubleshooting these systems. Any
> issues—such as technical errors, execution delays, or system malfunctions—must be managed on your own. FundedNext will not
> be held responsible for any impact these may have on your trading account."*
> - *"Traders are required to customize their EA or bot settings to align with their unique trading styles."*
> - *"Each EA or bot is required to employ a distinct strategy, avoiding identical trades across any accounts within FundedNext."*
> - *"**Using an EA that incorporates third-party applications such as Telegram or WhatsApp is strictly prohibited.**"*
> - *"Using an EA for latency abuse or order flooding is strictly prohibited."*

Source C — "What Trading Platforms Does FundedNext Futures Offer?" (page dated **September 30, 2026**),
https://helpfutures.fundednext.com/en/articles/17225732-what-trading-platforms-does-fundednext-futures-offer :
> FundedNext Futures supports **three trading platforms** — **Tradovate**, **NinjaTrader**, and **TradingView**. Each
> connects directly to your FundedNext Futures Account …
No API surface is described on any page reviewed; the only access described is via those three platforms. The on-disk
survey (`PROP-FIRMS-2026-10-27.md`, read 2026-10-27, `confidence: official`) records the same substance as
`"api": "own bots/EAs on Tradovate, NinjaTrader or TradingView (Tradovate accounts); VPS allowed; no API"`.
**G0-C2 — keep "no API" as an INFERENCE: it is the absence of any API article/documentation across the pages reviewed, not a
denial sentence.** The exact survey sentence was not reproduced verbatim on the pages fetched today; today's pages confirm
the permission and the platform-only access. The design therefore says **"no API surface documented"**, not "there is no API".

Source D — "What are the prohibited trading strategies of FundedNext Futures?" (page dated **September 30, 2026**),
https://helpfutures.fundednext.com/en/articles/14298337-what-are-the-prohibited-trading-strategies-of-fundednext-futures :
> **TL;DR**: **FundedNext Futures strictly prohibits trading strategies that exploit simulated market conditions or
> manipulate execution** … Prohibited practices include **platform error exploitation**, **account sharing**, **account
> rolling**, **multi-order spam**, **slow data feed abuse**, **spoofing**, **layering**, **grid trading**, **wash trading**,
> **latency arbitrage**, **reverse hedging**, **hedging with correlated instruments**, trading in **gapped or illiquid
> markets**, and trading within **2% of CME price limits**.
Plus the specific bracket sentence: *"Bracket trading is a strategy where traders set predefined buy & sell (limit/stop
orders) to take advantage of any-sided market movement … Exploiting the lack of slippage or using tight brackets to benefit
from favorable trade executions is prohibited."*

## 3. Account-rule facts the build must honour (Flex $50K; pages read 2026-11-01)
| Fact | Source | Bearing on the build |
|---|---|---|
| **40% Consistency Rule applies during the Flex Challenge** (profit target $2,500; **daily threshold 40% × $2,500 = $1,000**; exceed it and required total becomes highest day ÷ 40%) | Flex Challenge All Rules (updated >2 weeks ago) | This **is** the provenance of the brief's "40% consistency / $1,000 single-day firm cap". It is a **recalculation rule, not a breach** — a day above $1,000 does not fail the account, it lengthens the target. The desk's rail (treat it as a cap) is stricter and stays. |
| **No daily loss limit** on the Flex Challenge or the Flex FundedNext Account | both rules pages | The -$400 daily stop is a **desk rail**, not firm-side — the runbook's "engine-enforced -400/+900, firm-side $1,000/40%" attribution is correct, with the semantic note above. |
| **Maximum Loss Limit $1,500, trailing EOD, locks at $50,100**; a floating loss that breaches it breaches instantly | both rules pages | Hard firm breach line. The architecturally safe rule remains: no trade whose stop-out would cross -$400 (desk) and never approach MLL. |
| **No overnight or weekend holding**; all positions closed by **3:10 PM CT = 4:10 PM ET**, auto-closed if open; resumes 5:00 PM CT | Futures FAQ | Desk flat **16:05 ET = 3:05 PM CT** → 5 minutes before the firm's auto-close. Compliant; the executor's 16:05 auto-flatten has margin. |
| **No consistency rule and no daily loss limit on the funded account** | Flex FundedNext Account All Rules | If/when the eval passes, the "firm-side 40%/$1,000" attribution disappears (desk rails stay). Record the phase when quoting these rules. |
| **Contract limit $50K = 3 e-mini / 30 micro** | both rules pages | Beta 1 MES is well inside; no size change contemplated. |
| **Micro-scalping = trades closed within 10 s**; 30% warning, 40% = advance placed on hold (Challenge) / profit deducted (funded) | rules pages + micro-scalp policy | The beta's holds are minutes-to-hours by design; add an explicit guard: **no round-trip under 10 s** in `rails.py`. |
| A *different* FundedNext division (CFD/MetaTrader) bans EAs on accounts **$50,000 and above** | **G0-C3 — exact quote, `help.fundednext.com` article "Is EA allowed in FundedNext?" (page dated September 8, 2026), read 2026-11-01, https://help.fundednext.com/en/articles/8020763-is-ea-allowed-in-fundednext :** *"TL;DR: Yes, FundedNext allows the use of Expert Advisors (EAs), indicators, and trading bots on MetaTrader 4 (MT4) and MetaTrader 5 (MT5) platforms for accounts below $50,000. Traders on account sizes of **$50,000 and above** must trade manually and may not use Expert Advisors, trading bots, or any automated tools—including tools that only modify trade parameters such as SL, TP, or lot size. FundedNext does not allow the use of EAs or automated trading on the cTrader and Match-Trader platforms regardless of account size."* | **Not applicable and must not be cited as a blocker**: our eval is FundedNext **Futures** Flex $50K (automation explicitly allowed, Sources A/B); this article is the **CFD/MT4/MT5/cTrader/Match-Trader** division. Recorded to kill a false conflict before it is raised. |

## 4. The proposed autonomous path against the prohibited list (G1 must encode these guards)
| Prohibited class | Does the ORB15 beta / CDP executor touch it? | Required `rails.py` guard |
|---|---|---|
| Latency abuse / arbitrage | **No** — the desk decides ~16–70 s after data; the opposite of latency exploitation | refuse if intent is derived from data older than the freshness rule; never use out-of-date prices |
| Order flooding / multi-order spam | **No** — one order in flight, ≤2 trades/day | max one resting order per intent; cancel-before-new; refuse >2 trades/day |
| Spoofing / layering / grid / wash | **No** — one-sided bracket, one instrument, no cancellation churn | single intent per decision; no re-placement without a new decision; refuse if cancel-rate > policy |
| **Bracket-strategy exploitation** | **Not in the prohibited sense** — a normal one-directional breakout with a protective stop and a target, not "any-sided" OCO entry to exploit no-slippage fills | assert: exactly one side per intent, stop and target on the *same* side as the entry, no simultaneous buy+sell bracket, no re-entry from a stale bracket |
| Simulated-execution exploitation / tick scalping | **No** — holds are minutes; entry only 09:46–12:00 | refuse round-trips < 10 s (micro-scalp guard); refuse entries outside the window |
| Trading gapped/illiquid markets | **No by design** (RTH open window) — but must be enforced | refuse if session is a holiday/early close, if the market is gapped/illiquid per the session state, or outside 09:46–12:00 |
| Trading within **2% of CME price limits** | **No** — MES RTH opens are far from limits; must be enforced | refuse if last price is within 2% of the CME limit for the session (config value) |
| Third-party app inside the EA (Telegram/WhatsApp) | **No** — the executor is self-contained; notifications are after the fact | assertion in G2: the executor imports no messaging client; cards/ntfy stay outside the send path |
| Account sharing / copy / group | **No** — one trader, one account, own decisions | single-trader flock; no copy module; refuse if another order-placing process is found |

## 5. What G0 changes for the build (design **v3**, G1)
1. **Basis confirmed**: CDP ticket automation of the signed-in Tradovate tab is permitted in substance (automation allowed on
   the platform); NinjaTrader EA remains the supported alternative if Nox rejects DOM automation — no API surface documented.
2. **New rails** derived from the firm rules to add in `rails.py`: micro-scalp (<10 s) refusal, 2%-of-CME-price-limit proximity
   refusal, phase-aware consistency accounting (Challenge 40%/$1,000 recalculation vs funded account none), MLL proximity
   guard, self-contained-bot assertion, and the "one side only" bracket assertion.
3. **No change to the desk's rails** — they are stricter and stay exactly as bound.
4. **No support from the firm**: failures are ours → the fail-closed design (disable on ambiguity) is the correct posture.

## 6. Honest limits / what is NOT covered
- Pages can change; the quotes are pinned by page date + read date above. **G0-C4 — a fresh re-read of all four sources is a
  required step again before `enable=1` (G4), not only at G1**; if any clause has changed, the verdict is re-opened.
- **G0-C2** — "no API" is an **inference from absence** (no API article/documentation on the pages reviewed + the on-disk
  27 Oct survey); it is not an explicit denial sentence. Treated as "no API surface documented".
- No support ticket was sent (outside send; not authorised; ASK D9 option 2 keeps the own-device design as the settled
  answer). If Cayleb wants written confirmation from the firm, that is his message, not ours.

## 7. Checker verdict and conditions (folded)
**G0-SIG-2026-11-01-TS-01 — PASS.** Independent Tier S checker, verdict file
`board/reports/team-c-g0-g1-checker-verdict.md` (sha256 `afc44f85…`): all four cited pages re-read and matched verbatim;
account facts confirmed from the firm's own Flex pages; the false "$50K EA ban" conflict confirmed killed (CFD division).
Conditions, all applied in this revision:
- **G0-C1** — Source-B splice marked and split into B(i) (VPN/VPS) and B(ii) (automation) in §2.
- **G0-C2** — Source B URL added; "no API" kept as an inference from absence (not a denial) in §2/§6.
- **G0-C3** — the exact CFD/MetaTrader quote pasted into §3 with its URL and page date.
- **G0-C4** — fresh re-read of all four sources required before G4 (§6).
- **G0-C5** — G0 does not close G1: the design must be revised to v3 and re-signed (checker's G1 HOLD
  `G1-HOLD-2026-11-01-TS-01`, fixes C1–C4 recorded in `board/reports/team-c-autonomous-path-design-v3.md` §0.1).
