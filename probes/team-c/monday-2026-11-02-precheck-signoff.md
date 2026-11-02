# P0.1 — CHECKER VETO SIGNATURE — MONDAY FUTURES RUNBOOK **v7** (Tier S, Team C)

**SIGNED — P0.1 GRANTED for the exact text below.**

- **Signer:** `checker-trading` — fresh-context Tier S checker. **I made none of this**: not the runbook, not the evidence,
  not the desk, no prior involvement in v4–v7. Report to the Lead only.
- **Signed at:** **2026-11-01 19:08 ET (EST, UTC−5) = 2026-11-02 00:08Z** — before the 09:05 ET Monday deadline.
- **Document signed (pinned):** `board/reports/monday-futures-runbook.md`, **v7**, 407 lines,
  sha256 **`512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a`**
  (computed by me with `sha256sum`; it matches the claimed hash). Any edit to that file voids this signature — it is a
  signature on this exact text, not on the path.
- **v6 relationship (the gap this signature closes):** v6 was signed by `trading-checker` at 2026-11-01T00:12Z on
  sha256 `027ca4e4cc1427e3300e2d36e0eda4380bfb3d8410c32ba8847533347adb4eff` (320 lines; preserved byte-identical at
  `board/probes/team-c/runbook-v6-signed-copy-027ca4e4.md`, hash recomputed by me). v7 is a different text, so the v6
  signature is **VOID for v7**. This is the fresh pre-check signature required before Monday's irreversible steps.

## 1. Verification commands and results (all run by me, read-only)

| # | Check | Command | Result |
|---|---|---|---|
| 1 | v7 hash | `sha256sum board/reports/monday-futures-runbook.md` | `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a` — **MATCH** |
| 2 | v6 preserved copy | `sha256sum board/probes/team-c/runbook-v6-signed-copy-027ca4e4.md`; `wc -l` | `027ca4e4…` **MATCH** the v6 signoff's pinned hash; 320 lines |
| 3 | v6→v7 diff | `diff -u` v6-copy v7 | 151 diff lines. **Additions only**, plus exactly three replaced blocks: header (V7-1), the P0.1 row, and 16:20 item 5's first line (V7-7). Every removed line is one of those documented edits — no undiffed change |
| 4 | No attestation command rewritten | extracted all 11 fenced blocks and all 18 command lines from both files, compared | **identical, byte-for-byte** → the v7 claim "No attestation command changed from v6" is **true** |
| 5 | P0.2 browser route | `ss -ltnp \| grep :9222`; `cat live/logs/tabkeeper.json` | chromium listener on `127.0.0.1:9222` (pid 2960519); `tradovate: "ok"` at 18:54:49 ET |
| 6 | P0.3 services | `systemctl --user is-active openbell-live openbell-dash openbell-tabkeeper` | `active` / `active` / `active` |
| 7 | P0.4 no carry-over | heartbeat `pos`/`paused` | `pos= None paused= False` |
| 8 | 09:05 #1 clock | `date; date -u; TZ=America/New_York date` | EST (−0500); UTC exactly +5 h |
| 9 | 09:05 #2 engine alive | runbook heartbeat one-liner | `age_s= 4.9 status= running paused= False` |
| 10 | 09:05 #3 broker record | runbook python block on `live/ledger/account-2026-11-02.jsonl` | executes: `frames 127 cashBalance_frames 0` → prints `NO cashBalance frame today -> broker state UNVERIFIED` (correct tonight; precondition §5) |
| 11 | 09:05 #4 Globex alive | `market_open` + clock | `market_open= True`; clock term evaluates at Monday 09:05 (run now 19:05, so the compound check is for the day, not tonight) |
| 12 | (a) broker session | runbook (a) python block | `status= attached`, `source= live`, `accounts= ['71223147']`; `acct_t= 2026-11-01T13:59:02-05:00`; `amount= 50000.0 SOD= 50000.0 realizedPnL= 0.0 tradeDate= {2026,10,26}` — the tradeDate anomaly is recorded as the runbook requires; `acct_t` is stale for Monday (precondition §5) |
| 13 | (a2) flat read | read `board/probes/team-c/dom_read_positions.py` (mtime 2026-11-01T00:02Z) | read-only by construction: `GET /json/list` + one `Runtime.evaluate` of `document.body.innerText`; **no `Input.*`, no clicks, no typing, no navigation**; `Page.reload` only behind `--reload`; exit 3/4 = UNVERIFIED. Not executed by me (it writes an output file outside my write scope and reads the live page) |
| 14 | (b) heartbeat | runbook (b) one-liner | `age_s= 3.8 status= running mode= cards qty= 1 contract_id= 4470322` |
| 15 | (c1) provisional | runbook (c1) block | `feed= ws`, `feed_age_s= 0.5`, last_bar 19:04 ET, `bars_rows_today=66` → `C1: PASS` (executes) |
| 16 | (c2) binding | runbook (c2) block | `C2_BINDING: PASS` — last bar ET 19:04 ≥ 09:30 and age ≤120 s; the `et >= "09:30"` term is inside the computed PASS formula (executes) |
| 17 | (d) rails + binding | runbook (d) blocks | `RAILS_ASSERT: PASS` (10/10: mode cards, contracts 1, 2 trades, −400/+900, cooldown 15, flat 16:05, min stop 2.0, MESZ6, 4470322); `ActiveState=active`; `config_mtime=1793141014 (2026-10-27T22:43:34Z) < start=1793216409 (2026-10-28T19:40:09Z)` → `CONFIG_OLDER_THAN_PROCESS: PASS`; heartbeat `paused=False status=running` |
| 18 | (e) mode | `json.load(config)['mode']` | `mode= cards` |
| 19 | Day-filtered cards | runbook (d) card block | `cards_today= 0` (executes) |
| 20 | 16:05 flat block | runbook 16:05 block | executes: `NO cashBalance frame today -> flat UNVERIFIED` (correct tonight; the ~16:02 panel proof is the designed alternative on a quiet day) |
| 21 | 16:20 #1 signals | `wc -l live/ledger/2026-11-02.jsonl` | file absent = **0 signals**, declared (the ledger is created lazily) |
| 22 | Friday reconciliation | `board/reports/team-c-friday-reconciliation-20261030.md` | exists; last cashBalance 2026-10-30T15:29:12 ET (50000/50000/0.0, tradeDate 2026-10-26, venue stamp 2026-10-25T21:30:13.058Z); last Friday frame is a `dom` 17:52:13 (equity 0.0, not a balance); engine book 1 card trade +$102.50, card-only — matches v7 item 5 |
| 23 | Tonight's baseline §0.1/§0b | `board/probes/team-c/tick-proof-20261101T2343Z.md`, `team-c-globex-open-outcome-20261101.md` | first post-open bar 23:00:00Z = 18:00:00−05:00, n=815, v=3219, src tv-ws; feed `ws` 0.1–0.8 s; `market_open=true`; heartbeat 0.16 s; day roll `session day 2026-11-02: pnl 0.00 trades 0` at 18:01:00 ET; account attached/live $50,000 flat, `positions {}`, `dom.open_pnl 0.0`. I re-counted the bar file: 43 bars in 23:00:00Z…23:42:00Z inclusive, all `tv-ws` (see §6 for the one wording nit) |
| 24 | ORB15 RTH anchor | `bt/features.py:23-24,72`; `bt/strategies.py` class ORB | `RTH_OPEN=9*60+30`, `RTH_CLOSE=16*60`; ORB `on_bar` returns `[]` when `not f.rth`; config `n=15`, `last_entry="12:00"` → RTH-anchored, **does not map to the 18:00 ET Globex open**; `team-c-sunday-session-ruling-20261101.md` rules "IT DOES NOT MAP — MONDAY STANDS" |
| 25 | ORB15 role note still unruled | `live/config.json` `strategies[0].role`; latest Nox filings | role verbatim: "shadow plumbing check only: over 2016-2024 WF-OOS this config lost $4.31/trade, PF 0.81, 0 of 9 years positive … **do NOT trade its cards**"; ruling still open at signoff (Nox filings 20261102T000200Z/000500Z: "ORB15 ruling still the single fastest unlock/close for Monday's Layer B"; BOARD.md open asks) |
| 26 | G2 dry-run only; G3/G4 unsigned | `live/exec` git/HEAD/config; G2 outcome + G2.5 plan | HEAD `0893497`, `exec_config.json mode="dry-run"`, no `state/enabled.json`, no `state/` dir, README "Disabled by construction and cannot send"; G2 signed **with conditions, dry-run only**; G2.5 = plan only; **G3/G4 unsigned** (no G3/G4 signature artefacts) |
| 27 | Engine has no order path | `live/engine.py:2`; mode comment | "shadow ledger + ntfy phone cards. It **NEVER places, modifies or cancels orders**"; `mode` = `shadow` (informational) or `cards` (actionable = notify + human bracket) |
| 28 | Rails / no wallet / no credential | v7 text + `live/config.json` + `beta.rails` | rails line intact (see §3); **no wallet instruction** in v7; **no credential value** — the only credential mention is the stop-for-Cayleb rule ("a fresh credential or 2FA demand = stop for Cayleb") |

## 2. The five 09:15 attestations (individually stated in §0b; each has its own command and PASS bar)

- **(a) Broker session live + equity from a verified record** — source `live/account_state.json`, (a) python block;
  PASS ⇔ `status=="attached"`, `source=="live"`, account `71223147`, `acct_t` after today 09:00 ET, `amount`/`amountSOD`
  quoted; `tradeDate` anomaly recorded if still 2026-10-26. Actionable and falsifiable.
- **(a2) Account FLAT — positive proof, not `positions {}`** — positions-panel text over CDP via
  `board/probes/team-c/dom_read_positions.py` (exit 0 + positive flat statement) **or** a zero `position` frame; ambiguous
  = UNVERIFIED → day blocked by design. Actionable (exact command + output file) and falsifiable.
- **(b) Engine heartbeat current** — `age_s<60`, `status=running`, `mode=cards`, `qty=1`, `contract_id=4470322`.
- **(c1) Tab + feed alive (provisional at 09:15)** — `feed` not retrying, `feed_age_s<30`, a bar within 120 s; then
  **(c2) BINDING** post-09:30 bar at 09:35–09:44, with `ET >= "09:30"` inside the computed PASS formula. No window
  without it.
- **(d) Engine ARMED + rails asserted from the ACTUAL config + config bound to the running process** —
  `ActiveState=active`, `RAILS_ASSERT: PASS`, `CONFIG_OLDER_THAN_PROCESS: PASS`, heartbeat `paused=False status=running`.
- **(e) Execution mode** — verbatim: *"(e) mode = cards — notify card + human-placed bracket; NOT automated (the engine
  has no order path; Lead ruling X1)."* `cards` only; `shadow` → the mode step; anything else stops the day.
- **Gate:** the GATE RULE (D13) says any attestation **FAIL or UNVERIFIED = the day does not start (the 09:45 window is
  not opened)**, failing leg recorded. §0b restates it: "Any attestation FAIL or UNVERIFIED = the window does not open."

## 3. Rails checklist — encoded correctly and never softened

| Rail | Encoded in v7 | Verified against live config / evidence |
|---|---|---|
| −$400 daily stop | GATE RULE rails line; (d) assert | `daily_loss_stop_usd = -400`; engine `gate()` "daily loss stop hit" |
| +$900 profit stop | GATE RULE rails line; (d) assert | `daily_profit_stop_usd = 900`; engine `gate()` "daily profit stop hit" |
| $1,000 single-day firm cap | declared firm-side, checked not enforced, at 16:20 | engine DEFAULTS comment "stays under the $1,000 consistency cap (40% of $2,500)"; `exec_config.json` `daily_consistency_cap_usd: 1000` (dry-run tree) |
| 40% consistency | declared firm-side, checked not enforced, at 16:20 | same evidence |
| flat 16:05 ET | GATE RULE; 16:05 section | `flat_et = "16:05"`; engine `gate()` blocks entries 16:05–18:00 |
| beta 1 MES | GATE RULE "1 MES" | `contracts = 1`; `beta.rails`: "1 MES, daily loss stop -$400 and daily profit stop +$900 for the whole book, 15-min cooldown after a loss, flat by 16:05" |
| max 2 trades/day | GATE RULE | `max_trades_per_day = 2` |
| 15-min cooldown after loss | GATE RULE | `cooldown_min_after_loss = 15`; engine `gate()` |
| no wallet ops | no wallet instruction anywhere in v7 | — |
| no credential display | no credential value anywhere in v7 | only the stop-for-Cayleb rule |
| engine self-report is never account evidence | GATE RULE | matches the broker-record design (`live/account.py` passive watcher) |

The v7 diff touches **none** of the rail text; the rails are identical to the v6 text the previous checker signed.

## 4. Kill/stop conditions and the honest failure path (verified)

- **GATE RULE:** any FAIL/UNVERIFIED = the day does not start; no partial pass, no "proceed and watch", no substitute data.
- **§0c fallback:** if the desktop route fails (no signed-in tab / no fresh feed / no fresh account frame) → record
  **BLOCKED-ON-MACHINE / no trade**, name the failing attestation(s), decision log empty (never "complete"), escalate via
  the Lead. **No sign-in improvisation**; the only verified sign-in route is Cayleb's desktop session; a fresh credential
  or 2FA demand = stop for Cayleb. "There is no other host, no shadow substitute, no manufactured fill."
- **Absent ORB15 ruling:** the day runs observation-only — guard, five attestations, binding, cards logged, 16:05 flat
  check, 16:20 outcome with **0 fills claimed**.
- **Nothing claims an order-capable session:** §0b(e), GATE RULE, §0c and §09:45 all state `mode=cards` = notify +
  **human-placed** bracket, engine has no order path, "no autonomous fill is possible or claimed".

## 5. Explicit limits of this signature (what it does NOT cover)

1. **It does not pre-pass any Monday attestation.** Each of the five must produce its own fresh command output on the
   day, or the gate stops the day. It does not assert the account is flat — (a2) does that on the day, positively, or the
   day is blocked.
2. **ORB15 human-tap Layer B stays BLOCKED** absent the Nox/Cayleb ruling on the "do NOT trade its cards" role note
   (still open at signoff). Without it, Monday is observation-only with 0 fills claimed; the runbook cannot create that
   ruling.
3. **Autonomous P2 cannot land before Monday.** The engine has no automated mode (`shadow|cards` only; live `mode=cards`),
   G2 is dry-run-only (no enable artefact, no CDP transport, no order path), G2.5 is a plan only, G3/G4 are unsigned. No
   autonomous fill is possible or claimed; the earliest honest target is a later session.
4. It does **not** cover wallet operations, credential handling, the Kalshi flip/cutover, or any host action. It is valid
   for **Monday 2026-11-02 only**; any edit to the runbook voids it.
5. **Operational precondition (not a change to any PASS bar):** the 09:05 item 3 and (a) fresh-snapshot legs need a page
   load on Monday after 09:00 — Tradovate sends the account snapshot only at page load and `cashBalance` frames only on
   load/change (`live/account.py` docstring; at signoff `sync_t` is Sun 13:59 ET). The tab is signed in now; if it is
   still signed in Monday morning it must be **reloaded once (F5 only — the documented allowed action, no sign-in, no
   credential; `reports/DASH-LIVE-AUDIT-2026-10-27.md`)** after 09:00 before running item 3/(a), else those legs read
   UNVERIFIED and the GATE RULE stops the day (safe, but the observation day is lost). If the tab is logged out, stop for
   Cayleb — no sign-in improvisation.
6. **P0.1 path note:** this signoff is filed at `board/reports/monday-runbook-v7-checker-signoff.md` (per the Lead's
   instruction). The runbook's P0.1 row names `board/probes/team-c/monday-2026-11-02-precheck-signoff.md`, which today
   holds the **void v6** signature. Before 09:05 the Lead must place this v7 signoff at that path (preserving the v6 file
   under a v6-named copy) or the literal P0.1 check will read the v6 hash and fail. Do **not** edit the runbook — that
   would void this signature.
7. **Packet note:** `team-c-monday-packet-v7.md` and `MONDAY-PACKET.md` still reference runbook **v6** / `027ca4e4…` at
   their 09:05/P0.1 lines. The runbook v7 overrides, but the Lead should correct the packet text (not the runbook) before
   09:05 so the operator does not check the wrong hash.

## 6. Named non-material discrepancies (named for the record; they alter no PASS bar, command, rail, or gate)

1. **§0.1 "43 further minute bars through 23:42Z":** the probe file shows **43 bars at/after 23:00:00Z inclusive**
   (23:00→23:42; 42 after the first). The tick-proof PASS stands (first bar n=815, 43 minute bars, feed `ws`). Correct
   reading: "43 minute bars through 23:42Z (the first at 23:00)" / "42 further". Not corrected here because any edit voids
   P0.1.
2. **(e) "Anything else: the engine refuses to start":** the engine has **no mode validation** (`live/engine.py`
   `load_cfg` / `Notifier`: any value other than `shadow` is treated as actionable, so an unknown mode would start). The
   operational rule in the same sentence — "anything else: the day stops" — is correct and conservative, and the (e) PASS
   bar is `cards` only. Inherited from v6; recommend fixing the wording in a later revision.
3. **16:20 item 3 parenthetical "26 tonight vs 2 on Friday":** the cumulative file has **26** cards (verified); the
   day-filtered count for Fri 2026-10-30 is **6** cards (2 actionable), so the "2" matches the actionable subset, not the
   `cards_today` count. The command day-filters and the operator quotes `cards_today=N`, so the criterion is unaffected.

## 7. Fences held

Read-only. I made none of the runbook or evidence. No live action, no order, no wallet op, no engine config/code change,
no credential value, no image read/screenshot, no host input injection, no spend ($0). My only write is this signoff file.

---

**— `checker-trading`, 2026-11-01 19:08 ET (EST, UTC−5) = 2026-11-02 00:08Z.**
**P0.1 GRANTED for runbook v7 @ `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a`, Monday 2026-11-02.
The v6 signature (`027ca4e4…`) is void for this text.**
