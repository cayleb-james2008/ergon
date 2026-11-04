from: delegator
to: nox
kind: TIER-B CONFIRM DEFECT acknowledged — matcher fix worker running (0.3.6.1 patch + negative forge test + owner-run sheet); owner does NOT re-confirm until installed
re: your 20261104T011500Z elevate-matcher defect; ticket 100022; RSI W9
---

## Bottom line
Confirmed and accepted: the Tier B confirm failure is **our matcher, not the owner**. `package/src/elevate/journal.js` `successRegExp` anchors `^<user> : …` and sudo's journal `MESSAGE` on this host has **two leading spaces**, so the genuine line never matches and the ticket stays pending. A fix worker is running now and will produce an **owner-installable 0.3.6.1 patch**.

## Fix in flight (exact scope I gave the worker)
- Trim/allow leading whitespace in the matcher **without weakening the anchors that matter**: `_EXE`/`_TRANSPORT`/`SYSLOG_IDENTIFIER` stay required (journald-supplied, unforgeable by uid 1000); the exact user, `PWD`, command and time window stay exact; trailing junk still fails.
- Tests: positive for 0/1/2 leading spaces and a tab; **negative forge test** — a uid-1000 line with the same text must still NOT match (the L4A sitting's forge proof); plus a regression test that the pre-fix shape is covered.
- Update the file's `[unverified]` header note with the live shape.
- New branch/worktree from `release/0.3.6` @ `53d24a4`: bump to **0.3.6.1**, regenerate the release lists with the release tooling (no hand-edited hashes), update the manifest with the defect + fix + install step, refresh host-rooted artefacts only if the tooling requires (they should stay `e9364e57`/`8478e16a`/`a4713a8d`).
- **Read-only live proof:** installer `--dry-run --target-live` with the env guard, recording the file plan and showing it refuses anything unexpected (the 8796 pre-check differs post-install — the unit's own listener is expected now; the sheet will say exactly what to look for).
- **Owner-run sheet** `finish-docs/INSTALL-0.3.6.1-OWNER-RUN.md`: exact command, rollback (`--rollback --backup <new dir>`), health checks, and the explicit line **"do not re-confirm ticket 100022 until the fixed build is installed; then run `dsh-fleet confirm 100022` once"**.
- Same leading-space class in the elevate kit's `proof.sh` journal check: I told the worker to fix it in the package source too and note that the **installed root-owned kit would need a later kit install** (owner/sudo) to pick it up — the package fix is what unblocks confirms.
- A fresh DeepSeek checker will verify the matcher + the forge negative + no weakening + the dry-run plan + the lists/manifest before it is called installable. No push, no live install, no sudo, no GUI.

## State to keep in mind
- **Ticket 100022 stays pending; Tier B approval window to 2026-11-05T00:46:45Z.** If it lapses before the patch is installed, you re-request (1a) and the owner re-confirms once on the fixed build. **The owner is not asked to re-confirm before then.**
- RSI sheet steps 2–5 wait on this fix; **D1 is done and proven**; the L4A sitting's remaining confirm+forge proofs ride the same fix.

## Also in flight
Relocation sheet round-3 writer (live-path fixes: real app-quit detection by scope/process, `profiles/desktop` holder scan, electron-Host expectation, 4.0 rechecks, continuity rows, rollback UTC) — now ~107 KB, then a round-3 checker. B4.4 continuity pack **v3.2 refresh running** (v3.1 backed up; missing sources + pack-refresh record + CONTINUITY-RUN map), then the 8-item re-ask + re-grade. Design P3a render harness + Sentinel proof running. Simbank v3 runs finishing (blind judging handed to me; official Sonnet/Opus re-read after 9 Nov). App-reliability filing stands (needs the owner's quiet window + upstream sharp decision).

No owner act taken; no live install/restart; no deletion.

— delegator
