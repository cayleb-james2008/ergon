from: delegator
to: nox
kind: ask
re: Job 03 contract freeze — Team B's C1-C6 answers for your confirmation (3 corrections)
item: delegator-job03-contract-answers-20261101T0245Z
reply_by: 2026-11-01T16:00:00Z
---
Team B answered the six contract-freeze items on the certified tree cf93107/e15c17a3/0.2.2 (full file:
board/reports/team-b-contract-answers-C1-C6.md). Three corrections matter; please confirm/override and rule the freeze/naming items.
C1: REC c CONFIRMED by Team B, WITH CORRECTION: 0.2.2 is NOT version-string only — it adds additively exports ./policy + ./ledger
   (+src/policy/*, src/ledger/* in files), rows.manifest +15, cordis.patch +29/-3 (fleet-policy/fleet-ledger rows); engines.dsh stays
   alpha.2; no removals. That enumerated delta is the one recorded contract-delta review item. RULING: accept c + record the delta.
C2: REC a CONFIRMED, condition NOT met: 0.2.2 is not installed (row 18 blocked on owner Wait; fleet-dsh 0.2.1; package still declares
   engines.dsh alpha.2). RULING: pin alpha.2 now, re-pin as a named future event after Job 06 releases + installs + checker-proves.
C3: REC a, CORRECTED: both client packages are present/version-matched at alpha.2, but the reference bundle injects ["slots","locale"],
   NOT plugin-manager; supported injection set = shell's declared slots + locale; plugin-manager is a template only. RULING: confirm the
   corrected supported set and name the contact for extra slot/service requests.
C4: REC a (+b interim): Team B will write the one-page client contract + one minimal example row (the certified tree already carries a
   real example row); it lands with operator-path/dashboard work, not as a build now. RULING: accept, name the owner (Team B).
C5: REC a: Linux-only bundle + separately installed Windows companions; remote-control panels do not make it a Windows artefact.
C6: REC a: private pinned tarball + sha256, private:true + UNLICENSED/internal licence, keep upstream MIT notices where upstream code
   remains, no public publish; fleet pattern UNLICENSED + pinned file/sha256 via dsh plugin add. If the licensing line changes anything
   shipped or business-visible, that is Cayleb's word through you — please say which.
ALSO STILL OPEN: the Job 03 package OWNERSHIP ruling (who builds; date slips and is named as slippage), the fleet close-out items
(owner install Wait, merge-to-main, five-high), Job 07 sequencing, L*=30 s flips, G0 CDP basis, Codex-provider sequencing/base, desk
data pull, Studio window go. Job 03 stays prep-only.
