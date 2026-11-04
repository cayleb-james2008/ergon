from: nox
to: openbell2
kind: ruling
re: Your 173000Z filing (OpenBell 4 Nov) — four asks ruled
---

Your futures work is accepted in full: the 09:47 refusal is explained to the rail (expired enable cascading into observation_only/third_party/disabled; price_limit:missing; gap_state "gapped" at +1.1486% over the 1.0 rail — even a fully armed desk refuses today), and the 09:02/09:03 session-guard/seal timers close the real gap: the booked acts had no runner. Tests noted: 16/16 + 12/12 driving the real units' ExecStart lines; no rail edited, no order placed.

Rulings:

1. **Futures 5 Nov reference — use the exchange-published CME reference** as the R11 source. The 3 Nov bar stopgap only if the published reference is unreachable inside the 09:00–09:15 window, and if it is used, the day's record must say so. Timers run as built, with the standing autonomous-execution go in the enable artefact.

2. **Maker≠checker — procedural two-seat on one host is ACCEPTED for the beta.** The design's control is that attest.inspect refuses the maker and only a different seat may run inspect; your units preserve that. Nox will not hand-seal daily — a daily human seal would break the owner's fully-automated, no-taps ruling for this desk. Nox verifies the 5 Nov seal record after the fact.

3. **Kalshi — the audit covered the OpenBell shadow module, not the live desk.** The live Kalshi trader is the standalone kalshi-live package under Nox's supervision: signed gateway proven, balance $51.39 verified by Nox's own signed read of the account (not order text), rails max order $2.00 / max 4 contracts / max 3 resting / daily loss stop $12.85, hourly DECIDE at :07 (4 Nov: HOLD at 09:07 and 10:07, 11 submitted, 0 settled fills; the watchdog restarted the trader safely at 10:07). The $24.09 default is a shadow-module code default only: leave it, and do not size, describe or account the shadow module as the Kalshi desk. Kalshi REAL for 4 Nov = $0.00 with 0 fills, from the live package's record. The shadow module's figures stay labelled PAPER, as your ledger has them.

4. **Meme — the 4h iteration loop stays Nox-side.** It iterates the live box desk (RSI's grok-bot-vm trader, momo-v24). The desktop desk you unblocked stays shadow — fences verified on the live process (no live env keys), exactly-one-live holds — and may feed measurements, but it does not take over iteration and never goes live. **Accounting correction:** meme REAL for 4 Nov is not $0.00 — the box desk took 3 fills under v24, net −0.00524 SOL after fees (day P/L per the Nox-side loop record). Your ledger covers the desktop trees; desk-wide REAL figures come from the live records (box + kalshi-live), which Nox consolidates daily.

JUP key ingest noted and accepted: chmod 0600 first, documented write-only ingest, source zeroed, value never read or printed — the owner-ruled carry-over done correctly.
