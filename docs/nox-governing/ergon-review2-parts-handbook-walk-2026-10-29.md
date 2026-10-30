# Review 2, instalment 1 — GLM/Z.ai bounded review of Parts Handbook (walked by Nox, 29 Oct 2026)

Source document: `~/workspace/user/files/Ergon-Parts-Handbook-Code-Review_2_ptd5.pdf` (Cayleb handed it over 17:15 ET). Scope: commit 8218532, `suite/parts-handbook`, 22 files / +1,919 lines. Method: every file read, README invariants traced to enforcement, test suite run locally — 39/39 green, Node 22, stdlib-only confirmed. Page 2 of the PDF is a picture (contents page); all substantive sections were legible.

## The five findings

- **F-1 (High)** — The parts-coverage invariant is enforced only inside the planner. The README promises every plan, from any source, must satisfy validateCoverage before it is rendered or stored; drafts.js create()/revise() check shape only, and render.js never checks despite holding the intake. The flagship shared-draft flow is the exposure: a builder's on-site revision can over-use a part or name a part not in the box and is stored and rendered silently — the handbook invents or drops parts and the builder discovers it on the doorstep. Fix: store the intake with the draft; run validateCoverage in create() and revise(); optionally gate render too.
- **F-2 (Medium)** — KB fuzzy lookup is bidirectional containment, first match wins. Worked example: KB holds "KALLAX 2×2"; a query for "KALLAX 2×2 with doors" matches the plain entry, wears fact-grade `kb` provenance, passes coverage (subset use), and the door parts render as expected spares — wrong, valid, and dressed as fact. Winner is also decided by Map insertion order as the KB grows. Fix: scored matching (longest token overlap); comparable candidates → return a miss (falls through to labelled inference, the safe outcome).
- **F-3 (Medium)** — Inputs unbounded (parts, notes, string lengths; KB entry parts/steps/checks; draft plans). The suite already fixed this exact class elsewhere (budget-proxy round 3: "bound every forwarded input field"; review-gate caps). Fix: sibling-style caps (hundreds), errors naming field and limit.
- **F-4 (Low)** — KB load path trusts disk JSON: validated on write, not on load. A truncated or hand-edited file crashes the constructor or loads as an invalid entry feeding the planner with fact-grade provenance. Fix: validate on load; fail loudly naming the file, or quarantine with a warning.
- **F-5 (Low)** — Renderer trusts plan shape: a step without `uses` crashes (TypeError); a missing source renders the literal "[undefined]". validateCoverage already defaults defensively. Fix: default uses to [], explicit unknown-source marker; mostly subsumed if F-1 gates render.
- **Nits** — create() slug error reports the id instead of the title ("got undefined"); "nut " trailing space + no "nuts" entry, so "nuts and bolts" misses the hardware category; plan-level source "mixed" is unreachable today (comment it).

## What the review verified positively

Provenance is per-step, assigned at creation, never upgraded (kb / researched / inferred, with a rendered caution banner); the renderer is a pure function so the agent and human surfaces cannot drift; drafts are append-only with deep-copy accessors; atomic temp-file-plus-rename writes; intake rejects unknown fields; no network, no keys, deterministic stubs; the live MiniMax provider is a deliberate loud not-implemented stub; the README's known-limits section matches the build. Reviewer's verdict: strongest app in the suite.

## Reconciliation against Review 1 (Freebuff, 16 findings)

No direct duplicates — different scope (Review 1: suite-wide structure — webhook duplication, routing duplication, fs-guard and crypto copies, decomposition; Review 2 instalment 1: one component). The classes rhyme: F-3 is the input-bounding class the suite closed in the budget proxy, and "invariants enforced only halfway down the pipe" is the same disease as Review 1's drifting copy-pasted enforcement, one level down. This document is a bounded component review ("prepared for the Ergon suite review queue") — instalment one of the GLM programme, not the full-suite pass.

## Nox's recommendation (Cayleb confirmed the pattern at 17:12 for Review 1's answers)

Accept all five fixes into the post-Lite review-implementation queue (pickup item 3), with F-1 and F-2 as the gate for the component's clean admission into the company library — they are the two that touch the core promises (a fact is a fact; a plan accounts for every part). The review ships its own acceptance tests for each fix, so the done-whens are already written. Sequencing unchanged: GLM pushes first; implementation after Lite.
