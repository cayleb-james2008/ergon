# DESIGN AMENDMENT (text only, FOR NOX'S RULING) — G0 outcome: the interface the FundedNext clause forces

Prepared by trading-lead (Team C), 2026-10-31 19:2x ET. **Text only: nothing in `team-c-autonomous-path-design.md` has been
rewritten and no code exists.** The Lead's ruling: no build of the autonomous path until Nox rules; the basis is Nox's to
choose. This is the amendment text plus the comparison he asked for.

## A.1 — What G0 found (the fact the amendment rests on)
The eval is a **FundedNext Flex $50K**. The programme's own verified prop-firm survey
(`~/Work/projects/openbell/reports/PROP-FIRMS-2026-10-27.md` + `prop_firms_2026-10-27.json`; 61 rows from 8 verification
batches) records the FundedNext Flex rows as `"confidence": "official"`, read from `helpfutures.fundednext.com` price table
and Flex rules on **2026-10-27**:
> `"automation": "allowed"`, `"copy_across": "allowed"`, `"copy_within": "allowed"`,
> `"api": "own bots/EAs on Tradovate, NinjaTrader or TradingView (Tradovate accounts); VPS allowed; no API"`.

So: **own bots/EAs are permitted, but only through a platform login (Tradovate here) — there is no API on FundedNext.**
The design's preferred REST-API path is therefore unavailable on this account.

**Honesty limit (must travel with this amendment):** the clause was **not re-read tonight** — every web search and direct
fetch in this session timed out (30 s, repeated). The 2026-10-27 on-disk reading is the basis; G0 as written still requires
the clause quoted **with the date re-read** before any build. Re-reading is a network-dependent step, not a Cayleb act, and
must happen before G1.

## A.2 — Amendment to design §4: CDP ticket automation is the only autonomous path on this eval
Replace the §4 interface section for this account with:
> **On FundedNext Flex there is no API.** Own bots/EAs are allowed through the platform login (Tradovate), so the only
> autonomous execution path on this eval is **ticket automation of the already signed-in `trader.tradovate.com` tab over
> CDP 9222** — the same Chromium/route the passive feed and account watcher already use, but writing into the order ticket.
> The REST-API path is **not an option** on this account and must not be designed against. The CDP route is fragile by
> nature (DOM, tab navigation, page version changes), so **G2.5 becomes a required gate, not a fallback**: pinned selectors
> against a recorded page snapshot, a tab-navigation test, a guard that a send is possible only when the page is the
> expected order ticket, and screenshot evidence per step. Every other gate (G1 design, G2 dry-run, G3 demo, G4 pre-arming
> with Cayleb's written go) is unchanged, and the containment properties (fail-closed enable artefact, one process, one
> instrument, single-trader lock, flat-at-start, rails re-checked before every send, disable-on-ambiguity) are unchanged.

## A.3 — The supported alternative the clause allows: NinjaTrader / NinjaScript EA bridge (one paragraph)
The same clause permits own bots/EAs on **NinjaTrader**, which is a supported automation surface rather than DOM automation:
the eval's Tradovate account is connected inside NinjaTrader (a data subscription is normally required), and an ORB15
NinjaScript strategy runs inside the platform with native, event-driven order handling and platform-managed brackets. It
removes the browser from the execution path entirely (no tab, no selectors, no page-version risk) and gives a deterministic
API to test against. The cost is a **platform move**: NinjaTrader itself plus a market-data subscription (new spend → Cayleb),
porting the engine's ORB rules to NinjaScript and proving parity with the Python engine on the same bars, and deciding
whether the Python engine keeps the decision layer (emitting signals to the EA) or the EA owns the strategy. It also adds a
second order surface beside the existing Tradovate desk, so the one-trader rule and the reconciliation join (fills → cards →
decisions) must be rebuilt for the new path. It is the more supported route and the more expensive one.

## A.4 — Comparison (both allowed by the clause; neither is built)
| Axis | CDP ticket route (Tradovate tab) | NinjaScript EA bridge (NinjaTrader) |
|---|---|---|
| Basis in the clause | "own bots/EAs on Tradovate" | "own bots/EAs on NinjaTrader" |
| New spend | **none** (uses the existing session/route) | NinjaTrader licence + data subscription (Cayleb) |
| Fragility | high: DOM, tab navigation, page version | low: platform API, event-driven |
| Testability | G2.5 (pinned selectors, snapshots, per-step screenshots) + G2 dry-run | strategy tester + demo account; parity tests vs the Python engine |
| Credentials | existing signed-in session only; fresh session/2FA = stop for Cayleb | platform login + data credentials (Cayleb's paste) |
| Stack change | none (adds `live/exec/` beside the engine) | new platform in the loop; decision layer ownership decision |
| Time to a signed gate | shorter (no new spend/port), higher gate burden (G2.5) | longer (port + parity + licence), lower runtime risk |
| Kill path | disable flag, stop executor, manual card, human close in Tradovate | disable strategy, platform flatten, human close in Tradovate |
| Honest failure mode | a page change breaks it (fails closed, no send) | a strategy bug trades inside a platform that is built for automation |

## A.5 — Recommendation to Nox (the basis choice)
Two defensible bases; the choice is Nox's:
1. **CDP ticket route first** (recommended for the first contained build): no new spend, reuses the sanctioned desktop
   session route, keeps the whole change inside one `live/exec/` package, and the failure mode is a refusal rather than an
   unattended strategy. Its weakness is exactly why G2.5 is mandatory.
2. **NinjaScript EA bridge** if Nox will not accept DOM automation of a broker page as a basis, or if the CDP route fails
   G2.5: more supported, but it needs Cayleb's spend and a platform move, so it is a bigger job and a later gate.
Either way: G0's clause re-read first, then G1, and no build until Nox rules. The three rails and the one-trader rule do not
change on either basis.

## A.6 — What is needed next (in order)
1. Nox: pick the basis (CDP-first vs NinjaScript-first) and rule whether DOM ticket automation is an acceptable basis at all.
2. The clause re-read from the official FundedNext pages when the web returns (quote + date), then G1.
3. Only then: design-doc rewrite (§4 + G2.5 promotion), code, G2/G2.5/G3/G4, Cayleb's written go.
Nothing is built, armed, or sent tonight.
