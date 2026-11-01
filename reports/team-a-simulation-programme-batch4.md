# Team A — Ergon rehearsal SIMULATION programme prep: batch 4 report (bank harness + labelled self-test)

**Order:** Lead GO on batch 4 after batches 2b+3, with two hard labels: (1) the driver is a **simulation-bank harness, not
the rehearsal package** — no writes to the merged package, no `docs/acceptance` writes, no repo commits, inputs workspace
only; (2) the state-advanced batch run must be labelled in every artifact as a **bank self-test** (fixture → evidence-layout
exercise), explicitly **NOT the G1 run** and **NOT an acceptance rehearsal**, with the label in file names too if a reader
could confuse them. **Status:** batch 4 on disk and self-verified; **prep only** — $0, no model calls, no package edit,
no rehearsal run, no repo write. **Successor:** batch 5 — the fold-in decision, which the Lead takes to Nox; this lane does
not decide repo placement.

## 1. What landed

| File | Size (B) | SHA-256 (16) | What |
|---|---:|---|---|
| `harness.mjs` | 11,462 | `0827ab4221cb8ad0` | The simulation-bank harness — **not the rehearsal package**. `plan --company <id> --out <dir>`, `self-test --out <dir>`, `verify --out <dir>`. Reads a `fixture.json`, emits a run plan in the package's documented evidence shape and one labelled state-advanced bank self-test; writes **only** under `--out`. |
| `self-test-output/BANK-SELF-TEST-batch-run/` | 4 files | summary `d13171ce589c8fd3…`, acts `2a692360b01a1975…`, report `599ea4b10fe9e60c…`, readme `32009905f09c19f0…` | `BANK-SELF-TEST-summary.json` (label, notice, clocks, per-company rows, totals), `BANK-SELF-TEST-acts.jsonl` (76 act rows, every row labelled and `clock: state-advanced`), `BANK-SELF-TEST-report.md`, `BANK-SELF-TEST-README.md`. |
| `self-test-output/BANK-SELF-TEST-plan-harborline-plumbing/` | 3 files | plan.json `b2644e8ae2f9574d…`, plan.md `77cec761dc59237b…`, readme `6af974eb747610fd…` | The per-company run plan: cast with model placeholders, chain plan with end-scoring, divergence plan with `handlingKind`/`refusalExpected`, evidence-layout shape. |
| `SPEC.md` / `README.md` / `batch-plan.md` | updated | `36dbfbd9…` / `f5751142…` / `fd32983a…` | Harness + batch-4 state recorded; batch-4 row landed. |

Bank totals: 20 files, 248 KB (12 bank files + 7 self-test artifacts + harness). The six companies / 80 cast / 12 chains /
28 divergence cases / 2 cross-company collisions are unchanged.

## 2. Label compliance (the Lead's hard label 2, machine-checked)

- The label string is `BANK SELF-TEST — simulation-bank harness output. NOT the G1 run. NOT an acceptance rehearsal. Not evidence.`
  It is written into **every** generated artifact (README, summary, report, acts, plan md/json — and as a `label` field on
  every act row).
- **Every file name** carries `BANK-SELF-TEST` (e.g. `BANK-SELF-TEST-summary.json`), and every directory carries it too.
- `harness.mjs verify --out self-test-output` → `BANK SELF-TEST VERIFY: PASS — 7 files, every file labelled and named BANK-SELF-TEST` — exit 0.
- The summary's `notice` repeats: "NOT the G1 run. NOT an acceptance rehearsal. NOT evidence. No model was called." and
  `packagesTouched: none`.
- Clocks are separated exactly like the package's own honesty format: `clocks.live = {acts: 0, wallSeconds: 0}`,
  `clocks.stateAdvanced = {acts: 76, simulatedDays: 5, wallSeconds: 0}` — state-advanced time is never summed into live.

## 3. What the self-test actually did

`node harness.mjs self-test --out self-test-output` → `bank self-test written: …/BANK-SELF-TEST-batch-run (76 acts, 6 companies)`.
It enumerated, per company, every chain hop (as a `chain-hop` act), every chain end (as `chain-end`, `scored: true`),
and every divergence case (as a `divergence` act carrying `handlingKind`/`refusalExpected`/`packagePending`), stamped
each with a deterministic simulated day 1–5, and wrote the summary/report/acts. **No model was called; no package module
was imported; the acts are enumerated fixture steps in an evidence layout — a data-shape exercise, not an executed
interaction.**

Bank service counts (from the summary): 6 companies, 76 acts, 12 chains, 28 divergence cases; sizes 6–22.

## 4. Verification (observed, 2026-11-01 ~13:58–14:00 ET)

| Check | Command | Result |
|---|---|---|
| Harness self-test | `node harness.mjs self-test --out self-test-output` | exit 0, `76 acts, 6 companies` |
| Harness plan | `node harness.mjs plan --company harborline-plumbing --out self-test-output` | exit 0, plan dir written |
| Label verify | `node harness.mjs verify --out self-test-output` | `BANK SELF-TEST VERIFY: PASS — 7 files…` exit 0 |
| Unknown company | `plan --company nope` | exit 1, `unknown company: nope` + available ids |
| Usage (no args) | `node harness.mjs` | exit 2, usage |
| Usage (missing `--out`) | `self-test` without `--out` | exit 2, `--out is required` |
| Label removed (copy) | stripped the label from one report | exit 1, `…missing the BANK SELF-TEST label` |
| File renamed unlabelled (copy) | `report.md` instead of `BANK-SELF-TEST-report.md` | exit 1, `…file name lacks BANK-SELF-TEST` |
| Bank validator (unchanged) | `node validate.mjs` | `SIMULATION BANK OK: 6 companies, 80 cast…` exit 0 |
| Boundary | `git -C ~/workspace/harness-build/a-l5-foldin status --porcelain -b` | clean on `docs/l5-run-evidence` (merged head); no package/`docs/acceptance` write, no repo commit |

## 5. Honest limits

- **The self-test is not a run.** It enumerates fixture steps and lays them into the evidence shape; it does not execute
  agent behaviour, call a model, or produce acceptance evidence. The label says so in every file, and the file names say
  so too.
- **`verify` enforces labelling, not correctness.** It fails closed on a missing/misnamed label; it does not prove the
  plan matches the package's live template line for line (the package template may evolve after G1).
- **The state-advanced clock is bookkeeping, not measurement.** 76 acts over 5 simulated days with `wallSeconds: 0` is a
  deterministic fixture walk; it is never reported as live operation.
- **No package write, no repo commit, no spend, no credential, no send** — the Lead's hard label 1 held.
- Fold-in (repo vs workspace) remains batch 5 for Nox/Lead; batch 4 does not decide it.

## 6. Successor

- **Batch 5 (hand to Lead → Nox):** the fold-in decision — repo placement as a new `docs/acceptance/simulation/` area
  (docs-only branch + PR + fresh checker) vs staying a workspace bank until the package build is authorised. The Lead
  takes the batch-5 report to Nox; this lane does not decide.
- Until then: no further simulation-bank work claimed; the bank stays inputs-only. Blocked items unchanged: the RUN on
  G1 (Lane 1 installed output); the package build on Nox C1–C6 + W1.
