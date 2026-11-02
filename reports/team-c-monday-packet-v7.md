# MONDAY PACKET — **v7** (Team C corrections) — 2 Nov 2026 (ET)

Supersedes the 1 Nov 22:31 ET `board/reports/MONDAY-PACKET.md` **for the trading legs only** (01:00, 07:45 trading checks,
09:05, 09:15, 09:35–09:44, ~09:40, 09:45, 16:05, 16:20). Written by trading-lead 2026-11-01 08:5x ET / 13:5xZ after the
Sunday ground-truth checks in `board/reports/team-c-sunday-groundtruth-20261101.md`. Nothing in this file changes the rails,
the signed runbook, or any attestation rule. **Any attestation FAIL or UNVERIFIED = the 09:45 window does not open.**

## Corrections in v7 (what changed from the 22:31 ET packet)
1. **DST line was false.** The packet said the 05:50Z/07:15Z artefacts were "verified by trading-checker at ~07:20Z".
   There are **no 07:15Z artefacts** and **no checker audit**: the armed runner was SIGTERMed at `2026-11-01T06:48:20Z`
   ("on client request", DSH app-instance teardown — see
   `board/probes/team-c/dst-20261101T0715Z/ADJUDICATION.md`). Verdict **INCOMPLETE**; the fallback itself was clean
   (LATE re-run at 13:37:50Z: 9 PASS / 1 NA, chain seq 8073→11727, EST -0500). Checker sign still owed.
2. **Runner liveness + supervisor** added to the 07:45 sweep (Lead ruling 2026-11-01): any future armed run is launched as a
   `systemd-run --user --transient` unit, never as a `nohup` child of a session; a watchdog must alert if it is absent at
   T−5min and assert its artefacts at T+2min.
3. **Pre-09:05 P0 block** (P0.1–P0.4) and the **(a2) positions-panel flat read** are written in explicitly — (a2) is the leg
   that can end the day, and it was only implied before.
4. **Failure disposition** written in: Cayleb's desktop sign-in on the existing Chromium tab is the **only verified route**;
   a fresh credential/2FA demand = STOP FOR CAYLEB, never improvise; no other host, no shadow substitute, no manufactured fill.
5. **16:05 / 16:20 rules** written in (two accepted flat proofs; non-masking; headline = day-filtered completeness PASS/FAIL,
   PASS-empty defined).
6. **Kalshi 01:00 bullet corrected**: L\* = 30 s is **ruled and applied**; freeze tooling is **built and tested** (15 tests
   pass); what remains unruled is the stale-header wording and the Kelly posture.
7. **Weekend baseline recorded** so Monday 09:05 has a compare point (§0).

## 0. Baseline verified Sunday 1 Nov (compare point for Monday; all read-only)
| Item | Sunday value (timestamp) |
|---|---|
| Host clock | `US/Eastern (EST, -0500)`, synchronised, NTP active (13:33Z) |
| Kalshi units | poll/ref/shadow **enabled AND active**; installed copies IDENTICAL to repo copies; heartbeats < 3 s (13:37Z) |
| Kalshi capture | fresh to `2026-11-01T13:37:50Z`; 2,910,685 records, 0 errors; `readiness.py --day 2026-11-01` EXIT=0, 9 PASS/1 NA/0 FAIL |
| Kalshi ledger | decisions seq 11727, orders seq 89520; replay byte-identical |
| Futures units | openbell-live/dash/tabkeeper `active` + `enabled`; engine PID 1484 continuous since **Wed 28 Oct 15:40:09 EDT** |
| Futures feed | `feed="retrying (RuntimeError)"`, `feed_age_s=146367.1` (dead since Friday close), `market_open=False`, `last_bar_et=16:59`, last bars file `2026-10-30` — **expected on a weekend** |
| Futures config↔process | `CONFIG_OLDER_THAN_PROCESS: PASS` (`config_mtime 2026-10-27T22:43:34Z` < `start 2026-10-28T19:40:09Z`) |
| Tab route | Chromium listening on `127.0.0.1:9222`; `tabkeeper.json` `"tradovate": "missing"` (expected Sunday) — **Monday must turn this into a positive tab + fresh bar proof at 09:35–09:44** |
| Runbook | v7 sha256 `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a` = the P0.1 signed hash (v7, checker-trading) |

## 01:00 ET — Claude usage reset + Kalshi resume + laptop lane  [owner trading-lead; backup lead]
- Read state first: `research/kalshi15/RESUME.md`, `DECISIONS.md`, this packet, `team-c-sunday-groundtruth-20261101.md`.
- Units: `systemctl --user is-active/enabled openbell-kalshi-{poll,ref,shadow}`. Sunday state is enabled+active; if a unit is
  failed/inactive, read its journal exit code **before** any restart (exit 2–6 = needs a human, never blind-restart). If the
  repo unit copy changed, diff → re-copy → `daemon-reload` (Sunday: IDENTICAL, so a no-op).
- Readiness: `--day 2026-11-01` (complete EST day) then `--day 2026-11-02` once data exists; report per-check lines.
- Restart the resumable fit-split pull (exact command in resume pack §3 item 6); keep total Kalshi load ≤ 5 req/s.
- Rulings: **L\* = 30 s is ruled and applied** (docs). Do **not** hand-edit `scorer_agreement.json` (`latency.L_star` = 60,
  regenerates on the next `stage_a.py dump-s`); `params.provisional.json "l_star_s": 60` stays as a data file — Stage A must
  run with `--L 30`. Still FOR NOX: stale-header wording (Amendment 5), Kelly posture (Amendment 6).
- Freeze: `freeze.py`/`forward.py` built and tested (15 tests pass); freeze correctly refuses until the 7 `<FILL AT FREEZE>`
  slots are filled. Freeze target **Tue 3 – Wed 4 Nov**, pre-freeze checker slot first.
- Laptop lane (Job 04): a laptop session picks up `board/reports/team-a-job04-handoff.md`; not reachable from this desktop —
  if it does not resume, report that plainly.
- **Fallback if Kalshi units cannot be recovered:** record the failing unit + journal exit code, keep the lane dark, do not
  improvise a restart, escalate to the Lead. No live order exists in any case.

## 07:45 ET — Monday loose-ends sweep  [owner upkeep-lead; backup lead] + trading legs added in v7
- Execute `board/reports/team-e-monday-sweep-runbook.md` (pre-flight verified 33 read-only commands).
- **Runner liveness (new):** run `bash board/probes/team-c/runner-liveness-check.sh` (read-only; prints runner processes, every
  armed-run directory, the `ARMED_RUN_DONE` marker and the after-artefact list, plus the supervisor command for the next run).
  A promised artefact that is absent is a finding, not silence. Record the DST verdict as **INCOMPLETE — the 07:15Z armed
  instant is unrecoverable** (the adjudication is independently signed **PASS** by the Tier S checker:
  `board/reports/team-c-dst-checker-verdict.md`, 14:20Z); never reconstruct a missing instant after the fact.
- Trading pre-checks: Kalshi units still enabled+active + capture fresh; futures units active; engine heartbeat age < 60 s;
  `tabkeeper.json` tradovate state; runbook v7 sha256 still `512752e4…`; `systemctl --user show openbell-live -p
  ExecMainStartTimestamp` still 28 Oct (a restart changes the evidence frame).
- Feed: expected-weekend "no trader.tradovate.com tab" must become a **proof** at 09:05/09:35–09:44 (tab present + fresh
  bar after the 09:30 open) — not an eyeball, the runbook's computed (c2) block.

## P0 — before 09:05 (all four; any FAIL = do not start)  [owner trading-lead; backup lead]
- **P0.1** checker veto signature on runbook v7 exists, timestamped before 09:05, and the file's sha256 still equals
  `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a` (verified Sunday 13:33Z; re-check at 09:05).
- **P0.2** Chromium listener on `127.0.0.1:9222` **and** `tabkeeper.json` tradovate ≠ "missing".
- **P0.3** `systemctl --user is-active openbell-live openbell-dash openbell-tabkeeper` → three `active`.
- **P0.4** no carry-over: heartbeat `pos= None paused= False` (`card_valid_min = 2` expires any old card by construction).

## 09:05 ET — SESSION GUARD (five items, all or FAIL)  [owner trading-lead; backup lead]
Append every line + output to `board/probes/team-c/monday-2026-11-02/guard.md`.
1. **Clock/TZ** `date; date -u; TZ=America/New_York date` → ET shows **EST (-0500)**, UTC differs by exactly 5 h (Sunday:
   already EST; DST ended 06:00Z 1 Nov). Wrong → STOP.
2. **Engine alive** heartbeat `age_s < 60`, `status= running`, `paused= False`.
3. **No carry-over from the broker record** — today's `live/ledger/account-2026-11-02.jsonl`: a `cashBalance` frame **with
   its `t` quoted**. If the last balance frame predates today 09:00, the post-frame flat state is **UNVERIFIED** → (a2)
   governs. (`positions {}` is NOT evidence.)
4. **Globex alive, RTH not yet** `market_open= True` **and** clock < 09:30. (`market_open` is the CME Globex session; True
   every Monday morning — the v4 "market closed" demand was inverted.)
5. **Broker record reachable read-only** today's account file exists with ≥1 frame after 09:00 (from the tab load). Missing =
   tab not signed in → (a)/(c1) will fail; record and stop.
6. **R11 CME-limit record** — autonomous-path prerequisite (Lead ruling 2026-11-01: trading-lead owns it; produced **before**
   the 09:15 attestations). This is an **additional evidence artefact**, not one of the five attestations and not a change to
   runbook v7 / P0.1. Run:
   `bash board/probes/team-c/cme-limit-record/fetch-cme-limits.sh --day 2026-11-02 --out board/probes/team-c/monday-2026-11-02/cme-limit-record.md`
   (provenance = exchange pages: CME Price Limits U.S. equity-index rule + CME Fixing Prices ES Dec 2026; validated Sunday
   2026-11-01 14:18Z, status SET). Exit 0 = SET (record + `cme_limits-2026-11-02.json`); exit 2 = **UNSET → quote the reason
   in the guard; for the future autonomous bot UNSET = NO TRADE**. Monday is cards-only, so there is no trading consequence
   today — but the SET/UNSET line and its cause go in the report. Method: `board/probes/team-c/cme-limit-record/README.md`.

## 09:15 ET — PREFLIGHT: five attestations, each stated individually with its own evidence  [owner trading-lead; backup lead]
1. **(a) Broker session live + equity from a verified record.** `live/account_state.json`: `status=="attached"`,
   `source=="live"`, account `71223147` present, `acct_t` after today 09:00 ET, `amount`/`amountSOD` quoted. `tradeDate`
   still `2026-10-26` is the venue's stale field — record the anomaly, do not fail on it alone.
2. **(a2) Account FLAT — positively exercised or UNVERIFIED.** `python3 -I board/probes/team-c/dom_read_positions.py --out
   board/probes/team-c/monday-2026-11-02/positions-panel.txt`. Exit **0** + a **positive** flat statement = PASS; exit
   **3/4**, empty or ambiguous = **(a2) UNVERIFIED → the day does not start** (no improvising a page read on a Tier S day).
   Alternative accepted proof: a `position` frame in today's account file showing zero positions (none has ever appeared).
3. **(b) Engine heartbeat current.** `age_s < 60`, `status= running`, `mode= cards`, `qty= 1`, `contract_id= 4470322`.
4. **(c1) Tab + feed + bar alive — PROVISIONAL at 09:15.** `feed` not "retrying", `feed_age_s < 30`, last bar age ≤ 120 s.
   The **binding** proof is (c2) after the open.
5. **(d) Engine ARMED + rails asserted from the actual config + contract id + systemd start.** `ActiveState=active`;
   `RAILS_ASSERT: PASS` (mode cards, 1 contract, 2 trades/day, -400/+900, cooldown 15, flat 16:05, min stop 2.0,
   symbol MESZ6, contract_id 4470322); `CONFIG_OLDER_THAN_PROCESS: PASS`; heartbeat `paused=False status=running`.
   Day-filtered start card quoted if the engine was restarted today.
6. **(e) Execution mode in the actual config — STATE IT HONESTLY: `mode= cards` = notify card + HUMAN-placed bracket;
   NOT automated.** There is no autonomous execution mode; never claim otherwise. **Decision-log ARMED** = ledger dir
   writable + engine active + yesterday's ledger quoted (or documented empty). **PROVEN ON** = today's ledger exists after
   the first signal with ≥1 record matching the first card; no signal by 16:05 = declared **PASS-empty**.

**Any FAIL/UNVERIFIED anywhere above → the 09:45 window does not open; name the failing leg and stop.**

## 09:35–09:44 ET — BINDING (c2): a fresh bar AFTER the 09:30 open  [owner trading-lead; backup lead]
Run the runbook's computed block (its PASS formula contains the `et >= "09:30"` term): `feed` not retrying,
`feed_age_s < 30`, bar age ≤ 120 s, `last_bar_ET >= 09:30`. Quote the bar's `ts`, ET time, `src`. FAIL or no post-09:30 bar
→ record, do not open the window.

## ~09:40 ET — MODE STEP + RE-VERIFY  [owner trading-lead; backup lead]
- `mode=cards` already → **no switch**; re-run (b), (c2), (d), (e) and write "re-verified 09:40, mode cards".
- `mode=shadow` → the **only legal switch** is to `cards` (edit config, restart `openbell-live`, quote the new start card,
  re-run all attestations). **There is no switch to an autonomous mode**; building one is the Tier S successor, never a
  Monday-morning edit. Any other value = the engine refuses to start = day stops.

## 09:45 ET — WINDOW OPENS  [owner trading-lead; backup lead]
ORB15 fires 09:46–12:00 only. Log every card (day-filtered) into `board/probes/team-c/monday-2026-11-02/cards.md`; feed
STALE pauses entries; no forcing; poll heartbeat every 15 min.

## 16:05 ET — FLAT (freshness required; two accepted proofs)  [owner trading-lead; backup lead]
1. **Frame proof:** the last `cashBalance` frame today has `t` **after 16:00** AND engine `pos: null`.
2. **Panel proof:** run the read-only positions read at ~16:02; accept only a file written **at or after 16:00** carrying a
   **positive flat statement**. Last resort `--reload` only if the page is stale, never with a position open (may raise FEED
   STALE).
**The conditional pass cannot mask non-flatness:** any position or ambiguity = **FAIL (non-flat/unmanaged)**, never "flat".
Neither proof → record "unmanaged/unverified". The firm's 16:10 auto-close is never the flatten.

## 16:20 ET — OUTCOME CHECK: decision-log completeness is the HEADLINE  [owner trading-lead; backup lead]
1. Day-filtered signal count (`wc -l live/ledger/2026-11-02.jsonl`; missing = 0 signals, say so).
2. **HEADLINE — completeness PASS/FAIL:** every signal has (i) a day-filtered card and (ii) an outcome (fill/cancel/skip).
   PASS = zero gaps; FAIL = list every gap by signal; 0 signals = **PASS-empty** ("no signals today, no completeness claim").
   Limitation stated: internal consistency only — a signal the engine never logged cannot be detected.
3. Day-filtered cards quoted (`cards_today=N`; the file is cumulative, the day filter is mandatory).
4. Broker record: today's last `cashBalance` frame (amount, realizedPnL, tradeDate); any fill must appear in **broker frames**,
   never only in the engine ledger.
5. **Friday's missing post-close reconciliation** (owed): last balance frame Fri `2026-10-30T15:29:12 ET`
   (50000/50000/0.0, tradeDate 2026-10-26, venue stamp `2026-10-25T21:30:13.058Z`); after it the broker state is unverified
   (17:52:13 `dom` with equity 0.0 is not a balance). Verdict: broker flat as of 15:29 / card-only engine book.
6. Rails accounting: ≤2 trades; no -$400/+$900 touch; cooldown honoured; flat by 16:05; $1,000 firm cap and 40% consistency
   are firm-side (checked, not engine-enforced): state realised P/L and the consistency denominator.
- **A Monday 16:20 may not claim "autonomous fills"** (D14/X1): the honest outcome is "cards day (human-tap)", "no trade",
  or "failed at gate X". The definition of an autonomous-fill claim is design §10 and requires venue record + 1:1 joins +
  complete decision log + executor hash chain + rails accounting + attribution.

## FAILURE DISPOSITION / FALLBACK  [owner trading-lead; backup lead]
- **Only verified sign-in route = Cayleb's desktop session on the existing Chromium tab.** A fresh credential or 2FA demand =
  **STOP FOR CAYLEB**, never improvise. Nox→Outis/Codex is UNVERIFIED (Outis offline 151 d) — never present it as a route.
- If the desktop route fails: record `tabkeeper.json`, last account frame, engine log line, heartbeat feed/feed_age; declare
  **BLOCKED-ON-MACHINE / no trade**; decision log declared empty (not "complete"); no other host, no shadow substitute, no
  manufactured fill; escalate to the Lead. The smallest Cayleb act is "look at the Tradovate screen and confirm flat",
  recorded as his words.
- If a unit is dead: journal exit code first; exit 2–6 = needs a human, not a blind restart.

## Owners / backups (pinned)
| ET | Leg | Owner | Backup | Fallback |
|---|---|---|---|---|
| 01:00 | Kalshi resume + laptop lane check | trading-lead | lead | lane stays dark, record failing unit + exit code |
| 07:45 | loose-ends sweep + runner liveness | upkeep-lead | lead | partial coverage reported honestly |
| P0 / 09:05 | preflight + session guard | trading-lead | lead | any FAIL/UNVERIFIED = window does not open |
| 09:15 | five attestations | trading-lead | lead | failing leg named; (a2) can end the day |
| 09:35–09:44 | binding (c2) | trading-lead | lead | no post-09:30 bar = no window |
| ~09:40 | mode step + re-verify | trading-lead | lead | cards stays cards; no autonomous switch |
| 09:45–16:05 | window inside rails | trading-lead | lead | feed STALE pauses entries; no forcing |
| 16:05 | flat proof | trading-lead | lead | unmanaged/unverified, never "flat" |
| 16:20 | outcome check | trading-lead | lead | PASS-empty allowed; no manufactured activity |
| Tue 3–Wed 4 | Kalshi freeze (pre-freeze checker slot first) | trading-lead | lead | freeze refuses while prereg slots unfilled |

## What Monday cannot claim (stated for the checker)
- It cannot be autonomous (no order path; `cards` only) — "first autonomous fills" is the successor build's claim.
- Completeness is internal consistency only (a signal never logged cannot be detected).
- It cannot prove flat without a positive position surface — which is why (a2) can block the day.
- It cannot prove a Tradovate session before the tab loads; (a)'s `status=="attached"` + fresh `acct_t` is the bar.

> **v7 UPDATE (2026-11-02T00:10Z, Lead):** the runbook is **v7**, 407 lines, sha256 `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a`; the P0.1 signer is
> `checker-trading` (fresh Tier S) and its signoff is `board/reports/monday-runbook-v7-checker-signoff.md`, copied to
> `board/probes/team-c/monday-2026-11-02-precheck-signoff.md` (the void v6 signature is preserved as
> `…-precheck-signoff-v6-VOID.md`). **Operational precondition:** reload the Tradovate page once (F5 only, no sign-in)
> after 09:00 ET so attestation (a)/09:05 item 3 read a fresh page load; otherwise those legs are UNVERIFIED and the
> gate stops the day (safe).
