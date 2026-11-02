# MONDAY FUTURES DAY — 2026-11-02 — EVIDENCE SLOTS (`guard.md`)

Owner: `trading-lead` (Team C). Backup: `lead`. Tier S.
**Runbook of authority:** `board/reports/monday-futures-runbook.md` **v7**, 407 lines,
sha256 `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a`, **P0.1 SIGNED** by `checker-trading` at
2026-11-01 19:08 ET (signoff: `board/reports/monday-runbook-v7-checker-signoff.md` and
`board/probes/team-c/monday-2026-11-02-precheck-signoff.md`; void v6 signature kept at `…-precheck-signoff-v6-VOID.md`).
**ANY edit to the runbook voids P0.1 — do not edit it.** This file is the day's evidence log, not a rulebook.

Rules for this file: append every line + its command output under the matching section; put the time (`ET`) and the seat
(owner/backup). A leg is only PASS if its own command output is quoted here. **FAIL or UNVERIFIED = the 09:45 window does
not open** (gate rule). Absence of a record is not a pass.

Pre-staged helpers (created 2026-11-01 ~19:20 ET, dry-checked only — not executed):
- `runner.sh` — read-only operator runner (`bash runner.sh <step>`; `--help`; dry-checked with `bash -n`).
- `page-reload.py` — the one allowed page reload (CDP `Page.reload`, no sign-in, no navigation). Staged, not executed.
- `orb15-branches.md` — the two ORB15 operator notes (Layer B tap vs hold → observation-only).

---

## 0. PRE-GUARD — 09:00–09:05 ET (do these before item 1 of the guard)

### 0.1 P0.1 hash re-verify (FIRST COMMAND OF THE DAY)
- [ ] `sha256sum board/reports/monday-futures-runbook.md` → **must equal**
      `512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a`; `wc -l` = **407**.
      runner: `bash runner.sh hash`
- [ ] Time (ET) + result: ______________________________________________
- **If it differs:** STOP. The day does not start; report to the Lead immediately; a re-sign on the new text is required
  before any further step. Do **not** edit the runbook to "fix" it. *(Lead's standing addition, received 2026-11-01
  19:20 ET: "if the runbook hash differs at 09:00 … report immediately — the day does not start until a re-sign; do not
  edit the runbook." Recorded here verbatim in substance.)*

### 0.2 Page load precondition (checker-trading signoff §5)
The 09:05 item 3 and (a) fresh-snapshot legs need a Monday page load after 09:00: Tradovate sends the account snapshot
only at page load, and `cashBalance` frames only on load/change. The tab is signed in now; reload it **once, F5 only —
never a sign-in, never a credential** — after 09:00 ET and before the guard.
- [ ] Reload performed by: ______________________ — **MECHANISM APPROVED BY THE LEAD 2026-11-01 19:20 ET: the desk's own
      CDP `Page.reload` via `page-reload.py` (F5-equivalent, documented allowed action
      `reports/DASH-LIVE-AUDIT-2026-10-27.md:104`).** One reload only, on the already-signed-in tab; exit 3 (no tab) /
      logged-out / CDP failure = STOP for the Lead/Cayleb, never a sign-in.
- [ ] Reload time (ET): ____________ ; command: `python3 -I page-reload.py --out page-reload.json` (staged; exit 0 =
      reloaded and the tab is still `trader.tradovate.com`; exit 3 = no tab → stop for the Lead/Cayleb, never sign in).
- [ ] Post-reload verification (all three, quoted):
      a) `live/logs/tabkeeper.json` → `tradovate: "ok"`, `alerted: false`;
      b) `live/heartbeat.json` → `feed` not retrying, `feed_age_s < 30`, `paused: false` (a reload can emit a FEED STALE
         card; entries pause and recover — record any card by time, do not "fix" it);
      c) `live/account_state.json` → `frame_t` age < 60 s and `status: "attached"`, `source: "live"`.
- **If a reload fails to re-attach the feed, or the tab is logged out: STOP for the Lead/Cayleb — no sign-in
  improvisation.** Item 3/(a) would read UNVERIFIED and the gate stops the day.

### 0.3 P0.2 / P0.3 / P0.4 (runbook v7 P0 table)
- [ ] P0.2 browser route: `ss -ltnp | grep :9222` → Chromium listener; `cat live/logs/tabkeeper.json` → `tradovate` not
      `missing`.  result/time: ______________________________________
- [ ] P0.3 services: `systemctl --user is-active openbell-live openbell-dash openbell-tabkeeper` → three `active`.
- [ ] P0.4 no carry-over: heartbeat `pos= None paused= False`.
- runner: `bash runner.sh pages`

## 1. 09:05 ET — SESSION GUARD (five items; all five or FAIL)
- [ ] **1 Clock/TZ:** `date; date -u; TZ=America/New_York date` → ET shows **EST (−0500)**, `date -u` differs by exactly 5 h.
      result/time: ______________________________________
- [ ] **2 Engine alive:** heartbeat one-liner → `age_s<60 status= running paused= False`.  result/time: ______________
- [ ] **3 No carry-over from the broker record:** `cashBalance` frame today **with its `t` quoted**.  result: __________
- [ ] **4 Globex alive, RTH not yet:** `market_open= True` **and** clock `< 09:30`.  result/time: ______________________
- [ ] **5 Broker record reachable read-only:** today's `live/ledger/account-2026-11-02.jsonl` exists with ≥1 frame after
      09:00 (the tab load).  result: ______________________________________
- runner: `bash runner.sh guard` → each item prints; quote all five above.
- **Guard verdict:** PASS / FAIL(leg: ______) at ____:____ ET.

## 2. 09:15 ET — PREFLIGHT: the five attestations (each stated individually)
### (a) Broker session live + equity from a verified record
- [ ] `bash runner.sh a` → `status= attached | source= live | accounts= ['71223147'] | acct_t= <today after 09:00> |
      amount= 50000.0 SOD= 50000.0 realizedPnL= 0.0 tradeDate= {2026,10,26}` (tradeDate anomaly recorded, not smoothed).
- Verdict: PASS / FAIL ☐ ; output quoted: ______________________________________________

### (a2) Account FLAT — positive plate proof (positions panel over CDP), not `positions {}`
- [ ] `bash runner.sh a2` → writes `positions-panel.txt` and prints the panel text + `grep -c '"kind": "position"'`.
- PASS ⇔ exit 0 **and** a positive flat statement (empty positions table / "No positions" / 0 open). Ambiguous = UNVERIFIED
      → day blocked by design.  exit code: ______ ; flat statement quoted: ______________________________
- Verdict: PASS / UNVERIFIED ☐

### (b) Engine heartbeat current
- [ ] `bash runner.sh b` → `age_s<60 status= running mode= cards qty= 1 contract_id= 4470322`.
- Verdict: PASS / FAIL ☐ ; output: ______________________________________________

### (c1) Tab + feed alive — PROVISIONAL at 09:15 (binding is (c2))
- [ ] `bash runner.sh c1` → `C1: PASS` (feed not retrying, `feed_age_s<30`, a bar within 120 s).
- Verdict: PASS / FAIL ☐ ; `last_bar=` quoted: __________________

### (d) Engine ARMED + rails asserted from the ACTUAL config + config bound to the running process
- [ ] `bash runner.sh d` → `RAILS_ASSERT: PASS` (10/10) **and** `CONFIG_OLDER_THAN_PROCESS: PASS` **and**
      `paused=False status=running`.
- Verdict: PASS / FAIL ☐ ; both lines quoted: ______________________________________________
      (If `CONFIG_CHANGED_AFTER_START: FAIL` → restart the engine and re-run **all five** attestations.)

### (e) Execution mode — verbatim text
- [ ] `bash runner.sh e` → `mode= cards`. Attestation text, verbatim: **"(e) mode = cards — notify card + human-placed
      bracket; NOT automated (the engine has no order path; Lead ruling X1)."**
- Verdict: PASS / FAIL ☐ (`shadow` → mode step; anything else → day stops)

### Decision logging — ARMED (not "ON")
- [ ] `bash runner.sh armed` → `WRITABLE` + yesterday's ledger last record quoted (or documented empty).
      result: ______________________________________________

## 3. 09:35–09:44 ET — BINDING (c2): a fresh bar AFTER the 09:30 open
- [ ] `bash runner.sh c2` → `C2_BINDING: PASS` with `last_bar_ET >= 09:30` and age ≤ 120 s; quote the bar `ts`, ET and
      `src`.  result/time: ______________________________________________
- **No PASS ⇒ the window does not open.** (runner prints `FAIL — name the leg: tab/browser/feed/bar/bar-before-09:30`.)

## 4. ~09:40 ET — MODE STEP + RE-VERIFY (v7 expectation baseline)
- [ ] `bash runner.sh e` + `bash runner.sh b` + `bash runner.sh c2` + `bash runner.sh d` → re-read after 09:35; expected
      baseline: `mode=cards`, `status=running`, `qty=1`, `contract_id=4470322`, `paused=false`, `pos=null`.
- [ ] Write: "re-verified ____:____ ET, mode cards"; quote the (c2) bar.  result: ______________________________

## 5. 09:45 ET — WINDOW OPENS (observation; ORB15 branch decides the rest)
- [ ] All above PASS → window opened at ____:____ ET. **If any FAIL/UNVERIFIED, the window does NOT open.**
- [ ] ORB15 branch in force at 09:45: **H — HOLD / observation-only, 0 fills claimed — THIS IS NOW THE RULED STATE
      (not a default).** Source: **Nox 20261102T003600Z §1 — ORB15 HOLD (option ii)** (overnight drive run 2, dated
      2026-11-01 19:31 ET = 2026-11-02T00:36Z filing; relayed by the Lead 2026-11-01 19:43 ET). Verbatim in substance:
      *"Cards are observation-only until the record is re-cleared and the G3/G4 executor is built and signed; the
      tap-to-approve model is revoked; no deliberate-exception human-tapped live path on Monday."* → follow
      `orb15-branches.md` branch H. **Branch B is NOT available for Monday** (tap-to-approve revoked; the owner's model is
      autonomous execution inside the rails, and the record does not support an exception).
      **Re-clear path (future, not a Monday improvisation):** re-clearing ORB15 is a **Team C record question** brought back
      with evidence — the record re-cleared and a G3/G4 executor built and signed — never a Monday change of branch.
      ruling_on_record: Y (Nox 20261102T003600Z §1) ; time applied here: ____:____ ET
- [ ] Cards logged to `cards.md` (time, side, price, stop/target, tag, within 1.5 pts/2 min): ______________________
- [ ] 15-min heartbeat poll: ____:____ , ____:____ , ____:____ , ____:____ , ____:____ , ____:____

## 6. 16:05 ET — FLAT (freshness required; a stale frame is not a flat proof)
- [ ] `bash runner.sh flat` → proof 1: `cashBalance` frame `t` after 16:00 **or** proof 2: positions panel read at ~16:02
      (`positions-panel-1605.txt`, file written ≥16:00, positive flat statement).
- Verdict: PASS(proof 1/2) / UNVERIFIED → record "unmanaged/unverified", never "flat".  result: ______________________
- [ ] Engine `pos: null` quoted: ____________ ; any position/ambiguity = FAIL.

## 7. 16:20 ET — OUTCOME CHECK (decision-log completeness is the HEADLINE)
- [ ] 1 Signals: `wc -l live/ledger/2026-11-02.jsonl` (missing = 0, say so): ______
- [ ] 2 **Completeness PASS/FAIL (headline):** every logged signal has (i) a day-filtered card and (ii) an outcome
      (`fill`/`cancel`/`skip`). 0 signals → **PASS-empty** "no signals today, no completeness claim". gaps: ______
- [ ] 3 `bash runner.sh cards` → `cards_today=N` quoted: ______
- [ ] 4 Broker record: last `cashBalance` frame `amount`/`realizedPnL`/`tradeDate`: ______________________
- [ ] 5 Friday reconciliation: document `board/reports/team-c-friday-reconciliation-20261030.md`; Monday's fresh
      `cashBalance` frame closes the post-15:29 gap — quote it: ______________________
- [ ] 6 Rails accounting: ≤2 trades; no −$400/+$900 touch; cooldown honoured; flat by 16:05; $1,000 firm cap + 40 %
      consistency checked (firm-side) with day P/L and consistency denominator: ______________________
- [ ] **Outcome verdict:** ______________________ (observation-only / Layer B tap / BLOCKED-at-leg ______)

## 8. POST-DAY (after 16:20)
- [ ] N5 stop slot belongs to Team E (extension-host PID 2960775 only; Chromium 2960519 untouched) — not mine.
- [ ] G2.5 read-only capture (post-16:20, after N5, signed-in tab) — `board/probes/team-c/g2p5-2026-11-02/`.
- [ ] File this log + the day's one-page outcome in `board/reports/team-c-*.md`; report to the Lead.

— skeleton created trading-lead 2026-11-01T19:20 ET; slots stay empty until the day runs.

---

# RUN LOG — MONDAY 2026-11-02 (filled live by trading-lead; append-only)

## §0.1 P0.1 hash re-verify
- 09:00:35 ET — `cd board/probes/team-c/monday-2026-11-02 && bash runner.sh hash` → **FAIL — runbook path not resolved**
  (runner cwd defect, NOT a runbook change; `logs/0900-hash.log`). Named defect + fix: `PATH-BUG-FINDING.md`.
- 09:01:04 ET — hash re-run with corrected path resolution from `/tmp` (cwd-independent) → **PASS**,
  `runbook_sha256=512752e418a7950d2e43bed938f5716147494d74162dcbfbfba2a68389a2c86a`, `runbook_lines=407`.
- 09:03:31 ET — Lead re-ran the runner's hash step → **PASS** (`logs/0903-hash-rerun-PASS.log`).
- **09:05:35 ET — fixed runner, logged re-run → `runbook_sha256=512752e48…c86a`, `runbook_lines=407`,
  `HASH: PASS — P0.1 text is the signed text` (`logs/0905-hash.log`). Precondition satisfied by its own evidence.**
- Team E independently verified the artifact matches P0.1 (37,271 B, 407 lines, `512752e4…`) at 09:06 ET.

## §0.2 Page load (one reload, F5-equivalent, no sign-in) — per checker §5
- **09:01:31 ET — reload PASS.** Tab `69E2E1E83E14CBC452D216DA9244A63F`, `trader.tradovate.com`, before/after URL and title
  identical ("Tradovate - Dark Default"), `reload_reply_error: null`, no sign-in attempted (`page-reload.json`).
  Mechanism: the Lead-approved desk CDP `Page.reload`; note the staged `-I` invocation cannot import `websockets` (user-site,
  excluded by isolated mode) → executed with plain `python3` (same script/limits; deviation recorded in §7 of the day report).
- Post-reload checks (09:01:43 ET): tabkeeper `tradovate="ok" alerted=false`; heartbeat `feed="ws" feed_age_s=0.0
  paused=False`; account_state `status="attached" source="live" frame_t_age_s=0.1`; **`sync_t`/`acct_t` refreshed to
  `2026-11-02T09:01:36-05:00`**; **fresh `cashBalance` frame at 09:01:36** (amount 50000.0 / SOD 50000.0 / realizedPnL 0.0).
  `cards_today=0` → the reload produced no FEED STALE card.

## §0 P0.2 / P0.3 / P0.4 (09:01:54 ET)
- **P0.2 PASS** — `LISTEN 127.0.0.1:9222 users:(("chromium",pid=2960519,fd=96))`; tabkeeper `tradovate="ok"`.
- **P0.3 PASS** — `systemctl --user is-active openbell-live openbell-dash openbell-tabkeeper` → `active/active/active`.
- **P0.4 PASS** — heartbeat `pos= None paused= False`.

## §1 09:05 SESSION GUARD — **VERDICT: PASS (all five)** — run 09:05:05 ET (`logs/0905-guard.log`)
1. **Clock/TZ PASS** — `date` `Mon Nov 2 09:05:05 AM EST 2026`; `date -u` `02:05:05 PM UTC`; ET shows **EST (−0500)**,
   exactly 5 h difference.
2. **Engine alive PASS** — `age_s= 3.5 status= running paused= False`.
3. **No carry-over (broker record) PASS** — `frames 1126 cashBalance_frames 1`; **last cashBalance `t=2026-11-02T09:01:36.018187-05:00`
   amount=50000.0 SOD=50000.0 realizedPnL=0.0 tradeDate={2026,10,26}` (venue_ts 2026-10-25T21:30:13.058Z recorded, not
   smoothed) → fresh frame with its `t` quoted.
4. **Globex alive, RTH not yet PASS** — `market_open= True`; `et_now=09:05`; `RTH_NOT_YET: PASS (09:05 < 09:30)`.
5. **Broker record reachable read-only PASS** — `frames 1126 frames_after_0900 90`; last frame `09:02:29` (`contract`).
**Guard verdict: PASS at 09:05:05 ET — the day may proceed to preflight.**

## §2 09:15 PREFLIGHT — five attestations individually (run 09:15:05 ET)
| Leg | Verdict | Evidence (exact) |
|---|---|---|
| **(a)** broker session + equity | **PASS** | `status= attached \| source= live \| accounts= ['71223147']`; `acct_t= 2026-11-02T09:01:36.018187-05:00` (**after 09:00**); `amount= 50000.0 SOD= 50000.0 realizedPnL= 0.0`; `tradeDate= {2026,10,26}` recorded as the venue's stale field |
| **(a2)** account flat — as signed | **UNVERIFIED (signed command exit 4)** | `python3 -I dom_read_positions.py --out …` → `EVALUATE FAILED: No module named 'websockets'`, `a2_exit=4`. Cause: `websockets 10.4` is installed in the **user site** (`/home/cayleb/.local/lib/python3.14/site-packages`) and `python3 -I` (isolated: `-E -s`) excludes user site. The signed command cannot execute in this environment. |
| **(a2)** account flat — supplemental (SAME script, plain `python3`; deviation = interpreter flag only, recorded) | **positive flat evidence, NOT a signed-command pass** | `python3 dom_read_positions.py --out positions-panel.txt` → `exit 0`, 467 lines captured 2026-11-02T14:15:17Z. Panel: **`MESZ6 … POSITION 0 … -.-- USD`**, ESZ6/ZNZ6 `POSITION 0`; the **Positions table is empty** (headers `SYMBOL/NET POS/BOUGHT/SOLD/NET PRICE/OPEN P/L/INIT. MARGIN`, no rows); account header `EQUITY 50,000.00 USD`, `OPEN P/L 0.00 USD`. |
| **(b)** heartbeat current | **PASS** | `age_s= 3.1 status= running mode= cards qty= 1 contract_id= 4470322` |
| **(c1)** tab+feed alive (provisional) | **PASS** | `feed= ws \| feed_age_s= 0.3 \| last_bar_et= 09:14 \| bars_rows_today= 555`; last bar `2026-11-02T09:14:00-05:00`, age 65.1 s, `src= tv-ws`; `C1: PASS` |
| **(d)** ARMED + rails + config binding | **PASS** | `ActiveState=active`, `ExecMainStartTimestamp=Wed 2026-10-28 15:40:09 EDT`, `RAILS_ASSERT: PASS` (10/10 incl. mode cards, contracts 1, 2 trades, −400/+900, cooldown 15, flat 16:05, min stop 2.0, MESZ6, 4470322); `CONFIG_OLDER_THAN_PROCESS: PASS` (config 2026-10-27T22:43:34Z < start 2026-10-28T19:40:09Z); `paused= False status= running` |
| **(e)** execution mode | **PASS** | `mode= cards`, `heartbeat_mode= cards`. Verbatim: **"(e) mode = cards — notify card + human-placed bracket; NOT automated (the engine has no order path; Lead ruling X1)."** |
| decision-log ARMED | **ARMED** | ledger dir `WRITABLE`; engine active; no `2026-11-01` decision ledger file (documented empty — **not** "PROVEN ON") |
| cards today | 0 | `cards_today= 0` |

**Gate consequence (stated, not softened): one leg — (a2) — did not pass by its signed command. Under the runbook GATE
RULE ("a leg that cannot be proven is recorded as UNVERIFIED and treated exactly like a FAIL for the purpose of opening the
window") the 09:45 window is NOT opened by the operator. Escalated to the Lead for a ruling (options: checker-amended
command line pinning the same script without `-I`, or record the day as held at (a2)). The supplemental read is attached as
evidence of the underlying fact, not as a substitute pass.**

## §2b (a2) AMENDMENT EXECUTED — checker-trading ruling 2026-11-02 09:18 ET
- **Ruling:** `checker-a2-amendment.md` — (a2) AMENDMENT **APPROVED** as a recorded deviation (same reviewed script, only
  the interpreter `-I` flag dropped; runbook text NOT edited; P0.1 stays valid on `512752e4…`/407, re-verified 09:25 ET).
- **Signed-command status (unchanged):** `python3 -I dom_read_positions.py` → exit 4, `No module named 'websockets'` —
  recorded UNVERIFIED-by-signed-command; `websockets 10.4` is user-site and `-I` (`-E -s`) excludes user site (defect).
- **Amended (a2) executed 2026-11-02T09:39:50−0500** — `python3 board/probes/team-c/dom_read_positions.py --out
  …/monday-2026-11-02/positions-panel.txt` → **exit 0**, 484 lines captured (`14:39:50Z` header).
  **PASS bar unchanged — positive flat statement:**
  - `MESZ6 … POSITION / 0 / -.-- USD` (lines 275–277); `ESZ6 … POSITION 0 / -.-- USD`; `ZNZ6 … POSITION 0 / -.-- USD`;
  - **Positions table empty** — headers `SYMBOL / NET POS / BOUGHT / SOLD / NET PRICE / OPEN P/L, $` with no rows;
  - account header **`EQUITY 50,000.00 USD`**, **`OPEN P/L 0.00 USD`**.
  → **(a2) PASS under the checker's amendment.** No other script used; no substitution data.

## §3 09:35–09:44 BINDING (c2) — **PASS at 09:39:50 ET**
`feed= ws | feed_age_s= 0.0 | bars_rows_today= 579 | last_bar_ET= 09:38 | age_s= 110.1` → `C2_BINDING: PASS`
(fresh post-09:30 bar, age ≤120 s, `ET >= "09:30"` term true). Re-read 09:39:57: `age_s= 117.3` → **PASS** again.

## §4 ~09:40 MODE STEP + RE-VERIFY — **PASS at 09:39:57 ET**
- (b) `age_s= 4.0 status= running mode= cards qty= 1 contract_id= 4470322`
- (d) `ActiveState=active`; `RAILS_ASSERT: PASS` (10/10); `CONFIG_OLDER_THAN_PROCESS: PASS`; `paused= False status= running`
- (e) `mode= cards` (heartbeat_mode cards)
- (c2) re-read PASS (above)
→ re-verified 09:39:57, mode cards; expected baseline values matched.

## §5 09:45 WINDOW — **OPENED 2026-11-02T09:45:05−0500 — OBSERVATION-ONLY (ORB15 HOLD)**
- **ORB15 branch in force: H — RULED HOLD / observation-only, 0 fills claimed** (Nox 20261102T003600Z §1 — ORB15 HOLD
  (option ii); tap-to-approve revoked; branch B NOT available).
- Window state at open: heartbeat `age 1.6 s status running mode cards feed ws feed_age_s 0.1 market_open True
  last_bar_et 09:44 pos None trades 0 day_pnl 0.0`; **`cards_today= 0`**.
- **No card is tapped, no position is opened, no order is placed — by anyone.** Cards are logged as observations only; the
  day claims **0 fills**. Any card that appears is recorded in `cards.md` and the 16:20 outcome (completeness headline).

## §5b MID-DAY FEED OUTAGE — recorded as observed (no smoothing, no fix attempted)
- **Last good bar 13:41:00 ET** (`bars/2026-11-02.jsonl` last row `ts=1793644860`, file mtime 13:42:01; the engine's feed
  is the Tradovate page DOM over CDP).
- **13:42:16 ET** — engine logged `WARNING feed stale 61s, pausing entries` and sent the card
  `OpenBell FEED STALE — engine paused` (13:42:17). Designed behaviour: entries pause, nothing is placed.
- **Since then (checked 15:30–15:33 ET):** `heartbeat.feed="ws"` but `feed_age_s≈6535 s`, `last_bar_et=13:41`,
  `paused=True`, `pos=None`; the account watcher's frames stopped at **13:30:08** (`frame_t` stale ~6537 s; account ledger
  last row 13:30:08); `tabkeeper.json` shows `tradovate:"ok"`, `lane_feed:true` (blind-feed flag), `alerted:false`; the
  Tradovate tab is still present on :9222; all three services `active`, `NRestarts=0`. **No recovery observed.**
- **Root cause NOT established from the record** (the engine logs the stale condition, not the page's reason; no page
  content is read except the read-only panel capture used for the flat proof). No sign-in, no reload, no restart, no config
  change was attempted — the day is observation-only and the runbook does not authorise desk intervention.
- **Consequence for the flat gate:** proof 1 (fresh `cashBalance` frame after 16:00) cannot be produced (last frame
  09:01:36); the 16:05 flat proof therefore rests on proof 2 (read-only positions-panel capture at ~16:02, file written
  ≥16:00, positive flat statement) and the page-staleness qualification is stated in the day record.

## §6 16:05 FLAT GATE — **UNVERIFIED (recorded "unmanaged/unverified", NOT claimed flat)**
- **Proof 1 (fresh `cashBalance` frame after 16:00) — FAIL.** Last `cashBalance` frame is **09:01:36 ET**
  ($50,000 / SOD $50,000 / realized $0.00); `FRESH: FAIL — frame before 16:00`. The account watcher's frames stopped at
  **13:30:08 ET** (`frame_t` stale 6,537 s at 15:30; heartbeat `feed_age_s 8,627.8 s` at 16:05:08).
- **Proof 2 (read-only positions-panel capture ≥16:00, positive flat statement) — UNAVAILABLE.**
  - 16:01:50–16:02 ET panel read → **`EVALUATE FAILED` (empty error), exit 4, no file** — the page no longer answers.
  - 16:03:34–16:03:46 ET **last-resort `--reload`** (the runbook's documented last resort when the panel cannot be read
    because the page is stale; no position open by the engine's book; entries already paused) → **read after reload
    captured the Tradovate LOGIN SCREEN** (39 lines, 0 keyword lines; page text includes "Don't have an account? Try a free
    trial / Install App"). File: `positions-panel-1605.txt` (570 B, written 16:03:46). **No positive flat statement.**
  - **=> the session is logged out. The panel proof cannot exist.** Per the runbook: neither proof → record
    "unmanaged/unverified", never "flat".
- **STOP-FOR-CAYLEB condition:** the only verified sign-in route is Cayleb's desktop session. **No sign-in, no credential
  entry, no improvisation was attempted.** The desk remains logged out with entries paused; the day's observation continues
  as recorded only.
- Engine state at 16:05:08: `age_s=4.0 status=running mode=cards feed=ws feed_age_s=8627.8 last_bar_et=13:41 pos=None
  trades=1 day_pnl=72.5 paused=True`. The engine's own book is flat (`pos=None`), but **engine state is not account
  evidence** and the broker state after 13:30 is UNVERIFIED.

## §7 16:20 OUTCOME CHECK — **completeness PASS (1/1); 0 broker fills; observation-only; flat VERIFIED late (16:16:52)**
- **Signals (day-filtered engine ledger):** `live/ledger/2026-11-02.jsonl` **3 rows** — `signal 09:46:00` (side +1, qty 1,
  tag orb15_L), `fill 09:47:00` (shadow entry 7656.00, stop 7648.50, target 7671.00), `exit 10:15:00` (exit 7671.00,
  `usd_total 72.5`) → **1 signal**. (The runner's kind-based structural counter reads 0 because this engine ledger's rows
  carry no `kind` field — the operator/checker judges from the rows above; recorded as a known runner limitation, not a gap.)
- **Completeness (HEADLINE) — PASS 1/1:** the one signal has (i) its day-filtered card (`09:46:00 OpenBell BUY 1 MES`) and
  (ii) its outcome in the ledger (`exit 10:15:00`). **This is NOT a PASS-empty day** — one signal fired. It is a
  **card/shadow book only** trade: `trades=1`, `day_pnl +72.5`, **no broker counterpart**.
- **Broker record:** `fill_frames=0, position_frames=0, order_frames=0`; **2 `cashBalance` frames** — 09:01:36 and
  **16:16:52** (both `amount 50000.0`, `SOD 50000.0`, `realizedPnL 0.0`). **0 broker fills claimed.**
- **Flat:** 16:05 was UNVERIFIED (§6). **The desk is flat by 16:20 — verified late at 16:16:52:** fresh `cashBalance`
  $50,000 / SOD $50,000 / realized $0.00; `account_state` `attached`/`live`, `positions {}`, `dom.equity 50000.0`,
  `dom.open_pnl 0.0`, `frame_t` age 0.8 s. Recorded as **late** (the 16:05 instant had no proof).
- **Session outage/recovery (observed, not caused by Team C):** account frames stopped 13:30:08; last bar 13:41; engine
  paused 13:42:16; **logout detected 16:03:54** (card `OpenBell: Tradovate logged out` — the 16:03 last-resort reload made
  the logged-out page visible; the logout itself predates it); **feed OK 16:16:54 / engine resumed**, fresh account frames
  from 16:16:52. No sign-in, credential or config action was taken by Team C (the card's text names the Codex/Outis routed
  sign-in click — an external lane; recorded as observed, not attributed).
- **Cards today: 5** (BUY 09:46:00, EXIT 10:15:00, FEED STALE 13:42:16, LOGGED OUT 16:03:54, FEED OK 16:16:54).
- **Rails accounting:** live trades **0** (≤2 ✓ trivially); no −$400/+$900 touch; cooldown n/a; "$1,000 firm cap / 40 %
  consistency" n/a (checked, not enforced; no live trades); **flat by 16:05: UNVERIFIED at the instant, verified late**.
- **Friday reconciliation gap CLOSED by the Monday 09:01:36 frame** ($50,000 flat) — and re-confirmed by 16:16:52.
- **ORB15 HOLD branch in force all day** (Nox 20261102T003600Z §1): no tap, no position opened by anyone, 0 fills claimed.
