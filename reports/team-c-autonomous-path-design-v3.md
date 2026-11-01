# TEAM C — AUTONOMOUS FUTURES ORDER PATH: DESIGN **v3** (G1-revision per the checker's HOLD)

trading-lead, 2026-11-01 ~10:2x ET / 15:2xZ. **Text only. No code, no config edit, nothing armed, no order, $0.**
Basis: **G0 PASS** — independent Tier S checker signature **G0-SIG-2026-11-01-TS-01**
(`board/reports/team-c-firm-automation-verdict.md`; verdict file `board/reports/team-c-g0-g1-checker-verdict.md`, sha256
`afc44f85…`): automation permitted on this eval, **no API surface documented** → CDP ticket automation is the only autonomous
path. This v3 answers the checker's **G1 HOLD** `G1-HOLD-2026-11-01-TS-01` (fixes C1–C4) and supersedes v2 entirely.
Authority: Cayleb 31 Oct ruling + ASK D14 (proceed unless Cayleb says stop) + Lead X1 (Monday stays `cards`, "NOT automated").

## 0.1 What changed from v2 → v3 (the G1 HOLD, fixed)
| Checker fix | Where fixed in v3 |
|---|---|
| **C1 (decisive)** — reflect the R11 Lead ruling: trading-lead owner, production at the **09:05 guard BEFORE the 09:15 attestations**, dated **day-keyed** artefact with path/schema + exchange page + read timestamp (yesterday's value unusable), and an **attestation linkage**; delete §H.4 "who records it" | **§B.1** (owner/timing/path/schema/staleness/attestation linkage) and **§H** (H.4 removed; H.4 is now the G1 re-sign) |
| **C2** — R11's predicate must be the firm's **two-sided band** from the prior EOD settlement: Upper = Ref×(1+0.07−0.02), Lower = Ref×(1−0.07+0.02); limits re-set 4:05 PM CT; overnight vs RTH differ — a single scalar would pass a limit-down session | **§B row R11** + **§B.1** (formula, session windows, fail-closed) |
| **C3** — add the **cancel/replace-rate refusal row** (G0 §4 required it; "any rail unrepresented" veto) | **§B row R19** |
| **C4** — strike/override the stale REST-era credential text in base §§5/12-Q2/13; phase-gate the $1,000 wording consistently with R14; §A "there is no API" → "no API surface documented"; add an explicit reverse/correlated-hedging assertion to R13 | **§A.1** (base overrides), **§B R14** (phase-gated wording), **§A** (wording), **§B R13** (assertion) |
Also carried forward from v2: CDP-only §4 replacement, G2.5 promoted to a required gate, R1–R18 content, threat list, kill
path, gate evidence table, dry-run shape, out-of-scope list, and the base design §§1–3, 5–13 by reference **except the
overrides in §A.1**.

## A. §4 replacement — the interface (CDP ticket automation)
> **No API surface is documented on this eval.** Own bots/EAs are allowed through the platform login (Tradovate), so the only
> autonomous execution path here is **ticket automation of the already signed-in `trader.tradovate.com` tab over CDP 9222** —
> the same Chromium/route the passive feed and account watcher already use, but writing into the order ticket. The REST-API
> path is **not available** (no documented surface) and must not be designed against.
Operational shape:
- One package, one process, one instrument (MES / contract_id 4470322), one account, **one order in flight**.
- The executor uses the **existing signed-in session only**. It never signs in, never enters a credential, never handles a
  2FA prompt: a fresh credential/2FA demand = **STOP, disable, alert** (Cayleb's act, never improvised). **No key material
  exists, is read, is stored or is logged on this route.**
- The executor may navigate only between the **order ticket** and **positions** pages of that tab; any other page, any
  unexpected modal, any DOM that does not match the pinned map = **refuse and disable**.
- The CDP route is fragile by nature (DOM, tab navigation, page versions) → **G2.5 is a required gate**: pinned selectors
  against a recorded page snapshot, a tab-navigation test, an "expected ticket page" guard, and screenshot evidence per step.
- The desktop remains the initiator; **nothing listens** on any port; no inbound command channel exists.

### A.1 Base-design overrides (C4) — these base sentences are STRUCK or REPLACED
1. **Base §4** (REST-API preferred, CDP fallback) — **replaced** by v3 §A.
2. **Base §5 `venue.py`** — "the ONLY module allowed to open a socket" → **replaced** by: "`venue.py` is the **only** module
   allowed to drive CDP (attach/navigate/evaluate/dispatch in the tab); an AST test asserts no other module navigates,
   evaluates or opens a socket. CDP connection to the *existing* local listener is not network egress."
3. **Base §12 Q2** ("Tradovate API entitlement + who creates the key pair") — **STRUCK**. There is no API surface documented
   and no key pair; no credential is created, asked for or handled on this route.
4. **Base §13 credential-creep bullet** ("the key lives outside the repo (Cayleb paste, write-only)…") — **STRUCK/REPLACED**
   by: "no key exists on this route; the executor reads no secret; a fresh credential or 2FA prompt disables it and stops for
   Cayleb. G2 asserts (AST/static) that no secret file, env var or credential path is read."
5. **Base §2 "$1,000 single-day firm profit cap"** — **phase-gated** (see R14): on the **Flex Challenge** the firm's 40%
   consistency rule makes $1,000 the daily profit threshold (exceeding it **recalculates** the required total; it is not a
   breach); on a **Flex FundedNext Account** there is **no consistency rule and no daily loss limit**. The desk's $1,000 cap
   rail remains in force in **both** phases as a **desk rail**, not a firm rule.
6. Everything else in base §§1–3, 5–13 stands, including the §10 "autonomous fill" definition and §11 out-of-scope list.

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
| **R11** | **2% of CME price limits (firm) — two-sided band** | the day's `cme_limits-<day>.json` is SET, day-keyed and fresh; the last price is **not inside** the upper band `[Ref×(1+0.07−0.02), Ref×(1+0.07)]` or the lower band `[Ref×(1−0.07), Ref×(1−0.07+0.02)]`, using the active session's limit set | `price_limit` — **UNSET/stale/missing = refuse** |
| R12 | **Gapped/illiquid market (firm)** | session state not gapped/illiquid; refuse on a gap-open entry | `illiquid_or_gap` |
| R13 | **Bracket shape + no reverse/correlated hedging (firm)** | exactly one side; stop and target on the entry's side; no simultaneous buy+sell bracket; **one instrument only** — refuse if any other position exists in the account or an offsetting/correlated position (ES/MNQ/NQ/YM, same or correlated underlying) would be live | `bracket_shape` / `hedging` |
| R14 | **Phase-aware consistency (firm)** | **Challenge phase:** desk day-profit cap `daily_consistency_cap_usd` = $1,000 (the firm's 40% × $2,500 recalculation threshold, treated as a cap = stricter). **FundedNext Account:** no firm consistency rule and no firm daily loss limit — the desk cap rail still binds (it is a desk rail). Phase is a recorded config field, not inferred | `consistency` |
| R15 | **MLL proximity (firm)** | equity must stay above the trailing MLL line with the intended stop-out | `mll` |
| R16 | **Self-contained bot (firm)** | AST/import assertion: no messaging client, no third-party app in the send path | `third_party` |
| R17 | Single trader + flat at start (design §6) | flock held; account flat positively proven; enable artefact valid | `trader_lock`/`not_flat`/`disabled` |
| R18 | Kill/ambiguity | unknown ack, timeout, page mismatch, engine/executor disagreement → disable | `ambiguity` |
| **R19** | **Cancel/replace-rate guard (firm anti-spam)** | ≤1 replace per intent and any replacement needs a **fresh decision ref**; ≤3 cancel/replace events per rolling 10 min; cancel-before-new; one order in flight | `cancel_rate` / `replace_without_decision` |

### B.1 R11 — the daily CME limit record (C1 + C2: owner, timing, artefact, two-sided predicate)
**Owner: trading-lead** (Lead ruling 2026-11-01). **Produced during the 09:05 session guard, BEFORE the 09:15 attestations.**
It is an **additional evidence artefact** — it does **not** change runbook v6, its five attestations or the P0.1 signature.
- **Path / filename:** `board/probes/team-c/<day>/cme-limit-record.md` + `board/probes/team-c/<day>/cme_limits-<day>.json`
  (Monday: `board/probes/team-c/monday-2026-11-02/`). Generator + method:
  `board/probes/team-c/cme-limit-record/{README.md,fetch-cme-limits.sh}`.
- **Exchange provenance (quoted with page URL + read timestamp):** (1) CME *Price Limits* — the U.S. equity-index rule,
  verbatim: *"7%, 13%, and 20% price limits are applied to the futures fixing price and are effective from 8:30 a.m. CT –
  2:25 p.m. CT. Mondays through Fridays. From 2:25 p.m. to 3:00 p.m. CT, only the 20% price limit will be applied to the
  futures price fixing."* plus the overnight rule: *"7% up-and-down limits are effective 5:00 p.m. - 8:30 a.m. CT. Sundays
  through Fridays; and 3:00 p.m. - 4:00 p.m. CT, Mondays through Fridays. Between 3:00 p.m. - 4:00 p.m. the 7% price limit
  will not be allowed to breech the 20% daily limit."*; (2) CME *Fixing Prices* — the equity-index reference for ES Dec 2026
  (the MESZ6 underlying level), e.g. **7614.06 @ 30 Oct 2026 03:00:05 PM CT** (validation read 2026-11-01). **`Ref` is the
  prior session's exchange-published EOD settlement/fixing reference — never a desk-feed price.**
- **Predicate (C2, two-sided — a scalar would pass a limit-down session):** with `Ref` = prior EOD reference,
  `Up = Ref×(1+0.07)`, `Down = Ref×(1−0.07)`:
  - **refuse if `Ref×(1+0.07−0.02) ≤ price ≤ Ref×(1+0.07)`** (upper 2% band), or
  - **refuse if `Ref×(1−0.07) ≤ price ≤ Ref×(1−0.07+0.02)`** (lower 2% band).
  The 13%/20% levels are recorded and carry the same 2% bands for the sessions where they govern. **Limits are re-set after
  the 4:05 PM CT settlement publication; the overnight and RTH limit sets differ** and the active window selects the set:
  day 7/13/20% 08:30–14:25 CT; 14:25–15:00 CT only 20%; overnight 7% 17:00–08:30 CT and 15:00–16:00 CT.
  Validation values (Ref 7614.06): upper band **[7994.76, 8147.04]**, lower band **[7081.08, 7233.36]**.
- **Staleness (C1):** the sidecar is **day-keyed**. The executor requires `status=SET`, `day` == the trade date, and
  `read_at` from that morning's guard window. **Yesterday's record is unusable, never reused.** A record from a prior day, a
  different contract, or with a mismatched exchange read timestamp = `UNSET`.
- **Schema (`cme_limits-<day>.json`, schema 2):** `{schema:2, day, contract:"MESZ6", phase:"challenge|funded",
  ref_price, ref_ts, ref_source, limits:{up7,down7,up13,down13,up20,down20},
  refuse_bands:{upper:[lo,hi], lower:[lo,hi], upper13:[lo,hi], lower13:[lo,hi], upper20:[lo,hi], lower20:[lo,hi]},
  session:{day_limits_ct:"08:30-14:25", late_ct:"14:25-15:00 (20% only)", overnight_7pct_ct:"17:00-08:30 & 15:00-16:00"},
  exchange_rule, exchange_read_at, read_at, owner:"trading-lead", status:"SET"|"UNSET"}`.
- **Fail-closed:** record missing/undated/stale, exchange page unreachable, `Ref` unresolved, or any band unresolved →
  `UNSET` → **the bot refuses every intent that day (NO TRADE)**; the guard records `R11: UNSET <reason>`. Never inferred from
  the desk feed, a prior day, or a third-party site.
- **Attestation linkage (C1):** the five attestations (a)–(e) are unchanged and R11 is **not** an attestation. The 09:05
  guard evidence file (`guard.md`) carries one extra line: `R11: SET <relative path> | Ref=<value>@<exchange ts> |
  bands up=[lo,hi] down=[lo,hi] | read_at=<utc>` **or** `R11: UNSET <reason>`. The 16:20 outcome check quotes that line and
  states whether any intent was refused for `price_limit`. On an autonomous-capable day, `UNSET` = no entries all day (a
  rail refusal, logged by the executor); on Monday (cards-only) the line is recorded for routine/evidence with no trading
  consequence. Any attestation FAIL/UNVERIFIED rule is unchanged — R11 does not soften it.
- **Transport honesty:** `www.cmegroup.com` rejects the desk's direct HTTP fetch (403); the record names the transport
  actually used (reader route) and the page URL + read timestamp. Both-fail → `UNSET`.

## C. Threat list (G1 must sign that each is addressed)
1. DOM/page change makes a send impossible or targets the wrong control → G2.5 pinned map + expected-page guard + fail closed.
2. Tab navigation (feed/positions/event pages) during a send → guard refuses; executor re-asserts the page before submit.
3. Stale frame/price drives an entry → R8 freshness at intent time; page price re-read immediately before submit.
4. Double send (retry/double-click/rapid two intents) → one intent in flight, idempotent intent ids, hash-chained log.
5. Orphan bracket (parent filled, stop/target not placed) → reconcile after every transition; disable on orphan.
6. Session expiry / 2FA prompt mid-day → disable + alert; never attempt sign-in; no credential exists on the route.
7. Engine/executor disagreement on the same decision → executor refuses that decision and records it (cards stay info-only).
8. Firm rules: micro-scalp, price-limit band, illiquid/gap, bracket shape, reverse/correlated hedging, cancel/replace churn,
   self-contained bot → R10–R13, R16, R19.
9. Account-rule drift (consistency phase semantics) → R14 phase-aware; record the phase each day.
10. Firm-side auto-close at 4:10 PM ET → desk flatten 16:05 ET with 5 min margin (R5); never rely on the firm's auto-close.
11. Stale/reused R11 record → day-keyed sidecar + executor refuses a mismatched day (B.1).
12. Cancel/replace churn drifting into spam → R19 rate guard + disable on breach.

## D. Gates (each fresh-context checker with veto; maker never checker)
| Gate | Evidence to sign | Veto if |
|---|---|---|
| **G0** | `team-c-firm-automation-verdict.md` + quoted clauses + page/read dates + §4 guards | quotes do not support the verdict; "no API" stated as more than an inference |
| **G1 design** | **this v3** + base design §§1–3,5–13 with §A.1 overrides + rails table §B (R1–R19) + R11 §B.1 + threat list §C + G0 | any rail unrepresented, any credential in the plan, no kill path, G0 absent, R11 without the two-sided band/day-key/staleness |
| **G2 dry-run** | closed-day dry runs: byte-identical rerun hashes; compare vs cards 0/0/0; refusal table (R1–R19 each refuses); AST proof (only `venue.py` drives CDP; no messaging client; no secret read) | non-determinism, any refusal that sends, any second CDP/sockets opener |
| **G2.5 CDP/DOM** | page snapshot hash, pinned selector map, tab-navigation test, "expected ticket page" guard test, per-step screenshots — all with no send | a needed selector unpinned, or a send possible off the expected ticket page |
| **G3 demo** | demo place/cancel/replace/duplicate-id/partial/reject + demo order-history export joined 1:1 to intents | ambiguous ack semantics, duplicate accepted |
| **G4 pre-arming** | G1–G3 signatures + **Cayleb's explicit written go** + kill-path drill proven (disable flag, cancel-all, manual flatten card, executor stop, 16:05 auto-disable) + **G0-C4 fresh re-read of all four firm pages** | missing go, missing kill proof, any unsigned artefact, changed clause |

## E. Kill path (CDP specifics, on top of base §9)
1. `EXEC_ENABLE=0` / move `state/enabled.json` → the executor refuses all new sends within one loop tick and cancels resting
   brackets through the ticket. 2. `systemctl --user stop <executor unit>`. 3. Manual CARD path in the engine. 4. Human closes
   in Tradovate by hand — the executor never fights a manual action; an unexpected manual fill/cancel disables it.
5. 16:05 ET auto-disable regardless of state. 6. If the page becomes unreadable, the executor **fails closed**: it does not
   guess, it alerts and disables.

## F. Dry-run proof (G2) — same shape as base §7, with the v3 additions
Closed day(s) through the real intent/rails/executor code with `venue.py` replaced by a recording stub:
(i) rerun → identical sha256 per artefact; (ii) vs that day's cards → 0 missing / 0 extra / 0 mismatched; (iii) **refusal
table R1–R19, each asserted to produce no send**, plus stale feed, non-flat start, expired go, second process, unknown ack,
stale R11, R11 band hit (both sides), cancel-rate breach, replace-without-decision; (iv) static AST proof of the send
boundary + no credential/messaging import; (v) two day shapes (30 Oct cards day + one flat day). Artefacts to
`board/probes/team-c/exec-dryrun-<date>/`.

## G. What v3 explicitly does not change
The desk rails (strictly binding; firm rails added only as refusals), the engine or its config, the strategy, sizing, the
one-trader rule, the out-of-scope list (base §11), Monday's cards day and its attestation (e). No spend. No order before G4.

## H. Open items (only what the build needs)
1. **Basis**: accept CDP ticket automation as the basis (no spend) — or choose the NinjaTrader EA route (new spend → Cayleb).
   ASK D14 says proceed unless Cayleb stops; this asks for the *basis* ruling, not permission to exist.
2. **Demo access for G3**: which demo environment and who holds the demo keys (Cayleb/Nox) — no credential handled here.
3. **Go record**: the `go_id` artefact shape Cayleb will sign at G4 (one line, relayed in writing).
4. **G1 re-sign**: the same checker seat re-signs G1 against this v3; the two-sided R11 predicate (C2) and the R19 row (C3)
   are the substantive changes, and C1/C4 are the compliance changes. Nothing else is open from the HOLD.
