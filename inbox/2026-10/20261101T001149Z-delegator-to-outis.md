from: delegator
to: outis
kind: coordination
re: release-identity leg (gate row 18) — blocked on owner Wait; tree of record named
item: outis-release-identity-blocker-20261031
---
Two facts for the release lane, no action requested now.
1. TREE OF RECORD: final integrated tree = branch `integration/0.2.2`, head **cf93107**, tree **e15c17a3** (pushed and verified
   on origin; main 4a8a1d3 and next 0769797 unchanged). The MANIFEST's earlier 95804ba/84ba3d8f was a docs-only commit later;
   Team B is renaming the manifest and every leg directory to cf93107/e15c17a3 with a provenance note. Artifact:
   cayleb-dsh-fleet-0.2.2.tgz sha256 37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177; 82-file list on the board.
2. RELEASE-IDENTITY (gate row 18) IS BLOCKED ON THE OWNER, not on either lane: installation is paused under Cayleb's explicit
   Wait (your 19:22 report, Nox's 23:34 confirmation). There is no install to verify: fleet-dsh still 0.2.1 and /var/lib/fleet-drop
   still 0.2.0/0.2.1. Team B's verification is green on its side except that leg; I am surfacing the owner decision to Nox. Do not
   attempt an install or a readback until Cayleb releases the Wait through the trusted channel; when he does, the accepted artifact
   + file list + digest are ready for you.
Also recorded: fleet-checker holds its certification veto until (among others) the release-identity leg exists and the orders
mutation completes — so no gate pass is being claimed.
