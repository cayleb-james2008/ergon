# MONDAY PACKET — 2 Nov 2026 (ET). Single entry point for the overnight→Monday sequence.
State lives on disk; this file is the map. Owners in brackets. Evidence paths are under board/.
**Trading legs are corrected to v7** (`board/reports/team-c-monday-packet-v7.md`, trading-lead, 1 Nov 08:5x ET) — that file is
authoritative for the trading legs (01:00 Kalshi, 07:45 trading checks, 09:05, 09:15, 09:35–09:44, ~09:40, 09:45, 16:05, 16:20).
This map keeps the non-trading legs and the standing owner acts. Nothing here changes rails, the signed runbook or any attestation rule.

## 01:00 ET — Claude usage reset + lane resumes  [trading-lead / ergon-lead]
- Kalshi resume (Job 14): execute board/reports/team-c-kalshi-resume-pack.md in order, with v7 corrections. First: units state check
  (`systemctl --user is-active openbell-kalshi-poll openbell-kalshi-ref openbell-kalshi-shadow`) → readiness `--day 2026-11-01`
  then `--day 2026-11-02` → fit-split pull restart → Stage A order a-f. **L\* = 30 s is RULED AND APPLIED** (not pending); freeze
  tooling (freeze.py) + forward scorer (forward.py) are **built and tested** (15 tests pass); what remains FOR NOX is the
  stale-header wording and the Kelly posture. Freeze target Tue 3–Wed 4 Nov.
- Laptop lane (Job 04): the physical laptop is OUT of the topology — re-point at the Windows VM when it exists. If nothing resumes,
  report that plainly (no manufactured resume).
- B12 note: the handback question was CLOSED by Cayleb 31 Oct 17:51 (programme runs through DSH); no double-assignment.
- DST status: **INCOMPLETE** — the 05:50Z leg ran and passed; the 07:15Z leg never ran (armed runner SIGTERMed 06:48:20Z during a
  DSH app-instance teardown; `board/probes/team-c/dst-20261101T0715Z/ADJUDICATION.md`). Late reconstruction clean (13:37:50Z,
  9 PASS/1 NA/0 FAIL). Fresh Tier S checker verdict owed; nothing in it counts as passed. The old "verified by trading-checker
  ~07:20Z" line was false and is struck.

## 07:45 ET — Monday loose-ends sweep  [upkeep-lead; backup lead]
- Execute board/reports/team-e-monday-sweep-runbook.md (pre-flight verified 33 read-only commands).
- Check: Duet verdicts vs outstanding packs; trading session/feed ahead of 09:05; which lanes resumed after 01:00; reconcile the
  board's timed items. Feed: expected-weekend "no trader.tradovate.com tab" must become a PROOF at 09:05 (tab present + fresh bar).
- **Runner-liveness check** (new): any armed job must be a `systemd-run --user --transient` unit, never a `nohup` child of a
  session; alert if absent at T−5 min, assert artefacts at T+2 min. (The 07:15Z DST leg died exactly because it was a session child.)

## 09:05 ET — Futures session guard  [trading-lead; backup lead]
- Follow board/reports/monday-futures-runbook.md **v6** EXACTLY — sha256 027ca4e4cc1427e3300e2d36e0eda4380bfb3d8410c32ba8847533347adb4eff
  is pinned by the P0.1 signoff (board/probes/team-c/monday-2026-11-02-precheck-signoff.md). ANY edit voids the signature.
- 09:05 guard → 09:15 five attestations stated individually (a-e), with (e) honest "mode=cards — NOT automated"; (c) split:
  provisional 09:15 tab/feed proof + BINDING post-09:30 bar at 09:35-09:44; decision log ARMED/PROVEN; rails asserted from
  config+contract id+systemd start. Any FAIL/UNVERIFIED = the 09:45 window does not open, failing leg named.
- **P0.1–P0.4 pre-09:05 block** and the **(a2) positions-panel flat read** are explicit in v7 (§§3–4); (a2) is the leg that can
  end the day. **Failure disposition (v7 §5)**: Cayleb's desktop sign-in on the existing Chromium tab is the only verified route;
  a fresh credential/2FA demand = STOP FOR CAYLEB, never improvise; no other host, no shadow substitute, no manufactured fill.
- Checker: a **fresh Tier S checker holds veto per attestation**; P0.1 is signed but each attestation still needs its own Monday
  evidence. Do not self-certify.

## 16:05 ET — Desk flat + 16:20 outcome check  [trading-lead; backup lead]
- 16:05 flat proof (v6's accepted proof: fresh cashBalance frame with `t` OR the ~16:02 panel read written >=16:00; no conditional
  pass may mask a non-flat book). 16:20 outcome check with decision-log completeness as headline, day-filtered pass/fail
  (PASS-empty defined in v7 §7).
- Proof standard: broker record (orders/fills/P&L) + decision log, never the engine's self-report. A day with no qualifying setup is
  reported as such — never manufacture activity.

## Standing owner acts (do not bounce to teams)
- Install Wait release when Nox asks (fleet release-identity leg); rehearsal host purchase ~EUR 8/mo when Job 03 reaches it;
  Stage 3/4/5/7/8 acts later; Roblox playtest verdict when a game is playtest-ready; Roblox Studio window (P7) when Nox arranges it.
  URGENT personal: Amazon recovery, iSolve.
- Apotheca and Pump Drop never appear in reports.

## Scheduling note (Lead, 1 Nov 08:52 ET)
The previous session's armed timed deliveries died with its DSH app instance. Re-armed **in this session** as scheduled bookings:
01:00, 07:45, 09:05, 16:00 ET on 2 Nov. If this session is replaced before Monday, a successor must re-arm from this section.
