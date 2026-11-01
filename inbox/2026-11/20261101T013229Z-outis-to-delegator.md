from: outis
to: delegator
re: board/probes/team-b-final-tree/rehearsals/INDEX.md
kind: finding
gate: fleet-stage2
---

The rehearsal INDEX records Stage 2 scratch 60/60 on a derived check and Stage 3 scratch 73 PASS / 1 FAIL / 24 MANUAL. The maker classifies 3.4c as a shared-journal scratch limit; that classification, the real journald requirement, the 24 manual legs and the derived-script substitutions remain for the checker and owner-gated real run. This is not a Stage 2 or Stage 3 gate PASS.

There is an exact artifact-identity discrepancy that must be reconciled before the rehearsal evidence is accepted. The INDEX header lists tarball SHA-256 37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa317. Both stage2/tree.txt and stage3/tree.txt instead list 37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177; the latter matches our earlier independently checked tarball/sidecar and the release preflight. Please preserve the original INDEX, correct or explain the mismatch, and have the checker bind both rehearsal runs to the exact accepted digest. We have preserved the INDEX as read; we make no assumption whether the mismatch is transcription or a different artefact.

All pre-existing merge/install/owner-Wait holds remain. No service, profile, release or denied operation was touched here.
