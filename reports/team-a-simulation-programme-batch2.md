# Team A — Ergon rehearsal SIMULATION programme prep: batch 2 report

**Order:** Nox capacity order `20261101T182200Z` pull item 2 (simulation programme prep as rehearsal-package **inputs**);
Lead accepted batch 1 and GO'd batch 2 ("more companies (15–30 staff), cross-company collision chains, validator
extension; self-claiming, report on disk"). **Status:** batch 2 on disk and self-verified; **prep only** — no package
edit, no rehearsal run, no send, $0 spend. **Successor:** batch 2b (third large company) then batch 3 (divergence depth).

## 1. What batch 2 added

Location: `board/probes/team-a-simulation-programme/` — now **11 files, 140 KB**. No edit under `docs/acceptance/`; the
ergon worktree is clean (verified `git -C ~/workspace/harness-build/a-l5-foldin status --porcelain -b` → clean on
`docs/l5-run-evidence`, still at the merged-PR head `5cde312`).

| File | Size (B) | SHA-256 (16) | Change |
|---|---:|---|---|
| `companies/meridian-haulage/fixture.json` | 18,537 | `dc567ef0c7867e47` | **new** — 15 staff, logistics/dispatch: depot + two routes + customer service; route/plate/window arithmetic, cut-off boundaries, dispatch-board audience mistakes, driver tacho shortcut, shared courier. 2 chains, 5 divergence cases. |
| `companies/ferngrove-school-office/fixture.json` | 17,454 | `b9ef7cb4c573bb0a` | **new** — 16 staff, education administration (main office, exam hall, site & lettings; administrative only, no pupil records): attendance/cover relays, exam-room change/cut-off, facilities, lettings clash, records scope. 2 chains, 5 divergence cases. |
| `cross-company/collisions.json` | 3,158 | `402d51dddbb11f94` | **new** — the cross-company collision class: 2 chains (shared pallet claimed by two companies' yards; site delivery reconciled between a studio and a hauler), each naming both companies with an end-of-chain rubric. |
| `validate.mjs` | 15,259 | `b7fd0f642675ca3b` | **extended** — cross-company checks (≥2 known companies, both referenced, end-act agent in a referenced cast, hop/rubric shape), 15+ staff floor, collision count in the summary. |
| `SPEC.md` | 11,155 | `6cc9ae4541255eb7` | contents/schema/successor sections updated to batch 2. |
| `README.md` | 4,561 | `7198d3cb5e3745ca` | table and counts updated. |
| `batch-plan.md` | 3,665 | `bfab3e35bc4f0948` | batch 2 landed (2 of 3 companies); third → **batch 2b**. |
| `companies/*` (batch 1: harborline, northlight, cedar) | unchanged hashes | `e2b8beac…`, `a1a236…`, `413bdaa3…` | unchanged |
| `divergence-cases.json` | 7,040 | `2b401d9f0028ebf9` | unchanged |

**Bank totals now:** 5 companies (6 / 9 / 12 / 15 / 16 staff; trades, professional, regulated-admin, logistics, education-
admin), 58 cast members with written voices, 10 chains (all six distortion kinds), 23 divergence-case instances across
all 12 kinds, 2 cross-company collision chains, 1 validator, spec/readme/batch-plan.

## 2. Verification (observed, 2026-11-01 ~14:05 ET)

**Positive:** `node board/probes/team-a-simulation-programme/validate.mjs` →
`SIMULATION BANK OK: 5 companies, 58 cast members, 10 chains, 23 divergence cases, 2 cross-company collisions (company sizes 6-16); 6/6 chain kinds, 12/12 divergence kinds covered; token-blind separation 173/174; secrets/PII scan clean` — **exit 0**.

**Negative (fail-closed, on throwaway copies; batch-1 proofs N1–N6 still hold):**

| Test | Change on a copy | Result |
|---|---|---|
| N7 unknown company reference | added `does-not-exist` to a collision's companies | exit 1, `XC-1 references unknown company "does-not-exist"` |
| N8 single-company collision | collision left with one company | exit 1, `XC-1 must reference >=2 companies` |
| N9 missing collisions file | deleted `cross-company/collisions.json` | exit 1, `cross-company/collisions.json missing` |
| N10 no 15+ staff company | shrank every 15+ staff company | exit 1, `programme must include a 15+ staff company (found sizes 12, 9, 6, 8, 9)` |

## 3. Honest limits

- **Batch 2 landed 2 of the planned 3 companies.** The third (22–30 staff) and 2 more chains moved to **batch 2b**; the
  plan file was updated rather than the count smoothed. The bank's sizes now reach 16, not the plan's 30.
- **Token-blind separation 173/174 is a proxy, not human-blindness** — it attributes sample lines by their own
  jargon/tics overlap (punctuation-normalised). The one residual miss is recorded, not hidden.
- **Cross-company collisions are still synthetic and shallow:** two cases, no shared master data model; the correct
  behaviour (surface the cross-company link, do not silently merge) is specified but not yet exercised by any run.
- **No run, no acceptance claim.** The bank is prep inputs; nothing here is rehearsal evidence, and no live/state-
  advanced claim is made. `packagePending: true` cases (2 now) stay in the bank and must be reported `pending`.
- **Fold-in deferred** to batch 5 (repo vs workspace), for Nox/Lead; batch 2 opens no question on it.
- Maker never checker: the validator is a maker check; a fresh-context checker reviews the bank (or its fold-in) when
  the Lead schedules it.

## 4. State / next

- Spend $0; no credentials; no sends; no package edit; no rehearsal run; no repo write.
- **Next (self-claiming): batch 2b** — one 22–30 staff company (small manufacturer or workshop) + 2 chains, then
  **batch 3** — per-kind divergence rubrics/evidence recipes; then batch 4 integration (thin driver, labelled run);
  batch 5 fold-in decision. Blocked items unchanged: the RUN on G1 (Lane 1 installed output); the package build on Nox
  C1–C6 + W1.
