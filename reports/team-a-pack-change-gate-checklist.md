# Team A — pack-change gate checklist: Lumen token supersession (Midnight Slate → Ergon Lumen)

**STATUS: GATE CHECKLIST — NOT A PACK CHANGE, NOT AN APPROVAL, NOTHING LANDED.** This is the ordered gate the Lumen v2 token
supersession must pass before **Midnight Slate stops being Ergon's front-end theme**. It is the companion to
`board/probes/team-a-design-v2/PACK-CHANGE-PROPOSAL.md` (the proposal itself) and
`board/reports/team-a-design-v2.md` (the evidence report). Prepared 2026-11-02T06:42Z by `ergon-lead` (Team A). No
presentation, no owner contact, no image reads, $0.

**Set under this gate:** `board/probes/team-a-design-v2/` — 17 content files; freeze digest
**`2a8cc21bd5f516198ea82a97efc40d3ddff1f7820309fd33b2175ae946670dbe`**; re-verified 2026-11-02T14:58Z
(`sha256sum -c` 17/17 OK · `build-tokens --check` CURRENT · `check-comps` PASS).

**The supersession rule (binding, one sentence):** *Midnight Slate v1.0.0 remains the shipped Ergon front-end theme until
every step below passes and the verdict is recorded; only then does Ergon Lumen replace it as the front-end theme.*

## 1. The gate, in order (each step has an owner, evidence and a done-condition)

| # | Step | Owner | Evidence it must produce | Done-condition | Failure handling |
|---|---|---|---|---|---|
| **P0** | **Owner grades the direction** (the pages in his browser) | Owner (through Lead) | the intake form `team-a-v2-grading-intake.md` filled with his verbatim deltas; direction accepted or accepted-with-deltas | his grade is on the board and classified (A–E) | rejected/Class D → no pack change; new direction pass via Nox/Lead |
| **P1** | **Deltas applied + re-freeze** | Team A (maker) | only his named deltas applied; `build-tokens --check` CURRENT; `check-comps` PASS; new `FREEZE.sha256` + digest | the frozen set the checker sees matches the grade | any extra change → revert, re-freeze; ambiguity → ask through Lead, never guess |
| **P2** | **Component all-states pass** (G2 of the curation gate) | Build seat (Team A/design lane) | per admitted component: default/hover/focus/pressed/disabled/loading/error/empty + reduced-motion + High Contrast renders; missing state = fail | every admitted component has a complete state set | missing state → HOLD that component; it does not ship |
| **P3** | **Screenshot compare** | Image-permitted seat (**not** DSH — image-read crash class) | side-by-side vs the pack reference on Lumen tokens; differences **named** (tokens, spacing, type, motion) | no unnamed difference; every difference accepted or fixed | unnamed/"looks close" → FAIL; no ship |
| **P4** | **Fresh checker** (pack-conformance + unslop) | a checker who built none of it (Lead spawns) | verdict on the frozen set: tokens discipline, floors, anatomy, unslop tells, the accessibility floors below | **PASS** | defects → fix in place with the Lead's relay, re-freeze, re-run — maker never checker |
| **P5** | **Nox/owner approval recorded** | Nox (pack authority) + owner where his act is required | written approval on the board for the pack change (version `2.0.0`) | approval recorded | no approval → nothing lands |
| **P6** | **Landing** (docs-only pack update) | Lead merges (merge commit, no force) | PR: `DESIGN.md` tokens/anatomy/version block, `design/tokens.json`, `design/motion-spec.md` → pack motion section, generated outputs via `generate-theme.mjs`; **no product code** | PR merged + post-merge verification (`--check`, generated outputs consistent) | revert = token-set revert from git history (v1 kept) |
| **P7** | **Post-landing record** | Team A | version/changelog entry; supersession note (v1 → v2); parts-bin fences re-validated against the translucent identity; known limits recorded | board + pack both state the new theme; v1 remains in history | rollback note stands |

**Rules that hold across every step:** maker never checker; nothing is "done" on the maker's own summary; anything wider
than the owner's graded deltas returns to Nox/Lead; no purchases; no public posts/publication; no credential handling;
no owner-facing action from this lane.

## 2. Accessibility floors — LOCKED (translucency does not buy its way out)

These floors are **not negotiable at any gate step** and are measured on the **full composited stack (both ambient washes)**.
Current measured values from the frozen set (`check-comps.mjs` at 2026-11-02T14:58Z):

| Floor | Requirement | Current measured | Locked? |
|---|---|---|---|
| Body text `ink` | ≥ **7:1** on the brightest text plane | **9.21:1** | 🔒 locked |
| Secondary `muted` | ≥ **4.5:1** on the brightest text plane | **5.62:1** | 🔒 locked |
| Quiet `subtle` | ≥ **4.5:1**, base/panel contexts only by rule | **6.14:1** (surface-1) | 🔒 locked |
| Accent text | ≥ **4.5:1** | **5.01:1** | 🔒 locked |
| Status text on wash (ok/warn/danger) | ≥ **4.5:1** | **4.66 / 4.93 / 4.77:1** | 🔒 locked |
| Focus ring | 2 px accent ring + 2 px gap, always visible, never animated; ≥ 3:1 non-text | **6.47:1** | 🔒 locked |
| Control boundary | ≥ 3:1 non-text (with both ambient washes) | **3.28:1** (`lineBright` α 0.44) | 🔒 locked |
| Field boundary | ≥ 3:1 non-text (pack `fieldLine` rule) | **3.30:1** (`fieldLine` α 0.46) | 🔒 locked |
| Reduced motion | `prefers-reduced-motion: reduce` removes travel/stagger; state changes instant | present + asserted | 🔒 locked |
| Forced colours / High Contrast | translucency/blur/soft depth replaced by system colours + 1 px borders; no reliance on the theme | `@media (forced-colors: active)` + `[data-contrast="high"]` preview covering surfaces/buttons/pills/inputs/messages/links/character | 🔒 locked |
| Targets | 44 DIP minimum; 56 DIP on employee screens | asserted (7 controls raised in the fix rounds) | 🔒 locked |
| One accent forward action per view | exactly one accent forward action; Back/Close/Cancel never accent | asserted in the kit/comps | 🔒 locked |
| Unslop gate | no gradient text/buttons, glows, text-shadow, emoji-as-icons, sparkle badges, 800 weights, uppercase tracked labels, `transition-all`, pulse/ping loops | clean in the frozen set | 🔒 locked |

**What is NOT a floor argument:** "it looks better with less contrast"; "the blur hides it"; "only decorative text"; "the
owner asked for translucency, so the floors bend". None of these pass this gate — the floors are measured, and a candidate
that cannot meet them is changed until it does, or it does not ship.

## 3. What the supersession changes — and what it does not

| | |
|---|---|
| **Changes when this gate passes** | The Ergon **front-end theme** becomes Ergon Lumen (translucent surfaces, depth, blur, border ladder, ambient light field, motion tokens in the pack, neural-character identity). Midnight Slate v1.0.0 is superseded as the front-end theme and recorded as superseded in the pack version/changelog. |
| **Does NOT change** | The pack anatomy (foundations → components/all-states → behaviour/motion → version/changelog); the one-accent-forward-action rule; tokens-only discipline; the type scale; targets/focus floors; plain-language rules; the unslop gate; W1 ADD-only client shape; the fact that comps are not built surfaces; any product behaviour. |
| **Does NOT happen before the gate** | No shipped surface is re-themed early; no mixed-theme state is claimed as done; no comp is presented as a finished front end; no product code changes; no publication. |
| **Rollback** | v1 tokens and rules remain in git history; the token set is additive by structure (v1 names kept where meaning is kept), so revert = token-set revert + regenerate, not a rewrite. |

## 4. Sign-off block (fill only when the gate is actually at that step)

| Step | Verdict | By | Evidence path / commit | Date |
|---|---|---|---|---|
| P0 owner grade | `<accepted / accepted-with-deltas / rejected>` | `<owner via Lead>` | `<intake form link>` | `<...>` |
| P1 deltas applied + re-freeze | `<PASS>` | `<maker>` | `<new FREEZE.sha256 digest>` | `<...>` |
| P2 all-states | `<PASS / HOLD rows>` | `<build seat>` | `<evidence path>` | `<...>` |
| P3 screenshot compare | `<PASS / named diffs>` | `<image-permitted seat>` | `<evidence path>` | `<...>` |
| P4 fresh checker | `<PASS / defects>` | `<checker>` | `<checker report>` | `<...>` |
| P5 Nox/owner approval | `<approved / not yet>` | `<Nox / owner>` | `<board record>` | `<...>` |
| P6 landing | `<merged commit>` | `<Lead>` | `<PR + merge commit>` | `<...>` |
| P7 post-landing record | `<done>` | `<Team A>` | `<board + pack version>` | `<...>` |

**Until P6 is merged and P7 recorded, the answer to "what is Ergon's front-end theme?" remains: Midnight Slate v1.0.0
(shipped), with Ergon Lumen as a frozen, checker-reviewed direction proposal — not the theme.**
