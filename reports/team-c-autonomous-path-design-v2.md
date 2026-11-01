# TEAM C — AUTONOMOUS FUTURES ORDER PATH: DESIGN **v2** (CDP-only basis, post-G0)

trading-lead, 2026-11-01 ~09:1x ET / 14:1xZ. **Text only. No code, no config edit, nothing armed, no order, $0.**
Supersedes `team-c-autonomous-path-design.md` (31 Oct) **at §4 only** and promotes its G2.5; every other section of the base
design stands unchanged and is incorporated by reference (§§1–3, 5–13 of the base file, including the containment package,
single-trader lock, dry-run proof shape, four gates, kill path, out-of-scope list and the §10 "autonomous fill" definition).
Basis: `board/reports/team-c-firm-automation-verdict.md` (B0, read 2026-11-01) — automation permitted on this eval, **no API
surface offered → CDP ticket automation is the only autonomous path**; NinjaTrader/NinjaScript EA remains the supported
alternative only if Nox rejects DOM automation (new spend → Cayleb's decision). Authority: Cayleb 31 Oct ruling + ASK D14
(proceed unless Cayleb says stop) + Lead X1 (Monday stays `cards`, "NOT automated").

## A. §4 replacement — the interface (CDP ticket automation)
> **On FundedNext Flex there is no API.** Own bots/EAs are allowed through the platform login (Tradovate), so the only
> autonomous execution path on this eval is **ticket automation of the already signed-in `trader.tradovate.com` tab over
> CDP 9222** — the same Chromium/route the passive feed and account watcher already use, but writing into the order ticket.
> The REST-API path is **not an option** on this account and must not be designed against.
Operational shape:
- One package, one process, one instrument (MES / contract_id 4470322), one account, **one order in flight**.
- The executor uses the **existing signed-in session only**. It never signs in, never enters a credential, never handles a
  2FA prompt: a fresh credential/2FA demand = **STOP, disable, alert** (Cayleb's act, never improvised).
- The executor may navigate only between the **order ticket** and **positions** pages of that tab; any other page, any
  unexpected modal, any DOM that does not match the pinned map = **refuse and disable**.
- The CDP route is fragile by nature (DOM, tab navigation, page versions) → **G2.5 is a required gate**, not a fallback:
  pinned selectors against a recorded page snapshot, a tab-navigation test, an "expected ticket page" guard, and screenshot
  evidence per step.
- The desktop remains the initiator; **nothing listens** on any port; no inbound command channel exists.

## B. Rails → code predicates (what `rails.py` must refuse on)
Every row is a pure check over `(intent, state, config)` returning a named refusal. Desk rails are **never** relaxed here.
| # | Rail (source) | Predicate | Refusal reason |
|---|---|---|---|
| R1 | Daily loss stop -$400 (desk) | realised+open day P/L > -400 and "stop-out would cross -400" false | `daily_loss_stop` |
| R2 | Daily profit stop +$900 (desk) | realised day P/L < +900 | `daily_profit_stop` |
| R3 | Max 2 trades/day (desk) | trades_today < 2 | `max_trades` |
| R4 | 15-min cooldown after a loss (desk) | now − last_loss_ts ≥ 900 s | `cooldown` |
| R5 | Flat by 16:05 ET (desk) | now ET < 16:05 and no entry after 16:05; auto-flatten at 16:05 | `late_entry` |
| R6 | 1 MES, contract_id 4470322 (desk) | symbol/qty/contract match config | `instrument` |
| R7 | Min stop 2.0 pts (desk) | stop_pts ≥ 2.0 | `min_stop` |
| R8 | Feed fresh at intent (desk/design §7) | last bar age ≤ 60 s and feed not "retrying" | `stale_feed` |
| R9 | Session window | ET 09:46–12:00 and not a holiday/early close | `outside_window` |
| R10 | **Micro-scalp guard (firm: 10 s)** | no round-trip < 10 s (and no planned hold < 10 s) | `micro_scalp` |
| R11 | **2% of CME price limit (firm)** | the day's `cme_limits.json` is present, `status=SET`, `day` = trade date, `read_at` from that morning's 09:05 guard; last price not inside the 2%-of-limit exclusion band | `price_limit` — **fail closed if the record is missing, undated, stale or UNSET** (UNSET = NO TRADE) |
| R12 | **Gapped/illiquid market (firm)** | session state not gapped/illiquid; refuse on a gap-open entry | `illiquid_or_gap` |
| R13 | **One side only / no bracket exploitation (firm)** | exactly one side; stop and target on the entry's side; no simultaneous buy+sell bracket | `bracket_shape` |
| R14 | **Challenge-phase consistency (firm, phase-aware)** | if phase=Challenge: day profit must not exceed the 40% recalculation threshold (config `daily_consistency_cap_usd`, $1,000 for $50K) — treated as a cap (stricter); if phase=Funded: no firm consistency rail (desk rails still bind) | `consistency` |
| R15 | **MLL proximity (firm)** | equity must stay above the trailing MLL line with the intended stop-out | `mll` |
| R16 | **Self-contained bot (firm)** | AST/import assertion: no messaging client, no third-party app in the send path | `third_party` |
| R17 | Single trader + flat at start (design §6) | flock held; account flat positively proven; enable artefact valid | `trader_lock`/`not_flat`/`disabled` |
| R18 | Kill/ambiguity | unknown ack, timeout, page mismatch, engine/executor disagreement → disable | `ambiguity` |
### B.1 R11 — the daily CME limit record (owner ruled: trading-lead)
**Lead ruling 2026-11-01: trading-lead owns this record; produced during the 09:05 session guard, BEFORE the 09:15
attestations; unset = NO TRADE.** It is an **additional evidence artefact** — it does not change runbook v6, its five
attestations or the P0.1 signature (Monday is cards-only; the record builds the routine and the evidence trail).
- **Path:** `board/probes/team-c/<day>/cme-limit-record.md` + `cme_limits-<day>.json` (Monday:
  `board/probes/team-c/monday-2026-11-02/`). Method + generator: `board/probes/team-c/cme-limit-record/README.md` and
  `fetch-cme-limits.sh` (validated 2026-11-01 14:18Z, status SET, exit 0).
- **Provenance is the exchange page, never a desk-feed derivation.** Two exchange sources quoted with URL + read timestamp:
  (1) CME *Price Limits* — the U.S. equity-index rule, verbatim: *"7%, 13%, and 20% price limits are applied to the futures
  fixing price and are effective from 8:30 a.m. CT – 2:25 p.m. CT. Mondays through Fridays. From 2:25 p.m. to 3:00 p.m. CT,
  only the 20% price limit …"* plus the overnight 7% rule and the page's "As of Trade Date"/"Last Updated" lines;
  (2) CME *Fixing Prices* — the equity-index fixing price for ES Dec 2026 (the MESZ6 underlying level); validation read
  2026-11-01: **7614.06 @ 30 Oct 2026 03:00:05 PM CT**.
- **Computed levels:** exchange fixing price × exchange-published percentages (7/13/20%), plus the **2% exclusion band**.
  Validation values: 7% up/down 8147.04 / 7081.08; 13% 8603.89 / 6624.23; 20% 9136.87 / 6091.25; **no trade above 7984.10 or
  below 7233.36**.
- **2% interpretation (conservative, flagged for the G1 checker):** the firm does not define "within 2% of CME price
  limits"; the record applies the **union** of (A) distance to the limit level ≤ 2% of that level and (B) price move within
  2 percentage points of the 7% limit (≥5% from fixing). The bot refuses more, never less.
- **Fail-closed:** record missing/undated/stale (not produced in that day's guard), page unreachable, or any input unresolved
  → `cme_limits.json` **UNSET → the bot refuses every intent that day (NO TRADE)**, and the guard records `R11: UNSET` with
  the exact failure. Never inferred from the desk feed, a prior day, or a third-party site.
- **Transport honesty:** `www.cmegroup.com` rejects the desk's direct HTTP fetch (403); the record names the transport
  actually used (reader route) and the page URL + read timestamp. Both-fail → UNSET.
- **Executor contract:** reads only `cme_limits-<day>.json`; requires `status=SET`, `day` = the trade date, and a `read_at`
  from that morning. Anything else = refusal.

## C. Threat list (G1 must sign that each is addressed)
1. DOM/page change makes a send impossible or targets the wrong control → G2.5 pinned map + expected-page guard + fail closed.
2. Tab navigation (feed/positions/event pages) during a send → guard refuses; executor re-asserts the page before submit.
3. Stale frame/price drives an entry → R8 freshness at intent time; page price re-read immediately before submit.
4. Double send (retry/double-click/rapid two intents) → one intent in flight, idempotent intent ids, hash-chained log.
5. Orphan bracket (parent filled, stop/target not placed) → reconcile after every state transition; disable on orphan.
6. Session expiry / 2FA prompt mid-day → disable + alert; never attempt sign-in.
7. Engine/executor disagreement on the same decision → executor refuses that decision and records it (cards stay info-only).
8. Firm rules: micro-scalp, price-limit proximity, illiquid/gap, bracket shape, self-contained bot → R10–R13, R16.
9. Account-rule drift (consistency phase semantics) → R14 phase-aware; record the phase each day.
10. Firm-side auto-close at 4:10 PM ET → desk flatten 16:05 ET with 5 min margin (R5); never rely on the firm's auto-close.

## D. Gates (each fresh-context checker with veto; maker never checker)
| Gate | Evidence to sign | Veto if |
|---|---|---|
| **G0** | `team-c-firm-automation-verdict.md` (quoted clauses + page/read dates + §4 guards) | quotes do not support the verdict, or the "no API" reading is unfair |
| **G1 design** | this v2 + base design §§1–3,5–13 + rails table §B + threat list §C + G0 | any rail unrepresented, any credential in the plan, no kill path, G0 absent |
| **G2 dry-run** | closed-day dry runs: byte-identical rerun hashes; compare vs cards 0/0/0; refusal table (R1–R18 each refuses); AST proof (only `venue.py` drives CDP; no messaging client; no secret read) | non-determinism, any refusal that sends, any second CDP/sockets opener |
| **G2.5 CDP/DOM** | page snapshot hash, pinned selector map, tab-navigation test, "expected ticket page" guard test, per-step screenshots — all with no send | a needed selector unpinned, or a send possible off the expected ticket page |
| **G3 demo** | demo place/cancel/replace/duplicate-id/partial/reject + demo order-history export joined 1:1 to intents | ambiguous ack semantics, duplicate accepted |
| **G4 pre-arming** | G1–G3 signatures + **Cayleb's explicit written go** (his words, relayed in writing) + kill-path drill proven (disable flag, cancel-all, manual flatten card, executor stop, 16:05 auto-disable) + a fresh firm-page re-read | missing go, missing kill proof, any unsigned artefact |

## E. Kill path (CDP specifics, on top of base §9)
1. `EXEC_ENABLE=0` / move `state/enabled.json` → the executor refuses all new sends within one loop tick and cancels resting
   brackets through the ticket. 2. `systemctl --user stop <executor unit>`. 3. Manual CARD path in the engine. 4. Human closes
   in Tradovate by hand — the executor never fights a manual action; an unexpected manual fill/cancel disables it.
5. 16:05 ET auto-disable regardless of state. 6. If the page becomes unreadable, the executor **fails closed**: it does not
   guess, it alerts and disables.

## F. Dry-run proof (G2) — same shape as base §7, with the v2 additions
Closed day(s) through the real intent/rails/executor code with `venue.py` replaced by a recording stub:
(i) rerun → identical sha256 per artefact; (ii) vs that day's cards → 0 missing / 0 extra / 0 mismatched; (iii) **refusal
table R1–R18, each asserted to produce no send**, plus stale feed, non-flat start, expired go, second process, unknown ack;
(iv) static AST proof of the send boundary + no credential/messaging import; (v) two day shapes (30 Oct cards day + one
flat day). Artefacts to `board/probes/team-c/exec-dryrun-<date>/`.

## G. What v2 explicitly does not change
Rails (desk rails strictly binding, firm rails added only as refusals), the engine or its config, the strategy, sizing, the
one-trader rule, the out-of-scope list (base §11), Monday's cards day and its attestation (e). No spend. No order before G4.

## H. Open items for Nox / the Lead (only what the build needs)
1. **Basis**: accept CDP ticket automation as the basis (no spend) — or choose the NinjaTrader EA route (new spend → Cayleb).
   ASK D14 says proceed unless Cayleb stops; this asks for the *basis* ruling, not permission to exist.
2. **Demo access for G3**: which demo environment and who holds the demo keys (Cayleb/Nox) — no credential handled here.
3. **Go record**: the `go_id` artefact shape Cayleb will sign at G4 (one line, relayed in writing).
4. ~~CME limit input (R11): who records `cme_limit_price`?~~ **RULED 2026-11-01: trading-lead owns it** — produced at the
   09:05 guard before the attestations, exchange-page provenance + timestamp, **unset = NO TRADE** (see §B.1). Remaining
   sub-question for the G1 checker: accept the conservative union reading of "within 2% of CME price limits" (§B.1).
