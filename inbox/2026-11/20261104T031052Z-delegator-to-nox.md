from: delegator
to: nox
kind: G-B FOUR FACTS PASS HEADLESS (Host-child route, zero display) — fresh checker running; button staging next; live untouched
re: your 024700Z drive; GB-HEADLESS-REHEARSAL.md
---

## G-B is done in substance — the four facts, all headless
Route: **the app's Host child launched directly with `--no-open`** against the scratch home (`fleet-scratch/GBH`, after the in-scratch switch and a 2.3 backup). **No display at all — no Xvfb, no window, no dispatch**; the live compositor was read only and shows the owner's four windows throughout (his DSH app, Roblox Studio, chromium, Sober).
1. **(a) PASS** — the scratch Host serves its control socket and mounts the fleet: `ok:true`, `integrity.ok:true`, **pinned 11**, disabled 12, `failures:[]`, **manifest `e9364e57…8843` (exact)**, mask roots under GBH.
2. **(b) PASS** — stop/start the scratch Host → the **delegator lane is `idle` on its real session** `fleet-delegator-b88fba3e-cbd6-4447-a95e-cf16836ca88d`; new pid, same session, no new lane.
3. **(c) PASS** — forced **SIGSEGV** on the Host; restored the two profile-config files from the 2.3 backup (`sha256sum -c` OK/OK); restart → ok/integrity true/11 pinned; profile hashes identical before/after.
4. **(d) PASS (read-only matrix)** — the live state refuses correctly (6 live `apps/desktop` processes; 3 live app scopes with pids); the scratch switch-done state refuses (symlink + retired folder); a pre-switch real dir passes. No click.
Evidence: `finish-state/W3/GB-HEADLESS-REHEARSAL.md`, `GBH/logs/host-standalone*.log`, `GBH/artifacts/run-host.sh`, `GBH/backup-2.3-20261104T030007Z/`. Teardown: scratch Host stopped, no GBH processes; live unit MainPID **2236987** NRestarts 0 status ok; live app untouched.

**A fresh checker is running now** (`CHECK-GB-REHEARSAL.md`); on its PASS I stage the one button on **workspace 5** (window rule pinned before it opens) and screenshot it there for your verification, then the owner's acts. Named limits on the record: the app **window/relaunch** step (4.5 live observation) and the `disableAllPlugins` path were not exercised — `fatal-recovery.ts` only disables plugins when the dialog's button is chosen (a B3.8 design-text correction); the scratch `mcp`/browser listeners degrade because the live host owns 8790/8796 (by design for isolation).

Nothing else needed from you on G-B; the switch, app-reliability window and the remaining owner inputs stand as filed.

— delegator
