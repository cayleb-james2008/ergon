# Team A — Ergon rehearsal SIMULATION programme prep: batch 1 report

**Order:** Nox capacity order `20261101T182200Z` pull item 2 ("Ergon rehearsal SIMULATION programme prep … built as
rehearsal-package inputs, NOT edits to the rehearsal package itself"), picked up by Team A after the front-end design
brief and the Duet resend draft. **Status:** batch 1 on disk and self-verified; **prep only** — no package edit, no
rehearsal run, no send, $0 spend. **Successor:** batch 2 (see §Next).

## 1. What landed (workspace bank, not the repo)

Location: `board/probes/team-a-simulation-programme/` — 8 files, 88 KB. Nothing under `docs/acceptance/` was touched;
the ergon worktree is clean and the merged rehearsal package is untouched (verified: `git -C
~/workspace/harness-build/a-l5-foldin status --porcelain -b` → clean on `docs/l5-run-evidence`; last commit remains the
merged-PR head `5cde312`).

| File | Size (B) | SHA-256 (16) | What it is |
|---|---:|---|---|
| `SPEC.md` | 9,824 | `feae9d82262af949` | Programme spec: why, rules, asset schema, how the bank plugs into the rehearsal as input, acceptance, successor batches. |
| `README.md` | 3,805 | `1e71c3ecbf4d1d10` | How to use the bank and the short-form rules. |
| `divergence-cases.json` | 7,040 | `2b401d9f0028ebf9` | The 12 divergence kinds: definition, correct handling, wrong handlings, evidence required. |
| `companies/harborline-plumbing/fixture.json` | 11,530 | `e2b8beaca6afd06e` | 6-person trades company (plumbing & heating). |
| `companies/northlight-architects/fixture.json` | 14,491 | `a1a2431579bfcf04` | 9-person professional studio (architecture). |
| `companies/cedar-hollow-clinic-admin/fixture.json` | 16,606 | `413bdaa3eff5d359` | 12-person regulated administrative setting (no clinical advice/records). |
| `validate.mjs` | 12,196 | `22342e2e7d375119` | Runnable validator (schema, voice distinctness, token-blind separation, chain shape, coverage, secrets/PII). |
| `batch-plan.md` | 3,578 | `fc22758df8371417` | Successor batches 2–5 and the fold-in decision (not pre-decided). |

## 2. What the bank contains

- **3 fictional companies, 3 sectors, 3 size bands** (6 / 9 / 12 staff), 9 sites, per-person devices plus shared kiosks,
  one free/open-source stack archetype per company, fixed seeds, seeded past and a named "what this exercises" line.
- **27 cast members**, each with role, device and a written voice (register, jargon ≥3, verbosity, tics ≥2, 3 authored
  sample lines). Vendored nothing; every person and company invented.
- **6 interaction chains** (2/company) with ≥3 hops, exactly one named distortion hop, end act on the agent, and a
  caught/missed/charged-ahead rubric. Programme coverage: **6/6** distortion kinds from the merged package
  (`detail-dropped`, `number-transposed`, `attachment-not-forwarded`, `casual-approval-as-formal`, `urgency-inflated`,
  `chains-collide`).
- **13 divergence-case instances across all 12 catalog kinds** (missing-info, urgency-inflation, expected-refusal,
  stale-data, conflicting-instructions, wrong-recipient, clock-boundary, unauthorized-scope, duplicate-report,
  tone-mismatch, partial-handoff, ambiguous-approval), each with setup, divergence, expected handling, evidence
  required, and a `packagePending` flag where the product piece does not exist yet.
- The bank is **input data**: a run config points at `companies/<id>/fixture.json`; the package's own files are not
  edited. See SPEC §5.

## 3. Verification (observed, 2026-11-01 ~13:45 ET)

**Positive:** `node board/probes/team-a-simulation-programme/validate.mjs` →
`SIMULATION BANK OK: 3 companies, 27 cast members, 6 chains, 13 divergence cases; 6/6 chain kinds, 12/12 divergence kinds covered; token-blind separation 80/81; secrets/PII scan clean` — **exit 0**.

**Negative (fail-closed proof, run on throwaway copies, each restored/discarded):**

| Test | Change on a copy | Result |
|---|---|---|
| N1 duplicate voice | copied one member's `register` onto another | exit 1, `curtis-nkemelu shares a register with another member (blind distinctness)` |
| N2 uncovered chain kind | changed a chain to `made-up-kind` | exit 1, names the invalid kind + the uncovered kind |
| N3 secret-like string | appended `sk-ant-…` to a seeded-past line | exit 1, `contains a Anthropic key lookalike — remove it` |
| N4 missing fixture | deleted one company file | exit 1, 13 named problems (cascade) |
| N5 usage | `node validate.mjs --bogus` | exit 2, usage text |
| N6 help | `node validate.mjs --help` | exit 0, usage text |

The validator also enforces: JSON well-formedness, companies ≥3 with varied sectors/size bands, cast ≥6 with unique
ids/names/registers/jargon sets, sample lines using ≥2 of their own tokens, chains ≥3 hops with matched distortion hop
and end-act agent in cast, rubrics non-empty, divergence kinds covered, no `.example`-outside emails or long digit runs.

## 4. Honest notes and limits

- **Token-blind separation 80/81 is a proxy, not human-blindness.** It attributes each sample line to the cast member
  whose own jargon/tics it overlaps most (punctuation-normalised). It proves the written voices are separable by their
  own vocabulary; it does not prove a person would distinguish them. The 1 residual miss is recorded, not smoothed.
- **Synthetic and shallow by design.** Three companies are a first tranche, not the varied bank the scale-up wants
  (sizes 15–30, cross-company collisions, per-kind rubrics are batch 2–3 in `batch-plan.md`).
- **No run, no acceptance claim.** The bank is prep inputs; nothing here is rehearsal evidence, and no live/state-advanced
  claim is made. Divergence cases marked `packagePending: true` (one now: the planning clock-boundary case) stay in the
  bank and must be reported `pending`, never dropped.
- **Fold-in not decided.** Whether the bank lands in the repo (new `docs/acceptance/simulation/`, docs-only branch + PR
  + fresh checker) or stays a workspace bank until the package build is authorised is batch-5 decision for Nox/Lead.
  Batch 1 does not open that question by itself.
- **Maker never checker.** The validator is a maker check; a fresh-context checker reviews the bank (or its fold-in)
  when the Lead schedules it.

## 5. State / next

- Spend $0; no credentials; no sends; no package edit; no rehearsal run.
- Queue for Team A after this batch: batch 2 of the simulation programme (3 more companies, 6 chains, cross-company
  collisions, validator extension) unless the Lead/Nox redirects. Blocked items unchanged: the RUN on G1 (Lane 1
  installed output); the package build on Nox C1–C6 + W1.
