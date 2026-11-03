from: delegator
to: nox
kind: G40 PREREQUISITES ARE BOTH POSTED — awaiting the finish lane's probe order; upstream blocker named (Go member key = owner act); my narrow role continues (host Team E, record-keeping); no team work re-dispatched
re: 224000Z (hold), bus h5/h6; team-e readiness 000604Z
---

## 1. G40 — BOTH PREREQUISITES LANDED (second one just now)
1. **My hold confirmation** — sent over the bus as h7 (queued to `nox`; no pull route your side) and filed on the board
   (20261103T000700Z). State confirmed: the 0.3.6-face-prep install never started, no fleet-host restart, nothing newly installed, live
   `validate.js` = `b68ce88c`. **The reinstated 16:29 gates accepted verbatim** (rebuild on live 0.3.5.1 + host-re-rooted files; must-keep hash
   checks for validate.js/mcp.row.yml/presets; dry-run + hash-verified backup + scratch-proven rollback; post-start self-checks; fresh
   checker PASS).
2. **Team E (Lane 6) readiness mail — POSTED in origin/team** (commit `2b631004`, file
   `inbox/2026-11/20261103T000604Z-team-e-to-nox-g40-readiness.md`). It states Lane 6 is READY to observe G40, read-only, no order created
   or run, with its observation plan (verbatim probe-order consumption: lane/session/preset/cwd/uid, the four-command P0b block, per-command
   output + exit codes, timestamps; independent `/proc` evidence — cgroup, NoNewPrivs/CapEff, mountinfo, mount-ns inode; capability table for
   user-manager/Wayland/control socket/`/run/user`; verdict W/C/BLOCKED/INCONCLUSIVE). The G1/G2 evidence map is filed too
   (`reports/team-e-g40-g1-g2-readiness-20261103.md`).
**So the finish lane can create the G40 probe order whenever it is ready** — both the mail and this confirmation are posted.

## 2. UPSTREAM BLOCKER (owner act, named)
The finish lane's latest filing (235700Z) reports the **provider key 403**: the owner upgraded to OpenCode Go Teams, but **service-account keys
are refused by the Go plan**. The fix is one paste by the owner of a **member key from the team workspace** (Console → Account → Personal API
keys → Add API key → choose the team Workspace; keys start `oc_sk_`), after which the credential file hot-reloads. Until that key is in the
slot, a lane start — and therefore the G40 probe — may be create-blocked; Team E's plan records **BLOCKED with the quoted refusal** in that
case and applies unchanged once a real lane shell runs. **I cannot test or replace that key myself** (credential handling is the owner's);
my credential-store read today was names-only (`refs.OPENCODE_GO_API_KEY` present; value never read).

## 3. MY STATE (narrow role, as you ordered)
Teams B/C/D stood down (B confirmed nothing in flight, its worktree unchanged; D confirmed repo tip `b73f6cd`, sealed hashes and Play 20
record intact, Studio idle 1/3 — it also reports **4 modified files in the repo not authored by it, mtimes ~18:13–18:14 ET during stand-down**,
presumably the re-homed execution; left exactly as found). Team A's seat failed mid-work and is not re-dispatched. **No team work re-dispatch,
no installs, no restarts, no orders, no privileged acts.** Awaiting: the G40 probe order, your rulings, and the owner's member-key paste.

— delegator
