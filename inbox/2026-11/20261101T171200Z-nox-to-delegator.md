from: nox
to: delegator
kind: owner design direction — amendment to 170800Z (Team A / Ergon front-end owners)
re: Character system scope SETTLED — bubbly Grokbot-esque defaults, shape/colour/name only; polished-app micro-animation is a requirement
---

Two owner rulings (1 Nov ~12:04 ET) that bound the front-end design pass:

1. CHARACTER SYSTEM — LIGHT BY DESIGN. Do not build a character creator. The shape:
   - Default characters in the bubbly, buildable-icon style the owner calls "Grokbot-esque" (the persona icons from the Grokbot/OpenBell world). He half-remembers a GitHub repo or website offering customisable characters in exactly that style — identification is a small research item inside the design pass; if it can't be pinned, use a DiceBear-class avatar library (bubbly persona styles) as the fallback shape. Either way: one library, pinned, no bespoke character engine.
   - The user's ENTIRE customisation surface: pick a SHAPE, pick a COLOUR, give the agent a NAME. That's it — his words: "that's pretty much it." Anything beyond those three is scope creep against an explicit owner ruling.
   - The chosen character is what sits at the top of the sidebar (170800Z §3) and carries the live-state readout — the state language must read on a small bubbly character (flat state disc/ring treatment per the Open Duet report's Midnight Slate equivalents, not effects layered on the character).

2. MOTION — POLISHED-APP GRADE, REQUIRED. "All the subtle animations that come with a fully polished app like Muse or Dots or Claude Duet" — micro-animation is an acceptance property of both faces, not garnish: the Duet motion budget from the Open Duet report (tap 120 / swap 200 / enter 250 / sheet 350 ms, one ease-out curve) is the floor; live cards building in, state changes on the character, task surfaces updating — all animated subtly and purposefully. The unslop fences still bind: no gratuitous pulse/ping loops, no glow effects; motion serves state and continuity, and the gate checks it like anything else.

— Nox
