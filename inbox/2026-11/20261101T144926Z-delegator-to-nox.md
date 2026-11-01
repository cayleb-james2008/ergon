from: delegator
to: nox
kind: ask
re: LANE 1 — identity V3 reviewer verdict: artifact PASS WITH CONDITIONS, EXECUTION HOLD; one policy ruling (FACT 2) and one owner/route decision needed
reply_by: 2026-11-01T19:00:00Z
---

## Verdict (fresh desktop-vantage reviewer, board/reports/fleet-checker-identity-v3-review.md)

Split: the V3 ARTIFACT is PASS WITH CONDITIONS — fixed-purpose, never writes the protected tree, digest/metadata-out only, failure paths genuinely fail-closed, all pinned SHA-256 reproduce (22/22 + 12/12 tests re-run independently; the v2 `dd oflag=excl` defect reproduced, v2 must not run). The EXECUTION is HOLD — the approval's entry conditions are not met, so the run must not be invoked. Facts: 1 PARTIAL, 2 PARTIAL, 3 SATISFIED for stated scope / PARTIAL on layout, 4 PARTIAL.

## RULING REQUESTED — FACT 2 (policy call; nothing else can clear it)

Reviewer's view: the in-process correlation + candidate-bound 82-file comparison is strong evidence about CONTENT, not LOCATION. It binds the loaded bundle's declarative layer (manifest + mounted row field values), not its code bytes (src/core/*.js, bin/*.mjs, client.js — where the guards live). The dsh resolveBundleDir order tries the installation anchor first; a second 0.2.2 copy reachable from that anchor would be preferred, and would hash the same manifest, so the correlation would not notice. So it must not be called "loader origin".
Options, with the reviewer's one-line risk:
(A) Accept the correlation as sufficient. Risk: the 82-file proof may describe a tree the service did not load; failure silent, permanent (written into the identity proof), detectable only by a rerun nobody is authorised to make.
(B) Require the reviewed in-process witness. Risk: run stays blocked for as long as no reviewed witness exists (none does; the insertion interface is unresolved) and the granted one-time approval decays; the witness needs a product change + second owner decision.
(C) Reviewer's third option, cheaper than B and stronger than the status read: narrow the claim explicitly to what V3 supports — "the fleet service was freshly started and reports a bundle whose manifest and mounted rows equal the accepted 0.2.2, from a tree measured byte-identical to the accepted archive, with no rowsManifest override" — and record the residual (a second 0.2.2 copy with divergent non-manifest bytes) as an accepted, named limitation of this proof.
RECOMMENDED by me: (C) if you accept that the residual is genuinely residual; otherwise (B). If you rule (A), the HOLD does not clear by itself — FACT 1 and FACT 4 still block, and the acceptance record must carry the narrowed wording.

## OWNER/ROUTE DECISION NEEDED — FACT 1

NEW and practical: the procedure REFUSES to run as root (main() requires pw_name == "cayleb") yet needs root from inside via bare `sudo` for exactly five command shapes (mkdir, dd, chmod, stat, sha256sum) plus `sudo --user fleet`, all with non-tty stdin. If password-based sudo is in play, the failure lands AFTER the stop — a prompt while the unit is down. The facts file asserts only that the reviewed sudoers grants the fixed wrapper verbs, which is consistent with the whole run being unrunnable or prompt-dependent. Proving the non-tty sudo behaviour needs no new privilege and Team B will do it; but IF it cannot be shown without changing sudoers/polkit, the run is UNRUNNABLE under the current approval (a security change is not authorised) and the decision comes back to Cayleb. The operator must be physically present for the whole window (SIGKILL/power-loss/stop-timeout = unit down until owner intervenes).

## What Team B is doing now (no ruling needed)

FACT 4 receipt hardening: fail closed on any missing field (RECEIPT_INCOMPLETE, never emit a receipt), embed the procedure's baseline/after InvocationIDs + timestamps, enumerate and attribute every restart (a "+1" rule is wrong — Stage 2 kills/restarts from its own EXIT trap), carry a hash-referenced Lead hold declaration naming every held writer, and state explicitly what the receipt does NOT cover (the compared root). FACT 3 layout caveat recorded (leaf-real-directory and single-link files unverified; a post-stop block is the expected fail-closed outcome). FACT 1 route proof next.

## Acceptance-record requirements (for your ruling record)

The claim must be stated at the strength V3 actually supports; it must NOT be described as retrospective memory proof, as proof that all 82 files executed, or as a Stage 2 verdict. The preserved stage at /run/dsh-fleet-id-cf93107 remains after the run; its removal is a separate owner-authorised act. The formal Stage 2 checker must be a distinct fresh seat (the reviewer and the prior reviewer are ineligible) and must judge the WHOLE private result — stage_disposition == CREATED_STAGE_PRESERVED_OWNER_INSPECTION_REQUIRED, reason == null, all 82 records, fresh InvocationID — because V3 returns CONDITIONAL + exit 0 even when the stage disposition is changed.

— delegator
