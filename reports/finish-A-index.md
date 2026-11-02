# FINISH-THE-PLUGIN — Phase A: designs + gap reports (Team B)

**Filed:** 2026-11-02T09:20 ET (14:20Z) · **Author:** fleet-lead (Team B) · **Status:** design-only. Nothing built, nothing installed into the live
profile, no live session touched, no image read, no credential value read.
**Mission input:** owner order 07:34 ET, Nox 123400Z (`board/reports/FINISH-MISSION-PLAN.md`), Nox 112300Z (Owner Elevation), Nox 113000Z (handoffs through the MCP, wake-on-message, idle watchdog).
**Live baseline verified at 09:09 ET on the consolidated setup:** `@cayleb/dsh-fleet@0.3.3-consolidated` installed; control status `integrity` all
ok, rows `{ok, pinned 8, disabled 12, manifest 4cc45e71…}`, `lanes_allowed true`, host pid 1233729; unit `fleet-dsh` inactive; facade on
127.0.0.1/::1/tailnet :8790, GUI :3092; MCP `GET /mcp` → 405 (no server→client stream); `providers-get`: `opencode-go` credential present,
`anthropic` absent, `claude-code`/`codex` engines not installed.

## Phase-A artifacts

| # | Scope | Artifact |
|---|---|---|
| 2 | Comms layer (registry, wake-on-message, idle watchdog, per-participant tokens, presence) | `finish-A-1-comms-design.md` |
| 3 | Duet route + hookup (bridge design, one recommendation, Lane 6 brief, gap report) | `finish-A-2-duet-route-gap.md` (adopts `~/workspace/harness-build/finish-docs/leg3-duet-bridge-design.md`, with the mission-delta gap report added) |
| 4 | Owner Elevation (one-tap sudo + brief takeover; spec + threat model + consolidated-host gap) | `finish-A-3-owner-elevation-gap.md` (adopts `…/finish-docs/leg4-owner-elevation-design.md`, with the live Tier-B gap verified and added) |
| 1 | Visible face integration plan (sidebar, panels, activity view; client contract + data path) | `finish-A-4-visible-face-integration.md` |

## State correction the plan needs (verified, matters for batch 2)

1. **o1 is `blocked`, not in progress** (control `order-list`/`board-get` at 09:09 ET). Two independent facts: `orders o1` state `blocked`,
   `resume_state in_progress`, budget $0.5 with $0.02 unallocated; `claims` id 1 ended `lease-lost`; the lead session is `idle`, every
   builder is `stopped`, all tasks `open`. The block reason on the board: *"Dead work tree + commit gate: t1–t4 name absolute write targets
   under lanes/lead-e3fe3bb7/, which does not exist and cannot be created (parent lanes/ read-only for builders); builder lanes are not git
   repositories so the harness blocks every delivered build with no-commit; t1 gates t2–t4. Needs (a) one writable, git-initialised work
   tree visible to every builder with t2–t4 re-staged, or (b) per-lane artefacts acceptable + envelope room to replace t1 once."*
   → o1's face slice cannot land through its current task plan. **Named block B0** (needs Lead/Nox ruling: (a) or (b)).
2. **`o1` decomposition maps exactly onto this mission's item 1** — the face is o1's own spec, so the face build should replace o1's tasks,
   not compete with them. Recommended: re-stage o1 with the face package tasks (see D §7) once (a)/(b) is ruled.
3. The wake gap is already visible in live data: `roster-get` shows ten lane directories but all lanes `stopped`; the same id `builder:1`
   appears several times with different sessions, which is a registry/liveness defect, not a UI detail (A §2, G-A3).

## Top named gaps / blocks across all four legs (detail in each artifact)

| ID | Blocking | Kind | Owner / what closes it |
|---|---|---|---|
| B0 | o1 blocked on the dead-work-tree + commit gate (`no-commit` on lanes) | live structural | Lead/Nox ruling (a) or (b) |
| B1 | **No comms layer in the installed plugin at all** — 0.3.3 has no `src/bus`; `package.json` still exports `./git-bridge` to a missing file; the facade is POST-only with one shared bearer token; no registry, presence, watchdog, handoff ops | build | Team B batch 2 (A) |
| B2 | **Nothing can push to an external participant.** "Wake" for Duet cannot be a push by any documented route; the honest proof is bounded-latency delivery + pickup + ack | platform | Nox acceptance wording (B) |
| B3 | **No Claude credential / no Claude engine in the consolidated setup** (`providers-get`: anthropic absent, claude-code/codex not installed) → the "seat on the plugin's Claude engine provider" cannot start today | credential/engine act | owner or Nox route decision (B, C) |
| B4 | **Tier-B confirm is dead in the consolidated setup** — `/usr/local/bin/dsh-fleet-confirm` hardcodes `/var/lib/fleet/state/confirms`; the consolidated policy reads `<desktop-home>/fleet/state/confirms` (exists, empty) → a confirm never lands. This is today's cutover-stop class | packaging gap | Owner Elevation spec (C) must include the interim fix; Lane 6 gate |
| B5 | **Face has no supported data channel today**: client bundle is a hand-built Stage-1 skeleton with no `@Remote` surface; the supported live-data path is Typert-generated `ctx.remote.<ns>` (client plugin contract), which the fleet package does not generate | build + build-pipeline | Team B (D) + DSH-side build decision |
| B6 | **Client bundle changes need an app reload** to appear; the owner's app holds live sessions — reload is an owner-coordinated action, never forced | care fence | Nox/Lead window (D) |
| B7 | Token contract: Team A's pack is mid-polish (`ea68e398…` digest at issue; re-freeze pending) → the builder must pin the digest it built against; no hand-typed hex | dependency | Team A re-freeze (D) |

## Build order proposed (after Nox rules on B2/B3/B4 + Lead rules B0)

1. **A comms core** (registry + tokens + handoff/wake + ack + watchdog + presence) on a branch cut from the live 0.3.3 source tree, with
   the unshipped bus (0.2.0 modules in `~/workspace/harness-build/dsh-fleet-bus/package/src/bus`) as the base. Fresh checker runs the
   wake-and-ack proof; then the version/manifest bump and install path is the same variant pipeline proven yesterday.
2. **D face skeleton + data channel** in parallel (no owner reload until the whole sidebar slice is ready; then one coordinated reload).
3. **C Owner Elevation** spec → Lane 6 gate → Claude lane builds the privileged helper; one real elevation with the owner.
4. **B Duet bridge** courier + seat once B3 is decided; Lane 6 threat review before any external hookup.
5. **o1** re-staged onto the face build; final mission done-when audit with a fresh checker.
