# TEAM C — KALSHI RESUME PACK (Monday 2 Nov 2026, 01:00 ET)
Status: **READY TO EXECUTE — NOTHING EXECUTED.** trading-lead, 2026-10-31 18:4x ET. Tier S job (14): no live order
without a passed gate AND Cayleb's written go; the $51.39 is not traded; no key material printed/copied/committed.
Lane is dark until the Monday 01:00 ET weekly-reset booking; the services are running through the weekend and the capture
is the asset this pack preserves.

## CORRECTIONS APPLIED 1 NOV (read this before §0; the body below is the 31 Oct text and is stale where listed)
Verified by the Sunday ground-truth batch (`board/reports/team-c-sunday-groundtruth-20261101.md`, 13:3x–13:4xZ):
- **L\* = 30 s is no longer FOR NOX — it is RULED and APPLIED.** Nox batch-2 item 6 confirmed it; the applied text is in
  `research/kalshi15/DECISIONS.md` + `SPEC.md` (§5 row below is superseded). Two one-line flips deliberately left:
  `params.provisional.json "l_star_s": 60` (data file — Stage A must run `--L 30`) and
  `results/scorer_agreement.json latency.L_star: 60` (generated artefact — regenerates on the next `stage_a.py dump-s`).
- **§4 "Freeze artefacts built — NOT BUILT" is superseded:** `research/kalshi15/freeze.py` and `forward.py` are built and
  tested (`python3 -m pytest research/kalshi15/test_freeze.py research/kalshi15/test_forward.py -q` → **15 passed**,
  run 2026-11-01). The freeze itself is still correctly **refused** because the prereg has 7 `<FILL AT FREEZE>` slots —
  that refusal is the correct state until the slots are filled. Target Tue 3 – Wed 4 Nov unchanged.
- **Hourly-bar ruling (X6) resolved:** Nox batch-2 item 5 — the "300" was a coordinator-side transcription, corrected in his
  §5; the governing bar is 2,000 hourly events and nothing on our side needs amending.
- **§0 readings refreshed 2026-11-01 13:37:50Z:** units enabled AND active; installed unit copies IDENTICAL to repo copies
  (diff = no-op); capture fresh to 13:37:50Z, 2,910,685 records / 0 errors; `readiness.py --day 2026-11-01` EXIT=0,
  9 PASS / 1 NA / 0 FAIL; ledger decisions seq 11727, orders 89520.
- **WP10 tests are 167, not 172** (collected and passed 2026-11-01; repo unchanged by the run); package still ships OFF,
  guard table 64 combinations asserted.
- **Window B (DST 05:50Z/07:15Z) is INCOMPLETE** — 05:50Z leg passed, 07:15Z leg never ran (runner killed 06:48:20Z by the
  DSH app-instance teardown), checker audit absent; see `board/probes/team-c/dst-20261101T0715Z/ADJUDICATION.md`. Window A
  (20:07 rollover) completed with its `ARMED_RUN_DONE` marker. Do not describe Window B as verified.
- Still FOR NOX: the stale-header wording (Amendment 5) and the Kelly posture (Amendment 6). Nothing else in §5 is pending.

## 0. What must already be true (checked tonight, read-only — all PASS)
| # | Fact | Evidence (2026-10-31 ~21:5xZ) |
|---|---|---|
| A | Three units are user units, **enabled AND active** | `systemctl --user list-unit-files` (enabled), `is-active` -> active active active; **X8 correction to GROUND-STATE** |
| B | Capture fresh, schema-clean | readiness at 21:54:29Z: cutoff capture 21:54:28Z; 2,672,540 records, 0 errors |
| C | Ledger chain verifies | 4,375 decisions / 31,397 orders; `chain_integrity` PASS |
| D | Readiness 9 PASS / 1 NA / 0 FAIL | `board/probes/team-c/readiness-2026-10-31.{json,log}` |
| E | Nox's collector copy stale since 11:59:42Z -> check 8 NA by construction | X9; notice text in `team-c-c6-c10.md` C8.5 |
| F | Two clock-instant checks fall BEFORE Monday (tonight/early Sun) | §2 below |
| G | Rulings R1–R4 are FOR NOX; nothing may be logged before he confirms | `team-c-c6-c10.md` C8 |

## 1. Clock arithmetic (corrected; the RESUME table's "03:12 ET = 06:00Z" is wrong)
Host TZ = `America/New_York`. Tonight is EDT (UTC-4). US DST ends **Sun 2026-11-01**: 02:00 EDT -> 01:00 EST at **06:00Z**.
| Label | Instant (ET) | UTC | Meaning |
|---|---|---|---|
| 20:07 rollover | Sat 20:07 EDT | **00:07Z Sun 1 Nov** | UTC-day rollover: poller gzips 10-31 and opens 11-01 folders |
| fallback instant | Sun 02:00 EDT -> 01:00 EST | **06:00Z** | the repeated local hour is 01:00–01:59 (05:00–05:59Z EDT pass, then 06:00–06:59Z EST pass) |
| "03:12 ET" DST check | 03:12 **EST** (post-fallback local) | **08:12Z** | if the label was computed in EDT it is 07:12Z — either way it is NOT 06:00Z; the check window is 05:00–07:00Z |
| recommended run | ~03:15 local | **07:15Z** | after both passes of the repeated hour; read 04:45–07:15Z |
Rule: never re-label a check by its old ET time; each check below names its UTC instant.

## 2. The two clock-instant checks (run overnight; read-only; nothing else)

### 2.0 — FIXED ARTEFACT PATHS (the checker verifies these files, never a live read)
Both windows are **armed** (nohup scripts that survive session end; each directory has `RUN.md` with the pass criteria written
BEFORE the run, `run.sh`, and `exec.log`).

**Window A — UTC-day rollover, instant 00:07Z Sun 1 Nov (20:07 ET Sat 31 Oct). Dir:
`board/probes/team-c/rollover-20261031T0007Z/`**
| # | Artefact (exact filename) | Produced | Content |
|---|---|---|---|
| 1 | `readiness-before.json` + `readiness-before.log` | ~00:05:20Z | `readiness.py --day 2026-10-31`, full 10 checks + EXIT |
| 2 | `readiness-after.json` + `readiness-after.log` (+ `readiness-after-20261101.json`) | ~00:07:40Z | 10-31 final post-seam + 11-01 new day |
| 3 | `dayfolders.txt` | both instants (labelled) | `ls -la capture/` + `ref/` + sizes |
| 4 | `chain-head-before.txt` / `chain-head-after.txt` | 00:05:20Z / 00:07:40Z | `chain_head.json` + sha256 of decisions/orders + last seq/ts |
| 5 | `gaps-events-slice.log` | ~00:08Z | gaps/events rows 23:50Z–00:15Z for both UTC days + file tails |
| 6 | `units.txt`, `exec.log`, `run.sh`, `RUN.md` | — | unit state, execution log, the armed script, the criteria + results |

**Window B — DST hour, 05:00–07:00Z Sun 1 Nov (fallback 06:00Z). Dir: `board/probes/team-c/dst-20261101T0715Z/`**
| # | Artefact (exact filename) | Produced | Content |
|---|---|---|---|
| 1 | `readiness-before.json` + `readiness-before.log` | ~05:50Z | `readiness.py --day 2026-11-01` pre-fallback (partial day) |
| 2 | `readiness-after.json` + `readiness-after.log` | ~07:15Z | `readiness.py --day 2026-11-01` post-fallback (both passes done) |
| 3 | `dayfolders.txt` | both instants | capture/ + ref/ listings |
| 4 | `chain-head-before.txt` / `chain-head-after.txt` | 05:50Z / 07:15Z | head must advance across the repeated hour, never reset |
| 5 | `gaps-events-slice.log` | ~07:17Z | gaps/events rows 05:30–07:30Z + tails |
| 6 | `units.txt` (incl. `timedatectl` at both instants), `exec.log`, `run.sh`, `RUN.md` | — | EST offset proof + unit state + log + criteria/results |

Pass criteria are stated in each `RUN.md` before the run: readiness 9 PASS / 1 NA / 0 FAIL; the 10-31 day closed+gzipped and
11-01 folders opened; chain head **advances** (never resets); no unit restart/crash at the seam; any new gap quoted. If a unit
misbehaves or the capture is stale at the instant, it is recorded as a gap with the exact observation — **never reconstructed
after the fact**.
### 2.1 — 20:07 ET rollover -> 00:07Z Sun 1 Nov (owner trading-lead)
```bash
cd ~/Work/projects/openbell-kalshi
date -u                                        # expect 2026-11-01T00:0xZ
ls -la live/kalshi/capture/ live/kalshi/ref/   # 2026-10-31 gzipped/finished; 2026-11-01 folders opening
bash live/kalshi/install-units.sh status       # state + heartbeat ages (read-only)
python3 -I research/kalshi15/readiness.py --day 2026-10-31 \
  --out ~/Documents/deepseek-harness/default-workspace/board/probes/team-c/readiness-2026-10-31-rollover.json
journalctl --user -u openbell-kalshi-poll -n 40 --no-pager | tail -20   # look for the gzip/rollover lines, no errors
```
Pass = the 10-31 day is closed and gzipped, 11-01 files appear, units still active, readiness exits 0 with the same
9 PASS / 1 NA shape, and the ledger chain head advances (no reset). Record the chain head before/after.

### 2.2 — DST hour -> 07:15Z Sun 1 Nov (owner trading-lead)
```bash
cd ~/Work/projects/openbell-kalshi
timedatectl | grep -E "Time zone|Universal"     # America/New_York; after 06:00Z the offset shows EST (-0500)
ls -la live/kalshi/capture/2026-11-01/          # gaps.jsonl + events.jsonl exist (created by poller)
python3 - <<'EOF'
import json, glob
lo, hi = "2026-11-01T04:45:00Z", "2026-11-01T07:15:00Z"
for f in sorted(glob.glob("live/kalshi/capture/2026-11-01/*.jsonl*")) + sorted(glob.glob("live/kalshi/ref/2026-11-01/events.jsonl*")):
    print("--", f)
    for line in open(f, errors="replace"):
        try: d = json.loads(line)
        except Exception: continue
        ts = d.get("ts") or d.get("t_utc") or ""
        if lo <= str(ts) <= hi:
            print(ts, d.get("scope") or d.get("ev") or d.get("kind"), str(d.get("error") or d.get("note") or "")[:90])
EOF
python3 -I research/kalshi15/readiness.py --day 2026-11-01 \
  --out ~/Documents/.../board/probes/team-c/readiness-2026-11-01.json
```
Pass = (i) no capture/recorder crash and no unit restart around 06:00Z; (ii) `gaps.jsonl` shows no new overrun beyond the
weekend baseline (the poller's successor-ticker guess uses `America/New_York`; the documented risk is overrun gaps only);
(iii) readiness on the DST day exits 0 with no new FAIL (a `gap_ledger` WARN is acceptable and must be quoted, a FAIL is
reported); (iv) chain head advances monotonically across the repeated hour (epoch-based timestamps: no duplicate/negative
tick). Record both UTC instants + the local wall-clock around 01:00 twice.

## 3. Monday 01:00 ET sequence (owner trading-lead; backup lead; ~60–90 min)
1. **Confirm the reset is real** — read Nox's sensor board row for the Money Desk / usage meter (read-only,
   `~/Work/shared-workflows/lanes/board/ops/STATUS.md`); do not assume 01:00 means reset. Record the quoted line.
2. **Read state first**: `research/kalshi15/RESUME.md`, `DECISIONS.md`, and §2's two check results. Do not start from memory.
3. **Units state** (verify, do not blindly restart):
   ```bash
   cd ~/Work/projects/openbell-kalshi
   bash live/kalshi/install-units.sh status
   systemctl --user is-active openbell-kalshi-poll openbell-kalshi-ref openbell-kalshi-shadow
   ```
   If any unit is failed/inactive: `systemctl --user restart <unit>` (only after reading its journal for the exit code;
   exit 2/3/4/5/6 are "needs a human", never blind-restart), then re-check heartbeats (< 30 s).
   Installed units are COPIES of `live/kalshi/openbell-kalshi-*.service`; if the repo copy changed, re-copy and
   `systemctl --user daemon-reload` (diff first: `diff <installed> <repo>`).
4. **Readiness on the DST day + Monday**: `--day 2026-11-01` (from §2.2, re-quote) and `--day 2026-11-02` once the day has
   data. Report per-check lines, not just the summary.
5. **Rulings — only if Nox has confirmed them** (see §5). Nothing is logged before his words; if unconfirmed, ship the
   resume pack with the rulings still marked FOR RULING and say so.
6. **Restart the resumable fit-split pull** (it died with the build session; resumable, nothing lost):
   ```bash
   cd ~/Work/projects/openbell-kalshi && nohup python3 -I research/kalshi15/pull.py prints \
     --since 2026-08-08 --series KXETH15M KXSOL15M KXXRP15M KXDOGE15M KXBNB15M KXHYPE15M KXZEC15M KXNEAR15M \
     --max-hours 20.5 > ~/Work/projects/openbell-kalshi/data/kalshi15/pull-prints-20261102.log 2>&1 &
   ```
   Keep total Kalshi load ≤ 5 req/s (the poller uses ~2); at 2.8 req/s this is ~6 days of fit split + ~13 h of
   Aug 8–Oct 6. θ stays UNSET until prints land.
7. **Stage A / DEV report order** (from RESUME §Pending 2, unchanged; this is the early read):
   a. `python3 -I research/kalshi15/stage_a.py run --through 2026-10-31 --no-lock dry` once fit-split prints exist →
      θ and X chosen at the row of record in force (L\* per ruling, §5);
   b. `python3 -I research/kalshi15/stage_a.py btc-wanted`, then
      `pull.py prints --series KXBTC15M --tickers-file data/kalshi15/derived/btc_wanted.json --max-hours 6` (BTC ≈ 38
      requests/window; may stay partial);
   c. formal `stage_a.py run` (takes `DEV.lock`; the validation span was already seen in a dry run — descriptive only);
   d. `stage_a.py dump-s` and `second_scorer.py run` at the same `--done-lines`; `compare_scorers.py` must say **AGREE**;
   e. `sizing_sim.py run`; f. `dev_report.py` → `reports/KALSHI15-STAGE-A-<date>.md` (renders "NOT EVALUABLE" for the
      P/L bar while live latency is unmeasured).
8. **Report**: evidence file with the quoted readiness lines, the two check results, the pull's PID/log path, and the
   rulings' status. Nothing live, no order, no key.

## 4. Forward-counting start checklist (the gate after the early read; do NOT start the count early)
Preregistered rules: bars = 2,000 settled windows per coin (Arm A) / 2,000 hourly events (Arm B); D3 count starts at the
**later of the freeze and capture being healthy**; D5 any change after a look starts a new prereg version with a fresh
count; nothing live without a passed gate + Cayleb's written go.
- [ ] Capture healthy for the preceding 7 days (readiness PASS, gap ledger within warn, no unexplained restarts).
- [ ] Fit-split prints complete enough for θ/X; Stage A dry + formal run done; DEV report produced.
- [ ] Two independent scorers AGREE at the same `--done-lines` (currently 6,853 rows, 0 differences; re-quote after the run).
- [ ] Sizing table re-scored (needed for R3's "fraction set at the flip").
- [ ] **Freeze artefacts built** — currently **NOT BUILT**: no `frozen.json`, no pins/guards, no `freeze` tool in the repo
      (checked: only unrelated `research/*/freeze.py`). The freeze cannot be honestly executed until it exists; that is a
      work item for the first metered session, not something to improvise at 01:00.
- [ ] Prereg `PREREG-KALSHI15` `<FILL AT FREEZE>` slots filled; commit subject `kalshi15 prereg FROZEN:`; params hash pinned.
- [ ] Freeze target **Tue 3 – Wed 4 Nov** (booked); forward count begins at the freeze instant, recorded to the second.
- [ ] Forward-record scoring / daily-look tool — **NOT BUILT** either; the daily read-only look (D5) has no tool yet.
- [ ] Verdict dates quoted as arithmetic only: 15M earliest ~25 Nov; hourly ~late January.

## 5. Text to log the moment Nox confirms (nothing edited before that; full text in `team-c-c6-c10.md` C8)
| Item | Where it goes | Status |
|---|---|---|
| Amendment 4 — hourly bar = **2,000 hourly events** (X6: the 300 figure is not locatable here; Nox must name the file) | `SPEC.md` Appendix T + the section carrying 300 | FOR NOX |
| Amendment 5 — **order-path wording** (shadow has none; `live/kalshi_exec/` exists, disabled by construction; guard tests assert refusal) | `PREREG-DRAFT.md` §1 + any copy | FOR NOX (text already true in tests) |
| Amendment 6 — **Kelly posture** (heavier-than-quarter-Kelly at the flip, daily loss stop untouched; addendum amended) | `DECISIONS.md` D6 + Amendment 1; `SPEC.md` Appendix T item 4 | FOR NOX |
| L\* = 30 s DECISIONS row + the "Confirmed by Nox" line | `DECISIONS.md` Amendment-1 table | **FOR NOX'S RULING** — do not edit; leave `scorer_agreement.json latency.L_star = 60` until ruled |
If Nox answers only some: log exactly those, leave the rest marked, and say which in the resume report.

## 6. Do-not list for this lane (restated)
No live order ever on this lane without (i) the enable flag, (ii) a passing gate, (iii) Cayleb's explicit written go (none
of the three exists). No key material printed/copied/committed (the signing key lives only on the coordinator VM; Cayleb's
revocation is the kill switch). No new funds; $51.39 untouchable. No bar changes mid-count. No merges/deploys/pushes from
this lane tonight or Monday unless a separate ruling says so.
