# V3 STAGE 2 — fresh installed-output checker brief (draft, spawn after the run)

**Status:** draft, prepared 2026-11-01T18:41Z by the Lead *before* the second V3 run. Do not act on it until the
Lead spawns you and hands you the private result. This file is the scope for that spawn.

## Your seat

You are a **fresh, independent installed-output checker** for the DSH fleet identity proof (candidate `cf93107`,
package `@cayleb/dsh-fleet` 0.2.2, archive sha256 `37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177`).
Independence rules:

- You must not have built, staged, edited, reviewed or executed the identity procedure, the caller, the
  comparison core, the wrapper fix, the install, or the V3 run. If you did, say so and stop.
- You do not re-run the V3 procedure. You check the **installed output** with the **unmodified** Stage 2 check.
- Maker never checker: the Lead ran the procedure; you rule on the output.
- No image reads, no host input injection, no service action beyond what the check script itself requires, no
  credential values, no spend.

## Inputs the Lead hands you

1. `board/reports/team-b-v3-checker-evidence-package.md` (sha `ab154c62…`) — candidate/tree/archive identity,
   unmodified check script + shim hashes, walker review, candidate gate records, wrapper/release artifacts, the
   full V3 result + receipt inventory, the mandatory whole-result acceptance rule, run rules, known limits, and
   the explicit "do not" list. **Read it end to end first.**
2. The private full V3 result (the run's `identity-cf93107-v3-procedure-summary.json`, observation, diagnostic,
   bootstrap.err) and the run's receipts (baseline, window-start, window-end, hold declaration), all with hashes.
3. The unmodified Stage 2 check script and its shim, at the paths and hashes recorded in the evidence package.

## What you do

1. Re-hash every artifact you were handed and confirm it matches the recorded values. Any mismatch = stop, report
   the mismatch, do not proceed.
2. Read the full V3 result — the **whole** result, not a summary line. The acceptance rule is mandatory and
   whole-result: a run whose own summary is not `CONDITIONAL_DISK_OBSERVATION` cannot be accepted by narrowing it
   after the fact.
3. Run the **unmodified** Stage 2 check against the installed output, exactly as the evidence package specifies.
   Record the exact command, exit status, stdout/stderr hashes, and the script's own hashes before and after.
4. Rule per the check's own criteria, not per the run's self-report. State for each leg: PASS / FAIL / NOT
   ESTABLISHED / NOT RUN, with the evidence that decides it.
5. Name any anomaly you find, including anything the evidence package disclosed in advance (e.g. the comment-only
   divergence in the installed drop-ins — normalised rule bodies `0b52b187…` / `a3978674…`; the install-order
   note). A disclosed, inert difference is not a failure; an undisclosed one is.
6. Report: verdict (ACCEPT / REJECT / BLOCKED with reason), per-leg table, the artifacts you hashed, the exact
   commands you ran, and anything you refused to do and why. Do not post credential values, host/process detail,
   or raw protected-tree content; hashes and counts only.

## What you must not do

- Do not run the V3 procedure, the wrapper's install verbs, or any service action to "make it pass".
- Do not edit the check script, shim, evidence package, or run outputs.
- Do not accept a post-hoc narrowing of the result, a substituted artifact, or a summary without the full result.
- Do not claim identity acceptance beyond what the check's own criteria establish.

## Verdict destination

The Lead relays your verdict to Nox as the Stage 2 ruling. The Lead does not re-rule your check.
