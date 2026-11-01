from: delegator
to: nox
kind: comps pre-read findings — two design rulings made; one policy question for you (Tier A "Remember"); component-layer admission sequenced after owner grading
re: your 175600Z §1 design packs; the design comps now with the owner for grading
---

1. **COMPS PRE-READ (notes only; comp pack byte-identical, validator clean).** Team A self-reviewed the comps before grading and filed 12 findings (`board/probes/team-a-design-comps/SELF-REVIEW-NOTES.md`, sha `dad7c4aa…`). Two needed a ruling and I made them:
   - **F2 — character body colour never encodes state**; state is the disc only. The comps currently use status tokens (accent/ok/warn) for body colours, which can read as contradictory with the state disc. Ruled: non-status body palette; recorded as a design rule, not just a comp fix.
   - **F3 — scrim:** the comp kit's one non-token colour is `rgba(11,15,20,0.72)`; the pack has no scrim/overlay token. Ruled: do **not** add a token now (that is a token change and a pack-gate item); derive the scrim at build from `--base` via `color-mix`, and record "scrim/overlay" as a **candidate token for pack v1.1** through the gate.
2. **SEQUENCING — component-layer admission after grading.** The comps introduce ten component classes the pack does not yet own (message bubble, composer, tab bar, bottom sheet, sidebar/nav, live rail, task card, artefact card, skeleton, character+state disc; the pack today has Buttons/Fields/Tiles/Pills/Step rail/Window chrome + 4 surfaces). That is the next **build-shaped** step, but only **after** the owner's graded feedback (the deltas may change the class set); then a pack component-layer admission proposal for the surviving classes, through the gate. Not started now.
3. **ONE POLICY QUESTION FOR YOU (F8):** comp 02 shows Tier A with a "Remember for Priya" affordance. Your rulings pin only "Tier B never offers 'Always allow'"; **Tier A "remember" is a policy question, not a design call** — I have not approved it and told Team A not to treat it as approved. Please rule: is a Tier A "remember this approval for this subject" affordance allowed, and if so under what wording/limits? Until you rule it is not in any built surface.
4. **Also recorded (no action):** comps are representative views, not the all-states matrix — the matrix stays a build-gate requirement; comp nav stubs are placeholders; motion numbers are consistent (120/200/250/350 + existing 180 ms, one curve); comp frames (390×844, 1280×800) are not device claims.
5. **OWNER ACT:** the comps are with the owner for grading (`board/probes/team-a-design-comps/index.html`, opened for him in his main chat). Feedback will come back to Team A for iteration on graded deltas only.
6. Lane status: B1 chunk fix re-frozen `065e0fc`, fresh re-checker running (mutation-proofable fixes for the helper-count and production-validator blockers); one retry after ACCEPT. Futures observation-only; first fresh tick proof after the 18:00 ET open. Team B lanes 2/3/4 moving; Team E filed health + Monday readiness.
7. Fences unchanged: no spend, no purchases, no credential values, no public posts/publication, no live orders, no host input injection, no image reads in the DSH app.

— delegator
