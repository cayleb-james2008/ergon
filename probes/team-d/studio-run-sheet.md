# TEAM D — STUDIO WINDOW RUN-SHEET (one page)

> **2026-11-01 09:05 ET — STATE UPDATE, supersedes the preflight/handoff lines below where they differ.** Studio is
> **already open and signed in** (Vinegar, since 2026-10-31 19:31:42 ET; log `Authenticated : YES` 23:31:52Z;
> userId 1170918805; 0 Plays) and the MCP route is **verified live** (33 tools, `list_roblox_studios` → live Studio,
> `get_studio_state` Edit). The runner place copy has been refreshed to the sealed candidate test place
> **`c6651793…`**; `vinegar_run.py preflight` now refuses **only** on `listed: ['Place1']` — i.e. the owner must open
> `usg-desktop.rbxlx`; he no longer needs to open Studio from scratch (and if Studio was closed, he opens it first).
> **Lead ruling: do not drive the open Studio until Cayleb confirms it is his session (via Nox).** Current successor,
> exact one-step owner wording, first-sitting order and stop rules: `d5-window-day-runbook-20261101.md`. Evidence:
> `d1-route-state-20261101.md`.

Jobs 16 (street) and 17 (card). **State: the config-side repair is DONE** (line 192 deleted, before
`e10f08e4…` → after `752a7a2f…`, backup `~/.codex/config.toml.pre-studio-repair-20261031T183910`). Only the Studio side
remains. Opening Studio and any sign-in are **Cayleb's acts**; the agent never launches it.
Appendices (do not re-read unless something fails): `shared-repair-procedure.md` §10 · `upload-plan.md` ·
`audio-import-design.md` · `studio-evidence-sheet.md`.

## 1 · Preflight — agent, before saying a word to him

- [ ] `grep -c "enabled = false   # DISABLED 2026-10-28" ~/.codex/config.toml` → **0**; config still parses.
- [ ] `flatpak ps` → 0 · `ss -ltnp | grep 13469` → 0 · real `StudioMCP` processes → 0. If a stale server holds the
      port, use the two-step reap (`vinegar_run.py --reap-stale-mcp preflight`) and **show him the PID list first**.
- [ ] Street repo: `python3 tools/hostcheck.py preflight` → `quiet: true`, exit 0 (timed fixtures refuse a busy host).
      `python3 tools/vinegar_run.py status` → exit 0, no latch.
- [ ] Build hashes on disk: street `build/test.rbxlx` = `c6651793…`; card `UntitledAnimeCardGame.rbxlx` = `55cef5d3…`.

## 2 · Handoff — say exactly this (≈1 minute of his time)

1. "Open **Studio** (Vinegar). If it asks you to sign in, sign in as **Chrono_H3X**; if it asks for a password or 2FA,
   stop and tell me."
2. "Open the place I name: **street first** — the running copy of `build/test.rbxlx`."
3. "Leave it in **Edit** (not Play), park the window, and say *go*."
   → One Studio, ever. Two Studios revoked the login on 2026-10-16.

## 3 · The window — order, no deviations

| # | Step | Pass = | Record |
|---|---|---|---|
| 1 | MCP handshake: `~/.local/bin/roblox-studio-mcp` answers `initialize` + `tools/list` | response received | raw bytes → `board/probes/team-d/mcp-handshake-<stamp>.txt` |
| 2 | Read Studio's **own UI**: version + signed-in account | `0.745.0.7451137` / `Chrono_H3X` | screenshot |
| 3 | **Street game** native cells per `SPRINTS.md` — **≤3 Plays per Studio process** | per-Play exit 0 | exit code + JSON + stderr |
| 3b | After 3 Plays: tell him to close (Don't Save) and reopen for the next block | — | play counter `~/.cache/usg/vinegar-plays.json` |
| 4 | `python3 tools/vinegar_run.py verify-clean` | exit 0 (place clean, no helper, no flags) | exit + output |
| 5 | **Card game second** — only if real asset ids exist (they do not yet); otherwise skip | — | "skipped: no asset ids (uploads pending)" |
| 6 | `verify-clean` again if the card game ran | exit 0 | exit |
| 7 | Close: "Shut Studio — **Don't Save**; then run `hyprctl reload`" | he confirms | — |

## 4 · Stop rules (inside the window)

| Signal | Meaning | Do |
|---|---|---|
| exit 0 | pass | next Play |
| exit 1 | **real finding** | keep evidence, diagnose, **do not re-run to pass** |
| exit 2 | preflight refused, nothing changed | fix the stated reason (Studio closed/wrong place/host busy/3 Plays/lock) |
| exit 4 | host-contaminated | label it; a queue reruns once on the next reopen |
| exit 5 | crash / MCP disconnect | re-copy the place from `build/test.rbxlx`, reopen, `verify-clean` |
| same step fails twice | stop rule | stop the window and report — no third way |

Never: a second Studio, a Play outside the runner, killing his jobs for a quiet host, re-enabling the config line.

## 5 · Evidence to leave with (minimum before the window counts)

1. MCP handshake file + Studio version/account screenshot.
2. Per-Play exit codes + JSON results; failures kept as failures.
3. `verify-clean` exit 0 at the end (and after each block).
4. Street: the sprint kit's own per-Play records. Card (later): the 24+8 capture sheet — one screenshot per card, one
   trigger record per cue, each naming the data id and the build hash.
5. Session header written (date, machine, version, account, place, build sha open + close, who drove).

## 6 · Close-down and rollback

- Housekeeping: tell him **Don't Save**, then `hyprctl reload`. The config line **stays deleted** (the repair serves
  both games).
- If the route fails (no handshake): **restore** `~/.codex/config.toml.pre-studio-repair-20261031T183910`, report the
  failure with the handshake error — do not improvise a second repair.
- **Nox's condition:** if a fleet gate/fixture that depended on the disabled state surfaces here (it gates on port
  13469 / native fixtures), **record it and do not re-disable**. Nothing has surfaced yet — this window is where it
  would show.

## 7 · What this window does NOT do

No uploads (Cayleb's authorisation + Open Cloud key pending), no publication, no purchase prompts, no card-game
binding (no asset ids yet), no spend. Street native cells first is the whole point of the window.
