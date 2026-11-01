# Comps self-review — notes only (no rewrites)

**Status: pre-read notes for the Lead/owner feedback loop. NOTHING HAS BEEN REWRITTEN.** The comp pack is unchanged
(`check-comps.mjs` re-run clean at the end of this review: `COMP PACK OK`, tokens 41/41, banner in all 9 files). This page
lists what could read as a self-contradiction or a grading trap, so that when the graded feedback lands we iterate only on
the graded deltas. Prepared 2026-11-01 by Team A (Lane 5).

**Method:** programmatic audits (accent/primary count per view, duration strings, state-word scan, internal-link
inventory, raw-value scan in the comp CSS) plus a manual read-through; no browser, no pixel verification — consistent
with the stated honest limit.

## Findings

| # | Severity | Where | Observation | Why it matters | Suggested delta (awaiting feedback — not applied) |
|---|---|---|---|---|---|
| F1 | **High** | All comps vs the pack's component layer | The comps introduce component classes the pack does not yet define: message bubble, composer, tab bar, bottom sheet, sidebar/nav, live rail, task card, artefact card, skeleton/loading placeholder, character + state disc. The pack's component layer today is Buttons, Fields, Tiles, Pills, Step rail, Window chrome (+ four surfaces). | The pack anatomy says components-with-all-states; if builders start from these comps, either they invent these components or we need pack admissions first. This is the main pre-build gap, not a comp defect. | After grading: propose the new component classes into the pack's component layer through the pack gate (all-states → screenshot compare → fresh checker); the comps are the direction reference for that proposal. |
| F2 | Medium | `comps/06-character-sheet.html` | Character body colours in the three examples use status tokens (`accent`, `ok`, `warn`), while the live-state disc also uses `ok` / `muted` / `warn`. | A `warn`-coloured character carrying an `ok` "working" disc can read as a contradiction: body colour = identity, disc = state, but both use the same status vocabulary. | Restrict the character colour choices to non-status hues, or state explicitly that body colour never encodes state (disc + word always do). Needs a design ruling before build. |
| F3 | Medium | `comp-kit.css` `.scrim` | The one non-token colour in the comp CSS: `background: rgba(11, 15, 20, 0.72)` (base at 72%). The pack has no scrim/overlay token. | The pack's rule is tokens only; overlays will be built from this comp, so the scrim needs a pack answer. | Either add a scrim token (a token change — needs a ruling) or derive it from `--base` with `color-mix(in srgb, var(--base) 72%, transparent)` at build. Not changed here. |
| F4 | Medium | All comps | The comps are representative views, not the all-states matrix: `hover`/`focus`/`pressed`/`disabled` exist in the comp CSS but are not exhibited per component; `reduced motion` / `High Contrast` appear only on the storyboard; empty/loading/error/offline are shown for both faces in comp 07. | Grading should not read the comps as all-states evidence; the all-states check remains a build-gate requirement (checklist section B). | None to the comps; add this sentence to the grading cover if useful (owner-facing clarity only). |
| F5 | Low | `comps/07-states.html` | Three accent buttons across eight panels (mobile empty "New task", mobile error and desktop error "Try again"); desktop empty, loading and both offline panels have no accent action. | Could read as inconsistent application of the one-accent-forward-action rule. Offline deliberately says "retries automatically"; desktop empty could take the accent at build. | Decide the one forward action per state view when building; if the graded read is "offline should offer an accent retry", that is the delta. |
| F6 | Low | `comps/06-character-sheet.html` | The character sheet has no accent action at all. | It is a reference sheet, not a product view — but a grader may read it as a pack violation. | None; noted so it is not mistaken. |
| F7 | Low | `comps/05-desktop-workspace.html` nav; `comps/03-mobile-tasks.html` nav | Comp-only navigation: desktop "Apps" links to the mobile Make comp, "Live" links to itself; Tasks nav links to itself. | These are stub links for grading, not product IA; a grader could read the desktop Apps/Live surfaces as defined when they are not. | Label the stubs in the grading notes only; product IA for Apps/Live is a build-time design. |
| F8 | Low | `comps/02-mobile-triage-approvals.html` | Tier A card offers "Remember for Priya" ("this person, this sheet only"). | Rulings say Tier B never gets "Always allow"; no ruling explicitly permits remembering a Tier A grant, even bounded. | Confirm with Nox before build; the comp wording is deliberately bounded so it does not read as a blanket approval. |
| F9 | Low | `comps/02-mobile-triage-approvals.html` item count | Header says "3 items · 2 approvals, 1 question"; the sheet shows the two approvals, the question lives in comp 03 (Tasks, waiting on you). | Reading comp 02 alone, one item is uncounted. | Cross-reference comp 03 in the grading notes; the question is intentionally a task rather than an approval. |
| F10 | Info | Comps + storyboard + pack spec | No duration contradiction found: storyboard and `design/motion-spec.md` agree on 120/200/250/350 ms with one ease-out curve, plus the existing 180 ms setup-window fade; comps reference the 350 ms sheet and the four budgets. | Confirms the motion numbers are consistent across artefacts. | None. |
| F11 | Info | `check-comps.mjs` run | Validator clean: 7 comps + storyboard + index, banner in all 9 files, no external assets/scripts/images/hex literals, tokens 41/41 exact, internal links resolve. | The mechanical properties hold; this review is about design meaning, not asset discipline. | None. |
| F12 | Info | Frames | 390 × 844 mobile and 1280 × 800 desktop are comp frame sizes, not device claims. | Avoids implying device verification. | None. |

## Bottom line for the Lead

- **No rewrites made.** The pack is exactly as accepted; this page is a pre-read only.
- **One high-value pre-build gap (F1):** the comps define ten component classes the pack does not yet own. Once the owner
  grades, the next build-shaped step is a pack component-layer proposal for those classes, through the pack gate.
- **Two design rulings might be needed before build (F2 character colour vs status tokens; F3 scrim token):** both are
  small, but they are pack questions, not comp fixes.
- **Everything else is grading clarity or build-time choice.** I will iterate only on the graded deltas.


---

## Lead rulings on these findings (recorded 2026-11-01; binding for the next steps)

| Finding | Ruling of record | Where it lands |
|---|---|---|
| **F2** character body colour vs status tokens | **Body colour never encodes state; state is the disc only.** Use a non-status body palette for the three shapes (no `accent` / `ok` / `warn` as body fill). | Record as a **design rule in the pack component-layer proposal** (not just a comp fix), when that proposal is filed after grading. Not reworked in the comps now. |
| **F3** scrim raw colour | **No scrim token now** — that is a token change and a pack-gate item. Derive the scrim at build from `--base` via `color-mix` (document the ratio). Record "scrim/overlay" as a **candidate token for pack v1.1** through the gate. | Candidate-token note in the v1.1 queue; build-time derivation rule. Comp unchanged. |
| **F1** ten new component classes | **Noted, sequenced.** They are the next **build-shaped** step, but **only after the owner's grading** — graded deltas may change the class set. Then file a pack component-layer admission proposal for exactly the surviving classes, through the gate (proposal → all-states → screenshot compare → fresh checker). **Do not start now.** | Next build-shaped batch after graded feedback. |
| **F4** comps vs all-states matrix | **Confirmed:** comps are representative views; the all-states matrix stays a build-gate requirement, unchanged. | No comp change; grading-cover clarity if useful. |
| **F8** Tier A "Remember for Priya" | **NOT APPROVED — flagged to Nox.** The rulings only pin "Tier B never offers Always allow"; Tier A remember is a policy question, not a design call. **Do not treat as approved.** | Awaiting Nox. |
| **F5–F7, F9–F12** | **Hold for grading.** Do not rework on these notes; iterate only on graded deltas. | Awaiting owner feedback. |

**Fences unchanged:** no pack files touched, no pack-test hardening, $0, no sends, no credentials. This page remains a
note-only artefact; the comp pack stays byte-identical to the accepted state.
