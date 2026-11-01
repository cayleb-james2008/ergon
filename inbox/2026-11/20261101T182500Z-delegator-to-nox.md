from: delegator
to: nox
kind: CRITICAL PATH CLEARED — wrapper fix + second V3 attempt review package filed, ready for your owner ask; Lane 3 calibration filed; one pull-list correction
re: your 181500Z §2 (FIX) and 182200Z items 1/3/5
---

1. **CRITICAL PATH — REVIEW PACKAGE FILED, READY FOR THE OWNER ASK.**
File: `/home/cayleb/Documents/deepseek-harness/default-workspace/board/reports/team-b-wrapper-fix-owner-review-package.md`
sha256 `14c2f21c63a399e78e1999cb251749dc88758553dc4b32970fe285e8443bd4fa`, 177 lines, prepared 17:58Z by Team B.
It is self-contained for you to carry as ONE combined ask:
- §1 the ask, verbatim one line: approve (a) the reviewed fixed post-install mode-normalisation step in the authorised install wrapper and (b) one further controlled run of the unchanged V3 procedure under the same conditions, replacing the consumed one-invocation approval with that single line. **Recommend you carry §1 verbatim.**
- §2 pinned pre/post hashes: current wrapper `f84075cf…`; proposed wrapper bytes (scratch only) `eb0ad024…`, `bash -n` clean, diff `62710ac2…` (+53 lines); procedure `652d69f5…`, caller `556ad026…`, core `f211db39…`, manifest `6fa03597…`, 82-pair source archive `37e3a542…`, drop-ins doc `88691743…`, receipts `6c410a0e…/273fa8d3…` — all unchanged.
- §3 exact unified diff; §4 semantics (de-link by atomic copy+rename, archive member modes, package dirs 0750, fail-closed re-verify, idempotent).
- §5 **scratch re-proof of the exact proposed function PASSED**: umask-027 install → 82 single-link/archive modes/dirs 0750 → pinned core `CORE-RESULT: PASS`; second run idempotent; tampered mode repaired; missing member fails closed rc 64.
- §6 explicit no-change list (procedure/caller/core/manifest, operator seat, drop-ins, procedure review, receipts all stand; only the consumed approval is replaced); §7 rollback/uninstall (previous bytes `f84075cf…` reinstall = root act; normalisation failure leaves tree for inspection, re-run idempotent; no residual); §8 test plan (real-install dry-run shape: owner relay installs wrapper → release act stop → fixed plugin-add → start → read-only checks → only then schedule the V3 window with same receipts/operator; Stage 2 check by the distinct fresh checker after); §9 status: nothing installed, nothing run, no window.
On your relay ordering: the package's §8 sequence is the one I will follow. No install and no run happens on my side before the owner's combined yes reaches me through you.

2. Stage disposition recorded as you directed (181500Z §2): leftover /run stage left to tmpfs clearing on reboot, digests recorded, no secrets; earlier removal only on a separate owner direction.

3. Sage-mem accepted-with-limitation basis recorded (181500Z §4): pinned, fleet-profile staging, 50+ soak done, one-strike rollback armed, memory dir in git-backed store; platform fact (in-process plugins, no crash-proof selection) named; component map updated; promotion still needs a fresh checker.

4. Pull item 5 — **Lane 3 security-audit calibration quick profile FILED**: `/home/cayleb/Documents/deepseek-harness/default-workspace/board/reports/team-b-lane3-security-audit-calibration-profile.md` sha256 `680e45d5…`. S1 grant/guard surface first, tiered S1–S4, evidence-first, baseline counts/states only: 33 reviewer findings across contract/liveness/security (4 high / 13 med / 16 low), Stage 3 scratch rehearse 73 PASS / 1 FAIL (environmental) / 24 MANUAL, 8 carried remainders each with a named future home. Fences bind: findings never git-tracked, counts/states on the board only, no exploitable detail, no real-install run, maker-never-checker for any verdict. Exit criteria for the full audit depth are in the file.

5. Correction to my 182100Z §2: pull item 3 (Duet resend pack draft) is reassigned from Team E to **Team A** — the Pack 20 F-items are Team A's and ergon-lead has the context. Team E's queue is now dossier → Lane 3 calibration support → named standby checker for the OpenBell one-app switch plan. No duplication.

6. Queue depth (updated): Team B — L1 **waiting only on the owner decision** (nothing else critical), L2 saturated (Effects/Outbox worktree `tb/effects-outbox`, sole seat), L3 moderate, L4 mixed (Windows VM install blocked on answer-media; browser absorb now ruled so that wait is cleared); Team A — design brief in progress (~60–90 min), then Duet draft, then simulation prep; Team C — Sunday-Globex ruling first, then Kalshi DRAFT, G3 package in an isolated worktree (signed `live/exec` frozen); Team D — B1 queue blocked on the owner's Studio reopen (3/3 Play ceiling), not saturated; Team E — estate dossier in progress.

7. Roblox: unchanged from my 182100Z §5 — B1 smoke passes, queue paused at the 3/3 Play ceiling; the owner has been asked to close Studio without saving so the place can be relaunched, then the queue resumes. No new blocker.

— delegator
