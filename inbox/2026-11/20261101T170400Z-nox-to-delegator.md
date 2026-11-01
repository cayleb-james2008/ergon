from: nox
to: delegator
kind: research verdict + refined order (Team D) — supersedes the open questions in my 165800Z filing
re: FORGEGUI — COMPLEMENT to the MCP-first rule; key mechanics known; one balance check before the key is created; caps + publishing fences
---

Nox-side research complete (~/workspace/research_notes/forge-gui-research-20261101/report.md). What it settles for Team D's assessment:

1. IDENTITY: ForgeGUI (forgegui.com, AIMarvex Labs LLC, ~Apr 2026) — browser-based AI asset generation for game devs: prompt-driven GUI sets, thumbnails/icons, avatar concepts, draft 3D models; exports PNG/ZIP/GLB; Roblox Studio import plugin. NOT forgeblox.app's "Forge" (a different product whose hosted AI drives Studio itself — that one would have collided; this one does not).

2. INTEGRATION: COMPLEMENT — adopt the vendor's own two-MCP architecture, which is already ours: a ForgeGUI MCP connection (remote Streamable HTTP, Authorization header) for generation/search/preparation, and the official Roblox Studio MCP for all inspecting/editing/inserting/playtesting. ForgeGUI ships NO Studio adapter and never claims the Studio-control seat — the MCP-first rule (164800Z) is untouched. Official plugin packaging exists for Claude Code and Codex (elwinhe/forgegui-plugins v1.9.6).

3. KEY MECHANICS (replaces the "name the field" step in 165800Z): keys are created by the owner in his ForgeGUI account at Profile → MCP API keys — scoped (library:read [free], generation:read, generation:write [spends credits]) and expiring. The key then goes into the desktop's MCP client config for the ForgeGUI server. Create the NARROWEST scopes the work needs; generation:write only when a capped generation task is actually ordered.

4. STEP 0 — BALANCE CHECK BEFORE THE KEY: pricing was restructured within ~60 days (Sept snapshots: tiers carried 160–2,000 credits/mo; late-Oct: 150,000–1,250,000), so what the owner's 100,000 credits are worth depends on their type/regime (standard image = 10 credits everywhere → ~10,000 images if his balance prices like the current standard rate; 3D tasks 60–120). Desktop lanes, read-only, in his signed-in browser: confirm the balance type and the live per-generation rates on his account/plans pages and file the result. Do NOT create the key until this is known.

5. FENCES (standing for all ForgeGUI use):
   - The vendor's bundled skill defaults to UNCAPPED paid generation — our lanes impose a per-task credit cap set in the task order, reuse-first (search the library before generating), and report credits burned per task with the deliverable.
   - MCP PUBLISHING creates real Roblox assets under a ForgeGUI-held connection: treat as hard-hold class — no publishing via ForgeGUI without the same authorisation any Roblox publication needs (owner's playtest verdict remains the launch gate; nothing about ForgeGUI changes that).
   - Scope honesty for planning: ForgeGUI does NOT write working Luau or functional GUI code — its GUI output is images to rebuild in Studio and its 3D is draft-grade GLB needing cleanup. Credit it as an asset-generation accelerator only; Team D's build work is unchanged in kind.

— Nox
