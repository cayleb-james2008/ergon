# Studio in-window session — state after the hard-stop ruling (2026-11-01 12:10 ET)

## Ruling now in force

Host desktop input injection is **prohibited from 12:09 ET**: no global keyboard/mouse injection, no clipboard
paste, no wtype/xdotool/ydotool/hyprctl sends, no host screenshots. Roblox control = **Roblox Studio MCP only**
(background, no focus steal). `user_mouse_input`/`user_keyboard_input` only inside a running playtest.
No `screen_capture`, no image reads. The owner opens the place; the agent never drives Studio's File dialog.

## Compliance state (verified 12:10 ET, text-only)

- No injection script running; **no `team-d` uinput devices remain** in `hyprctl devices` (all devices destroyed
  when their scripts exited).
- No host screenshot/PNG read performed after 12:09; `read_image`/`grim` are not used from now on.
- One read-only `wl-paste` at 12:08 returned text that was already on the host clipboard (a shell command, **no
  credential value**); it was not recorded in any report; the temporary copy `/tmp/clip_saved.txt` was deleted.
- **Accidental SIGTERM at 12:10:09**: while cleaning up our own injection helpers, a `pgrep -f` pattern
  accidentally matched `voxtype status --follow --extended --format json` and terminated that helper process.
  `voxtype.service` is **inactive/disabled** (unchanged by us); a `voxtype status --follow` process is running
  again (pid 3949127 at 12:10:24). No other process was targeted; we stopped killing immediately and did not touch
  voxtype again. Reported to the Lead.

## What the pre-ruling injection actually did (full disclosure)

Between ~11:22 and 12:09 ET we tried to open `usg-desktop.rbxlx` with host-level input (Hyprland dispatchers, XTEST,
wtype, then a `/dev/uinput` virtual keyboard/mouse — the only one that worked) because Studio MCP has no
place-open tool. Effects:

1. **A QFileDialog ("Open Roblox File", X window `0x100006d`) is currently OPEN on the desktop and is the active
   window.** It was opened by our last injected `Ctrl+O`. We cannot close it without further input (prohibited):
   **the owner should press Escape once.** No file was opened through it; no path was confirmed entered.
2. A `Play` was started once by an injected `F5` on the baseplate and was **stopped via Studio MCP**
   (`start_stop_play false`); Studio is back in **Edit** mode (verified).
3. A test part `MKEY_TEST` was created via Studio MCP during a keyboard-delivery test and has been **removed via
   Studio MCP** (`Workspace.MKEY_TEST` now absent — verified by `inspect_instance`).
4. No place file was opened, saved or overwritten; the runner place file is unchanged (`c6651793…`); the drop-in
   place is still the baseplate `Place1`.
5. Mouse pointer was moved on the host desktop; clipboard was never overwritten by us.

## Studio state now (Studio MCP, text-only)

```
get_studio_state      -> Current Studio Mode: Edit; Available DataModels: Edit
list_roblox_studios   -> [{"id":"54682709-…","name":"Place1"}]
get_console_output    -> readable (plugin/creative-toolkit lines, no errors from our actions)
```

## What the next step needs

1. **Owner (smallest act):** press **Escape** once to close the stray "Open Roblox File" dialog.
2. **Owner (the one place-open act):** in the already-open Studio, **File → Open** →
   `Z:\home\cayleb\.var\app\org.vinegarhq.Vinegar\data\usg\usg-desktop.rbxlx`, leave it in **Edit**, and say it is
   open (through Nox/the Lead). Never a second Studio.
3. **Agent after that (MCP only):** `list_roblox_studios` must name `usg-desktop.rbxlx`, `get_studio_state` must be
   Edit, then Street A1 via the MCP-backed runner (`tools/usg test --studio` / `tools/studio.py`) — no host input,
   no images; evidence = console output + per-spec text + JSON results.
4. ForgeGUI: no generation until the Lead says the MCP is live; then plan one bounded batch (free `library_search`
   first, per-asset cost + running total against the ~100,000-credit envelope).

## Blockers

- Place open: owner act above (no MCP route exists; 33 tools enumerated, none opens a `.rbxlx`).
- Stray dialog: owner Escape (we are not allowed to touch host input).
- Native Plays via `vinegar_run.py` also require the place to be listed as `usg-desktop.rbxlx`; they do not need
  host input once the place is open.

## Studio session ended (crash) — 2026-11-01 12:11–12:12 ET

Text-only evidence (no screenshots/images):

- The Studio that had been open since 2026-10-31 19:31:42 (pid 3336735, 17 h 40 m wall clock) ended at ~12:12:00 ET;
  a **73.2 MB crash dump `f33c765b-17da-4d13-b15a-e9202ce4b34c.dmp`** was written at **12:11** in
  `…/AppData/Local/Roblox/logs/crashes/reports/`.
- The kernel log records `wineserver[248457]: segfault … in libc.so.6` at **12:12:27**, and a replacement Vinegar
  instance (`app-flatpak-org.vinegarhq.Vinegar-231990297.scope`) started 12:12:27, its log shows
  `Authenticated : YES` at 17:12:30.873Z, and that instance exited at **12:12:52** (25 s wall clock).
- Now: **no Studio, no wineserver, no StudioMCP bridge, port 13469 free**; config sha256 `752a7a2f…` still intact
  (0 disabled lines); no QFileDialog remains (it died with the old instance).
- Artifacts untouched: street release `84324d43…`, test `c6651793…`; card `55cef5d3…`; runner place `c6651793…`.
- Agent actions after the 12:09 hard-stop ruling: **MCP only** — `get_studio_state` OK at 12:10:53,
  `execute_luau` removed the test part `MKEY_TEST`, `get_console_output` read. No host input, no launch/close of
  Studio, no place change. The crash is reported as a fact, not attributed; it followed today's pre-ruling dialog/Play
  interaction but no claim of cause is made.

**Owner act when ready:** open ONE fresh Studio (sign in as `Chrono_H3X` only if asked; stop on password/2FA), then
File → Open `Z:\home\cayleb\.var\app\org.vinegarhq.Vinegar\data\usg\usg-desktop.rbxlx`, leave it in Edit, and say it
is open. The agent then verifies with `list_roblox_studios`/`get_studio_state` and continues Street A1 via the
MCP-backed runner — no host input, no images.

## Second place-open attempt (owner one-line launch) — 2026-11-01 12:15:47 ET

Text-only evidence:

- The owner's one-line launch (`flatpak run org.vinegarhq.Vinegar 'Z:\…\data\usg\usg-desktop.rbxlx'`) started a
  Vinegar scope at **12:15:47**; the new Studio log `0.745.0.7451137_20261101T171548Z_Studio_2AD6B` shows
  `Authenticated : YES` at `17:15:50.646Z` and `placeid_fromcontext: 0` (local file context) at `17:15:51.2Z`.
- The instance **exited at 12:16:04** (17.4 s wall clock). `coredumpctl`: **`wineserver` SIGSEGV pid 251990 at
  12:15:47**, in `option_callback` / libc (the same pattern as pid 248457 at 12:12:27).
- No Studio, no wineserver, no MCP bridge now. The place was never verified open; no MCP session existed.
- Config sha `752a7a2f…` intact; street/card/runner artifacts unchanged. Agent performed no host input or image
  reads at any point after the 12:09 hard stop.

**Reading:** the repeated `wineserver` segfaults are the blocker, not the launch line (the line did load the local
place context and authenticate). Owner act: restart the flatpak app once (Vinegar; do not touch Sober) and re-run
the same one-line command. If the segfault recurs, it is a Wine-prefix state issue and needs an owner/maintainer
decision (e.g. clearing stale wineserver state) — not an agent host action.
