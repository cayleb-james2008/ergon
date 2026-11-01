# FLEET-CHECKER — FORMAL STAGE 2 PASS SIGN-OFF (Mission 1 close-out gate)

Owner: `fleet-checker` (independent fresh-context checker; did not build the R6 work or the rehearsals). Date: **2026-10-31 21:32 ET (01:32Z)**.
Authority: Lead order 2026-11-01T01:24Z ("re-check the report against §3b; verify both rehearsals; judge the 3.4c classification; if every open row is honestly declared, sign the formal Stage 2 PASS, with rows 11/12/13/18/16 declared as ruled").

## 1. What this sign-off certifies

**TREE OF RECORD:** branch `integration/0.2.2`, head **`cf93107bc6e31ed265332b005d75b63d8acb300e`**, tree **`e15c17a3c93919de6b627e791a469dd9ddeefb28`**, package `@cayleb/dsh-fleet` **0.2.2** (`package.json` sha256 `ef93187a…`), remote ref `refs/heads/integration/0.2.2` = `cf93107` verified 19:49:29 ET. My evidence seat: `git archive cf93107` (tar sha256 `407d5f59…`); evidence root `board/probes/team-b-final-tree/checker-verify/` (INDEX inside).

**VERDICT: FORMAL STAGE 2 PASS — SIGNED**, over gate report rows 1–10, 14, 16 and 19 exactly as the checker certified them in `board/reports/fleet-checker-certification-cf93107.md` §3b, **with the open rows declared below and the B5 known limits carried (§4).**

## 2. Verified inputs on which the signature rests

1. **`board/reports/stage2-gate-report-DRAFT.md` (21:10 ET)** re-checked line by line against my §3b: the row table matches my certified statuses for rows 1–10, 14, 16, 19; rows 11 (UNVERIFIED/Nox), 12 (RAN, open), 13/18 (BLOCKED ON OWNER WAIT), 15 (OPEN) are honestly declared; **no PASS language appears on any unrun row**; §2c carries my three B5 known limits; §8 keeps every open row open; `abfb842` is stated as excluded per ruling B.
2. **Rehearsal evidence verified on `cf93107`** (`board/probes/team-b-final-tree/rehearsals/`; my verification `checker-verify/rehearsals-review/verification.md`): both legs bound to the tree/package/artefact identity above, commands and raw logs present, timestamps consistent.
   - **Stage 2 scratch, 100 entities: PASS 60/60, 0 fail**, `RACE_N=100`, scratch socket verified (`/tmp/tb-r9/control.sock`, never the real one), derived check diff limited to path/unit substitutions, 410-row invariant clean. **Verified.**
   - **Stage 3 default mode: 73 PASS / 1 FAIL / 24 MANUAL, verified.** The single FAIL is `3.4c` journald fresh-head delivery; `3.4a` chain verification PASSED in the same run (395 rows recomputed, every prev/hash matched). The 24 MANUAL are the Tier-B/polkit legs by design.
3. **3.4c judgement (mine, with corrected attribution):** the failure is a **scratch-environment contamination, not a package defect** — the "bad" journal lines are the **Stage-2 scratch host's own action-log chain head** (seq 2861, head `4d874599…`; identical to the final row of `scratch-tb-r9/s2home/fleet/log/actions.jsonl`), left running while the Stage-3 check ran. They are **not** from the real 0.2.1 service (the journal contains no `fleet-chain` lines from uid fleet at all). So: classification "SCRATCH LIMIT" **correct in kind**; the maker's stated cause ("shared with the real service") is **wrong and corrected here**; **the journald leg does not strictly require the real install** — it requires a **journal-clean window** (stop other fleet-core hosts) and would pass on a scratch re-run; the real install is the authoritative vantage. **3.4c is NOT claimed PASS in this sign-off.** It is a Stage 3 (Job 07) leg, so its non-pass does not block this Stage 2 gate, but it must not be recorded green.

## 3. Open rows — declared as ruled (none of them is certified PASS by this sign-off)

| Row | Declaration |
|---|---|
| 11 Five-high repro | **UNVERIFIED** — no valid verdict; stays named until the reviewers' existing repro scripts are run as five recorded tests (no new tooling). Nox disposition requested. |
| 12 Rehearsals | **EVIDENCED, not green:** Stage 2 scratch PASS 60/60 (verified); Stage 3 default-mode run recorded (73 PASS / 1 environment-caused FAIL / 24 MANUAL by design); **3.4c not claimed PASS** (see §2.3), to be settled by a journal-clean re-run or the real install. |
| 13 / 18 Release-identity | **BLOCKED ON OWNER WAIT** — installation paused by Cayleb's explicit Wait; no install/readback exists; `fleet-dsh` still 0.2.1; `/var/lib/fleet-drop` 0.2.0/0.2.1. The **artefact itself is verified** (sha256 `37e3a542…`, 82/82 packed files byte-identical to `cf93107`, version 0.2.2). **This sign-off does not certify the install or the running identity.** |
| 15 R6-10 remainders | **OPEN** — dispositions recorded incl. Security F9; Contract F11 has no recorded disposition; one regression test per verified high or an unverified name still owed. |
| 16 Repo reconciliation | **PASS for the integration tree** (main `4a8a1d3` ancestor; 0 deletions vs main; `evidence/` byte-identical; `abfb842` not an ancestor; candidate integrated). **Merge to `main` is the release lane's step.** |

**Mission 1 close condition is NOT yet fully satisfied** — "0.2.2 installed and identity-verified" (row 18) remains outstanding under the owner Wait, together with rows 11/15/16-merge. This sign-off records the **formal Stage 2 PASS over the gate evidence on the named tree**; it does not claim the Mission-1 close-out is complete.

## 4. B5 known limits carried (as required)

1. The walker no longer asserts the aggregate envelope; the replacement coverage is the composed spend regression tests (green on the final tree). Stated, not a silent gap.
2. The pinned `R6-10` seed-1546 test is a **fixture-path guard**, not a product-regression proof (it also passes on the pre-fix package under the new walker).
3. `board/reports/fleet-checker-walker-review-ee14591.md` is the independent review record for the "own reviewed change" requirement (verdict: SAFE and CORRECTLY SCOPED; test-only; 52 finding kinds unchanged).

## 5. Required record corrections before this DRAFT is published as the final report (text-only; no verdict changes)

1. **Header base facts:** remove "No 0.2.2 artefact exists." — false; the artefact is verified (row 18).
2. **§7 handoff status:** the walker review is **issued** (not "pending"); the artefact **exists and is verified**; the fresh-checker verdict is **this sign-off** (not "not started").
3. **Row 12 text:** replace the cause "host-global journal shared with the real service" with the corrected finding in §2.3 (the maker's own Stage-2 scratch host was the writer; journal-clean window / real install — not "needs the real install" as a precondition), and update "Leg stays OPEN until the checker verifies" → "checker-verified (§2.2/§2.3); Stage 2 rehearsal PASS; 3.4c not claimed".
4. **§9 provenance:** "no residue leg has run" is stale — residue legs (19:30–21:21) and both rehearsals (21:05–21:21) have run and are evidenced.
5. Optional: §7's residue-leg list predates the 19-leg residue bundle + rehearsals + `checker-verify/`; align it with `checker-verify/INDEX.md`.

## 6. Signature

Signed by `fleet-checker` for the **formal Stage 2 PASS** on `cf93107`/`e15c17a3`/0.2.2, with rows 11/12/13/18/15/16 declared as in §3 and the B5 limits in §4. **This sign-off is void if any open row above is later presented as PASS without its missing evidence, or if the certified tree hash changes.** Evidence: `board/probes/team-b-final-tree/checker-verify/` (25 verifiable legs + INDEX), `board/reports/fleet-checker-certification-cf93107.md`, `board/reports/fleet-checker-baseline.md`, `board/reports/fleet-checker-walker-review-ee14591.md`, raw logs `/home/cayleb/fc-checker-work/out/`.

— `fleet-checker`, 2026-10-31 21:32 ET
