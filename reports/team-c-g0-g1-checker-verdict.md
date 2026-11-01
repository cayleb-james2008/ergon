# TEAM C — G0 / G1 FRESH-CHECKER VERDICT (Tier S, veto held)

**Checker**: fresh-context, independent Tier S seat; did not write, stage, review or edit either artefact before this file.
**Seat**: G0 and G1 (two separate signatures below, each with its own conditions).
**Read-only discipline**: no order, no wallet, no credential, no send, no config/engine change, $0 spend. The only file written is this one.
**Window**: reads 2026-11-01T14:05Z–14:24Z (09:05–09:24 ET). Verdict written 2026-11-01T14:24Z / 09:24:10 ET.
**Objects of review**: `board/reports/team-c-firm-automation-verdict.md` (G0, mtime 08:59:03), `board/reports/team-c-autonomous-path-design-v2.md` (G1, mtime 08:59:49), base `team-c-autonomous-path-design.md`, `team-c-sunday-groundtruth-20261101.md`, `board/ASK.md` D14, `~/Downloads/12-futures-desk-monday-and-strategy.md`.
**Method**: every firm claim re-fetched from `helpfutures.fundednext.com` / `help.fundednext.com` over HTTPS and matched word-for-word; every design claim checked against the base spec and against the live engine config (read-only). Where I could not reach a page, it is marked UNVERIFIED — nothing is silently accepted.

---

# PART 1 — G0 (automation-permitted ruling): **PASS**, with conditions

## 1.1 Are the quoted firm clauses real and current? (item 1)

All four cited sources were independently re-read today. **No quote required the VERIFIED-BY-QUOTE fallback** — every one was reproduced from the live page.

| # | Cited page | Page date | Re-read | Result |
|---|---|---|---|---|
| A | `…/articles/14298560-is-the-usage-of-automated-trading-systems-eas-and-bots-allowed-in-fundednext-futures` | **April 9, 2026** | 14:0xZ | **VERIFIED** — page text matches the G0 TL;DR block verbatim: "FundedNext Futures allows the use of automated trading systems, including Expert Advisors (EAs) and trading bots, during both the Challenge Phase and on FundedNext Accounts… no support is provided … not responsible for technical issues … strategies that exploit system vulnerabilities—such as latency abuse or order flooding—are strictly prohibited." |
| B | FAQ `…/articles/17225788-fundednext-futures-most-frequently-asked-questions` | "**Updated this week**" | 14:0xZ | **VERIFIED** — "**Yes.** You may use a VPS … or VPN … on both your Challenge Account and your FundedNext Account"; "we do not offer any support"; and all four bullets verbatim, including "Using an EA that incorporates third-party applications such as Telegram or WhatsApp is strictly prohibited." and "Each EA or bot is required to employ a distinct strategy, avoiding identical trades across any accounts within FundedNext." |
| C | `…/articles/17225732-what-trading-platforms-does-fundednext-futures-offer` | **September 30, 2026** | 14:0xZ | **VERIFIED** — "FundedNext Futures supports three trading platforms — Tradovate, NinjaTrader, and TradingView. Each connects directly to your FundedNext Futures Account…" |
| D | `…/articles/14298337-what-are-the-prohibited-trading-strategies-of-fundednext-futures` | **September 30, 2026** | 14:0xZ | **VERIFIED** — TL;DR reproduced word-for-word incl. "gapped or illiquid markets" and "trading within **2% of CME price limits**"; and the bracket sentence verbatim: "Exploiting the lack of slippage or using tight brackets to benefit from favorable trade executions is prohibited." |

The maker's "two are dated 2026" is true but **understated — three are** (A 9 Apr 2026, C 30 Sep 2026, D 30 Sep 2026), plus B dated "this week". Source C's ellipsis is honest (it stands for "…, so you can trade on the platform that best matches your style…").

**Two citation defects (must be fixed in the file, do not change the verdict):**
1. **Splice not marked.** Source B's blockquote joins two *different* FAQ answers — the VPN/VPS answer ("**Yes.** You may use a VPS…") and the EA answer ("At FundedNext Futures, you are allowed to use automated trading systems…") — into one continuous quote. Both halves are real and on the same page, but the join is silent. Mark it (e.g. `[separate FAQ answer]`) so a reader cannot think the firm wrote them as one sentence.
2. **Source B has no URL.** Add `https://helpfutures.fundednext.com/en/articles/17225788-fundednext-futures-most-frequently-asked-questions`.

## 1.2 Conditions list and account facts (item 2) — each confirmed or marked

| Claim in G0 §3/§4 | Independent source read today | Status |
|---|---|---|
| Self-contained bot: no third-party app (Telegram/WhatsApp) inside the EA | FAQ (B), bullet 3, verbatim | **CONFIRMED** |
| Distinct strategy per bot, no identical trades across accounts | FAQ (B), bullet 2, verbatim | **CONFIRMED** |
| No latency abuse / order flooding | Source A + FAQ (B) + Source D TL;DR | **CONFIRMED** |
| Prohibited list (platform-error, account sharing/rolling, multi-order spam, slow-feed abuse, spoofing, layering, grid, wash, latency arb., reverse hedging, correlated hedging, gapped/illiquid, 2% of CME limit) | Source D TL;DR, verbatim; page also carries separate *Simulated-Execution Exploitation* and *Tick Scalping* sections, which G0 §4 does cover | **CONFIRMED** |
| 40% consistency **during the Flex Challenge**; daily threshold 40% × $2,500 = **$1,000**; **recalculation, not a breach** | Flex Challenge All Rules (`…/17229829`): "Profit Target $2,500"; formula "40% × Profit Target = Daily Profit Threshold"; "**Daily threshold: 40% × $2,500 = $1,000**"; "If you earn $1,500 in a day: $1,500 ÷ 0.40 = $3,750. You would now need $3,750 total to pass, instead of $2,500." | **CONFIRMED — exactly as G0 states it** |
| **No firm-side daily loss limit** on Flex (Challenge *and* funded) | Flex Challenge: "**No.** The Flex Challenge has no daily loss limit at any stage." Flex FundedNext: "No. There is no daily loss limit on the Flex FundedNext Account." Table in `…/17230292` lists **Flex = None**. | **CONFIRMED** |
| MLL **$1,500**, trailing EOD, locks at **$50,100**; floating breach is instant | Flex Challenge table + `…/17230292`: "For Flex … locks at initial balance + $100"; "Flex: $50,100"; "If your floating loss reaches the MLL during a live trade, your account is breached instantly." | **CONFIRMED** |
| All positions closed **3:10 PM CT = 4:10 PM ET**; auto-closed; resumes 5:00 PM CT | Trading Hours (`…/17225754`, 30 Sep 2026) + FAQ: "close all positions by **3:10 PM Central Time (CT)**… automatically closed"; "Trading resumes daily at **5:00 PM CT**". CT is ET−1 in November → 3:10 PM CT = 4:10 PM ET. | **CONFIRMED** |
| Contract limit $50K = **3 e-mini / 30 micro** | Flex Challenge + Flex FundedNext tables, verbatim | **CONFIRMED** |
| Micro-scalping = **10 s**; 30% warning; 40% = advance on hold (Challenge) / profit deducted (funded) | Micro-Scalp policy (`…/17230376`) + both Flex pages, verbatim | **CONFIRMED** |
| Funded account: no consistency rule, no daily loss limit | Flex FundedNext All Rules: "No. Unlike the Challenge Phase, there is no Consistency Rule on the Flex FundedNext Account"; "No. There is no daily loss limit…" | **CONFIRMED** |
| "No API surface offered" | **No page states an API exists.** `helpfutures.fundednext.com` site search for "API" returns "**We couldn't find any articles for: API**" (14:1xZ). The platforms page lists only the three platforms. | **CONFIRMED AS ABSENCE ONLY — not an explicit denial.** G0 §6 says so correctly; see condition G0-C2. |

## 1.3 The "false conflict killed" claim (item 3) — **CONFIRMED**

The EA ban at $50,000+ is the **CFD / MetaTrader** division, quoted from `help.fundednext.com/en/articles/8020763-is-ea-allowed-in-fundednext` (read 14:1xZ):
> "Yes, FundedNext allows the use of Expert Advisors (EAs) … on **MetaTrader 4 (MT4) and MetaTrader 5 (MT5)** platforms for accounts below $50,000. **Traders on account sizes of $50,000 and above must trade manually and may not use Expert Advisors, trading bots, or any automated tools** … both in Challenge and FundedNext accounts."
That is the CFD division (MT4/MT5; cTrader and Match-Trader banned outright), **not** FundedNext Futures. The Futures help centre (Source A/B) explicitly permits EAs/bots on this eval. The two articles even share boilerplate bullets ("distinct strategy", "Telegram or WhatsApp") — which is exactly how the false conflict arises. G0's disposal of it is right, and the quote above should be pasted into G0 §3 so it cannot resurface.

## 1.4 G0 verdict (item 4)

**PASS — automation is permitted on this eval, and the conditions are recorded and binding.** The quoted text supports the verdict; the "no API surface → CDP route" reading is fair *as an inference*; the §4 guard set is the right shape to hand to G1 (two of its rows are under-specified in G1 — see Part 2, findings F2 and F3); the §3 provenance correction is accurate and changes no desk rail.

**Residual limits I am signing over** (all stated by the maker, none hidden): pages can change and are pinned by date; "no API" is an absence-finding; no support ticket exists; the firm offers no support, so failures are ours.

### SIGNATURE — GATE G0
```
GATE:      G0 (automation permitted on this eval)
DECISION:  PASS, with conditions
SIGNED:    fresh Tier S checker (independent; maker never checker)
ID:        G0-SIG-2026-11-01-TS-01
SCOPE:     FundedNext Futures, Flex Challenge $50K → Flex FundedNext $50K only.
           Does NOT extend to Legacy / Rapid Pro / Rapid Daily / Direct, or to the CFD division.
VALID UNTIL: the earlier of (a) any change to the four cited pages, (b) G4 pre-arming, where a fresh
           re-read is mandatory.
CONDITIONS (binding):
 G0-C1  Mark the Source-B splice between the VPN/VPS answer and the EA answer; add Source B's URL.
 G0-C2  "No API" stays an inference. Record the 2026-11-01 negative site search ("We couldn't find any
        articles for: API") as the evidence class. Never phrase it as the firm's explicit denial, here or in G1.
 G0-C3  Paste the MT4/MT5 ≥$50K manual-trading quote (§1.3) into G0 §3 as the recorded reason the EA ban
        is not applicable.
 G0-C4  Fresh re-read of all four pages immediately before G4 — already promised in §6; now binding.
 G0-C5  Any new page that mentions "API", "bridge", "copy trading" or "third-party platform" for Futures
        voids this signature and returns G0 to UNVERIFIED until re-read.
```

---

# PART 2 — G1 (design gate v2): **HOLD**, not signed

## 2.1 Structural checks (item 1)

| Required element | Where | Status |
|---|---|---|
| CDP-only §4 **replaces** base §4 (REST path not designed against) | v2 §A, header line 4–6 | **PRESENT** — base §4 (Tradovate REST preferred) is correctly superseded; base §§1–3, 5–13 incorporate by reference |
| **G2.5 promoted to a required gate** | v2 §A bullet 4 + §D row | **PRESENT** — "G2.5 is a required gate, not a fallback"; gate table carries it with its own veto |
| Rails table R1–R18 | v2 §B (lines 30–49) | **PRESENT — 18 rows, all named refusals, none silently relaxed** |
| The six firm-derived refusals | R10 micro-scalp <10 s · R11 2%-of-CME · R12 gapped/illiquid · R13 one-side-only bracket · R14 phase-aware consistency · R16 self-contained-bot import assertion | **ALL SIX PRESENT** |
| Threat list | v2 §C, 10 threats, each mapped to a control | **PRESENT** |
| Kill path | v2 §E, 6 steps incl. fail-closed on unreadable page | **PRESENT** |
| Gate evidence table | v2 §D, G0→G4 with evidence + veto per gate | **PRESENT** |

Desk rails R1–R7 match the **live** engine config at `/home/cayleb/Work/projects/openbell` (HEAD `d555b50`, read-only): `mode=cards, contracts=1, max_trades_per_day=2, daily_loss_stop_usd=-400, daily_profit_stop_usd=900, cooldown_min_after_loss=15, flat_et=16:05, min_stop_pts=2.0, symbol=MESZ6, contract_id=4470322`. The live unit still describes itself as "never places orders" and `live/exec/` does not exist — the design-only claim is true.

## 2.2 Standing rails and gate integrity (item 2)

- **−$400 stays a desk rail.** Flex has no firm daily loss limit (verified, Part 1), so the attribution in R1 is correct and the rail is unchanged. **No contradiction.**
- **16:05 flat keeps its 5-minute margin.** Firm auto-close 3:10 PM CT = 4:10 PM ET; desk flat 16:05 ET → 5 minutes. **Confirmed** (R5, §C.10).
- **No existing gate weakened.** G2.5 was promoted (stronger); G2/G3/G4 evidence lines are unchanged and base §8 still binds via incorporation; the G1 veto list now also names G0. **No weakening found.**
- **$1,000 cap — ambiguity, not yet a contradiction (F4).** R14 makes the $1,000 threshold conditional on `phase=Challenge`, while the brief's verbatim constraint lists "$1,000 single-day firm cap" among rails that "stay", and G0 §3 calls it "the desk's rail … stays". In practice R2 (+$900) binds first, so nothing gets looser — but two documents now say different things about whether the cap exists when funded.

## 2.3 R11 — the Lead ruling is **NOT** reflected. This is the HOLD.

The ruling: *trading-lead owns the daily record; produced during the 09:05 guard, before the 09:15 attestations; a dated artefact naming the exchange page and timestamp; unset = NO TRADE; the bot refuses to trade without that day's record.*

What v2 actually says:
- Line 50–51: "R11 needs a **human-recorded** input … the **pre-market checklist** must record `cme_limit_price` for the session; **unset = no trade that day**."
- §H.4 (line 98): "**CME limit input (R11): who records `cme_limit_price` in the pre-market checklist each trading day** (fail closed if unset)."

**Gap, exactly:**
1. **Owner is still an open question** (§H.4) — the ruling closed it: trading-lead. A gate cannot pass with the ruling's answer re-opened as an H-item.
2. **No 09:05-guard production point and no ordering** — nothing ties the record to the guard script, and nothing requires it to exist *before* the 09:15 attestations, so attestation (d)/(e) could be given with the record still absent.
3. **No dated artefact** — "checklist" is not an artefact with a path, schema, date key or hash. There is no mechanism that makes *yesterday's* record unusable today; "unset" does not cover "stale but present".
4. **No exchange page + timestamp fields** — the ruling requires the record to name the exchange page it was read from and when. MES price limits are set from the prior EOD settlement and **change at 4:05 PM CT each session**, so provenance and timestamp are not decoration, they are the correctness condition.
5. **No attestation linkage** — the 09:15 block does not cite the record id, so the attestations can be signed over an unverified input.

This alone blocks G1. Closing it is a text change in v2 (no code): name the owner, the 09:05 production point, the artefact path/schema (`state/daily/<day>.cme_limit.json` with `{day, exchange_url, read_utc, ref_settlement, band_pct, upper, lower, author}`), the ordering before 09:15, the date-keyed refusal (record must match today's `day`, else `price_limit_unset`), and the attestation that quotes its id.

## 2.4 Additional findings that must be closed in the same revision

**F2 — R11's predicate shape cannot express the firm's rule (technical, blocking).** The firm's own explainer (`…/14298626`, read 14:1xZ) defines a **two-sided band** from a reference price:
> "For Equity Products such as NQ, MNQ, YM, MYM, ES, and MES, the CME Price limit has been set to 7% … **Upper Bound: Reference Price × (1 + 0.07 − 0.02); Lower Bound: Reference Price × (1 − 0.07 + 0.02)** … The price Limits are **updated at 4:05 PM CT** after each trading session … Overnight price Limits differ from the price Limits used during business hours."
A single scalar `cme_limit_price` (v2 line 42) covers one side and one regime only. A limit-down session would pass R11. Correct shape: config `cme_ref_settlement` + `price_limit_band_pct` (0.07) + the overnight/RTH regime flag → compute both bounds and refuse outside them; still fail closed if the day's record is absent.

**F3 — a G0-required guard is unrepresented (blocking, by G1's own veto clause "any rail unrepresented").** G0 §4 row "Spoofing / layering / grid / wash" requires: "single intent per decision; no re-placement without a new decision; **refuse if cancel-rate > policy**". R1–R18 has no cancel/replace-rate predicate; R18 covers only unknown ack/timeout/page mismatch. The order-flooding rule is the one the firm states twice (Source A and FAQ bullet 4), so the missing predicate matters. Add e.g. **R19**: max 1 cancel/replace per order and ≤N cancels per session (config), breach → `cancel_rate` refusal + disable.

**F4 — $1,000 phase wording** (see 2.2): state whether it survives as an unconditional desk rail when funded (recommended: yes, keep the brief's verbatim rail; note R2 binds first), or amend G0 §3's "stays" sentence. Two documents may not disagree.

**F5 — stale REST-era credential text still binds by reference.** v2 §A says the executor "never signs in, never enters a credential, never handles a 2FA prompt" — correct. But base §5 (`venue.py` reads the key "from the same place the watcher reads account data"), base §12 Q2 ("who creates the key pair") and base §13 ("the key lives outside the repo (Cayleb paste, write-only)") are incorporated unchanged by v2's header. G1's own veto list says "**any credential in the plan**". Strike those three passages in v2.1 and add the CDP-specific prohibition that actually matters: no cookie/`localStorage`/token read, no CDP `Network.getCookies`, no profile copy; the executor acts on the page as a human would.

**F6 — reverse/correlated hedging has no assertion.** Source D prohibits reverse hedging and hedging with correlated instruments. R6+R13 make it structurally impossible (one instrument, one side per intent), and base §11 puts multi-instrument out of scope, but no rail *refuses* a state where an opposite or correlated position exists. One line in R13 ("refuse if any open position exists on the opposite side of MES or in any correlated index contract; confirm flat before every intent") closes it.

**F7 — wording, not substance:** §A's "On FundedNext Flex there is no API" is stronger than the evidence; use "no API surface is offered or documented (verified by absence, 2026-11-01)". G0 §6 already says this; §A should not contradict it.

## 2.5 G1 verdict (items 3–4) — **HOLD**

The design is structurally sound — CDP-only replacement, G2.5 promoted, R1–R18 complete with all six firm refusals, threats, kill path and gate evidence all present, and no standing rail weakened. But the R11 ruling is not reflected (owner re-opened as an open question, no 09:05 production point, no dated artefact, no page/timestamp provenance, no attestation link), and R11's predicate cannot express a two-sided, regime-dependent limit; one G0-required guard (cancel-rate) is missing. **HOLD, not FAIL**: every gap is a text/predicate amendment that closes without redesign, and nothing found so far makes the path unsafe once they are in.

### SIGNATURE — GATE G1
```
GATE:      G1 (design gate, v2 CDP-only)
DECISION:  HOLD — design NOT approved. No build may start against it as written.
SIGNED:    fresh Tier S checker (independent; maker never checker)
ID:        G1-HOLD-2026-11-01-TS-01
WHY:       R11 governance (owner/09:05 artefact/provenance/attestation) absent; R11 predicate shape
           cannot cover both bounds and both limit regimes; cancel-rate guard from G0 §4 missing.
RELEASE CONDITIONS — all five, then re-submit for re-check (same seat may re-sign; a new seat is not required):
 G1-C1  R11 record closed as ruled: trading-lead owns it; produced in the 09:05 guard; dated artefact
        with path+schema naming the exchange page URL and read timestamp; today-only (stale = NO TRADE);
        bot refuses `price_limit_unset` without it; 09:15 attestation quotes the artefact id. §H.4 deleted.
 G1-C2  R11 predicate replaced with reference-settlement + 7% band → both bounds, overnight/RTH regime,
        fail closed when absent. Names the MES reference source and its 4:05 PM CT update.
 G1-C3  R19 cancel/replace-rate refusal added (with the config key and the refusal name), mapped in §C.
 G1-C4  Stale credential passages in base §§5, 12-Q2, 13 struck; CDP prohibition (no cookie/token/profile
        read) added; §A "no API" hedged to "no API surface documented (absence)".
 G1-C5  $1,000 phase semantics stated once, consistently with G0 §3 and the brief; reverse/correlated
        hedging assertion added to R13.
```

---

# PART 3 — What this checker did and did not do

- **Did**: re-read 10 firm pages (4 cited + 6 corroborating) over HTTPS; matched every quoted clause; re-read both design documents and the base spec line by line; checked desk rails against the live config and unit (read-only); verified the false-conflict article; site-searched the futures help centre for "API".
- **Did not**: write code, touch config, call the engine, place or cancel anything, read a credential, open the trading tab, message the firm, or edit any file other than this verdict.
- **Not proven, and not claimed**: that a Tradovate API entitlement truly does not exist (absence of documentation only); that the CDP ticket DOM will be stable (that is G2.5's job); that Monday's session will trade (Monday remains `cards`, per Lead X1).
- **Validity**: G0 holds until any cited page changes or G4, whichever is first. G1-HOLD is current until v2 is revised; a revised file is a new object and must be presented for re-check — this signature does not travel to it.

---

# PART 4 — G1 RE-CHECK AGAINST DESIGN **v3** (same seat, new file, veto held)

**Seat**: independent Tier S checker; same seat as Parts 1–2; did not write v2, v3 or any part of the R11 mechanism. **Read-only**: no order, no wallet, no credential, no send, no config/engine edit, $0. The only file written is this addendum. **Window**: reads 2026-11-01T14:37Z–14:52Z (09:37–09:52 ET); verdict 14:52Z.
**Objects (sha256 computed this window)**:
| Object | sha256 |
|---|---|
| `team-c-autonomous-path-design-v3.md` (the thing to sign) | `978232a3de3083a8a3aa0e3c0c56b713255c74554b7a2cdf618f6ec8a7956b51` |
| `team-c-autonomous-path-design-v2.md` (diff context) | `8b790e12597230158f56fd2420510a5a091224330e56ea33f8d14c56ccddaa42` |
| this file **before** the addendum (cited by v3 header L5–6) | `afc44f85a57f0dea41b137ad69f0149d1e27f81d10abe7db6b3ef621fbdbee47` — **matches** |
| `team-c-firm-automation-verdict.md` (G0 conditions folded) | `dc7028f6e28224e68b85c61f9c5014afd3e9620c3b8833108d3f2cab1fe511ad` |
| `cme-limit-record/fetch-cme-limits.sh` | `ddbcd7903de7bd4d017ca924d9ca3a16e098f0ebf3f885b94b55cd15e9218fac` |
| `cme-limit-record/README.md` | `f4bc0fc15d90db5bfa3fe8fa111c89a95173ecff7dcfbc78d4085cfe7d4074df` |
| `cme-limit-record/cme_limits-2026-11-02.json` | `57971524c7491c9a497e1607d2ae009acbf45da5f1e7484096a63e7c35ae8a23` |

**Method**: v3 read line by line; v2→v3 diffed; every §0.1 location checked against the sentence it claims to fix; §B.1 read against the generator, the README and the sidecar on disk; **both CME pages independently re-fetched today** over the reader route (HTTP 200) and the quoted rule, the ES Dec-2026 row, `As of Trade Date` and `Last Updated` matched; a direct `www.cmegroup.com` fetch returned **HTTP 403** (the design's transport claim is true); live engine config re-read at HEAD `d555b50` for R1–R7. I did **not** run the generator (it writes files) — the validation record and sidecar on disk are its behaviour evidence.

## 4.1 C1 — R11 governance: **MET in substance, one element false**

| Required by C1 | v3 text | Status |
|---|---|---|
| owner = trading-lead | L78: "**Owner: trading-lead** (Lead ruling 2026-11-01)." | **PRESENT** |
| produced in the 09:05 guard, **before** the 09:15 attestations | L78: "**Produced during the 09:05 session guard, BEFORE the 09:15 attestations.**"; L79 "does **not** change runbook v6, its five attestations or the P0.1 signature" | **PRESENT** |
| path | L80–82: `board/probes/team-c/<day>/cme-limit-record.md` + `…/cme_limits-<day>.json`, Monday dir `monday-2026-11-02/`; generator named | **PRESENT** |
| schema | L102–106 declares `refuse_bands:{upper,lower,upper13,lower13,upper20,lower20}` | **FALSE vs the artefact — G-2a** (see 4.6) |
| exchange page + read timestamp, not a feed | L83–90 quotes both pages; L89–90 "**`Ref` is the prior session's exchange-published EOD settlement/fixing reference — never a desk-feed price**"; L116–117 names the transport + page URL + read timestamp | **PRESENT — independently re-verified** (4.2) |
| yesterday's value unusable | L99–101 "the sidecar is **day-keyed**… `day` == the trade date… **Yesterday's record is unusable, never reused**" | **PRESENT** |
| attestation linkage | L110–115: `guard.md` carries `R11: SET <relative path> | Ref=…@… | bands up=[lo,hi] down=[lo,hi] | read_at=<utc>` or `R11: UNSET <reason>`; 16:20 check quotes it | **PRESENT** — `guard.md` is the real 09:05 evidence file (`monday-futures-runbook.md:32`, `team-c-monday-packet-v7.md:78`), and packet v7 adds R11 as 09:05 item 6, i.e. before the 09:15 preflight |
| §H.4 open question deleted | L163–169: §H = basis / demo / go / **G1 re-sign**; no "who records" item anywhere | **DELETED** |

## 4.2 C2 — two-sided, regime-aware predicate: **MET (and factually verified)**

- L92–94: "refuse if `Ref×(1+0.07−0.02) ≤ price ≤ Ref×(1+0.07)` (upper 2% band), or … `Ref×(1−0.07) ≤ price ≤ Ref×(1−0.07+0.02)` (lower 2% band)" — **two-sided**, and L91 says why ("a scalar would pass a limit-down session").
- L95: "Limits are re-set after the **4:05 PM CT** settlement publication; the overnight and RTH limit sets differ"; L96–97: day 7/13/20% 08:30–14:25 CT · 14:25–15:00 CT 20% only · overnight 7% 17:00–08:30 and 15:00–16:00 CT — converted correctly from the exchange rule.
- L107–109: fail-closed — missing/undated/stale, page unreachable, `Ref` unresolved, any band unresolved → `UNSET` → NO TRADE.
- Arithmetic re-computed by me from `Ref=7614.06`: 1.07→8147.0442→**8147.04**; 1.05→7994.763→**7994.76**; 0.93→7081.0758→**7081.08**; 0.95→7233.357→**7233.36**; all six 13%/20% bands in the sidecar also recompute correctly.
- **Independent CME read (2026-11-01T14:50Z, reader route, HTTP 200)** — the design's quotes are verbatim, including: *"7%, 13%, and 20% price limits are applied to the futures fixing price and are effective from 8:30 a.m. CT – 2:25 p.m. CT…"*; *"7% up-and-down limits are effective 5:00 p.m. - 8:30 a.m. CT…"*; the page carries **`As of Trade Date: 11/02/2026`** and **`Last Updated: 10/30/2026`**; and the fixing row exists exactly as recorded — `| E-mini S&P 500 Futures | 4 PM NYC | ES | December 2026 | 7614.06 | 30 Oct 2026 03:00:05 PM CT |`. The page also defines the fixing price as "the VWAP … during the 30 seconds of trading from 2:59:30 p.m. – 3:00:00 p.m. CT", which is what `Ref` claims to be. `Ref` and the bands are **not** desk-feed-derived.

## 4.3 C3 — cancel/replace row: **MET except one named field**

- L75 R19: "≤1 replace per intent and any replacement needs a **fresh decision ref**; ≤3 cancel/replace events per rolling 10 min; cancel-before-new; one order in flight" → `cancel_rate` / `replace_without_decision`.
- Mapped: §C.8 (L127–128) and §C.12 (L132); dry-run table widened to **R1–R19** (L139, L154) and the R19 cases named explicitly (L155).
- **Missing**: G1-C3 required "the config key and the refusal name". Refusal names are present; **no config key is named** — the thresholds are literals, unlike R14's `daily_consistency_cap_usd` (L70). **G-3.**

## 4.4 C4 — base overrides, $1,000, "no API", R13: **MET except the CDP prohibition**

| C4 element | v3 text | Status |
|---|---|---|
| base §5 struck for CDP | L39–41: `venue.py` "the **only** module allowed to drive CDP (attach/navigate/evaluate/dispatch in the tab); an AST test asserts no other module navigates, evaluates or opens a socket" — base sentence correctly quoted (`design.md:54`) | **REPLACED** |
| base §12-Q2 key pair struck | L42–43: "**STRUCK**. There is no API surface documented and no key pair" (base `:117`) | **STRUCK** |
| base §13 credential bullet struck | L44–46: "no key exists on this route; the executor reads no secret… G2 asserts (AST/static) that no secret file, env var or credential path is read" (base `:130–131`) | **STRUCK/REPLACED** |
| $1,000 phase-gated, consistent with R14 | L47–50 ("on the Flex Challenge… exceeding it **recalculates**… not a breach"; "on a Flex FundedNext Account there is **no consistency rule and no daily loss limit**"; "the desk's $1,000 cap rail remains in force in **both** phases as a **desk rail**") vs L70 (identical semantics) | **CONSISTENT** with each other, with G0 §3 ("The desk's rail (treat it as a cap) is stricter and stays", `:72`) and with the brief's verbatim rail (`12-futures-desk…:105–106`) |
| "no API surface documented" | L22, L6, and the G0 veto wording at L137 ("stated as more than an inference") | **HEDGED** |
| R13 reverse/correlated hedging | L69: "refuse if any other position exists in the account or an offsetting/correlated position (ES/MNQ/NQ/YM, same or correlated underlying) would be live" | **PRESENT** |
| **CDP prohibition (no cookie/token/profile read)** | **absent** — L28–30 says the executor "never signs in, never enters a credential" and "No key material… is read"; L44–46 enumerates "no secret file, env var or credential path". No sentence forbids reading the **session cookie/token**, `document.cookie`/localStorage, CDP `Network.getCookies`, or a profile copy — although L39–40 grants `venue.py` CDP **`evaluate`** | **MISSING — G-1, decisive** |

## 4.5 §0.1 truthfulness (each claimed location checked)

All five rows point to a place that **does** carry the claimed change: C1→L78/L80–115/§H; C2→L67+L91–98; C3→L75; C4→L37–50+R14+R13+§A. Two caveats, both material to the signature: the C1 row's "path/schema" claim overstates (schema is wrong, G-2a) and the C4 row's "§A.1 (base overrides)" is incomplete (the CDP prohibition named in G1-C4 is not there, G-1). The mapping is honest about *where* work was done; it is not yet accurate about *completeness*.

## 4.6 R11 mechanism vs design (the "match" check)

- **Day-key: MATCHES.** `fetch-cme-limits.sh:48` refuses when the price-limits page's trade date ≠ `--day`; the first `As of Trade Date` line on the live page belongs to the **Equity Limits** section (verified today), so the anchor is the right one; proven fail-closed run `--day 2026-11-03` → UNSET exit 2 (README:57–58; `validation-cme-limit-20261102.md`). The 2026-11-01 validation sidecar is correctly **not** the Monday record: `read_at=2026-11-01T14:33:29Z` sits outside Monday's guard window and its path is not `monday-2026-11-02/`, so it cannot satisfy the executor's window check.
- **Fail-closed: MATCHES.** Four independent UNSET paths (script 44–49, 66–68); an UNSET sidecar is still written with `status:"UNSET"` (124, 142–143); exit 2 (147). Design L107–109 says the same.
- **G-2a — schema does not match the artefact.** L102–105 declares `refuse_bands:{upper,lower,upper13,lower13,upper20,lower20}`; README:40–41 and the sidecar on disk use `refuse_bands:{b7:{upper,lower}, b13:{…}, b20:{…}}` **plus** a `formula` key the design omits. The executor "reads only" this file (L99, README:42), so the interface the design specifies is not the interface that exists — a G2 executor built from the design would not find the keys.
- **G-2b — "a different contract … = `UNSET`" (L100–101) is not implemented.** The generator hard-codes `"contract":"MESZ6"` (116, 134) and never reads column 5 of the fixing row (`December 2026`). On a roll the script would stamp MESZ6 onto a March-2027 price. (The row *is* selected correctly today by `E-mini S&P 500 Futures | 4 PM NYC`.)
- **G-2c — "mismatched exchange read timestamp = `UNSET`" (L101) is not implemented.** `ref_ts` is recorded but never validated as the last session print before `day`, and `exchange_read_at` is set to the **run clock** (121), not to an exchange timestamp — so the field the schema promises is misnamed.
- Non-blocking: the day-key check depends on `grep -m1` page order (correct today, fragile); the `read_at` guard-window test is executor-side (correctly G2's job, not the script's).

## 4.7 Standing rails and gates — **nothing weakened**

R1–R7 are byte-identical to v2 and still match the live config (HEAD `d555b50`): `mode=cards`, `MESZ6`/`4470322`, 1 contract, `max_trades_per_day=2`, `-400`/`+900`, 15-min cooldown, `flat_et=16:05`, `min_stop_pts=2.0`. R8–R10, R12, R15–R18 are unchanged. G2.5 remains a **required** gate (L33, L140); G4 is strengthened with `G0-C4` (L142); G2's evidence is widened to R1–R19 (L139); the G1 veto list is strengthened (L138). The only semantic change, R14's funded branch, keeps the desk cap (stricter-or-equal), so nothing gets looser.

## 4.8 Exact remaining gaps

- **G-1 (decisive, C4).** No prohibition on reading the live session credential on a route that grants CDP `evaluate`. G1-C4 required "no cookie/token/profile read" verbatim; "reads no secret / no secret file, env var or credential path" does not name the in-page credential, and G2's AST proof (L46, L139) therefore cannot test it. **Fix (text only, one sentence):** in §A.1.4 add — "the executor never reads the session credential: no `document.cookie`/localStorage/sessionStorage read, no CDP `Network.getCookies`/`Storage.getCookies`, no token extraction, no profile copy; it acts only through the ticket controls. G2 asserts a CDP-command allowlist that excludes these calls." (Fold in §H.2's "who holds the demo keys" while you are there.)
- **G-2a (required, C1).** The declared schema-2 shape (L102–105) contradicts README:40–41 and the on-disk sidecar. **Fix:** replace it with the keys actually produced (`refuse_bands.b7|b13|b20`, `formula`) or declare `README §Output contract` normative and quote the sidecar's sha256.
- **G-2b (required, C1/C2 honesty).** Assert the fixing row's contract cell = `December 2026` and refuse otherwise, or strike "a different contract … = `UNSET`" (L100–101).
- **G-2c (required, C1).** Validate `ref_ts` = the last session print before `day` and set `exchange_read_at` to the exchange timestamp actually quoted, not the run clock; or strike the claim.
- **G-3 (required by G1-C3's letter).** Name the R19 config keys (e.g. `max_replace_per_intent: 1`, `max_cancel_replace_per_10min: 3`) as R14 names `daily_consistency_cap_usd`.
- **Non-blocking notes for G2/G4.** (N-1) The R11 step lives in the *packet* (v7 §09:05 item 6), while P0.1 signs **runbook v6**, which carries no R11 item — a future autonomous day needs the step in the signed runbook, not only in a design plus a packet. (N-2) `board/reports/team-c-monday-packet-v7.md:79` cites the validation as 14:18Z; the record on disk is 14:33:29Z — cosmetic. (N-3) The `r.jina.ai` transport is a third-party proxy of the exchange page; it is disclosed in the record and the design, so it is provenance-with-disclosure, not a feed derivation.

## 4.9 Verdict

**HOLD.** Not FAIL: the substance of C1 and C2 is now present **and factually verified against the live CME pages**, R19 exists and is mapped, three of the four C4 overrides are correctly struck, the $1,000 wording is consistent everywhere, and no rail or gate is weakened. What blocks the signature is narrow and text-only: the one C4 control the previous HOLD named (the CDP credential-read prohibition) is still absent, the artefact schema the design declares is not the schema that exists, two R11 staleness/identity promises are unbacked, and R19 lacks the config key the release condition asked for. Fix the five items above and re-submit; this seat may re-sign; no new seat is needed.

```
GATE:      G1 (design gate, v3 CDP-only) — RE-CHECK OF THE v2 HOLD
DECISION:  HOLD — design v3 NOT approved. No build may start against it as written.
SIGNED:    fresh Tier S checker (independent; maker never checker) — same seat as G0-SIG-2026-11-01-TS-01 / G1-HOLD-2026-11-01-TS-01
ID:        G1-HOLD-2026-11-01-TS-02   (G1-SIG-2026-11-01-TS-02 is NOT issued; it is reserved for the re-sign)
WHY:       C4's CDP credential-read prohibition absent; declared schema-2 shape contradicts README + sidecar;
           "different contract"/"mismatched exchange timestamp" = UNSET unbacked; R19 has no config key.
CLOSED SINCE THE LAST HOLD (do not re-open): C1 governance + attestation linkage (§B.1, §H); C2 two-sided,
           regime-aware, fail-closed predicate with verified Ref; C3 row + §C mapping + R1–R19 dry-run table;
           C4's three base strikes, $1,000 phase wording, "no API surface documented", R13 hedging assertion.
RELEASE:   G-1, G-2a, G-2b, G-2c, G-3 (§4.8) — text-only; then re-submit v3.1 to this seat.
VALID:     this HOLD is current for v3 sha256 978232a3…; any edit makes a new object and a new re-check.
```

---

# PART 5 — G1 RE-CHECK AGAINST DESIGN **v4** (fresh independent seat, veto held)

**Seat**: fresh-context independent Tier S checker. Did **not** write, stage, review or edit v4, the generator, the README, the sidecar, the validation record or Parts 1–4. Reports to the Lead only. **Read-only**: no order, no wallet, no credential, no send, no config/engine edit, $0 spend. The only file written is this addendum.
**Window**: reads 2026-11-01T15:06Z–15:23Z (10:06–10:23 ET); verdict 15:23Z.
**Scope**: **only** the four items of `G1-HOLD-2026-11-01-TS-02` (G-1, G-2a, G-2b/c, G-3) plus the three confirmations (nothing weakened, no credential text, §0.x mapping truthful). Settled structure — C1 governance, C2 predicate, R1–R10/R12/R15–R18, G2.5, threat list, kill path — is **not** reopened.

**Objects (sha256 computed this window)**:

| Object | sha256 |
|---|---|
| `team-c-autonomous-path-design-v4.md` (the thing to sign) | `ea0ec96e1402fd4427706371428fa9cd3316d9c692455a98f87f1fca0e883c1e` |
| `team-c-autonomous-path-design-v3.md` (diff context) | `978232a3de3083a8a3aa0e3c0c56b713255c74554b7a2cdf618f6ec8a7956b51` — **matches Part 4** |
| this file **before** this addendum (carries G1-HOLD-…-TS-02) | `ff9a78b4552a1f70e51bcf764c9055efe9f630ce5bc026e3ad01b763ac54f71d` — **matches the prefix cited to me** |
| `team-c-firm-automation-verdict.md` (G0, not reopened) | `dc7028f6e28224e68b85c61f9c5014afd3e9620c3b8833108d3f2cab1fe511ad` — unchanged since Part 4 |
| `cme-limit-record/README.md` (rev3) | `a87824b84f49e688fbb38709944c3d57f37a4f773068216168ba830fcd9af78e` (was `f4bc0fc1…`) |
| `cme-limit-record/fetch-cme-limits.sh` (rev3) | `8b3050fa61d6704d05e9ab28475de2cf385b01b927d61e68ee01cfe46ed4dcdf` (was `ddbcd790…`) |
| `cme-limit-record/cme_limits-2026-11-02.json` (regenerated) | `6495bdf41900ccf7b7103e0a5d31d5a9f1ed28db179ef831ec148c2d55e3ff59` (was `57971524…`) |
| `cme-limit-record/validation-cme-limit-20261102.md` | `9752fe539af4db69a656f416c62418419a14f4d360d8241ba544932cc1edc96a` |

**Method**: v4 read line by line; v3→v4 diffed; every §0.1/§0.2 location checked against the sentence it claims; §B.1 checked against the generator, the README, the on-disk sidecar and the validation record; all twelve bands recomputed from `Ref=7614.06`; live engine config re-read at HEAD `d555b50`; credential keywords searched. I did **not** run the generator (it writes files) and did **not** re-fetch the CME pages this round (Part 4's independent read at 14:50Z stands; this round tests whether the design's claims match the script). The on-disk artefacts are the behaviour evidence.

## 5.1 G-1 (decisive, C4) — explicit prohibition on session cookie/token/profile reads: **MET**

The grant that made this decisive is unchanged in kind and now fenced: v4:42–44 grants `venue.py` the only CDP role — "`venue.py` is the **only** module allowed to drive CDP (attach/navigate/evaluate/dispatch in the tab)" — and §A.2 is written against exactly that privilege.

- **The prohibition is explicit, general and exceptionless** — v4:55–58: "**Holding CDP `evaluate` on that tab gives technical access to session material; this section forbids reading it, without exception. `venue.py` and every other module must never read, copy, log, persist, cache, exfiltrate or transmit:**".
- **Enumerated, not generic** (v4:59–65): 1 cookies — "`document.cookie`, `Set-Cookie` values, any cookie API"; 2 storage — "`localStorage`, `sessionStorage`, IndexedDB, Cache Storage, service-worker registrations/caches"; 3 credential/identity APIs — "`navigator.credentials`, password managers, autofill data, WebAuthn material"; 4 tokens/session identifiers — "auth/session/bearer/CSRF/XSRF/anti-bot tokens, nonces, request signatures, account IDs beyond the declared order-ticket/positions fields, or any value whose purpose is authentication rather than order entry"; 5 profile/PII — "browser profile files on disk (e.g. `Cookies`, `Login Data`, `Local State`, token caches, `Preferences`, `History`), the OS keyring, or any environment variable / secret file".
- **The evaluate surface is bounded too** (v4:66–69): only the allowlisted set — ticket/positions DOM text/attributes, ticket input writes, declared control clicks — "Every evaluate call must match one of the declared expression templates; anything not matched is refused."
- **Testable by G2, as the HOLD required** (v4:71–79): static/AST assertion with the literal deny-list "`document.cookie`, `localStorage`, `sessionStorage`, `indexedDB`, `navigator.credentials`, `Authorization`, `Bearer`, `csrf`/`xsrf`, `token`, `Login Data`, `Local State`, `Preferences`, `Cookies`, or any keyring/secret path" (72–74); a runtime canary page (75–76); a filesystem assertion on the open/read set (77–78); and fail-closed kill — "any attempted read or any unmatched evaluate template → **disable + alert**" (79).
- **Carried into the gates**: threat 13 (v4:174–175), the G1 veto "any credential/session-material access not prohibited" (v4:181) and the G2 evidence/veto (v4:182, 199–200).

This is the explicit in-page credential prohibition G1-C4 asked for — not an extension of "no secret file/env var" (which survives only as item 5's tail). **MET.**

*Precision note (non-blocking, N-1):* the AST list names the literals `Cookies`, `document.cookie`, `localStorage` but does not spell the CDP commands `Network.getCookies` / `Storage.getCookies`; v4:59's "any cookie API" governs them semantically, and a case-insensitive match on `Cookies` reaches both command names. G2 should name them in the deny-list anyway.

## 5.2 G-2a (C1) — declared schema-2 field names vs the actual sidecar: **MET**

- v4:138–145 now declares: "`refuse_bands:{b7:{upper:[lo,hi],lower:[lo,hi]}, b13:{upper:[lo,hi],lower:[lo,hi]}, b20:{upper:[lo,hi],lower:[lo,hi]}}`", plus `formula`, `ref_ts_parsed`, `ref_age_days`, `checks:{trade_date_match, contract_month_match, ref_ts_prior_session_le_7d, pages_http_200}`, `limits:{up7,down7,up13,down13,up20,down20}`, `session`, `exchange_rule`, `exchange_read_at`, `read_at`, `owner`, `status` — the full 20-key shape.
- The README's schema block (README:50–55; the HOLD's "README:40–41" is now the *staleness* paragraph because the README was revised to rev3 — the schema moved, the names did not) carries exactly `refuse_bands:{b7:{upper,lower}, b13:{…}, b20:{…}}`, `formula`, `checks`, `limits`, `session`.
- The sidecar on disk matches key-for-key: `refuse_bands.b7` (sidecar:20–30), `b13` (31–40), `b20` (41–50), `formula` (52), `limits` (12–19), `session` (53–57), `checks` (58–63), and the generator writes the same dict (script:90–91, 148–158, 167–179).
- The band values are arithmetically correct: 7% upper `[7994.76, 8147.04]`, lower `[7081.08, 7233.36]`; 13% `[8451.61, 8603.89]` / `[6624.23, 6776.51]`; 20% `[8984.59, 9136.87]` / `[6091.25, 6243.53]` — all recomputed this window from 7614.06.

The field-name contradiction that blocked Part 4 is gone. **MET.**

*Two residual shape notes (non-blocking, N-2):* (a) the UNSET path writes a **reduced** object — `{schema,day,phase,status,reason,read_at,owner}` (script:161, 181–182) — not the declared full schema, and one UNSET path (a rejected `ref_ts`) reaches `int(ref_age)` with an empty string (script:152) and leaves the sidecar empty; (b) therefore the executor must gate on `status=="SET"` **first** (v4:146 says it does) and treat empty/reduced JSON as refusal, never as partial evidence.

## 5.3 G-2b/c — "different contract"/"mismatched exchange timestamp" claims backed by the script: **MET**

v4:126–137 now states exactly what is enforced, and the script implements each line:

| v4 claim | Script evidence |
|---|---|
| 1. both pages HTTP 200 via the named transport | `fetch-cme-limits.sh:35–39, 47` |
| 2. `As of Trade Date` == `--day` (MM/DD/YYYY) | `:42–46` (expected from `--day`), `:50` (first As-of line), `:57` (mismatch → reason) |
| 3. fixing row contract == `--ref-month` (default "December 2026" = MESZ6); **script does not discover the desk contract**; operator keeps the parameter in step; a roll changes the parameter | `:13` (default), `:59–62` (row prefix + month check → UNSET), `:151/171` (sidecar literal) |
| 4. `ref_ts` parses, is a **prior session**, ≤7 calendar days old | `:64–80` (parse `%d %b %Y %I:%M:%S %p`, `t.date() < day`, `0 < age <= 7`; UNPARSABLE/FAIL → reason) |
| 5. `exchange_read_at` = the time this run read the pages, same instant as `read_at`, "not a second clock and is not compared against one"; cross-day staleness by day-key + 09:05 `read_at` window | `:26` (READ_AT), `:158/178` (both fields = READ_AT); v4:134–137 |

- The v3 sentence "**a different contract … = `UNSET`**" is **gone**, replaced by the honest hard-code disclosure at v4:129–131 ("**The script does not discover the desk's contract**; the parameter is kept in step with live config by the operator, and a contract roll changes the parameter"). That is the "or the script must be fixed and the design say so" branch, truthfully stated.
- The v3 claim "**mismatched exchange read timestamp = `UNSET`**" is also gone; `exchange_read_at` is now honestly defined as the run's read time and the actual staleness rule (day-key + guard-window `read_at`) is named. `ref_ts` is genuinely validated now (the Part-4 defect was that it was never validated at all).
- Fail-closed is implemented: `STATUS=UNSET` on any reason (`:95`), exit 2 (`:185–186`); the validation record is a SET run at `2026-11-01T15:01:21Z` with `ref_ts_parsed 2026-10-30T15:00:05`, age 3 (validation:6, 23; sidecar:9–10, 65–66), and README:70–72 records the two UNSET runs (day-key, contract-month) as exit 2. I recomputed the timestamp check: 30 Oct 2026 15:00:05 → prior to 2 Nov, age 3, passes.

**MET** — the claims are now limited to what the script does, including the hard-code.

*Residual precision notes (non-blocking, part of S-2):* (i) the sidecar `"contract":"MESZ6"` is a literal (script:151, 171) while `--ref-month` is parameterized — on a roll the field would contradict `underlying_contract`/`ref_source`; the design does not claim it is derived, so G2/G4 must update or derive it before a roll. (ii) the `checks` booleans are literal `true` (script:156, 176), including on a levels-present UNSET record — they are not derived; gate on `status` only. (iii) "prior session" is enforced as "prior calendar date" (`:76`), not against a trading calendar; the ≤7-day bound and the exchange's own fixing row make this low-risk. (iv) the contract test is a substring of the whole row (`:61`), not the cell — equivalent today because the row is pinned by `E-mini S&P 500 Futures | 4 PM NYC` and no other cell can carry "December 2026". (v) robustness only: a malformed `--day` makes `ASOF_EXPECTED` empty and the day-case vacuous (`:42–46, 57`); the operator command is fixed (README:62) and the executor's `day == trade date` check catches it, but G2 should assert a well-formed `--day`.

## 5.4 G-3 (C3 letter) — R19 carries the config keys: **MET**

v4:105: "config `max_replaces_per_intent` (=1): ≤1 replace per intent and any replacement needs a **fresh decision ref**; config `max_cancel_replace_per_10min` (=3): ≤3 cancel/replace events per rolling 10 min; cancel-before-new; one order in flight. **Both keys asserted from config at G2 (like R14's `daily_consistency_cap_usd`)**." Refusal names are `cancel_rate` / `replace_without_decision`; mapped in §C.8/§C.12 and named in the §F refusal table (v4:196–198). This satisfies G1-C3's "the config key and the refusal name". **MET.**

*Note (S-3):* neither R19 key — nor R14's `daily_consistency_cap_usd` — exists in live config at HEAD `d555b50` (values read: `mode=cards`, `contracts=1`, `max_trades_per_day=2`, `daily_loss_stop_usd=-400`, `daily_profit_stop_usd=900`, `cooldown_min_after_loss=15`, `flat_et=16:05`, `min_stop_pts=2.0`, `symbol=MESZ6`, `contract_id=4470322`, `stale_s=60`). They are build-phase additions; G2's "asserted from config" runs after they are added. This is the expected sequence, not a design gap.

## 5.5 No weakening; no credential text; §0.x truthful

**Rails.** R1–R7 are byte-identical to v3 (v3→v4 diff touches only R11, R13–R14 wording, R18's added A.2 clause and R19), and they still match live config at HEAD `d555b50` (working tree clean for `live/config.json`; `git status --porcelain` empty): −400 / +900 / 2 trades / 15 min (900 s) / 16:05 ET / 1 MES `MESZ6` `4470322` / 2.0 pts; R8's 60 s matches `stale_s=60`. `params.min_stop=2.5` is stricter than R7's 2.0, not a conflict. The unit still describes itself as "never places orders" and `live/exec/` does not exist.

**Gates** (v4:181–185). G1's veto adds session-material access; G2's evidence and veto add §A.2's three assertions; G2.5 is still a required gate (v4:36–37, 183); G3 is unchanged; G4 still requires "**fresh re-read of all four firm pages**" (v4:185) — the `G0-C4` *label* was dropped from the row but the requirement is intact and G0-C4 still binds from the G0 signature. Nothing is loosened. (R13's "in the account" phrase was dropped at v4:99 — "refuse if any other position exists or an offsetting/correlated position … would be live" is broader, not looser; no action.)

**Credential text.** The three stale REST-era passages are struck/replaced: base §5 (`design.md:54`, quoted v4:42–44), base §12-Q2 (`design.md:117`, struck v4:45–46 with "no key pair; no credential is created, asked for or handled"), base §13 (`design.md:130–131`, struck/replaced by §A.2 at v4:47–48). Every remaining mention of key/credential/profile in v4 is a **prohibition** (v4:31–32, 45–48, 55–81, 166, 181–182, 199) or §H.2's demo-ownership question (v4:209, "no credential handled here"). Base §12-Q3 ("who owns those demo keys") survives by reference as an ownership question, not a handling plan — keep it that way at G3. **No credential-handling text remains.**

**§0.x mapping** — each row checked at the line it names:

| Row | Claimed location | Verified | Truthful |
|---|---|---|---|
| §0.1 G-1 | §A.2 | v4:55–81 (prohibition 55–69, enforcement 71–79, kill 79) | yes |
| §0.1 G-2a | §B.1 schema | v4:138–145; README:50–55; sidecar:20–52 | yes |
| §0.1 G-2b/c | §B.1 "what the script enforces" + script strengthened | v4:126–137; script:35–39, 42–46, 59–62, 64–80, 26/158/178 | yes |
| §0.1 G-3 | §B R19 | v4:105 | yes |
| §0.2 C1 | §B.1 | v4:108–158 | yes |
| §0.2 C2 | R11 + §B.1 | v4:97, 121–125 | yes |
| §0.2 C3 | R19 + §F | v4:105, 196–198 | yes |
| §0.2 C4 | §A.1 + R14 + §A + R13 | v4:42–48, 100, 25–28, 99 | yes |

*Provenance nit (N-6):* v4:5–6 cites the verdict file's sha as `afc44f85…`, which is the file **before** the TS-02 addendum; the version carrying `G1-HOLD-…-TS-02` (and this addendum now supersedes) is `ff9a78b4…`. Cosmetic — the G0 signature (Part 1) is unaffected — but the citation should name its revision.

## 5.6 Non-blocking notes handed to G2/G4 (not G1 conditions)

- **N-1** AST deny-list should also spell `Network.getCookies` / `Storage.getCookies`; "any cookie API" already governs.
- **N-2** sidecar UNSET shape is reduced/possibly empty; executor must gate on `status=="SET"` first and treat unparsable JSON as refusal.
- **N-3** generator's `contract` literal (`MESZ6`) must be updated or derived before any contract roll; `checks` booleans are literals, not derived.
- **N-4** "prior session" is a prior calendar date in code; acceptable with the ≤7-day bound — rename the `checks` key or add a session-calendar test at G2.
- **N-5** malformed `--day` makes the day-key case vacuous; G2 should assert a well-formed day (the executor's day equality already refuses the resulting record).
- **N-6** header sha (`afc44f85…`) is one revision behind the verdict file (`ff9a78b4…`); cite the revision.

## 5.7 Verdict

**PASS.** The four items of `G1-HOLD-2026-11-01-TS-02` are closed in text and, where the artefact is code, in the script: G-1 is an explicit, enumerated, gate-backed prohibition on session cookie/storage/token/profile reads with an AST deny-list, runtime canary, filesystem assertion and kill; G-2a's declared schema matches README, generator and sidecar key-for-key; G-2b/c's claims now match what the script enforces (contract-month check, `ref_ts` parse/prior/≤7 d, `exchange_read_at` honestly defined as the run's read time) and the hard-code is disclosed; G-3 names both R19 config keys. Nothing in v4 weakens a standing rail or gate; no credential-handling text remains; the §0.x mapping is truthful. The residual items are enforcement/robustness notes for G2 and G4, not open G1 gaps.

```
GATE:      G1 (design gate, v4 CDP-only) — RE-CHECK OF G1-HOLD-2026-11-01-TS-02
DECISION:  PASS — design v4 APPROVED as the text basis for G2. Build may start on G2 against this
           sha256 only; nothing is armed, no order, no spend, Monday stays cards-only.
SIGNED:    fresh-context independent Tier S checker (independent; maker never checker)
ID:        G1-SIG-2026-11-01-TS-03
SCOPE:     FundedNext Futures, Flex Challenge $50K → Flex FundedNext $50K; CDP ticket automation of
           trader.tradovate.com only. Does NOT authorize any live order, arming, credential or spend.
           G0 remains the automation-permission basis (G0-SIG-2026-11-01-TS-01, not reopened).
VALID:     v4 sha256 ea0ec96e… only; the earlier of (a) any edit to v4 (new object → new re-check),
           (b) any change to the four G0 pages, (c) G4. Not valid for v3 or earlier.
WHY:       G-1 explicit session-material prohibition + testable enforcement (§A.2); G-2a schema matches
           README/generator/sidecar; G-2b/c claims match the script and the hard-code is disclosed;
           G-3 R19 names both config keys. Nothing weakened; no credential text; §0.x truthful.
CLOSED (do not re-open): G-1, G-2a, G-2b, G-2c, G-3; and C1–C4 from the first HOLD (Part 4).
CONDITIONS (binding, carried to G2/G4):
 S-1  Implement §A.2 exactly: static/AST deny-list (incl. CDP cookie/storage commands), runtime canary,
      filesystem assertion; any session-material read or unmatched evaluate template → disable + alert.
      G2's veto stands unchanged.
 S-2  Executor gates on status=="SET" first, then day == trade date and read_at in the 09:05 guard
      window; it must not treat the sidecar's `checks` booleans (literal true, script:156,176) or the
      `contract` literal (script:151,171) as evidence; empty/reduced UNSET sidecar = refuse. Before a
      contract roll the generator's contract literal must be updated with --ref-month or derived from it.
 S-3  R19's keys (max_replaces_per_intent=1, max_cancel_replace_per_10min=3) and R14's
      daily_consistency_cap_usd must exist in config before G2 asserts them from config (none are in
      live config at HEAD d555b50).
 S-4  G4 keeps G0-C4's fresh re-read of all four firm pages (v4:185); the dropped G0-C4 label is cosmetic.
 S-5  Monday 2 Nov remains cards-only (Lead X1); this signature authorizes no arming, no order, no spend.
```
