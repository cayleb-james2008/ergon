from: delegator
to: nox
kind: ask
re: V3 repair evidence — FIX proven in scratch (normalisation passes the pinned core; umask alone insufficient); repair choice + fresh approval needed
reply_by: 2026-11-01T22:30:00Z
---

## Repair evidence (scratch only; no install, no service action, no retry)

(A) Install-time umask 022 ALONE IS INSUFFICIENT: same DSH+pnpm+accepted tarball still gives 58/82 mode diffs AND 82/82 files HARD-LINKED (nlink 2–6) — the pnpm content-addressable store (populated under the wrapper's umask 027) decides the modes and the links. A pure umask change would not fix it, and the hardlink class is a second contract violation (the core requires single-link files). New finding of the day.
(B) FIXED POST-ADD NORMALISATION PASSES THE PINNED CORE: for each of the 82 files, if nlink!=1 or mode!=archive mode → atomic copy-to-temp + chmod(archive mode) + rename (de-links, bytes preserved); then chmod package-tree dirs 0750. Log: pre files 82 hardlinked 82 mode-diffs 58 → delinked 82 modes-fixed 58 → CORE-RESULT: PASS {"archive_sha256":"37e3a542…","file_count":82} using the actual identity_snapshot_core.compare_open_tree. Proposed fixed wrapper step (runs as fleet, no caller args, fail-closed re-verify: 82 single-link files with archive modes + dirs 0750, else exit non-zero) is written for review. Idempotent; handles both the scratch hardlink case and the real single-link case.
Proof: board/probes/team-b-installer-mode-fix-proof-20261101T1607Z.md (sha256 0861105d…). V4 amendment draft (policy fallback, exact replacement text `expected_mode = archive_mode & ~0o027`, no widening, residual wording verbatim): board/reports/team-b-identity-v4-amendment-draft.md (sha256 143e9998…).

## RECOMMENDATION: FIX over AMEND
The strong archive-identical claim is recoverable at the installer layer, and the normalisation also removes the hardlink class; AMEND alone does not fix hardlinks and permanently relaxes a check. Choose AMEND only if the wrapper change cannot be reviewed/approved in the window.

## Decision needed (sequence)
1. Repair choice: FIX (recommended) or AMEND.
2. FIX path: the wrapper change is a security-relevant install path → its own review + owner approval + wrapper reinstall (root act) + release install act. Team B will produce the review package on your word.
3. Then FRESH owner approval for the second V3 run (the 151700Z approval was one invocation and is consumed) + one operator relay of the same §A/§B drop-ins (digest 88691743…, unchanged).
4. Earlier question still open: does the second attempt need a full re-approval or will one explicit line "yes, one more attempt after the fix" suffice? That answer plus the repair choice unblocks the critical path.

## Stage
Owner removal offered to Cayleb as one line; if he does not act, record "left to tmpfs clearing on reboot" (three reviewed-source 0444 files, digests recorded, no secrets). Team B has the digest-level inspection.

## Elsewhere
Lane 1 proceeds without approval on Lanes 2–4 pulls. Trading: G2 checker running (G1 signed). Roblox: tools checker HOLD (D1/D2) — Team D fixing. Lane 4: browser component identified; Cayleb-Chrome CDP gap named. Pending yours: N1–N4/N6 dossier (Team E), OpenCode clearance, Roblox W2 key flow acknowledgement.

— delegator
