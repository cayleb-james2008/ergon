# ForgeGUI MCP — local setup for the Roblox lane (1 Nov 2026)

Requested by Cayleb: "For any roblox control needed stuff use the roblox studio mcp + i have a forgegui api key
i can send you at anytime, get it setup, i only got about 100000 credits for the roblox team to work with."

## What ForgeGUI is / how it connects
- ForgeGUI (https://forgegui.com) is an AI game-asset generator (game UI, icons, thumbnails, 3D/GLB, classic clothing).
- Its MCP endpoint: **https://forgegui.com/mcp** (Streamable HTTP, POST; GET -> 405; unauthenticated -> 401
  "A valid ForgeGUI MCP API key is required", `WWW-Authenticate: Bearer`). Auth = `Authorization: Bearer <key>`.
- The API key is created at forgegui.com → **Profile → MCP API keys**, with per-scope keys:
  `library:read` (free discovery), `generation:read` (job status), `generation:write` (paid generation — spends
  credits), `publication:*` (publishing; NOT needed for our work), `runs:*`.
- **Roblox Studio MCP is separate** and already live (roblox-studio-mcp, 33 tools) — Studio control/insertion stays
  on that connection; ForgeGUI generates assets only.

## How it is wired here (DSH)
- Local stdio bridge (ForgeGUI's own reviewed 70-line bridge, `bridge.mjs`, SDK @modelcontextprotocol/sdk 1.31.0)
  at `~/Work/tools/forgegui-mcp/` with a launcher (`launch.mjs`) that reads the API key at startup from
  `~/.config/forgegui/mcp-key` (0600, created by Cayleb; never in chat/config/args/logs).
- DSH profile row added to the personal profile's `cordis.patch.yml`:
  `- id: forgegui-mcp / name: "@deepseek-ai/dsh-mcp-client" / transport: stdio / serverName: forgegui /
  command: /usr/bin/node / args: ["/home/cayleb/Work/tools/forgegui-mcp/launch.mjs"] / failOnStartupError: false /
  toolCallTimeoutMs: 180000`. Backup of the patch file kept as `cordis.patch.yml.bak-<ts>`; YAML validated.
- Activation: restart the DSH desktop app (MCP servers mount at boot). If the key file is missing the launcher
  exits 1 and the app still boots (failOnStartupError=false); tools then appear as `mcp__forgegui__*`.

## Key handling (binding)
- The key is Cayleb-entered only: created in his ForgeGUI account, saved with a no-echo command to the 0600 file.
  It is never printed, never in chat/board/commands/history/logs/screenshots/archives, never read back by an agent.
- Scopes requested for the planned work: `library:read` + `generation:read` + `generation:write`. No publication scopes.
- Revocation: forgegui.com → Profile → MCP API keys (revoke) and delete the local key file.

## Budget / spend envelope
- Cayleb's stated envelope: **~100,000 ForgeGUI credits for the Roblox team** (prepaid credits; no new purchase).
- Generation spends credits per asset; different generation types cost different amounts. The Roblox lane must:
  use the free `library_search` first; record the per-job credit cost and a running total; stop at the envelope;
  no purchase/top-up without Cayleb; no `outcome_unknown` automatic retries (check existing job status first).
- Publication is NOT part of this setup: assets come back as files/URLs and are imported through Studio/Open Cloud
  routes we already control.

## Verification plan (free, no credits)
1. Restart app → confirm exactly one `forgegui` MCP server loads and its tools are listed.
2. Free `library_search` (query "tree") — proves authentication without spending credits.
3. Only then: a bounded generation test under the envelope, with a stable request_id, job_id recorded, status
   polled to terminal, artefact verified before any Studio import.

## Supply-chain note
- Source reviewed: ForgeGUI's desktop bridge (70 lines) + launcher written locally; endpoint pinned to
  https://forgegui.com/mcp. The GitHub repo declares **no licence** — treated as internal, non-redistributable
  local tooling; no code is vendored into the fleet plugin.

## RULING (Cayleb, 1 Nov 12:05 ET) — no foreground input on his desktop
Cayleb reported that Team D's computer-use was overlapping his own clicking on the DSH window (hyprctl
send_key_state + clipboard paste steal focus). Ruling in force:
- Host desktop: NO global keyboard/mouse injection, NO clipboard paste, NO wtype/xdotool/hyprctl sends, NO host
  screenshots. These interfere with his work and are a hard stop.
- Roblox control = Roblox Studio MCP ONLY (33 tools enumerated; background, no focus steal). `user_mouse_input`/
  `user_keyboard_input` are Studio-internal and allowed only inside a running playtest, never to drive the host.
- No `screen_capture`/image reads (DSH host image-decode crash class).
- Opening a .rbxlx place: no MCP tool opens a place file — the OWNER opens it once (File→Open in the already-open
  Studio), then the team drives via MCP. No Studio window launches over his work.
- Anything that genuinely needs GUI automation goes to an isolated workspace (agent-workspace-linux), never the host.

## VERIFIED END-TO-END (12:20 ET, before any app restart)
Ran the bridge standalone with the owner's key file (the key is read inside the launcher; never printed):
- initialize OK; tools/list = **41 tools**, including `generation_gui`, `generation_image`, `generation_model_3d`,
  `generation_rig_3d`, `generation_remesh_3d`, `generation_animate_3d`, `generation_sound_effect`,
  `generation_music`, `remove_background`, `enhance_prompt`, `drawing_reference`, `auto_separate`,
  `generation_status`, `generation_retry`, `animation_actions`, `project_create`, `project_list`,
  `personal_assets_search`, `publication_*` (NOT to be used: publication stays unauthorised).
- Free call `library_search("tree")` → `{"items":[],"search_status":"complete"}` — authentication accepted
  (no 401), zero credits spent.
- Remaining step for in-app availability: restart the DSH app so the MCP row mounts (tools appear as
  `mcp__forgegui__*`). The row is in the profile patch; the launcher reads the key file at startup.
- Budget: hold at 0 credits per the approved Roblox asset plan (Jobs 16/17 need no new assets); one free
  `library_search` inventory batch authorised when live; any generation must be a bounded, per-asset-costed run
  inside the ~100,000-credit envelope and explicitly authorised first.

## ACTIVATION FIX + LIVE (12:41 ET)
- The first config attempt used a top-level `- id: forgegui-mcp` patch entry; the loader's patch format requires an
  **`insert:` list** for new plugins (`- insert: [{id, name, config}]`), so it was silently ignored (per-entry warning
  only). Fixed to the insert-list form; YAML validated; backup `cordis.patch.yml.bak2-<ts>` kept.
- The app's HMR picked the change up WITHOUT a restart: plugin list now shows `include:forgegui-mcp` /
  `@deepseek-ai/dsh-mcp-client` / enabled / fiberPhase=active / patchId=forgegui-mcp, and the bridge process
  `/usr/bin/node .../forgegui-mcp/launch.mjs` runs as a child of the app.
- Tool names will be `mcp__forgegui__*`; first verification step is the free `library_search("tree")` (no credits),
  then the single free inventory batch authorised for the Roblox lane. Spend stays at 0 credits.
