# MONDAY FUTURES RUNBOOK — Mon 2 Nov 2026 (ET) — OPERATOR CHECKLIST **v6** (checker-addressed)

Owner: **trading-lead** (Team C). Backup: **lead**. Tier S. Rails never change here.
v6 (2026-11-01 00:1xZ) = v5 (which fixed **trading-checker D1–D15**) plus the checker's six residuals **R1–R6** (see the last two
tables). Every attestation carries its **own command and its expected output**; two attestations that were inexecutable as
written now have real commands: (a2)'s positions-panel read (`board/probes/team-c/dom_read_positions.py`, read-only CDP) and
the 16:05 flat proof (two accepted proofs, neither able to mask a non-flat book).
**This file is the pre-check evidence for P0.1: no live step runs until the trading-checker signs P0.1.**

## GATE RULE (D13) — read this first
**Any attestation FAIL = the day does not start (the 09:45 window is not opened); record which leg failed and stop.**
There is no partial pass, no "proceed and watch", no substitute data. A leg that cannot be proven is recorded as
**UNVERIFIED** and is treated exactly like a FAIL for the purpose of opening the window.

Rails (fixed): -$400 daily stop / +$900 profit stop / $1,000 single-day firm cap / 40% consistency / flat 16:05 ET;
1 MES, max 2 trades/day, 15-min cooldown after a loss. Working desk `~/Work/projects/openbell`; broker record = the passive
Tradovate account-watcher frames; **engine self-report is never account evidence**.
The engine's enforced stops are -$400/+$900; the **$1,000 day cap and 40% consistency are firm-side rules and are not
engine-enforced** — they are checked, not enforced, at 16:20.
**Honest frame (Lead ruling X1):** the engine has only `shadow | cards`; `cards` = notify card + **human-placed bracket**.
There is **no autonomous execution mode**. Attestation (e) must read **"mode=cards — NOT automated"**. Never claim otherwise.

## P0 — before 09:05 (all four must hold; any FAIL = do not start)
| # | Check | Command (exact) | Expected output / PASS |
|---|---|---|---|
| P0.1 | Checker VETO signature on this checklist | (Lead-held) | `board/probes/team-c/monday-2026-11-02-precheck-signoff.md` exists, signed by `trading-checker`, timestamp before 09:05 |
| P0.2 | Browser route + tab | `ss -ltnp \| grep :9222` and `cat ~/Work/projects/openbell/live/logs/tabkeeper.json` | a Chromium listener on `127.0.0.1:9222`; `"tradovate"` is **not** `"missing"` |
| P0.3 | Services up | `systemctl --user is-active openbell-live openbell-dash openbell-tabkeeper` | three lines `active` |
| P0.4 | No position carry-over (D1: `card_out` does not exist in the heartbeat — dropped) | `python3 -c "import json;h=json.load(open('$HOME/Work/projects/openbell/live/heartbeat.json'));print('pos=',h['pos'],'paused=',h['paused'])"` | `pos= None paused= False`. An entry card cannot be pending: `config.json` `card_valid_min = 2`, so any card older than 2 min is expired by construction |

## 09:05 — SESSION GUARD (owner trading-lead; backup lead)
Append every line + its command output to `board/probes/team-c/monday-2026-11-02/guard.md`. All five, or FAIL.
1. **Clock/TZ.** `date; date -u; TZ=America/New_York date` -> the ET line shows **EST (-0500)** and `date -u` differs by
   exactly 5 h. DST ended Sun 1 Nov 06:00Z; an EDT host would shift 09:46/12:00/15:55/16:05 by an hour. Wrong -> STOP.
2. **Engine alive (D2: own command).**
   `python3 -c "import json,time;h=json.load(open('$HOME/Work/projects/openbell/live/heartbeat.json'));print('age_s=',round(time.time()-h['t'],1),'status=',h['status'],'paused=',h['paused'])"`
   -> `age_s=` **< 60**, `status= running`, `paused= False`.
3. **No carry-over, from the broker record (D9: the `cashBalance` frame — a `dom` frame is not a balance and there is no
   `positions` key in the account frames).**
```bash
python3 - <<'EOF'
import json,os,datetime as dt,zoneinfo
NY=zoneinfo.ZoneInfo("America/New_York")
p=os.path.expanduser("~/Work/projects/openbell/live/ledger/account-2026-11-02.jsonl")
rows=[json.loads(l) for l in open(p)] if os.path.exists(p) else []
cb=[r for r in rows if r.get("kind")=="cashBalance"]
print("frames",len(rows),"cashBalance_frames",len(cb))
if cb:
    r=cb[-1]; print("last cashBalance t=",dt.datetime.fromtimestamp(r["t"],NY).isoformat(),"amount=",r.get("amount"),"SOD=",r.get("amountSOD"),"realizedPnL=",r.get("realizedPnL"),"tradeDate=",r.get("tradeDate"),"venue_ts=",r.get("timestamp"))
else: print("NO cashBalance frame today -> broker state UNVERIFIED")
EOF
```
PASS ⇔ a `cashBalance` frame exists today **with its `t` quoted** (`positions {}` is **NOT** evidence — see (a2)). **If the last balance frame is
older than today 09:00, the broker flat state after that frame is UNVERIFIED** (this is exactly the owed Friday
reconciliation) -> attestation (a2) governs.
4. **Globex alive, RTH not yet (D15: `market_open` is the CME Globex session, so it is TRUE every Monday morning — the v4
   "market closed" demand was inverted).**
   `python3 -c "import json;h=json.load(open('$HOME/Work/projects/openbell/live/heartbeat.json'));print('market_open=',h['market_open'])"` and `TZ=America/New_York date +%H:%M`
   -> `market_open= True` (Globex alive) **and** the clock **< 09:30**. `False` here means the feed/engine is wrong -> STOP.
5. **Broker record reachable read-only.** Today's `live/ledger/account-2026-11-02.jsonl` exists with at least one frame
   after 09:00 (from the tab load). Missing -> the tab is not signed in -> (a)/(c1) will fail; record and stop.

## 09:15 — PREFLIGHT: five attestations, each with its own command + expected output
**(a) Broker session live + equity from a verified record (D7: strict — restored/stale data must not pass).**
```bash
python3 - <<'EOF'
import json,os,datetime as dt,zoneinfo
NY=zoneinfo.ZoneInfo("America/New_York")
d=json.load(open(os.path.expanduser("~/Work/projects/openbell/live/account_state.json")))
print("status=",d.get("status"),"| source=",d.get("source"),"| accounts=",list((d.get("accounts") or {}).keys()))
a=(d.get("accounts") or {}).get("71223147") or {}
t=a.get("t"); print("acct_t=",dt.datetime.fromtimestamp(t,NY).isoformat() if t else None,"| amount=",a.get("amount"),"SOD=",a.get("amountSOD"),"realizedPnL=",a.get("realizedPnL"),"tradeDate=",a.get("tradeDate"))
EOF
```
PASS ⇔ `status == "attached"` **and** `source == "live"` **and** account key `71223147` present **and** `acct_t` is
**after today 09:00 ET** **and** `amount`/`amountSOD` quoted. If `tradeDate` is still `2026-10-26` (the venue's last
cashBalance stamp is `2026-10-25T21:30:13.058Z`), record the anomaly — the balance is verified, the trade date is the
venue's stale field.
**(a2) Account FLAT — positively exercised or UNVERIFIED (D8: `positions {}` is the default dict; no `position` frame exists
in any `account-*.jsonl`, so absence is not evidence).**
```bash
grep -c '"kind": "position"' ~/Work/projects/openbell/live/ledger/account-2026-11-02.jsonl || true
```
PASS requires a **positive** proof after today's tab load. There are exactly two accepted proofs; the first has a command
that exists on disk now and is reviewable before 09:00:
1. **Positions-panel text over CDP — exact command** (read-only: `GET /json/list` + one `Runtime.evaluate` of
   `document.body.innerText`; no `Input.*`, no clicks, no navigation; the script is
   `board/probes/team-c/dom_read_positions.py`, written 2026-11-01 00:0xZ):
```bash
mkdir -p ~/Documents/deepseek-harness/default-workspace/board/probes/team-c/monday-2026-11-02
python3 -I ~/Documents/deepseek-harness/default-workspace/board/probes/team-c/dom_read_positions.py \
  --out ~/Documents/deepseek-harness/default-workspace/board/probes/team-c/monday-2026-11-02/positions-panel.txt
```
   Expected: exit **0** and a captured text file whose positions area carries a **positive flat statement** (an empty
   positions table / "No positions" / 0 open / no MES row). Exit **3** = no `trader.tradovate.com` tab -> **(a2)
   UNVERIFIED, the day is blocked by design** (no improvising a page read at 09:15 on a Tier S day). Exit 4 = CDP/evaluate
   failure -> same. **Judging rule:** only a positive flat statement passes; anything ambiguous = UNVERIFIED.
2. A position frame in today's `account-*.jsonl` that shows zero positions (none exists historically — if it ever appears,
   quote it; `grep -c '"kind": "position"'` above).
**Neither proof available -> FLAT: UNVERIFIED -> the day does not start** (blanket rule). Escalate to the Lead; the smallest
Cayleb act is "look at the Tradovate screen and confirm flat", recorded as his words, not as our inference.
**(b) Engine heartbeat current (D2: own command, not a pointer).**
`python3 -c "import json,time;h=json.load(open('$HOME/Work/projects/openbell/live/heartbeat.json'));print('age_s=',round(time.time()-h['t'],1),'status=',h['status'],'mode=',h['mode'],'qty=',h['qty'],'contract_id=',h['contract_id'])"`
-> `age_s < 60`, `status= running`, `mode= cards`, `qty= 1`, `contract_id= 4470322`.
**(c1) Tab + feed alive — PROVISIONAL at 09:15 (D3: a post-09:30 bar cannot exist at 09:15; the binding (c2) is 09:35–09:44).**
```bash
python3 - <<'EOF'
import json,time,datetime as dt,os,zoneinfo
NY=zoneinfo.ZoneInfo("America/New_York")
h=json.load(open(os.path.expanduser("~/Work/projects/openbell/live/heartbeat.json")))
bar=os.path.expanduser("~/Work/projects/openbell/live/bars/%s.jsonl"%dt.datetime.now(NY).strftime("%Y-%m-%d"))
rows=open(bar).read().splitlines() if os.path.exists(bar) else []
last=json.loads(rows[-1]) if rows else None
print("feed=",h["feed"],"| feed_age_s=",round(h["feed_age_s"],1),"| last_bar_et=",h["last_bar_et"],"| bars_rows_today=",len(rows))
if last: print("last_bar=",dt.datetime.fromtimestamp(last["ts"],NY).isoformat(),"| age_s=",round(time.time()-last["ts"],1),"| src=",last.get("src"))
print("C1:", "PASS" if ("retrying" not in str(h["feed"]) and h["feed_age_s"]<30 and last and time.time()-last["ts"]<=120) else "FAIL — name the leg: tab/browser/feed/bar")
EOF
```
**(d) Engine ARMED + rails asserted from the ACTUAL config and contract id (D4/D5/D14: the v4 "rails:" startup line does
not exist — `grep -c "rails:"` = 0; the engine never logs rails, and it has run since Wed 28 Oct so there is no "today's"
startup line unless restarted).**
```bash
systemctl --user show openbell-live -p ExecMainStartTimestamp,ActiveState,ExecMainPID
python3 - <<'EOF'
import json,os
c=json.load(open(os.path.expanduser("~/Work/projects/openbell/live/config.json")))
want={"mode":"cards","contracts":1,"max_trades_per_day":2,"daily_loss_stop_usd":-400,"daily_profit_stop_usd":900,
      "cooldown_min_after_loss":15,"flat_et":"16:05","min_stop_pts":2.0,"symbol":"MESZ6","contract_id":4470322}
bad=[]
for k,v in want.items():
    got=c.get(k); ok = got==v
    print(("OK  " if ok else "BAD "),k,"=",got,"(want",v,")")
    if not ok: bad.append(k)
print("RAILS_ASSERT:", "PASS" if not bad else "FAIL "+str(bad))
EOF
python3 -c "import json;h=json.load(open('$HOME/Work/projects/openbell/live/heartbeat.json'));print('paused=',h['paused'],'status=',h['status'],'strategy=',h['strategy'])"
```
**Bind the config file to the RUNNING process (R3: a file edited after the engine loaded can look safer than the process):**
```bash
START_EPOCH=$(date -d "$(systemctl --user show openbell-live -p ExecMainStartTimestamp --value)" +%s)
CFG_EPOCH=$(stat -c %Y ~/Work/projects/openbell/live/config.json)
echo "config_mtime=$CFG_EPOCH ($(date -d @$CFG_EPOCH -u +%Y-%m-%dT%H:%M:%SZ)) start=$START_EPOCH ($(date -d @$START_EPOCH -u +%Y-%m-%dT%H:%M:%SZ))"
[ "$CFG_EPOCH" -lt "$START_EPOCH" ] && echo "CONFIG_OLDER_THAN_PROCESS: PASS" || echo "CONFIG_CHANGED_AFTER_START: FAIL — restart the engine before trading"
```
(Verified 2026-11-01 00:0xZ: `config_mtime=1793141014 (2026-10-27T22:43:34Z)` < `start=1793216409 (2026-10-28T19:40:09Z)`
-> PASS. If it ever prints FAIL, the engine must be restarted and all five attestations re-run — a config edit after start
is not in force.)
PASS ⇔ `ActiveState=active`, `RAILS_ASSERT: PASS`, `CONFIG_OLDER_THAN_PROCESS: PASS`, heartbeat `paused=False`,
`status=running`. If the engine was restarted
today, also quote the new `logs/engine-2026-11-02.log` first lines and the day-filtered start card (below); if not
restarted, write "engine continuous since `ExecMainStartTimestamp` (28 Oct 15:40:09 EDT, PID 1484); no start line today by
design".
Day-filtered start card (D5: the STOPPED card is dropped — it fires only in one narrow case and cannot prove "not stopped";
systemd `ActiveState` is the proof):
```bash
python3 - <<'EOF'
import json,datetime as dt,zoneinfo,os
NY=zoneinfo.ZoneInfo("America/New_York")
d0=dt.datetime(2026,11,2,tzinfo=NY).timestamp()
p=os.path.expanduser("~/Work/projects/openbell/live/logs/cards.jsonl")
cards=[json.loads(l) for l in open(p) if l.strip()]
today=[c for c in cards if d0 <= c["t"] < d0+86400]
print("cards_today=",len(today))
for c in today: print(dt.datetime.fromtimestamp(c["t"],NY).strftime("%H:%M:%S"),"|",c.get("title"),"|",str(c.get("body"))[:160])
EOF
```
**(e) Execution mode in the engine's ACTUAL config.**
`python3 -c "import json,os;print('mode=',json.load(open(os.path.expanduser('~/Work/projects/openbell/live/config.json')))['mode'])"`
-> `mode= cards`. Attestation text, verbatim: **"(e) mode = cards — notify card + human-placed bracket; NOT automated (the
engine has no order path; Lead ruling X1)."** If it prints `shadow`, the mode step below applies. Anything else: the engine
refuses to start; the day stops.
**Decision logging — defined criterion (D6: the v4 "today's ledger exists at 09:15" is impossible; the file is created
lazily on the first event and no start record is written).**
- At 09:15, **"decision-log ARMED"** = `test -w ~/Work/projects/openbell/live/ledger` **and** engine `active` **and**
  yesterday's ledger exists with its last record quoted (or a documented empty). Command:
  `test -w ~/Work/projects/openbell/live/ledger && echo WRITABLE; ls -la ~/Work/projects/openbell/live/ledger/ | tail -3`
- **"decision logging PROVEN ON"** = after the first card/signal of the day, today's `live/ledger/2026-11-02.jsonl` exists
  with ≥1 record and that record corresponds to the first card (quote both). If **no signal has occurred by 16:05**, the
  verdict is **PASS-empty**: "0 signals, 0 records, declared" — not a failure and not a pass of a completeness claim.
- The 16:20 headline uses the day-filtered completeness criterion (§16:20).

## 09:35–09:44 — BINDING (c2): a fresh bar AFTER the 09:30 open (D3; R6: computed, not eyeballed)
The window must not open without this. Run this block (its PASS formula contains the **ET ≥ 09:30** term):
```bash
python3 - <<'EOF'
import json,time,datetime as dt,os,zoneinfo
NY=zoneinfo.ZoneInfo("America/New_York")
h=json.load(open(os.path.expanduser("~/Work/projects/openbell/live/heartbeat.json")))
bar=os.path.expanduser("~/Work/projects/openbell/live/bars/%s.jsonl"%dt.datetime.now(NY).strftime("%Y-%m-%d"))
rows=open(bar).read().splitlines() if os.path.exists(bar) else []
last=json.loads(rows[-1]) if rows else None
now=time.time(); et=""; age=None
if last:
    et=dt.datetime.fromtimestamp(last["ts"],NY).strftime("%H:%M"); age=now-last["ts"]
print("feed=",h["feed"],"| feed_age_s=",round(h["feed_age_s"],1),"| bars_rows_today=",len(rows),"| last_bar_ET=",et,"| age_s=",round(age,1) if age is not None else None)
ok = ("retrying" not in str(h["feed"])) and h["feed_age_s"]<30 and last and age is not None and age<=120 and et>="09:30"
print("C2_BINDING:", "PASS" if ok else "FAIL — name the leg: tab/browser/feed/bar/bar-before-09:30")
EOF
```
Quote the bar's `ts`, ET time, `src`. FAIL (or no bar with ET ≥ 09:30) -> record, do not open the window.

## ~09:40 — MODE STEP + RE-VERIFY
- `mode=cards` already -> no switch; re-run (b), (c2), (d), (e) and write "re-verified 09:40, mode cards".
- `mode=shadow` -> the only legal switch: edit `live/config.json` `"mode": "cards"`, `systemctl --user restart openbell-live`,
  then quote the new start card (day-filtered) and re-run all five attestations. There is **no switch to an autonomous mode**;
  building one is the Tier S successor (`team-c-autonomous-path-design.md`), never a Monday-morning edit.

## 09:45 — WINDOW OPENS
ORB15 fires 09:46–12:00 only. Log every card (day-filtered cards command above) into
`board/probes/team-c/monday-2026-11-02/cards.md`: time, side, price, stop/target, tag, and whether price was within 1.5 pts /
2 min. Feed STALE pauses entries; no forcing. Poll heartbeat every 15 min.

## 16:05 — FLAT (D11: freshness is required — a stale frame is not a flat proof)
```bash
python3 - <<'EOF'
import json,os,datetime as dt,zoneinfo
NY=zoneinfo.ZoneInfo("America/New_York")
p=os.path.expanduser("~/Work/projects/openbell/live/ledger/account-2026-11-02.jsonl")
rows=[json.loads(l) for l in open(p)] if os.path.exists(p) else []
cb=[r for r in rows if r.get("kind")=="cashBalance"]
if cb:
    r=cb[-1]; print("last cashBalance t=",dt.datetime.fromtimestamp(r["t"],NY).isoformat(),"amount=",r.get("amount"),"realizedPnL=",r.get("realizedPnL"))
    print("FRESH:", "PASS" if dt.datetime.fromtimestamp(r["t"],NY).strftime("%H:%M")>="16:00" else "FAIL — frame before 16:00, flat UNVERIFIED")
else: print("NO cashBalance frame today -> flat UNVERIFIED")
EOF
```
**Two accepted proofs (R4: a quiet day may produce no `cashBalance` frame after 16:00 — frames arrive on page load/change):**
1. **Frame proof:** a `cashBalance` frame with `t` **after 16:00** AND the engine `pos: null`.
2. **Panel proof (alternative, and the one that can exist on a quiet day):** run the read-only positions read at ~16:02 and
   require its output file to be written **at or after 16:00** and to carry a positive flat statement:
```bash
python3 -I ~/Documents/deepseek-harness/default-workspace/board/probes/team-c/dom_read_positions.py \
  --out ~/Documents/deepseek-harness/default-workspace/board/probes/team-c/monday-2026-11-02/positions-panel-1605.txt
stat -c '%y %n' ~/Documents/deepseek-harness/default-workspace/board/probes/team-c/monday-2026-11-02/positions-panel-1605.txt
```
**Last resort only — an exact read-only tab reload** (if the panel cannot be read because the page is stale): add
`--reload` to that command. Caveat, stated in the script: a reload can produce a **FEED STALE** card in the engine (entries
pause; nothing is placed), and it must **never** be used while a position is open.
**The conditional pass cannot mask non-flatness:** PASS requires a positive flat statement from (1) or (2). A panel text that
shows any position, or is ambiguous, is **FAIL (non-flat / unmanaged)**, not a pass. If neither proof exists: record
**"unmanaged/unverified"**, never "flat". The engine blocks entries 16:05–18:00 and FundedNext auto-closes 16:10 — the firm's
auto-close is never the flatten.

## 16:20 — OUTCOME CHECK: decision-log completeness is the HEADLINE, as a pass/fail (D10)
Run and quote all four; the headline is item 2.
1. **Day-filtered signal count.** `wc -l ~/Work/projects/openbell/live/ledger/2026-11-02.jsonl` (missing file = 0 signals;
   say so).
2. **Completeness PASS/FAIL (headline).** For every signal in today's ledger there is (i) a day-filtered card in
   `logs/cards.jsonl` and (ii) an outcome (`fill`/`cancel`/`skip`) in the ledger. **PASS** ⇔ zero gaps; **FAIL** ⇔ list every
   gap by signal. **If 0 signals: PASS-empty, stated as "no signals today, no completeness claim".** A missing decision
   record for a fired signal is the headline failure.
3. **Day-filtered cards.** The command in (d) above; quote `cards_today=N` (the file is cumulative — 26 tonight vs 2 on
   Friday — so the day filter is mandatory).
4. **Broker record.** Today's last `cashBalance` frame (item 3/16:05 command): `amount`, `realizedPnL`, `tradeDate`; any fill
   must appear in the broker frames, never only in the engine ledger.
5. **Friday's missing post-close reconciliation (owed).** Last balance frame 2026-10-30T15:29:12 ET (50000/50000/0.0,
   tradeDate 2026-10-26; venue stamp 2026-10-25T21:30:13.058Z); **after that frame the broker state is unverified** — the last
   Friday frame is a `dom` at 17:52:13 (`equity 0.0`), which is not a balance. Compare with the engine's 2026-10-30 card book
   (1 card trade, +$102.50, `mode: cards`, no broker counterpart). Verdict: broker flat as of 15:29 / card-only engine book.
6. **Rails accounting.** ≤2 trades; no -$400/+$900 touch; cooldown honoured; flat by 16:05; $1,000 firm cap and 40%
   consistency are firm-side (checked, not enforced): state the day's realised P/L and the consistency denominator.

## FAILURE DISPOSITION (D12: the only VERIFIED sign-in route is Cayleb's desktop)
- Do not sign in. `live/RUNBOOK.md` §8: Claude never signs in. **Nox -> Outis/Codex is UNVERIFIED** (Outis offline 151 d;
  B12 lane silent) — do not present it as a working route. **Cayleb's desktop sign-in on the existing Chromium tab is the only
  verified route**; a fresh credential or 2FA demand = **stop for Cayleb**, never improvise.
- Record: `cat live/logs/tabkeeper.json`, the last account frame, the engine log line (`no trader.tradovate.com tab`), the
  heartbeat `feed`/`feed_age_s`.
- The day is **BLOCKED-ON-MACHINE / no trade**: guard recorded, the failing attestation(s) named, decision log declared
  empty (not "complete"), no other host, no shadow substitute, no manufactured fills. Escalate through the Lead only.

## BOOKINGS — pinned (owner / backup)
| ET | Booking | Owner | Backup | If X1 = hold |
|---|---|---|---|---|
| Sat 20:07 (00:07Z Sun) | Kalshi UTC-day rollover check — armed: `board/probes/team-c/rollover-20261031T0007Z/` | trading-lead | lead | unaffected |
| Sun 05:50Z / 07:15Z | Kalshi DST-hour check — armed: `board/probes/team-c/dst-20261101T0715Z/` | trading-lead | lead | unaffected |
| Mon 01:00 | Claude reset; Kalshi resume (pack §3) | trading-lead | lead | unaffected |
| 09:05 | Futures session guard | trading-lead | lead | runs (read-only evidence) |
| 09:15 | Preflight: five attestations + decision-log ARMED | trading-lead | lead | runs; (e) = "cards, NOT automated" |
| 09:35–09:44 | **Binding (c2)** post-09:30 fresh bar | trading-lead | lead | runs; no window without it |
| ~09:40 | Mode step + re-verify | trading-lead | lead | re-verify only |
| 09:45 | Trading window opens | trading-lead | lead | **SLIPS** on a hold |
| 16:05 | Desk flat (fresh frame required) | trading-lead | lead | recorded |
| 16:20 | Outcome check (day-filtered completeness headline) | trading-lead | lead | runs; PASS-empty if no signals |
| Tue 3–Wed 4 | Kalshi freeze target (pre-freeze checker slot first) | trading-lead | lead | unaffected |

## D1–D15 DISPOSITION (what changed in v5)
| D | Fix in v5 |
|---|---|
| D1 `card_out` absent | dropped from P0.4 and 09:05; carry-over proven by `pos`/`paused` + the `card_valid_min=2` expiry argument |
| D2 (b) had no command | (b) and 09:05 item 2 each carry their own heartbeat command with age/status |
| D3 (c) impossible at 09:15 | split: (c1) provisional tab+feed+bar-alive at 09:15; **(c2) binding post-09:30 bar at 09:35–09:44**; no window without it |
| D4 fictional `rails:` line | removed; rails asserted from `config.json` (every field) + `contract_id` + systemd start timestamp/PID + the real day-filtered start card if restarted |
| D5 STOPPED card | dropped as a proof; `systemctl show ActiveState` + heartbeat `paused`/`status` + day-filtered cards are the proof |
| D6 decision logging at 09:15 | replaced with **ARMED** (writable ledger dir + engine active + yesterday's ledger) and **PROVEN ON** (first event line matches first card) / **PASS-empty** |
| D7 restored/stale (a) | PASS now requires `status=="attached"`, `source=="live"`, account `71223147`, `acct_t` after 09:00; `tradeDate` anomaly recorded |
| D8 vacuous flat | (a2) separate attestation: positive frame or positions-panel DOM read; else **FLAT: UNVERIFIED -> day does not start** |
| D9 wrong Friday frame/key | 09:05 item 3 points at the `cashBalance` frame and its `t`; states that after the last balance frame the state is unverified |
| D10 completeness not pass/fail | 16:20 rewritten as four pass/fail items; day-filtered signals/cards; PASS-empty defined |
| D11 stale 16:05 flat | 16:05 requires a frame `t` after 16:00, else "unmanaged/unverified" |
| D12 unverified sign-in route | Cayleb's desktop sign-in is the only verified route; Nox/Outis marked UNVERIFIED |
| D13 no blanket gate | GATE RULE at the top: any FAIL/UNVERIFIED = the window does not open |
| D14 no config-vs-rails assert | (d) asserts every rail field + `symbol=MESZ6` + `contract_id=4470322`; firm caps declared firm-side |
| D15 inverted `market_open` | 09:05 item 4 inverted: **Globex alive (`market_open: true`) and RTH not yet (clock < 09:30)** |

## What this runbook cannot prove (stated for the checker)
- **Completeness is internal consistency only (R5).** The only enumeration of signals is the engine's own ledger, so a signal the engine never logged cannot be detected by the 16:20 check — it proves that every logged signal has a card and an outcome, not that every market event produced a signal. "Decision-log completeness is the HEADLINE" means that, and no more.
- It cannot prove the account is flat without a positive position surface (D8) — that is why (a2) can end the day.
- It cannot prove a Tradovate session exists before the tab loads; (a)'s `status=="attached"` + fresh `acct_t` is the bar.
- It cannot make the desk autonomous (X1): Monday is a cards day; "first autonomous fills" is the successor build's claim.

## R1–R6 DISPOSITION (what changed from v5 to v6)
| R | Fix in v6 |
|---|---|
| R1 | 09:05 item 3 PASS now requires the `cashBalance` frame **with its `t` quoted** and states that `positions {}` is NOT evidence (points at (a2)) |
| R2 | (a2) has a real read: `board/probes/team-c/dom_read_positions.py` (read-only `GET /json/list` + one `Runtime.evaluate` of `document.body.innerText`; no `Input.*`, no clicks, no navigation), exact command, output file, expected text shape, and exit-3 = UNVERIFIED -> day blocked by design |
| R3 | (d) binds `config.json` to the running process: `stat -c %Y config.json` epoch **<** `ExecMainStartTimestamp` epoch, both quoted (verified PASS: 2026-10-27T22:43:34Z < 2026-10-28T19:40:09Z); FAIL -> restart and re-run all five |
| R4 | 16:05 accepts **two** proofs: a `cashBalance` frame after 16:00, or the positions-panel read at ~16:02 (file written ≥16:00, positive flat statement); a `--reload` last resort with the FEED STALE caveat; the conditional pass cannot mask a non-flat book (any position or ambiguity = FAIL) |
| R5 | completeness limitation stated: internal consistency only — a signal the engine never logged cannot be detected |
| R6 | (c2) is its own computed block with the `et >= "09:30"` term in the PASS formula |
