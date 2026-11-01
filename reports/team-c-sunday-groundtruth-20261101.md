# TEAM C — SUNDAY GROUND TRUTH (C-1…C-4) — 1 Nov 2026

trading-lead, 2026-11-01 08:5x ET / 13:5xZ. Sunday: **no live trading action exists and none was taken.** Read-only except
board files. No order, no wallet, no credential, no Kalshi flip, $0 spend. Every line below was produced by running the
command shown (or is quoted from a file whose timestamp is given). Full C-1 evidence:
`board/probes/team-c/dst-20261101T0715Z/ADJUDICATION.md`.

## C-1 — DST clock-instant check: **INCOMPLETE; adjudication independently signed PASS**
- 05:50Z leg ran and its **before-side checks** passed (RUN.md criteria 1/3/4 are two-instant criteria and remain unmet):
  `readiness-before.log` read at `2026-11-01T05:50:02Z`, day 2026-11-01, `EXIT=0`, 9 PASS / 1 NA / 0 FAIL; `units.txt` at
  05:50:02Z shows `EDT, -0400` (pre-fallback) and poll/ref/shadow active.
- 07:15Z leg **never ran**: no `chain-head-after`, `readiness-after`, `gaps-events-slice`, `dst-slot-audit` files; no
  `ARMED_RUN_DONE`; no runner process. Absence here is evidence because RUN.md fixed the artefact paths before the run.
- Cause **confirmed** (upgraded from "unexplained"): the armed runner (bash pid 3542392, started `2026-11-01T00:21:43Z` in
  `dsh-subprocess-2987977-08f90ab27e8f.scope`) was SIGTERMed **"on client request" at `2026-11-01T06:48:20Z`** (= 01:48:20
  EST), when the old DSH desktop app instance (scope …2987797, **10h 0m 29.8s** wall clock) was released and a new instance
  started at 06:46:16Z. Wall-clock arithmetic matches (06:48:20Z − 6h26m37.919s = 00:21:42.081Z ≈ the 00:21:43Z start).
  `nohup` gave no protection: the process lived in a subprocess scope owned by the app instance. **The identity of the
  systemd client is UNVERIFIED** (attribution rests on 1 s co-timing + the DSH scope namespace + the new-instance timing).
  **Not a DST defect** (kill is 48m20s after the fallback instant; capture never restarted). Kernel log for the window has
  82 UFW lines and **no OOM/OOM-kill**; no tripwire kill record.
- **Systemic finding + requirement:** any armed run launched from a DSH tool call dies with the app instance. The next armed
  run must be a `systemd-run --user --transient` unit (own scope, `--collect`), with a watchdog alerting if the runner is
  absent at T−5min and a T+2min assertion that the expected artefacts exist and are non-empty. A one-line runner-liveness
  check goes into the Monday 07:45 sweep (Lead ruling 2026-11-01T13:2xZ).
- Checker audit **absent at adjudication time**: newest trading-checker file was `trading-checker-kronos-verification.md` at
  `2026-11-01T02:47Z`, ~4 h *before* the armed run; nothing written after. The P0.1 signature explicitly excludes the
  clock-instant checks.
- Late read-only re-run (Lead-approved, labelled LATE/RECONSTRUCTED; it is **not** "the same commands as run.sh" — it omits
  `dst_slot_audit.py` and the EV detail rows): `readiness-after-LATE.log` read at `2026-11-01T13:37:50Z`, `EXIT=0`,
  9 PASS / 1 NA / 0 FAIL; `CRITERION_3_LATE: PASS` — decisions seq 8073→11727, orders 59785→89520, no reset, **measured at
  13:37:50Z** (proves no reset over a stronger interval, not an observation at 07:15Z); `timedatectl` `EST (-0500)`; capture
  continuous across 06:00Z (books 505–631 rows/10 min; last row before 07:25:00Z = 07:24:59.687Z, last in the wider slice
  07:29:59.687Z); 30 gap records in 05:30–07:30Z, **all `reason: overrun`** at 15-min window boundaries, no restart/crash. The
  fallback itself was clean; the **armed** second instant remains unproven.
- **C-1 CLOSED as independently signed:** fresh Tier S checker verdict **PASS** at
  `board/reports/team-c-dst-checker-verdict.md` (14:20Z) — it reproduced every material claim from primary sources, confirmed
  the INCOMPLETE verdict is correct/complete/not self-certified, and its eight margin corrections **C1–C8** are folded into
  `ADJUDICATION.md` (inline + §7 appendix). The **07:15:00Z armed instant is recorded unrecoverable**. Two flags carried
  forward: the systemd client identity is UNVERIFIED, and a **pre-existing replay divergence** (+9 decision lines vs live;
  same shape in both legs) is flagged for the Lead — not a DST defect.

## C-2 — Monday runbook: ground truth verified, drift found, v7 written
Live checks run 13:30–13:45Z (08:30–08:45 ET), all read-only:
| Item | Command | Result at timestamp |
|---|---|---|
| Kalshi units enabled | `systemctl --user is-enabled openbell-kalshi-{poll,ref,shadow}` | `enabled / enabled / enabled` (13:30Z) |
| Kalshi units active | `systemctl --user is-active …` | `active / active / active` (13:30Z; also 13:37:50Z in LATE `units.txt`) |
| Unit copies vs repo | `diff <installed> <repo>` x3 | **IDENTICAL** for poll/ref/shadow (no re-copy needed) |
| Kalshi capture freshness | readiness cutoff | capture books `2026-11-01T13:37:50Z`; heartbeats poll 1.2 s / ref 0.6 s / shadow 0.8 s; 2,910,685 records, **errors 0** |
| readiness.py | `python3 -I research/kalshi15/readiness.py --day 2026-11-01` | `EXIT=0`, 9 PASS / 1 NA / 0 FAIL (13:39:55Z) |
| Futures units | `systemctl --user is-active openbell-live openbell-dash openbell-tabkeeper` | `active/active/active`, all `enabled`; engine `ExecMainPID=1484`, start **Wed 2026-10-28 15:40:09 EDT** (continuous) |
| Futures rails | config dump | `mode=cards, contracts=1, max_trades_per_day=2, daily_loss_stop_usd=-400, daily_profit_stop_usd=900, cooldown_min_after_loss=15, flat_et=16:05, min_stop_pts=2.0, symbol=MESZ6, contract_id=4470322, card_valid_min=2` |
| Config↔process binding | `stat config.json` vs `ExecMainStartTimestamp` | `config_mtime=1793141014 (2026-10-27T22:43:34Z) start=1793216409 (2026-10-28T19:40:09Z)` → **CONFIG_OLDER_THAN_PROCESS: PASS** |
| Futures feed (weekend baseline) | `heartbeat.json` | `age_s=3.2 status=running mode=cards paused=False qty=1 contract_id=4470322 feed="retrying (RuntimeError)" feed_age_s=146367.1 market_open=False last_bar_et=16:59` |
| Tab route | `ss -ltnp \| grep :9222`; `cat live/logs/tabkeeper.json` | Chromium **listening on 127.0.0.1:9222** (pid 2960519); `"tradovate": "missing"`, `"et": "2026-11-01T08:39:17-05:00"` — expected on a Sunday; Monday must turn this into a positive tab+bar proof |
| Runbook hash | `sha256sum board/reports/monday-futures-runbook.md` | `027ca4e4cc1427e3300e2d36e0eda4380bfb3d8410c32ba8847533347adb4eff` — **exactly the P0.1 signed hash, signature not void** |
| Host clock | `timedatectl` | `US/Eastern (EST, -0500)`, synchronised, NTP active — the 09:05 clock leg is satisfied today but must be re-run on the day |

**Drift found in `board/reports/MONDAY-PACKET.md` (22:31 ET) and corrected in the v7 file:**
1. **FALSE (**line 12): "DST artefacts from 05:50Z/07:15Z … verified by trading-checker at ~07:20Z". No 07:15Z artefacts
   exist and no checker audit exists. Corrected to INCOMPLETE + the cause + the LATE evidence + checker-pending.
2. **Missing:** runner-liveness check in the 07:45 sweep and the supervisor requirement for the next armed run (Lead ruling).
3. **Missing:** the pre-09:05 P0 block (P0.1–P0.4) and the (a2) **positions-panel** flat read (`dom_read_positions.py`;
   exit 3/4 = UNVERIFIED = day blocked by design). The packet summarised the 09:15 five attestations but not the leg that can
   actually end the day.
4. **Missing:** failure disposition — Cayleb's desktop sign-in is the **only verified route**; fresh credential/2FA demand
   = STOP; never another host, never a manufactured fill.
5. **Missing:** the 16:05 two accepted proofs (fresh `cashBalance` frame ≥16:00 **or** positions-panel read written ≥16:00,
   positive flat statement only; ambiguity = FAIL) and the 16:20 item list (headline = day-filtered completeness PASS/FAIL,
   PASS-empty defined).
6. **Stale** (01:00 Kalshi bullet): "three rulings + L*=30s are FOR NOX where marked". **L\* is now ruled** (Nox batch-2
   item 6) and applied to `DECISIONS.md` + `SPEC.md`; the freeze tooling is **built and tested** (no longer the resume pack's
   "NOT BUILT"). What remains unruled: the stale-header wording (Amendment 5) and the Kelly posture (Amendment 6); plus two
   one-line flips left deliberately (see C-3).
7. **Missing:** the weekend baseline above, so Monday 09:05 has a compare point ("feed retrying since Friday close, tab
   missing, last bar Friday 16:59, engine continuous since 28 Oct" — any deviation at 09:05 is the finding).
8. **Note:** P0.1 signs v6 only; every attestation still needs its own Monday evidence. The packet's "P0.1 is signed but each
   attestation still needs its own Monday evidence" is correct and kept.

Deliverable: `board/reports/team-c-monday-packet-v7.md` (full sequence, owner + backup + fallback, corrected DST line). It is
a **Team C file by discipline** — the Lead's `MONDAY-PACKET.md` is not mine to overwrite; adopting v7 (or copying the
corrected 01:00/07:45 lines into the packet) is the Lead's write. The runner-liveness check is a runnable read-only script:
`bash board/probes/team-c/runner-liveness-check.sh` (verified 13:50:45Z: reports "NONE ALIVE", flags the DST dir
`INCOMPLETE — no ARMED_RUN_DONE`, and shows the rollover dir completed with its marker — the control case).

## C-3 — Kalshi shadow ground truth, read-only (no flip, no order)
| Fact | Evidence (command + result, timestamp) |
|---|---|
| Units enabled **and** active | `is-enabled` → enabled/enabled/enabled; `is-active` → active/active/active (13:30Z; LATE `units.txt` 13:37:50Z also active, heartbeat 2.7/0.5/0.9 s) |
| Installed = repo copies | `diff -q` x3 → IDENTICAL (poll/ref/shadow), so the 01:00 re-copy step is a no-op unless the repo copy changes |
| Capture freshness | readiness cutoff `capture books 2026-11-01T13:37:50Z`, observed 00:00:00Z→13:37:50Z; `capture_validator` 2,910,685 records, **0 errors**, partial_tail 0 |
| readiness.py on captured data | `python3 -I research/kalshi15/readiness.py --day 2026-11-01` → `EXIT=0`, **9 PASS / 1 NA / 0 FAIL**; NA = `nox_reconcile` (check 8, NA by construction — Nox's collector copy is the independent one; Nox repaired it 22:36Z on 31 Oct) |
| Ledger chain | decisions seq **11727** (11,728 lines), orders seq **89520** (89,521 lines); `chain_integrity` PASS, `replay_determinism` PASS byte-identical over the common prefix (122.5 s) |
| Two-scorer agreement | `research/kalshi15/results/scorer_agreement.json` (date_utc `2026-10-31T17:39:32Z`): `"result": "AGREE: 6853 rows each, 0 differences; summaries and the late side file identical (tol 1e-9, integers exact)"`; history 6,788 (10:35) → **6,853** (11:40) → **6,853** (13:45, after Nox Amendment 1). LATEST figure on disk = **6,853 rows, 0 differences** |
| WP9 (readiness/freeze/report work-package) | readiness checks implemented and green: first run 9 PASS / 1 NA (`RESUME.md`), re-run today identical shape. `freeze.py` + `forward.py` built; their tests run by me now: `python3 -m pytest research/kalshi15/test_freeze.py research/kalshi15/test_forward.py -q` → **15 passed in 0.10s** (8 + 7, matching the P0.1 signoff's claim). Freeze still correctly **refused** (7 `<FILL AT FREEZE>` prereg slots unfilled); target Tue 3–Wed 4 Nov |
| WP10 (live-path package, ships OFF) | `live/kalshi_exec/` exists with engine/guard/intent/rails/signer/venue/gateway_sim/compare + tests; `python3 -m pytest live/kalshi_exec -q` → **167 passed in 1.06s** (repo unchanged by the run: `git status` identical before/after). Guard table 64 combinations asserted (`test_guard.py:59 self.assertEqual(n, 64)`). Status: built + dry-run proven, **shipped OFF by construction**; DESIGN.md flags 7 conflicts C1–C7 for Nox (latency regime, ledger flush, "no order path" wording, Kelly vs D6, whole-contract rounding, venue semantics, machine-readable gate) |
| Drift note | the resume pack says "172 tests" for WP10; the package collects/passes **167** today. Either 5 were merged out or the 172 counted another scope — reported, not smoothed. |
| L\* = 30 s | **RULED** by Nox (batch-2 item 6) and applied to `research/kalshi15/DECISIONS.md` + `SPEC.md` (team-c-c15-c19). Two one-line flips deliberately left: `params.provisional.json "l_star_s": 60` (data file — Stage A must pass `--L 30`, or the file is flipped on Nox's authorisation) and `scorer_agreement.json latency.L_star: 60` (generated artefact — regenerates to 30 on the next `stage_a.py dump-s`; do not hand-edit) |
| Three rulings | (1) hourly bar 2000 — **RESOLVED by Nox batch-2 item 5**: 300 was a coordinator-side transcription, corrected in his §5; papers are right, no amendment our side. (2) stale header / "no order path exists" — **still FOR NOX** (Amendment 5 wording; the true text exists in tests). (3) Kelly posture — **still FOR NOX** (Amendment 6; daily loss stop untouched) |
| No live action | No flip, no order, no key material read; $51.39 untouched. The live path requires (i) enable flag, (ii) passed gate, (iii) Cayleb's written go — none exists |

## C-4 — The TWO OPEN ASKS to Nox (for the Lead to batch)
**Ask 1 — frozen Round-3 set (blocks Job 12 item 4).** There is no frozen 16-candidate set, no frozen scoring run and no
FRAGILE verdict record on this machine (`grep FRAGILE` across `~/Work` + board = 0 hits; `research/promptstrat/` holds the
prereg, `build_log.json`, `freeze_manifest.json`, sim code — not the set). Re-deriving the 16 from the brief would be a new
unfrozen construction, forbidden by the frozen rule.
*Recommended one-line answer for Nox:* "Hand over the exact frozen 16-candidate set + scoring harness as files; if it cannot
be produced, state it is lost and Job 12 item 4 is recorded BLOCKED-LOST — no re-derivation, no scoring against the frozen
rule."

**Ask 2 — shared trial ledger ownership (blocks the Kronos row's merge, Job 12 item 3 tail).**
`research/kronos/ledger_row.json` is complete; the shared trial ledger it merges into (`research/promptstrat/ledger.json`,
build item 7) does not exist on this tree. Creating a programme-level ledger from the trading lane would be inventing an
artefact this lane does not own.
*Recommended one-line answer for Nox:* "Nox/programme owns `research/promptstrat/ledger.json`; merge
`research/kronos/ledger_row.json` as its first entry (contract §17 ref, origin-list sha256 `29f7cbb5…`). The trading lane does
not create it and keeps the row as a recorded gap until then."

## C-5 — successor: see `board/reports/team-c-autonomous-path-build-plan.md`
Large-batch plan for the Tier S autonomous order path (ASK D14, proceeding unless Cayleb says stop; G0 answered: FundedNext
Flex permits own bots/EAs only through the platform login, **no API on the eval → CDP ticket route is the only path**).
Monday remains guard/preflight only; no fills claimed.
**Batch progressed after the Lead's approval (same day, 14:00Z):** **B0 done** —
`board/reports/team-c-firm-automation-verdict.md` (official clauses re-read 2026-11-01: automation explicitly allowed during
Challenge and Funded accounts on Tradovate/NinjaTrader/TradingView; no API surface offered; conditions = self-contained bot,
distinct strategy, no latency abuse/order flooding, prohibited-class list; account facts = 40% consistency + $1,000 daily
threshold **during the Flex Challenge only** (it is a recalculation rule, not a breach), no daily loss limit firm-side,
MLL $1,500 trailing EOD locking at $50,100, close by 3:10 PM CT = 4:10 PM ET, contract limit 3 e-mini/30 micro, micro-scalp
10 s, 2%-of-CME-limit prohibition). **B1 drafted** — `board/reports/team-c-autonomous-path-design-v2.md` (CDP-only §4,
G2.5 mandatory, R1–R18 rails table with the six new firm-derived refusals, threat list, kill path, gates). Both are
**drafts awaiting the G0/G1 fresh-checker seats**; no code, no config change, no order.

## What is NOT done / next
- C-1 is not closed: needs the fresh-context Tier S checker the Lead is spawning (artefact paths in ADJUDICATION.md §6).
- No Monday action taken or possible today; the 01:00 ET Kalshi resume and the 07:45/09:05/16:05/16:20 sequence are Monday's.
- Successor batch: (i) 01:00 ET Kalshi resume per the corrected packet v7; (ii) 07:45 sweep with the runner-liveness line;
  (iii) 09:05 guard → 09:15 attestations → 09:35–09:44 binding → 09:45 → 16:05 flat → 16:20 outcome; (iv) after the reset,
  the order-path G1/G2 build with a fresh checker per gate. No idle lane: the next write is the Monday guard artefact.
