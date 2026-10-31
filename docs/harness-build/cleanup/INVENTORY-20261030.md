# Harness inventory: outis-desktop (merged), 30 Oct 2026

Phase 1 of the harness cleanup ordered by Cayleb at 21:20 ET. This is the merged inventory from five read-only sweeps (packages, apps, services, dirs, dependents) plus a completeness re-check. Nothing was deleted, stopped, started or installed, and no sudo was used. Sweep evidence lives in this folder: `inventory-{packages,apps,services,dirs,dependents}.md`.

Rules this inventory follows (ASSUMPTIONS.md, 21:22 ET):
- **Removal means archive.** Data and config dirs are moved to `/home/cayleb/Work/_archive/harness-removal-20261030/<name>/`. Cayleb empties that folder himself.
- **User-level uninstalls** (mise, npm, uv) run directly.
- **Root steps** are listed as commands for Cayleb.
- **HOLD** applies to a harness in the DELETE class when a live service, running process or live code path depends on it, or when it is unclear whether the item is a harness at all. HOLD items need Cayleb's own yes; a yes relayed through someone else does not count.

## 1. Summary

| # | Harness / runtime | Class | Kind | Footprint | Running now? | Services / jobs | Live dependents |
|---|---|---|---|---|---|---|---|
| K1 | ChatGPT desktop / Codex (app + Codex CLI) | **KEEP** (Codex CLI flagged) | desktop app + CLI | ~73G | Codex Chrome extension host only | codex-desktop-control-ydotoold (disabled); 2 Codex automations (PAUSED) | linux_desktop/linux_background MCP |
| K2 | Claude Desktop + Claude Code CLI | **KEEP** (CLI flagged) | desktop app + CLI | ~20.6G | yes (4.4212.0 + Claude Code 2.1.309 sessions) | moneydesk-intake.timer, lane-sensor.timer (`claude -p`) | Money Desk, lane sensor |
| K3 | DeepSeek Harness (dsh) + fleet | **KEEP** (not installed system-wide yet) | agent runtime | ~1.1G runtimes | no | none (`/var/lib/fleet` absent) | none |
| G1 | OpenCode (CLI + desktop + always-on service) | **HOLD: named for DELETE, live-service gate** | CLI + Electron app + user service | ~7.6G | **yes**: PID 1625 `opencode-cli serve --service`, 127.0.0.1:49374 | opencode-service.service (enabled, Restart=always) | 4 active timers + Omarchy default agent (see §3) |
| D1 | Gemini CLI + Antigravity (agy) | **DELETE** | CLI (mise + npm) + Electron app + agy binary | ~1.37G | no | none | none (config references only) |
| D2 | OpenClaw (local install + remnants) | **DELETE** | npm-global gateway + data + Windows-VM dir | ~1.59G | no | none | none |
| D3 | Freebuff (Codebuff desktop; manicode) | **DELETE** | AppImage + Electron profile | ~0.58G | no | none | hardening-verify expected-state (config only) |
| D4 | Grok CLI (xAI `grok` 1.0.50) | **DELETE** | agent CLI | 248M | no (idle since 10-19) | .bashrc:24-27 | none (verified) |
| D5 | OpenMausBot 0.1.102 | **DELETE** | multi-agent AppImage | 265M | no | none | none |
| D6 | OpenWork (com.differentai.openwork) remnant | **DELETE** | Electron config remnant | 2.6M | no | none | none |
| D7 | oh-my-pi (`omp`) + mise agent-cache remnants | **DELETE** | wrapper + caches | ~0.1M | no | none | none |
| H1 | Grok Bot desktop app (xAI, v0.78.1) | **HOLD** | agent AppImage + local-exec daemon | ~575M | **yes**: PIDs 11930/11998 + children | app-grok-bot-*.scope (transient) | Cayleb's line to the live Grok trader (never-touch rule) |
| H2 | Paperclip / Pantheon (Zeus control plane + lanes) | **HOLD** | agent orchestration platform (retired) | ~22.1G | no | 7 units + 6 timers, all disabled/static | 13 Claude skill symlinks, local-only workspace repos, forge-local-worker soft check |
| H3 | Outis pilot support plane (outis-control + outis-mesh) | **HOLD** | Ergon/OpenClaw support service + MeshCentral | 1.3G | **yes**: PIDs 1488, 1489/1764/1839 (127.0.0.1:8787/8788) | outis-control.service, outis-mesh.service (enabled) | enrolled remote device `laptop-pilot` |
| H4 | Prime Agent remnant | **HOLD** (unclear whether a harness, per ASSUMPTIONS) | logs + setup script | ~96K | no | none | none |
| H5 | browser-use 0.13.10 (uv tool) | **HOLD** (unclear whether a harness) | LLM browser-agent framework CLI | 219M | no | none | none |
| H6 | Meta Muse CLI cached tarball (muse-code 1.3.0) | **HOLD** (Nox is a Muse agent) | cached download | 300M | no | none | unknown (Nox) |
| H7 | herdr 0.9.1 (pacman) | **HOLD** (unclear whether a harness; root needed) | terminal workspace manager for agent CLIs | 20.7MiB + 4K | no | none | none |
| H8 | Talos runtime orphans | **HOLD** (unclear whether a harness; next to a wallet) | orphan user units + project dirs | ~0.2M | no | talos-dashboard/facilitator (disabled) | none |
| H9 | `~/.winvm-scratch` | **HOLD** (attribution unproven) | Windows-VM scratch | 345M | no | none | `.winvm-openclaw/win.iso` symlink points here |
| H10 | voice-agent (Jarvis) template | **HOLD** (tiny, not installed) | config template | 8K | no | none | none |
| N* | Model servers, agent tools, Hermes bot, Outis OS popup and others (§6) | **NOT-A-HARNESS** | n/a | n/a | some | see §6 | n/a |

### Totals (du -sh, rounded)

| Class | Size | Notes |
|---|---|---|
| DELETE, ungated (D1–D7) | **≈ 4.05G** | All user-level, no root needed |
| HOLD on live gate: OpenCode (G1) | **≈ 7.63G** | Named in Cayleb's DELETE order |
| DELETE set total, Cayleb's list + class finds | **≈ 11.7G** | Of this, ~2.6G is reinstallable binaries removed by mise/npm uninstall (mise opencode 1.6G, mise gemini 0.3G, npm gemini-cli 0.1G, npm openclaw 0.6G); ~9.1G moves to the archive |
| HOLD (H1–H10) | **≈ 24.9G** | Paperclip ~22.1G, outis-pilot 1.3G, Grok Bot ~0.58G, the rest small |
| KEEP | ≈ 95G | Codex ~73G, Claude ~20.6G, DSH runtimes ~1.1G + harness-build 183M |

Root steps needed for the DELETE set: **none**. The only root-only item is herdr (HOLD).

## 2. DELETE rows: removal plans

Shared setup for phase 2. This is a plan only; nothing has run.

```bash
A=/home/cayleb/Work/_archive/harness-removal-20261030
mkdir -p "$A" && chmod 700 "$A"     # the archive will hold credentials (auth.json, oauth_creds.json, tokens)
# arch <name> <path>...  : move each path to $A/<name>/<path relative to $HOME>, never overwriting
arch(){ n=$1; shift; for p in "$@"; do [ -e "$p" ] || [ -L "$p" ] || { echo "MISSING $p"; continue; }; r=${p#/home/cayleb/}; mkdir -p "$A/$n/$(dirname "$r")" && mv -n "$p" "$A/$n/$r"; done; }
cp -a ~/.bashrc ~/.bashrc.bak-20261030; cp -a ~/.config/mise/config.toml ~/.config/mise/config.toml.bak-20261030
cp -a ~/.config/omarchy/plugins/cayleb.agents/manifest.json{,.bak-20261030}; cp -a ~/.config/omarchy/extensions/omarchy-menu.jsonc{,.bak-20261030}
```

The archive is on the same btrfs subvolume as `$HOME` (`/dev/mapper/root[/@home]`, device 30), so each `mv` is an instant rename. There is 608G free. `~/Work` is not a git repo, so archived credentials cannot be committed by accident.

### D1. Gemini CLI + Antigravity (agy): DELETE, ≈1.37G
Named in the order. Not running, no services. `~/.gemini` is shared by Gemini CLI and Antigravity, so they are handled as one row. Never run `antigravity --version`: it opens the GUI.

| Path | Size | What |
|---|---|---|
| `~/.local/share/mise/installs/gemini` | 302M | 0.61.0, 0.63.0, 0.64.0, 0.65.0 (+ shim `~/.local/share/mise/shims/gemini`) |
| `~/.local/share/mise/installs/node/26.7.0/lib/node_modules/@google/gemini-cli` | 100M | npm global 0.63.0 + `node/26.7.0/bin/gemini` |
| `~/.local/bin/agy` | 212M | agy CLI v2.24 |
| `~/.local/bin/agy.1792436412526340068.old` | 200M | stale self-update copy |
| `~/.local/bin/antigravity` | link | → `~/.local/opt/Antigravity/antigravity` |
| `~/.local/opt/Antigravity` | 504M | desktop app |
| `~/.config/Antigravity` | 12M | Electron profile |
| `~/.gemini` | 37M | GEMINI.md, settings, **oauth_creds.json, google_accounts.json**, history, skills (12 links), antigravity/, antigravity-cli/ |
| `~/.local/share/applications/antigravity.desktop` | tiny | menu entry |
| `~/.local/share/icons/hicolor/512x512/apps/antigravity.png` | tiny | icon |
| `~/.local/bin/omarchy-agent-usage-antigravity` | 12K | Omarchy bar usage collector |
| `~/.cache/mise/gemini` | 56K | mise metadata |

- **(a) User-level uninstall:**
  `mise uninstall --all gemini`; delete the `gemini = {...}` line from `~/.config/mise/config.toml` (without this, `mise install` reinstalls it); `mise exec node@26.7.0 -- npm uninstall -g @google/gemini-cli`; `mise reshim`. **Never remove node 26.7.0**: it also runs the KEEP MCP servers (linux_desktop, chrome-devtools) and outis-mesh.
- **(b) Archive:**
  `arch gemini-antigravity ~/.local/bin/agy ~/.local/bin/agy.1792436412526340068.old ~/.local/bin/antigravity ~/.local/opt/Antigravity ~/.config/Antigravity ~/.gemini ~/.local/share/applications/antigravity.desktop ~/.local/share/icons/hicolor/512x512/apps/antigravity.png ~/.local/bin/omarchy-agent-usage-antigravity ~/.cache/mise/gemini`
- **(c) Services / autostart:** none. Config edits instead:
  - Remove the `antigravity` collector entry in `~/.config/omarchy/plugins/cayleb.agents/manifest.json` (line ~27).
  - Remove `"antigravity"` from the aliases and "Antigravity" from the description in `~/.config/omarchy/extensions/omarchy-menu.jsonc:34-35`.
  - The monitor script `omarchy-agent-usage-monitor` keeps harmless antigravity branches.
  - Then run `update-desktop-database ~/.local/share/applications`.
  - **Leave** `.bashrc:16-17` and `.bash_profile:7-8` ("Added by Antigravity CLI installer"). Those lines put `~/.local/bin` on PATH, which the KEEP launchers (`claude`, `chatgpt`, `claude-desktop`) need. At most reword the comment.
- **(d) Root:** none.

### D2. OpenClaw (local install + remnants): DELETE, ≈1.59G
Named in the order, remnants included. Not running, no units. The board says "OpenClaw is OUT (removed 29 Oct)", and Ergon moved to DSH.

| Path | Size | What |
|---|---|---|
| `~/.local/share/mise/installs/node/26.7.0/lib/node_modules/openclaw` | 600M | openclaw@2026.10.1 + `node/26.7.0/bin/openclaw` + shim `mise/shims/openclaw` |
| `~/.openclaw` | 592M | config, 5 agents, tmp 457M, skills (12 links); **openclaw.json holds gateway.auth.token + an OpenAI auth profile** |
| `~/.cache/openclaw` | 0 | empty |
| `~/.winvm-openclaw` | 399M | QEMU Windows VM for OpenClaw; **local-login.txt (credential)**; `win.iso` symlink → `~/.winvm-scratch` (H9) |

- **(a)** `mise exec node@26.7.0 -- npm uninstall -g openclaw && mise reshim`
- **(b)** `arch openclaw ~/.openclaw ~/.cache/openclaw ~/.winvm-openclaw`
- **(c)** none.
- **(d)** none.
- **Out of scope, do not touch** (Cayleb's project files, his call): `~/Work/openclaw-personalized-agent-baseline` (2.7G repo), `~/Work/Ergon-Windows-Test-1.5.1` (~324M installer bundle). The running Outis pilot support plane is the Ergon product's OpenClaw support service and is held separately as H3.

### D3. Freebuff (Codebuff desktop): DELETE, ≈575M
Named in the order. Not running. The launcher is already broken: it points to the 0.0.179 AppImage, but only 0.0.188 is on disk (verified).

| Path | Size | What |
|---|---|---|
| `~/Applications/freebuff` | 183M | Freebuff-0.0.188 AppImage + png |
| `~/.config/Freebuff` | 350M | Electron profile (session cookies) |
| `~/.config/freebuff-desktop` | 42M | state, projects, **device key** |
| `~/.freebuff` | 48K | mcp.json |
| `~/.config/manicode` | 216K | Codebuff CLI legacy config |
| `~/.cache/@codebufffreebuff-desktop-updater` | 4K | updater |
| `~/.local/bin/freebuff-desktop` | 172B | broken launcher |
| `~/.local/share/applications/freebuff.desktop` | tiny | menu entry |
| `~/.knowledge.md` | 1.6K | Freebuff/Codebuff instruction file (written by harness-squared install_defaults) |
| `~/.agents/mcp.json.before-freebuff-20261017` | 291B | install backup. **Only this file**; the rest of `~/.agents` is shared and stays |

- **(a)** none (AppImage install).
- **(b)** `arch freebuff ~/Applications/freebuff ~/.config/Freebuff ~/.config/freebuff-desktop ~/.freebuff ~/.config/manicode ~/.cache/@codebufffreebuff-desktop-updater ~/.local/bin/freebuff-desktop ~/.local/share/applications/freebuff.desktop ~/.knowledge.md ~/.agents/mcp.json.before-freebuff-20261017`, then `update-desktop-database ~/.local/share/applications`
- **(c)** No units. Config:
  - `~/Work/security/verify/expected-state.json` lists `~/.config/freebuff-desktop` modes, so hardening-verify will flag them missing. Drop those lines (that is the security lane's file).
  - The harness-squared `install_defaults.py` target `.knowledge.md` would recreate the file on a re-run. See §7, item 6.
- **(d)** none.

### D4. Grok CLI (xAI `grok` 1.0.50): DELETE, 248M
In the "anything else in that class" set. **This is separate from the Grok Bot app (H1).**
- The Grok Bot daemon has no reference to `~/.grok`.
- No unit or script calls `~/.grok/bin`, `~/.local/bin/grok` or `~/.local/bin/agent`. The only reference is the PATH line of the retired `paperclip-pantheon.service` (re-checked).
- The never-touch memory rule covers the Grok VM bot and its wallet, not this CLI.
- The apps sweep proposed HOLD. Resolved DELETE because nothing calls it, and the archive keeps it reversible.

| Path | Size | What |
|---|---|---|
| `~/.grok` | 248M | binary, **auth.json**, sessions, worktrees.db, plugins, completions |
| `~/.local/bin/grok` | link | → `~/.grok/bin/grok` |
| `~/.local/bin/agent` | link | → `~/.grok/bin/agent` |

- **(a)** none.
- **(b)** `arch grok-cli ~/.grok ~/.local/bin/grok ~/.local/bin/agent`
- **(c)** Delete the `.bashrc:24-27` block (`>>> grok installer >>>`: PATH plus completions).
- **(d)** none.

### D5. OpenMausBot 0.1.102: DELETE, 265M
A multi-agent team harness (19 bots; leads on Codex, Grok, Claude, Antigravity and OpenCode). Not running, no launcher, no unit. The dirs sweep proposed HOLD because it was driven from Codex task folders. Resolved DELETE: it is a separate harness, not part of the ChatGPT/Codex app, and the Codex task folders that drove it stay untouched.
- **(b)** `arch openmausbot ~/.local/opt/openmausbot-0.1.102`
- **(a/c/d)** none.
- **Do not touch:** `~/Documents/Codex/2026-10-08/task-5/openmausbot-migration`, `~/Documents/Codex/2026-10-09/task-6`, `~/Work/session-artifacts/openmaus-team-completion-20261013`.
- `~/.cua-driver` (50M, NOT-A-HARNESS) was used only by the OpenMaus and Grok Bot VMs. It becomes an orphan; that is optional cleanup for Cayleb.

### D6. OpenWork remnant: DELETE, 2.6M
An OpenCode-based agent desktop app. The binary is gone; only config is left (last used 09-15).
- **(b)** `arch openwork ~/.config/com.differentai.openwork ~/.config/@openwork` (the first holds **openwork-server-tokens.json**)

### D7. oh-my-pi + mise agent remnants: DELETE, ~0.1M
**Do not run `omp`.** It runs `mise use -g github:can1357/oh-my-pi`, which would reinstall the tool.
- **(b)** `arch omp ~/.local/bin/omp ~/.local/bin/omp.bak-20260918-192028 ~/.omp ~/.cache/mise/pi ~/.cache/mise/github-can1357-oh-my-pi ~/.cache/mise/cursor-agent`
- **Leave** `~/.local/share/mise/installs/.mise-installs.toml`. It is mise-managed and its stale entries (copilot, crush, oh-my-pi, pi, npm:@xai-official/grok, http:muse) install nothing.
- **Leave** the `pi-ai` dependency inside the DSH runtime (KEEP).

## 3. G1. OpenCode: HOLD on live-service gate (named for DELETE), ≈7.63G

Cayleb named OpenCode in the DELETE set. It is held only because a live service and active timers depend on it (ASSUMPTIONS 21:22). Stopping a live service needs Cayleb's own yes. The Money Desk lane owns the dependent scripts in `~/Work/shared-workflows/pumpfun-drop` and `pumpfun-automation`, so it must drop their OpenCode checks in the same change. The plan is fully written and can run as soon as both happen.

**Running:** PID 1625 `~/.config/ai.opencode.desktop/cli/2.2.0-x64/opencode-cli serve --service`, listening on 127.0.0.1:49374, up since 10-28 15:40. KillMode=process. The OpenCode agents were retired 10-25, and the database has been idle since 10-28 15:40.

**Live dependents** (sweeps, re-checked):

| Dependent | Effect when OpenCode is gone |
|---|---|
| `outisz-watchdog.timer` (1 min) → `pumpfun-drop/ops/outisz_watchdog.py` | pgreps opencode-cli, reads opencode.db and service.json, sends an "OpenCode service down" alert |
| `outisz-freshness.timer` (5 min) → `freshness.py check_opencode_service` | exits 1 every run |
| `outisz-state-maintenance.timer` (1 min) → `state_build.py` + `owner_brief.py` (calls OpenCode over HTTP via `oc_runtime.py`) | owner brief reports "session service down" |
| `hardening-verify.timer` → `Work/security/verify/expected-state.json` (opencode.db paths) | flags missing paths; already failing on the gone opencode-mobile.service |
| `outisz-alpha-worker.service` (disabled) | After=opencode-service, hard-codes `~/.local/bin/opencode` |
| Not scheduled: `model_gateway.py`, `gateway_bakeoff.py`, scoreboard `collect_opencode`, `orient_build.py` | would fail if run |
| `~/.config/omarchy/defaults/agent` = `opencode` | `omarchy agent` would launch a missing tool |
| Omarchy usage collector (D1) | reads `~/.config/opencode/antigravity-accounts.json` |
| `~/.codex/rules/default.rules:5-17` | stale allow-rules for opencode-mobile.service (KEEP-side file; cleanup optional) |
| harness-squared `install_defaults.py` target `.config/opencode/harness.md` | a re-run would crash (§7, item 6) |

| Path | Size | What |
|---|---|---|
| `~/.local/share/opencode` | 3.6G | opencode.db 3.39G (USG/Outisz history), snapshot 352M, **auth.json**, `pump-lettering/` 7.8M JPEG work assets |
| `~/.local/share/mise/installs/opencode` | 1.6G | 1.18.35, 1.19.0, 2.0.32, 2.1.4 (+ shims opencode, opencode2) |
| `~/.config/ai.opencode.desktop` | 1.1G | desktop profile + the running `cli/2.2.0-x64` |
| `~/Applications/opencode` | 946M | OpenCode-v2 AppImage + 3 backups (v1.19.3, original-20261017, persistent-browser) |
| `~/.opencode` | 203M | standalone CLI (`~/.local/bin/opencode` points here) |
| `~/.config/opencode` | 99M | config, harness.md, browser-groups, **antigravity-accounts.json\***, skills (19 links + 4 unique: desktop-loop, visual-signoff, next-options, hf-asset-upload) |
| `~/.cache/opencode` | 43M | cache |
| `~/.cache/oc-tmp` / `~/.cache/oc-scratch` | 28M / 0 | scratch |
| `~/.local/share/opentui` | 7.2M | TUI tree-sitter cache |
| `~/.local/state/cortexkit` | 3.0M | antigravity-auth plugin state |
| `~/.local/state/opencode` | 24K | service.json (**server password**) |
| `~/.cache/@opencodedesktop-updater` | 4K | updater |
| `~/.config/opencode-service.env` + `.bak-20261022` | 12K | unit env (**NVIDIA_NIM_API_KEY**) |
| `~/.config/systemd/user/opencode-service.service` + `default.target.wants/` link + `.pump-maintenance-backup-20261024T210312Z/opencode-service.service` | small | unit, enable link, old backup copy |
| `~/.local/bin/opencode`, `opencode-desktop`, `apotheka-opencode` | small | launchers |
| `~/.local/share/applications/opencode.desktop`, `apotheka-opencode.desktop` | small | menu entries |

Removal plan, to run only after the gate opens:
1. **Gate:** Money Desk drops or retires the OpenCode checks in `outisz_watchdog.py`, `freshness.py`, `state_build.py`, `owner_brief.py` and `expected-state.json`, then Cayleb gives his own yes.
2. **(c) Service:**
   - `systemctl --user disable --now opencode-service.service`
   - `pgrep -af opencode-cli` must come back empty. Because of KillMode=process, any survivors are reported as named residuals and are not force-killed.
   - `arch opencode ~/.config/systemd/user/opencode-service.service ~/.config/systemd/user/.pump-maintenance-backup-20261024T210312Z/opencode-service.service`
   - The `default.target.wants` link is removed by `disable`. If it is still there, archive it.
   - `systemctl --user daemon-reload && systemctl --user reset-failed`
3. **(a) Uninstall:** `mise uninstall --all opencode`; delete the `opencode = "latest"` line from `~/.config/mise/config.toml`; `mise reshim`.
4. **(b) Archive:** `arch opencode ~/.local/share/opencode ~/.config/ai.opencode.desktop ~/Applications/opencode ~/.opencode ~/.config/opencode ~/.cache/opencode ~/.cache/oc-tmp ~/.cache/oc-scratch ~/.local/share/opentui ~/.local/state/cortexkit ~/.local/state/opencode ~/.cache/@opencodedesktop-updater ~/.config/opencode-service.env ~/.config/opencode-service.env.bak-20261022 ~/.local/bin/opencode ~/.local/bin/opencode-desktop ~/.local/bin/apotheka-opencode ~/.local/share/applications/opencode.desktop ~/.local/share/applications/apotheka-opencode.desktop`, then `update-desktop-database ~/.local/share/applications`
5. **Config edits:**
   - Delete `.bashrc:29-30` (`# opencode` / PATH `~/.opencode/bin`).
   - Set `~/.config/omarchy/defaults/agent` to `claude`.
   - `outisz-alpha-worker.service` belongs to Money Desk; they retire it.
6. **(d) Root:** none.
7. **Leave** (Cayleb's or Money Desk's files): `~/Documents/ChatGPT/Apotheka Videos/opencode.json`, the pumpfun-automation `opencode.json`, `~/Work/shared-workflows/pumpfun-drop/ops/oc_runtime.py`.

## 4. HOLD rows: what Cayleb decides

| # | Item | Paths (size) | Why held | Decision needed / plan on yes |
|---|---|---|---|---|
| H1 | Grok Bot desktop app | `~/.local/opt/grok-bot` (530M: current + previous AppImage + .sand-update dirs), `~/.config/Grok Bot` (44M; sand-secrets.json, gateway-descriptor.json), `~/.grokbot` (284K; local-exec daemon credential), `~/.local/share/grok-bot/appimage` (link), `~/.local/share/applications/grok-bot.desktop`, `~/.local/share/icons/hicolor/512x512/apps/grok-bot.png` | Running (PIDs 11930/11998, signed in). It is Cayleb's line to the Grok team that runs the live rsi copy-trader, which a memory rule says never to touch. Not named in the order. | Keep or remove. On yes: **Cayleb quits the app himself**, then `arch grok-bot <paths>`. |
| H2 | Paperclip / Pantheon | `~/.local/share/paperclip-pantheon` (16G: workspaces 8.2G, private 2.6G, runtime 2.3G, _retired 1.4G, instance 1.2G, state 215M); `~/.paperclip` (3.1G); `~/.local/share/paperclip-studio-lane` (3.0G); `~/.config/paperclip-pantheon` (24K; credentials.env, RETIRED); units paperclip-pantheon (+.d), art-lane, studio-lane, muse-render, muse-review, yt-uploader, roblox-hub (.service/.timer + retired.conf drop-ins); launchers `pantheon-ctl`, `studio-lane`, `yt-uploader`; `paperclip-pantheon.desktop` | Retired 10-15/10-18 (RETIRED marker, all units disabled). Still holds **local-only git repos with no origin**: asteria-roblox 7.3G, whose games/street has unfetched USG branches visual-pilot, usg-city-kit, cars-pbr and PAN-175; also helikon-card-game, muse-os, project-olympus and zeus-hq. **13 Claude Code skills symlink into it** (verified). `~/.claude/settings.json:4-24` has its permission rules. forge-local-worker.timer soft-checks the muse-os GPU-HOLD file (missing means no hold; does not break). | Archive or delete. First rescue the workspace repos (fetch the street branches into `~/Work/projects/untitled-street-game`), then copy or drop the 13 skill links. On yes: archive the units, then `arch paperclip <tree>`. **Never touch** `~/Work/projects/pantheon` or `pantheon.desktop` (they point at his project). |
| H3 | Outis pilot support plane | `~/.local/share/outis-pilot` (1.3G: releases, control-config.json tokens, provider.env, anthropic-proxy.json, connector-jobs.sqlite, MeshCentral data/backups); `~/.config/systemd/user/outis-{control,mesh}.service` + wants links | Running. The release README says it is the Linux support service for Ergon's personalized OpenClaw assistant: Anthropic messages proxy, shared budget, memory API, MeshCentral remote management of the one enrolled Windows device `laptop-pilot`. The sweeps split: DELETE as an OpenClaw remnant vs. KEEP side because "Outis" is also Cayleb's ChatGPT agent name. Removing it cuts that laptop off. | Keep serving laptop-pilot, or offboard it and then remove. On yes: `systemctl --user disable --now outis-control.service outis-mesh.service`, archive the units, daemon-reload, `arch outis-pilot ~/.local/share/outis-pilot`. |
| H4 | Prime Agent remnant | `~/.prime` (92K logs, last 10-07), `~/.local/bin/setup-logfare-provider` (4.3K; writes Prime models.json) | ASSUMPTIONS names it ambiguous. No binary, no dependents. | On yes: `arch prime ~/.prime ~/.local/bin/setup-logfare-provider`. Leave the `.bashrc:19-22` NVIDIA NIM block; other tools use the env var. |
| H5 | browser-use | `~/.local/share/uv/tools/browser-use` (219M; bins browser-use, bu, browser-harness, browser-harness-mcp …) | An agent framework with its own CLI. Not referenced in any KEEP MCP config. | On yes: `uv tool uninstall browser-use`. |
| H6 | Meta Muse CLI tarball | `~/.local/share/mise/http-tarballs/a86cc70d…` (300M, muse-bin 1.3.0-R3401.1), `~/.cache/mise/http-muse` (12K) | Nox is a Meta Muse agent. Confirm Nox does not need a local Muse CLI. | On yes: `arch muse-cli <both paths>`. |
| H7 | herdr | `/usr/bin/herdr` (pacman 0.9.1-2, explicit, nothing requires it), `~/.config/herdr` (4K) | "Terminal workspace manager for AI coding agents". It hosts agent CLIs but runs no LLM loop itself. | Keep, or **Cayleb runs** `sudo pacman -R herdr`, then `arch herdr ~/.config/herdr`. |
| H8 | Talos runtime orphans | `~/.config/systemd/user/talos-runtime.service.d`, `talos-dashboard.service` (+.d), `talos-facilitator.service` (point at the missing `~/Work/projects/talos-os`); `~/.talos` (208K), `~/.config/talos/ollama.env`, `~/.config/talos-os/payment.env` | The drop-ins look like an autonomous agent runtime, but this is dead project data next to a wallet. **Never touch `~/.talos-wallet`.** | On yes: archive the orphan unit files only, then daemon-reload. |
| H9 | `~/.winvm-scratch` | 345M (09-18 Windows-VM OCR screenshots, answer/apps ISOs, win.iso) | Predates the OpenClaw VM. `.winvm-openclaw/win.iso` links here, but it is not proven OpenClaw-only. | On yes: `arch openclaw ~/.winvm-scratch`. |
| H10 | voice-agent template | `~/.config/voice-agent` (8K: config.yaml.example, service template) | Agent template, never installed. | On yes: `arch voice-agent ~/.config/voice-agent`. |

## 5. KEEP rows (untouched)

| # | Item | Paths (size) | State |
|---|---|---|---|
| K1 | ChatGPT desktop / Codex | pacman `openai-codex-desktop` 26.1021.22027-1 (`/usr/lib/chatgpt` 1662MiB, `/usr/bin/chatgpt`, `/usr/bin/codex-desktop`); `~/.local/share/chatgpt-official` 3.2G (26.1013.40934 active + 26.930.31730); `~/.local/bin/chatgpt`; **Codex CLI (FLAG)** `~/.local/share/mise/installs/codex` 2.7G (0.153.4–0.164.0); `~/.codex` 39G; `~/.config/Codex` 276M; `~/.cache/Codex` 1.8G; `~/.cache/codex-runtimes` 2.1G; `~/.local/state/codex` 594M; `~/.local/share/codex-background-control` 12G + `-next` 9.9G (linux_desktop/linux_background MCP) | App not running (stale SingletonLock, dead PID 5530). The Codex Chrome extension host runs in Chromium. Two automations PAUSED. Old version pruning is possible later and out of scope. |
| K2 | Claude Desktop + Claude Code | `~/.local/opt/claude-desktop` 4.1G (7 versions; 4.4212.0 active); `~/.config/Claude` 2.4G (bundled claude-code 2.1.309); **Claude Code CLI (FLAG)** `~/.local/share/mise/installs/claude` 2.1G (2.1.261–2.1.310) + `~/.local/bin/claude` wrapper; `~/.claude` 12G; `~/.cache/claude-cli-nodejs` 15M; `~/.local/state/claude-mods` 620K | Running. The CLI is called headless by moneydesk-intake.timer and lane-sensor.timer. |
| K3 | DeepSeek Harness (dsh) + fleet | `/var/lib/fleet` **absent**; `dsh` not on PATH; no dsh/fleet units or processes. Stage 0 owner kit is committed in `~/workspace/harness-build/dsh-fleet` (HEAD `6300623` "stage 0: apply re-check fixes … kit hash 1014c5daf1fe") but **not yet run** (needs Cayleb's root). Runtimes: `~/Documents/Codex/2026-10-30/task-2/dsh-cli-runtime` 541M (`@deepseek-ai/dsh` 0.2.2-alpha.2, includes pi-ai), `dsh-runtime` 227M, `dsh-npm-cache` 256M, `dsh-speech-runtime` 73M, `dsh-tools` 23M, `dsh-pnpm`, `dsh-sandbox`; `task-7/dsh-browser-evidence-*` (10 dirs); `task-5/dsh-valid-replace.mjs`; empty leftover `node/26.7.0/lib/node_modules/@deepseek-ai` (0B; leave for the fleet build) | Install state comes from the plugin build's Stage 0, not this cleanup. |

**Shared, never remove:**
- mise **node 26.7.0** (hosts the D1/D2 npm globals and the KEEP MCP runtimes plus outis-mesh; uninstall packages only).
- `~/.agents` (8.1M shared skill library; only the Freebuff backup file moves).
- `~/AGENTS.md` (read by DSH and by Codex run from `$HOME`).
- The `~/.local/bin` PATH lines in `.bashrc`/`.bash_profile`.

## 6. NOT-A-HARNESS (out of scope, listed so the after-state is not mistaken)

- **Model servers.** None runs an agent loop.
  - `llama.cpp-gfx1151` (pacman, `/usr/bin/llama-server`)
  - `~/bin/ollama` (28M)
  - `~/.local/opt/mimo` (116M)
  - units mimo-server, coding-orca, coding-qwen, coding-pair.target, kairos-laya (all inactive)
- **Agent tools and libraries.**
  - agent-browser 0.35.1 (npm 88M) + `~/.agent-browser`
  - opencli 1.8.7 (`@jackwener`, 31M)
  - `~/.local/share/agentreach-tool` (66M, mcporter) + `~/.agent-reach` (21M)
  - `~/.cua-driver` (50M; orphaned once D5 and H1 go)
  - `~/.config/browser-harness` (680K)
  - `~/.local/bin/tiny-agents` (huggingface_hub entry point)
  - `@higgsfield/cli`, vercel (npm)
  - MCP servers linux_desktop, linux_background, rea, openbb, chrome-bridge.service (chrome-devtools-mcp)
- **Hermes** = `outisz-community-status.service`, The Lair Telegram publisher bot (active). This is a name collision; there is no Hermes harness.
- **Outis OS popup.**
  - `~/.local/bin/outis-os` is a 9.5M Tauri ELF built locally from his own cargo tree. Data: `~/.local/share/com.outis.os` 100K, `~/.local/share/outis` 312K.
  - Hyprland window rules (`~/.config/hypr/outis.lua`) and SUPER+SHIFT+J (`outis-summon.sh` POSTs to 127.0.0.1:1424).
  - Not running, and nothing listens on :1424. It is a webview shell with no LLM API strings. Flagged only.
- **Wallet-adjacent: never touch.** `~/.gmgn-agent-runner` (README slot), `~/.talos-wallet`.
- **Unidentified Electron.** `~/.config/Electron` (1.9M) and `~/.cache/electron` (328M, build cache). Optional cleanup.
- **Ergon remote git.** The ergon remote branch `freebuff/18a6bc7d…` and PR #5 (OpenCode 2.0 port) are remote and unaffected.

## 7. Completeness re-check (gaps verified by the merger, read-only)

1. **Shell rc.**
   - `.bashrc:24-27` (grok) and `:29-30` (opencode) are in the plans above.
   - **Newly found:** `.bashrc:16-17` and `.bash_profile:7-8` "Added by Antigravity CLI installer" add `~/.local/bin` to PATH. **Keep them**, because the KEEP launchers live there.
   - `.bashrc:19-22` NVIDIA NIM block (comment mentions Prime Agent): keep.
   - No zsh, fish, environment.d or uwsm entries.
2. **mise.**
   - The global config pins `gemini` and `opencode`. Both lines must go, or `mise install` reinstalls them.
   - Shims `gemini`, `openclaw`, `opencode` and `opencode2` exist, so run `mise reshim` after the uninstalls.
   - node 20.20.2 has no harness globals.
3. **Other package managers.**
   - pipx: httpie, pip-audit, semgrep, sslyze, twitter-cli (none are harnesses).
   - uv: browser-use (H5), pycdlib, vncdotool.
   - cargo: cargo-xwin, rsign2.
   - flatpak: Blender, Boxes, Sober, Vinegar, virt-manager.
   - No snap, no pnpm or npm user-global dirs. `/usr/lib/node_modules` holds only npm tooling.
4. **pacman.** The only agent-related packages are `openai-codex-desktop` (KEEP), `herdr` (H7) and `llama.cpp-gfx1151` (not a harness). No system units for any harness.
5. **KEEP MCP configs.** `~/.claude.json`, `~/.claude/settings.json`, `~/.config/Claude/claude_desktop_config.json`, `~/.codex/config.toml` and `~/.agents/mcp.json` each have **0** references to any DELETE tool. `settings.json` has only Paperclip permission rules (H2).
6. **Harness-squared.** `install_defaults.py` TARGETS (`.config/opencode/harness.md`, `.gemini/GEMINI.md`, `.openclaw/workspace/coordinator/AGENTS.md`, `.knowledge.md`) and the `h2.py` agent skill dirs reference DELETE harness paths. A re-run would crash with FileNotFoundError or recreate files. This is an always-on installer, so per CLAUDE.md the prune goes to `shared-workflows/harness-squared/PROPOSALS.md` for Cayleb's yes. It is not edited directly.
7. **Skill links.**
   - DELETE harness skill dirs are almost all links: `~/.config/opencode/skills` 19 links + 4 unique, `~/.gemini/skills` 12, `~/.openclaw/skills` 12, `~/.gemini/antigravity-cli/skills` 12. Archiving keeps the 4 unique OpenCode skills.
   - In the other direction, only the 13 `~/.claude/skills` links into Paperclip exist (H2).
8. **Autostart.**
   - `~/.config/autostart` has only limine-snapper-notify, fcitx5 and print-applet.
   - Hyprland has no exec line for any harness; only the Outis OS rules and keybind (N/A).
   - No crontab binary. No `~/.claude/scheduled-tasks`.
9. **Timers.** All 16 active timers were re-checked. No unit file or direct script references a DELETE binary, apart from the OpenCode dependents in §3.
10. **Desktop entries and icons.**
    - `~/.local/share/applications` has antigravity, apotheka-opencode, freebuff, grok-bot, opencode, pantheon and paperclip-pantheon, all assigned above.
    - Icons: antigravity.png (D1), grok-bot.png (H1).
    - `/usr/share/applications/chatgpt.desktop` is KEEP.
11. **Archive target.** Same btrfs subvolume (`[/@home]`) and outside any git repo. Needs `chmod 700`: it will hold auth.json ×2, oauth_creds.json, the OpenClaw gateway token, the Freebuff device key, the OpenWork server tokens, the OpenCode server password and the NIM key.
12. **Running harness-class processes (re-checked 21:3x).** OpenCode PID 1625, outis-control 1488, outis-mesh 1489/1764/1839, Grok Bot 11930/11998 + children. No Gemini, agy, OpenClaw, Freebuff, OpenMaus, Paperclip, browser-use or dsh processes.

**How sweep disagreements were resolved:**
- **OpenCode:** four sweeps said DELETE, the dependents sweep said HOLD. Result: HOLD on the live gate, with the full plan written.
- **Grok CLI:** the apps sweep said HOLD. Result: DELETE, since no caller exists (verified).
- **OpenMausBot:** the dirs sweep said HOLD. Result: DELETE.
- **Prime Agent:** two sweeps said DELETE. Result: HOLD, as ASSUMPTIONS names it.
- **Paperclip control plane:** the services sweep said DELETE. Result: HOLD, because it is one tree with the workspaces and skill links.
- **outis-control:** the services sweep said DELETE. Result: HOLD, because it is live and serves an external device.
- **Freebuff on-disk version:** the dependents sweep said 0.0.179. The disk has 0.0.188 (verified).

## 8. Needs Cayleb (his own hand; relayed yes does not count)

1. **OpenCode (G1):** yes to stop the live `opencode-service`, after Money Desk drops its OpenCode checks. Everything else for OpenCode is pre-planned.
2. **Grok Bot app (H1):** keep or remove. It is running and tied to the Grok trader. If removed, he quits it himself.
3. **Paperclip (H2):** archive or delete, workspace-repo rescue (unfetched USG street branches), and the 13 Claude skill links.
4. **Outis pilot (H3):** keep serving `laptop-pilot`, or offboard it and remove `outis-control` + `outis-mesh`.
5. **Small unclear items:** Prime Agent remnant (H4), browser-use (H5), Muse CLI tarball (H6, check with Nox), Talos orphan units (H8), `~/.winvm-scratch` (H9), voice-agent template (H10).
6. **Root-only:** `sudo pacman -R herdr`, only if he wants herdr gone (H7). The DELETE set itself needs no root.
7. **Flags:** the Claude Code CLI (mise, 9 versions) and the Codex CLI (mise, 7 versions) are treated as KEEP. He can say otherwise.
8. **Always-on change:** the harness-squared `install_defaults.py` / `h2.py` target prune goes to PROPOSALS.md (§7, item 6).
9. **When satisfied:** he empties the archive himself with `rm -rf /home/cayleb/Work/_archive/harness-removal-20261030`.

## 9. Phase-2 order and after-state checks

**Order:**
1. Shared setup (§2).
2. D3 Freebuff, D6 OpenWork, D5 OpenMausBot, D7 omp, D4 Grok CLI, D2 OpenClaw, D1 Gemini/Antigravity.
3. mise config edit (gemini line) + `mise reshim`; `.bashrc` grok block; Omarchy tile and menu edits; `update-desktop-database`.
4. G1 OpenCode, only when its gate opens.
5. HOLD items, only on Cayleb's yes.

**After-state checks**, as the outside judge for phase 3:
- `command -v gemini agy antigravity openclaw grok agent omp freebuff-desktop` → all empty. After G1, also `opencode opencode-desktop`.
- `ls ~/.local/share/mise/shims | grep -E 'gemini|openclaw|opencode'` → empty after the respective steps.
- `mise exec node@26.7.0 -- npm ls -g --depth=0` → no `openclaw`, no `@google/gemini-cli`. Still lists `agent-browser`, `@jackwener/opencli`, `vercel`, `@higgsfield/cli`.
- `pgrep -af 'agy|antigravity|openclaw|freebuff|openmaus'` → empty.
- `systemctl --user list-unit-files | grep -i opencode` → empty after G1.
- `ls -d` of every archived path → "No such file". The same paths exist under `$A/<name>/`.
- **KEEP intact:**
  - `claude --version` and `codex --version` succeed.
  - Claude Desktop is still running.
  - `/usr/bin/chatgpt` and `~/.local/bin/chatgpt` are present.
  - `ls ~/.local/share/mise/installs/node/26.7.0/bin/node` exists.
  - The linux_desktop MCP still starts.
  - `~/Documents/Codex/2026-10-30/task-2/dsh-*` are unchanged.
  - `~/workspace/harness-build` and the ergon/dsh-fleet repos are untouched.
- **Dependents:**
  - After the D-rows: `systemctl --user is-failed hardening-verify.service`. Expect new missing-path warnings for Freebuff until `expected-state.json` is updated.
  - After G1: `systemctl --user status outisz-freshness.service` exits 0, and the watchdog sends no "OpenCode down" alert.
