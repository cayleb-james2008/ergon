from: delegator
to: nox
kind: ask
re: browser-control reconciliation FILED — two rulings needed (pick for the mirror; meaning of "own Chrome"); sage-mem staging shape proposed; rig milestones
---

1. BROWSER CONTROL — COMPARISON FILED: board/reports/team-b-browser-control-reconciliation.md (sha256 fb492a8a…). Head-to-head of the installed `dsh-inner-browser-use` 0.41.0 (regrevia, MIT) vs the audit pick `dsh-builtin-browser` 0.5.2 (both integrity-verified, candidate tarball sha1/sha512 match npm).
   - Leg A: inner does the side-panel mirror WITHOUT host mutation (client injects into sidebar-right); the candidate's side panel needs its desktop bridge which PATCHES the app's main.js (append guarded import, backup/revert) — the audit's crash class; Windows-verified only; our host is a dev build. Recommendation: do not put the main.js bridge on this host; inner-browser-use is the lower-risk absorb for the mirror.
   - Leg B (DECISIVE): NEITHER controls Cayleb's RUNNING Chrome. Both launch his installed Chrome/Edge binary with a plugin-owned `--user-data-dir` (candidate system-browser.js:745-746; inner CHROME_BINARIES). RULING NEEDED: does "control of Cayleb's own Chrome" mean (i) his live session/profile — then it is a NAMED GAP needing a separate CDP-attach decision, or (ii) his installed browser as carrier — then both candidates cover it.
   - RULING NEEDED (2): the pick for the mirror. My recommendation follows Team B's: inner-browser-use, no main.js patch. NEITHER is absorbed until you rule.
2. SAGE-MEM STAGING SHAPE (proposed for your confirmation, since Team B is waiting before starting admission): a dedicated STAGING DSH profile under Team B scratch (e.g. scratch-tb-l3/staging-profiles/sage-mem-soak) — NOT the owner's personal profile and NOT the installed fleet profile (Rule W single-writer, Lane 1 only). The 50+ cycle soak + fresh checker + one-strike rollback run there; only a PASS promotes into the single package's component map. Confirm or correct.
3. RIG MILESTONES (Lane 4): Android AVD `rig-phone` created inside the rig and HEADLESS BOOT VERIFIED (27.9 s, adb device); Windows node ready (80 GiB disk, OVMF secboot vars, virtio-win ISO, swtpm TPM 2.0 working, well-formed autounattend, launcher) with LAUNCH SMOKE PASS (VNC loopback 5902 listening, monitor responsive) after the vvfat fix. Status: board/reports/team-b-lane4-rig-status.md. Next: unattended Windows install observation + Android tap/screenshot round-trip + panel skeleton.
4. LANE 2: isolated worktree `tb/effects-outbox` @ cf93107 created (integration untouched) + ordered build queue (Effects/Outbox → bus package assembly → hands race fix → contract amendment → composed-path e2e). LANE 3: real-install Stage 3 prerequisites filed (waits on identity repair/approval).
5. WRAPPER REVIEW PACKAGE READY (board/reports/team-b-wrapper-mode-normalisation-review-package.md) — sketch, tests, risk register, post-FIX sequence; nothing implemented; ready for your FIX ruling.
6. STILL PENDING FROM YOU: V3 repair decision (FIX vs AMEND) + fresh approval for the second run; the two browser rulings above; N1–N4/N6 dossier (Team E); OpenCode clearance. Crash diagnosis corrected to the image-read path (163740Z + 164047Z) — memory plugin cleared.

— delegator
