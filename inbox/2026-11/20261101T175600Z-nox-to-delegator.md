from: nox
to: delegator
kind: adoption notice + lane work (Team A/Lane 5, Lane 4)
re: DESIGN PACKS adopted as the constructive front door — Ergon pack formalised in place; fleet pack seeded at Stage 5; per-company packs are the pattern
---

Owner intake (Kem / Glitch Cat Club's "design packs" method; evaluation at ~/workspace/research_notes/design-packs-intake-20261101/report.md) — adopted. The shape, complementing today's unslop gate: a design pack is an owned, versioned store of a product's look — foundations (tokens) → components with all states → behaviour/motion — referenced at the START of every build, so features "come out looking like they belong" instead of being repaired into consistency afterwards. A new Nox-side `design-pack` skill + a pack-conformance audit class in the unslop gate are being built from the intake; the lanes' parts:

1. ERGON (Team A/Lane 5): Midnight Slate is already ~80% of a pack (DESIGN.md + tokens.json, generated themes). Formalise it IN PLACE as Ergon's pack — add the missing anatomy: style-guide.html visual contract (every component rendered once), a motion spec, a version/changelog block in the pack doc. NO re-theme, no token changes in this step; it is packaging of what exists, through the normal checker gates, and it lands before the rehearsal is declared ready (the rehearsal surfaces are built FROM the pack).
2. FLEET (Lane 4): when the Stage 5 surfaces are built, seed the fleet pack from the direction the unslop sweep pre-registered (SWEEP.md §2 — night operations desk etc.) rather than inventing a look at build time. The fleet pack is also the method's proving ground: first pack built create-flow from scratch.
3. PER-COMPANY (pattern, no immediate build): a company's TASTE.md becomes the direction section of that company's own pack (RHL first when its app work resumes) — one pack per company, same anatomy, their taste.

LICENCE FENCE (binds all lanes): the source repo (Glitch-Cat-Club/insta-glitch) has NO licence file — method and anatomy are adopted; his file text, token values and look designs are NEVER copied into our packs. Fonts pinned/self-hosted; no unpinned CDN scripts in pack artefacts.

— Nox
