# Team A — reel intake vetting: Kinetics · OriginKit · React Bits (Nox 103100Z)

**STATUS: INTAKE VETTING FILED — LICENCE/PROVENANCE/PINNING RULINGS + REFERENCE NOTES ONLY. NOTHING INSTALLED, VENDORED,
ADMITTED OR PURCHASED; NO LIVE BUILD TOUCHED.** Prepared by `ergon-lead` (Team A) 2026-11-02 ~16:20Z from the owner's reel
"Component libraries to avoid AI slop" (Nox commission 103100Z: Kinetics CSS motion effects ~99, OriginKit animated
components for Framer/React/AI tools with an MCP surface, React Bits animated component collection). Same treatment as the
2026-11-01 UI-library annex: **licence line + evidence URL, provenance/pinning, TAKE/LEAVE, what (if anything) we adapt.**
The parts-bin annex additions are filed in `team-a-design-pack-ui-library-parts-bin-annex.md` §10. **$0; no image reads; no
installs; no install into any live build; nothing unvetted enters the pack.**

**Frozen context:** the Lumen set stays frozen at `board/probes/team-a-design-v2/` digest
**`2a8cc21bd5f516198ea82a97efc40d3ddff1f7820309fd33b2175ae946670dbe`** (17 files); this intake changes nothing in it — no
token, no comp, no pack file. Presented copy unchanged (owner grade pending).

## 1. Verdicts (headline first)

| # | Library | Licence line of record (evidence) | Verdict | What we adapt |
|---|---|---|---|---|
| **K1** | **Kinetics** — `kinetics.colorion.co`; repo `github.com/ckissi/kinetics` | **NO LICENCE FOUND — all rights reserved by default.** No `LICENSE`/`LICENSE.md`/`COPYING`/`licence.txt`/`license` in the repo (raw = HTTP 404 ×5); GitHub API licence `null`; `package.json` `"private": true`; site `/terms`, `/license`, `/about` = HTTP 404; no licence text on the gallery pages. Repo: 629★, pushed 2026-10-30, owner **Csaba Kissi (Elerion ltd)**. Site sells a "production version" waitlist (Pro). | **NOT USABLE — reference only, no code.** No licence means no copying, no adapting snippets, no vendoring, no install. | **Reference notes only** (concept-level): spring-physics vs fixed easing; effect-category checklist; their own adaptation caveat. Nothing code-level. |
| **K2** | **React Bits** — `reactbits.dev`; repo `github.com/DavidHDev/react-bits` | **MIT + Commons Clause v1.0** — `LICENSE.md` "Copyright (c) 2026 David Haz"; permits use/modify/distribute "as part of an application, website, or product"; **Commons Clause: no sell/sublicense/redistribute the components themselves — alone, in a bundle, or as a ported version** (same for tools/sounds). Licence file sha256 `4bdd1fa59a6b7d5954bc3b213dbc34e455c635a94bbcecf9293553797d0fad02`; commit `4dc7b5b7772e1217c8fb8d2a7aa074302180e5b4` (2026-10-31T00:02:37Z); 51,065★. | **IN-APP ONLY** — allowed inside the Ergon app / per-company tenant source (notice retained); **never** the substance of a distributed design/component pack, and **never a ported-version redistribution**. Pro (paid) barred. | **Reference notes only for the pack** (no code): Micro-category state completeness, text-reveal discipline, card/nav patterns. |
| **K3** | **OriginKit** — `originkit.dev` (Framer/React/MCP; 250+ animated components + page sections) | **Bespoke usage licence, not open source** — `originkit.dev/docs/licensing` + `/docs/terms` (fetched 2026-11-02): commercial use in your own projects/apps/"products you sell"; modify freely; unlimited projects; client work OK; open-source apps OK if not redistribution of the catalog; **no attribution required**; AI/MCP use for own development + shipping agent-assembled apps. **Prohibited:** include components in **templates/themes/starter kits**, distribute as **design assets/UI kits/Figma files**, **redistribute the library**, resell components, mirror/cache the catalog for redistribution, expose the catalog via your own API/product/agent, train a competing library, authorship/brand claims. Pricing: Free (limited: 3 copies/day) / Pro $79/yr / Ultimate $99/yr — **paid tiers barred by the purchase fence**. | **IN-APP ONLY, and NOT into the design pack** (packs/templates/design assets are explicitly barred). Components embedded in the Ergon *built product* are licence-clean; the pack is not. **MCP plugin** `github.com/vellum-ai/originkit` is **MIT** (Vellum, Inc.; commit `d27526a2e2e728fd2c0c13b28021ade6908211a7`; licence sha256 `2418c77f…`) — that is *tooling* code, it does not change the library licence. | **Reference notes only**: MCP component-discovery workflow (registry pattern), state/category checklist, Framer/React parity. |

**One-line ruling:** *Kinetics = no licence → reference only; React Bits = MIT+Commons → in-app only, no pack/ported redistribution; OriginKit = usage-only → in-app only, nothing in the pack or as design assets; none is vendored and nothing is installed.*

## 2. Provenance and pinning

| Library | Provenance | Pin (record only) | Distribution route | Notes |
|---|---|---|---|---|
| **Kinetics** | Owner **Csaba Kissi** (Elerion ltd), repo `ckissi/kinetics`; site `kinetics.colorion.co` ("CSS & React Spring Animation Examples" — 162 animated UI examples, 99 React examples, 153 AI prompts per site meta) | commit **`c20739e345bed373ad29d7a3bc2203c470aa9650`** (2026-10-30T17:08:45Z, "Add collapsible Colorion sidebar and contain the Pro banner"); repo pushed 2026-10-30; **no licence file to pin** | Gallery site with "copy code" snippets + Pro waitlist | Pinning a commit is for *record of what was studied*, not for reuse (no licence) |
| **React Bits** | **David Haz** (`DavidHDev`), `react-bits.dev`; 51,065★, pushed 2026-10-31 | commit **`4dc7b5b7772e1217c8fb8d2a7aa074302180e5b4`**; licence sha256 **`4bdd1fa5…`**; free library counts: Text Animations 33 · Animations 38 · Components 48 · Micro 34 · Backgrounds 63 | shadcn/jsrepo registry (`npx shadcn add @react-bits/<Name>-TS-TW`); MCP registry config `@react-bits: https://reactbits.dev/r/{name}.json` (`/get-started/mcp`) | ⚠️ npm `react-bits@1.0.5` is an **unrelated 2022 package** — never cite/use it as React Bits. Pro is paid → unmade. |
| **OriginKit** | `originkit.dev` (Vellum, Inc. per MIT plugin); hosted library, no public component repo found; API-served with free API key (10 fetches/day per plugin README) | no library version is published; **pin = licence pages retrieval 2026-11-02** + terms URL; **MCP plugin** commit `d27526a2…`, MIT licence sha256 `2418c77f…` | Framer/React copies; MCP server for agents; **free API key required to fetch source** | Paid plans ($79/$99 per year) **unmade** under the purchase fence |

## 3. TAKE / LEAVE (written against the Lumen pack + unslop gate)

**TAKE — reference notes only (no code; see §4 for the mapping):**
- **Spring-physics thinking (Kinetics):** motion with weight/follow-through vs fixed easing — useful as a *review lens*, not a token change (Lumen's floor is one ease-out curve + 120/200/250/350; a spring variant would be a deliberate motion-token proposal, not silently adopted).
- **Adaptation caveat as a checklist item (Kinetics' own README):** gallery snippets are not all standalone; "check keyboard behavior, focus states, and reduced-motion support when adapting them" — folded into the component-admission review checklist.
- **Micro-state completeness (React Bits `Micro`, 34):** buttons/toggles/indicators as a completeness reference for the all-states matrix (we already require default/hover/focus/pressed/disabled/loading/error; compare, don't copy).
- **Text-reveal discipline (React Bits Text Animations, 33):** one-shot, budgeted reveals; respect `prefers-reduced-motion`.
- **Component/nav/card patterns (React Bits Components, 48):** structure reference only.
- **MCP component-discovery workflow (OriginKit MCP):** the *pattern* of an agent browsing/fetching vetted components from a registry — relevant to our design-pack tooling and the fleet face; the catalogue itself may not be mirrored/cached.

**LEAVE — the unslop tells and licence-barred material:**
- All **code/snippets** from Kinetics (no licence) and all **ported or redistributed** React Bits/OriginKit components (Commons Clause / usage licence).
- Every **background/visual pack** (gradients, particles, shaders, metallic/liquid/chrome), cursor trails/magnetic cursors, click sparks, marquee, shimmer/scramble/typewriter gimmicks, games, prism/parallax — the exact tells the unslop gate removes.
- Anything requiring **purchase** (React Bits Pro, OriginKit Pro/Ultimate) — unmade.
- Any workflow that **mirrors or caches** the Originkit catalogue (explicitly prohibited), and any use as **design assets/templates/starter kits** (prohibited).

## 4. Reference notes adopted (motion/component material for a finished Lumen surface — NOT installs)

These are notes for the design/component lanes; they change nothing in the frozen set.

| Ref | Note | Lumen mapping |
|---|---|---|
| **R1** | Spring physics is a different curve family. Lumen keeps **one ease-out curve** and the 120/200/250/350 budgets; any spring proposal must be a named motion-token proposal with reduced-motion floor and fresh-checker pass. | `--ease`, `--motion-tap/swap/enter/sheet` — unchanged |
| **R2** | Press feedback ≈ **120 ms**, hover/selection ≈ **200 ms**, content arrival ≈ **250 ms**, sheet ≈ **350 ms** — matches what the reel libraries demonstrate at their best; treat any effect needing longer as suspect. | `--motion-tap/swap/enter/sheet` — confirmation only |
| **R3** | All-states matrix completeness: default · hover · focus-visible · pressed · disabled · loading · error · empty · reduced-motion · High Contrast — borrowed as a **review checklist** for every admission row (S1–S8). | `team-a-component-layer-admission-skeleton.md` fields |
| **R4** | Keyboard/focus/reduced-motion check before any adaptation (Kinetics' own caveat) — include screen-reader name + focus-visible ring evidence. | C-A11Y + focus floors |
| **R5** | Text reveals are **one-shot** and budgeted; no looping typewriters/scrambles; no gradient text. | unslop + C-MOTION |
| **R6** | MCP-fetchable component registries are a legitimate tooling pattern (React Bits shadcn MCP registry; OriginKit MCP server); we may adopt the **workflow** (agent browses a vetted registry) but never mirror/cache a third-party catalogue. | fleet-face/tooling note, not a pack change |
| **R7** | Framer/React parity (OriginKit) is a useful *two-engine* completeness idea for the two Ergon faces; reference only. | design-pack anatomy note |
| **R8** | Their own licensing fences differ per library — the intake records them, so no one later "just copies a snippet" from a library without a licence (Kinetics) or with a Commons Clause (React Bits/OriginKit-class). | annex §10 |

## 5. What happens next (queued, not done)

- **Nothing is admitted.** These libraries add no component to the pack: K1 has no licence; K2/K3 are in-app only and barred
  from design packs. If we ever want an in-app component from K2/K3, it goes through the normal curation gate with the
  licence line stated and the pack boundary checked (never as a distributed/ported pack).
- The **reference notes** above are available to the design/component lanes; applying any of them to a built surface is a
  normal build/gate action, not a pack edit.
- If the owner's grade brings **polish deltas on the final set**, they go through the same intake → re-freeze → checker path
  (graded deltas only) — this intake does not touch the frozen set.

## 6. Fences held

No purchases (React Bits Pro, OriginKit Pro/Ultimate unmade); no installs; no code vendored or copied; no image reads; no
live build touched; no pack admission; **$0**; no owner contact. Evidence is primary-source HTTP retrieval + GitHub API
metadata, with the URLs and pin hashes above. **Filed for the Lead to route to Nox** (Team A does not contact Nox directly).
