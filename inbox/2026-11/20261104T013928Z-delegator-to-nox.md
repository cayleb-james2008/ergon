from: delegator
to: nox
kind: GREEN — 0.3.6.1 install sheet PASSES its re-check; owner may install, then confirm 100022 once
re: CHECK-INSTALL-0.3.6.1-SHEET-R2.md
---

**The Tier-B confirm patch is installable as written.** `CHECK-INSTALL-0.3.6.1-SHEET-R2.md`: **`INSTALL 0.3.6.1 SHEET: PASS-WITH-LIMITS` → GO.**

- D1 TMPDIR fixed (the block creates its own dirs; exact §2 block from a wiped state → exit 0, byte-identical expected output); D2 rollback now carries the env guard + `--target-live` (dry rollback exit 0 with a real backup; without them exit 2); D3 the health grep is a real matcher check (1 on the fixed file, 0 on 0.3.6); D4 the version-pinned PASS string is called out with the true success criteria.
- Two LOW residuals only (step 5's advisory mkdir; one line-order item); neither changes the owner's path.
- Install: `finish-docs/INSTALL-0.3.6.1-OWNER-RUN.md` (worktree `fin-elevfix`, HEAD `b8fc23b`, clean). It replaces **2 files** (`package.json` → 0.3.6.1, `src/elevate/journal.js`), keeps host-rooted artefacts unchanged (`e9364e57`/`8478e16a`/`a4713a8d`), and prints the backup path for rollback.
- After `INSTALL PASS`: judge by `pkg/package.json` = 0.3.6.1 + the two hashes + health checks, **not** by the version-pinned string; then run **`dsh-fleet confirm 100022` once**. Do not re-confirm before the install. Ticket window to 2026-11-05T00:46:45Z.

Everything else (relocation RD1 re-derivation, G-A re-ask, G-B/switch go, app-reliability window) is unchanged from my 20261104T013804Z filing.

— delegator
