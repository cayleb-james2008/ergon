# Team A — vetted UI-library parts bin: SHORTLIST for the Ergon + fleet design packs

**STATUS: PROPOSAL ONLY. NOTHING ADMITTED, VENDORED, PURCHASED OR COMMITTED.** This is the item-3 deliverable of Nox
order `20261101T194800Z` (reel `DfNBVUroUTO`): the shortlist of components/patterns worth admitting into the **Ergon
Midnight Slate pack** and the **fleet pack**, each through the pack curation gate. It sits on top of the licence annex
`board/reports/team-a-design-pack-ui-library-parts-bin-annex.md` (primary-source licence verification + per-library
fences + Lead rulings R1–R4, filed 2026-11-01 20:20Z). Prepared by Team A (`ergon-lead`, Lane 5), 2026-11-01 ~23:48Z.
Spend **$0**; no purchases; no code copied into any repo or pack; no image reads.

Scope note: **both packs are downstream of the owner's grading of the design comps** (`board/probes/team-a-design-comps/`,
filed for OWNER GRADING). Component-layer admission is the next build step **after** that grading, not before. Fleet-pack
rows are marked **provisional**: the fleet pack's SWEEP.md is Nox-side and was not readable on this seat.

## 1. Licence lines (headline first — every row inherits its library's line)

| # | Library | Licence line of record | Evidence URL (primary source) | What may be used |
|---|---|---|---|---|
| L1 | **Vengeance UI** | **MIT** — "Permission is hereby granted … to deal in the Software without restriction, including … use, copy, modify, merge, publish, distribute, sublicense, and/or sell" | https://raw.githubusercontent.com/Ashutoshx7/VengeanceUI/main/LICENSE · API: https://api.github.com/repos/Ashutoshx7/VengeanceUI (`"spdx_id":"MIT"`) | **CLEAN**: vendor into an app repo / ship in a pack, MIT notice retained |
| L2 | **Animate UI** | **MIT + Commons Clause** — "distribute … as part of an application, website, or product"; "do not sell or redistribute the components themselves in their original form—whether alone or in a bundle" | https://raw.githubusercontent.com/imskyleen/animate-ui/main/LICENSE.md · README: https://raw.githubusercontent.com/imskyleen/animate-ui/main/README.md | **In-app only** (Ergon app / per-company tenant package) with notice retained; **never** the substance of a distributed components pack/bundle — Lead ruling **R1** |
| L3 | **Uiverse** | **MIT per element** — site-wide: "All content (UI elements) on this site are published under the MIT License"; ToS §29: user content is licensed "Specifically, the MIT license." | https://uiverse.io/elements (footer) · ToS §29: https://app.termly.io/policy-viewer/policy.html?policyUUID=bdc0e7cf-cb88-4f24-9cb5-13761e20dedb | **CLEAN** with attribution (author + element URL) and MIT notice; provenance must be re-checkable at admission — ruling **R2** (no bulk dumps) |
| L4 | **Forge UI** (`forgeui.in`, **not** Roblox ForgeGUI) | **Bespoke usage-only licence** — "This license grants usage rights, not redistribution rights"; Pro terms bar redistribution/resale/sublicence | https://raw.githubusercontent.com/AmanShakya0018/forgeui/main/LICENSE · https://forgeui.in/termsofservice | **Pattern study only** — no code copied, no Pro purchase; clean-room reimplementation — ruling **R4** |
| L5 | **Uilora** | **Commercial/premium, not open source** — non-transferable licence to incorporate into applications; "may not redistribute Uilora components as a standalone UI library or component pack … without our written consent"; free tier "limited to personal and open-source projects" | https://www.uilora.com/terms · https://www.uilora.com/pricing · shape study: https://www.uilora.com/get-started/mobile/platform-notes | **Not usable** — shape/pattern study only; no purchase, no redistribution — ruling **R3** |

Re-verification this batch: the Vengeance UI and Animate UI licence files were re-fetched directly (HTTP 200) on
2026-11-01 ~23:44Z; their text is as quoted above. Uiverse/Uilora/Forge UI lines are the annex's primary-source findings
(2026-11-01 19:52–20:10Z); no change is claimed since. **Animate UI maintenance:** ACTIVE (commits 2026-10-27,
releases 2.4.0/2.4.1) — the "dormant since Dec 2025" audit flag is contradicted by primary evidence and must not be
repeated.

**Net rule (Lead rulings R1–R4, of record):** vendor only **L1 and L3 (with notices)**; **L2 in-app only**; **L4 and L5
patterns only**. Nothing is admitted until it passes §5's gate *and* its licence line is clean for the intended use.

## 2. The bars every row is judged against (the packs' own rules)

- **Midnight Slate** (`DESIGN.md`, `design/tokens.json`): dark-only; one indigo accent; **no gradients, glass, shine,
  reflection or drop shadows**; depth only `base → panel → raise`; tokens only, no hand-typed hex; **one accent forward
  action per view**; 44 DIP targets (56 DIP employee buttons); 2 px accent focus ring.
- **Unslop gate** (Nox `20261101T154200Z` §1; upstream `github.com/heydamir/unslop`, CC BY 4.0, credit carried):
  purple→blue gradients, gradient text, glows, emoji-as-icons, sparkle badges, 800-weight headings, uppercase tracked
  labels, off-grid spacing, greeting headlines, hype copy, glowing pills, rainbow charts, `transition-all`, pulse/ping loops.
- **Duet motion budget**: tap 120 / swap 200 / enter 250 / sheet 350 ms, one ease-out curve, no loop/glow; reduced-motion
  and Windows High Contrast respected.

A row that cannot be restated on tokens, inside the motion budget, without a glow/gradient/spectacle tell **fails at
proposal** and is not on this list.

## 3. The shortlist — 12 rows (PROPOSED ONLY; none admitted)

Legend: **Ergon** = Ergon Midnight Slate pack; **Fleet** = Lane 4 fleet design pack (**provisional** — SWEEP.md not on this
seat). Gate path is the same four steps for every row (§5); the "Gate state" column says where the row actually is:
**G1** proposal filed (this file) · **G2** all-states render · **G3** screenshot compare · **G4** fresh-checker verdict.

| # | Component / pattern (source) | Pack | Licence + evidence anchor | TAKE (interaction pattern) | LEAVE (aesthetics that collide) | Gate state |
|---|---|---|---|---|---|---|
| **S1** | Bottom-navigation minimise-to-icons (Animate UI) | Ergon | L2 in-app (R1); https://github.com/imskyleen/animate-ui | Chat/Tasks/Make tab bar that gives content room back; minimise = state, not decoration | state disc glow, uppercase tracked labels, looping attention cues | G1 only |
| **S2** | Accordion / collapsible (Animate UI) | Ergon | L2 in-app (R1) | Task-detail and approvals expand rows; honest state without a modal | glow open/close chrome, `transition-all`, bounce | G1 only |
| **S3** | Carousel mechanics (Animate UI / embla) | Ergon | L2 in-app (R1); embla MIT (upstream) | Make-artefact gallery without writing a slider; keyboard + reduced-motion aware | autoplay, looping, glowing dots, parallax cards | G1 only |
| **S4** | Dialog / sheet composition (Animate UI) | Ergon | L2 in-app (R1) | Approvals sheet; enter/exit fits the 350 ms sheet budget | glass/blur scrim, sheet bounce, drag-handle glow | G1 only |
| **S5** | Heading / entry reveal (Vengeance UI) | Ergon | L1 MIT; https://raw.githubusercontent.com/Ashutoshx7/VengeanceUI/main/LICENSE | Onboarding/empty-state heading arrives once, 250 ms, purposeful | gradient text, aurora meshes, particles, looping hero demo | G1 only |
| **S6** | Shared-element nav transition (Vengeance UI) | Ergon | L1 MIT | Sidebar → chat "one click into the agent" continuity at 200 ms | `transition-all`, tilt/3D stretch, spectacle hover | G1 only |
| **S7** | Switch/toggle + field-focus (Uiverse) | Ergon | L3 MIT per element; https://uiverse.io/elements | Employee settings controls that read flat and stateful; 44 px targets | glow track, neon/neumorphic button skins, 3D press | G1 only; **needs R2 provenance record** (author + element URL) |
| **S8** | Input/field focus treatment (Uiverse) | Ergon | L3 MIT per element (as S7) | Forms feel alive via focus ring, not decoration (`fieldLine` → 2 px accent ring) | animated glow, shimmer placeholder, gradient border | G1 only; R2 provenance record |
| **S9** | Copy-button + code-block (Animate UI) | Fleet (prov.) | L2 — **R1 boundary**: in-app only; a distributed components bundle is barred | Operator logs/diagnostics: copy + monospace block with 120 ms feedback | rainbow syntax themes, shimmer text, glass panel | G1 only; **blocked by R1 if the fleet pack is distributed as a components bundle** → clean-room pattern-only or drop |
| **S10** | Tooltip (Uiverse) | Fleet (prov.) | L3 MIT per element | Status pills/queue help without clutter; 120 ms, flat `raise` surface | arrow glow, drop shadow, gradient background | G1 only; R2 provenance record |
| **S11** | Notification-stack pattern (Forge UI) | Fleet (prov.) | L4 **patterns only** (R4); https://raw.githubusercontent.com/AmanShakya0018/forgeui/main/LICENSE | Queue/alert stacking that reads like a night desk; enter 250 ms; severity tokens only for real severity | shimmer, glow cards, gradient borders, pulse loop, "toast theatre" | G1 only; clean-room only (no code) |
| **S12** | Usage/pricing-table pattern (Uilora) | Fleet (prov.) | L5 **shape study only** (R3); https://www.uilora.com/terms | Operator console plan/usage display; `line` dividers, no gradient headers | gradient/pattern backgrounds, magnetic cursor, 3D transforms | G1 only; no code, no purchase |

**Nothing on this list may be dropped into a pack silently.** Every row starts at G1 and must run G2→G4 in order.

## 4. Why these twelve (and not more)

- The Ergon rows (S1–S8) are the interaction surfaces the Duet shell actually needs and that the merged pack currently
  lacks primitives for: tab-bar minimise, collapsible rows, a gallery without a bespoke slider, a sheet, one entry
  reveal, a nav continuity, and two flat form controls. Each can be restated on Midnight Slate tokens with no new
  aesthetic language.
- The fleet rows (S9–S12) are operator-console primitives (logs, tooltips, alert stacks, usage tables) — the night-desk
  shape — not marketing components.
- **Deliberately not shortlisted:** every background/visual pack, "Crazy Components", magnetic cursor, parallax/scroll
  spectacle, 3D transforms, shimmer text, marquee, confetti, glass/blur chrome, glow buttons, neon borders. They are the
  exact tells the unslop gate exists to remove.

## 5. The curation gate (how a proposal becomes an admission)

Same gate for both packs; **maker never checker** — the final step is a checker who did not build the component.

| Step | What happens | Evidence it must produce | Owner |
|---|---|---|---|
| **G1 Proposal** | This row, completed: library, version/commit, element id + author + URL, licence line from §1, fetch date, intended pack, restyle plan, explicit leave-list | The row in this file (and the annex) | Team A (done — §3) |
| **G2 All-states check** | Render the component once per state: default, hover, focus, pressed, disabled, loading, error, empty, + reduced-motion and Windows High Contrast variants. Missing state = fail | A per-state capture set; every state named | Design/build lane |
| **G3 Screenshot compare** | Compare against the pack's rendered reference on Midnight Slate tokens (and fleet tokens for fleet rows); differences named (tokens, spacing, type, motion), not "looks close" | Side-by-side evidence + named diff list | Design/build lane |
| **G4 Fresh checker** | A checker who did not build the component records PASS/FAIL; only PASS admits | Checker verdict on the board; admission record appended to this file | Lead spawns the checker |

Admission record (appended only after G4 PASS): `row · library@version · element id/author/URL · licence line · fetch
date · pack · restyle plan · leave-list · checker identity + verdict + date`. The component's provenance block carries
the licence line and notice.

**Nothing is vendored before G4**, and then only for libraries whose §1 line is clean for that use:
L1/L3 may ship with notice; L2 only inside the Ergon app / per-company tenant package (never as a distributed
components bundle); L4/L5 never ship code — patterns are reimplemented clean-room.

## 6. Sequence, ownership, and what blocks what

1. **Owner grades the design comps** (`board/probes/team-a-design-comps/`, feedback = graded deltas only). This is the
   standing dependency; nothing here starts before it.
2. **Component-layer admission** then begins for the Ergon pack with S1–S8, in that order (S1/S2/S4 first — the shell
   needs them), each through G2→G4. Rows S7/S8 additionally need the R2 provenance record before G2.
3. **Fleet pack rows (S9–S12)** wait for (a) the fleet pack's own definition (SWEEP.md is Nox-side; rows are
   provisional), and (b) the R1 boundary ruling for S9 if the fleet pack is distributed as a components bundle.
4. **No purchase is made at any point** (Forge UI Pro, Uilora Pro/Ultimate: unmade under the standing fence), and no
   public post/publication happens in this lane.

## 7. Fences held (this deliverable)

No purchases; **no code vendored into any repo or pack**; **no pack admission**; no generation service; no image reads;
no credentials; no sends/publication; **$0 spend**. This file and its annex are documentation and proposals only. The
"Animate UI dormant since Dec 2025" audit flag is not repeated anywhere (primary evidence: active 2026-10-27).
