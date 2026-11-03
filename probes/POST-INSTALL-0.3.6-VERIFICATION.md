# POST-INSTALL VERIFICATION — 0.3.6 on the live fleet (Lead, read-only, 3 Nov 15:08–15:11 ET)

Install: owner-run at 15:07:56 ET from `fin-int36/release/install-0.3.6.sh --target-live`; installer ended
`INSTALL PASS: 0.3.6 is in place, the bus is mounted and a rubric verdict with criteria as text registers on the running host`.
Backup `fleet/backups/rel036-20261103T200755Z`. This file records the Lead's INDEPENDENT read-only verification; no writes, no restarts,
no token values, DB opened mode=ro and the chain verified on a consistent `.backup` copy in `fleet-scratch/POST36-CHK/`.

## Unit / version / files
- `dsh-fleet-consolidated`: active; **MainPID 383250** (old 3671044 gone); ActiveEnterTimestamp **15:07:56 EST**; NRestarts 0.
- `@cayleb/dsh-fleet` version **0.3.6**; package tree **220 files = 147 pre + 73 added**.
- Pre/post comparison against `finish-state/INSTALL36/evidence/pre-pkg-files.sha256` (147 files, tree hash `791acbac…`):
  - 0 pre files missing; 37 in-package files changed — **every one is in the release list**; **0 changed outside the release list**;
  - 73 added files — **every one is in the release list**; 0 added outside it;
  - **all 111 release-listed files match their release hash** (including `profile/cordis.patch.yml`).
- `src/orders/validate.js` sha256 `b68ce88c6de137834b6faa2a…` (must-keep kept).

## Host-rooted artefacts (vs the reviewed ones)
- `rows.manifest.json` `e9364e57…`; `presets/delegator.patch.yml` `8478e16a…`; `profiles/fleet/cordis.patch.yml` `a4713a8d…` — all equal the
  artefacts the whole-package check reviewed. Installer self-check: **11 pinned rows / 12 disabled ids**, manifest `e9364e577f40`, failures [].
- `mcp.row.yml` (not in the release list, not replaced): `6e4e7509…`, unchanged.

## Database / chain
- `pragma integrity_check` = ok (mode=ro).
- Events: **2256**, head `63de3fb33b5924a434d9b8650a41be55db033e6c99d3d45e2029cb5c15139d68`.
- **Chain verified with the installed package's own `verifyChain`** (`src/core/store.js`) on a consistent `.backup` copy, anchored to the
  journald-recorded head `seq 2198 / d9c9ba87…`: `{"ok":true,"count":2256,"head":{"seq":2256,"hash":"63de3fb3…"}}` — contiguous, links match,
  anchor present (catches truncation/rewrite).
- `policy` 2 rows (`roster.models`, `comms.grant`); `roster` 43 rows; `orders` 12 / `tasks` 30; **0 non-fixture non-terminal orders**.

## Runner surface / sockets / services
- Listeners: 8790 (127.0.0.1 + tailnet + `[::1]`), 3092, **8796 on 127.0.0.1 only (NEW — hosted-browser MCP; plain GET returns 405, auth
  required)**, 8791/9333/8799 as before. **`control.sock`, `egress.sock`, `mcp-admin.sock` are present again** in `/run/user/1000/fleet`
  (they had been unlinked since 13:21:05; the restart recreated them — anomaly fixed).
- `fleet-tmp-sweep.timer` enabled + active; `~/fleet-scratch/bin/tmp-sweep.sh` installed 15:07.
- OpenBell desk untouched: heartbeat `running / feed ws / feed_age 0.1 / paused False / pos None / trades 0 / day_pnl 0.00`.
- Bus: `bus.who` works (`ext:delegator` fresh, `lead` idle, participants cayleb-local/ext:delegator/nox); `bus.pull` empty; facade 15 tools,
  6 `bus.*`, `bus.status` routed (installer self-check lines).

## Findings / hygiene (no action taken by the Lead)
1. **8796 is live but still undocumented** — not in the release notes, not in the protected-ports lists (the CHECK-PKG MEDIUM M1 is now a live
   fact). Doc-only fix owed; consider a per-host `mcpPort` for scratch hosts.
2. **The DSH host prints its live UI token into its start log** (`$HOME/fleet/log/consolidated-<UTC>.out`, mode 0644; the same line is in
   journald). Pre-existing DSH behaviour, not caused by the install. The Lead did NOT copy the value anywhere. Recommend redaction/0600 or
   rotate-by-restart, owner/finish-lane decision.
2b. Neither finding affects service health.

## Limits
- No write, no restart, no control-socket call, no sudo, no GUI, no token/secret printed.
- The Lead's own post-install checker subagent failed before returning (no result); the checks above are scripted read-only verification, run
  directly against hashes/DB/chain — stronger than a text review for these items. The finish lane may run its own post-install health check.
