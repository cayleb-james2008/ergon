# TEAM B — CONSOLIDATED STAGE 2 GATE PACK (corrected FINAL + INDEX + F11 Block I + host facts)

**Maker deliverable for Nox's gate ruling. Not a PASS, not a self-certification.** Filed 2026-11-01T14:30Z by
`fleet-lead` (Team B). This index names every component, its digest, and its status. The formal Stage 2 PASS on
`cf93107` (independent checker, 2026-10-31 21:32 ET) stands on its own record; the rows Nox dispositioned are
corrected here so the pack is self-consistent.

## Components (all paths relative to the workspace root)

| # | Component | Path | SHA-256 | Status |
|---|---|---|---|---|
| 1 | **Corrected FINAL gate report** — original 21:36 ET report + new **§10 Amendment Register** superseding §1 statuses for rows 11/12/13/15/18 (no row becomes PASS; Mission 1 still not closed) | `board/reports/stage2-gate-report-FINAL.md` | `5df0093263c4aaebf203cb5249f559a1f76de87b1e360ea194569c0c07a9ff3d` | FILED |
| 2 | **INDEX correction** — rehearsal INDEX header digest `…fa317` → `…fa177`, erroneous string preserved, three-way anchoring recorded (release preflight, sidecar `sha256sum -c`, both run-tree records) | `board/probes/team-b-final-tree/rehearsals/INDEX.md` | `7ee7f4b12e5028edc24a5b932b4a8a020e3c9b963c9661e56520b30cec4027bc` | FILED |
| 3 | **F11 Block I amendment** — exact replacement Block I text naming long-context tiers + cache rates as the pricing basis; code evidence (`ledger.js` `0a682cb4…`, `API.md` `864826e3…`, `index.js` `a60f5d92…`); harm direction recorded (over-prices, fail-conservative) | `board/reports/team-b-f11-block-i-amendment.md` | `fe790e02b15d5101547f4aebd253f73b7cee7d7586700ed01787c344730a8b02` | FILED — F11 closes by this text landing in the pack |
| 4 | **Four host facts (Lane 1)** — digest-level verdicts: read route established / execution route owner-level only; loader origin partial with the gap named; top-level mode+ownership established; no-writer window definable + quiet baseline receipt | `board/probes/team-b-host-facts-20261101T1402Z.md` | `fcfb700ed9f79846c1288c415196260c9c54aa9157aed81e0ac74613fc37714f` | FILED to Nox (14:01Z) |
| 5 | **Identity V3 independent review packet** — fresh desktop-vantage reviewer instructions, pinned inputs, read-only checks, five review questions, hard constraints | `board/probes/team-b-identity-v3-review-packet.md` | `1f1aa9fa4064f0e0de811e4338718f4a84ab9b3ee873f137f8e98fc3e91e7482` | FILED; reviewer running (Lead-spawned) |
| 6 | **Window receipt script v3** — read-only no-writer-window baseline/start/end receipts, reviewer-hardened (fail-closed `RECEIPT_INCOMPLETE`, procedure-invocation tie-in, in-window restart enumeration + separate baseline proof, hashed Lead hold declaration, explicit blind-spot statement); NNP-free launcher via `systemd-run --user` | `~/workspace/harness-build/scratch-tb-l1/window-receipt-v3.sh` (sha `6c410a0e…`) + `window-receipt-run.sh` (sha `273fa8d3…`); v1/v2 preserved | dry-run PASS: start `779ba03b…`, end `aaac7c86…`; real window-end awaits the run | TESTED (dry-run PASS + fail-closed cases) |
| 7 | **Host-facts addendum + reviewer verdict** — reviewer `board/reports/fleet-checker-identity-v3-review.md` (HOLD execution / artifact PASS-with-conditions); Team B addendum after the review: FACT 1 proven **negative** (root sudo password-required for all five staging shapes; only fixed fleet wrapper verbs are NOPASSWD ⇒ **V3 UNRUNNABLE under the current approval, owner decision needed**), FACT 3 layout caveat, FACT 4 receipt v2, FACT 2 unchanged | `board/probes/team-b-host-facts-addendum-20261101T1505Z.md` | `fcfb700e…` (facts) + addendum | FILED |
| 8 | **Identity V3 run record (one-time approval consumed)** — executed once 15:46:23–15:47:40Z, **fail-closed with a real finding**: caller refused on `package tree failed comparison: full file mode mismatch`; service restored (`e44cc343…`, NRestarts 0); stage preserved; receipts `af2f99fc…` / `3843b354…` / `311d4760…`; **root cause diagnosed and reproduced: the authorised wrapper's `umask 027` normalises archive modes (0644→0640, 0755→0750; 58 files) — under umask 022 the same installer is 82/82 archive-identical**. Identity FAILED for this install; fresh owner approval required for any second attempt. | `board/probes/team-b-identity-v3-run-record-20261101T1555Z.md` | FILED |
| 9 | **Installer-fix proof** — scratch proof that a fixed post-add mode-normalisation (de-link via copy+rename, set archive member modes, chmod package dirs 0750) makes the pinned `identity_snapshot_core.compare_open_tree` **PASS** (82 files, archive `37e3a542…`); also proves install-time umask alone is insufficient (58 mode diffs + 82 hardlinks from the pnpm store). Proposed fixed wrapper change for review. | `board/probes/team-b-installer-mode-fix-proof-20261101T1607Z.md` | FILED |
| 10 | **Identity v4 amendment draft (policy alternative to the fix)** — exact replacement text for the mode leg (`expected_mode = archive_mode & ~0o027`, no widening; single-link/ownership/dirs/bytes unchanged), named residual, trade-off table, deliverables if chosen. Draft only, not applied. | `board/reports/team-b-identity-v4-amendment-draft.md` | DRAFT |

## Correction ledger (what changed, why, on whose ruling)

1. **Row 11 (five-high):** Nox `20261101T034100Z` — all five CLOSED; H1/H2 carry one shared desktop-coverage residue
   (policy mutants idx 40/54 judged only by composed-host suites; died on desktop WIP `5184978`), a coverage boundary,
   not a defect. Row 11's literal framing is unsatisfiable (repro scripts never committed); the R6 regression tests
   are the executable successors carrying the verdicts. Not claimed as a PASS of the literal framing.
2. **Row 12 (rehearsals):** INDEX transcription error explained by Nox (61/64 hex shared; `fa177` anchored three ways)
   and corrected in place with the erroneous value preserved. Stage 3 `3.4c` stays a Job-07 leg; 24 MANUAL are
   Tier-B/polkit by design.
3. **Row 15 / F11:** Nox ruled the ledger's tiers + cache rates are the real pricing basis; contract text was behind
   the code. Closure = Block I amended to name them as the basis — text filed here. Security F9 unchanged (exact
   reason and future home recorded).
4. **Rows 13/18 (install + release-identity):** install executed 2026-11-01 03:45–03:46Z through the authorised fixed
   wrapper (accepted archive `37e3a542…`; `0.2.1 → 0.2.2`; restarted, active/running). The approved one-time V3 identity
   run was then executed once on 2026-11-01 15:46:23–15:47:40Z and **FAILED CLOSED with a real finding** — caller
   refused: `package tree failed comparison: full file mode mismatch`. Root cause diagnosed + reproduced: the wrapper
   runs `umask 027`, so the sanctioned installer produces archive modes masked (0644→0640, 0755→0750; 58 files; package
   root 0750); under umask 022 the same installer is 82/82 archive-identical. Identity is **FAILED for this install**;
   the approval is consumed; no retry; a second attempt needs a fresh owner approval after the installer fix or a
   reviewed contract amendment. Preserved stage inspected (three root 0444 files match pinned digests); removal is a
   separate owner-authorised act. Formal Stage 2 on the installed output: NOT RUN.
5. **Row 16:** `integration/0.2.2` @ `cf93107` remote-verified; `abfb842` excluded; merge to `main` remains Nox's
   ruling / release owner's step. No merge, no force-push.

## What is NOT in this pack, and must not be inferred

- No installed-byte / loader-origin proof is claimed (see fact 2's named gap: canonical directory/inode binding and
  later dynamic loads require either a reviewed in-process witness — a new owner decision + product change — or an
  explicit checker ruling that the in-process manifest correlation plus the approved caller's candidate-bound
  comparison suffices). **Nox rules that policy; Team B names it.**
- No formal Stage 2 check on the installed output has run; it needs the identity chain + a fresh eligible checker who
  did not build or review the candidate or the helper.
- Merge to `main` is not done and not authorised by this pack.
- `abfb842` stays excluded from release ancestry.

## Gate ruling now needed (Nox)

1. Accept this pack as the maker-side correction of items (1), (2) and F11.
2. Rule the fact-2 loader-origin policy (witness vs checker ruling) — Lead has filed the request with Team B's evidence.
3. Rule on the merge of `cf93107` into `main` (release owner's step, no force-push) and on Mission 1 closure once the
   installed-output identity + formal check have run.

## Successor

Identity V3 review → operator run (owner-controlled seat, one invocation, exclusive window with receipts) → fresh
installed-output checker runs the unmodified Stage 2 check → Nox gate ruling. In parallel: Lane 2 isolated scratch
acceptance, Lane 3 Stage 3 red-team package, Lane 4 surfaces/rig.
