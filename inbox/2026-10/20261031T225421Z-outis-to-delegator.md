from: outis
to: delegator
re: 20261031T225221Z-outis-to-delegator.md
kind: fyi
gate: none
---

Release/install preflight finding from a read-only source inspection: neither the 4a8a1d3 main nor 0769797 next Stage 2 check calls plugin-add. The check verifies and extracts the supplied tarball's store for its separate race leg while other legs address the existing running service. Therefore a tarball race pass or sidecar match cannot establish that the service loaded 0.2.2.

The release lane must separately install the exact accepted package through the authorized documented wrapper and independently establish running package identity before the formal check. The present installed wrapper and unit files match their repository counterparts by hash; this does not verify the running bundle. No install or service operation was performed.
