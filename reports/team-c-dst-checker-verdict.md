# Team C — DST clock-instant check (C-1): INDEPENDENT Tier S CHECKER VERDICT

Checker: fresh-context, independent; I did not build, write or stage any artefact audited here.
Written 2026-11-01T14:20Z / 09:20 EST, read-only. No live action, no order, no wallet, no credential handling, $0.
Subject: `board/probes/team-c/dst-20261101T0715Z/ADJUDICATION.md` (mtime 2026-11-01T13:51:29Z) and every file it cites.
Only file written by me: this report.

## VERDICT TABLE

| # | Item | Verdict |
|---|---|---|
| 1 | 05:50Z leg actually ran and passed | **PASS** |
| 2 | 07:15Z leg genuinely absent | **PASS** |
| 3 | Stated cause (SIGTERM 06:48:20Z, app-instance teardown, no OOM/kill/reboot) | **PASS — independently corroborated** (one attribution link inferred, not recorded) |
| 4 | LATE leg honestly labelled; numbers as the files say | **PASS** (numbers and label all verified; one sentence about method is false — C3) |
| 5 | Overall verdict INCOMPLETE honest and complete; no self-certification | **PASS** |

**OVERALL: PASS — the adjudication's verdict is sound and complete.**
The verdict INCOMPLETE is correct, correctly reasoned, and explicitly not self-certified. Every material claim was
independently reproduced from primary sources (journal, live ledger, capture files, sha256 recomputation, process table).
Eight supporting statements are inaccurate at the margin and are listed as C1–C8; none changes the verdict.
Nothing here is passed on the maker's word.

---

## 1. The 05:50Z leg — RAN, and its before-side checks passed. **PASS**

Primary evidence, all timestamps UTC:
- `readiness-before.log:1` — `readiness 2026-11-01  read at 2026-11-01T05:50:02Z  observed 2026-11-01T00:00:00Z..2026-11-01T05:50:00Z`.
- `readiness-before.log:14` — `EXIT=0`.
- `readiness-before.log:3` — `PASS  capture_validator     5 files, 1567086 records (...), errors 0` (the "1,567,086 records, 0 errors" claim is exact).
- `readiness-before.log:4` — `PASS  gap_ledger ... starts 0 restarts 0 crashes 0; longest gap 60s`.
- `readiness-before.log:5` — `PASS  ref_health ... clock 35 lines offset -0.85..2.571 ms stratum [2]`.
- `readiness-before.json` summary parsed: `{"NA": 1, "PASS": 9}`, `day: "2026-11-01"`; per-check statuses are exactly 9 PASS / 1 NA (`nox_reconcile`) / 0 FAIL / 0 WARN.
- `units.txt:1-4` — `Local time: Sun 2026-11-01 01:50:02 EDT` / `Universal time: Sun 2026-11-01 05:50:02 UTC` / `Time zone: US/Eastern (EDT, -0400)` — genuinely pre-fallback (fallback 06:00:00Z).
- `exec.log:9` — `- 2026-11-01T05:51:29Z captured: chain-head-before.txt, ... readiness-before.{json,log} (day 2026-11-01)`.
- `chain-head-before.txt:1-7` — `# instant_utc=2026-11-01T05:50:02Z`, `decisions_lines=8074` / `decisions_seq 8073`, `orders_lines=59786` / `orders_seq 59785`, plus both sha256s.
- Independent sha256 recomputation on the live ledger: `head -8074 decisions.jsonl | sha256sum` = `38233aaa0e7bcfe48222ee0e4dcaf400cc2fe15d7d5eb3ac0b1cf432d65e3ebc`
  = `chain-head-before.txt:6`; `head -59786 orders.jsonl | sha256sum` = `58d6d2d14e38c163be4aaaeb35651c5181b9620265ce707c4825afb0cf1dc47c` = `chain-head-before.txt:7`. Both match exactly.
- Independent `seq` check on the live ledger: line 8074 carries `"seq": 8073`; the file is strictly consecutive (`prev+1`, 0 exceptions) over all lines observed, so the snapshot seq is real and no line was rewritten.

Caveat (C6): ADJUDICATION.md:12 heads this "RAN, PASSED its own criteria". RUN.md:23-26 criteria 1, 3 and 4 are
*two-instant* criteria (before **and** after). Only the before-side checks were satisfiable; the run's criteria remain unmet.
The headline verdict INCOMPLETE already prevents a misread, but the heading should say "before-side checks passed".

## 2. The 07:15Z leg — genuinely absent. **PASS**

- Directory listing: **no** `chain-head-after.txt`, `readiness-after.json`, `readiness-after.log`, `gaps-count-after.txt`,
  `gaps-events-slice.log`, `dst-slot-audit.txt`. Only the `*-LATE` variants exist (and belong to §4).
- A wider search (`find ~/Work/projects/openbell-kalshi ~/.../board -name readiness-after.json -o -name chain-head-after.txt -o -name dst-slot-audit.txt`)
  returns only the *other* run directories (`rollover-20261031T0007Z/`, `dst-drytest-20261101T0027Z/`) — nothing for this run.
- `RUN.md:37-38`: the file ends at `## Armed run started 2026-11-01T00:21:43Z` + the 05:51:29 capture line. `grep -n ARMED_RUN_DONE RUN.md`
  → no match. (run.sh:189 would have appended it.)
- `exec.log` mtime 2026-11-01T05:51:29.498Z; `RUN.md` same. The next write in the whole directory is 13:37:50Z — a 6h46m22s hole.
- `ps aux | grep -Ei 'dst-2026|run\.sh|dst_slot'` → empty at audit time (14:05Z). The 13:30Z instant itself is not re-observable;
  nothing has written to the directory since 13:39:55Z, so no unrecorded continuation is plausible.
- run.sh:127-131 would have appended a `timedatectl` block to `units.txt` at 07:15 *before* writing the snapshot. `units.txt`
  contains exactly two blocks: `# 2026-11-01T05:50:02Z` and `# 2026-11-01T13:37:50Z`. The 07:15 branch never executed its first command.
- `exec.log:2-8` contains exactly the four "Armed run started" lines the adjudication names (23:46:02Z, 00:14:05Z, 00:20:50Z, 00:21:43Z).

## 3. The stated cause — corroborated by the journal. **PASS** (one inference, declared)

`journalctl --user -o short-iso` reproduces the quoted lines **verbatim** (they are not paraphrased):

```
2026-11-01T01:46:16-05:00 outis-desktop systemd[1387]: Started app-deepseek\x2dai\x2ddsh\x2ddesktop-3921591.scope.
2026-11-01T01:48:20-05:00 outis-desktop systemd[1387]: dsh-subprocess-2987977-08f90ab27e8f.scope: Sending signal SIGTERM to process 3542392 (bash) on client request.
2026-11-01T01:48:20-05:00 outis-desktop systemd[1387]: dsh-subprocess-2987977-08f90ab27e8f.scope: Sending signal SIGTERM to process 3925954 (sleep) on client request.
2026-11-01T01:48:20-05:00 outis-desktop systemd[1387]: dsh-subprocess-2987977-08f90ab27e8f.scope: Consumed 1min 51.944s CPU time over 6h 26min 37.919s wall clock time, 1.6G memory peak.
2026-11-01T01:48:21-05:00 outis-desktop systemd[1387]: app-deepseek\x2dai\x2ddsh\x2ddesktop-2987797.scope: Consumed 3h 48min 23.857s CPU time over 10h 29.808s wall clock time, 2G memory peak, 344.5M memory swap peak.
2026-11-01T01:52:00-04:00 outis-desktop systemd[1387]: dsh-subprocess-2987977-c4144d47452d.scope: Consumed 71ms CPU time over 89ms wall clock time, 82.3M memory peak.
```

Checked point by point:
- **SIGTERM at 06:48:20Z** — `01:48:20-05:00` = 06:48:20Z. Exact.
- **"on client request"** — exact wording; two processes in the runner's scope (bash 3542392, its `sleep 5` child 3925954) are killed in the same second, plus `opencode` pid 3419867 in a sibling scope. This is a scope stop, not a single-process signal.
- **Same process as the armed run** — scope wall clock `6h 26min 37.919s` → start 00:21:42.081Z, matching `exec.log:8` `Armed run started 2026-11-01T00:21:43Z` to <1 s; `BOARD.md:407,440,462,604` independently name pid 3542392 as the armed runner.
- **26m40s before 07:15:00Z** — 07:15:00 − 06:48:20 = 26m40s. Correct. **48m20s after 06:00:00Z** — correct.
- **App-instance teardown** — the old app scope `…-2987797.scope` is released 1 s after the SIGTERM, and a new app scope
  `…-3921591.scope` is started at 06:46:16Z (`uwsm` also logs a fresh `gtk-launch deepseek-harness-official.desktop` at 06:48:23Z).
- **Ownership of the runner's scope** — DSH names Linux scopes `dsh-subprocess-<creating-process.pid>-<rand>`
  (`packages/subprocess/subprocess-local/lib/types/linux-scope.js`, `unitStem()`), so the runner's scope was created by DSH
  pid 2987977. DSH pid 2987977 is bounded to the old app's lifetime: `dsh-subprocess-probe-2987977-*` appear at 20:47:54Z
  (3 s after the old app scope's arithmetic start 20:47:51Z) and the app's pid 2987797 is the immediately preceding pid;
  the tripwire snapshot log shows ppid=2987977 children. The journal does **not** print a cgroup→app mapping — this one
  attribution step is inference, and it is the only inference in the chain.
- **No OOM** — `journalctl -k --since 05:30Z --until 07:30Z` has 82 lines, **all** `[UFW BLOCK]`; zero `oom`/`oomd`/`killed process`
  lines in the whole 05:30–07:30Z window.
- **No reboot** — `uptime -s` = `2026-10-28 15:38:46`; `last reboot` shows the last boot 28 Oct.
- **No unit restart/crash** — `systemctl --user show openbell-kalshi-{poll,ref,shadow} -p NRestarts` → 0/0/0, `ExecMainStartTimestamp`
  unchanged since 31 Oct; no kalshi lines at all in the 05:30–07:30Z journal.
- **Tripwire** — `…/incident-20261031-app-kills/tripwire.log` has no 05:xx/06:xx entry. Weak support only: that detector is
  documented as defective (`board/reports/team-e-phase1.md:101-110`), so its silence is not evidence either way. The journal is.

Corrections: **C1** — ADJUDICATION.md:47 glosses the old app's `10h 29.808s` wall clock as "10h29m". systemd spells minutes
as `min` (cf. `6h 26min 37.919s`), and `systemd-analyze timespan "10h 29.808s"` parses it to 36,029.808 s = **10h 0m 29.8s**.
The quoted line is exact; the prose is wrong by ~28.5 min. Immaterial. **C2** — ":52 `(journalctl -k` window empty)" is false
(82 UFW lines); the substantive claim (no OOM-kill line) is true and verified.

## 4. The LATE leg — labelled honestly; every number matches its file. **PASS**

Label and provenance:
- Filenames carry `-LATE`; `chain-head-after-LATE.txt:1` — `# LATE after-leg re-run, instant_utc=2026-11-01T13:37:50Z  (target instant was 2026-11-01T07:15:00Z)`.
- `late-leg.sh:2` — `# LATE re-run of the 07:15Z after-leg — read-only, identical commands to run.sh; labelled LATE (not the armed check).`
  (see C3 on "identical commands").
- ADJUDICATION.md:87-89 — "It does **not** substitute for the armed 07:15Z instant ... it is not a checker signature."
  No attempt to pass the reconstruction off as the armed observation.

Mtimes (epoch-resolved, so the ±4/±5 h offset is the real DST shift, not a rewrite): `late-leg.sh` 13:37:50.395Z;
`chain-head-after-LATE.txt` 13:37:50.502Z; `units.txt` (LATE block) 13:37:50.622Z; `gaps-count-after-LATE.txt` 13:37:50.633Z;
`readiness-after-LATE.{json,log}` 13:39:54.880Z / 13:39:55.157Z; `gaps-events-slice-LATE.log` 13:39:55.183Z;
`late-leg.log` 13:39:55.201Z. `late-leg.log:1` `LATE_LEG_START 2026-11-01T13:37:50Z`, `:6` `LATE_LEG_DONE 2026-11-01T13:39:55Z`.

Numbers, quoted against the file:
- **EXIT** — `readiness-after-LATE.log:14` `EXIT=0`. ✔
- **PASS/NA/FAIL** — `:3-12` gives exactly 9 `PASS` + 1 `NA` (`nox_reconcile`) + 0 FAIL; `readiness-after-LATE.json` summary
  `{"NA": 1, "PASS": 9}`, `day: "2026-11-01"`. ✔
- **capture_validator** — `:3` `5 files, 2910685 records (book=46589 fees=13 gap=32 market=11192 schedule=1 settled=495
  status=163 trade=2852200), errors 0`; the adjudication's parenthetical omits `fees=13`/`schedule=1` but the total and every
  quoted count are exact. `:2` `cutoff capture books 2026-11-01T13:37:50Z`; JSON `capture_books.bytes_read=184656229`,
  `partial_tail 0`. ✔
- **gap_ledger** — `:4` `worst 0.184% (XRP) ... gap records 32 (book 1234s trades 62s meta 0s); starts 0 restarts 0 crashes 0;
  longest gap 60s`. ✔
- **ref_health** — `:5` `clock 82 lines offset -9.613..3.072 ms stratum [2]`. ✔
- **settlement_reconcile** — `:6` `result vs rule 486/486, shadow vs result 486/486`. ✔
- **replay_determinism** — `:8` `byte-identical over the common prefix: decisions 11728 lines, orders 89521 lines ... 122.5 s`. ✔
- **chain_integrity / heartbeats** — `:9`, `:12` `poll=1.2s ref=0.6s shadow=0.8s`. ✔
- **Criterion 3 (chain movement)** — `chain-head-after-LATE.txt:2-7` `decisions_lines=11728` / `decisions_seq=11727`,
  `orders_lines=89521` / `orders_seq=89520`; `late-leg.log:3-4` `8073 -> 11727` / `59785 -> 89520`; `:5`
  `CRITERION_3_LATE: PASS — chain advanced across the repeated hour, no reset, no line count went backwards`. ✔
  Independently: line 11728 carries `"seq": 11727`; line 89521 `"seq": 89520`; the live file is strictly consecutive with 0
  exceptions; `head -11728 decisions.jsonl | sha256sum` = `88499116c9bb...` = `chain-head-after-LATE.txt:8`, and
  `head -89521 orders.jsonl | sha256sum` = `f84de5c4...` = `:9`. The prefix hashes prove the ledger was not rewritten.
- **Criterion 4 (EST)** — `units.txt:15-21` `Local time: Sun 2026-11-01 08:37:50 EST` / `Universal time: ... 13:37:50 UTC` /
  `Time zone: US/Eastern (EST, -0500)` / `synchronised: yes` / `NTP service: active`; `:26-28` poll/ref/shadow active,
  heartbeat 2.7 / 0.5 / 0.9 s. ✔
- **Criterion 2 (gaps)** — independently re-parsed `capture/2026-11-01/gaps.jsonl`: 32 rows; **9** before 05:50:02Z
  (= `gaps-count-before.txt:2` `gaps_2026-11-01=9`); **30** inside 05:30–07:30Z, all `"reason": "overrun"`, all
  `"what": "book"`; the only non-overrun rows are a `TimeoutError` + `error streak ended` pair at 01:25:40Z/01:26:02Z,
  i.e. **outside** the window — so "no error/crash/restart record in the window" is true. `gaps-events-slice-LATE.log:7`
  `rows_total=32 rows_in_window=30`. ✔
- **Capture continuity** — independently recomputed from `capture/2026-11-01/books.jsonl`: the ten 10-minute buckets
  05:45→07:25Z are exactly `[505, 511, 631, 593, 569, 548, 516, 542, 627, 610]` (sum 5,652); the last row before 07:25:00Z is
  `2026-11-01T07:24:59.687172Z`. This is a direct refutation of a seam hole at 06:00Z. ✔

Corrections: **C3** — ADJUDICATION.md:60 "Same commands as `run.sh`" is not true. `late-leg.sh` reproduces the snapshot,
units, gap-count, readiness and slice steps, but it **omits `dst_slot_audit.py`** (run.sh:132-139) and the `EV ...` detail
rows (run.sh:112-114); it also (correctly) omits `ARMED_RUN_DONE` and the exec.log/RUN.md writes. Since the section exists to
establish the reconstruction's fidelity, it should say exactly what was and was not re-run. **C8** — ":84 last row in window
07:24:59.687Z" is the last row before 07:25:00Z; the last book row in the 05:30–07:30Z slice window is `07:29:59.687148Z`.
Ambiguous, not false.

## 5. The overall verdict — honest, complete, not self-certified. **PASS**

- **INCOMPLETE is the right verdict.** The second armed instant has no observation; the evidence is missing and cannot be
  recovered by any later run. Calling it PASS would be a fabrication; calling it FAIL would assert a defect the data does
  not show. INCOMPLETE is exactly right.
- **It does not overclaim.** The LATE leg is explicitly fenced (":87-89") and the "failures" are named as procedural.
- **It does not self-certify.** ":89 `it is not a checker signature. Verdict stays INCOMPLETE until a fresh-context checker
  signs C-1`" and ":98 `=> UNVERIFIED by the checker`". This is the opposite of self-certification.
- **It is complete on the material points.** Absence of the after-leg artefacts, absence of an `ARMED_RUN_DONE`, absence of a
  runner, absence of a checker artefact, and the cause are all present and all verified above.

Minor overstatements, all covered by conditions: ":12 "PASSED its own criteria" (C6) and ":74 "PASS across the repeated hour"
(C7 — the after-LATE snapshot is 13:37:50Z, 6h22m after the target; it proves no reset over 05:50:02→13:37:50, which is a
*stronger* interval than the hour, but it is not a measurement at 07:15Z).

## 6. Conditions (must be applied before this document is quoted)

- **C1** (ADJUDICATION.md:47): "10h29m" → **"10h 0m 29.8s"** (systemd `10h 29.808s` = 10 h + 29.808 s; old app scope start
  20:47:51Z, matching `dsh-subprocess-probe-2987977-*` at 20:47:54Z).
- **C2** (:52): "(`journalctl -k` window empty)" → "kernel log contains 82 lines, all UFW BLOCK; no OOM/OOM-kill".
- **C3** (:60): "Same commands as `run.sh`" → "same snapshot/readiness/slice steps; `dst_slot_audit.py` NOT re-run; EV detail
  rows omitted".
- **C4** (:82): "(6 book gaps / 270 s)" → the 05:50Z read held **7** overrun/book rows (05:45:26–05:45:59Z) plus 2
  trades rows = 9 total; 270 s is `readiness-before.log:4`'s correct book duration.
- **C5** (:94): the quoted `find board/ -newermt ...` does **not** return only the before-leg + LATE files — e.g.
  `board/probes/team-d/mcp-handshake-20261101T084546.txt` (13:45:46Z) and `board/reports/MONDAY-PACKET.md` also qualify.
  The substantive claim (no trading-checker artefact after `2026-11-01T02:47:24Z`) is true; the sentence should be narrowed
  to "no trading-checker file".
- **C6** (:12): "PASSED its own criteria" → "passed its before-side checks; RUN.md criteria 1/3/4 remain unmet".
- **C7** (:74): "PASS across the repeated hour" → "no reset over 05:50:02→13:37:50Z, which covers the hour".
- **C8** (:84): qualify the "last row" as the last row before 07:25:00Z (or cite 07:29:59.687148Z for the 05:30–07:30Z slice).

## 7. What remains UNVERIFIED (stated, not passed)

1. **The identity of the systemd client** that issued the stop. The journal records "on client request" but not the caller.
   The app-teardown attribution rests on: 1 s co-timing with the old app scope's release, the DSH scope namespace
   (`dsh-subprocess-<pid>`), and the new instance starting 2 min earlier. Strong, but inferential.
2. **Why the old app instance was torn down** (user close / relaunch / crash) is not recorded on disk.
3. **The 07:15:00Z instant itself is unrecoverable.** This is the substance of INCOMPLETE and no reconstruction can change it.
4. **Replay divergence (observation, not a DST defect):** the LATE readiness's replay produced 11,737 decision lines vs 11,728
   live (+9; the before leg shows the same +9 shape, 8,083 vs 8,074). Readiness PASSes on the common prefix, and the
   adjudication's one-line quote is accurate but omits the divergence. Flagged for the Lead; not introduced by the DST leg.

## 8. Checked, and NOT a defect (to pre-empt a false alarm)

- `readiness-before.log:9` says `chain_integrity decisions: 8092 lines` while `chain-head-before.txt:2` says 8074 lines;
  likewise `readiness-after-LATE.log:9` (11,746) vs the LATE snapshot (11,728). This is the ledger growing between the
  snapshot and the readiness check's own read — the same +18 shape in both legs. Not a contradiction.
- The `-0400` offsets on pre-fallback mtimes and `-0500` on post-fallback mtimes are the DST shift itself, resolved from
  epochs; no file was backdated or rewritten.
- `run.sh` sha256 = `9e1b564fef4fa03a733a5fb07e49a75c436368803a228a83fd8a89637bf1c967`, unchanged since the dry test
  (`dry-test.md:5`); `late-leg.sh` is a separate file, so the armed script was not edited by the reconstruction.
- P0.1 exclusion is real: `board/probes/team-c/monday-2026-11-02-precheck-signoff.md:49-50` — "It does **not** cover ... the
  two clock-instant checks (verified separately from their own artefacts)".

## 9. Method (reproducible)

Read-only commands used: `journalctl --user -o short-iso --since/--until` (windows 05:30–07:30Z, 06:44–06:52Z, 00:00–09:00Z);
`journalctl -k` same window; `systemctl --user show ... -p NRestarts,ExecMainStartTimestamp`; `uptime -s` / `last reboot`;
`ps aux`; `stat -c '%y %Y'`; `sha256sum` over `head -N` of both ledger files; a streaming JSON pass over
`capture/2026-11-01/{gaps,books,events}.jsonl`; `find`/`grep` over `board/`. Live ledger advanced monotonically during the
audit (decisions 11,728 → 12,043 lines; seq 12,042), with seq still strictly consecutive — i.e. the capture stack is healthy
now, which is a post-hoc observation and is **not** presented as evidence for the 07:15Z instant.

— end of report —
