# PERSONAL DSH PROFILE — plugin inventory snapshot (Nox urgent ask 163900Z)

Snapshot taken 2026-11-01T16:33-16:36Z (11:33-11:36 ET) while Cayleb is actively uninstalling plugins.
Profile: apps/desktop/.desktop-build/development/home/profiles/desktop (the desktop app's personal profile).

## Files in this snapshot
- `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`, `cordis.yml`, `cordis.patch.yml` — the profile config at snapshot time (perishable).
- `inventory.tsv` — every installed package in the profile's node_modules: name, version, path, license (197 packages).
- `bundles.txt` — the profile's dsh.profile.bundles list at snapshot time (21 bundles).
- `plugin-manager-logs.txt` — plugin-manager operation log listing.
- `inner-browser-use/` — the BACKGROUND-BROWSER plugin identity: `dsh-inner-browser-use` 0.41.0 (package.json copy, dir listing, identity.json). It is STILL INSTALLED and ENABLED at snapshot time; this is the component Lane 4 must absorb into the single package.
- `memory-data-state.txt` — memory plugin data-store presence at snapshot time.

## Removed today (from the plugin-manager operation logs; timestamps ET)
- 11:30:31 `@furongjun1999/dsh-memory` 0.9.21 — THE CRASH CULPRIT (owner-confirmed; disabled by the Lead at 11:31, then uninstalled by the owner at 11:30:31 per the manager log).
- 11:30:37 `dsh-mattpocock-skills-deck` 1.9.0-beta.32
- 11:32:03 `dsh-flymemory` 1.8.3
- 11:32:13 `@lemoncat7/dsh-knowledge` 2.17.0
- 11:32:21 `@stardustlc/dsh-dream` 0.13.7
(So the memory component in the single package is an OPEN GAP: the replacement is being re-derived by Nox, not re-picked.)

## Still installed at snapshot time (13 deps)
@changfenhuang/dsh-genui 0.15.0 · @deepseek-ai/dsh-subagent-claude-code (link) · @deepseek-ai/dsh-subagent-codex (link) · @jayyuen66/dsh-scrapling 0.1.6 · @opencode-compat/dsh-bridge 0.4.4 · @yiyunet/dsh-kill-ai-slop 0.7.17 · cursor-opencode-provider 0.8.5 · dsh-claude-style 0.19.6 · dsh-cool-git 0.4.0 · dsh-free-search 0.9.26 · dsh-inner-browser-use 0.41.0 · dsh-plugin-upgrader 1.6.1 · dsh-univer-office 0.3.9
Plus the official base bundles listed in bundles.txt.

## MEMORY PLUGIN DATA — PRESERVED (untouched copy)
- Source dirs present at snapshot: `.../development/home/.dsh-memory` (544K) and `/home/cayleb/.mdcg` (20K).
- Preserved (copy, not moved) to `/home/cayleb/Work/_archive/lane6-preservation-20261101/memory-plugin-data-20261101T1633Z/` — 576K, 104 files, with SHA256SUMS.txt.
- Migration-vs-archive is Nox's replacement-plan decision; the data is untouched in place and copied for safety.
