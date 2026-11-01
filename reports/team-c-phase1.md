# TEAM C — TRADING (Jobs 12-15), PHASE 1 + FIRST BATCH (C1–C5)

Teammate: **trading-lead**. Tier S throughout. Written 2026-10-31 ~18:20 ET (22:20Z), read-only night.
Scope of this batch: ground truth + the Monday plan. **No live order, no wallet operation, no broker session opened, no
credential handled, no new funds, no public send, no write to `~/Work/shared-workflows/lanes/board/**`, `money-desk` lane
not reopened.** All probes are reads or local in-process runs; every claim below carries the command that produced it.

Machines touched: this desktop (`outis-desktop`) only. Desk machine / coordinator VM / desk agent = not reachable (C2).

**Batch-2 update (2026-10-31 18:3x ET):** the Lead reviewed this report and ruled — X1 recorded as "cards, NOT automated"
(no engine change tonight); X8 accepted (GROUND-STATE M3 corrected); X2/X3/X4 accepted as Monday facts; X6 = 2000 stands
(location question to Nox); X7 = L\*=30 s FOR NOX (no DECISIONS.md edit); X9 relayed to Nox; X10 = repo `docs/CUTOVER.md`
is the reference; X12 evidence-only. Follow-on work is in `team-c-c6-c10.md` (C6–C10) and in runbook v2.

---

## 1. PIDM1 — the four jobs in my own words

**Job 12 (futures).** Get the ORB15 beta on the FundedNext $50K Flex eval (FN***52337) actually trading autonomously
inside fixed rails, and carry the strategy pipeline (MNQ prospective test, Kronos verdict, Round 3) forward without ever
arming a record the evidence has not cleared. Proof of Monday = fills in the **broker** record, plus a complete decision
log; the engine's own ledger is never the proof.
**Job 13 (momo).** Run one bot, one strategy, one new version per hour judged on live fills net of fees, inside the fund
boundaries (existing desk wallet only, exactly one live trader, no new funds) — first establish the live desk's ground
truth from its own data files, not the chat thread, then ship v5 by the direct machine route.
**Job 14 (Kalshi).** Keep building shadow evidence toward preregistered bars; rule the three contradictions in writing;
nothing live without a passed gate AND Cayleb's written go; the $51.39 is untouchable.
**Job 15 (cutover).** Move the live meme desk onto the merged OpenBell build (master 6f582f8) as one operation, with the
documented traps (data-driven exit rule, 5 s exit polling, one trader at a time, Cayleb moves the SOL) checked from the
deployed artefacts, not the repo seed.

## 2. Assumptions made (challengeable)
A1. "The deployed futures desk" = the user units running from `~/Work/projects/openbell` (checked: unit `WorkingDirectory`
    and `ExecStart` both point there). The merged master clone (`~/Work/projects/openbell-unified`, github/master =
    6f582f8) is the cutover target, not what is running.
A2. The desk's live momo data lives on the desk machine, which is not on this desktop's tailnet as reachable (C2).
A3. `readiness.py` run at 21:54Z is a legitimate stand-in for the two slipped self-checks **in the sense of data
    coverage** (both readings are of captured files); it cannot stand in for the two clock instants (20:07 ET / 03:12 ET)
    that have not occurred yet at 18:20 ET. The real re-run is Monday per `RESUME.md`.
A4. Broker account 71223147 is the eval account (amount/amountSOD 50000, FFlex rules in `live/account.json`: start 50000,
    max_loss 1500, target 2500, consistency 0.40, lock 50100). The broker record is the passive Tradovate account watcher's
    frame log, not an API read.
A5. A "card" is not an order. `mode: cards` emits an actionable card; the human places the bracket (RUNBOOK §Mode, §2).
A6. No spend: everything tonight is $0 (local reads + local test runs). Envelope for these jobs is $0 new funds.

## 3. Conflicts, gaps and contradictions found tonight (each with its evidence)

**X1 — BLOCKING, Job 12: the deployed futures engine has no autonomous execution mode.**
`live/config.json` says `"mode": "cards"` and its own `beta.what` reads "Cayleb taps every order himself"; the engine
allows only `shadow | cards` (`git grep -n "not in (\"shadow\", \"cards\")"` on `github/master:live/engine.py`: lines 25,
67), and the unit Description is "MES signal engine (passive Tradovate feed, shadow ledger, ntfy cards; **never places
orders**)". Cayleb's 31 Oct ruling quoted in the Job-12 brief says the beta is FULLY AUTOMATED with no per-trade taps.
=> The Monday 09:15 attestation (e) cannot honestly attest "automated". Runbook carries the disposition + the decision
needed; this is the programme's single largest trading conflict tonight.

**X2 — Job 12: the brief's "flat week, $0.00 P/L" is broker-true but engine-false, and that gap is the failure story.**
Friday 2026-10-30 the card book took one ORB15 short and booked +$102.50 by target (`live/ledger/2026-10-30.jsonl`:
`{"event":"exit","why":"target","usd":102.5,"day_pnl":102.5,"mode":"cards"}`), while the last broker frame is
`realizedPnL 0.0, amount 50000, tradeDate 2026-10-26` (2026-10-30T15:29:12 ET). The engine's number is a card-book
simulation; the broker never saw an order. Any report that quotes `day_pnl: 102.5` as desk P&L would be false.

**X3 — Job 12: Friday's post-close reconciliation does not exist.** No reconciliation artefact on this desktop
(`find ~/Work/projects/openbell -iname "*reconcil*"` -> no hits; no report dated 10-30). It must be produced from the
on-disk frames at the Monday 16:20 check (runbook §16:20). It cannot be completed from a fresh broker read tonight
without opening a session — which is forbidden tonight.

**X4 — Job 12: the market feed has been dead since Friday close.** `live/heartbeat.json` at 18:03 ET: `feed "retrying
(RuntimeError)"`, `feed_age_s 90202.5`, `last_bar_et "16:59"`, `market_open false`; engine log repeats
`feed attach/read error: RuntimeError('no trader.tradovate.com tab'); retry in 30s`. Weekend-expected, but it means the
Monday guard must prove the tab and the feed, not assume them.

**X5 — Job 13/15: no machine route to the live meme desk exists (route = NONE, not merely unknown).**
Nox's own lane registry (`~/Work/shared-workflows/lanes/board/ops/LANES.json`, read-only) declares
`{"id":"grok","name":"Grok / RSI desk","kind":"external"}` and the file's own note defines `external` as **"no machine
route: mail to it is handed to Nox"**. This desktop's shadow host layer (`~/Work/projects/memedesk-host`) exists and its
RUNBOOK states "The desktop runs no live trader. The Grok trader is the only live one." Tailnet: `muse 100.110.154.92`
(idle, some traffic), `nothing-phone-3a`, and offline `cayleb-desk` (253d), `outis` (151d), `srv1360751` (253d); no
`~/.ssh`, no host alias resolution, no network mounts (`mount | grep -E "cifs|nfs|sshfs|9p"` -> none). The one mailbox
that exists (`money-desk/intake`) delivers to the **moneydesk Claude lane**, not the desk machine, and the Grok side is
reached only by Cayleb pasting text into Grok (`GROK-RSI-NOTE.md`: "Cayleb pastes this into Grok"; "Claude has not touched
the VM or the wallet" — the live desk is on Grok/RSI's VM). => Job 13's ground-truth step and Job 15's deployed-file
checks are **BLOCKED-ON-MACHINE** until a route is given (see C2 for what I tried and what I could not read).

**X6 — Job 14: the "hourly bar reads 300" contradiction is not locatable on this machine.** Searched
`SPEC.md`, `PLAN.md`, `PREREG-DRAFT.md`, `DECISIONS.md`, `BUILD-LOG.md`, `FACTS.md` and the whole `openbell-kalshi`
worktree for a 300-event/hourly-bar statement: the only hourly-count statement on record is D2 = **2,000 hourly events**
(`DECISIONS.md:10`, `PLAN.md:237`, `PREREG-DRAFT.md:25`). The `300` figures found are Brier lookback rows
(T1-300, SPEC R22), not a bar. Either the doc carrying 300 is coordinator-side, or the brief's characterisation is stale.
The ruling is drafted to be safe either way (see §6 R1) but **the exact location must be named before the amendment is
logged**.

**X7 — Job 14: the L\* = 60 s confirmation of 13:17 ET is contradicted by the measured availability.** New measurement
(18:xx 20:xx UTC file `results/availability_2026-10-31.json`): decisions readable **16.127 s p50 / 16.735 s p90** after
the decision tick (+24.586 / 27.713 s after `a_asof`); with intent pull ~1–2 s and gateway <1 s end-to-end ≈ 19–20 s, so
the smallest row of record at or above it is **30 s**, and 30 s is also above the data-time p90 (27.7 s). 60 s therefore
looks too slow. This needs Nox's ruling + a DECISIONS.md log line; the sentence "provisional L\* = 60 s used for θ/X" is
still what `DECISIONS.md` and `scorer_agreement.json` record.

**X8 — Job 14: the Kalshi systemd "enabled" claim was mis-stated in GROUND-STATE.** GROUND-STATE M3 says "System units
openbell-kalshi-poll/-ref/-shadow are enabled (list-unit-files)". They are **user** units, not system units:
`systemctl list-unit-files 'openbell*'` -> "0 unit files listed"; `systemctl --user list-unit-files` -> three enabled;
`systemctl --user is-active` -> three **active**. The actual state is better than "enabled by links"; the wording in M3 is
wrong and should be corrected (see §4 C1).

**X9 — Job 14: Nox's independent collector copy is stale, so readiness check 8 is NA by construction.**
`~/Work/projects/kalshi-shadow-data/data/kalshi-shadow-2026-10-31.jsonl` (51 MB, mtime 08:35 ET, 165,051 lines) last line
`{"type":"gap","ts":"2026-10-31T11:59:42Z",...}` — pushes stopped at 12:00Z, ~10 h before this probe. Local capture is
fresh to 21:54:28Z. That is why `nox_reconcile` = NA ("no time overlap with Nox's clean cycles; settlements both 45,
result disagree 0, value disagree 0") and not a failure.

**X10 — Job 15: the named pre-cutover checklist file is absent from this machine.**
`openbell-cutover-prechecks-2026-10-31.md` ("in the trading goal files, including section 3b") is not on
`outis-desktop`: `find /home/cayleb -name "*cutover*"` / `"*precheck*"` return only the Downloads brief and unrelated
projects. The **repo-side** cutover document does exist and is substantial (`github/master:docs/CUTOVER.md`, read in
full); C5 is built from that + the repo, and the missing goal-file is listed as a gap for the Lead (it may live on the
desk machine / coordinator side).

**X11 — Job 15: the repo seed exit rule is verified, the DEPLOYED one is not.** `github/master:meme/desk/seed/live/exit_rule.json`
= variant B, `maxHoldH 0.5`, `promotedAt 2026-10-23T00:02:00.519Z`, evidence eqB 1.048195 > eqA 0.987537, ddB 0.0728328 <
ddA 0.113242. The desktop shadow copy agrees byte-content-wise. But the file that matters is the one the **live desk
machine** runs, plus its startup log line — unreadable from here (X5). Same for the 5 s polling: `git grep POLL_SEC
github/master` finds only `trader.js: poll: +(process.env.POLL_SEC || 20)` and a changelog note "POLL_SEC=20"; nothing
5 s in the repo — consistent with "5 s exists only on the desk machine", but unverified from here.

**X12 — desktop meme-desk shadow host is in a restart loop (evidence only, not ours to fix tonight).**
`systemctl --user status memedesk-desk.service` -> `failed`, exit `78/CONFIG`; journal: `desk-start: refused: settings NOT
READY`; it retried at 18:13:04 ET while probing. This is the *desktop shadow* host (`~/Work/projects/memedesk-host`), not
the live trader; recorded so nobody mistakes it for live-desk state. `memedesk.target` active, `memedesk-sync.timer`
active, `var/sync/applied` = `25d1574…` (26 Oct lineage).

**X13 — GROUND-STATE M3 minor: the openbell checkout confusion.** `~/Work/projects/openbell` (the running desk) is an
older clone with **no remote**, HEAD `fd82bb1`; the GitHub repo is `cayleb-james2008/openbell`, whose only branch is
`master` @ **6f582f8** (`gh api repos/cayleb-james2008/openbell/branches/master`), and the clone that carries it is
`~/Work/projects/openbell-unified` (`github/master` = 6f582f8, worktrees: `openbell-unified` @ 6687b79 `unify`,
`openbell-kalshi` @ 4f73291 `stack/4-kalshi-shadow`). Anyone told "openbell master = 6f582f8" must not look at
`~/Work/projects/openbell`.

**X14 — Job 14 left-over open item (not one of the three rulings): the intent format v1 → v1.1 amendment and rail
values are Nox's, still open.** Recorded in `RESUME.md` §3; nothing built on them; no action tonight.

---

## 4. BATCH EVIDENCE (quoted commands, results, timestamps)

### C1 — Kalshi: unit state, readiness re-run, scorer agreement, WP9/WP10, data freshness

**(C1.1) Unit state — enabled AND active (user units).**
```
$ systemctl --user list-unit-files | grep -i -E "openbell|kalshi"
openbell-kalshi-poll.service      enabled  enabled
openbell-kalshi-ref.service       enabled  enabled
openbell-kalshi-shadow.service    enabled  enabled
$ systemctl --user is-active openbell-kalshi-poll openbell-kalshi-ref openbell-kalshi-shadow
active
active
active
$ systemctl list-unit-files 'openbell*'          # system scope: 0 unit files listed
$ systemctl --user list-timers --all | grep -i kalshi   # no timers: these are Type=simple services
```
(read 2026-10-31 ~21:53Z). Installed by the owner 12:23 ET (per `RESUME.md`), `ExecStart=python3 -I .../poll.py run` etc.,
`WorkingDirectory=%h/Work/projects/openbell-kalshi`, `ProtectSystem=full`; the shadow unit's
`RestrictAddressFamilies=AF_UNIX` means the paper engine cannot open a socket at all. Nothing here trades.

**(C1.2) The owed self-checks, re-run on captured data — `readiness.py`, 10 checks, 9 PASS / 1 NA / 0 FAIL, exit 0.**
```
$ cd ~/Work/projects/openbell-kalshi && timeout 1800 python3 -I research/kalshi15/readiness.py \
    --day 2026-10-31 --out .../board/probes/team-c/readiness-2026-10-31.json
readiness 2026-10-31  read at 2026-10-31T21:54:29Z  observed 2026-10-31T12:33:55Z..2026-10-31T21:54:28Z  ledger tick 2026-10-31T21:54:00Z
cutoff  capture books 2026-10-31T21:54:28Z  ref 2026-10-31T21:54:29Z  ledger decisions/orders tick 21:54:00Z (4375/31397 complete lines)
PASS  capture_validator     5 files, 2672540 records (book=31590 fees=39 gap=6 market=7554 schedule=3 settled=547 start=3 status=114 stop=2 trade=2632682), errors 0
PASS  gap_ledger            unhealthy-book share worst 0.036% (HYPE) over 9 coins; gap records 6 (book 63s trades 37s meta 0s); restarts 2 crashes 0; longest gap 24s
PASS  ref_health            seconds with n>=1: worst 99.973% (BNB); venue connects 33 disconnects 26 errors 19; clock offset -2.678..2.29 ms
PASS  settlement_reconcile  405 windows settled: result vs rule 405/405, shadow vs result 333/333 (0 pending, 72 before the shadow started, 0 missing); mismatches 0
PASS  proxy_wrong_side      worst coin 3/36 (8.3%) ZEC; all coins under the 10% warn line (report only)
PASS  replay_determinism    byte-identical over the common prefix: decisions 4375, orders 31397 up to tick 21:54:00Z, 46.1 s
PASS  chain_integrity       decisions 4375 lines head 44d1526b1f0c5f97; orders 31397 lines head a56073229356a79c
NA    nox_reconcile         no time overlap with Nox's clean cycles; settlements both 45 (result disagree 0, value disagree 0)
PASS  counts                healthy settled windows per coin 35–36; days to 2,000 at observed rate 21.0–21.6 (ideal 21.1)
PASS  heartbeats            age poll=1.0s ref=1.7s shadow=0.8s; errors_1h poll=0 shadow=0
EXIT=0
```
Full JSON + log: `board/probes/team-c/readiness-2026-10-31.{json,log}`.
**Honest limit:** this is not the two *clock-instant* checks. Per `RESUME.md` §Tonight's checks: 20:07 ET = UTC-day
rollover (00:00Z, ~2 h after this run) and 03:12 ET = the DST hour; their session crons were cancelled and the re-run is
owed Monday (`--day 2026-10-31` and `--day 2026-11-01`, plus reading `live/kalshi/capture/2026-11-01/gaps.jsonl` and
`events.jsonl` around 05:30–07:00Z). The data those readings need is accumulating now (capture is fresh to 21:54:28Z).
The RESUME table's "03:12 ET (06:00Z)" mapping is arithmetically wrong (06:00Z = 02:00 EDT = the fallback instant;
03:12 ET = 07:12Z EDT / 08:12Z EST) — flag for the Monday runner.

**(C1.3) Two-scorer agreement — confirmed from the artefact, figures quoted.**
`research/kalshi15/results/scorer_agreement.json` (date_utc 2026-10-31T17:39:32Z):
`"result": "AGREE: 6853 rows each, 0 differences; summaries and the late side file identical (tol 1e-9, integers exact);
six latency rows {2,5,10,30,60,90} (2 and 5 informational)"`; history: 10-31 10:35 AGREE 6788 rows (pre-amendment),
11:40 AGREE 6853 (post-amendment), 13:45 AGREE 6853 with the six-row latency set. Note the JSON still carries
`"latency": {"L_star": 60, "rows_of_record":[10,30,60,90], "informational":[2,5]}` — X7 is why that `L_star` should move
to 30 s.
Method caveat (from the file itself): the second scorer was "built from SPEC.md only; forbidden from the first scorer's
code and results"; scope is decisions/fills/P&L/Brier given the model's p, not the model itself (checked by the canary).

**(C1.4) WP9 status — reproduced by me, not taken from the summary.** `readiness.py` = the WP9 check set; result above:
**9 PASS / 1 NA / 0 FAIL**, matching the "WP9 readiness 9 PASS / 1 NA / 0 FAIL" claim.

**(C1.5) WP10 status — verified by running the package's own tests and the dry-run comparison, not by reading the log.**
```
$ PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s live/kalshi_exec -p "test_*.py" -t .
Ran 167 tests in 0.880s
OK                                   # 2026-10-31 ~22:0xZ
$ cat live/kalshi_exec/state/dryrun/compare.json
"ok": true, "missing": 0, "extra": 0, "price_or_side_mismatch": 0,
"shadow_orders": 145, "matched_same_ticker_side_price": 8, "refused_with_reason": 137,
"refusal_reasons": {"sized_zero:below_one_contract": 124, "would_cross": 13}, "theta": 0.06, "L": 60
$ for f in exec_events latency refusals summary venue-state would_be_orders; do sha256sum dryrun/$f dryrun-rerun/$f; done
SAME exec_events.jsonl  SAME latency.json  SAME refusals.jsonl  SAME summary.json  SAME venue-state.jsonl  SAME would_be_orders.jsonl
```
Dry-run summary: `final_cash_usd 24.09`, `shadow_fires 145`, `sized_to_zero 124`, `refused_by_rails 13`, `would_be_orders 8`,
`place_intents_written 8`, `placed_acked 3`, `cancels_acked 3`, `mock_fills 0`, `unsettled_positions 0`,
`daily_loss_stop_trips 0`. => WP10 is built, OFF by construction, dry-run reproducible byte for byte on rerun.
Count discrepancy to note: `RESUME.md` says "172 tests"; my discovery run found **167** test methods (OK). Probably
subTest counting; no failure either way. Core shadow stack also run fresh:
`python3 -m unittest discover -s live/kalshi` -> `Ran 247 tests in 30.951s, OK` (log
`board/probes/team-c/tests-live-kalshi.log`, EXIT=0).

**(C1.6) Data-file freshness.**
- Local capture: fresh to `2026-10-31T21:54:28Z` (readiness cutoff), 2,672,540 records, 0 schema errors; heartbeats
  poll/ref/shadow age 1.0/1.7/0.8 s at 21:54Z.
- Ledger: 4,375 decisions / 31,397 order events, chain verified.
- Nox's independent copy: 51 MB, 165,051 lines, last record `2026-10-31T11:59:42Z` (stale ~10 h; X9) — hence
  `nox_reconcile` NA.
- Bar arithmetic (from readiness `counts`): 35–36 healthy settled windows per coin today; ~21.0–21.6 days at the observed
  rate to 2,000/coin (ideal 21.1 d at 94.9/day) — consistent with the brief's "earliest ~25 Nov" arithmetic, not a
  promise.

### C2 — momo: machine route (result: **route = NONE / not obtainable from this desktop**)
What I tried, in order, and what it returned:
1. Running processes: `pgrep -af "trader\.js|supervisor|momo"` -> **no match** (only this probe's own command line); the
   only trading processes are the futures stack (`live.engine` 1484, `live.dash` 1481, `live.tabkeeper` 995188,
   `arena.py` 1496) and the three Kalshi shadow services (`poll.py` 2588758, `refrec.py` 2588809, `shadow.py` 2713864).
   No momo, no trader, no supervisor on this desktop (GROUND-STATE M3 re-confirmed).
2. Filesystem: `~/Work/projects/openbell-meme-desk` is RSI's repo clone; `data/` holds only Oct-28 paper/bench files
   (`data/live/ledger.jsonl` 3.2 MB, `data/exit_rule.json`, `data/sizing.json` — all mtime Oct 28); **`data/strategy/` does
   not exist**, i.e. no `live.log`, no `status.json`, no ledger for v4. `supervisor/` is the Oct-28 local copy
   (`supervisor.log` 28 Oct 11:00, `supervisor.pid` dead). The live desk data named in Job 13 is not here.
3. Host layer: `~/Work/projects/memedesk-host` RUNBOOK: "The desktop runs no live trader. **The Grok trader is the only
   live one.**" The desktop shadow unit is failing (X12) and, even if healthy, is a public-RPC shadow, not the live book.
4. Network routes: no `~/.ssh` (directory absent), no SSH config, no host alias (`ssh muse` cannot resolve, per GROUND-STATE
   M0, re-confirmed by the missing config); tailnet membership `tailscale status`: outis-desktop 100.114.17.67 (this box),
   muse 100.110.154.92 (idle; 436 tx/348 rx), nothing-phone-3a 100.65.183.43, outis Windows offline 151 d, cayleb-desk
   offline 253 d, srv1360751 offline 253 d. Nothing on the tailnet advertises the desk machine.
5. Board mail path: Nox's own registry is the authoritative answer — `LANES.json` row
   `{"id":"grok","name":"Grok / RSI desk","kind":"external"}`, and the file's rule: **`external` = "no machine route: mail
   to it is handed to Nox"**. So there is no board mailbox that reaches the desk; only a human/Nox relay.
6. Processes and mounts: `pgrep -af "trader.js|supervisor|momo|shadow.py|poll.py|refrec|desk"` -> the only matching live
   processes are the futures dashboard (`live.dash --account`), the desktop shadow refresher
   (`memedesk-host/refresh/refresher.py --loop`, whose desk unit is failed, X12) and display/session furniture; **no momo,
   no trader, no supervisor**. `mount | grep -E "cifs|nfs|sshfs|9p"` -> no network mount; only gvfs/portal FUSE.
7. The one mailbox that does exist: `~/Work/shared-workflows/money-desk/intake` (a Nox -> **moneydesk lane** mail
   dispatcher: `moneydesk-intake.timer` every 2 min, git-board transport on branch `team`, heartbeat
   `2026-10-31T22:20:01Z`, ledger state `done 3 / skipped 4`). It reaches the moneydesk Claude lane, **not** the Grok/RSI
   desk machine. And the Grok side of that desk has no machine transport at all: `money-desk/GROK-RSI-NOTE.md` says
   "Cayleb pastes this into Grok" and "Claude has not touched the VM or the wallet" — i.e. the live desk runs on Grok/RSI's
   own VM and the only carriage is the human. (Read-only peek; the moneydesk lane stays CLOSED until Mon 01:00 ET — not
   reopened, no mail sent, no worker started.)
**Conclusion: Job 13's ground-truth step cannot start from this desktop. `route = UNKNOWN` is too generous — the lane's own
registry says there is NO machine route; the only path is a Nox-mediated handoff (or a direct answer from RSI on its own
machine).**
**Data I could NOT read (must be treated as unverified, never as zero):** v4's fills since 11:28 ET, its
`data/strategy/live.log`, `data/status.json`, the live ledger files, the parameter definitions v4 actually ran, the
version-note log, whether the thread's last (unverified, unknown-sender) message changed anything, the reviewer's state,
and whether exactly one live trader is running right now. I reconstructed nothing from the chat thread (the brief's rule);
I have no access to it from here at all.

### C3 — futures: desktop sign-in route, broker record access, last known state (read-only)
**Sign-in route (of record):** `live/RUNBOOK.md` §8 "Sign-in (routing rule, Cayleb 2026-10-27 02:38 ET)": *Claude never
signs in.* `openbell-tabkeeper.service` keeps Chromium (CDP 9222) + the `trader.tradovate.com` tab and, on logout, sends
one phone card + one board flag to Nox; Nox routes the sign-in click to Outis/Codex, which signs in with the existing
session. Tonight's route state: Chromium is up on `127.0.0.1:9222` (`ss -ltnp | grep :9222` -> `LISTEN … pid=2960519`),
but **no Tradovate tab**: `live/logs/tabkeeper.json` at 18:04 ET -> `"tradovate": "missing"`, `"alerted": true`,
`"lane_logout": true`; `account-format.log` 16:19 ET "no Chromium on CDP port 9222 / no trader.tradovate.com tab".
**No broker session was opened tonight.** A fresh credential/2FA demand on Monday = stop for Cayleb.
**Broker record access used before (read-only):** the passive account watcher (`live/account.py --account`, run by
`openbell-dash.service`): it attaches to the Tradovate tab the same passive way as the feed and only reads received
account frames, writing `live/ledger/account-YYYY-MM-DD.jsonl` and `live/account_state.json`; it never types or clicks.
That is the record to cite on Monday — not the engine's `heartbeat.json`.
**Last known account state (broker frames, Friday):**
```
live/account_state.json:  "status": "no trader.tradovate.com tab"   (current)
accounts 71223147: amount 50000.0, amountSOD 50000.0, realizedPnL 0.0, tradeDate 2026-10-26
risk  71223147: minNetLiq 50000.0 / maxNetLiq 50000.0
autoliq 71223147: trailingMaxDrawdown 1500.0, limit 50000.0, mode EOD
positions {}           sync_t 2026-10-30T15:29:12 ET   source "live"
last event in live/ledger/account-2026-10-30.jsonl: {"kind":"dom","ev":"dom","equity":0.0,"open_pnl":0.0,"et":"2026-10-30T17:52:13-04:00"}
last cashBalance frame: {"amount":50000.0,"amountSOD":50000.0,"realizedPnL":0.0,"tradeDate":{"year":2026,"month":10,"day":26},
                         "timestamp":"2026-10-25T21:30:13.058Z"}  (received 2026-10-30T15:29:12 ET; Tradovate pushes on change only)
```
=> Last verified broker state: **flat, $50,000.00, $0.00 realised, no positions** (frame stamped 2026-10-25T21:30:13Z,
last received Friday 15:29 ET). Nothing after that is verified from this desktop.
**Engine state (self-report, NOT account evidence):** `live/heartbeat.json` 18:03 ET: `status running`, `mode cards`,
`qty 1`, `strategy ORB(last_entry=12:00,max_stop=15.0,min_stop=2.5,n=15,r=2.0,stop_mode=mid)`, `feed "retrying
(RuntimeError)"`, `feed_age_s 90202.5`, `last_price 7614.5`, `contract_id 4470322`, `bars 4903`, `last_bar_et 16:59`,
`pos null`, `day_pnl 102.5`, `trades 1`, `paused false`, `market_open false`. The `day_pnl 102.5` is the 2026-10-30 card
book (X2) and must never be quoted as account P&L.
**Friday's post-close reconciliation: does not exist** (X3). Nearest artefacts: `live/ledger/2026-10-30.jsonl` (engine book,
1 card trade), `live/ledger/account-2026-10-30.jsonl` (broker frames), `live/logs/cards.jsonl` (last card 2026-10-30 17:52
mtime). No reconciliation file/report anywhere on this desktop.

### C4 — Monday runbook
Written to `board/reports/monday-futures-runbook.md` (09:05 guard; 09:15 five attestations individually with evidence
source; mode-switch before 09:45 + re-verify; 09:45 open; 16:05 flat; 16:20 outcome with decision-log completeness as the
headline; DST gate; owner/backup/fallback per step; the X1 decision item for Cayleb). Nothing in it has run.

### C5 — Job 15 pre-checks (plan only; nothing executed)
**Master verified:** `gh api repos/cayleb-james2008/openbell/branches/master` ->
`6f582f87db3917e2147bd02d509ec1886fc774a8  Merge pull request #3 from cayleb-james2008/stack/3-review — Unify 3/3: deep
review, every verified finding fixed`; the repo has **exactly one branch** (`master`; `gh api .../branches` -> `["master"]`),
pushedAt 2026-10-31T10:51:26Z. Local clone `~/Work/projects/openbell-unified`: `git rev-parse github/master` = 6f582f8…
and `git rev-parse 6f582f8` == `github/master` -> **YES**. (Do not use `~/Work/projects/openbell`: no remote, HEAD fd82bb1
— X13.) Repo-side cutover document `docs/CUTOVER.md` read in full (4 sections: futures side, meme desk move, ntfy rotation,
tidy-up).

**Must be true before the cutover starts — checklist with the exact check (all currently BLOCKED-ON-MACHINE unless noted):**
1. **Deployed exit-rule file byte-checked + startup-log line captured.** Read the file the *live trader* loads
   (`DATA/live/exit_rule.json` on the desk machine; `DATA` = the desk's own data dir) and its startup line "exit rule B
   (…exit_rule.json, promoted …): maxHoldH 0.5 overrides MAX_HOLD_H=…" (the trader logs this; see
   `meme/desk/test/trader_live.test.js:310-317`). Compare bytes to the repo seed
   (`sha256sum` of the deployed file vs `git show github/master:meme/desk/seed/live/exit_rule.json`) — the repo seed is
   variant B, maxHoldH 0.5, promotedAt 2026-10-23. **A missing/corrupt DATA file silently falls back to variant A at 24 h,
   so this is the first pre-check and a failure stops the cutover.** Evidence from here: seed verified; deployed = NOT
   READABLE (X5/X11).
2. **5-second exit polling located and carried over.** Not in the repo (`POLL_SEC` default 20 in `meme/desk/trader.js:23`;
   the 5 s was set on the desk machine 30 Oct). Locate its exact source on the desk machine (env line in the desk's
   runner/unit, `POLL_SEC`), record the file+line, and carry it into the new host's env; verify by the startup log or the
   trader's reported poll interval. NOT READABLE from here.
3. **`MAX_HOLD_H` environment confirmed.** Repo default 24 h in `trader.js`; run.sh sets paper 0.5; the persisted exit
   rule overrides it at startup. On the desk machine: print the live trader's env (name/value is not a secret) and record
   it; expected 0.5 via the exit-rule file, not the default.
4. **Ledger continuity prepared.** Both ledgers + demotion scores copied from the desk machine's copy, checksums recorded
   before/after; the new build must append, not truncate (`data/live/ledger.jsonl`, the demotion scores file(s),
   `data/live/trades.csv`). Desktop has only the Oct-28 snapshot → the copy source is the desk machine.
5. **ntfy topic rotation ready.** `~/.config/openbell/ntfy.json` exists (mode 600, 190 B; content NOT read — it is the
   secret). Rotation procedure is `docs/CUTOVER.md` §3 (mv to `.old`, `python3 -m live.engine --test-push` regenerates,
   subscribe/unsubscribe, restart `openbell-live` + `openbell-tabkeeper`, test card, delete `.old`). This touches the
   *futures* side; the meme desk's phone/Watchdog path is the Cloudflare Worker secret (item 6). Preparation = record the
   file's existence + the doc's steps; execution waits for cutover.
6. **Cloudflare Worker redeploy ready.** `meme/desk/watchdog/cf/worker.mjs` exists in master (+ `gen.js`, `pusher.js`,
   `login.html`); redeploy is needed for the phone page to show a signature-less trade as "unknown" instead of "paper"
   (`docs/CUTOVER.md` §2). Also `~/.config/memedesk/push_secret` must match a single `PUSH_SECRET` on the Worker — a
   second pusher would compete with RSI's live one, so the pusher stays off in shadow. Preparation = confirm the file and
   the secret-handling rule; no repo change.
7. **Section 3b** (from the brief; the goal-file is missing here — X10): archive the desktop `openbell-meme-desk` tree
   ONLY after the host layer that binds it is repointed and verified (a live service bind-paths it; archiving first breaks
   the desk). Repo `docs/CUTOVER.md` §4 confirms the ordering ("keep both old trees until the new desk has run a full day,
   then remove them").
8. **Preconditions that gate the whole job:** Job 13 stable (v5 verified from desk data, loop cycling, chat reconciled,
   0 open positions) — impossible tonight (X5); Cayleb moves the SOL himself (his act); the new host's
   `meme/host/RUNBOOK.md` "Moving from the old layout" section followed in order; `JUP_API_KEY` loaded write-only via
   `meme/host/settings/settings.py ingest` — and note `lite-api.jup.ag` retired 2026-10-30, so without the carried-over
   key every Jupiter call fails (`docs/CUTOVER.md` §2). Never print the key; the request that it be present is a gate.
9. **Monetary/ownership boundary:** one trader at a time, never two, at any point; no new funds; the existing book only.
10. **Post-cutover watch (done-condition):** first scans, first fills, deployed-rule exit visible in the log (0.5 h max
    hold), old runtime confirmed dark, on-chain wallet state before/after read directly.

**Nothing in C5 was executed: no cutover, no archive, no worker redeploy, no topic rotation.**

---

## 5. Kalshi — the drafted rulings text **FOR RULING** (not applied; no file in the repos was edited)

All four texts below are drafts for Nox/Cayleb's ruling. They are written as the amendment text that would be logged; the
build side must not edit `PREREG-DRAFT.md`, `DECISIONS.md` or `SPEC.md` without the ruling (DESIGN.md C3 already refuses
to reword the draft without one). **Status: FOR RULING — NOT APPLIED.**

**R1 (hourly bar).** *Ruling asked:* "The hourly arm (Arm B, KXBTCD belly) bar counts **2,000 hourly events**, per D2 of
the settled decision file (2026-10-31 06:57 ET, Nox for Cayleb). Any 300-event/hourly-bar figure is superseded; the section
carrying it is amended to 2,000 before the resume pack ships. The 15M arm's bar is unchanged at 2,000 settled windows per
coin. No bar moves mid-count (D5)."
*State of the text:* the 300 figure is **not present** in the docs on this desktop (X6) — the amendment can only be
logged after the lane names the file/section that carries it; the D2 reading above is the only hourly-count statement on
record and is what the amendment enforces.

**R2 (stale header).** *Ruling asked:* "The sentence '**No order path exists in this stack**' (PREREG-DRAFT §1) is
superseded (Nox Amendment 1, SPEC Appendix T item 3; DECISIONS.md line struck). Replacement wording, everywhere: 'The
shadow stack (`live/kalshi`, `research/kalshi15`) contains no order path. A separate execution package
(`live/kalshi_exec/`) exists and is disabled by construction: it cannot place an order without (i) the enable flag, (ii) a
passing gate, (iii) Cayleb's explicit go relayed in writing. The guard tests assert that refusal.' Nothing else in §1
changes."
*Evidence:* `PREREG-DRAFT.md:10-11` still reads it; `live/kalshi_exec/test_no_order_path.py` part B asserts the
supersession; `test_refuses.py` asserts the order path exists and refuses (guard table 64 combinations, all refuse);
`DESIGN.md` C3 leaves exactly this wording to Nox. **This one can be applied the moment it is ruled** — the correct text
already exists in the tests and in SPEC Appendix T.

**R3 (Kelly posture).** *Ruling asked:* "Live sizing is not capped quarter-Kelly only. The recorded posture stands:
Cayleb directed heavier sizing to escape zero-contract rounding at small capital — at $24.09, quarter-Kelly sizes round to
zero for most signals (dry run: 124 of 145 fires, 85.5%). Live sizing therefore uses the **re-scored sizing table's**
fraction, chosen at the flip; quarter-Kelly stays the comparison arm in the record; the 100%-deploy arm stays a simulated
comparator and never drives live orders. **The daily loss stop is untouched.** The D6 addendum is amended to read:
'quarter-Kelly is the floor reference; the live fraction is set at the flip from the re-scored sizing table; the daily loss
stop is unchanged.' Funding and sizing remain Cayleb's decisions; no live order on this ruling."
*Evidence:* `DECISIONS.md` D6 (Cayleb verbatim, 100% deploy) vs `Amendment 1` D6 addendum ("capped quarter-Kelly only") vs
the recorded posture in the brief — a three-way contradiction, and the D6 addendum is the text that must change.
`SPEC.md:153` quarter-Kelly definition; `live/kalshi_exec/sizing_live.py`; dry run `sized_to_zero 124/145`.

**R4 (provisional L\* = 30 s — correction to the 13:17 ET confirmation).** *Ruling asked:* "The provisional latency row
for θ/X selection becomes **L\* = 30 s**, replacing the 60 s reading confirmed 2026-10-31 13:17 ET, on measured
availability: decisions readable 16.127 s p50 / 16.735 s p90 after the decision tick (`results/availability_2026-10-31.json`;
81 decision lines, 9 ticks, 10 min) and 24.586 / 27.713 s after `a_asof`; with intent pull ~1–2 s and gateway <1 s the
end-to-end from the tick is ≈ 19–20 s, and 30 s is the smallest row of record at or above both the end-to-end figure and
the data-time p90. The 60 s row stays a row of record but is no longer the θ/X selection row. Per Amendment 3 item 1, if
the gateway's measured end-to-end latency lands above the row in use, the threshold re-derives at that row **before any
flip**. Rows 2 s and 5 s stay informational. Reversible before the freeze; logged in DECISIONS.md."
*Evidence:* `availability_2026-10-31.json` (quoted above); `DECISIONS.md` "Confirmed by Nox (13:17 ET): provisional
L\* = 60 s"; `scorer_agreement.json` `latency.L_star: 60`.

---

## 6. Job 12 — the one decision the Lead must batch to Cayleb (not asked tonight)
**X1.** The deployed engine cannot execute autonomously. Options: (a) run Monday in `cards` (real execution data; the
"first autonomous fills" proof cannot be met and the same human-tap cause repeats); (b) authorise a new Tier S autonomous
order path first (checker + dry run + go), Monday becomes guard/preflight only; (c) hold Monday. My recommendation:
(a) for Monday's data + (b) as the successor build. Nothing is asked tonight (Saturday, no live action); this is for the
Lead's consolidated ask.

## 7. What blocks what (and on what)
| Item | Status tonight | Blocked on |
|---|---|---|
| 12 Monday run | Runbook ready (C4) | checker VETO signature; the X1 decision; a live Tradovate tab/feed Monday (not tonight) |
| 12 MNQ / Kronos / Round 3 / settle-or-close | not started | Phase-2 go; MNQ needs qualifying bars or the feed-lane's evidenced failure report; Kronos needs the frozen run completed (20,667/22,200 at brief) |
| 13 ground truth + v5 | BLOCKED-ON-MACHINE | no machine route (X5); data unreadable; Nox-mediated or desk-side access required |
| 14 rulings + resume | drafting done; re-run done | Nox's ruling on R1–R4; the DST/rollover instants occur tonight/early Sun — real re-run Monday; X6's file location |
| 15 pre-checks | master + repo-side checklist verified; goal-file missing | desk machine (all deployed-artefact checks); Job 13 stable before it (Section 6.1) |

## 8. Checker / maker discipline tonight
- Nothing tonight is a Tier S irreversible step (all reads + local test runs): no order, no wallet, no flip, no merge, no
  deploy happened, so no pre-check signature was required for this batch.
- **Before the Monday Tier S steps** (guard/preflight/mode-switch/window/flat/outcome), the Programme Lead must spawn a
  fresh-context **trading-checker with VETO** and it must sign the pre-check evidence **before** the step (Tier S §4.2;
  TEAM-ASSIGNMENTS "Checker seats"). None is spawned; no item in this report is DONE (maker never checker).
- The checker should verify from the venue/desk records named in the runbook, not from this report's summaries.

## 9. Spend
$0.00 used of a $0 new-funds envelope (reads, local `readiness.py`, local `unittest` runs). No purchases, no accounts, no
funds, no key handling, no public sends. Nothing written outside `board/reports/` + `board/probes/` (+ the two probe logs
under `board/probes/team-c/`). The `openbell-kalshi` worktree's `results/` was NOT overwritten: the readiness JSON was
written to `board/probes/team-c/` via `--out`.

## 10. Next step + successor
Successor (Phase 2, on the Lead's go): (1) the X1 decision + checker spawn, then strictly: Monday guard -> preflight ->
mode step -> window -> flat -> outcome per the runbook; (2) Job 14: apply R1–R4 once ruled, then resume pack (self-check
re-run at/after the real instants, fit-split pull restart command from RESUME §Pending 1, unit-state check already
evidenced); (3) Job 13/15 wait on the desk-machine route — the Lead should batch one question: **what is the approved route
to the desk machine / who is the relay?** Until then they stay BLOCKED-ON-MACHINE with the evidence above.
A quiet Saturday night was expected; no live action exists and none was manufactured.
