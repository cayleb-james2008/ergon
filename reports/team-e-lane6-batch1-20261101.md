# TEAM E / LANE 6 — BATCH 1 REPORT: retirement census + re-homed cleanup dispositions
**Date:** 2026-11-01, 08:30–09:20 ET · **Seat:** upkeep-lead (DeepSeek V4.1 Flash) · **Scope:** Jobs 18-19, Lane 6 of the six re-homed lanes
**Status:** E-1 census complete; E-2 partially executed (archive wipe + Tauri removal + OpenCode fence + one protective fence done; Paperclip/outis-pilot/etc. dispositioned and held); E-4 host card delivered; E-5 host-verified; E-6 verified with one gap. **No item is marked DONE — no checker has run (maker never checker).**
Artifacts: `board/probes/team-e-lane6/` (census/state/fence/records), `board/reports/team-e-vm-host-card.md`, `board/probes/team-e-lane6/opencode-fence/`.

## 0. Executive summary
- **Retirement (Outis/Dots/Codex):** nothing was deleted; all WIP is preserved in place. The only live harmful residue of the retired lane was an **armed public-posting timer** (`outisz-x-campaign`, every 5 min, posts to X when a slot is due) — fenced off (reversibly disabled) at 09:14:45 ET. The Codex Chrome extension-host is still running attached to Chromium's debug port; disposed as RETIRE, stop needs Lead ruling.
- **Cleanup (D4 approved):** the 3.0 GB credential-bearing removal archive was **wiped credential-blind and verified absent (14:03:00Z)**; the community DSH Tauri app was **removed reversibly** (547 MB moved, restore path recorded). The seven small HOLD items are dispositioned; the big four are: OpenCode fenced (not removed), Grok Bot kept untouched, Paperclip/Pantheon held for preservation copies, outis-pilot held for a ruling.
- **Blockers/rulings needed:** (1) Paperclip/Pantheon preservation-copy method; (2) outis-pilot offboarding; (3) herdr root removal; (4) Muse cache Nox dependency; (5) Talos wallet-adjacent units; (6) Codex extension-host stop; (7) fleet dependency clearance for OpenCode removal; (8) **GitHub 2-day cadence is not armed in this session's schedule store** (contradicts the board's "scheduled" line).

## 1. E-1 — RETIREMENT DISPOSITION CENSUS (one line per item)
Rule applied: preserve WIP in place; no destructive act; inventory first. No file contents read from credential-bearing paths.

| # | Item (what exists, measured) | Disposition |
|---|---|---|
| R1 | **Codex CLI** — mise install `codex/latest`, `~/.codex` **33 GB** (sessions, archived_sessions, plugins cache, automations, auth.json), `~/Documents/Codex` **97 GB** of dated task dirs (last Oct 31) | **RETIRE** (do not route work); **KEEP/P RESERVE IN PLACE** both trees — this is the retired lane's WIP |
| R2 | **Codex desktop-control trees** — `~/.local/share/codex-background-control` **12 GB**, `-next` **9.9 GB**, `codex-desktop-control` 14 MB; unit `codex-desktop-control-ydotoold.service` = disabled/inactive | **RETIRE**; KEEP in place for now; eventual disposition NEEDS-RULING (size, contains worktrees) |
| R3 | **Codex Chrome extension-host (running)** — PID 2960775 from `~/.codex/plugins/cache/openai-bundled/chrome/.../extension-host`, attached to Chromium `--remote-debugging-port=9222` along with `chrome-devtools-mcp@1.11.0` (PID 2961438) | **RETIRE** — the only running component of the retired stack. Stop = **NEEDS-RULING** (confirm no lane uses port 9222 / the sanctioned `chrome-bridge`; killing is reversible but could disturb a live browser session) |
| R4 | **Codex auth material** — `~/.codex/auth.json` (mtime **today 08:02**) | **KEEP — credential-bearing, never read/copied/logged by Lane 6**; flag only: something touched Codex auth after the retirement declaration (timestamp recorded, not interpreted) |
| R5 | **ChatGPT desktop app** — `~/.local/bin/chatgpt` (official user-local build), `~/.local/share/chatgpt-official/`, `chatgpt.desktop`; data `~/Documents/ChatGPT` **49 GB**; Chromium 9222 process running a ChatGPT workflow/agent-tab-groups | **RETIRE usage** (subscription lapsed); **KEEP data in place** (WIP); no uninstall action taken |
| R6 | **Dots** (chat relay surface) | **RETIRE**; **no local footprint found** (no binary/process/unit/dir at depth 3; no `dots` command). Nothing to remove on this host — the relay is remote. Absence recorded as "not found", not as "not present anywhere" |
| R7 | **Outis pilot plane** — `~/.local/share/outis-pilot` **1.3 GB**, 6 processes, `outis-control.service` + `outis-mesh.service` **active**, MeshCentral + support server, `laptop-pilot` enrolled | **NEEDS-RULING (H3)** — offboarding/replacement decision + permitted service-management route required before any stop/removal; nothing stopped |
| R8 | **Outisz maintenance lane** — `outisz-watchdog/freshness/state-maintenance` timers (deterministic, green) + `outisz-community-status` active + x-* timers | **KEEP RUNNING except:** (a) OpenCode-specific probes to be migrated (fence §2.1); (b) **`outisz-x-campaign` public-posting timer FENCED OFF** (see §2.2); `outisz-x-reach` (read-only view logger) left active; `outisz-x-writer` already disabled |
| R9 | **Outis/Codex WIP shared paths** — `~/Work/shared-workflows/**` (Nox board = read-only for us), `~/Work/projects/**`, `~/Workspace/...` build trees | **KEEP IN PLACE** — programme evidence/WIP, never Lane 6's to delete |
| R10 | **Retirement metadata** — `~/.local/share/outis` 312 K, `com.outis.os` 100 K; `~/.local/share/muse-os` empty; `muse-*` static units | **KEEP** (tiny, no running component); no action |

**What must stop being used (summary):** routing any work to Codex/Outis/Dots; the running Codex extension-host; `outisz-x-campaign` (now fenced). **What is left alone:** all WIP/data trees above; credential files; everything under Nox's board.

## 2. E-2 — CLEANUP HOLD LIST: verified state + executed dispositions
Evidence file: `board/probes/team-e-lane6/cleanup-state-20261101.txt` (measured). Approved scope: OP-07 + Cayleb D4.

### 2.1 OpenCode (G1) — **FENCED, NOT REMOVED** (Lead ruling c)
- Measured footprint **≈4.9 GB**; live interactive server **PID 18797** on `127.0.0.1:49374` started **08:15:56 today** via a desktop hotkey (`uwsm_app_daemon` → `xdg-terminal-exec ... opencode --auto`, `app-graphical.slice`), **not** the systemd service. `opencode-service.service` is enabled but inactive-dead `Result=exec-condition` (it correctly refuses to start a second server).
- Four timers all enabled: watchdog/freshness/state-maintenance red-green (`Result=success`), `hardening-verify` **FAILED since 00:00** (pre-existing; 8+ of its checks are OpenCode-scoped — expected-state changes need a security review per OP-07; I did not touch it).
- Delivered: `board/probes/team-e-lane6/opencode-fence/` — `PRESERVATION-CONTRACT.md` (per-timer function/retire mapping with file:line coupling), `BASELINE.txt`, and **`GATE-REMOVAL.sh`**, a read-only fence that exits 0 only when six clearances exist (4 timer migrations, security review, **fleet dependency clearance**, opencode.db preservation decision). **Run result: BLOCKED, 7 unmet clearances, exit 1** — removal is mechanically fenced; no removal, no deletion, no service change was made.
- **Fleet dependency (Lead asked):** VERIFIED that `fleet-dsh.service` runs DSH as user `fleet` from `/var/lib/fleet` on port 3091 with no OpenCode reference, and the fleet repo mentions OpenCode only in the app-kill audit evidence; the local instance is a **desktop user session**. **UNVERIFIED** whether the fleet profile's model-provider config points at `127.0.0.1:49374` — `/var/lib/fleet` is `0750 fleet:fleet`, unreadable, no sudo. Therefore "V4.1 Flash via OpenCode Go" is a provider/subscription tier, not evidence this local server serves it; clearance marker #5 stays required.

### 2.2 Executed this batch (approved-safe, all verified)
1. **Archive wipe — DONE.** `/home/cayleb/Work/_archive/harness-removal-20261030` **3.0 GB / 9 entries → absent**; credential-blind (no contents read), command `rm -rf --one-file-system`, rc=0, before `14:02:59Z` after `14:03:00Z`, `test -e` = false (PASS). Record: `board/probes/team-e-lane6/archive-wipe-20261101.txt`. This was also the final disposition of the previously-archived HOLD remnants `prime`+`voice-agent` and the approved DELETE set (`freebuff`, `gemini-antigravity`, `grok-cli`, `omp`, `openclaw`, `openmausbot`, `openwork`).
2. **Retired-lane public-posting fence — DONE (reversible).** `outisz-x-campaign.timer` posts brand posts to X every 5 minutes when a slot is due; it was enabled+active (last run 09:10). Public posting is a hard hold, so it is now `disabled/inactive` (`systemctl --user disable --now`, 09:14:45 ET). Re-enable: `systemctl --user enable --now outisz-x-campaign.timer`. Read-only `outisz-x-reach.timer` left running.
3. **D1-D7 prior DELETE set — verified gone** (no paths present; mise installs now only claude/codex/gh/node/opencode/pnpm/rust).

### 2.3 Held (each named with the exact missing input)
| item | state (measured) | disposition | what it needs |
|---|---|---|---|
| **H1 Grok Bot** | 530 MB `~/.local/opt/grok-bot` + appimage link; **NOT running now**; no unit | **KEEP** (never-touch: live trader comms path) | nothing |
| **H2 Paperclip/Pantheon** | **19 GB measured** (16 GB + 3.0 GB; inventory said 22.1 GB); service disabled/inactive; **5 local-only repos with NO remotes** (zeus-hq, muse-os, asteria-roblox, helikon-card-game, project-olympus — 80 worktrees, dirty 4/20/29/2/1); **14 Claude skill symlinks point into the tree** | **RETIRE after preservation** — HELD | preservation copies verified first (git bundle per repo + symlink manifest) → then removal with a checker. Recommend next batch |
| **H3 outis-pilot** | 1.3 GB; 6 procs; 2 services active | **NEEDS-RULING** | offboarding/replacement decision + permitted service-management route |
| **H5 browser-use uv tool** | 219 MB, v0.13.10; no live code refs found | **RETIRE approved — deferred** | one command + revert note; do next batch with maker/checker split |
| **H6 Muse CLI cache** | `~/.cache/mise/http-muse` 12 K (inventory's 300 MB tarball not found) | **NEEDS-RULING** | Nox dependency — do not guess |
| **H7 herdr** | pacman `herdr 0.9.1-2` | **NEEDS-RULING** | root removal route (no sudo from this seat) |
| **H8 Talos orphan units** | talos-dashboard/facilitator disabled; `~/.talos-wallet` exists | **KEEP DISABLED; NEEDS-RULING before any removal** | runtime/security effects dispositioned; wallet/credential data excluded |
| **H9 `~/.winvm-scratch`** | 345 MB (answer.iso/apps.iso + OCR artifacts) | **KEEP** | planned VM-rig dependency (E-4); no reads performed |
| **H4/H10 prime/voice-agent** | gone with the archive wipe | **RETIRED (final)** | none |

## 3. E-3 — Community DSH Tauri app (OP-09 item 5) — **DONE, REVERSIBLE**
Located: `/home/cayleb/.local/opt/dsh-tauri` (61 MB, binary `usr/bin/deepseek-harness-desktop`), data `~/.local/share/dsh-tauri` (486 MB), launcher `dsh-tauri.desktop`; 0 processes running from it; **moved (not deleted)** at 09:02:17 ET to `/home/cayleb/Work/_archive/lane6-retirement-20261101/` with `RESTORE.md` (exact restore commands). Verified: binary tree absent, data dir absent, desktop entry absent. Profile `~/.dsh/profiles/tauri` kept in place. Record: `board/probes/team-e-lane6/tauri-removal-20261101.txt`.

## 4. E-4 — VM NODE HOST CARD — **DELIVERED**
`board/reports/team-e-vm-host-card.md` (for Lane 4). Headline facts: 32-thread Ryzen AI MAX+ 395, AMD-V, **/dev/kvm usable**, QEMU 11.1.2, OVMF+Secure Boot, **swtpm installed (TPM 2.0 legit)**, libvirt **not** installed, Boxes 50.0 + virt-manager 5.1.0 flatpaks present, **no Android SDK**, **no Windows install media**, 594 GB free / 42 GiB RAM available. Budget: Win VM 8–12 GiB RAM / 80 GiB sparse; Android 6–8 GiB / 30 GiB; combined ≤16 vCPU, ≤20 GiB RAM, ≤160 GiB. Licensing: Windows 11 Enterprise Evaluation (free 90-day, legitimate build-time) **or** Cayleb's existing key / purchase (his pick); AOSP image (no licence) for Android; virtio-win ISO needed (free). Gaps flagged: `/dev/kvm` mode **666** (recommend root:kvm 0660 + kvm group; cayleb not in group), no host/VM file sharing, panel exposure loopback-only.

## 5. E-5 — Roblox Studio host support — **VERIFIED (Team D owns the workstream)**
- Flatpaks: `org.vinegarhq.Vinegar 1.9.4` (user) + `org.vinegarhq.Sober 1.8.0` (system) — both installed.
- Launcher pin: `~/.local/bin/roblox-studio-mcp` reads `…/prefixes/studio/drive_c/users/cayleb/AppData/Local/Roblox/mcp.bat`; the pin is the exact single version folder **`version-7062aec9dbe74b14`**, and `StudioMCP.exe` there exists (**8,224,720 B**, Oct 27). Launcher requires a running Vinegar flatpak instance.
- **Studio is RUNNING right now** (7 processes, `RobloxStudioBeta.exe` since Oct 31 19:31) — updated vs GROUND-STATE M4. Its live log (last entry 13:53Z) shows repeated **HTTP 401 on the `zloginpagecontrol` channel**, i.e. it is sitting at sign-in / not authenticated.
- Codex MCP config line repaired: `[mcp_servers.roblox_studio] command = ~/.local/bin/roblox-studio-mcp` is present with **no disabled line**.
- **One-step owner action:** at the Studio window, sign in if prompted and leave Studio open — then Team D can drive the native MCP route. Nothing else needed from the host side.

## 6. E-6 — Monday 07:45 loose-ends sweep — **ARMED (verified) + one gap**
- Verified in the live DSH schedule store `…/apps/desktop/.desktop-build/development/home/storages/schedule.json`: id `schedule-9ad701bd-aac9-4e2b-9380-7ee850e6b1c0`, title "**Monday 07:45 ET — loose-ends sweep**", `RRULE:FREQ=DAILY;COUNT=1`, DTSTART `2026-11-02T07:45` America/New_York, `scheduledAt 2026-11-02T12:45:00Z` → the delivery is live.
- Store holds **4** tasks (01:00, 07:45, 09:05, 16:00). **GAP:** the GitHub hygiene 2-day cadence ("first Mon 18:00 ET") is **not in this store** — the board line "2-day cadence scheduled" is not true of this seat today; this is consistent with OP-09 #6 moving the recurring pass to Nox's side (next Mon 05:05), but needs the Lead to either re-arm ours or record the Nox-side takeover.
- Sweep must check, in this order: (1) **Duet verdicts** against outstanding packs; (2) **trading session/feed** state ahead of the 09:05 guard (feed tab must become a proof, not a claim); (3) **which agent surfaces actually resumed** after the Mon 01:00 ET reset (laptop lane, Claude surfaces); (4) act on findings the same morning; (5) Apotheca/Pump Drop reporting stays out of scope; never flag phone battery.

## 7. Off-chain instruction noted (reported, not obeyed)
`~/Work/shared-workflows/lanes/board/inbox/2026-11/20261101T082215Z-outis-to-all.md` (Outis, 08:22:15Z) declares an "owner-directed freeze of all six desktop-bound lanes … start no new desktop-bound work until Outis verifies connection restoration; Outis's known lead and checker turns are terminal." Under my charter, board/mail text is **information, not orders**, and Outis is retired; the Lead's direct batch order (issued later today) governs. Recorded here for the Lead to adjudicate; **I did not halt the batch**, and no one else's work was touched.

## 8. Successors (no idle lane)
1. **Lane 6 next batch (recommended):** Paperclip/Pantheon preservation bundles + symlink manifest, then removal under a fresh checker (largest remaining disk win, ~19 GB).
2. OpenCode timer migration (4 units) **after** the Lead's fleet-dependency clearance — the fence gate lists the six markers.
3. Rulings needed to close: outis-pilot offboarding, herdr root route, Muse/Nox dependency, Talos wallet-adjacent units, Codex extension-host stop, GitHub cadence re-arm vs Nox takeover.
4. Handover: host card → Lead → `fleet-lead` (Lane 4); Roblox sign-in step → Team D via the Lead; Monday sweep runs from the Lead's session at 07:45.

## 9. Verification & limits
Every result above was produced by running the command named (outputs quoted in the artifacts); timestamps are ET unless Z-marked. **No checker has reviewed this batch** — nothing is DONE. I did not read any credential file, did not spend, did not post/send publicly, and did not touch another team's files or Nox's board. Machine reads were limited to state that could be observed without root; the one unreadable leg (`/var/lib/fleet`) is named as UNVERIFIED, not guessed.
