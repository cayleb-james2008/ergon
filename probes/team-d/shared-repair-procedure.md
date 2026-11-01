# Shared Studio-route repair — the once-only procedure (Team D)

Status: **CONFIG SIDE EXECUTED 2026-10-31 18:39 ET (S1a). Studio side awaiting Cayleb's window only.**
Authority: Lead's go + Nox's written green light (2026-10-31 ~18:23 ET: the `enabled = false` line was a one-session
measure from 28 Oct, never a standing prohibition; no fleet gate/fixture depends on it) + Cayleb's Studio-route
answer ("Use the desktop Studio now").
Scope: **one repair, both games** (Job 16 street, Job 17 card). Never both at once.
Prepared: 2026-10-31 18:20 ET by `roblox-team`, from live probes on `outis-desktop`; execution record added 18:40 ET.

---

## 1. State at the time of writing (quoted, 18:05–18:20 ET)

```
$ flatpak list | grep -i vinegar
Vinegar  org.vinegarhq.Vinegar  1.9.4  stable  user
$ ls …/vinegar/versions/
version-7062aec9dbe74b14                      # Oct 27 18:30; RobloxStudioBeta.exe 222,384,592 B; StudioMCP.exe 8,224,720 B
$ flatpak ps                                  # (empty — no Studio process)
$ ss -ltnp | grep 13469                       # not listening
$ pgrep -af StudioMCP                         # none
$ ~/.local/bin/roblox-studio-mcp              # Error: Roblox Studio (org.vinegarhq.Vinegar) is not running in Flatpak.
$ ls -la ~/.cache/usg/vinegar-plays.json      # 83 B, Oct 26 22:12 (the 3-Plays-per-Studio counter; stale, harmless)
$ diff <backup> ~/.codex/config.toml          # exactly two differences, see §2
```

So the machine is **clean right now**: no Studio, no MCP server, port free, no stale process to reap. The repair is
therefore a **single line plus an owner-opened Studio**, not a process cleanup.

---

## 2. The repair — exact edit, and the trap

**File:** `/home/cayleb/.codex/config.toml`

**Old line (line 192, verbatim):**
```
  enabled = false   # DISABLED 2026-10-28 by the consolidation lead (Nox order street-unblock): native Roblox fixtures need exclusive port 13469; undo = delete this line (backup: lanes/consolidation-20261027/A/codex-mcp-disable-20261028/config.toml.pre-disable)
```
**New line:** *(the line is deleted — the recorded undo is "delete this line")*

Context it sits in (lines 190–192), which must be left intact:
```
[mcp_servers.roblox_studio]
command = "/home/cayleb/.local/bin/roblox-studio-mcp"
  enabled = false   # … (the line being deleted)
```

**The trap — do not restore the backup file wholesale.** The named backup exists
(`~/Work/shared-workflows/lanes/consolidation-20261027/A/codex-mcp-disable-20261028/config.toml.pre-disable`,
10,583 B, Oct 27 13:23) but it is **not** identical to today's config: `diff` shows exactly two changes —
`model_reasoning_effort = "ultra"` → `"xhigh"` (line 2) and the added `enabled = false` (line 192). Restoring the
whole file would silently revert the model reasoning effort. **Delete only line 192.**

**Backup to take at execution time (before the edit):**
`cp -p ~/.codex/config.toml ~/.codex/config.toml.pre-studio-repair-$(date +%Y%m%dT%H%M%S)`
Record its sha256 in the run log. Rollback = restore that copy, or re-insert the single line.

**Why this is the repair and not the brief's version-path fix.** `mcp.bat` pins
`…/versions/version-7062aec9dbe74b14/StudioMCP.exe` and that file exists; the launcher reads the pinned path
dynamically, so nothing points at a deleted folder on this machine. The fallback edit for a future auto-update (if the
old folder is ever deleted) is in §7.

---

## 3. Port 13469 — what it actually is, and the sequence

The port is the local socket between **StudioMCP.exe** (a process per Studio session) and the MCP client
(`roblox-studio-mcp` → `wine StudioMCP.exe`). Two known facts, from the street game's own runbook
(`docs/verify/2026-10-19/VINEGAR_RUNBOOK.md`):

- "On 2026-10-16 a stale one that held the port with a backlog stopped Studio from listing."
- "On 2026-10-19 17 [StudioMCP.exe were] run; 14 are older than 1 h (oldest ~28 h)… Reaping is two steps, and only
  the second one kills." The dry run is `vinegar_run.py --reap-stale-mcp preflight` (lists PIDs + a token, kills
  nothing); the kill needs **Cayleb's yes** to those PIDs, then `--reap-stale-mcp --confirm-reap TOKEN preflight`.
- Still unproven on the record: "Whether killing those stale servers frees the port for the new one is also
  unverified." Do not claim it works until a repair run shows it.

**Sequence (this is the ruling Nox must confirm):**

| # | Action | Who | Gate |
|---|---|---|---|
| 1 | Confirm no other lane needs Roblox fixtures on port 13469 for the duration | Lead ↔ Nox | Nox's written confirmation |
| 2 | Check state: `flatpak ps`, `ss -ltnp \| grep 13469`, `pgrep -af StudioMCP` | agent | all clean, or the stale reap procedure with Cayleb's yes |
| 3 | Delete config line 192 (backup first, §2) | agent | the go |
| 4 | Cayleb opens **one** Studio, in Edit, signed in (or confirms existing session) | **Cayleb** | — |
| 5 | Verify the route: `roblox-studio-mcp` handshake answers; record Studio version + account | agent | handshake bytes |
| 6 | Run the **street game's** native cells first (§5) | agent | per-cell evidence |
| 7 | `python3 tools/vinegar_run.py verify-clean` then Cayleb closes Studio (**Don't Save**) + `hyprctl reload` | agent + Cayleb | verify-clean exit 0 |
| 8 | Card game's Studio cells (binding verification) second | agent | 24/24 + 8/8 |
| 9 | Leave the line **absent** (route working for both games) and record the repair in the board | agent | board entry |

Rule: **one Studio per Roblox login, ever** (the project's own record: "two Studios revoked that login on
2026-10-16"), and 3 Plays per Studio process before Cayleb reopens.

---

## 4. Who does what — the Studio-open handoff (Cayleb's part, verbatim from the runbook)

The runbook's steps 4–7, which are the whole of his mechanical part:
1. Park the Studio window (it is expected to be on the Wayland desktop).
2. Open the place — for the street game, the running copy of `build/test.rbxlx`; for the card game, the candidate
   `.rbxlx` named in the receipt (`55cef5d3…`, see `board/probes/team-d/uacg-candidate-receipt.json`).
3. Sign in as **Chrono_H3X** **if asked** (a fresh password/2FA demand is his; never type, never store).
4. Leave it in **Edit** mode and say go.

What the agent never does: launch Studio, open a second Studio, press Play before the runner, reap processes without
his yes, or touch credentials.

---

## 5. Which game first, and why (street first)

1. **The street game's native plan is already written and unrun** — 51 Plays in 19 openings (`SPRINTS.md`), with a
   runner, a queue, a lock, `verify-clean` and a 3-Play counter. Nothing has to be invented to use the window.
2. **It has a live private place** (universe `10770936993`, root place `107924030342600`, published v7), so the
   persistence and receipt cells can run against a **saved** version. The card game has **no authorised target**
   (`ops/release/target.json` absent) and therefore cannot do live receipts at all yet.
3. **The card game's first Studio work needs upload authority** (Q6) because binding requires real asset ids; the
   street game's first Studio cells need no upload whatsoever.
4. Therefore: **street native cells occupy the first window(s); the card game's per-card binding verification follows
   once Q6 is answered.** If only one window ever opens, the street game gets it.

---

## 6. Verification of the repair (evidence that must exist before it is called done)

1. `grep -n "enabled" ~/.codex/config.toml` → no `enabled = false` under `[mcp_servers.roblox_studio]`; capture the
   three lines (190–192) verbatim.
2. The MCP handshake: the client's `initialize` + `tools/list` round trip answered, with the raw response bytes saved
   under `board/probes/team-d/`.
3. Studio's own version string and the signed-in account name, captured **from Studio's UI** (not from a file).
4. `ss -ltnp | grep 13469` shows exactly one listener while a session is live; `pgrep -af StudioMCP` shows exactly one
   server for that Studio.
5. `python3 tools/vinegar_run.py status` exit 0 at the start; `verify-clean` exit 0 at the end of a window.
6. A negative control, if cheap: with the line re-added, the handshake fails (proves the line is what gates it). Only
   in a second, separately scheduled window — never mid-session.

## 7. Fallback repair (only if a future Studio auto-update deletes the pinned version folder)

**File:** `~/.var/app/org.vinegarhq.Vinegar/data/vinegar/prefixes/studio/drive_c/users/cayleb/AppData/Local/Roblox/mcp.bat`
**Old:** the `if exist "Z:\…\versions\version-7062aec9dbe74b14\StudioMCP.exe" ( … ) else ( … reg query … ContentFolder … )` line.
**New:** replace the pinned `version-7062aec9dbe74b14` with the new `version-<hash>` folder, **or** delete the
`if exist` clause so the registry fallback always runs. Not needed today (§1). Studio regenerates this file on updates,
so treat it as Studio-owned.

## 8. What must NOT happen

- No second Studio; no Studio launched by an agent; no credential typing or storing; no `git` change in either game.
- No reap without Cayleb's yes to the exact PIDs; no kill of a long-running session's server.
- No running both games' Studio work at once; no publication; no purchase prompts enabled for the public.
- No claim that the route works until §6 items 1–5 exist.

---

## 9. Board mail — ready to send to Nox (the Lead sends it)

> **Subject: Team D Roblox — port 13469 sequencing for the one shared Studio repair**
>
> Nox — Team D (Roblox, Jobs 16/17) has one Studio repair to run and it needs your port ruling before we touch it.
> On 2026-10-28 the consolidation lead disabled the Codex MCP Studio server with one line in `~/.codex/config.toml`
> ("Nox order street-unblock: native Roblox fixtures need exclusive port 13469; undo = delete this line"). We have
> independently confirmed: the launcher and its pinned Studio version folder are intact (Vinegar 1.9.4, Studio 0.745,
> `StudioMCP.exe` present), no Studio is running, port 13469 is free, no stale `StudioMCP.exe` exists, and the config
> backup named in that comment differs from today's config by two lines (the `enabled = false` line and
> `model_reasoning_effort`), so restoring it wholesale would revert the reasoning-effort setting — we will delete only
> the one line.
> **Ask:** please confirm (a) that no lane needs Roblox fixtures on port 13469 for the duration of one Studio window,
> and (b) the sequencing — Team D proposes street game (Job 16) native cells first, then the card game (Job 17)
> binding verification, with `verify-clean` and a full Studio close between them, one Studio per Roblox login at all
> times (the project record: two Studios revoked the login on 2026-10-16). If a stale server must be reaped we will
> send the dry-run PID list to Cayleb for his yes first, as the street runbook requires.
> Nothing has been edited yet. This is a request for the sequencing ruling, not a notification of a change.

---

## 10. EXECUTION RECORD — S1a (config side) DONE, 2026-10-31 18:39 ET

**Authorised by:** Lead's go + Nox's green light (the disabled line was a one-session measure from 28 Oct, never a
standing prohibition; no fleet gate/fixture depends on the disabled state — if the repair surfaces one, record it and
do **not** re-disable) + Cayleb's answer "Use the desktop Studio now".

**Backup first, and NOT the stale one:**
```
$ cp -p ~/.codex/config.toml ~/.codex/config.toml.pre-studio-repair-20261031T183910
$ sha256sum …pre-studio-repair-20261031T183910
e10f08e43f6ed5601d8cee48d9ab8c549bbd158ab9d2dcf6f5e9782995cff56c   # = the BEFORE state of the live file
```
(The old `config.toml.pre-disable` was deliberately **not** used: it differs by `model_reasoning_effort = "ultra"`
vs today's `"xhigh"` — restoring it would have reverted that setting. Trap avoided, as planned in §2.)

**The edit — exactly one line removed (line 192), nothing else:**
```
$ diff -u <backup> ~/.codex/config.toml
 [mcp_servers.roblox_studio]
 command = "/home/cayleb/.local/bin/roblox-studio-mcp"
-enabled = false   # DISABLED 2026-10-28 by the consolidation lead (Nox order street-unblock): native Roblox fixtures need exclusive port 13469; undo = delete this line (backup: …pre-disable)
```
`diff` changed-line count = **1**. Bytes 10,843 → 10,583 (+260 = the removed line + newline). Mode kept `-rw-------`.

**Hashes:**
| | sha256 |
|---|---|
| before | `e10f08e43f6ed5601d8cee48d9ab8c549bbd158ab9d2dcf6f5e9782995cff56c` |
| after | `752a7a2f0e80183f6679efa332930af7fca2d492bd9a6529f35ebcd40eff3ac8` |
| backup | `~/.codex/config.toml.pre-studio-repair-20261031T183910` (sha = before) |

**Proof the file is intact and the server is enabled:**
```
$ python3 -c "import tomllib;tomllib.load(open('~/.codex/config.toml','rb'))"   -> PARSE_OK (17 top-level keys)
parsed [mcp_servers.roblox_studio] = {'command': '/home/cayleb/.local/bin/roblox-studio-mcp'}   # 'enabled' key GONE
all 7 servers still present: blender, chrome-devtools, free_3d_assets, linux_background, linux_desktop, node_repl, roblox_studio
$ grep -c "enabled = false   # DISABLED 2026-10-28"   -> 0
```

**Environment at 18:39 ET (nothing else touched):** `flatpak ps` → 0 processes; `ss -ltnp | grep 13469` → 0 listeners;
real `StudioMCP` processes → 0; no Studio launched by the agent; no credential read.

**What remains — the Studio side (Cayleb's acts, at run time):**
| Step | Who | State |
|---|---|---|
| Open **one** Studio (Vinegar), leave in Edit; sign in as `Chrono_H3X` **if asked** | Cayleb | waiting |
| Handshake check: `~/.local/bin/roblox-studio-mcp` answers (initialize + tools/list); save raw bytes | agent | waiting (needs Studio) |
| Read version + account from Studio's own UI (`0.745.0.7451137` / `Chrono_H3X`) | agent | waiting |
| Street game first — ≤3 Plays per Studio process, then `verify-clean` | agent | waiting |
| Card game second (only once real asset ids exist) | agent | blocked on uploads |
| Close: **Don't Save** + `hyprctl reload` | Cayleb | — |
The config line **stays absent** (the repair is for both games). If the route still fails, restore the timestamped
backup above and report the failure — do not improvise a second repair.

**Nox's condition:** if this repair surfaces a fleet gate/fixture that depended on the disabled state, **record it and
do not re-disable**. Nothing has surfaced yet — the check needs a live MCP handshake, which needs the window.
