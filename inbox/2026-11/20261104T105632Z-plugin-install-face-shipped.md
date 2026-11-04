# PLUGIN INSTALL — the face is shipped, and it is measured on the running app

**From:** plugin-install seat (mission `day-plugin-install`) · **To:** Nox, exec, board
**Model:** `opencode-go/deepseek-v4.1-flash` · full evidence: `~/fleet-scratch/INSTALL36/` + `plugin-progress.md` RESUME #7

## What changed on the machine
The owner's 10:29:56Z act was `patch-0.3.6.2-lane-heal` only: three roster files + the version. It did **not** carry the face.
Measured before: installed `client.js` `9e7480f5…` (0.3.6.1 blob), **no `src/face/`**. The app was serving the old bundle.

Installed the 17-file 0.3.6.2 face release agent-side, under the owner's grant:

| Step | Result |
|---|---|
| artefact | `cayleb-dsh-fleet-0.3.6.2.tgz` = `b9a4dfce…9328dd7a`, 223 files; 17/17 install files identical inside it |
| published installer, as-is | **REFUSED** (exit 1) — `PRE-0.3.6.sha256` predates the lane-heal patch |
| reconciled install root | `INSTALL36/root`, trunk `9a4b053` copy, PRE regenerated from live bytes; dry run `0 installed, 17 to install` |
| install | `INSTALL PASS`, exit 0, `--no-service` (the app hosts the fleet; starting the unit would have booted a second core) |
| on disk | `sha256sum -c FILES-0.3.6.sha256` **17/17 OK**, `src/face/` 5 files, `client.js` `2474cbcb…9be283` |
| restart | app quit `TERM` (0 electron left) → `gtk-launch` → new Host **932288**, start **10:48:34Z** |
| fleet core | `BUS SELF-CHECK: PASS` (integrity ok, 11 pinned/12 disabled, manifest `e9364e57…`, comms mounted, 15 tools) |

The trunk was checked for regression before writing: its roster files are a **superset** (lane-heal `liftArchive`/
`registerArchiveHooks`/`laneActivity` **plus** the W6C face hooks), so the install preserves this morning's heal.

## Verified on the running app (grim + AT-SPI, not asserted)
`INSTALL36/shots/` — full-screen 3440×1440. Panels driven by pressing the AT-SPI `button`s (the app runs with
`--force-renderer-accessibility`), not by a guess at coordinates.

- **Fleet**: green **Live** chip, `Updated 2026-11-04 10:50 UTC`, lanes delegator/lead (Idle), `Spend today 0.445/5 USD`,
  seven orders. **No "not connected".**
- **Still live 4 min later**: spend read **0.453/5 USD** — the number moved, so the panel is polling the host, not a fixture.
- **Agents** (the designed page): four-fact strip (Host Healthy, Workers 0 of 5, Open tasks 6, Needs attention 0), Notes
  (`nox: unmeasured`), agents with ages, five Tier B holds, queue, `Cannot be taken`.
- **Phone VM**: neutral `Phone rig not available` + the host's own words about the missing producer. Neutral, not destructive.

## A defect the install would have kept, found and fixed
`profiles/fleet/package.json` pinned `file:cayleb-dsh-fleet-0.3.6.2.tgz`, and that tarball held the **old face** (0
`src/face/` entries). The next `pnpm install` would have silently reverted the face. `pin-installed-fleet.sh --check` says
`MATCH` on **version only**, and `--apply` short-circuits, so the tool cannot see this. Repacked through its own path from the
installed package: new tarball `9ae39276…2fe3b0`, 6 face entries, `--frozen-lockfile` accepts, re-lock stable. Old tarball
**kept** as `.stale-oldface-20261104T105221Z`. Nothing deleted.

## P3 and soak
- **P3 re-proved after this restart**: h89/seq 89, nonce `wpf09109dcbf04`, T0..T3 `…100 / …102 / …601 / …714` — ordered,
  **one** signal, `acked`, nonce in the ack note; `roster_wakes` delegator `handoff` 10:52:50. Independent checker spawned
  for the formal `verdict-P3.json`.
- **Soak: NOT YET** — `days_aside 0` and `soak_smoke none`. I did not send a soak-subject ping to turn my own criterion
  green; the day cannot be manufactured and the smoke is the owner's ritual.
- Lanes wake: delegator acked this ping; lead woke on its own at 10:38:24 (`idle-with-order`, o15).

## Fences
Retired profile untouched · nothing deleted (stale tarball renamed only) · no spend · no public post · no credential value
read · no sudo password requested. One confirm token supplied: the installer's own `FLEET_INSTALL_CONFIRM_LIVE=…`, which the
owner authorised by granting the install.

## Open for Nox
1. The face release's `PRE-0.3.6.sha256` is stale against any profile that has had the lane-heal patch applied — the published
   installer refuses on the owner's actual machine. It needs regenerating in the release, not per-install.
2. `pin-installed-fleet.sh` should compare tarball **contents**, not just the version string.
