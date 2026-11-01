# S1 — the Studio-window card (ONE screen)

**Purpose:** the whole shared-repair studio window on one screen, so the window is used once and closed cleanly.
**Status:** **CONFIG SIDE DONE 2026-10-31 18:39 ET** — Nox's green light + Cayleb's "Use the desktop Studio now" both
received; line 192 deleted (before `e10f08e4…` → after `752a7a2f…`, diff = 1 line; backup
`~/.codex/config.toml.pre-studio-repair-20261031T183910`). **Only the Studio side remains, and opening/signing in is
Cayleb's act.** Steps 0.1–0.3 below are therefore already satisfied — do not repeat them.
Companion file: `shared-repair-procedure.md` (rationale + §10 execution record). Evidence sheet: `studio-evidence-sheet.md`.

## 0 · BEFORE he opens Studio (5 min, agent, no Studio) — **0.1–0.3 DONE**

| # | Do | Pass looks like | State |
|---|---|---|---|
| 0.1 | `cp -p ~/.codex/config.toml ~/.codex/config.toml.pre-studio-repair-$(date +%Y%m%dT%H%M%S)`; record its sha256 | backup exists, sha recorded | **DONE** — `…pre-studio-repair-20261031T183910`, sha `e10f08e4…` |
| 0.2 | Delete **only** line 192 (`enabled = false …`) under `[mcp_servers.roblox_studio]` | `grep -n enabled ~/.codex/config.toml` shows none under that block; `diff` vs backup shows exactly the one deleted line | **DONE** — after sha `752a7a2f…`, diff = 1 removed line, TOML parses, 7 servers intact |
| 0.3 | `flatpak ps` · `ss -ltnp \| grep 13469` · `pgrep -af StudioMCP` | all empty (or reaped with Cayleb's yes) | **DONE at 18:39 ET** — 0 Studio, 0 listeners, 0 StudioMCP (re-check at the window) |
| 0.4 | `python3 tools/hostcheck.py preflight` (street repo) — **before** any timed run | `quiet: true`, exit 0 | to do in-window |
| 0.5 | `python3 tools/vinegar_run.py status` | exit 0, no latch | to do in-window |

## 1 · Handoff to Cayleb (his hands, ~2 min — say exactly this)

1. "Open **Studio** (Vinegar). If it asks you to sign in, sign in as **Chrono_H3X**; if it asks for a password or 2FA,
   stop and tell me."
2. "Open the place I name — street game: the running copy of `build/test.rbxlx`; card game: the candidate
   `…/UntitledAnimeCardGame.rbxlx` (sha `55cef5d3…`)."
3. "Park the window, leave it in **Edit** mode (not Play), and say *go*."
   → **One Studio only, ever.** Two Studios revoked the login on 2026-10-16.

## 2 · The window itself (order is the procedure)

| Order | Step | Record |
|---|---|---|
| 1 | Verify the route: `~/.local/bin/roblox-studio-mcp` answers (initialize + tools/list) | raw response bytes → `board/probes/team-d/mcp-handshake-<stamp>.txt` |
| 2 | Read Studio's **own** UI: version string + signed-in account | screenshot + text |
| 3 | **Street game first** — native cells per `SPRINTS.md` (max **3 Plays** per Studio process) | per-Play exit code + JSON result + `run` stderr |
| 4 | `python3 tools/vinegar_run.py verify-clean` | exit 0 (proves place clean, no helper, no flags) |
| 5 | **Card game second** — only if the upload/bind round has produced real asset ids; otherwise skip | 24 binding shots + 8 cue records |
| 6 | `verify-clean` again if the card game ran | exit 0 |
| 7 | Tell Cayleb: "close Studio — **Don't Save**; then run `hyprctl reload`" | he confirms |

**Stop rules inside the window:** same step fails twice → stop; exit 1 = a real finding (keep it, do **not** re-run to
pass); exit 2 = preflight refused (fix the stated reason, nothing changed in Studio); exit 4 = host-contaminated
(label it); exit 5 = crash (owner step 9: re-copy the place, reopen, `verify-clean`). Never kill his jobs to get a
quiet host. Never launch a second Studio. Never press Play outside the runner.

## 3 · AFTER he closes (5 min, agent)

- Revert nothing unless the route failed: the config line **stays deleted** (the repair is for both games).
- Write the repair record: config sha256 before/after, the diff line, handshake file, Studio version + account,
  per-cell replay, `verify-clean` exit, and what the card game did or skipped.
- If anything failed: restore `config.toml.pre-studio-repair-*` and say so — a failed repair is reported, not hidden.

## 4 · One-screen truth table

| Thing | Value / answer |
|---|---|
| Studio | Vinegar 1.9.4 flatpak app, Studio 0.745, `…/versions/version-7062aec9dbe74b14/StudioMCP.exe` present |
| The repair | delete **one** line, `~/.codex/config.toml:192` (`enabled = false …`); never restore the wholesale backup (it differs by `model_reasoning_effort` too) |
| Games order | street (Job 16) first — 51-Play plan written, live private place; card game (Job 17) second — needs real asset ids first |
| Ceiling per window | 3 Plays per Studio process, then he reopens; one Studio per login |
| What closes the window | `verify-clean` exit 0 → **Don't Save** → `hyprctl reload` |
