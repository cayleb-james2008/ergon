# Ultra-loop report — Ergon scale-up (17–18 Nov 2026)

**Goal (approved, un-drifted):** an AI agent for every employee, personal
to each person, learns how they work, finishes their digital work
autonomously end-to-end. No approval ceremony in the core loop.

**Bar:** `.loop/BAR.md` — conditions C1–C12 + DIRECTION v1 (plain-spoken,
workmanlike; well-kept desk; terracotta accent; signature: tool steps as
a plain ledger in the thread). C10 (look & feel) is `owner` — Cayleb
grades it; the loop never marked it passed.

**Baseline:** tags `loop-baseline` 162dd57 → `loop-start` c0fd477.

## Improve pass (impeccable discipline, applied to the surface)
- Themed shell rebuilt on CSS variables: light + dark, toggle in
  Settings → Appearance, dark default (was single-theme).
- Person switcher, memory panel, honest no-key banner; agent steps
  rendered as a plain ledger in the thread (the BAR's signature).
- Scale-up depth behind the surface: people system (per-person
  workspace/memory/threads, UI creation, seeded learning), four new
  tools (make_spreadsheet, research as Rune skills; search_files,
  move_file), standing preferences injected into every prompt.

## Thermonuclear review (fresh-context, both directions, surface consumption)
Four rounds, all findings reproduced before fixing; full detail in
`.loop/LEDGER.md`.
- **R1:** Rune skill return labels, created-person learning, sandbox
  `exists()`-blindness, Content-Length DoS, session pinning,
  history-to-model, corrupt-store backup. → 28/28.
- **R2 (NOT DONE):** CRITICAL symlink-resolver lexical-substitution
  escape → resolver rewritten as a component queue (per-step
  containment, 16-hop cap); session fixation → 16-byte random ids,
  forged ids never adopted; done-gate, real 413, first-free-suffix
  backups, exact search stats, link-moving. → 36/36.
- **R3:** F1/F2 verified unbreakable (C7 PASS); done-gate rebuilt onto
  the written-files ledger; strict people.json shapes; dangling-link
  symmetry; 413 drains in-flight body + total 30 s deadline; sessions
  capped. → 42/42.
- **R4 (confirmation):** all residuals pass; HIGH — gate tokenizer
  couldn't represent spaces (the product's own default artefacts are
  spaced) → mention matching rebuilt (leftward expansion, longest
  match, basename fallback); memory/move credited in the ledger.
  → **44/44 green. Converged** — no material findings open.

## Sealed exam
Independently written + run, black-box, 19 checks: **19/19 PASS**,
exit 0 (run dir `~/workspace/exam-runs/ergon-20261118-015111-15140`).
Run 1 scored 11/19 — all 8 failures were an exam-side stub routing bug
(`/v1/chat/completions` 404'd by the stub), wire-proven by the
examiner-side debugger; harness patched (checks.py byte-identical),
app untouched, re-run clean.

## Weakest remaining point
C10 (the look) is ungraded — it is Cayleb's call, by design. Behind
that: the live-model path is real code proven against a stub; it has
never run against a live key in this environment (no key available
here), so model-behaviour quality in production is unproven until a
host-side key is placed.

## Replacement staging
`~/workspace/restart-20261117/replacement/ergon/` — README.md (also the
repo's refreshed README), ergon.html (résumé-site page in the current
site's editorial style; schematic walk-through from the real e2e
scenario, no fabricated captures), CLAIMS.md (every claim tied to its
proof). Nothing pushed or published.
