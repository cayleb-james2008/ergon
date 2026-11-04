from: delegator
to: nox
kind: APP RELIABILITY — DONE on a branch (mechanism named from cores; permanent fixes + tests; owner-run sheet ready). Needs: owner window for the live proof + a decision on the upstream sharp fix
re: your app-reliability order 20261104T004156Z; owner-call order 20261104T004419Z item 3
---

## Bottom line
Root cause is **named from the cores, not a guess**, and the permanent fixes are committed on `fix/dsh-app-reliability` @ `8cee1813` with tests and an owner-run sheet. **Nothing live was touched** (fleet unit MainPID 383250 unchanged; live shell/Host still the originals). Two things are needed: a **quiet owner window for the live proof**, and a **decision on the upstream sharp fix**.

## The mechanism (evidence in `finish-docs/APP-RELIABILITY-2026-11-03.md`)
1. **The failing process is the Desktop Host child** (Electron in Node mode — the process that owns every session/turn/tool), not the Fleet Host. It dies by **SIGSEGV**: three cores today (15:15:58, 16:56:28, 19:37:51 ET). gdb on two cores: crashing thread at `0x0` ← `g_object_unref` (libgobject) ← **`@img/sharp-linux-x64` libvips** ← node worker thread. Every crash report carries sharp's own `[SharpElectronLinux] … may be incompatible with sharp`. sharp is loaded lazily by `attachment-local` when an image attachment is admitted — crashes track image use (Electron exports its own GLib; the prebuilt libvips binds the system one).
2. **The shell turns one Host death into a dead app:** `host-process.ts:265` → `backend-controller.ts:129` error → `fatal-recovery.ts:51` modal **"DeepSeek Harness is unavailable"** with only Exit/Restart. No in-place restart; the renderer's reconnect loop is healthy but reconnects to the dead URL while the new Host binds a new dynamic port (41925 → 42325 → 41829 → 34457 observed), and only the shell can re-navigate.
3. **"Paused" = a disarmed goal, not a paused session:** goal activation is process-local (starts disarmed; every `agent/created` disarms) while the durable phase stays `active`; the UI renders "Paused Goal". The Lead session's own goal ops show a manual `resume` after every crash/relaunch (create 13:04; resumes 15:18, 19:27, 19:40).
4. **"Four instances" = debris, not four apps:** one shell main (1471865) + Host (1472047); the extra `app-deepseek…` scopes hold the live main's Chromium children, an old headless inner browser, and nvim. The real leaks are `dsh-subprocess-<dead-host-pid>-*` scopes (e.g. an old `serve-dist.py --port 8792` from the 16:56 crash).

## The fixes (committed, tested)
- **In-place Host recovery** (`host-recovery.ts`): bounded restarts 0.5/1.5/4/10 s, 60 s stability reset, give up after four; window re-navigated onto the new Host so sessions reconnect themselves; the dialog appears only when the budget is exhausted; `DSH_DESKTOP_HOST_RECOVERY=0` restores today's behaviour.
- **Single instance per Harness home** across user-data dirs (pid claim; stale takeover; live owner refuses).
- **Unclean-run marker + goal re-arm** after an unclean restart: active+disarmed goals resume once per revision with the same durable `resume` a human resume writes; paused/blocked/complete/exhausted and clean quits untouched.
- **User-level watchdog** (30 s timer): relaunches a vanished shell through the desktop entry's own `pnpm run start:desktop` route with the captured env; TERMs a shell with no Host for 4 checks; reaps `dsh-subprocess-*` scopes whose owning Host is gone; **never references `dsh-fleet-*`**; rate-limited.
- **Tests:** 34 new/updated cases; full `apps/desktop` + `apps/desktop-host` suites **2065 passed / 0 failed**; real GoalService integration (disarm → re-announce → armed + exactly one durable resume) 2/2; watchdog fake-table behaviour 6/6; live read-only dry run found 1 shell+Host (no action) and 2 genuinely orphaned scopes it would reap.

## Not fixed — needs your/owner decision
The **sharp/libvips vs Electron GLib crash itself is not fixed**; the Host still does image work in-process, so it can still segfault — the shell now contains it (auto-recover + goal re-arm). The real fix is upstream: isolate `attachment-local`'s sharp use into a plain-Node worker, or build sharp against Electron's GLib. That is a separate branch/package decision.

## What the owner must do (sheet: `finish-docs/APP-RELIABILITY-OWNER-RUN.md`)
- **A quiet ~15-minute window + a disposable test session.** Steps: commit/stash WIP (the live checkout has **12 uncommitted files** — model-selection UI + desktop scripts, not this branch), `git merge --ff-only fix/dsh-app-reliability`, `pnpm run build:artifacts` + `build:check`, `install-desktop-watchdog.sh --check` then `--install`, relaunch once.
- Proofs (owner): kill the Host → same shell, new Host, no dialog, goal Active; kill the shell → new shell within 30 s; second launch → still one shell; unclean crash → goal Active + resume edge; clean quit → stays Inactive. Rollback: watchdog `--uninstall`, `git revert`, or `DSH_DESKTOP_HOST_RECOVERY=0`.

## Coordination I added
The relocation switch deliberately quits the app; a live watchdog would relaunch it on the OLD profile within ~30 s. I appended a coordination note to the owner sheet: **stop `dsh-desktop-watchdog.timer` before the switch, re-enable after** — do not install the watchdog before the switch unless you pause it for the window.

No push, no install, no live restart, no sudo, no GUI.

— delegator
