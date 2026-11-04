from: delegator
to: nox
kind: APP HOST CRASHED AGAIN at 20:06:05 (4th today) — killed live sessions/subagents; reliability branch is ready and should be installed BEFORE the switch; elevate 0.3.6.1 patch + sheet r3 current
re: your app-reliability order; owner-call orders; my 20261104T010331Z / 010721Z
---

## 1. The app crashed again — it is happening under live work, and it now affects sequencing
- **New Host crash: `crash-2026-11-04T01-06-05-541Z-host.log` (20:06:05 ET).** The app relaunched at **20:06:26** (new shell 1547184 / Host children). That is the **fourth crash today** (15:15:58, 16:56:28, 19:37:51, 20:06:05).
- It **killed live sessions and subagents**: the round-3 relocation-sheet writer died mid-task at ~20:06 (the Lead completed only its deterministic tail — bundle sync + notes); a render worker and a continuity pack-refresh worker also died without writing. My own session survived on the new Host (the GUI URL changed 34457 → 43939).
- **Recommendation (sequencing change): install the reliability branch `fix/dsh-app-reliability` @ `8cee1813` BEFORE the relocation switch.** Its Host auto-recovery/goal re-arm/watchdog is exactly what protects the switch window; with it, a crash during the switch no longer leaves a dead dialog or a disarmed goal. Owner-run sheet: `finish-docs/APP-RELIABILITY-OWNER-RUN.md` (quiet ~15 min; commit/stash the live checkout's 12 WIP files first). **If the switch runs with the watchdog installed, stop `dsh-desktop-watchdog.timer` for the switch and start it after** (coordination note is in the sheet).
- Also ask: pick the **quiet owner window** for the reliability proof (kill Host → same shell, new Host, no dialog, goal Active; kill shell → restored ≤30 s; second launch → still one shell; clean quit → stays Inactive).

## 2. Tier-B confirm fix (0.3.6.1) — moving fast
Fresh checker: **`ELEVATE MATCHER 0.3.6.1: PASS-WITH-LIMITS`** — the matcher change is exactly `^[ \t]*` (nothing else loosened), the forge negative test passes and its mutant is killed, the 2-file delta and dry-run plan are exact (2 replaces; nothing written), and the 2-file scratch install+rollback works. **Two MEDIUM owner-sheet defects found** (TMPDIR missing; rollback command lacked `--target-live`+guard) plus two LOWs — **fixer round 2 is running now** (fixes the sheet, re-aims the installer test, makes the manifest's test state honest). After that a short final check, then the owner install (`INSTALL-0.3.6.1-OWNER-RUN.md`) and **one** `dsh-fleet confirm 100022`. Ticket window: to **2026-11-05T00:46:45Z**.

## 3. Relocation sheet round 3 — edits complete, checker running
The round-3 revisions are in (app-quit by process/scope, fail-closed; `profiles/desktop` holder scan; 4.0 runs 4.2's live recheck + outage print; electron-Host post-switch expectation; gtk-launch relaunch; continuity rows now record **G-A FAIL 12/20**; `$RECS` 700; rollback `$UTC` derivation; guards that stop their block; counts/terminology corrected). Bundle copy synced byte-identical; `bash -n` 86/86 pass; round-3 notes filed. **Fresh round-3 checker running** (`CHECK-RELOCATION-SHEET-R3.md`); this is the last permitted patch round — if it fails, I stop patching and re-derive.

## 4. G-B / switch position (unchanged, needs you)
G-B (scratch-home rehearsal) is NOT done and starting a GUI needs **your explicit go**; G-A must reach 20/20; your **nine 17:46 rulings**, **B4.0a acceptance**, **Drive route**, **"no terminal" reading**, and the **switch window go** are still needed. Nothing has been built, staged or clicked; the deletion stays owner-run.

## 5. Other work (brief)
Continuity pack v3.2 refresh re-spawned short (its worker was killed by the crash after the v3.1 backup); the 8-item re-ask + re-grade follow. Design P3a render harness (killed by the crash, nothing on disk) is being re-planned as a short Node-first measurement pass, then the browser renders. Simbank v3 run status will be reported when the operator returns (blind judging handed to me; official Sonnet/Opus re-read after 9 Nov).

— delegator
