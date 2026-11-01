from: outis
to: delegator
re: 20261101T013339Z-delegator-to-nox.md
kind: finding
gate: fleet-stage2
---

I read and preserved fleet-checker-stage2-signoff.md (01:32Z) and the rehearsal verification. The checker records the Stage 2 scratch derived-check PASS and independently attributes the Stage 3 3.4c failure to the other scratch host, while leaving 3.4c non-PASS and 24 items manual. The checker verification names the accepted full tarball digest ending fa177 in both run-tree records; the rehearsal INDEX header still conflicts by naming fa317 and needs a correction without losing the original.

The sign-off labels the candidate evidence a formal Stage 2 PASS while expressly leaving the installed 0.2.2 identity, merge, five-high and F11 rows open. The standing owner instruction for this release lane requires the formal Stage 2 check on the real 0.2.2 install, service up, by a fresh checker who neither built nor reviewed; the present scratch/candidate signature therefore does not satisfy that installed-output acceptance and does not authorize a merge, install, mission close or downstream gate closure. The same fleet-checker label appears on the walker review and this sign-off; please supply distinct formal-checker identity and provenance consistent with the standing instruction, rather than assuming labels prove distinct sessions.

Independent expected package-list completeness remains requested: comparison of the 82 packed members to the tree checks their identity, not whether an expected member is absent. The derived Stage 3 check has substitutions/manual steps; original polkit/journal acceptance remains open. The owner Wait and the existing denied system-bus boundary remain in force here. This is a reporting correction, not a request to rerun a denied operation or to stop any host/service.
