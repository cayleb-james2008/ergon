# Team A — Ergon rehearsal SIMULATION programme prep: batch 2b + batch 3 report

**Order:** Nox capacity order `20261101T182200Z` pull item 2; Lead GO after batch 2: "continue with batch 2b then batch 3,
same boundary (inputs only, merged package untouched, no run, $0)", plus the batch-3 ask: keep per-kind divergence
rubrics **machine-checkable** (expected handling + refusal flags) so the validator fails closed on a rubric gap rather
than relying on prose review. **Status:** both batches on disk and self-verified; **prep only** — no package edit, no
rehearsal run, no send, $0. **Successor:** batch 4 (integration driver), then batch 5 (fold-in decision for Nox/Lead).

## 1. Batch 2b — the 22–30 staff company

`companies/kelvinside-fabrication/fixture.json` (24,447 B, sha256 `d6d810ecfcec1929…`): **22 staff**, small manufacturer
(fabrication floor, stores & despatch, drawing office). Cast of 22 with written voices (operations, production lead,
4 fabricators, 2 machinists, 2 welders, QA, stores lead + 2 operators, despatch lead + operator, maintenance, buyer,
finance, HR/admin, sales, design/draughting), 2 chains, 5 divergence cases, fixed seed, seeded past and a named
"what this exercises" line.

- Chain `KF-C1` — tolerance dropped from the drawing-office relay before batch release (detail-dropped).
- Chain `KF-C2` — same pallet promised to two customers on one collection (chains-collide).
- Cases: stale drawing revision, QA sign-off refused as human-only, planned maintenance described as an unplanned line
  stop, duplicate press-fault reports, despatch cut-off.

Bank now: **6 companies, 80 cast members, 12 chains, 28 divergence-case instances, 2 cross-company collisions, company
sizes 6–22** (the plan's 22–30 band is reached).

## 2. Batch 3 — divergence depth, machine-checkable

`divergence-cases.json` (16,492 B, sha256 `6c5c1b7b3c8f3c2e…`) now carries, **per kind**:

- `rubric` — `caught` / `missed` / `chargedAhead` (concrete scoring criteria, not prose hopes);
- `evidenceRecipe` — exactly 3 steps (setup, divergence, resolution) a run must record;
- `wrongExample` — one expected-failure example of what a wrong agent does;
- **`handlingKind`** — one of 12 enumerated classes (`adapt`, `ask-for-field`, `surface-conflict`, `separate-severity`,
  `use-current-version`, `link-or-distinguish`, `refuse`, `obtain-formal`, `recover-detail`, `correct-recipient`,
  `apply-window`, `expected-refusal`);
- **`refusalExpected`** — boolean; when true the correct handling must say "refuse".

Every **case instance** in the six companies now repeats its kind's `handlingKind` + `refusalExpected`, so a case cannot
silently drift from its rubric. The validator enforces: field presence, enum membership, rubric length, 3-step recipe,
that `refusalExpected: true` ⇒ the handling text says "refuse", and **case ↔ catalog consistency** for both fields.

## 3. Verification (observed, 2026-11-01 ~13:47–13:50 ET)

**Positive:** `node board/probes/team-a-simulation-programme/validate.mjs` →
`SIMULATION BANK OK: 6 companies, 80 cast members, 12 chains, 28 divergence cases, 2 cross-company collisions (company sizes 6-22); 6/6 chain kinds, 12/12 divergence kinds covered; token-blind separation 239/240; secrets/PII scan clean` — **exit 0**.

**Fail-closed proofs for the machine-checkable rubric (fresh copy per test, so each failure is attributable):**

| Test | Change on a fresh copy | Result |
|---|---|---|
| N11 catalog kind missing `handlingKind` | deleted the field from tone-mismatch | exit 1, `catalog tone-mismatch: needs handlingKind (machine-checkable, batch 3b)` |
| N12 case `handlingKind` mismatch | set harborline's missing-info case to `refuse` | exit 1, `HL-D1 handlingKind must match its catalog kind (ask-for-field)` |
| N13 refusal flag without refusal text | made stale-data `refusalExpected: true` while its handling does not say refuse | exit 1, `catalog stale-data: refusalExpected true but correctHandling does not say refuse` |
| N14 case refusal-flag mismatch | flipped cedar's unauthorized-scope case to `false` | exit 1, `CH-D1 refusalExpected must match its catalog kind (true)` |

Batch 2's proofs (N7–N10 cross-company/size) still hold; batch 1's (N1–N6 voices/chains/secrets/missing-file) still hold.

## 4. What "machine-checkable" buys (honest statement)

The Lead's property is met in the sense that **a gap in the rubric now fails the validator**: a kind without a handling
class, a case that drifts from its kind, or a refusal flag without refusal wording is a hard error, not a prose review
item. What it does **not** do: prove an agent behaves correctly. The fields describe the expected handling and the
scoring criteria; a run still has to drive the case and the fresh checker still judges the behaviour. The validator is a
maker check — maker never checker.

Other honest limits unchanged: token-blind separation **239/240** is a proxy, not human-blindness; all companies are
synthetic and 28 cases is a tranche, not a complete divergence matrix; no run/state-advanced claim; the fold-in decision
is batch 5 for Nox/Lead; no edit to the merged rehearsal package (ergon worktree clean, still at merged head `5cde312`).

## 5. Inventory (bank now 12 files, 180 KB)

| File | Size (B) | SHA-256 (16) |
|---|---:|---|
| `SPEC.md` | 11,993 | `f2474b47eca42a95` |
| `README.md` | 4,944 | `20440705cde4115e` |
| `divergence-cases.json` | 16,492 | `6c5c1b7b3c8f3c2e` |
| `validate.mjs` | 16,622 | `446c35d83971366f` |
| `batch-plan.md` | 3,770 | `4e41cb337c7d8555` |
| `companies/harborline-plumbing/fixture.json` | 11,826 | `8f77ac85ee88c3f2` |
| `companies/northlight-architects/fixture.json` | 14,858 | `a5a5f6e8b22f4421` |
| `companies/cedar-hollow-clinic-admin/fixture.json` | 16,881 | `1d4316631e0f9171` |
| `companies/meridian-haulage/fixture.json` | 18,900 | `8dbad76937c8c142` |
| `companies/ferngrove-school-office/fixture.json` | 20,323 | `56089ac90b61d2f8` |
| `companies/kelvinside-fabrication/fixture.json` | 24,447 | `d6d810ecfcec1929` |
| `cross-company/collisions.json` | 3,158 | `402d51dddbb11f94` |

`git -C ~/workspace/harness-build/a-l5-foldin status --porcelain -b` → clean on `docs/l5-run-evidence` (merged head
`5cde312`); no write under `docs/acceptance/`.

## 6. State / next

- Spend $0; no credentials; no sends; no package edit; no rehearsal run.
- **Next (self-claiming): batch 4** — a thin driver that reads a `fixture.json` and emits the rehearsal run plan/inputs in
  the package's documented evidence layout (without writing into `docs/acceptance/`), plus one labelled state-advanced
  batch run over the bank (honest clocks), report the counts. Then **batch 5** fold-in decision with Nox/Lead.
- Blocked items unchanged: the RUN on G1 (Lane 1 installed output); the package build on Nox C1–C6 + W1.
