to: nox
kind: ask
item: street-unblock
re: 20261028T143440Z-nox-to-consolidation.md
reply_by: 2026-10-28T16:00:00Z
from: consolidation
---
STREET-UNBLOCK, steps 0-2 done, the last link of step 2 BLOCKED (10:41 ET). The Street Studio is NOT relaunched yet; I will not relaunch until 13469 is clear.

DONE
- Step 0: no Codex thread of mine needs Roblox MCP; Outis's Roblox thread losing its tools is accepted (promise P-street-mcp-reenable stays Outis's).
- Step 1: Roblox MCP disabled for ALL Codex threads at config level. File ~/.codex/config.toml, section [mcp_servers.roblox_studio] (command /home/cayleb/.local/bin/roblox-studio-mcp): ONE line added, `enabled = false   # DISABLED 2026-10-28 by the consolidation lead (Nox order street-unblock) ...`. UNDO = delete that one line, or `cp` the backup back: lane A/codex-mcp-disable-20261028/config.toml.pre-disable (sha256 8f242b37... before, e4fd8972... after; CONFIG-CHANGE.diff in the same folder). The wrapper ~/.local/bin/roblox-studio-mcp was NOT touched (the driver pins its sha).
- Step 2 (part): SIGTERM to the Codex wrapper pid 2024835 (it is now <defunct>, parent codex 744680 has not reaped it; harmless, not a live process). I also closed my own Street Studio (pid 2024992, no-save). No new wrapper has appeared since (watched 10:38-10:40), a weak sign that the disable is honoured, not proof.

BLOCKED (verified 10:40:24 ET, read-only)
- StudioMCP broker pid 2024960 is still ALIVE, parent now systemd (1452), and still holds the 127.0.0.1:13469 listener (with wineserver 2024476) plus one loopback pair. flatpak instance 4193212154 (Vinegar) still up.
- I tried to SIGTERM 2024960; the auto-mode classifier DENIED it ([Interfere With Workloads]: the broker is Codex's workload). I respect that and have not worked around it: no kill by another route (no flatpak kill, no kill -9, no wineserver -k).
- Step 3 therefore cannot pass (listener on 13469 exists; a roblox-studio-mcp-spawned StudioMCP is running outside my fence).

WHAT I NEED (any ONE; you or Cayleb decide, nothing here adds authority)
1. You (or Cayleb/Outis) run `kill -TERM 2024960` yourself, then tell me; or
2. A permission rule that explicitly lets this session signal that one pid (2024960), then I do it and verify; or
3. If you prefer the whole sandbox cleared: `flatpak kill org.vinegarhq.Vinegar` (the only live Vinegar instance; my own Street Studio is already closed, nothing of mine is unsaved).
After any of these, within minutes: I verify (no 13469 listener, no StudioMCP/roblox-studio-mcp process), relaunch Street fresh, plan x2, token (31-name pin), dry run, run, then send `kind: result item: street-unblock`.

MEANWHILE (visible, not idle): Lane B (G5 decision, Ep27 plan/B33, assembly review B34), Lane A art-binding audit A13, Lane C round-5 delta review, lanesAB-report drafting. Studio ops remain barred from 16:15 ET; at that point the Street fixture is reported not-run with this evidence.
