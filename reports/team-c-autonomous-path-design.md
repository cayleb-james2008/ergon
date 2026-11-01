# TEAM C — AUTONOMOUS FUTURES ORDER PATH: DESIGN SPEC (Tier S successor)
**Design only. No code written, no engine/config edited, nothing armed.** trading-lead, 2026-10-31 18:4x ET.
Authority: Cayleb's 31 Oct ruling (the ORB15 beta is fully automated, engine executes inside the rails, Cayleb notified
after, no per-trade taps) + Lead ruling D14/X1 (this is the successor build; Monday runs in `cards` with attestation (e)
stated as "cards, NOT automated"). Any order-placing capability is Tier S: **four checker gates with veto + Cayleb's
explicit written go** before it can trade.

## 1. Problem it solves (the honest statement)
The deployed engine has no order path at all: `live/engine.py` allows only `mode: shadow | cards`
(`if cfg["mode"] not in ("shadow","cards")`, engine.py:25,67), and the systemd unit's own description is "MES signal
engine (passive Tradovate feed, shadow ledger, ntfy cards; **never places orders**)". `cards` = human places the bracket.
So the beta has never had a way to fill without Cayleb tapping, the eval has been flat for a week with the card path
partly delivering, and "first autonomous fills" cannot be produced by today's code. This spec defines the smallest
contained path that could, and the evidence each gate needs.

## 2. Non-negotiable rails (unchanged, re-checked in-process before every send)
- -$400 daily loss stop; +$900 daily profit stop; $1,000 single-day firm profit cap; 40% consistency rule; **flat 16:05 ET**
  (before FundedNext's 16:10 auto-close).
- 1 MES (contract_id 4470322 / MESZ6), **max 2 trades/day**, **15-minute cooldown after a loss**, no entries 16:05–18:00 ET,
  no entries in a holiday/early-close session, min stop 2.0 pts, and "no trade whose stop-out would cross -$400".
- No new funds, no second account, no size increase. **Exactly one live trader at a time.**
- Verification is the venue's record (orders/fills/P&L frames) + the decision log — never the engine's self-report.

## 3. Gate G0 — may this eval be traded automatically at all? (a check before any build)
Prop-firm terms differ on automated entry. Before one line of executor code: read the eval's current terms/FAQ for the
automation and API clauses (on-file docs first; a support question is an outside send and needs Cayleb's OK — Job 12 item
5b pattern). If the firm forbids automated order entry on this eval, **the build stops here** and the honest answer is
"autonomy is not available on this account; cards is the only path" — that is a finding for Cayleb, not a workaround.
Artefact: `board/reports/team-c-firm-automation-verdict.md` (quote the clause + date read), checker-signed.

## 4. Interface choice — ONE venue path, recommended in this order
**Preferred: Tradovate REST API** (demo/`demo.tradovateapi.com` first, live only after G3), because a request/response
surface is testable, replayable, idempotent with `clientOrderId`, and auditable; it also cannot be broken by a tab
navigation or a Chromium restart. Requires an API key pair created **by Cayleb** in the Tradovate account settings
(write-only paste, never printed; if the key is not available the path does not start).
**Fallback (only if no API entitlement): CDP ticket automation of the already signed-in `trader.tradovate.com` tab**
(the same Chromium/9222 route the watcher uses, but writing into the order ticket). Weaker on every axis: DOM-fragile,
harder to make deterministic, and the tab can change under it — acceptable only with the extra G2.5 gate below.
**Rejected:** a second broker/session (breaks sign-in discipline and the one-trader rule), key material on the desk
machine, any inbound-command channel (the desktop is the initiator; nothing listens).

## 5. Architecture (contained: one package, one process, one instrument, one account)
New package `live/exec/` next to the engine; the engine itself is **not modified** (it keeps emitting the card/decision).
- `intent.py` — frozen intent schema v1: `{intent_id, decision_ts, symbol, side, qty, entry_ref_price, stop_pts, target_pts,
  day, card_hash, created_utc}`; intents are derived 1:1 from the engine's ledger/cards, never invented.
- `rails.py` — **pure** function `check(intent, state, config) -> (allow, reasons[])` covering every rail in §2 plus:
  trading day/session state (holiday/early close), feed freshness, and the "stop-out would cross -$400" rule. No I/O.
- `guard.py` — enable flag + Cayleb's go record + single-trader lock + flat-at-start check. Fail-closed: any missing,
  corrupt or expired artefact -> refuse. `state/enabled.json` holds `{enabled:false|true, go_id, signed_by_checker,
  expires_utc, reason}`; default on disk is **disabled**, and the file is written only by the manual arming step.
- `executor.py` — the only process that may send. State machine: `IDLE -> INTENT -> GUARD -> SEND -> ACK -> MANAGE -> FLAT`.
  One order in flight at a time; every transition appended to a hash-chained `state/intents.jsonl` (prev_hash like the
  Kalshi ledger). On any ambiguity (timeout, unknown ack, venue error): **disable and alert**, never retry blindly.
- `venue.py` — the ONLY module allowed to open a socket (AST test asserts nothing else imports a network client).
  Methods: `place(intent)`, `cancel(order_id)`, `replace(order_id, new)`, `flatten(account)`, `positions(account)`,
  `fills(since)`. Every call carries `clientOrderId = "ob-<day>-<seq>"`; duplicate-id behaviour is tested at G3.
- `reconcile.py` — joins fills (venue) to intents and engine decisions; the day is "clean" only if
  0 unmatched fills, 0 unmatched intents, and every decision has an outcome.
- `state/` — `enabled.json`, `trader.lock` (flock), `intents.jsonl`, `dryrun/`, `daily/<day>.json` (start/end equity,
  rails counters, kill reason).

## 6. Single-trader lock and flat-at-start (never two traders, two implementations)
1. `flock` on `state/trader.lock`; a second executor exits 3 immediately.
2. Startup refuses unless: engine heartbeat fresh, config rails all present and typed, **account flat** (broker frames
   show `positions {}`), no manual/off-card position, enable artefact valid.
3. The engine keeps running in `cards` mode beside it for the first live day (information only): both produce the same
   cards; only `executor.py` may send. If the executor and engine disagree on a decision, the executor refuses that
   decision and records the disagreement.
4. Any other order-placing process found on the account -> executor disables itself and alerts (one trader rule).

## 7. Dry-run proof (must be byte-identical on rerun, before anything is sent anywhere)
Mode `--dry-run --day <closed day>`: consume that day's `live/ledger/<day>.jsonl` + `live/logs/cards.jsonl` through the
real `intent.py`/`rails.py`/`executor.py` code with `venue.py` replaced by a recording stub. Writes
`state/dryrun/intents.jsonl` and `state/dryrun/summary.json`. **Proof:** (i) rerun -> identical sha256 per artefact;
(ii) comparison vs the day's cards: 0 missing / 0 extra / 0 mismatched intents (the Kalshi WP10 `compare.py` pattern);
(iii) the refusal table: every one of the §2 conditions, plus stale feed, non-flat start, expired go, second process,
unknown ack — each asserted to produce **no send**; (iv) static AST proof that only `venue.py` may open a socket and that
no credential is read from the repo. Dry-run stays on closed windows until every proof lands.

## 8. Four checker gates, each with VETO (fresh context, never the builder)
| Gate | When | Evidence the checker signs (on disk) | Veto condition |
|---|---|---|---|
| **G1 design** | before code | this spec + the rails table + G0 firm-verdict + threat list | any rail unrepresented, any credential in the repo plan, G0 absent |
| **G2 dry-run** | before any demo send | dry-run artifacts: byte-identical rerun hashes, compare 0/0/0, refusal table all-refuse, AST proof, enabled.json default-disabled | any non-determinism, any refusal that sends, any second socket opener |
| **G2.5 (CDP fallback only)** | before demo send | DOM selector map pinned to a recorded page snapshot, tab-navigation test, screenshot evidence per step | any step needing a selector not pinned, or a send possible while the page is not the expected order ticket |
| **G3 demo venue** | before any live arming | demo account orders: place / cancel / replace / duplicate-clientOrderId / partial fill / reject, with the demo's own order-history export and intents joined 1:1 | any ambiguity in ack semantics, any duplicate accepted |
| **G4 pre-arming** | before `enable=1` | G1–G3 signatures + Cayleb's **explicit written go** (his words, relayed in writing) + a kill-path drill: disable flag, cancel-all, manual flatten card, engine stop — each proven | missing go, missing kill proof, or any unsigned artefact |
The arming step itself is manual and single: write `state/enabled.json` with `go_id` + the checker's signature id, set the
service env `EXEC_ENABLE=1`, restart only the executor. Disarming is the reverse and always allowed without approval.

## 9. Kill path (documented before arming, proven at G4)
1. `EXEC_ENABLE=0` / remove-move `enabled.json` -> executor refuses all new sends within one loop tick and cancels resting
   brackets. 2. `systemctl --user stop <executor unit>`. 3. Manual `CANCEL`/`CLOSE NOW` card path (already in the engine).
4. The human can always close the position in Tradovate by hand; the executor must never fight a manual action — if it
   sees an unexpected manual fill/cancel it disables itself. 5. A daily auto-disable at 16:05 ET regardless of state.

## 10. What must be true for Monday 16:20 to claim "fills" honestly (definition of done for Job 12)
A "fill" claim requires ALL of the following, quoted from files:
1. The venue's own record shows the order(s): order ids, fills, timestamps (Tradovate order/fill history or account
   frames carrying them — not the engine's ledger).
2. Each fill joins 1:1 to a decision record in `live/ledger/<day>.jsonl` and to a card/row in `live/logs/cards.jsonl`.
3. The decision log is complete: every signal has an outcome (`fill` / `cancel` / `skip`) with no orphan signal.
4. The executor's `state/intents.jsonl` hash chain verifies and its intent ids match the decisions.
5. Rails accounting: ≤2 trades, no -$400/+$900 touch, cooldown honoured, flat by 16:05, and equity movement reconciles
   from the broker's start-of-day balance to the end-of-day balance (fills + fees, tolerance stated).
6. Every fill is attributable to the executor (`clientOrderId` prefix) — a human/off-card fill is named as such.
If any item is missing the claim is **not** "autonomous fills": it is "cards day (human-tap)", "no trade", or "failed at
gate X". This definition applies equally to the cards-mode Monday and to the future autonomous day.

## 11. Explicitly out of scope
No multi-venue, no multi-instrument, no sizing changes, no new funds, no scaling, no strategy changes, no change to any
rail or to the engine's decision logic, no Kalshi/meme integration, no general desktop control (Stage 8 owns that), no
credential in any repo/log, no inbound port on the desktop, no unattended hours outside 09:30–16:05 ET.

## 12. Open questions (for Cayleb / Nox; none asked directly by this team)
Q1. Firm automation clause (G0) — is automated order entry permitted on the FundedNext Flex eval? (Clause + date needed.)
Q2. Tradovate API entitlement + who creates the key pair (Cayleb, write-only) — or is CDP ticket automation the only option?
Q3. Is there a demo account to run G3 against (FundedNext demo or Tradovate demo) and who owns those demo keys?
Q4. Who holds the kill (Cayleb at the desk, or the checker via the disable flag) and where is the go recorded?
Q5. Does the $1,000 single-day firm cap count net-of-fees the way the engine's +$900 stop does? (Affects the rail re-check.)
Q6. Confirm the intended meaning of "notified after": ntfy card per fill (current) is enough, or a digest?

## 13. Risk register (top)
- A bug that trades: mitigated by G1–G4, default-disabled artefact, refusal table, one-order-at-a-time, auto-disable on
  ambiguity, and the 16:05 hard flatten.
- Venue semantics unknown (reject/partial/dup): mitigated by G3 on demo, clientOrderId namespacing, and "disable on ambiguity".
- Feed staleness driving a stale entry: rail check requires fresh feed at intent time (same 60 s rule as the engine).
- Dual control (engine cards + executor): cards stay information-only; only `executor.py` may send; disagreement -> refuse.
- Browser fallback fragility: extra G2.5 gate, pinned selectors, screenshot evidence per step; still the weaker path.
- Credential creep: the key lives outside the repo (Cayleb paste, write-only), secret scan in the pre-commit gate,
  no key in logs/cards; the executor reads it from the same place the watcher reads account data.
Estimated build (not a promise): G1 ~1 session, package + tests ~2–3 sessions, G2/G2.5 ~1, G3 demo ~1 (needs demo access),
G4 + arming after Cayleb's go. Monday 2 Nov cannot be an autonomous day; the earliest honest target is a later session
with every gate signed.
