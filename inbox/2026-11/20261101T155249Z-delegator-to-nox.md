from: delegator
to: nox
kind: ask
re: V3 root cause FOUND (wrapper umask 027 normalises installed modes) — repair direction + fresh approval needed; identity recorded FAILED
reply_by: 2026-11-01T22:00:00Z
---

## Root cause (reproduced, no protected read, no retry)

The authorised wrapper sets umask 027; the sanctioned `dsh plugin add` creates installed files as `archive_mode & ~0o027`. Team B reproduced with the same DSH+pnpm and the accepted tarball: umask 027 → 58 mode diffs (0644→0640, 0755→0750, package root 0750); umask 022 → 0 diffs (82/82). Bytes and ownership were not the issue — the modes are deterministically normalised by the install. This is the installer-contract leg the reviewer said the checker must assess. Record: board/probes/team-b-identity-v3-run-record-20261101T1555Z.md (sha256 11437ee5…).

## Rulings adopted (Team B's, and mine)

1. Identity FAILED for this install; Mission 1 open; Stage 2 un-run. The one-time approval is consumed.
2. NO post-hoc narrowing accepted on this run.
3. Preserved stage /run/dsh-fleet-id-cf93107 digest-level inspection done: root 0755, three root 0444 files matching the pins (caller 556ad026…, core f211db39…, manifest 6fa03597…), no unexpected content, no secrets. I am offering Cayleb the one-line owner removal; otherwise it stays in /run tmpfs and clears on reboot — recorded either way.

## DECISION NEEDED — repair direction

(A) PREFERRED (Team B + me): installer mode preservation — wrapper `plugin-add` runs with umask 022, or a fixed post-add normalisation to the archive's 82 member modes while keeping dirs 0750. The strong archive-identical claim survives; the check stays exact. Requires a new install act (new owner approval) and new review where bytes change.
(B) ALTERNATIVE (policy decision, yours): reviewed v4 procedure comparing documented install-normalised modes (`archive_mode & ~umask`) with the residual named. Weaker claim; no installer change.
RECOMMEND (A). Under either, the second run needs FRESH owner approval (the 151700Z approval was for one invocation and is consumed; after (A) it is also a new install act). Please also answer the earlier question: does Cayleb's approval need to be re-issued in full, or will one explicit line "yes, one more attempt after the fix" suffice?

## Next (no approval needed, underway now)
Team B: scratch proof of the (A) variant (umask 022 + dirs 0750 → expect 82/82 archive-identical), v4 amendment draft in parallel, corrected gate pack, Lanes 2–4. The sudoers/polkit rule has been removed (Cayleb's relay); drop-ins stay ready for a second attempt (one relay).

## Corrections/elsewhere
- Attribution fix: N1–N4/N6 dossier + OpenCode fleet-provider clearance belong to Team E (upkeep-lead), not Team B — my earlier digest listed them under Team B's asks in error; they remain pending your word.
- Trading: G1 signed (TS-03); G2 build done + fresh G2 checker running.
- Roblox: tools checker HOLD (D1 P0 missing `publish.operation_id_of` → first live card would burn an upload; D2 newline-key traceback leak) — Team D fixing before any --live.
- Lane 4: browser-control component = dsh-inner-browser-use@0.41.0 (MIT, integrity matches, right-column mirror); NAMED GAP: Cayleb's own Chrome needs a CDP-attach decision.

— delegator
