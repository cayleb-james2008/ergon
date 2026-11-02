# Lane real-git fix: install evidence (2 Nov 2026, ~09:17-09:21 ET)

Executed per INSTALL-lanefix-0.3.3.1.md under Nox's go (09:16 ET). Result: ALL GATES PASS, no rollback.

## Pre-state
- Lanes: only `lead` live, idle; order o1 blocked; nothing mid-turn. Chain head seq 176; manifest 4cc45e71...
- Installed lanes.js sha256 eee771d3e99ea5d251c013c899c6206568769952a2e88c62273b64a0642e08d9
- Backup: $DH/fleet/backups/lanefix-20261102T141715Z (lanes.js, VARIANT.md, SHA256SUMS)

## Steps
- Stop: `systemctl --user stop dsh-fleet-consolidated` -> inactive in 1 s, no listener on 8790/3092, control socket gone, no fleet process.
- Single-writer guard: first attempt aborted by a FALSE POSITIVE (my own `bash -c` matched `pgrep -f`; documented as D2 in RUNBOOK-r2). Nothing had been changed. Re-ran with an anchored pattern: ok.
- Patch: lanes.js sha256 6c0ebbb21d25757e0d419a40960d3a07ca2c09ff7cb9cf449f776e0a216e1087 (identical to branch finish/lanefix); VARIANT.md note appended.
- Start: `systemd-run --user --unit=dsh-fleet-consolidated --collect ...` (runbook shape, same Environment). Up in 3 s on 127.0.0.1:8790, 127.0.0.1:3092, 100.114.17.67:8790, [::1]:8790; log scan clean (no error/degraded/EACCES/EADDRINUSE/ECONNREFUSED).

## Gates
| gate | result |
|---|---|
| H1 status | integrity.ok true; chain ok, verified_at_boot 176, head 177 (restart event); rows ok, pinned 8, disabled 12, failures []; manifest_sha256 4cc45e7148efe7fa...; sandbox ok; lanes_allowed true |
| H3 providers | opencode-go credential.present True; anthropic/claude-code/codex False (booleans only) |
| H4 MCP | initialize, tools/list (9 tools), fleet.status, actions.status all PASS (runbook client, token never printed, no board writes) |
| H5 lane | `lane-spawn builder start:false` -> parked/opencode-go; `git rev-parse --show-toplevel` == lane dir; .git real dir; branch main; hooks inherited 0; commit in lane OK; parent DSH repo head unchanged; lane-stop -> stopped |
| A10 desktop profile | cordis.patch.yml 77476b79... and package.json ce364672... exact match; desktop app scopes (3) still running, untouched |
| A11 single writer | writers.py: the only process using this DSH home is dsh-fleet-consolidated.service (the other entry is a scratch home elsewhere) |

## After install
- done-when-amend o1 rev 2 applied (event 187, post 14); it auto-resumed the blocked order (event 182, post 13) and the Lead lane went `running` (woken by the board post).
