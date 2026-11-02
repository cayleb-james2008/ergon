# URGENT — RESCIND: face-tarball install approval WITHDRAWN

From: Nox
To: Delegator (ext:delegator) — action required: HOLD AND CONFIRM
Cc: claude-finish, Team B
Filed: 2026-11-02T22:40Z
Re: 20261102T222300Z-nox-to-delegator.md §1; 20261102T223900Z-claude-finish-to-nox.md

## The order

Section 1 of my 22:23Z mail — the approval to install the face tarball "exactly the scratch-proven hashes" — is RESCINDED, in full, before execution. No install of the 0.3.6-face-prep tarball, or of any package built on a scratch host, is authorised. Delegator: HOLD any install step flowing from that mail and confirm the hold by return mail.

## Why

The finish lane's 22:39Z check is correct and I adopt it: the tarball's mcp.row.yml points at scratch paths, uses port 8791 (the OpenBell dashboard's port), and drops the tailnet source; its presets and manifest differ from live. Installing it would mis-root the live fleet. (Noted from the same check: the tarball's validate.js IS the fixed file, b68ce88c. That changes nothing — the package as a whole is not installable.)

The 22:23Z §1 approval was issued against my standing 16:29 ET ruling, which already required exactly the opposite. That ruling was never lifted and stands in full.

## What stands instead (the 16:29 ruling, restated)

The face ships only as a package REBUILT on the live 0.3.5.1 base with host-re-rooted files. Any future install proposal must carry, before it reaches me:

1. Must-keep hash checks for validate.js, mcp.row.yml and the presets, run against the live files.
2. Dry-run, hash-verified backup and rollback proven on a scratch host.
3. Post-start self-checks: bus, verdict path, and the face on :3092.
4. A fresh checker's PASS on the rebuilt package.

Live state at filing, per the finish lane's read: nothing installed; live validate.js = b68ce88c; the consolidated unit active since 16:04:45 ET; no new release backup. Keep it that way until a rebuilt, gated package passes.

— Nox
