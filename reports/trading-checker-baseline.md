# TRADING-CHECKER — INDEPENDENT BASELINE (Team C, Tier S throughout)

Checker: `trading-checker` (fresh context; did not build any of this). Report to `lead`, never to Cayleb.
Written: **2026-10-31 19:20-19:40 ET / 23:20-23:40Z** (Saturday night, market closed).
Scope: V1-V6 of the baseline batch. **Nothing here certifies any step.** This is the checker's baseline: what I
reproduced myself, what matched, what did not, and the conditions I will demand before signing.

Method rules I held to: read-only against live trees; no broker session, no wallet, no credential, no order; every
claim below is a command I ran and its output, not the maker's summary. My writes went only to
`board/probes/trading-checker/`, `/tmp/checker-dryrun/` and `/var/tmp/tc-tmp/`; I did not modify either repo (the dirty
state recorded in §7 is the makers').

Claims audited (read as CLAIMS, not facts): `board/reports/team-c-phase1.md` (C1-C5),
`team-c-c6-c10.md`, `team-c-kalshi-resume-pack.md`, `team-c-autonomous-path-design.md`,
`board/reports/monday-futures-runbook.md` (v4), plus the maker artefacts under `board/probes/team-c/`.

---

## 0. Verdict summary

| # | Baseline item | Result | One-line basis |
|---|---|---|---|
| V1 | Kalshi readiness re-run | **MATCH** | 10 checks, 9 PASS / 1 NA / 0 FAIL, EXIT=0, check names identical to maker's |
| V2 | Unit state (user units) | **MATCH** | 3/3 `enabled` AND `active`; system scope 0 unit files; `FragmentPath` in `~/.config/systemd/user/` |
| V3 | Shadow + WP10 tests, dry-run, live path OFF | **MATCH** | 247 OK, 167 OK; my own fresh dry run byte-identical to the maker's 7/7 data files + 2/2 outbox; `require_live` → `not_enabled` |
| V4 | Two-scorer agreement + availability | **MATCH with wording corrects** | my own `compare_scorers.py` run: `rows A 6853 B 6853; differences 0; AGREE`; availability file figures confirmed; "identical" and "≈19-20 s" are loose (see §4.2-§4.3) |
| V5 | Monday futures runbook honesty/completeness | **NOT FIT TO SIGN** | 5 attestations are NOT each separately evidenced; two cite evidence that cannot exist at the stated time; the completeness check is not a pass/fail (D1-D15 in §5) |
| V6 | Veto conditions in writing | **DELIVERED** | §6 |

**Baseline verdict for the Lead: V1-V4 evidence is real and reproducible; V5 must be fixed before Monday 09:05, and
until it is fixed my VETO stands on the Monday step.** Detail and exact discrepancies in §5 and §7.

Environment note (affects every team): **`/tmp` is a 32 GiB tmpfs at 98% inode use (`df -i /tmp` → 1,018,625/1,048,576
IUsed, 29,951 free)**; I hit one hard `ENOSPC: no space left on device` from `mkdtemp` at 19:28 ET. The big trees are
other teams' (`/tmp/ergon-*` ≈ 2-4 GB each, `/tmp/claude-1000` 3.8 GB). I did not delete anything outside my own paths;
I moved my scratch to `/var/tmp/tc-tmp`. Any teammate launching subprocesses/temp trees can fail at random while this
stays full — this is a shared-resource risk for the 09:05-16:20 Monday window, not a trading risk.

---

## 1. V1 — Kalshi readiness, independently re-run

Command (my own output path; `readiness.py` is read-only on live folders):

```
$ cd ~/Work/projects/openbell-kalshi
$ python3 -I research/kalshi15/readiness.py --day 2026-10-31 \
    --out .../board/probes/trading-checker/readiness-checker-2026-10-31.json
readiness 2026-10-31  read at 2026-10-31T22:52:05Z  observed 2026-10-31T12:33:55Z..2026-10-31T22:52:05Z  ledger tick 2026-10-31T22:51:00Z
cutoff  capture books 2026-10-31T22:52:05Z  ref 2026-10-31T22:52:05Z  ledger decisions/orders tick 2026-10-31T22:51:00Z  (4816/35369 complete lines)
PASS  capture_validator     5 files, 3009093 records (...), errors 0
PASS  gap_ledger            unhealthy-book share worst 0.033% (HYPE) over 9 coins ...; gap records 6 ...
PASS  ref_health            seconds with n>=1: worst 99.973% (BNB) ...; venue connects 34 disconnects 27 errors 20; clock ...
PASS  settlement_reconcile  441 windows settled in the day: result vs rule 441/441, shadow vs result 369/369 compared ...
PASS  proxy_wrong_side      wrong-side per coin ... (all coins far under the 10% warn line; report only)
PASS  replay_determinism    byte-identical over the common prefix: decisions 4816 lines, orders 35369 lines ...
PASS  chain_integrity       decisions: 4825 lines, head 94b0422d23739fbc; orders: 35369 lines, head f2432d4ad80eb2ac
NA    nox_reconcile         no time overlap with Nox's clean cycles; settlements both 45 (result disagree 0, value disagree 0)
PASS  counts                healthy settled windows BNB=40 BTC=40 ... ZEC=40; days to 2,000 ... (ideal 21.1 d at 94.9/day)
PASS  heartbeats            heartbeat age poll=0.9s ref=0.7s shadow=1.7s; errors_1h poll=0 shadow=0
EXIT=0
```

Comparison to the maker's log (`board/probes/team-c/readiness-2026-10-31.log`, read at 21:54:29Z):

| # | Check | Maker (21:54:29Z) | Checker (22:52:05Z) | Same verdict? |
|---|---|---|---|---|
| 1 | capture_validator | PASS | PASS | yes |
| 2 | gap_ledger | PASS | PASS | yes |
| 3 | ref_health | PASS | PASS | yes |
| 4 | settlement_reconcile | PASS | PASS | yes |
| 5 | proxy_wrong_side | PASS | PASS | yes |
| 6 | replay_determinism | PASS | PASS | yes |
| 7 | chain_integrity | PASS | PASS | yes |
| 8 | nox_reconcile | NA | NA | yes (same reason: no overlap; Nox stale) |
| 9 | counts | PASS | PASS | yes |
| 10 | heartbeats | PASS | PASS | yes |
| — | totals / exit | **9 PASS / 1 NA / 0 FAIL, EXIT=0** | **9 PASS / 1 NA / 0 FAIL, EXIT=0** | **yes** |

Every numeric difference between the two runs is explained by 58 minutes of live capture between the reads, not by a
disagreement: records 2,672,540 → 3,009,093; settled windows 405 → 441; ledger decisions 4,375 → 4,816; orders
31,397 → 35,369; worst per-coin wrong-side ratio 3/36 → 3/40 (ZEC 8.3% → 7.5%, still report-only). Check 8's detail is
identical in substance: `settlements both 45 result_disagree 0 value_disagree 0 (ours 441, nox 154)` — Nox's copy is
stale, NA by construction, as the maker said.

**Honest limit (confirmed, not just repeated):** this run does NOT exercise the two owed *clock-instant* checks. The
`--day 2026-10-31` run covers windows closing inside the UTC day; the 20:07 ET (00:07Z) rollover and the DST hour
(05:00-07:00Z) are still owed and must be run by their UTC instants, not their old ET labels. The maker's pack says this
correctly (including the arithmetic correction that "03:12 ET" is 08:12Z EST / 07:12Z EDT, never 06:00Z) — I checked the
arithmetic against the host TZ, it is right.

**Discrepancy V1-D:** none. The maker's "9 PASS / 1 NA / 0 FAIL, EXIT=0" is reproduced exactly.

---

## 2. V2 — Unit state, correct scope (USER units)

```
$ for u in openbell-kalshi-poll openbell-kalshi-ref openbell-kalshi-shadow; do
    systemctl --user is-enabled $u.service; systemctl --user is-active $u.service
    systemctl --user show $u.service -p UnitFileState -p ActiveState -p SubState -p ExecMainStartTimestamp -p FragmentPath; done

openbell-kalshi-poll    enabled | active | UnitFileState=enabled ActiveState=active SubState=running
                        ExecMainStartTimestamp=Sat 2026-10-31 12:23:45 EDT
                        FragmentPath=/home/cayleb/.config/systemd/user/openbell-kalshi-poll.service
openbell-kalshi-ref     enabled | active | UnitFileState=enabled ActiveState=active SubState=running
                        ExecMainStartTimestamp=Sat 2026-10-31 12:23:45 EDT
                        FragmentPath=/home/cayleb/.config/systemd/user/openbell-kalshi-ref.service
openbell-kalshi-shadow  enabled | active | UnitFileState=enabled ActiveState=active SubState=running
                        ExecMainStartTimestamp=Sat 2026-10-31 13:34:13 EDT
                        FragmentPath=/home/cayleb/.config/systemd/user/openbell-kalshi-shadow.service

$ systemctl list-unit-files 'openbell*'        # SYSTEM scope
UNIT FILE STATE PRESET
0 unit files listed.
$ ls /etc/systemd/system/openbell*             # no such file or directory
```

Comparison to `team-c-phase1.md` (C1.1): **MATCH** — enabled AND active for all three, user scope, system scope empty.
The maker's "X8 correction to GROUND-STATE" is correct (they are user units).
Two extra facts the maker's line does not state, noted for the record (not discrepancies): the shadow unit's current
incarnation started **13:34:13 EDT** (restart, consistent with `ledger-v1-20261031T1734Z`), while poll/ref started
12:23:45; and the units are `Type=simple` services, no timers.

**Discrepancy V2-D:** none.

---

## 3. V3 — Tests, dry run, byte-identity, live path OFF

### 3.1 Suites (my runs, `PYTHONDONTWRITEBYTECODE=1`)
```
$ python3 -m unittest discover -s live/kalshi_exec -p "test_*.py" -t .   → Ran 167 tests in 0.898s — OK (EXIT=0)
$ python3 -m unittest discover -s live/kalshi                            → Ran 247 tests in 29.701s — OK (EXIT=0)
```
Matches the maker's 167 and 247. The maker's note that `RESUME.md` says "172 tests" while discovery finds 167 is
confirmed as a count-of-test-methods matter; I also got 167. No failures, no skips reported differently.

### 3.2 Dry run reproduced by me from scratch (not their rerun)
```
$ python3 -I live/kalshi_exec/engine.py dryrun --theta 0.06 --L 60 \
    --from 2026-10-31T12:34:00Z --to 2026-10-31T17:30:00Z --out /tmp/checker-dryrun
dryrun theta=0.06 L=60: 145 fires, 124 sized to zero, 8 intents, 0 mock fills, 0 settlements,
final equity $24.0900, reconcile OK                                             (EXIT=0)
```
Byte comparison of **my fresh run** vs the maker's `live/kalshi_exec/state/dryrun/` (sha256):

| file | result |
|---|---|
| exec_events.jsonl | SAME (9a44e447…) |
| latency.json | SAME (4c1b6795…) |
| refusals.jsonl | SAME (0ef0bd6d…) |
| summary.json | SAME (3eabbaf9…) |
| venue-state.jsonl | SAME (cf1cc1c7…) |
| would_be_orders.jsonl | SAME (6366fd65…) |
| outbox/intents.jsonl, outbox/chain_head.json | SAME (c3fa5c7f…, cd27ceb1…) |
| compare.json | maker file b4dc4675…; my `compare.py` output semantically identical (below) |

I then re-ran the comparison itself against the live ledger:
```
$ python3 -I live/kalshi_exec/compare.py --would-be /tmp/checker-dryrun/would_be_orders.jsonl \
    --ledger live/kalshi/ledger --theta 0.06 --L 60 --out /tmp/checker-dryrun/compare.checker.json
{ "L": 60, "ok": true, "missing": 0, "extra": 0, "price_or_side_mismatch": 0, "explained": 145,
  "shadow_orders": 145, "matched_same_ticker_side_price": 8, "refused_with_reason": 137,
  "refusal_reasons": {"sized_zero:below_one_contract": 124, "would_cross": 13}, "would_be_orders": 8 }   EXIT=0
```
Semantic diff against the maker's `compare.json`: identical (all keys/values). The maker's "byte-identical rerun"
claim is thus confirmed **and strengthened**: it is reproducible from a clean output dir, not only under their rerun.

### 3.3 Live path OFF by construction (checked structurally AND exercised)
```
$ cat live/kalshi_exec/exec.json
{"enabled": false, "environment": "demo", "role": "desktop"}          # == config.SHIPPED
$ ls live/kalshi_exec/GO.json live/kalshi_exec/verdict.json live/kalshi_exec/STOP   → none exist
$ KALSHI_EXEC_LIVE=I_UNDERSTAND_THIS_PLACES_REAL_ORDERS python3 -  (load config, guard.require_live(default home))
cfg: False demo desktop | has GO: False has verdict: False has STOP: False
REFUSED: not_enabled
$ (same probe with the env var UNSET)                                  → REFUSED with no env: not_enabled
```
Plus the package's own static proofs passed inside the 167 (network confinement: only `transport_live.py` may import a
network library; every public `LiveVenue` method starts with `require_live`; the lazy transport import is inside one
private method only). `transport_live.UrllibTransport.send` requires a `LiveToken` minted only by `require_live` and
refuses any host other than the token's environment. **The shipped build cannot place an order tonight** — that is my
independent finding, not the maker's claim.

**Discrepancy V3-D:** none.

---

## 4. V4 — Two-scorer agreement and the availability behind L* = 30 s

### 4.1 Agreement artefact — re-verified by running the comparator myself
`research/kalshi15/results/scorer_agreement.json` records `rows: 6853`, result
`"AGREE: 6853 rows each, 0 differences; summaries and the late side file identical (tol 1e-9, integers exact); six
latency rows {2,5,10,30,60,90} (2 and 5 informational)"`. I did not take that string: I ran the comparator on the two
scorers' actual output files (first scorer under `research/kalshi15/results/`, second under `data/kalshi15/derived/`):
```
$ python3 -I research/kalshi15/compare_scorers.py \
    research/kalshi15/results/first_scorer data/kalshi15/derived/second_scorer
rows A 6853 B 6853; differences 0
AGREE                                                                                     EXIT=0
```
Supporting hashes: first and second `rows.jsonl` are byte-identical (`740a7aa5…` both); the artefact's recorded
`first_scorer.{rows,summary,late}` hashes match the files exactly, and the recorded `second_scorer.{late,summary}`
hashes match too (`965477b0…`, `fd5c6549…`). The comparator is not vacuous: its own suite
(`test_compare_scorers`, 7 tests) plants a 1-centi and a 1-milli change and expects them caught — **7 tests OK, EXIT=0**
when I ran it.

Correction of wording (minor, not a failure): the artefact says the summaries and the late side file are "identical",
but their sha256 differ (first summary `f54c1a02…` vs second `fd5c6549…`; late `ff13520d…` vs `965477b0…`). What is
true is what `compare_scorers.py` checks: identical within `tol 1e-9` on summary values and on the three defined late
keys (`fired_prints`, `placed`, `late`), with scorer-private `_`-keys ignored. So "numerically identical under the
declared tolerance", not "byte identical". The maker's own method section hints this; the word "identical" should be
narrowed in the design text.

### 4.2 Availability measurement behind L* = 30 s — numbers verified from the file
The artefact is `research/kalshi15/results/availability_2026-10-31.json` (written by
`live/kalshi_exec/measure_availability.py`, which only reads the ledger and polls `decisions.jsonl` every 0.25 s):
```
available_after_tick_s:      p50 16.127187967300415, p90 16.734870672225952, max 16.825629234313965, min 15.270883321762085
available_after_data_time_s: p50 24.5862033367157,   p90 27.712876081466675, max 28.805349826812744
decision_lines 81, ticks 9, minutes 10.0
```
The maker's quoted figures 16.127 / 16.735 / 24.586 / 27.713 are **exact**. The derivation to "≈19-20 s end-to-end" is
`tick p90 16.735 + pull 1.5 + gateway 0.5 = 18.735 s` (p50: 18.127 s), i.e. honestly **≈18-19 s**, not 19-20; the
`max` gives 18.83 s. Tiny, but a checker does not let a rounded-up number stand unqualified.

Three qualifications that matter for the ruling (all defensible, but must be on the record next to L* = 30):
1. **Sample is thin.** 9 ticks / 81 decision lines / one 10-minute window (measured ~13:4x ET Saturday, services healthy).
   p90 over 9 samples is essentially the 2nd-worst sample; no confidence interval, no repeat. It is a real measurement,
   not a distribution.
2. **Two of the three hops are assumed, not measured.** `pull_s 1.5` and `gateway_s 0.5` are dry-run defaults; the real
   gateway has not been built (WP10 is a simulator). So "≈19 s end-to-end" is measurement + assumptions, not end-to-end
   measurement. The design text says the gate must re-derive at the gateway's *measured* end-to-end latency — that
   sentence is the load-bearing one and must survive editing.
3. **The margin at L*=30 is 0.3 s, not 2.3 s.** Nox's ruling (batch-2 answer 6: "30 s sits above the data-time p90 of
   27.7 s") compares 30 to 27.713. Add the two assumed hops and the data-time end-to-end is
   `27.713 + 1.5 + 0.5 = 29.713 s` — only **0.287 s** under the 30 s row. If the live measurement reproduces 27.713 or
   drifts up by >1%, the 30 s row no longer covers it and the row must go to 60 until re-derived. L* = 30 is
   therefore **provisional and fragile**, which is exactly how the maker labels it, but the fragility should be stated
   as 0.3 s, not "above the p90".

4. **A second, contradictory-looking end-to-end number exists in the WP10 summary and must not be mistaken for a
   measurement.** `live/kalshi_exec/state/dryrun/summary.json` has
   `end_to_end_latency: {p50 64.488372, p90 71.307457, engine_s p50 62.488372, pull 1.5, gateway 0.5, row 90,
   row_note: "the smallest preregistered row at or above the end-to-end p90"}` — while the same file says `row_L: 60`.
   This is not a latency measurement: in the dry-run model `gateway_sent = decision tick + L`, so
   `end_to_end = L + (t_f - a_asof)` **by construction** (60 + ~4.5 s at p50 = 64.5; 60 + ~11.3 s at p90 = 71.3).
   The reported "row 90" is a circular output of having run at L=60, not evidence that the real end-to-end is 64-71 s.
   Read as a measurement it would contradict the 16.1/16.7 s availability and push the gate to row 90. **Any gate text
   that quotes 64.5 s as "the end-to-end" is wrong; the number is `L + data-to-tick lag`.** I will veto a gate that uses
   it as a measured figure.

**Discrepancy V4-D:** (a) "identical" should read "identical within tol 1e-9 on the defined keys" (summary/late bytes
differ); (b) "≈19-20 s" should read ≈18-19 s from the file; (c) the dry-run `end_to_end_latency` block is a
construction and is not labelled as such — label it or remove it from the gate evidence. None of these undoes the
agreement result or the L* = 30 ruling (Nox independently confirmed it), but they are corrections to the record.

---

## 5. V5 — Audit of the Monday futures runbook (`monday-futures-runbook.md` v4)

### 5.1 The five attestations: is each separately evidenced with a concrete source?

| Att. | Runbook line | Evidence source it names | Does the source produce it? | Verdict |
|---|---|---|---|---|
| (a) | 51-56 | `live/account_state.json` (`status`,`accounts`,`positions`) + last frame of `live/ledger/account-2026-11-02.jsonl`; PASS = status≠no-tab, amount 50000.0, amountSOD 50000.0, realizedPnL 0.0, positions {} | command works, **but PASS can be satisfied by restored/stale data** (see D7/D8) | **WEAK** |
| (b) | 57 | "heartbeat `t` age < 60 s, `status: running` (command in P0.4)" | **P0.4 prints `pos` and `card_out`, not `t`/`status`** — the named command does not produce this evidence (D2) | **NO SOURCE** |
| (c) | 58-85 | 4 legs: tabkeeper.json tab not missing; `ss` :9222 listener; heartbeat feed not retrying + `feed_age_s < 30`; fresh bar ≤120 s old with ET time ≥ 09:30; one-liner prints PASS/FAIL | sources exist and the one-liner self-computes PASS/FAIL — **but the bar leg cannot be satisfied at 09:15, before 09:30** (D3) | **CONTRADICTORY** |
| (d) | 86-88 | heartbeat `paused:false`/`status:running`; "no STOPPED card today in `live/logs/cards.jsonl`"; "today's startup line in the engine log reads `rails: mode cards, 1 MES, max 2 trades/day, day stops -400 / 900, cooldown 15 min after a loss, min stop 2.0 pts, flat 16:05 ET`" | paused/status exist; **no `rails:` line exists anywhere in the engine code or log** (`grep -c "rails:" live/logs/engine-2026-10-28.log` → `0`); engine log is named by *start* day (`engine-<day>.log`) and the process started 28 Oct (D4); "no STOPPED card" has no date filter and the STOPPED card fires only in one narrow case (D5) | **FABRICATED SOURCE** |
| (e) | 89-92 | `live/config.json` `mode` prints `cards`; text "mode = cards — notify card + human-placed bracket; **NOT automated** (the engine has no order path)" | command prints `cards` tonight; the "no order path" claim is TRUE — the feed's CDP allow-list is `{Network.enable, Network.disable, Runtime.evaluate}` with "Never Input.*, Page.*, Storage, cookies"; engine docstring "It NEVER places, modifies or cancels orders"; only `shadow|cards` modes | **GOOD** |

So: **1 of 5 attestations is fully evidenced (e); (a) has a real command but its PASS criteria can be met by restored
data (D7/D8); (b) names no command that produces its evidence (D2); (c) is internally impossible at its scheduled time
(D3); (d) cites a log line that does not exist in the code or the log (D4).** The Lead's ruling that (e) must say
"mode=cards — NOT automated" **is honoured** — line 90-91 says it plainly:
`"(e) mode = cards — notify card + human-placed bracket; NOT automated (the engine has no order path; Lead ruling X1)."`

### 5.2 Is the failure disposition right if the Tradovate tab/feed is absent?
Right in substance, wrong on the recovery route.
- Right: do not sign in; STOP FOR CAYLEB on a fresh credential/2FA; record tabkeeper + last account frame + the
  `no trader.tradovate.com tab` engine line; day BLOCKED-ON-MACHINE, no trade; "no other host, no other browser, no
  shadow substitute, no manufactured fills"; decision log "declared empty (not complete)"; escalate via the Lead.
- Wrong/misleading: the disposition's first sentence routes the sign-in click **"Nox -> Outis/Codex with the existing
  session"**. `board/GROUND-STATE.md` M0 records **outis (Windows) offline 151 d** and no SSH/alias route
  (`ssh muse` does not resolve); M2 records the **B12 lane (Outis/Codex desktop) SILENT since Fri 17:04 ET, 21h+ at
  probe**. So the documented recovery route is on a lane that is dark by the programme's own ground state. The real
  route is Cayleb signing in on the desktop Chromium tab (which the disposition does name last) — the runbook should
  say that first and mark the Nox/Outis route UNVERIFIED (D12).
- Note: the browser debug port **is** currently listening (`ss -ltnp | grep :9222` → `LISTEN … 127.0.0.1:9222
  users:(("chromium",pid=2960519))`) while `live/logs/tabkeeper.json` says `"tradovate": "missing", "alerted": true,
  "lane_logout": true`. So P0.2's two legs are not equivalent: the port can be up with no tab. P0.2/P0.3 as written are
  fine; the risk is treating "port up" as "session live".

### 5.3 Is the decision-log completeness check defined (what makes it pass/fail)?
**No.** Runbook 114-123 says: "`grep -c '"event"' live/ledger/2026-11-02.jsonl` and `wc -l live/logs/cards.jsonl`:
every signal must have a card and a ledger outcome (`fill`/`cancel`/`skip`). Signals:cards:outcomes -> complete, or
every gap named." Four separate holes:
1. **Wrong command.** `live/logs/cards.jsonl` is cumulative, not per-day. Tonight it has **26 lines**; Friday's own
   cards were **2** (entry 10:11, exit 10:26) while Friday's ledger has **3** events (`grep -c '"event"'` = 3). The
   command as written compares 3 vs 26 and cannot mean anything. It must filter cards by `t` for the session day.
2. **"or every gap named" is not a pass criterion.** It lets the check pass by *narrating* gaps; a FAIL state does not
   exist. Define: PASS ⇔ for each ledger `signal` there is exactly one card with the same `tag`/`bar_ts` and a terminal
   ledger outcome (`fill`/`reject`/`exit`/`blocked`); every unmatched item is FAIL-INCOMPLETE and named.
3. **No independent enumeration of "signals".** The engine is the only source of signals (the ledger); if the engine
   drops a signal, nothing in this check can see it. So the check proves *internal consistency*, not completeness. If a
   verifiable count is required, it needs a second source (re-run the same strategy on the same bars and compare signal
   sets), or the claim must be weakened in writing to "every logged signal has a card and an outcome".
4. **The file may not exist when the check runs.** `_ledger()` creates `ledger/<session-day>.jsonl` lazily on the first
   event; there is **no start/decision record** at engine start (`engine.py` writes only signal/fill/reject/exit/blocked).
   At 09:15 on a quiet morning the file does not exist; `grep -c` on a missing file is an error, not a result. The
   "Decision logging ON" step (93-94) is therefore **unsatisfiable before the first signal** (D6) — and the quiet-day
   path ("no file") is not given a verdict.

### 5.4 Numbered defects (the veto list for the runbook)

Severity: **[B]** blocker for a confident signature, **[M]** material, **[m]** minor.

| # | Sev | Defect (line) | Fix I will require |
|---|---|---|---|
| D1 | **B** | P0.4 and 09:05 item 3 test `card_out` in `heartbeat.json` (34, 41). The field does not exist: `heartbeat()` writes `pos, day_pnl, trades, paused, feed, feed_age_s, market_open, last_bar_et, …` but **not** `card_out`. Ran the exact command: prints `None None`; the stated pass evidence is "None False". | Drop `card_out`, or add it to the heartbeat writer and re-test. PASS must be reproducible verbatim. |
| D2 | **B** | Attestation (b) says "command in P0.4" but P0.4 prints only `pos`/`card_out` — not `t` age or `status`. (57) | Give (b) its own command (`heartbeat.json` t/status, age computed) and quote it. |
| D3 | **B** | Attestation (c) requires "a fresh bar with a timestamp AFTER the open (09:30 ET)" but the five attestations are scheduled at **09:15**. At 09:15 the condition is unsatisfiable; the only later (c) run is the 09:40 re-verify, and the runbook does not say the 09:15 (c) is provisional and 09:40 binding. (58-85, 96-100) | Split (c): 09:15 records tab/port/feed/heartbeat; the **binding** (c) is re-run 09:35-09:44 with the post-09:30 fresh bar. The window must not open without it. |
| D4 | **B** | Attestation (d) quotes a startup line — "`rails: mode cards, 1 MES, max 2 trades/day, day stops -400 / 900, cooldown 15 min after a loss, min stop 2.0 pts, flat 16:05 ET`" — that does **not exist**. `grep -c "rails:"` on the live log = **0**; the engine never logs rails. The real start signal is an ntfy card "OpenBell engine started" with body `mode=cards strategy=… qty=1 MES, flat by 16:05 ET` (no stops/cooldown/min-stop/trades), plus config.json. Also the engine log filename is `engine-<start-day>.log`: the live process **started Wed 28 Oct 15:40:08** and is still the same PID 1484 — on Monday there is no "today's startup line" unless the engine is restarted. (86-88) | Replace with: (i) `systemctl --user show openbell-live -p ExecMainStartTimestamp,ActiveState` paired with the log file for that same start day; (ii) read `config.json` and assert every rail field + `symbol=MESZ6`, `contract_id=4470322`; (iii) quote the actual "engine started" card body. If a restart is performed, the startup line requirement becomes satisfiable — say which. |
| D5 | **M** | (d) "no STOPPED card today" has no date filter (command missing), and the STOPPED card fires only in the narrow case "stopped with shadow position open" (`engine.py:346`) — a crash or clean stop leaves no STOPPED card, so the check cannot prove "engine not stopped". (86-88) | Use systemd state + `heartbeat.paused` + the "session day … pnl … trades …" log line; drop or date-filter the STOPPED grep. |
| D6 | **B** | "Decision logging ON — today's `live/ledger/2026-11-02.jsonl` exists with a start/decision record" (93-94) is impossible at 09:15: the file is created lazily on the first event and **no start record is ever written**. A quiet morning has no file at all. | Define pass as either "file exists with ≥1 record" or "declared not-yet-created (no signal) and the start card + `cards.jsonl` entry exist"; give the missing-file case a verdict. |
| D7 | **B** | Attestation (a) PASS can be satisfied by **restored/stale** data: `account.py` restores the last balance from `account_state.json` marked `"source": "restored"`, and `status` merely ≠ "no trader.tradovate.com tab" allows `"starting"` / `"retrying (…)"` / `"attached"`. PASS does not require `status == "attached"`, `source == "live"`, freshness (`accounts[*].t` after 09:00) or the account id. Tonight's file shows exactly the trap: `status "no trader.tradovate.com tab"`, account `71223147`, `amount 50000.0`, `amountSOD 50000.0`, `realizedPnL 0.0`, `tradeDate 2026-10-26`, `t` = Fri 15:29:12 ET. (51-56) | PASS ⇔ `status == "attached"` AND `source == "live"` AND key `71223147` AND `accounts["71223147"]["t"]` after 09:00 AND (quote `tradeDate`; if still 2026-10-26, record the anomaly — the venue's own cashBalance `timestamp` is `2026-10-25T21:30:13.058Z`). |
| D8 | **B** | `positions {}` is not a proven flat-detector. No `position` frame exists in **any** `live/ledger/account-*.jsonl` (10-27…10-30 kinds: contract/account/cashBalance/accountRiskStatus/userAccountAutoLiq/dom — zero position frames), so `positions` is the **default empty dict**, never positively shown to go non-empty. "Absence from the record is not evidence." | Keep `positions {}` but pair it with an independent broker-surface check (positions panel screenshot/DOM read) before writing "FLAT"; or state that `positions {}` is unproven as a detector. |
| D9 | **B** | 09:05 item 3 asks the last frame of `/live/ledger/account-<prev>.jsonl` to show "`positions {}` / `realizedPnL 0.0`". That frame schema has **no `positions` key** (account frame keys: `id, active`; cashBalance keys: `id, accountId, amount, amountSOD, realizedPnL, tradeDate, timestamp`). Also "last frame" Friday is a **dom** frame at 17:52:13 (`equity 0.0`), not the 15:29:12 balance frame — so "broker flat Friday" is **not evidenced after 15:29**. (41-42, 120-122) | Point at the `cashBalance` frame, name its `t`, and require a frame time; state plainly that after the last balance frame the broker state is unverified (this is exactly the owed Friday reconciliation). |
| D10 | **B** | 16:20 headline completeness is not defined as pass/fail (see §5.3) and the command is wrong (cumulative cards file). (114-123) | Rewrite as §5.3 items 1-4. |
| D11 | **M** | 16:05 flat check has no freshness requirement: "last frame positions {} / cashBalance" could be hours old (Friday: last balance 15:29, last dom 17:52). A stale flat frame passes a flat check while a late fill goes unseen. (109-112) | Require a frame with `t` **after 16:00** and quote it; if none, record "unmanaged/unverified", never "flat". |
| D12 | **M** | Failure disposition's recovery route ("the click routes Nox -> Outis/Codex") is on a lane GROUND-STATE records as dark (Outis offline 151 d; B12 silent 21h+). (125-132) | Say Cayleb's desktop sign-in is the only verified route; mark Nox/Outis UNVERIFIED. |
| D13 | **M** | No blanket gate for the five attestations: 09:15 says "write each attestation" but only (c) and (e) carry an explicit stop. A reader can proceed with (a) or (d) failed. (49-50) | Add one sentence: "Any attestation FAIL = the day does not start (window not opened); record which leg failed." |
| D14 | **M** | No config-vs-rails assertion of `symbol=MESZ6` / `contract_id=4470322` (a wrong contract id would card the wrong instrument), and no read of the firm rails ($1,000 day cap, 40% consistency) even though the runbook lists them as rails. (17-22, 86-92) | Add to (d): read `config.json`, assert every field in the rails table + symbol/contract_id; state the firm caps are firm-side and not engine-enforced, and that the engine stops are the only enforced ones. |
| D15 | **B** | 09:05 item 4 requires "Market closed pre-open — heartbeat `market_open: false`", but `live/bars.market_open()` is the **CME Globex** session (Sun 18:00 → Fri 17:00, daily halt 17:00-18:00). I called it for Mon 2026-11-02: `09:05 True, 09:15 True, 09:40 True, 16:05 True, 16:20 True`; Saturday 19:30 → False. So item 4 **FAILS every Monday morning**, and the guard's own rule is "Any FAIL -> stop". (43) | Invert: "Globex session alive (`market_open: true`) and the 09:30 regular session has not yet opened"; drop "market closed". |

Additional observation (not a defect by itself): the engine has been up **3 days** (PID 1484 since Wed 28 Oct 15:40:09)
and `_roll_day()` resets `day_pnl`/`trades` only when a **closed bar** arrives (`engine.py:138-159`). Tonight's
heartbeat still carries Friday's `day_pnl 102.5, trades 1`. That reset is automatic on the first Monday bar — but the
runbook should make it *visible*: require the log line `session day 2026-11-02: pnl 0.00 trades 0` (and
`heartbeat.day_pnl == 0`, `trades == 0`) before the window opens. It is a free, strong proof that the feed is live and
the day has rolled; the runbook currently misses it.

### 5.5 What would let an unverifiable claim slip through as a pass (explicit list)
1. A restored/stale `account_state.json` balance (source `"restored"`, Friday 50000/0.0) under attestation (a).
2. `positions {}` by default — no position frame has ever been observed, so emptiness is not evidence.
3. `status ≠ "no trader.tradovate.com tab"` accepts `"starting"`/`"retrying (…)"` as good.
4. "complete, or every gap named" at 16:20 — narration substitutes for a criterion.
5. The engine's own word for mode/armed: reading `config.json` does not prove the running process loaded it (needs
   process start time + log pairing — D4).
6. The dry run's `end_to_end_latency` (64.5 s) read as a measurement instead of `L + data-to-tick lag`.
7. The provisional L* = 30 used as the gate row after the flip without a live re-measurement (margin 0.29 s).
8. A stale "last frame" at 16:05 passing as "flat".
9. The browser port up being read as "session live" (tonight: port up, tab missing).
10. A "no STOPPED card" grep passing on a file with no date filter and a card that only fires in one case.

### 5.6 What the runbook gets right (on the record)
- (e) states plainly "mode = cards — NOT automated", exactly as the Lead required; no "automated" language anywhere.
- The honest frame (24-26) is correct: the deployed engine has only `shadow | cards`; Cayleb places the bracket himself.
  I verified the "no order path" structurally (CDP allow-list, engine docstring, modes), not just by reading the claim.
- The failure disposition forbids the right things (no sign-in, no substitute lane, no invented fills, declare the log
  empty not complete).
- 09:05 item 1 (TZ/EST check) is correct for Mon 2 Nov, including the exact fallback instant (06:00Z) and the warning
  that an EDT host shifts every booking by an hour.
- Friday recon 3 correctly cites the two file paths and the numbers (15:29:12 ET, 50000/50000/0.0, tradeDate 2026-10-26)
  — I verified all four facts; only the word "last frame" is wrong (D9).
- No live action is planned anywhere; nothing in this runbook trades tonight.
- Conflict to flag, not a defect: 09:05 item 4's `market_open: false` demand is inverted against the engine's own
  Globex definition — see **D15** in §5.4. That one is a day-killer as written.

---

## 6. V6 — My veto conditions, in writing

General rule for every Tier S step (12-15, 09): **a step that reaches me without pre-check evidence on disk gets an
automatic veto, not a provisional pass.** Pre-check evidence means: a file under `board/probes/team-c/`, the exact
command(s) + verbatim output for each pre-condition, run against the **pinned commit with a clean tree** (`git status`
empty at the SHA named), the named owner+backup, and the rollback path. I re-run what can be re-run; the maker's summary
is never evidence. Any of these vetoes automatically: a secret in chat/log/git; a rail changed; two writers on one
resource; evidence that is only the maker's own summary; a credential request to me.

### A. Arming a LIVE futures order path (autonomous or semi-autonomous placement, successor build G0-G4)
- A1 Rails frozen and byte-checked in-process before **every** send: −$400 / +$900, max 2 trades/day, 15-min cooldown
  after a loss, flat 16:05 ET, 1 MES, min_stop 2.0, `symbol=MESZ6`, `contract_id=4470322`; the firm's $1,000 day cap and
  40% consistency named as firm-side (not engine-enforced) and unchanged. `config.json` sha256 + `ExecMainStartTimestamp`
  + startup line must all name the same config.
- A2 Fail-closed enable, the WP10 pattern: default OFF; arming needs a GO file with scope hash + expiry + environment;
  STOP file checked before every placement; six-condition guard table where **each** condition is shown to fail closed
  by a negative test I run. I sign the scope and the refusal table **before** code review ends.
- A3 Dry-run proof from a pinned commit: byte-identical rerun; guard-table mutation (remove each condition → suite
  fails); static AST proof that only the venue transport can open a socket; refusal tests for every rail; no order
  possible in dry mode.
- A4 Demo-venue send test (place/cancel/replace, duplicate client id, partial fill, reject), evidenced from the
  **broker's own frames**, not the engine's log. No live order in this step.
- A5 Broker-record truth: every fill reconciles to a card/decision; the intent log is never accepted as fill evidence.
- A6 Kill path documented and rehearsed (STOP + flatten), and flat-by-16:05 verified with a timed test.
- A7 Cayleb's explicit written go on disk, dated, in his own words (not relayed by a teammate).
- A8 Post-arm observation: first-hour intent/refusal/venue-state lines compared to the dry-run signature; any
  unexplained refusal = stop.
- A9 I never see or handle a key; if the venue needs one, Cayleb enters it write-only.
- Code freeze after my sign-off; any change re-opens the gate.

### B. Kalshi live flip (any real order, any size; the $51.39 is not traded live)
- B1 Gate verdict from the **real** forward-scoring tool (not the stub): pre-registered bar met — 2,000 settled hourly
  expiries per arm, θ/L fixed at the row matching measured end-to-end; the tool that writes `verdict.json` reviewed by
  me, pinned.
- B2 **L\* must be re-measured at flip time** with ≥ 24 h and ≥ 50 ticks in the real process including the live gateway
  hop; the flip row is the smallest row ≥ the measured end-to-end p90. If measured p90 > 29.7 s, row 60 until re-derived.
  The 10-minute/9-tick provisional is not sufficient for the flip itself (only for the ruling).
- B3 `guard.require_live` six conditions demonstrated live and individually fail-closed: `exec.json` enabled + scope
  hash; phrase env; unexpired GO with matching `scope_sha256`; real `verdict.json`; no STOP; environment `prod` on
  **both** sides. I run the negative tests myself.
- B4 Rails proven: daily loss stop, per-order/window/daily commit caps, max open orders, min notional, latency budget,
  book max age — unit + integration refusal tests each.
- B5 Signed scope: series list, caps, environment, one account id; GO expiry.
- B6 Kill: STOP honoured between the rails check and the send (tested at that exact point), cancel-all path, no orphan
  resting orders.
- B7 No new funds; only the existing $51.39; Cayleb's written go names that figure.
- B8 One writer: poll/ref/shadow untouched; ledger chains verify before and after.
- B9 First-hour post-flip review against the dry-run signature; any unexplained line = STOP.

### C. Wallet operation (meme desk: any sign/send/transfer)
- C1 What/why/exact address/amount from the desk's own record, not chat memory; a statement that the bot cannot do it.
- C2 Proof of exactly one live trader at the moment of the operation (process + supervisor + two independent reads) —
  the stand-down plan forbids two live traders.
- C3 Exit rule B present and byte-checked (variant B, `maxHoldH 0.5`), trader startup line shows it loaded, `MAX_HOLD_H`
  0.5 or unset — never 24 (Nox checklist §1); 5-second exit polling located and carried, marked **UNPROVEN** until the
  first live exit under the new deployment (Nox §2).
- C4 Ledger continuity from the desk's own copy (both ledgers + demotion scores), hash-verified before/after.
- C5 No new funds ever; only an operation the bot's own trading would do — otherwise it is Cayleb's act and I do not
  sign it.
- C6 Key never printed/copied/logged; wallet file handling read-only; no chat/log/git trace.
- C7 Exact reverse operation documented and a stop-and-do-nothing branch.
- C8 `JUP_API_KEY` carried agent-side only, never printed (Nox §3).

### D. Cutover step (openbell unified: any step that changes live state)
- D1 Nox checklist §1 executed and quoted: deployed `DATA/live/exit_rule.json` byte-identical to seed (variant B,
  `maxHoldH 0.5`); trader startup line prints it; `MAX_HOLD_H` 0.5 or unset, never 24.
- D2 §2: the 5 s exit polling located exactly (env/config/code diff vs the repo which has none), carried, and **UNPROVEN**
  until the first live exit; the ~40 s copy-exit lag acknowledged in writing.
- D3 §3: ntfy topic rotated; Worker redeployed and the phone page shows "unknown" correctly; ledgers (both + demotion
  scores) copied and DATA starts from the copy, not the seed; JUP key carried agent-side, never printed.
- D4 §3b: desktop host layer repointed off the old tree **and verified** before the tree moves (a missing BindPaths
  source crash-loops the unit and the sync logs every 2 minutes).
- D5 One live trader at all times: stand-down **first** (stop supervisor → 0 open → copy ledger → fresh desktop wallet →
  Cayleb moves SOL → start desktop live → stop box supervisor). Cayleb's SOL move is his act; I touch no wallet.
- D6 Cutover gating = "no new failures outside the known baseline" (Nox §4): I re-run the baseline myself and diff the
  failing-suite list; any new failure = veto.
- D7 Pinned commit (`openbell` master `6f582f8`) and a **clean tree** — no uncommitted edits. (The Kalshi tree fails
  this today: see §7 K1/K2. Different repo, same demand for the cutover repo.)
- D8 Rollback verified reversible: restore box supervisor, restore tree, restore worker, restore topic; written stop
  condition.

**Standing vetoes (not step-specific):** any Tier S step without pre-check evidence; any step whose evidence is a
summary rather than a command+output; any request for credentials; any change to the rails; any attempt to trade the
$51.39 live; any "assume it passed" language; running the Monday futures day on the **current** runbook v4 (D1-D15
unfixed).

---

## 7. Discrepancy register (all named)

| ID | Sev | Item | Evidence |
|---|---|---|---|
| K1 | **M** | Kalshi working tree is **dirty** and carries an unexplained edit: `live/kalshi/model.py` +1 line `# tamper`. Comment-only (semantically inert) but unexplained and uncommitted; the readiness/test runs are against this dirty tree, not a pinned SHA. | `git status --short`; `git diff live/kalshi/model.py` → `+# tamper` at EOF |
| K2 | m | Other uncommitted maker edits in the same tree: `data/kalshi15/manifest.json` (+1120/−…), `research/kalshi15/{DECISIONS,RESUME,SPEC}.md`, untracked `research/kalshi15/{forward,freeze,test_forward,test_freeze}.py` and `results/first_scorer.*`. Any gate must name a pinned commit; today's evidence is not at one. | `git status --short`, `git diff --stat` |
| K3 | m | `scorer_agreement.json` says summaries + late file "identical"; their bytes differ (sha256 above). True claim = identical within `tol 1e-9` on defined keys. | §4.1 |
| K4 | m | "≈19-20 s end-to-end" from the availability file is **≈18-19 s** (16.735 + 1.5 + 0.5 = 18.735); the honest margin at L*=30 is **0.29 s** after adding pull+gateway to the data-time p90 (27.713 + 2.0 = 29.713). Sample: 9 ticks / 81 lines / 10 min. | §4.2 |
| K5 | **M** | `summary.json` `end_to_end_latency` (p50 64.5 s, row 90, `row_L` 60) is a **construction** `L + (t_f − a_asof)`, circular when read as a measurement; it contradicts the 16.1/16.7 s availability if misread. Must be labelled. | §4.3 |
| K6 | m | Maker's `RESUME.md` "172 tests" vs 167 discovered — confirmed as a count-of-methods discrepancy; no failure either way. | 167 OK on my run |
| K7 | **B** | Runbook v4 defects D1-D15 (§5.4), incl. the fabricated `rails:` line, the `card_out` field that does not exist, the impossible-at-09:15 (c) and "Decision logging ON" checks, the vacuous `positions {}` flat proof, and the inverted `market_open` check (D15). | §5.4 |
| K8 | **M** | Runbook says Friday's "last frame" is 15:29:12 and "positions {}"; the file's last frame is a **dom** frame 17:52:13 and no `positions` key exists in any frame kind. Broker state after 15:29 Friday is unverified (that is the owed recon). | §5.4 D9 |
| K9 | **M** | Runbook recovery route cites the Nox→Outis/Codex lane that GROUND-STATE records as dark (Outis offline 151 d; B12 silent 21h+). Only Cayleb's desktop sign-in is a verified route. | §5.2 D12 |
| K10 | m | Runbook's 16:20 cards count is cumulative (26 tonight) vs per-day (Friday 2); the command as written cannot compute completeness. | §5.3 |
| K11 | m | Availability measurement window is 10 min / 9 ticks; L*=30 rests on it + Nox's independent confirmation. It is a real measurement but thin — do not let it be quoted later as a distribution. | §4.2 |
| K12 | — | Nox collector copy stale, check 8 NA by construction: confirmed in both runs (ours 441 settlements vs Nox 154; overlap 0 s). Not a defect. | V1 table |

Facts I verified that the makers claimed and that are **true** (so they should not be re-litigated): three units
enabled+active as **user** units (X8); readiness 9/1/0 exit 0; 247 + 167 tests OK; dry run byte-identical and
reproducible from scratch; live path OFF (`exec.json enabled:false`, no GO/verdict/STOP, `require_live →
not_enabled`); scorer agreement 6,853 rows / 0 differences; availability 16.127/16.735 and 24.586/27.713; engine has no
order path (CDP allow-list + modes); `mode=cards`; Friday numbers 15:29:12 / 50000 / 50000 / 0.0 / tradeDate 2026-10-26.

---

## 8. What this batch does NOT certify, and what I need next

- **Nothing is certified by this batch.** It is the checker's baseline: V1-V4 are independently reproduced (and stand);
  V5 is rejected as written; V6 is on record.
- Not covered tonight (deliberately, Tier S): broker session, order, wallet, credential, Kalshi live, cutover; the two
  Kalshi clock-instant checks (they must run at 00:07Z and 05:00-07:00Z); the live gateway hop of the latency (not
  built); the Friday broker reconciliation (needs Monday's broker record).
- **To sign Monday's futures day I need** (a) runbook v5 fixing D1-D15, with every attestation carrying its own
  command + expected output and a blanket "any FAIL = day does not start"; (b) `card_out` either defined in the
  heartbeat or dropped; (c) the (c) split with the binding post-09:30 re-check; (d) the real config/rails/contract-id
  assertion + `session day 2026-11-02: pnl 0.00 trades 0`; (e) the completeness check defined as a criterion with a
  day-filtered cards count; (f) the market_open check inverted to "Globex alive, RTH not yet". Then P0.1's signoff file
  can exist before 09:05 — not before.
- **To sign any Kalshi live flip:** the full §6-B list, including a flip-time latency re-measurement (29.7 s is the
  number to beat).
- Environment: `/tmp` inode exhaustion (98%) is a live risk to any team's subprocess runs tonight; someone with
  authority over `/tmp/ergon-*` should clear it before Monday's window (I did not touch another team's files).

— `trading-checker`, 2026-10-31 19:3x ET. Reports to `lead`.
