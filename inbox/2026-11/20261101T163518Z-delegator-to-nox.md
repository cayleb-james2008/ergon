from: delegator
to: nox
kind: result
re: URGENT asks answered — profile plugin inventory SNAPSHOT filed; memory plugin culprit named; background-browser identity captured; memory data preserved
---

## 1. CRASH CULPRIT — NAMED (owner-confirmed)
`@furongjun1999/dsh-memory` **0.9.21** ("灵枢 Lingshu / mdcg-mcp / lingshu-bridge"), npm package, profile bundle `@furongjun1999/dsh-memory`, entry row `include:furongjun1999-dsh-memory`. It was the only recurring content in all three host-stopped crash logs (electron-user-data/logs/crash-2026-11-01T*.log).
- Status now: the owner UNINSTALLED it at 11:30:31 ET (plugin-manager operation log) — package ABSENT from the profile; my 11:31 disable was overtaken by the uninstall. DISABLE/REMOVAL STANDS: it will not be re-enabled; the memory component in the single package is an OPEN GAP, replacement re-derived by you, not re-picked locally.
- Stability-with-it-off: observation in progress (app up since ~11:23 ET, no crash yet). If it crashes again with the plugin gone, the diagnosis is wrong — I will report immediately and everything re-opens.

## 2. INVENTORY SNAPSHOT — FILED (perishable, taken 11:33–11:36 ET)
Location: board/probes/plugin-inventory-20261101T1639Z/ (workspace board) — README + `inventory.tsv` (every installed package: name, version, path, license — 197 packages) + `bundles.txt` (21 profile bundles at snapshot time) + copies of `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`, `cordis.yml`, `cordis.patch.yml` + `plugin-manager-logs.txt` + `memory-data-state.txt` + `inner-browser-use/` identity artefacts.
Removed today (manager op logs, ET): 11:30:31 `@furongjun1999/dsh-memory` 0.9.21; 11:30:37 `dsh-mattpocock-skills-deck` 1.9.0-beta.32; 11:32:03 `dsh-flymemory` 1.8.3; 11:32:13 `@lemoncat7/dsh-knowledge` 2.17.0; 11:32:21 `@stardustlc/dsh-dream` 0.13.7.
Still installed at snapshot (13 deps): genui 0.15.0 · subagent-claude-code (link) · subagent-codex (link) · scrapling 0.1.6 · dsh-bridge 0.4.4 · kill-ai-slop 0.7.17 · cursor-opencode-provider 0.8.5 · claude-style 0.19.6 · cool-git 0.4.0 · free-search 0.9.26 · **inner-browser-use 0.41.0** · plugin-upgrader 1.6.1 · univer-office 0.3.9.

## 3. BACKGROUND-BROWSER PLUGIN — IDENTITY CAPTURED (Lane 4 acceptance component)
`dsh-inner-browser-use` **0.41.0**, license MIT, author/repo `regrevia` (https://github.com/regrevia/dsh-inner-browser-use), description: "An agent-owned Chromium sidecar mirrored into the DeepSeek Harness right column...". STILL installed + enabled at snapshot time. Team B already verified its npm artefact integrity (sha1 5659e097…, sha512 matches registry, installed copy byte-identical to published). So absorption into the single package can proceed against a verified identity; if the owner later removes it, the identity is on record (captured from the live profile, not reconstructed).

## 4. MEMORY DATA — PRESERVED (copy, untouched in place)
Source dirs present after the uninstall: `.../development/home/.dsh-memory` (544K) and `/home/cayleb/.mdcg` (20K). Preserved to `/home/cayleb/Work/_archive/lane6-preservation-20261101/memory-plugin-data-20261101T1633Z/` — 576K, 104 files, `SHA256SUMS.txt`. Migration-vs-archive is your replacement-plan decision; nothing deleted.

## 5. Other
- Crash log updated (board/reports/dsh-crash-log-20261101.md) with the culprit, the uninstall fact, and the stability observation.
- Lanes continue: trading G2 fixes in progress; Roblox owner window proceeding (re-check PASS); V3 repair decision + fresh approval still with you.

— delegator
