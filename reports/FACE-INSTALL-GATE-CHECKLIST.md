# FACE LIVE-INSTALL GATE CHECKLIST — pre-written so the window is fast and safe (Lead, 2 Nov 17:00 ET)
Scope: installing `0.3.5.1-face-prep` (tarball `7a125df2…ecf5`, manifest `d8226990…9e42`, overlay `74e763b0…`) into the **fleet profile**
(`:3092` surface; G-D1 = fleet profile, never the owner's personal profile). Owner: executing session gates it; Team B supplies the artifact;
Lead verifies; a fresh checker audits after. **No install without Nox's go.**

## 0. Preconditions (all must be recorded before touching the install)
1. Nox's explicit go for THIS artifact sha (no go for "a face install" in general).
2. **Installer self-check proves `validate.js` survives** (the 0.3.5.1 fix `b68ce88c…` remains byte-identical after install) + the
   must-keep hash list refuses any release that would overwrite it.
3. **Single-writer**: fleet host stopped or confirmed sole writer for the window; no lane running; no other host on the fleet profile.
4. Backup: current fleet profile tree + `fleet/state` snapshot; the installer's own backup path recorded (the 0.3.5 installer writes
   `fleet/backups/rel*`).
5. Sealed/owner surfaces: owner's desktop profile untouched (hash check before/after), no owner-app restart implied by this install (the app
   keeps running; the face appears on the next fleet-profile page load — confirm with Team B).
6. Rollback rehearsed: exact command + the backup path + the post-rollback health check.

## 1. Install (executing session runs; Lead records)
- Dry-run → real run; capture the installer log verbatim.
- Post-start self-check must include: control status, rows integrity, `fleet-face` row present **once**, facade `initialize`, tool count 15
  (nine core + six bus), `bus.status` routed.
- **Must-keep assertion**: `src/orders/validate.js` sha unchanged after install (`b68ce88c…`).

## 2. Post-install gates (fail-closed; any fail → rollback)
| # | gate | pass condition |
|---|---|---|
| H1 | host health | control status ok; integrity ok (chain/sandbox/rows/guard); lanes_allowed true; failures [] |
| H2 | rows | manifest `d8226990…9e42`; 10 pinned / 12 disabled; `fleet-face` present exactly once |
| H3 | validate.js | sha unchanged `b68ce88c…` (the 0.3.5.1 fix survived) |
| H4 | face data channel | `GET /fleet-face/snapshot` on the fleet host answers 200 with `live:true` (or an honest `live:false`+errors state — never a crash) |
| H5 | client bundle | the fleet profile's served client bundle loads and contains the face markers; `main['fleet']` + `sidebar.panellist` registered once |
| H6 | bus | `bus.status` ok; the six bus tools present; seeded participants unchanged |
| H7 | single writer | one host on the fleet profile; no duplicate fleet.db writer (`writers.py` inventory) |
| H8 | owner surface | desktop profile hashes unchanged; owner's app not restarted by the install |
| H9 | rollback | a byte/mode comparison of the backup proves a clean restore path (rehearsed dry-run) |

## 3. Rendering gate (the owner's grade domain)
- The face must render in the fleet profile's web surface (`:3092`); the owner grades **rendered visuals only** (no stills as evidence).
- What the Lead verifies (not image reads): the page's DOM/computed styles show the panels, the status pill, live/partial states, and the
  FIXTURE word only where fixtures are used; the agent-activity view lists lanes; the VM panels show honest "not wired" states.
- Any "live data not connected yet" state must be honest, not a fake value — no zero where the value is unknown.

## 4. What this install does NOT cover
- The relocation/app switch (Part B) — separate ruling/gates.
- The Duet bridge/proof — separate.
- Owner Elevation — owner sitting.
- Rendered acceptance is the owner's grade; the checklist only verifies the install and the honest data states.

— delegator (Lead), pre-written so the install window needs no new analysis; nothing here has been executed.
