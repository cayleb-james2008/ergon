# Annex — vetted UI-library parts bin (design packs)

**Status: VETTED ANNEX — PROPOSALS ONLY. NOTHING VENDORED, NOTHING PURCHASED, NOTHING ADMITTED.** Annex to the
Nox-side `design-pack` skill (skill not present on this seat; filed here as the board artefact to travel with it).
Commissioned by Nox `20261101T194800Z` (owner's reel `DfNBVUroUTO`, "5 UI libraries that make your site look
expensive"); fenced by "no purchases; nothing vendored into any repo until its licence line in this annex says clean
for that use; source URL + licence evidence per library". Prepared by Team A (`ergon-lead`, Lane 5), 2026-11-01.
Primary sources were fetched on 2026-11-01 between 19:52Z and 20:10Z; every claim below names its URL.

**Intake note (honest):** Nox's intake report `~/workspace/research_notes/ui-libraries-reel-intake-20261101/report.md`
is **not reachable from this seat** — the directory does not exist here (checked 2026-11-01 19:5xZ, and again during the
commission). The reel's five names and their descriptions are taken from Nox's commission text; all licence findings are
from the primary sources named below, not from the intake file. If the intake file names a different fork/owner for any
library, that is a Nox-side correction.

## 1. Licence rulings — headline first

| # | Library | Primary licence (evidence) | Per-company repo shipping | Standalone pack distribution | Pattern reuse | Verdict line |
|---|---|---|---|---|---|---|
| 1 | **Vengeance UI** | **MIT** — `github.com/Ashutoshx7/VengeanceUI/blob/main/LICENSE` ("MIT License Copyright (c) 2025-2026 Ashutoshx7 …"), GitHub API licence `mit` | **CLEAN** (vendor with MIT notice) | **CLEAN** (MIT permits redistribution; keep notice) | CLEAN | **CLEAN for all uses**, subject to the pack gate |
| 2 | **Animate UI** | **MIT + Commons Clause** — `github.com/imskyleen/animate-ui/blob/main/LICENSE.md` ("distribute … as part of an application, website, or product"; Commons Clause: "do not sell or redistribute the components themselves in their original form—whether alone or in a bundle") | **CLEAN as part of the application/product**, licence + copyright notice retained; the repo may hold the app source, **not** a redistributable component collection | **NOT CLEAN** — a standalone components pack/bundle in original form is redistribution the Commons Clause bars | CLEAN | **CLEAN in-app; NOT clean as a redistributed components pack** |
| 3 | **Uiverse** | **MIT per element** — site footer: "All content (UI elements) on this site are published under the MIT License" (`uiverse.io/elements`); ToS §29: user content licensed "Specifically, the MIT license." | **CLEAN** with attribution (author + element URL) and MIT notice | CLEAN (MIT) | CLEAN | **CLEAN**, user-submitted caveat below |
| 4 | **Forge UI** (`forgeui.in`, **not** Roblox ForgeGUI) | Free subset: bespoke usage-only licence — `github.com/AmanShakya0018/forgeui/blob/main/LICENSE` ("Permission is granted to use, copy, and modify … for personal and commercial projects, including client work and production applications. You may NOT: Repackage, redistribute, or resell …; Upload this code or derivatives to other repositories … intended for distribution … This license grants usage rights, not redistribution rights."). Pro: paid licence, `forgeui.in/termsofservice` ("may not redistribute, resell, share, or sublicense ForgeUI Pro, even with modifications") | **NOT CLEAN for vendoring** (usage rights only; no redistribution, no upload to distribution-intended repos) | **NOT CLEAN** | CLEAN (pattern study; clean-room reimplementation only) | **Patterns only; Pro unmade (purchase fence)** |
| 5 | **Uilora** | **Commercial/premium, not open source** — `uilora.com/terms` (effective 2026-05-03): non-exclusive, non-transferable licence to use/copy/modify/incorporate into applications; "You may not redistribute Uilora components as a standalone UI library or component pack — either free or paid — without our written consent." Pricing FAQ: "Free tier is limited to personal and open-source projects"; "Pro gives you an unlimited commercial license" | **NOT CLEAN for our commercial use on the free tier**; Pro/Ultimate are paid plans and the purchase fence blocks them | **NOT CLEAN** (redistribution as a pack needs written consent) | CLEAN (shape/pattern study only) | **Not usable without a purchase we must not make; patterns only** |

**One-line ruling:** *vendor only #1 and #3; use #2 in-app but never as a redistributed pack; #4 and #5 are pattern-study
only, and neither free nor paid #4/#5 may be bought.*

## 2. Licence evidence per library (primary sources, fetched 2026-11-01)

### 2.1 Vengeance UI — MIT (confirmed)
- Licence file: https://raw.githubusercontent.com/Ashutoshx7/VengeanceUI/main/LICENSE — "MIT License Copyright (c) 2025-2026 Ashutoshx7 … Permission is hereby granted, free of charge, to any person obtaining a copy … to deal in the Software without restriction, including … the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell …"
- GitHub API https://api.github.com/repos/Ashutoshx7/VengeanceUI — `"license": {"key":"mit","spdx_id":"MIT"}`; `pushed_at 2026-10-21T15:24:07Z`; 1,247 stars; not archived. Maintenance: recent (11 days before fetch).
- Repo description (README): modern animated components for landing pages; "Copy. Paste. Customize."; React/TS/Tailwind/Framer Motion.
- **Ruling: CLEAN** for repo vendoring and pack distribution with the MIT notice retained. Still passes the pack gate.

### 2.2 Animate UI — MIT + Commons Clause; in-app clean, pack-redistribution barred; maintenance ACTIVE (audit flag contradicted)
- Licence file: https://raw.githubusercontent.com/imskyleen/animate-ui/main/LICENSE.md — "MIT + Commons Clause License Condition Copyright (c) 2025 Elliot Sutton … distribute the Software **as part of an application, website, or product** … ## Commons Clause Restriction … so long as you do not sell or redistribute the components themselves in their original form—whether alone or in a bundle."
- README repeats the reading: https://raw.githubusercontent.com/imskyleen/animate-ui/main/README.md — "use it in any application, website or product, commercial or not, but don't sell or redistribute the components themselves."
- Repository: https://github.com/imskyleen/animate-ui (owner `imskyleen`, not an `animate-ui` org); GitHub API licence metadata reports `"key":"other","spdx_id":"NOASSERTION"` because the Commons Clause is not an SPDX-standard licence — **the repo's LICENSE.md text governs**.
- **Maintenance status:** API `pushed_at 2026-10-27T09:57:52Z`; commits API latest five are 2026-10-27 (`dec9850` "fix: release 2.4.1 …", `b037f85` "chore: release 2.4.0", `f377120` docs, `5819d11` feat bottom navigation, `44a1718` fix docs); created 2025-03-26; 4,815 stars; not archived. **The audit flag "dormant since Dec 2025" is contradicted by primary evidence: the repo was active on 2026-10-27, five days before this annex.** Record the disagreement, do not repeat the flag.
- **Ruling (precise):** the Commons Clause bars *selling or redistributing the components themselves, in original form, alone or in a bundle*. Shipping restyled components **as part of a company's application/product** (inside its app source, deployed as that product) is permitted, commercial use included, with the notice retained. A **design pack that is distributed as a components collection** — a public package, a marketplace-style artefact, or a pack handed to another company as a component library — is a "bundle" of components and is **NOT clean**. Therefore: Animate UI source may live in a per-company **application** repo; it must not be the substance of a pack artefact that is distributed as components. If a future pack is ever distributed as a standalone library, drop Animate UI from it or get written permission.
- Maintenance risk: active on 2026-10-27; no dormancy finding to carry.

### 2.3 Uiverse — MIT per element
- Site-wide statement (footer of every page fetched, e.g. https://uiverse.io/elements): "Uiverse | The universe of UI **MIT License** All content (UI elements) on this site are published under the MIT License." The same footer appears on element pages, e.g. https://uiverse.io/Yaya12085/quick-husky-74, which also shows the author's "MIT License" line.
- Terms of Service (linked from the site; hosted at Termly): https://app.termly.io/policy-viewer/policy.html?policyUUID=bdc0e7cf-cb88-4f24-9cb5-13761e20dedb — §29 CONTENT: submitting content grants Uiverse and other users a worldwide, non-exclusive, royalty-free licence "… Specifically, the MIT license."
- **Caveat (user-submitted):** the site relies on uploaders warranting originality ("Content is yours … and/or you have the right to use it"); there is **no per-element licence metadata field** beyond the author and the site-wide MIT statement — a component copied by an uploader from elsewhere would carry a defective provenance, and the site disclaims responsibility. Mitigation: record per-element author + URL + fetch date in the pack provenance; re-check any element whose origin looks derivative before admission.
- **Ruling: CLEAN** with attribution (author + element URL) and the MIT notice retained; 4,508 elements, many are glow/3D/spectacle — the fences in §3 remove most.

### 2.4 Forge UI (forgeui.in) — free subset usage-only; Pro paid and redistribution-barred
- Free repo licence: https://raw.githubusercontent.com/AmanShakya0018/forgeui/main/LICENSE — "Copyright (c) 2025 Aman Shakya All rights reserved. Permission is granted to use, copy, and modify this code for personal and commercial projects, including client work and production applications. You may NOT: - Repackage, redistribute, or resell this code or any modified version of it as a UI library, component collection, template, starter, or similar product. - Upload this code or derivatives to other repositories, marketplaces, or platforms intended for distribution (free or paid). - Claim this code or derived components as your own work. This license grants usage rights, not redistribution rights."
- Pro terms: https://forgeui.in/termsofservice — "Buying ForgeUI Pro grants you a non-exclusive, non-transferable licence … You may not redistribute, resell, share, or sublicense ForgeUI Pro, even with modifications. You may not publish ForgeUI Pro as open source…"; pricing on https://forgeui.in — $169/$99 one-time → **a purchase, therefore unmade under the fence**.
- **NOT the Roblox ForgeGUI**: this is a React/Tailwind/shadcn component site (forgeui.in, GitHub AmanShakya0018/forgeui). The Roblox asset tool used elsewhere in this programme is a different product entirely (MCP ForgeGUI / forgegui.com); no relationship, no shared licence, no confusion in the packs.
- **Ruling:** free subset is **NOT clean for vendoring or redistribution** ("usage rights, not redistribution rights"; no upload to distribution-intended repositories). It is clean only for **pattern study** with clean-room reimplementation. Pro is excluded twice over (purchase fence + redistribution prohibition).

### 2.5 Uilora — premium, not open source; commercial terms read
- Terms: https://www.uilora.com/terms (§1 License Grant): "Uilora grants you a non-exclusive, non-transferable, worldwide license to use, copy, modify, and incorporate Uilora components into your own applications … covers both personal and commercial use … You may not redistribute Uilora components as a standalone UI library or component pack — either free or paid — without our written consent. Premium-tier components … are licensed per the plan you subscribe to; free-tier components are available under this license at no charge."
- Pricing FAQ: https://www.uilora.com/pricing — "Can I use these in client projects? Pro gives you an unlimited commercial license … Free tier is limited to personal and open-source projects." Plans are paid (Pro/Ultimate) → **purchase fence: unmade**.
- **Undecidable/conflicting:** §1 says free-tier components are available under the licence at no charge while the pricing FAQ limits the free tier to personal/open-source. For our **commercial product** the safe reading is that the free tier does not cover us and the paid tier is barred by the purchase fence. **Ruling: not usable**; patterns only.
- **The interesting shape (why it is in the bin at all):** one library, two faces — Next.js web (`/get-started/web/**`) and React Native mobile (`/get-started/mobile/**`); mobile platform notes cover iOS/Android shadows-vs-elevation, blur fallbacks, UI-thread animation, 120 Hz and haptics (https://www.uilora.com/get-started/mobile/platform-notes). That one-library/two-surface anatomy is the reference for Ergon's two faces, and it is a **shape study only**.

## 3. Per-library fences — what we take, what we leave (against the packs' own rules)

Rules the fences are written against: **Midnight Slate** (`docs/DESIGN.md` + `design/tokens.json`: dark-only; one indigo
accent; no gradients, glass, shine, reflection or drop shadows; depth only by `base → panel → raise`; tokens only, no
hand-typed hex; one accent forward action per view; 44 DIP targets, 56 DIP employee buttons; 2 px accent focus ring);
**unslop gate** (Nox `20261101T154200Z` §1: purple→blue gradients, gradient text, glows, emoji-as-icons, sparkle badges,
800-weight headings, uppercase tracked labels, off-grid spacing, greeting headlines, hype copy, glowing pills, rainbow
charts, `transition-all`, pulse/ping loops; upstream `github.com/heydamir/unslop`, CC BY 4.0, credit carried); **Duet
motion budget** (tap 120 / swap 200 / enter 250 / sheet 350 ms, one ease-out; no loop/glow; reduced-motion and High
Contrast off).

| Library | TAKE (interaction pattern / component class) | LEAVE (collides with the rules) |
|---|---|---|
| Vengeance UI | heading/entry reveal as a single 250 ms enter; shared-element nav transition 200 ms; section layouts as composition reference | gradient text/backgrounds, aurora/glow meshes, particles, marquee, 3D tilt, spectacle hovers, looping animation hero demos, `transition-all` |
| Animate UI | bottom-navigation minimise-to-icons pattern (mobile tab bar), accordion/collapsible primitives, carousel *mechanics* (embla), dialog/sheet enter-exit composition, copy-button + code-block pattern (fleet logs) | auto-advancing/looping carousels, attention-seeking demos, confetti/particles, glow or blur layers on chrome, motion outside the 120/200/250/350 budget, `transition-all` |
| Uiverse | switch/toggle, input/field focus treatment, checkbox/radio, tooltip, breadcrumb, table patterns — after restyle to tokens only | glow/3D/neon/neumorphism/gradient buttons, gradient text, shimmer text, animated loaders (loop tells), hover spectacle, arbitrary palettes, `transition-all` |
| Forge UI | animated form state progression, OTP input pattern, notification-stack pattern — **as clean-room patterns only, no code** | shimmer text, glow cards, gradient borders, spectacle hovers, `transition-all`, and every Pro component (unmade) |
| Uilora | two-face one-library anatomy (shape study), pricing/usage-table pattern, sheet/tab anatomy — patterns only | all backgrounds/gradient packs/blurred/pattern/noise, "Crazy Components" (parallax, distort, prism, trail), magnetic cursor, 3D transforms, haptics/glass references; no code |

## 4. Shortlist — proposed for the packs (curation gate: proposal → all-states → screenshot compare → fresh checker)

**Status of every row: PROPOSED ONLY.** No component is admitted; no code is vendored; nothing enters a pack until the
gate below passes and the licence line in §1 says clean for the use. "Ergon pack" = Midnight Slate pack (docs design
pack, Nox `20261101T175600Z` §1); "fleet pack" = Lane 4 fleet design pack (direction seeded from the unslop sweep;
SWEEP.md itself is Nox-side and not on this seat — rows marked **provisional**).

| # | Component (source) | Pack | Why it earns a slot | Restyle / fence notes |
|---|---|---|---|---|
| S1 | Bottom-navigation minimise-to-icons (Animate UI) | Ergon | Directly serves the Duet mobile shell: Chat/Tasks/Make tab bar that gives content room back | Flat tokens only; state disc not glow; swap 200 ms; labels never uppercase-tracked |
| S2 | Accordion / collapsible (Animate UI) | Ergon | Task detail + approvals expand rows; honest state without modals | 120/200 ms; `line` dividers only; focus ring 2 px |
| S3 | Carousel mechanics (Animate UI, embla) | Ergon | Make-artefact gallery without writing a slider | No autoplay; swap 200 ms; no dots glow; keyboard + reduced motion |
| S4 | Dialog/sheet composition (Animate UI) | Ergon | Approvals sheet; the 350 ms budget already matches the pack spec | Flat scrim token; no blur/glass; no sheet bounce |
| S5 | Heading/entry reveal pattern (Vengeance UI) | Ergon | Onboarding/empty-state heading that arrives once with purpose | Single 250 ms enter; no loop; no gradient text |
| S6 | Shared-element nav transition (Vengeance UI) | Ergon | Sidebar → chat "one click into the agent" feel | 200 ms; no `transition-all`; High Contrast off |
| S7 | Switch/toggle + field-focus pattern (Uiverse, after provenance check) | Ergon | Employee settings controls that read flat and stateful | Rebuild on tokens; 44 px targets; no glow track; no 3D |
| S8 | Input/field focus treatment (Uiverse) | Ergon | Forms feel alive without decoration | `fieldLine` → accent ring 2 px/2 px gap; no animated glow |
| S9 | Copy-button + code-block pattern (Animate UI) | Fleet (provisional) | Operator logs/diagnostics need copy + monospace blocks | Mono `Cascadia Mono`; tokens only; no rainbow syntax; copy feedback 120 ms |
| S10 | Tooltip pattern (Uiverse) | Fleet (provisional) | Status pills/queue help without clutter | Flat `raise` surface, hairline; 120 ms; no arrow glow |
| S11 | Notification-stack pattern (Forge UI — pattern only, clean-room) | Fleet (provisional) | Queue/alert stacking that reads like a night-desk, not a toy | No code; no shimmer; enter 250 ms; no pulse loop; danger token only for real severity |
| S12 | Usage/pricing-table pattern (Uilora — pattern only) | Fleet (provisional) | Operator console plan/usage display | No code; tables with `line` dividers; no gradient headers |

**Not shortlisted, deliberately:** every background/visual pack, "Crazy Components", magnetic cursor, parallax/scroll
reveals as spectacle, 3D transforms, shimmer text, marquee, confetti, glass/blur chrome, glow buttons and neon borders.
They are the tells the gate exists to remove.

## 5. Pack curation gate (how a proposal becomes an admission)

1. **Proposal** — filed in this annex (§4) with: library, version/commit, element id + author + URL, licence line from §1,
   fetch date, intended pack, restyle plan, and what is left out. No proposal, no admission.
2. **All-states check** — the proposed component rendered once per state: default, hover, focus, pressed, disabled,
   loading, error, empty, plus reduced-motion and Windows High Contrast variants. Missing state = fail.
3. **Screenshot compare** — against the pack's rendered reference on Midnight Slate tokens; differences named (tokens,
   spacing, type, motion), not "looks close".
4. **Fresh checker** — a checker who did not build the component (maker never checker) records the verdict; only PASS
   admits. The annex row is updated with the admission record; the component's provenance block carries the licence line
   and notice.
5. **Nothing vendored before step 4**, and then only for libraries whose §1 line is clean for that use.

## 6. Undecidable / to escalate (said plainly, not guessed)

- **Animate UI, pack distribution:** the Commons Clause text is clear for redistribution of the components "in their
  original form, alone or in a bundle"; the boundary between a restyled in-app component and a distributed pack is an
  interpretation. This annex rules: **in-app only, never the substance of a distributed pack**. If Nox ever wants an
  Animate-UI-derived pack distributed outside an application, that needs written permission — escalate, do not assume.
  This is an engineering-licence ruling, not legal advice.
- **Uiverse per-element provenance:** the site-wide MIT statement is the licence evidence; there is no per-element licence
  metadata, and user submissions are only warranted by uploaders. Mitigation is attribution + provenance re-check; a
  derivative-looking element is not admitted.
- **Uilora free-tier scope:** ToS §1 and the pricing FAQ conflict on whether the free tier is licence-wide or
  personal/open-source only. For our commercial product, treat as **not covered**; any different reading needs written
  consent.
- **Forge UI free/Pro boundary:** the free GitHub repo does not enumerate which site components are free vs Pro; only the
  Pro terms are explicit. Without a purchase (barred), the free subset cannot be fully enumerated → patterns only.
- **Nox intake report unreachable** this seat (`~/workspace/research_notes/ui-libraries-reel-intake-20261101/report.md`):
  if it identifies different owners/forks, correct this annex before any gate run.

## 8. Lead engineering rulings on the undecidables (recorded 2026-11-01)

The Lead ruled the four open items; these are the operating rules for this annex and all pack work. Where §1/§6 say
"interpretation", these rulings are the decision of record.

| # | Item | Ruling of record |
|---|---|---|
| R1 | **Animate UI boundary** | Usable **only as in-app components inside the Ergon app / a per-company tenant package**, licence + copyright notice retained. **Never** as a standalone redistributable components pack or bundle in original form. If we ever want to distribute a pack containing them, **written permission from the maintainers first — no exceptions.** |
| R2 | **Uiverse** | Admit an element only when its **provenance is re-checkable at admission time** (author + element URL recorded in the pack). Where provenance cannot be re-checked, **exclude it**. No bulk element dumps. |
| R3 | **Uilora** | **Not usable — shape/pattern study only**; no purchase, no redistribution. |
| R4 | **Forge UI** | **Pattern study only**; no code copied, no Pro purchase. |
| — | **Net rule** | Vendor only **Vengeance UI and Uiverse (with notices)**; **Animate UI in-app only**; **Forge UI and Uilora patterns only** — all through the pack gate in §5. |

Consequences already reflected above: §1 row 2 and §2.2 are qualified by R1 (in-app, never a distributed pack); §2.3 is
qualified by R2 (admission requires re-checkable provenance, no bulk dumps); §1 rows 4–5 and §2.4/§2.5 are pattern-only by
R3/R4. The stale "Animate UI dormant since Dec 2025" flag is not repeated anywhere and must not be reintroduced (primary
evidence: active commits 2026-10-27).

## 9. Fences held

No purchases (Forge UI Pro and Uilora plans unmade); no code vendored into any repo or pack; no pack admission; no
generation service used; $0 spend. This annex is documentation and proposals only.
