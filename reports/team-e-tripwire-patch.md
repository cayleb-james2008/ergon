# TRIPWIRE PATCH — E5b evidence (v1 → v2.1), 2026-10-31 18:31-18:39 ET

Author: `upkeep-lead` (Team E). Approved by `lead` ("patch + restart the passive tripwire next batch —
record before/after evidence, a controlled catch test that does not touch other apps, the two sensor
defects fixed, and a rollback command"). Everything below was run read-mostly on `outis-desktop`; the
recorder observed, it signalled nothing. No application was touched, started, stopped or closed.

Files: `~/workspace/harness-build/evidence/incident-20261031-app-kills/`
- `tripwire.py` — patched (sha256 `e54ee6428fd110ffd8f32d21a1ccce536e714f002177d3c47655a51f9b30314f`)
- `tripwire.py.bak-20261031T222918Z` — original (sha256 `3df138b96734e50ff71170235005942b2a1d82ca64b27e07c546aad59b18f886`)
- `tripwire.log`, `hypr-events.log` — same append-only logs as before
- Evidence of the run: `board/probes/team-e-e5/tripwire-{before,restart,tests,tests-v21}.log`

---

## 1. Why (the two defects, restated from the first batch)

1. **Flatpak gap** — v1 matched Sober with `cmdline[0].endswith('/sober')`. Sober runs as a flatpak
   (`app-flatpak-org.vinegarhq.Sober-…scope`, launched via `gtk-launch`), whose host-visible argv is
   bwrap's, so **Sober deaths were invisible**: Sober's scope ended at 12:27:32 today and v1 logged
   nothing (`tripwire.log` stayed at its single 10:20:33 line).
2. **Zombie race** — v1 logged an exit only `if not os.path.exists('/proc/<pid>')`. A process that is a
   zombie for even one 0.1 s poll disappears from the comparison set without ever being logged, so
   exits that get coredumped (e.g. the Chromium SIGSEGV at 16:19:21) can be silently dropped.
   Additional v1 weaknesses fixed while in there: no close-focus capture, duplicate killer lines, no
   restart marker, and a Hyprland reader thread that dies on a transient socket error.

## 2. What the patch does (v2.1)

| # | Change | Why it matters |
|---|---|---|
| 1 | `cgroup_sober()`: a pid whose `/proc/<pid>/cgroup` names `org.vinegarhq.Sober` is treated as the Sober app (checked every ~2 s; zombies skipped) | catches the flatpak app that v1 could never see |
| 2 | `GONE` is logged on set difference alone, with `proc_still_exists=<bool>` recorded instead of gating on it | zombie/coredump exits are no longer lost |
| 3 | Hyprland reader tracks the focused window id; a `closewindow` of a **non-focused** window writes `CLOSEFOCUS` + a 5-second process/journal snapshot | that close cannot be a keypress: it is a signal, a close request or CDP — the app-kill signature |
| 4 | killer-exec lines are written once per pid (v1 wrote one per 0.1 s poll) | no duplicate noise |
| 5 | `START <version> pid=<n>` line at boot | restarts are visible in the evidence log |
| 6 | Hypr reader survives socket errors (`try/except`) and reconnects every 2 s | v1's reader could die silently |
| 7 | **v2.0→v2.1 fix found during this test**: the cgroup scan re-added zombies and logged the same `GONE` every 2 s (observed: 13 lines for one pid) — zombies are now skipped and each pid's exit is reported once | test-driven correction, recorded below rather than hidden |

## 3. Before / after (quoted)

**Before** (22:29:18Z, `tripwire-before.txt`):
```
ActiveState=active  SubState=running  MainPID=1509552  ActiveEnterTimestamp=Sat 2026-10-31 10:08:38 EDT
NRestarts=0  Nice=19  CPUWeight=10  IOSchedulingClass=3  WorkingDirectory=!/home/cayleb
tripwire.log 291 bytes mtime=2026-10-31 10:20:33.93   hypr-events.log 214928 bytes mtime=18:29:15
```
**After** (22:38:33Z):
```
ActiveState=active  MainPID=3178689  ActiveEnterTimestamp=Sat 2026-10-31 18:35:25 EDT  NRestarts=0
Nice=19  CPUWeight=10  IOSchedulingClass=3  WorkingDirectory=/home/cayleb
tripwire.log 39588 bytes    hypr-events.log mtime advancing (18:36:44 last sample)
START line: 2026-10-31T18:35:25.341 START v2.1-20261031T2236Z pid=3178689 watch=chromium,sober(flatpak) kill_exec close_focus
```
Delta to note honestly: `WorkingDirectory` renders as `/home/cayleb` rather than `!/home/cayleb` (the
`!` is a systemd display prefix; the process cwd is `/home/cayleb` in both cases). Everything else is
property-identical; the unit name is unchanged (`fleet-incident-tripwire.service`), so prior references
still resolve. Restart command used (documented for replay):
```bash
systemctl --user stop fleet-incident-tripwire.service
systemd-run --user --unit=fleet-incident-tripwire \
  --description='Passive incident tripwire (logs window events, killer execs, app disappearance; signals nothing)' \
  --property=Nice=19 --property=CPUWeight=10 --property=IOSchedulingClass=idle \
  --working-directory=/home/cayleb \
  /home/cayleb/workspace/harness-build/evidence/incident-20261031-app-kills/tripwire.py
```

## 4. Controlled catch tests (no real app touched)

All stand-ins were synthetic: a `sleep` re-argv'd as `/usr/lib/chromium/chromium`, and a throwaway
systemd **scope named like the Sober app** so the cgroup matcher sees exactly what it would see for the
real flatpak. None of the real Chromium/Sober processes were signalled or disturbed.

| Test | Command shape | Result (quoted from `tripwire.log`) | Verdict |
|---|---|---|---|
| Selftest (focus logic) | `./tripwire.py --selftest` | `SELFTEST PASS closefocus` (exit 0) | PASS |
| A — chromium main death | `bash -c 'exec -a /usr/lib/chromium/chromium sleep 25' &` then `kill` | `18:35:30.410 GONE chromium pid=3178760 proc_still_exists=False` — exactly **1** line, still 1 after 3 more cgroup scans | PASS |
| B — Sober flatpak scope death | `systemd-run --user --scope --unit=app-flatpak-org.vinegarhq.Sober-E5TEST3 sleep 40`, pid found by `/proc/<pid>/cgroup` match, then killed | cgroup match printed `app-flatpak-org.vinegarhq.Sober-E5TEST3.scope`; `GONE sober pid=3185496` — exactly **1** line, still 1 six seconds later | PASS |
| C — de-duplication | historical audit of the v2.0 bug | 13 `GONE sober pid=3176541` lines between 18:32:26 and 18:32:53; **0 after the v2.1 START at 18:35:25** | PASS (fixed) |
| D — Hyprland subscription | threads + wchan + log mtime | `threads: 2`; main thread `wchan=hrtimer_nanosleep`, reader `wchan=unix_stream_read_generic`; `hypr-events.log` mtime advanced to `18:36:44` (after the 18:35:25 restart) | PASS (live) |

Test scopes were stopped/collected afterwards; `systemctl --user list-units` shows no `E5TEST` unit
remaining and no test `sleep` process (verified 22:38Z).

## 5. Rollback (one command, exact)

```bash
D=~/workspace/harness-build/evidence/incident-20261031-app-kills
cp -p "$D/tripwire.py.bak-20261031T222918Z" "$D/tripwire.py"
systemctl --user stop fleet-incident-tripwire.service
systemd-run --user --unit=fleet-incident-tripwire \
  --description='Passive incident tripwire (logs window events, killer execs, app disappearance; signals nothing)' \
  --property=Nice=19 --property=CPUWeight=10 --property=IOSchedulingClass=idle \
  --working-directory=/home/cayleb "$D/tripwire.py"
# verify: systemctl --user show fleet-incident-tripwire.service -p ActiveState,MainPID
# and:    sha256sum "$D/tripwire.py"  -> 3df138b9…  (the original)
```
Rollback restores the byte-identical v1 script; the log files are append-only and are not modified by a
rollback (history is preserved, including the 13 v2.0 duplicate lines, kept as evidence of the bug).

## 6. Limits / residual risks (stated, not hidden)

- **CLOSEFOCUS was not exercised against a live window** — doing so would mean opening and closing a
  window on Cayleb's desktop, which is intrusive and was out of scope for a read-mostly batch. The
  decision function is unit-tested (`--selftest` PASS) and the tracking code runs in the same verified
  live reader (test D); the first real unfocused close will produce the first `CLOSEFOCUS` line. Treat
  the first such line as a lead, not proof: Hyprland can emit the focus change before the close.
- **Snapshots are noisy by design** — each catch pastes up to 40 young processes and ~1.8 KB of journal
  tail (which can include other agents' command lines). That is how the sender gets named; it also means
  `tripwire.log` grows fast on events. Current size 39.6 KB after five test events.
- **The recorder still cannot name the sender by itself** — it captures what happened in the 5 s before
  an exit (processes, cgroup, journal). Naming a sender needs the next real occurrence.
- **Cost**: cgroup scan every ~2 s over `/proc` plus the 0.1 s killer poll — the unit runs at
  `Nice=19`, `CPUWeight=10`, `IOSchedulingClass=idle` (measured ~1 % CPU before and after).
- **v2.0's 13 duplicate lines remain in `tripwire.log`** (18:32:26-18:32:53) — deliberately not deleted;
  §4-C documents them as the bug that v2.1 fixed.

---

## 7. v2.2 (19:13 ET) — parent capture on every GONE (Lead follow-up, one restart)

**What changed:** every `GONE` line now appends `ppid=<n> parent_alive=<bool> harness=<yes|no>
parent=<parent command line, 160 chars>`. The parent pid is captured **while the main process is alive**
(`PPID` map refreshed each scan; `/proc/<pid>` is often gone at GONE time), the parent's existence is
re-checked at GONE, and `harness=yes` fires when the parent command line looks like a test runner
(`node/npm/npx/pnpm/yarn/playwright/pytest/python -`, `node … --test`, `node tests/…`). So a
harness self-close (parent = node test run) and a browser whose parent is systemd/uwsm are
distinguishable at a glance. Version `v2.2-20261031T2320Z`, sha256 `34c2a202…` →
compiled, `--selftest closefocus=PASS parent_info=PASS`.

**Restart (one, recorded in `board/probes/team-e-e6/tripwire-v22-restart.log`):**
before `active, MainPID 3178689, since 18:35:25` → after `active, MainPID 3260368, since 19:13:06,
NRestarts=0`; same unit name/properties; `START v2.2 … gone_parent` line in the log. **Rollback unchanged:**
`cp tripwire.py.bak-20261031T222918Z tripwire.py` + stop + the documented `systemd-run` line (§5), which
restores the byte-identical v1 script.

**Controlled tests after the restart (no real app touched):**
| Test | Result |
|---|---|
| harness parent, alive at GONE | `19:18:05.819 GONE chromium pid=3274147 … ppid=3274145 parent_alive=True harness=yes parent=node --test tests/dashboard-drafts.test.mjs …` |
| non-harness parent (bash) | `harness=no parent=?` / parent exited → `parent_alive=False` (correct) |
| parent died first (child reparented) | `ppid=1387 parent_alive=True harness=no parent=/usr/lib/systemd/systemd --user` — **known limit:** if the spawner dies before the browser, the logged parent is the *current* (reparenting) parent, not the original spawner. Fixable later by keeping first-seen ppid; not worth another restart tonight. |

## 8. RECONCILIATION — the 18:55:14 GONE was the programme's own checker run (LEAD CLOSED)

Evidence (read-only, no intrusive tests):
- Tripwire snapshot at the GONE moment: child chromium main `3221336`, parent `3221324`, grandparent
  `3215402`; parent argv `node --test-reporter-destination=/tmp/ergon-test-output-RWppI4/tap.txt
  --test-reporter=tap tests/dashboard-drafts.test.mjs`; **parent `etimes=0`** (started <1 s before,
  **still alive at the GONE** — it appears with full args in the same `ps` snapshot).
- The independent `ergon-checker`'s suite log `/tmp/ergon-checker/logs/suite-p20-709bf0e.log`
  (mtime 18:58:58; `.time` = `real 4m24.757s` ⇒ run window ≈18:54:34–18:58:59) contains
  `CHECK /tmp/ergon-checker/wt/p20-709bf0e/support --test-reporter=tap tests/dashboard-drafts.test.mjs`
  — the **same test file and reporter shape** as the parent's argv; its worktree was prepared 18:53 and
  the tap-dir naming (`/tmp/ergon-test-output-*`) matches the parent's destination path.
- 18:55:14.988 falls **inside** that 4 m 25 s window and **outside** Team A's suite windows
  (22:00–22:27Z, 22:35:49–22:40:01Z = 18:00–18:27, 18:35:49–18:40:01 ET).
- No coredump for 3221336; `proc_still_exists=True` (normal reap path, not a crash); **no `CLOSEFOCUS`**
  (no window was closed); the interactive browser (2960519) untouched.

**Verdict: expected own-process close of the programme's own independent checker run.** It is **not**
evidence of an external killer. The app-kill mystery therefore has **no surviving candidate sender** and
stays honestly open with the tripwire armed (v2.2, parent capture live). Team A's evidence stands on its
own: its suite launches its own Playwright Chromium and closes only that instance (`browser.close()` in
finally), 0 repo-wide hits for pkill/killall/process.kill on chromium, and Cayleb's pid 2960519 untouched.

## 9. Known precision limit observed at 19:18 ET — flatpak launch helpers log as momentary "sober" GONEs

At 19:18:17 ET a real Sober launch happened (`uwsm_app-daemon: received: app -- gtk-launch
org.vinegarhq.Sober.desktop` → `Started app-flatpak-org.vinegarhq.Sober-2506056210.scope`), and the
window opened normally at 19:18:21 (`openwindow>>…org.vinegarhq.Sober,Sober`, then focused). The
recorder logged **7 `GONE sober` lines** in that 3-second window for the launch's short-lived helpers —
`bwrap --args 77 -- sober --`, `bwrap … xdg-dbus-proxy`, `/proc/self/exe` — each with
`parent_alive=True` and `harness=no`. **These are not app deaths**: Sober is alive and running
(`ps`: `bwrap --args 77 -- sober --` still present; window open). The cgroup matcher counts every process
in the Sober scope, including helpers that exit during startup.

Why it matters for the mystery: a future reader must not read 7 `GONE sober` lines at a launch as an
app-kill. **Refinement for a later batch (not done tonight, no restart):** treat only the scope leader as
the app main and ignore `bwrap`/`xdg-dbus-proxy`/`/proc/self/exe` helpers; or require the process to have
survived ≥2 s before its exit is reportable. Recorded here so the log's shape is understood.

## 10. QUEUED for the next tripwire touch (Lead order, 2026-10-31 19:2x ET — no work tonight)

**Item:** land the Sober-launch refinement that removes the last known false-positive class (§9).
- **Option A (preferred): leader-only.** Treat only the flatpak *scope leader* (`bwrap --args 77 -- sober --`,
  i.e. the process whose `/proc/<pid>/cgroup` is the scope root and whose ppid is the user manager, or the
  lowest pid in the scope) as the Sober app main; ignore `xdg-dbus-proxy`, `/proc/self/exe` and other
  helper children.
- **Option B (fallback): lifetime rule.** Report a Sober GONE only if the process lived ≥ 2 s (helpers die
  in <1 s during startup). Cheap and safe; loses very short-lived real crashes, so A beats B unless A proves
  unreliable in testing.
- **Acceptance:** (i) a real-looking launch sequence (headless synthetic flatpak-ish scope with helper
  processes) produces **0** `GONE sober` lines while the "app" stays up; (ii) killing the scope leader
  produces **exactly 1** `GONE sober` with parent capture; (iii) `--selftest` passes; (iv) one restart with
  before/after MainPID + rollback recorded as in §7.
- **Explicitly not done tonight** (Lead: "no work tonight"). Next batch: E7b.
