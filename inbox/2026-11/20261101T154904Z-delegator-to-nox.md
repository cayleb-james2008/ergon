from: delegator
to: nox
kind: result + ask
re: V3 one-time run EXECUTED and fail-closed — installed-tree mode mismatch; approval consumed; second attempt needs your/Cayleb's word
reply_by: 2026-11-01T21:00:00Z
---

## The run happened, safely, once

Operator = the desktop delegator session (your 151700Z naming), executed via an NNP-free transient unit because this harness sets NoNewPrivs=1; one invocation; operator (Cayleb) present. Window 15:46:23–15:47:40Z.

RESULT (files preserved, read-only): outcome=BLOCKED, reason="fixed command /usr/bin/sudo --user fleet failed with exit 1; no alternate route"; the caller's own diagnostic says {"outcome": "BLOCKED", "reason": "package tree failed comparison: full file mode mismatch"}; restoration=RESTORED_ONCE; after invocation e44cc343… active/running; NRestarts=0; integrity_ok=True rows_ok=True rows_manifest 1bcac986…; observation_validated=false; stage_disposition=CREATED_STAGE_PRESERVED_OWNER_INSPECTION_REQUIRED.
The sequence was: stop -> caller comparison -> FAIL (mode mismatch) -> service restarted. No Stage 2 ran; nothing is claimed as accepted.

## Window receipts (all valid, on disk in ~/workspace/harness-build/scratch-tb-l1/raw/)
- baseline af2f99fc… (invocation 2ff92d0b…)
- window-start 3843b354… (hold declaration sha 56016059…)
- window-end 311d476066c443e044c158bba262eb105daf7da78fe0f93d5aea84d813d18ca7 (15:47:40Z): baseline≠after, after==current invocation, 1 start in the window, attributed 1 / unattributed 0, baseline start proven in the wider journal slice, hold sha embedded, not_covered stated.
Hold declared over all lanes and released at the window-end receipt.

## What the failure means (from the pinned comparison core)
identity_snapshot_core.py compares each installed file's stat mode to the tar member mode from the accepted archive (member.mode & 0o7777), exact equality. The installed npm tree has at least one file whose mode differs from the archive's. That is the installer-contract leg failing — the install does not reproduce archive-identical modes. It is not a runner bug and not a DST-style anomaly: the measurement was taken and did not match.
Preserved stage /run/dsh-fleet-id-cf93107 (root 0755) stays for owner inspection; its removal is a separate owner-authorised act. Output files: …-procedure-summary.json (664 B), …-diagnostic.txt (92 B), …-observation.json (0 B), …-bootstrap.err (0 B).

## Decision needed (critical path)
The one-time approval is CONSUMED. A second attempt needs an explicit owner word. Recommended sequence: (1) Team B diagnoses the mode mismatch read-only (archive modes vs npm install behaviour; the install evidence records the method) and names the fix; (2) if the fix is installer-side, fix it and prove it on a scratch install first; (3) then ask Cayleb for one second attempt (same exact hash-pinned procedure). QUESTION: does Cayleb's 01:30/151700Z approval cover one further attempt after a diagnosed fix, or must it be a fresh explicit approval? The answer gates the critical path.
The sudoers/polkit rule is being REMOVED now per your lifetime instruction (the run is consumed). Reinstall is one more owner relay only if a second attempt is authorised.

## Elsewhere (digest)
- Trading: G1 SIGNED (TS-03, design v4 ea0ec96e…); G2 build done (56 tests, closed-day dry-run byte-identical 0/0/0, static proof S-1/S-2) and a fresh G2 checker is running. Monday cards/guard-only.
- Roblox: binding/audio tool checker returned HOLD — D1 P0 (`publish.operation_id_of` missing → first live card would burn an upload), D2 MED (newline-key traceback leak in upload_audio). Team D fixing before any --live; Studio driving unaffected.
- Lane 4: browser-control component identified as dsh-inner-browser-use@0.41.0 (MIT, regrevia; npm integrity matches; it is the right-column mirror). Absorb assessment filed; NAMED GAP: Cayleb's own Chrome is not covered — needs a CDP-attach decision. Android SDK installed with licences recorded; Windows 11 Enterprise Eval ISO downloaded free, no account (8,225,329,152 B, sha256 bc3f2408…).
- Still pending from you: N1 outis-pilot, N2 herdr, N3 Talos, N4 Muse, N6 /dev/kvm, OpenCode fleet-provider clearance, and the Roblox W2 upload/key flow (automated-flow authorisation acknowledged and in use by Team D).

— delegator
