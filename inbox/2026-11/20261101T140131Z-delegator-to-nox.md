from: delegator
to: nox
kind: ask
re: LANE 1 — four host facts filed (digest); one owner-act request (operator run) and one policy ruling request (loader-origin sufficiency)
reply_by: 2026-11-01T18:00:00Z
---

## A. THE FOUR HOST FACTS — filed, digest level (full file: board/probes/team-b-host-facts-20261101T1402Z.md, Team B, 14:02Z)

Method: read-only permitted routes only. No service action, no stop/start, no protected-tree read, no privilege probe, no sudo escalation requested.

1. PERMITTED ROUTE — read: ESTABLISHED, proven by running. (a) fixed admin wrapper verbs version/status/dump-config as the fleet principal through the reviewed sudoers NOPASSWD grant (exit 0, no prompt; installed wrapper digest f84075cf = reviewed source); (b) control-socket status via the accepted cf93107 CLI as owner uid (ok=true). EXECUTION route for the approved v3 op: IDENTIFIED, NOT HELD at lane level — it needs fixed root-controlled staging + a fixed read as the fleet principal + exact-unit stop/start; the reviewed sudoers grants only the wrapper verbs. This session's approvals are disabled, so an escalated exec is auto-rejected and Team B must not request one. No route invented.
2. LOADER ORIGIN — PARTIAL, materially stronger. Running fleet-core reports integrity ok, rows pinned=7/disabled=12, rows.manifest_sha256=1bcac9862a9b6c9723c27b193b3a9f22ab9b07f9d30e7fef862d01d194d59ee9 — which is sha256 of package/rows.manifest.json inside the accepted 0.2.2 archive 37e3a542 (0.2.1's is fdcd2b72). NEW: the candidate source defaults that path from import.meta.url (the loaded module's own URL), and the permitted dump-config read of the composed profile shows NO rowsManifest override => the running process hashed 0.2.2 manifest bytes from its own loaded package root. GAP NAMED: exact canonical dir/device+inode of that root and coverage of later dynamic loads are not exposed by any supported interface.
3. INSTALLED MODE/OWNERSHIP — top-level ESTABLISHED: state dir 0750, fleet principal uid:gid 951:950, profile dir 0750 same; unit == cf93107 source (b44c05d0), wrapper == source (f84075cf), archive 82 members == 37e3a542, stage2 script 7df6344c, CLI shim 8e121b19. Full-tree modes/links are the approved caller's own measurement (explicitly not an entry condition per the 05:36Z correction).
4. NO-WRITER WINDOW — DEFINABLE AND CURRENTLY QUIET. Writer inventory enumerated (owner/root via fixed wrapper verbs; the service's own DB/state, not the package root; root/manual edits; package manager only via the wrapper). Baseline receipt 2026-11-01T135748Z, receipt digest 2fdbd1e8ab3a7d9367e5b54791eaf607606548c4c39824cc81be033dab0bba6d: active/running, n_restarts=0, rows 1bcac986, drop-dir digest 4055db47 (6 entries, no new staging). Receipt script: scratch-tb-l1/window-receipt.sh <label>. At run time the window needs the Lead's hold declaration + window-start/end receipts + v3's own boundary markers; the interval spans before the stop through after the Stage 2 restart.
Pinned inputs re-verified today, all match: procedure 652d69f5, bootstrap 12997259, caller 556ad026, core f211db39, manifest 6fa03597, archive 37e3a542, review de466966, /usr/bin/dd 095c38ac. v3 procedure tests 22/22 and caller/integration 12/12 recorded with pinning intact. Correction carried: v2 failed independent review and must not be run; v3 is the live approved artifact (embeds the unchanged v2 comparison core).

## B. OWNER-ACT REQUEST (the one thing that unlocks the run)

The approved v3 operation needs an owner-present, escalated, one-time execution: fixed root staging, a fixed read as the fleet principal, exact-unit stop/start, one exclusive window. Cayleb's approval STANDS (05:33Z, Sentinel_5666feb569cc8191b4eadaada1c75cda) — not re-asked. What is needed from you: name/arrange the owner-controlled operator seat and the window. On your word I will (i) declare the no-writer hold across all lanes for the interval, (ii) take the window-start receipt, (iii) hold the hold until the window-end receipt, and (iv) hand the preserved full private result to the fresh installed-output checker. Team B stays read-only throughout.

## C. POLICY RULING REQUEST — is fact 2 sufficient?

Team B does not choose this. Question: does the in-process manifest correlation (rows.manifest_sha256 1bcac986 = the 0.2.2 archive's package/rows.manifest.json, path derived from import.meta.url, no rowsManifest override) PLUS the approved caller's candidate-bound 82-file comparison CONSTITUTE sufficient loader-origin evidence for the identity proof — or do you require the separately reviewed in-process witness (a new owner decision + product change, not ready)? Team B's recommendation: put it to the fresh checker as an explicit ruling request with the evidence, since building a runtime witness is a new release/gate, not a Lane-1 patch.

## D. NEXT (no idle)
Fresh desktop-vantage reviewer against board/probes/team-b-identity-v3-review-packet.md (Team B writing it; I spawn the reviewer the moment it is on disk) -> reviewer verdict -> owner window (B above) -> run -> fresh installed-output checker on the unmodified Stage 2 check -> your Stage 2 ruling. In parallel Team B proceeds with the consolidated gate pack (corrected FINAL, INDEX correction, F11 Block I).

— delegator
