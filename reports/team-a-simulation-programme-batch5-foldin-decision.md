# Team A — simulation programme batch 5: fold-in decision paper (decision input only)

**STATUS: DECISION INPUT — NOT DECIDED, NOT ACTED.** Nothing has been created in the repo, no branch, no PR, no commit,
no `docs/acceptance/` write. This paper is for the Lead to take to Nox; Team A does not decide repo placement of the
simulation bank. Prepared 2026-11-01 under the Lead's rule ("Fold-in decision stays for batch 5, brought to me with the
report — I take it to Nox").

## 1. The question

Where does the Ergon rehearsal simulation bank live going forward?

- **Option A — fold into the repo now**, as a **new** `docs/acceptance/simulation/` area on a docs-only branch + PR +
  fresh checker (no edits to the merged rehearsal package, no product code, no runtime).
- **Option B — stay a workspace bank** (`board/probes/team-a-simulation-programme/`) until the package build is
  authorised, then decide with the package's input contract in hand.
- **Option C — split**: fold the docs-only spec/README/validator contract now, keep the company fixtures workspace-side
  until the package schema is ruled.

## 2. What would be folded (evidence)

The bank as it stands: 20 files, 248 KB, at `board/probes/team-a-simulation-programme/`.

- 6 fictional companies (6/9/12/15/16/22 staff; trades, professional, regulated-admin, logistics, education-admin,
  manufacturing), 80 cast members with written voices, 12 interaction chains (all six distortion kinds, end-scored),
  28 divergence-case instances across all 12 kinds, 2 cross-company collision chains.
- Machine-checkable divergence rubrics: 12 `handlingKind` classes + `refusalExpected` flags, case↔catalog consistency,
  `validate.mjs` fails closed on rubric gaps (proofs N11–N14) and on voice/chain/secret/PII problems (N1–N10).
- A labelled simulation-bank harness (`harness.mjs`: plan / self-test / verify) and a labelled bank self-test artifact
  set; every artifact carries "BANK SELF-TEST — … NOT the G1 run. NOT an acceptance rehearsal. Not evidence." in content
  and file name; `verify` fails closed on a missing/misnamed label (N18–N19).
- It is **input data**, not product code: no imports into the package, no runtime, no model calls, no network; the
  harness writes only under `--out`; nothing touches the merged rehearsal package (`docs/acceptance/demo-company/**`
  unchanged) or the ergon worktree (clean at merged head `5cde312`).

## 3. Option A — fold in now (docs-only branch + PR + fresh checker)

| | |
|---|---|
| What lands | New directory `docs/acceptance/simulation/` containing SPEC.md, README.md, divergence-cases.json, `companies/**`, `cross-company/collisions.json`, `validate.mjs`, `harness.mjs`, and the labelled self-test output. **No** file outside the new directory changes except `docs/EXIT-CODES.md` (two new entry points must be tabled or the suite fails) and a pointer row in `docs/acceptance/` if desired. |
| Process | Branch cut from then-current `main` (fetch main explicitly in this clone); docs-only PR; fresh-context checker re-runs `validate.mjs` + harness plan/self-test/verify + the entry-point/exit-codes tests; Lead/Nox merges. Maker never checker. |
| Benefits | Source of record lives with the acceptance work; checker provenance; usable by the package/rehearsal work the moment G1 lands; versioned with the tree; ends the workspace-only drift risk; no runtime risk. |
| Costs / risks | One checker cycle; a PR while `main` moves (mitigable: rebase before PR); the bank could be mistaken for the rehearsal package (mitigable: README boundary text, the BANK SELF-TEST labels, and no edits to `demo-company/`); `EXIT-CODES.md` must table `validate.mjs` and `harness.mjs` (mechanical). |
| Does it touch the RUN or the package? | **No.** Not the G1 run, not an acceptance rehearsal, not the package. Purely new docs + two entry-point rows in an existing doc. |

## 4. Option B — stay workspace until the package build is authorised

| | |
|---|---|
| Benefits | Zero repo churn now; the package build may rule an input contract that changes the schema; nothing to maintain against a moving `main`; the bank is self-contained and re-usable as-is. |
| Costs / risks | Not visible to repo readers or builders; no checker provenance; drift/duplication risk (someone rebuilds it inside the package); workspace durability depends on this machine; the later fold-in becomes a bigger, older diff. |
| Does it touch the RUN or the package? | No — but it also cannot be consumed by the package without a later fold-in step. |

## 5. Option C — split (spec now, fixtures later)

| | |
|---|---|
| Benefits | Puts the contract (schema + validator) in the repo for review while the company data stays flexible. |
| Costs / risks | Two fold-ins instead of one; a spec without its fixtures is harder to checker-verify (the validator needs the assets); the package build would still need the assets path. |
| Assessment | Strictly more process for less evidence than A, unless Nox wants the schema frozen before the fixtures are reviewed. |

## 6. Recommendation

**Option A, docs-only, with the boundary stated in the README and the checker scoped to it.** Reasons:

1. The bank is ready and self-verifying; the only remaining work is procedure (branch, PR, checker), and that procedure
   is exactly what makes it trustworthy for the package/rehearsal work later.
2. It is **inert**: no code imported, no runtime, no model calls; the risk profile is a docs PR, not a build.
3. It removes the largest risk of Option B — a workspace-only asset that a future builder rebuilds or drifts from.
4. It gives the package build the divergence/chain input contract **before** G1, so the rehearsal can consume it without a
   second rework.

**Conditions on Option A:** cut from then-current `main`; new directory only (plus the two EXIT-CODES rows); README
boundary text and the BANK SELF-TEST labels kept verbatim; fresh-context checker re-runs `validate.mjs`, the harness
`plan`/`self-test`/`verify`, and the exit-codes/package-contents tests; no edits to `docs/acceptance/demo-company/`.

**If Nox prefers to wait**, Option B loses nothing today: the bank is self-contained, the validator and labels are
carried in it, and the fold-in can be done as a single docs-only batch once the package input contract is ruled.

## 7. What the Lead should ask Nox

1. Which option: A (fold now, docs-only + checker), B (stay workspace until the package build is authorised), or C (split)?
2. If A: scoping confirmation — new `docs/acceptance/simulation/` area only, `docs/EXIT-CODES.md` entry-point rows, no
   edits to the merged rehearsal package, fresh checker on the docs-only PR. Nothing here moves the RUN or the package
   gates, and this paper does not assume approval.
3. If B: no action; the batch-5 record closes as "stays workspace, no repo write", and the fold-in reopens with the
   package build.

## 8. State (nothing acted on)

- No branch, no PR, no commit, no `docs/acceptance/` write, no repo change. The ergon worktree remains clean at the
  merged-PR head `5cde312`; the bank remains at `board/probes/team-a-simulation-programme/`.
- Spend $0; no model calls; no credentials; no sends.
- Bounded evidence for the decision: bank inventory (20 files/248 KB), `validate.mjs` PASS, harness label `verify` PASS,
  fail-closed proofs N1–N19 (all named in the batch reports).
