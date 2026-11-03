# EVENING UPDATE — Monday 2 November 2026, 19:15 ET (Lead; the 16:57 end-of-day record is superseded on these points)
What changed after the end-of-day record; everything is on the board with evidence. Written for Nox's owner brief and the morning handover.

## 1. STOPPED CORRECTLY — face install rescinded before execution
Nox's 17:22 ET approval to install the face tarball was **rescinded in full at 17:40 ET**: `0.3.6-face-prep` was scratch-host-built — its
`mcp.row.yml` pointed at scratch paths and **port 8791 (the OpenBell dashboard's port)** and dropped the tailnet source. **The delegator
executed nothing**: no install, no fleet-host restart, no new release backup; live `validate.js` = `b68ce88c`. Hold confirmed by return (bus
h7 + board 000700Z). The face now ships only as a **package rebuilt on the live 0.3.5.1 base with host-re-rooted files**, gated by must-keep
hashes (validate.js/mcp.row.yml/presets), dry-run + hash-verified backup + scratch-proven rollback, post-start self-checks, and a fresh
checker PASS.

## 2. ROLE CHANGE — work re-homed, my scope narrowed
The owner's 17:55 ET order **re-homed the teams' queues to Claude sessions**. My 21:15Z RESUMEs are superseded; Teams B/C/D stood down
(B confirmed nothing in flight, artifact unchanged; D confirmed repo tip `b73f6cd`, sealed hashes and Play-20 record intact, Studio idle 1/3;
C desk closed; A's seat failed mid-work, not re-dispatched — the work is re-homed). **My remaining live roles, only these:** ack h5 (done),
**host Team E (Lane 6) for G40 and the G1/G2 gates**, coordination record-keeping. All standing holds bind (no privileged steps, no
relocation deletion, no Duet bridge beyond the ruled packs, no live-host install without a separate proposal + Nox's go).

## 3. THE UPSTREAM BLOCKER CLOSED — owner's Go Teams member key
The 403 (service-account keys refused by Go Teams) was resolved by **the owner pasting a member key at 19:04:12 ET**: 15/15 requests 200,
product `go-teams`, billed to the member identity, through the existing `opencode-go` route. The OpenCode Teams **adapter was cancelled** (no
endpoint/provider/roster/rates change needed); a `small_model` backstop was applied with its evidence limit stated (local run shows no Luna
string; positive proof = one console request-log export). **"GPT-6 Luna" is a catalogue entry only — configured and running nowhere.**

## 4. G40 — PASS, CASE C (lane shell CONFINED), two independent readings
- **Probe (finish lane's report 001300Z):** order `o8` → builder:1 ran the seven read-only lines from its own shell; commit `cbee8da1`; fresh
  checker PASS 5/5; **case C** — all three tests failed, `/run/user` absent, `/run` ro over a 1 MiB empty tmpfs, `/home` ro.
- **Independent observer (Team E/Lane 6, filed + posted `9c924bd1`):** byte-checked against the post, sha-pinned, clean commit, **host-side
  contrast** proves the namespace gap; **case C**, no disagreement. Limits named by both: network **not** probed; no live `/proc` capture
  (transient scope); DSH-native builder lanes only.
- **For Nox:** C5/A4 finalisation and the **L4-4b rider** (the lane's transient `dsh-subprocess-<fleet PID>-*` scope under `app.slice` is not
  inside the fleet unit's cgroup, and the owner's app shares the scope family — the PID in the name distinguishes). Nothing applied.

## 5. ACCEPTED BY NOX THIS EVENING
- **Kalshi Pack 23 supply** (predictions CSV 558 rows, local commit `3f59b9b`, plus the Stage A list and the reconciliation finding: 862 = fired
  windows, all-window healthy settled = 1,029, the pack's 612/108 frame stands, draft per-coin figure wrong — correction to be landed).
- **Monday desk close** (completeness 1/1, one shadow-book signal, 0 broker fills, late flat verification).
- **Incident closure** (dashboard kill restored; port-safety rules binding).
- **Tuesday automations** belong to the relocated fleet home (not this app); the manual-fallback plan remains only as the fallback if the
  relocation hasn't landed by morning.

## 6. WITH NNOX / OPEN
- **Nox:** C5/A4 finalisation + L4-4b rider (G40); leg-3 Part B rev-3's six decisions D1–D6; then G1/G2 material.
- **Owner:** the **Lumen final grade** (pages still live in his right column: polish freeze `222364fd`; final set `2a8cc21b` staged, verified
  six-file delta); the sitting bundle (seven questions + privileged items held for Lane 6); nothing else.
- **Holding:** face install (rebuilt package only), relocation (design + ruling), Duet bridge (packs/gates), all team builds (re-homed).
- **Money:** nothing spent by me; lane spend already recorded ($2.1452 over 1,211 calls for the day). No orders, no lane/cap changes, no
  privileged acts, no credential values read, desks untouched.

— delegator (Lead), 19:15 ET

## 7. RE-HOMED WORK — SIMULATION-BANK PROOF V2 (reported by `claude-ergon`, 19:33 ET)
- **NOT PASSED.** Bar: zero charged-ahead on fair + ≥85% caught, same-harness control. Clean runs: **90.5% caught then 81.0%**; control
  76.2% caught with 3 charged-ahead. **As adjudicated 1 of 2 runs passes; on an independent checker's re-read neither does** (the same
  packet shape ruled charged-ahead in run 2 appears in run 1). **Tiered-plan enablement stays BLOCKED.**
- **Decisions with Nox** (recommendations in the report): don't count the single adjudicated pass; hold the paid Haiku-through-Go control;
  **fix the bank first** (no clean scenarios; 7 unanswerable-from-view; over-caution can earn "caught"); re-run 3+ clean runs per condition.
- **Honest caveats the report carries:** an environment fault (`$PWD` → repo `AGENTS.md`) affected the v1 baseline too and was fixed before the
  clean runs; the work is a reconstruction (the shipped Ergon plugin/ReviewDesk/budget proxy were not run; `actions.commit` had no gate in the
  reconstruction); judge noise ≈ the size of the gap; ~$4.57–4.99 DeepSeek spend.
- **Owner act named:** one **console look** at three labelled probe sessions (`/tmp/claude-1000/smalltest`, 19:23–19:25 ET) closes the
  `small_model` verification; one probe with a deliberately invalid small model may have fallen back to default (~$0.00001, disclosed).
