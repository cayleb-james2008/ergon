# Order o1: closure record (administrative closure under Nox's Option C ruling, 2 Nov 2026)

**Order.** o1, "fleet visible surfaces": a drop-in DSH client plugin package (`packages/client/ui-fleet`) built by a DeepSeek builder lane on the fleet roster. Tier 1, done-when mode `rubric` (revision 4, four criteria).

**Status of the order in the harness.** `in_progress`, held by the Lead, with every non-final task `done` and banked and the Lead's consolidated report filed (post 163). The final task t5 is `open` and cannot be taken by any component of the shipped harness (below). The order is therefore closed ADMINISTRATIVELY on the evidence in this record, not by the harness.

## 1. The route gap, stated plainly (Nox's condition 2)
A rubric final task of a tier-1 order cannot reach `done` in release 0.3.5 / 0.3.5.1. Three components, each correct against its own specification, leave the job to one another:
1. `orders/finisher.js` runs the final task by itself only for a `command` done-when; for a `rubric` it is "left open for a fresh Checker lane (Stage 4)" (the guard `doneWhen.mode !== 'command'`).
2. `roster/checkflow.js:33` starts a Checker lane only for a task that is `built` and **not final** (`|| t.final`). No Checker is ever started for a final task.
3. The owner route `verdict-tier2` (`orders/tasks.js`) accepts only tier-2 tasks ("a tier-1 task; the in-harness Checker closes it").
So no component could reach t5. **This is a gap in the harness, not a defect in o1's work.** Nox ruled (a) Option A refused: a rubric is not rewritten into a command to fit a missing route; (b) Option B scheduled as the next package (the roster starts a fresh Checker for a rubric final task; the verdict route accepts tier-1 final tasks through that Checker), under the liveness gate: a live drive of a rubric-final order end to end, no fakes, before anyone calls it working. Not built now.
Lesson filed: `harness-squared/library/coordination/entries/2026-11-02-a-route-that-cannot-fire-is-not-a-route`.

## 2. The closure evidence (Nox's condition 1: the actual outputs, not the Lead's report alone)
**(a) Fresh Claude check over all four rubric criteria, run against the commits themselves.** Report: `finish-docs/CHECK-o1-remediated.md`. One fresh checker with no conclusions given, its own `git archive` extraction of the head commit `14909696` from the builder's lane repository:
| criterion | met | what was run |
|---|---|---|
| 0 state layer committed, tests pass, no fixture shown as real | yes | `node --test` 43/43 (state 13, rig 10, panels 20); types, strict parse, loopback guard, live and rehearsal sources and view model read; guard fuzzed (0.0.0.0, 127.1, decimal and hex IPv4, credentials, `127.0.0.1.evil.com` all refused); refusal before any transport proven (0 transport calls, 0 stub connections) |
| 1 fleet sidebar panel from the view model, tests pass | yes | FleetPanel renders lanes, meters, orders from `fleetViewModel`; t8 tree 31/31, head panels 20/20 |
| 2 phone-VM and Windows-VM panels, loopback-only, tests pass | yes | rig 10/10 incl. one real 127.0.0.1 round trip with readback equal to sent and serial surfaced, 0.0.0.0 / 10.0.0.5 / example.com refused before transport; Windows client surface exactly [frame, state], no input path |
| 3 every built commit checkable, tokens only, fences, C4 contract | yes | all six commits extract and pass their tests (t7 133, t8 31, t9 43, t4 43, t12 43, head 43); head `verify-tokens: 26 files clean`, `verify-contract: 3 panels clean`, **export surface client [apply, inject], root [apply]**; its own greps found no hex, rgb, hsl, oklch, gradient, blur, backdrop-filter, emoji; every colour is `var(--dsw-alias-*)`; one main key and one sidebar.panellist per panel, key equal to id |
**Mutation testing by the checker:** 84 mutations applied to its own extraction; 62 caught by the tests or fences, **22 survived** (listed in the report). The checker states no rubric failure is hidden by a survivor; the survivors are test-coverage gaps.

**(b) Task-level verdicts, each by a fresh Checker lane** (t4, t7, t8, t9, t11, t12 all `done`, round 1). Note t11's verdict was a static rubric derivation (the rubric session has no execution tool, as the checker said); the fresh Claude check in (a) re-ran it.

**(c) The Lead's consolidated report, post 163**, with the correction in section 3.

## 3. What the evidence did NOT support, and corrections the record must carry
- **The Lead's "one chain" provenance is wrong** (checker defect, medium). Post 163 says the six commits form one chain 1480019b -> fb8b703 -> ... -> 14909696. In fact `1480019b` (t7) exists only in lane `builder_1-7149427f`; the head descends from a different root in lane `builder_1-d60df12d`, and the head's state layer is a **different implementation** from t7's (13 state tests against t7's 133). The 133-test suite does not cover the head's state code. Criterion 0 still holds for the head (verified directly); the provenance narrative in post 163 should not be relied on.
- **Test coverage is thin where the rubric leans on reading** (medium). Panel behaviour is tested by source-text regexes and pure view-model checks; `apply()` is never executed by a test. Survivors include the Live/Rehearsal chip gate on the fleet, phone and Windows panels, the SSE-frame refusal of a rehearsal snapshot, the phone panel's displayed serial and readback, and the disabled phone form. "No fixture data presented as real" therefore rests on the checker reading the code, not on a guarding test.
- **Fences have two holes** (low): a glow via `box-shadow` on an alias token and a decorative `keyframes` animation pass both fences although the unslop list says no glows and no decorative animation.
- Low: the documented test command fails on one installed Node (mise v26.7.0, MODULE_NOT_FOUND) and passes on two others; quoted line numbers in the C4 doc have drifted; the Windows panel can show a failure banner while its `live` flag is true.

## 4. Not verified (stated by the checker, carried here)
No type-check, no tsdown bundle build, no React render of any panel, no `apply()` against a real slot, no check of `--dsw-*` names against the real design pack, and no behaviour under a real SSE facade, AVD or Windows VM. The **rendered result and the integration into the plugin client bundle are out of this rubric** (graded under leg 1 of the plugin finish mission).

## 5. Closure executed (2 Nov 2026, 16:35 ET, Nox's ruling)
The order was **cancelled by the owner op `order-cancel` with this record as the reason**, under Nox's ruling "cancel it citing the closure record; Option C is then fully discharged" (events 1862 and 1863, board post 202). **The harness has no `cancelled` state: a cancelled order is stored as `failed`.** Read the stored `failed` on o1 and on its final task t5 as an administrative closure, NOT as a failed deliverable: the recorded `fail_reason` is "cancelled by cayleb-local: Closed administratively under Nox ruling Option C ... See O1-CLOSURE-RECORD.md", and the fresh check met all four rubric criteria. The Lead and the builder lane are idle (not stopped).

## 6. Next-package scope (rulings of 2 Nov: Option B and the provenance item)
- **Option B:** the roster starts a fresh Checker for a rubric final task, and the verdict route accepts tier-1 final tasks through that Checker; shipped under the liveness gate (a live drive of a rubric-final order end to end, no fakes).
- **Provenance and coverage must be test-backed (Nox):** fixture provenance and "no fixture shown as real" are proven by a mutation or provenance check the Checker RUNS, never by a maker's claim that a checker repeats. This closes the weak leg named in section 3 (the "one chain" claim, the surviving Live/Rehearsal-gate mutations).
- The package is built on release/0.3.5.1, and its installer self-check must prove `validate.js` survives before any install is proposed.
