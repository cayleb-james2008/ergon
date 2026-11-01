# MONDAY 07:45 READINESS — Lane 6 note (drift named)
**2026-11-01 14:56 ET · read-only. Verdict: Monday desktop bookings GO; my lane's runbook valid; two superseded references + one timing nuance named.**

## 1. Bookings armed (verified in the live schedule store `storages/schedule.json`)
| booking | scheduledAt (UTC) | state |
|---|---|---|
| Mon 01:00 ET — usage reset + lane resumes | 2026-11-02T06:00:00Z | armed |
| **Mon 07:45 ET — loose-ends sweep** (prompt points at `MONDAY-PACKET` + `team-e-monday-sweep-runbook`) | **2026-11-02T12:45:00Z** | **armed** |
| Mon 09:05 ET — futures guard + attestations | 2026-11-02T14:05:00Z | armed |
| Mon 16:00 ET — flat proof + outcome check **+ N5 + G2.5 window** | 2026-11-02T21:00:00Z | armed (title carries your N5 steer) |
| Every 2 days — GitHub estate read-only sweep | 2026-11-03T14:00:00Z (Tue 09:00 ET) | armed (Lead-owned) |

## 2. 07:45 runbook — still valid
- Runbook file unchanged since 31 Oct 19:19; dry-run asset present (`monday-dryrun.sh`, sha256 `f75e032a…`; last verified dry-run log 31 Oct 19:05). Checks A (Duet/Kimi), B (trading session/feed + CDP proof), C (surfaces resumed) are still command-accurate.
- **Today's changes do not touch any runbook step**: the runbook uses no fleet wrapper, no sudo/polkit, no installed-tree path, and the V3/Stage-2 restart did not change any user-unit state its checks read. Pre-flight inputs (Nox sensor STATUS.md, BOARD.md, board report mtimes) unaffected.
- Backup owner remains `lead` (§5); missed sweep must be reported as missed.

## 3. Drift named (only these)
1. **Runbook §0 / §6 GitHub cadence is superseded:** §0 says the scheduler refused and gives the manual fallback, §6 expects "Mon 2 Nov 18:00 ET manual". The Lead re-armed a **GitHub 2-day sweep, next Tue 3 Nov 09:00 ET (14:00Z)** — manual fallback no longer needed; the 07:45 sweep should note it as scheduled, not pending.
2. **Fleet invocation/wrapper changed since the runbook was written:** `b2178a05…`/MainPID 313468, wrapper `eb0ad024…` (now healthy, NRestarts 0). No runbook dependency; recorded for the record.
3. **V3 drop-in removal cannot be independently verified from this seat** (`/etc/sudoers.d` and `/etc/polkit-1/rules.d` are root-only; a non-interactive `sudo -n -l` shows only the *standing* owner-approved fleet NOPASSWD shapes, not the window-specific drop-in bytes). The runbook does not use them; no action needed — recorded as unverifiable-from-seat, not as drift.
4. **N5 timing nuance:** the 16:00 booking carries the N5 steer, but the approved window is **16:20** (`WINDOW_START=2026-11-02T16:20:00-05:00`); execution happens after the outcome check, as instructed. State verified: `APPROVED.md` present, extension-host PID 2960775 alive, `:9222` still chromium 2960519, nothing executed.

## 4. N5 arming re-verified (unchanged)
`APPROVED.md` at the script's marker path · time-gated `--apply` (refuses before Mon 16:20 ET, proven) · target PID 2960775 alive · Chromium 2960519 still owns :9222 · no manifest moved yet. **After the 16:20 outcome check on the Lead's wake: run, verify acceptance, file N5-RESULT.txt.**

## 5. Patch + records (Lead follow-ups, 15:00 ET)
- **Runbook patched:** §0 now carries a PATCHED banner (recurring GitHub pass ARMED by the Lead, next Tue 3 Nov 2026 09:00 ET / 14:00Z; manual fallback retained only as history/fallback) and the old "manual" paragraph is marked SUPERSEDED; §6 item 5 updated to the armed Tue 09:00 cadence. All other sections untouched.
- **N5 timing recorded as correct per approval, no change:** the 16:00 booking carries the steer, the approved window is 16:20, execution happens after the outcome check.
