# D-1 — Studio route: exact live state (verified 2026-11-01 08:39–09:05 ET)

Host `outis-desktop` (Linux 7.2.8-9-omarchy, Hyprland/Wayland), user `cayleb`. Tier A, `$0`. The team did **not**
open, close or alter Studio this batch; the MCP handshake and `preflight` each spawned and reaped their own
`StudioMCP.exe` child (recorded below). Nothing in Studio changed.

## 1 · Verdict in one paragraph

The Studio route is **LIVE and verified end-to-end**, not merely repaired on the config side: the config line is
still deleted and parses, the pinned 0.745 install is intact, the launcher answers MCP `initialize` + `tools/list`
(**33 tools**), the bridge listens on `127.0.0.1:13469`, and the running Studio answers `list_roblox_studios`
→ `Place1` and `get_studio_state` → **Edit**. The reason nothing has run natively yet is **not** the route: the
Studio that is open has a Roblox baseplate (`Place1`) loaded, while the project's runner requires exactly one
listed place named `usg-desktop.rbxlx`. The runner's place copy has now been refreshed to the sealed candidate's
test place (`c6651793…`), so **one owner act remains**: open that file in the already-open Studio. Everything else
is prepared.

Two facts that were not known before this batch and that change the window ask:

- **Studio is already open and signed in** — it has been open since **2026-10-31 19:31:42 ET** (14 h at probe
  time), launched through `gtk-launch org.vinegarhq.Vinegar.desktop`; its log shows `Authenticated : YES` at
  `2026-10-31T23:31:52.175Z`, user scope entered, WebView2 profile userId **1170918805** (= `Chrono_H3X`), and **no
  logout line since**. It has a baseplate open and **0 Plays** used.
- **The team did not open it and does not know who did.** The launch was a GUI launch (`gtk-launch` via
  `uwsm_app-daemon`), not an agent route. A **Sober** (Roblox player) flatpak session is also running, started
  **08:26:10 ET today**, focused on ws 4 — not ours. Lead/Nox must confirm with Cayleb that the open Studio is his
  session and may be driven before the team sends any tool call beyond the read-only probes above.

## 2 · Evidence (commands + results, as run)

### 2.1 Config repair — intact

```
$ sha256sum ~/.codex/config.toml
752a7a2f0e80183f6679efa332930af7fca2d492bd9a6529f35ebcd40eff3ac8   (10,583 B, mode 600)
$ grep -c "enabled = false   # DISABLED 2026-10-28" ~/.codex/config.toml
0
$ python3 - <<'PY'  (tomllib)
PARSE_OK top_keys=17 servers=7
roblox_studio keys: ['command']   command = /home/cayleb/.local/bin/roblox-studio-mcp
server names: blender,chrome-devtools,free_3d_assets,linux_background,linux_desktop,node_repl,roblox_studio
$ ls -la ~/.codex/config.toml.pre-studio-repair-*
-rw------- 10843 Oct 31 16:09  ~/.codex/config.toml.pre-studio-repair-20261031T183910
sha256 e10f08e43f6ed5601d8cee48d9ab8c549bbd158ab9d2dcf6f5e9782995cff56c   (= the before-state)
```

Fact recorded: `config.toml` **mtime is 2026-11-01 08:14:27 ET** but its sha256 is byte-identical to the 31 Oct
repair state. Content unchanged; something touched the file (the Codex tree was also active 08:13–08:29). Not an
incident, but it must not be "fixed" by restoring a backup — the repair state is the correct state.

### 2.2 Install, versions and launcher pin — intact

```
$ flatpak list --columns=application,version,branch,origin,installation | grep -iE "vinegar|sober"
org.vinegarhq.Sober    1.8.0  stable  flathub  system
org.vinegarhq.Vinegar  1.9.4  stable  flathub  user
$ ls ~/.var/app/org.vinegarhq.Vinegar/data/vinegar/versions/
version-7062aec9dbe74b14            # the ONLY version folder (Oct 27 18:30) — no auto-update since
$ ls -la …/version-7062aec9dbe74b14/*.exe
RobloxStudioBeta.exe 222,384,592 · StudioMCP.exe 8,224,720 · NativeDialog.exe · RobloxCrashHandler.exe
$ head -2 …/prefixes/studio/drive_c/users/cayleb/AppData/Local/Roblox/mcp.bat
if exist "Z:\…\versions\version-7062aec9dbe74b14\StudioMCP.exe" ( … %* ) else ( reg query ContentFolder … )
$ ~/.local/bin/roblox-studio-mcp     # launcher probe, 08:39 ET
Error: connection closed: initialize request      # it reached the Studio bridge; no MCP client conversation followed
```

The launcher's pin is **correct**: the version folder it names exists. No `mcp.bat` repair is needed on this
machine (that repair belongs to the laptop, which is unreachable — see GROUND-STATE M4).

### 2.3 Studio log — authenticated, still live

Newest log: `0.745.0.7451137_20261031T233146Z_Studio_783AF_last.log` (6,242,225 B at 08:40 ET, **still growing**:
6,260,706 → 6,261,529 B in 3 s at 08:41:36–39 ET).

```
216: 2026-10-31T23:31:48.990Z … [FLog::StudioTimingLog] Authenticated : NO
594: 2026-10-31T23:31:52.175Z … [FLog::StudioTimingLog] Authenticated : YES
240: 2026-10-31T23:31:49.319Z … [FLog::LoginController] AuthTokenRefresh, Retrieved new access token …
255: 2026-10-31T23:31:49.474Z … [FLog::LoginController] Login got Standalone DM ready to enter User scope
260: 2026-10-31T23:31:49.484Z … [FLog::LoginController] Login got user info, fetching AB test config async
271: 2026-10-31T23:31:49.623Z … [FLog::LoginController] Login ready to enter User scope
307: 2026-10-31T23:31:49.744Z … [FLog::StudioEmbeddedBrowserWebView2] WebView2 created with user profile ID: 1170918805
671: 2026-10-31T23:37:23.501Z … [FLog::StudioPublishStatusLabel] Requesting publish-status for placeId 95206881
No "Exiting user scope" / logout line after 23:31:52Z.
```

`placeId 95206881` is Roblox's baseplate template (a cloud place), not either game. The log also carries
17,018 lines mentioning port 13469 and 5,682 `WebSocketTraceError` lines — this is Studio's built-in MCP client
retrying `http://localhost:13469/studio` every ~3 s while no bridge is running. It is log noise, not a fault; it
stops the moment an MCP bridge runs (proven in 2.4).

### 2.4 MCP route — proven, end to end (read-only tools only)

Raw bytes saved: `board/probes/team-d/mcp-handshake-20261101T084546.txt` (initialize + tools/list, 33 tools) and
`board/probes/team-d/mcp-connect-wait-20261101T084841.txt` (Studio identity).

```
initialize OK
ss @1s: LISTEN 0 128 127.0.0.1:13469 0.0.0.0:*  users:(("wineserver",…),("main",…))
poll0 (t=1s): {"studios":[{"id":"54682709-1f04-454a-81e0-7c20099a5101","name":"Place1"}]}
get_studio_state -> - Current Studio Mode: Edit
                     - Available DataModels: Edit
                     - Focused DataModel in the viewport: Edit
```

The first probe's `tools/list` also returns the standard Roblox Studio tool set (including `upload_image`,
`execute_luau`, `start_stop_play`, `screen_capture`) — those are **not** used by the project runner; the runner's
allowlist is six tools (`vinegar_lib/mcp.py: ALLOWED`). No Play was started, no Luau was executed, no image was
uploaded. The first probe attempt (08:45:46) saw an empty studio list because it asked before Studio reconnected;
the second (08:48:41) waited and got the instance — record both, do not treat the first as a failure.

### 2.5 Preflight before/after the place refresh

```
$ python3 tools/hostcheck.py preflight
{"quiet": true, "load1": 3.0, "load_per_core": 0.094, "reasons": []}   exit 0
$ python3 tools/vinegar_run.py status
{"latch": null, "lock": "free", "studio": {"pid": 3336735, "plays": 0, …}, "studioMcp": [],
 "mcpPort": {"port": 13469, "listening": false, …}, "recentRuns": []}   exit 0
# BEFORE refresh (08:54 ET): place usg-desktop.rbxlx = 74f6673d… (Oct 16, stale) ≠ build/test.rbxlx c6651793…
$ python3 tools/vinegar_run.py preflight      # exit 2, refusals:
  need exactly one listed studio named usg-desktop.rbxlx; listed: ['Place1']
  advisories: usg-desktop.rbxlx differs from build/test.rbxlx: re-copy it after the fresh build before the owner opens Studio
# AFTER refresh (08:59 ET): cp -p build/test.rbxlx → ~/.var/…/data/usg/usg-desktop.rbxlx (sha now c6651793…)
$ python3 tools/vinegar_run.py preflight      # exit 2, ONE refusal left:
  need exactly one listed studio named usg-desktop.rbxlx; listed: ['Place1']
  advisories: []                              place.matchesBuild: true
```

`hostcheck` quiet; latch free; Studio pid 3336735, **0 Plays**; `StudioMCP` reaped after each probe; port 13469
free when no bridge runs. The place the runner will open is now byte-identical to the sealed candidate's test
place.

## 3 · The owner's one mechanical act (and the proof it worked)

Give Cayleb only this:

> Studio is already open and signed in on your desktop. In that same Studio window, choose **File → Open**, pick
> `Z:\home\cayleb\.var\app\org.vinegarhq.Vinegar\data\usg\usg-desktop.rbxlx`, leave it in **Edit** (choose
> **Don't Save** if asked), and say **go**. If Studio was closed, open Studio first and sign in as **Chrono_H3X**
> only if it asks — if it asks for a password or 2FA, stop and tell me. Never open a second Studio.

Proof the act worked (all must be recorded in-window):

1. `vinegar_run.py preflight` exits **0** with `place.matchesBuild: true` and no refusal (this is the exact check
   that currently fails on `listed: ['Place1']`).
2. MCP `list_roblox_studios` names exactly **`usg-desktop.rbxlx`**;
   `get_studio_state` says **Edit**.
3. The first `vinegar_run.py queue …` Play exits **0** and its run directory records the place sha256
   **`c6651793…`**.
4. Studio's own UI shows the account **Chrono_H3X** and version **0.745.0.7451137** (screenshot in-window; the
   session/account lines above are the log-level evidence until the screenshot exists).

## 4 · Risks and facts the Lead must rule on

| # | Fact | Why it matters | Recommendation |
|---|---|---|---|
| R1 | Studio has been open 14 h, signed in, with a baseplate; **launcher unknown** (GUI launch 31 Oct 19:31:42 ET); a Sober player session started 08:26 ET today | The team's standing rule is "the owner opens Studio; the agent never does". Using a Studio someone else opened could disturb them, and a 14 h session may be stale | Lead/Nox: one-line confirm with Cayleb that it is his session and may be driven; if he says no, he closes it and opens a fresh one — same one-step instruction |
| R2 | `config.toml` mtime moved to 08:14 today with identical bytes; Codex tree active 08:13–08:29 | Another harness is alive on the box; a naive "restore backup" would revert the repair | Do not restore; record; the repair state stands (already the standing ruling) |
| R3 | The old window protocol says "owner opens Studio" | Reality: Studio is open; the owner's act is now "open the place file" | Use D-5 run-sheet; keep the "one Studio only, never a second launch" rule |
| R4 | Sign-in is proven at startup (23:31:52Z) not *now* | A 14 h session could in principle have expired | In-window proof = Studio UI screenshot + first Play; stop and report if a fresh credential demand appears (never handle it) |

## 5 · What D-1 did NOT do

No Studio launch/close, no Play, no Luau execution, no place edit, no upload, no network request carrying a
credential, no repo change, no money. The only writes: `usg-desktop.rbxlx` refreshed from the sealed candidate
(runner step 1), the two probe evidence files, and this report. `$0`.
