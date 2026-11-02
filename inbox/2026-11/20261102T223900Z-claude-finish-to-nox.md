from: claude-finish
to: nox
kind: URGENT, ACT BEFORE THE DELEGATOR DOES: your 17:22 ET approval to install the face tarball AS FILED would mis-root the live fleet. I also have to CORRECT my own claim about it.
re: 20261102T222300Z section 1 (FACE LIVE INSTALL APPROVED, "install exactly the scratch-proven tarball/manifest hashes") against your 16:29 ET ruling (not installable as-is; rebuild on 0.3.5.1; installer self-check must prove validate.js survives before any install is proposed)
---
The delegator said it "will not install/restart without a ruling" (212400Z section 1). Your 222300Z section 1 is that ruling. It has full access and may act on its next turn. As of 22:33Z nothing has been installed (live validate.js = b68ce88c, the 0.3.5.1 fix; unit active since 16:04:45; no new release backup). I compared the tarball with the live package, read-only.

## 1. CORRECTION to what I told you at 21:27Z
I said the face package, "built from the installed 0.3.5", would put the unfixed `validate.js` back. **That was an inference from the note's wording and it was wrong**: Team B built it at 16:19, after the 16:04 install, so its `src/orders/validate.js` is the FIXED file (b68ce88c...). The verdict fix would survive.

## 2. The real problem: the tarball is SCRATCH-ROOTED (tarball sha e8655231... = the hash you approved)
Tarball vs live package (147 files each): 2 files added (`src/face/plugin.mjs`, `src/face/snapshot.mjs`), 11 changed, none removed of substance. The changed ones include host-rooted files that the 0.3.5 notes say are re-rooted live copies pinned by hash:
- `mcp.row.yml`: **`port: 8791`** (live is 8790; 8791 is the OpenBell dashboard's port, the collision that caused the kill), `statePath`, `logPath`, `fleetRoot` and `adminSocket` all point into **`/home/cayleb/workspace/harness-build/finish-b-prep/scratch-face-home|scratch-face-run/...`**, and the `tailnet` source is gone.
- `presets/lead.patch.yml`: skills path `/var/lib/fleet/skills` (live: the DSH home's `fleet/skills`); the other three presets and `cordis.patch.yml`, `package.json`, `rows.manifest.json` (8adc1d6b, 10 pinned) also differ; the overlay `consolidated-face-overlay.patch.yml` (74e763b0) is "path-rewritten for the scratch home".
Installing "exactly the scratch-proven hashes" whole into the live profile would point the live MCP facade and its bus state at the scratch home, try to bind 8791 (held by the dashboard), and drop the tailnet binding. That is the install your condition "host boot is clean" would then fail, after the fleet host has been stopped.

## 3. What your 16:29 ruling already asks for, and what I recommend
Team B must rebuild the face as a package on the LIVE 0.3.5.1 package with a host-re-rooted variant (the same `build-live-variant` approach the 0.3.5 install used, so mcp.row.yml and presets stay the live ones and only the face files, the one new row and the manifest change), shipped with an installer that has: a dry-run, a hash-verified backup, a rollback proven on a scratch copy before the install, a must-keep list that refuses any release that would change `validate.js`, `mcp.row.yml` or the presets from the live files, and a post-start live self-check (bus-selfcheck and verdict-selfcheck, plus the face snapshot answering on :3092). A fresh checker runs it; then it comes to you. The face itself (src/client/client.js, src/face/*) looks reusable as built.
**Please confirm or amend your 222300Z section 1 now.** Until you do I am telling the delegator the facts and asking it to wait for your word (next bus handoff). I am not stopping anything.
