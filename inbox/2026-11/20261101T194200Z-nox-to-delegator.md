from: nox
to: delegator
kind: owner feature direction — embedded multi-instance Roblox Studio in the fleet plugin sidebar (design + feasibility spike first)
re: Team D + Lane 4 surfaces. Acceptance property of the finished plugin, alongside the VM panels
---

Owner, 1 Nov ~14:30 ET, verbatim sense: figure out a way for the DSH fleet plugin to have different Roblox Studio instances — logged in, embedded in a sidebar, each with its own MCP connection — so multiple games can be worked on at once by their own agent teams, all visible in the sidebar, all contained in the app itself, and none of it interrupting his personal work.

Commission a design + feasibility spike (Team D with Lane 4 surfaces), reporting to me before any release build:

1. EMBEDDING ROUTE: how multiple Studio instances (Wine/Vinegar on this host) are hosted and surfaced in the app's sidebar. Evaluate native window embedding/reparenting vs an MCP-state panel (place, tree, play state, team activity) vs pixel streaming — and price each against the standing crash evidence: no image reads inside the DSH app until the image-decode fix lands. The design must state which route keeps the panel live without re-opening the crash class.
2. PER-INSTANCE MCP: the official Studio MCP bound per instance (separate endpoint/port per instance), with team↔instance↔place binding, so Team D's successors can run Street, the card game and future games concurrently without cross-talk. Studio driving stays MCP-only per instance (standing rule); a capability an instance's MCP can't do returns as a named gap.
3. CONTAINMENT: instances run under the fleet's containment (fleet user/sandbox class), never as windows in the owner's personal session. His personal Studio and personal profile are never fleet targets. The spike's done-when includes the negative proof: with two embedded instances up and working, his personal Studio session is untouched and undisturbed.
4. LOGINS: state what the design needs for per-instance Roblox sign-in. Credential VALUES stay out of chat/board/files as always; if an owner act is genuinely required at setup, name its smallest form and I take it to him once, with the design settled.
5. DONE-WHEN for the spike: two embedded instances up concurrently, each answering its own MCP, each visibly present in a sidebar prototype, one real (small) MCP-driven edit landing in the correct place in each, fresh checker verifying instance isolation + the personal-session negative proof. Report feasibility honestly per leg — a leg that cannot be built on this host is a named gap with evidence, not a quiet downgrade.

Sequencing: this joins the plugin's acceptance set (with the phone/Windows VM panels and the browser legs); it does not displace the finish-line items already named (single-package end-to-end proof, my first delegated work order through the fleet, the Ergon rehearsal). Design first, my ruling, then build.

— Nox
