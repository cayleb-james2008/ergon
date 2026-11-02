# FINISH THE PLUGIN — mission plan (owner order 2 Nov 07:34 ET, via Nox 123400Z)
**Owner's words in substance:** stop reporting "almost done" — there is still a lot on the table; keep delegating and actually finish it fully.
**Done is the full scope below, demonstrated. A landed piece is progress inside the mission, never "done".**

## State at kickoff (09:05 ET, verified on the consolidated host)
- Consolidated setup: fleet host healthy (integrity ok, rows 8/12 manifest 4cc45e71…, lanes_allowed true); unit retired; one writer.
- A5b proven (real lane, real turn, priced ledger ≈$0.0013). Owner pick executed: lead seat → opencode-go/deepseek-v4.1-flash
  (Tier B ticket 100001 consumed; source=table; actor cayleb-local). Anthropic key entry off the owner's list.
- **o1 in_progress** (claimed by the lead lane): 5 tasks decomposed — t1 "Fleet-state layer…" claimed by builder:1 (live, spending),
  t2 plugin manifest + main[fleet] + sidebar.panellist, t3 VM panels, t4 token/contract/fence verification + docs, t5 run the done-when.
- Named blocker (filed to Nox): checker seat still anthropic/claude-haiku-5-5, no credential in the setup → maker-never-checker verdicts
  need a route decision (Nox/owner pick; not asked of the owner by us).
- Care fence + single writer + no image reads bind all work. No deadline talk.

## Scope → owner of workstream → proof (done-when)
| # | Scope | Owner | Proof required |
|---|---|---|---|
| 1 | **Visible face** in the one app: sidebar fleet section (lanes/meters/orders, live state), phone-VM panel (AVD round-trip), Windows-VM panel (monitor-level), multiple contained Studio instances (own login/connection/game/agent team, never the owner's personal Studio), agent-activity view | Team B leads build; Team A supplies design-pack tokens; Team D supplies Studio containment recipe; Claude lanes for harness-sensitive client pieces | renders live in the owner's app from live state (no fixture data shown as real); owner grades rendered visuals |
| 2 | **Comms layer**: participant registry, wake-on-message, idle watchdog scoped to active work, per-participant tokens, presence/liveness | Team B design+gap first, then build | a handoff to a parked participant wakes them and returns an ack through the bus |
| 3 | **Duet route + hookup**: cloud-reachable authenticated bridge (Duet is cloud-side; outbound-only from desktop), standing seat inside the plugin, Lane 6 threat review | Team B design+build; Lane 6 review via Nox | handoff through the bus wakes Duet and returns his ack; no email work queue riding the proof |
| 4 | **Owner Elevation**: one-tap sudo approval + brief takeover, per-command requests, owner-only entry, one-shot, audit-logged, privileged helper | Team B design; Claude lane builds the privileged helper; Lane 6 hard gate | Lane 6 gate passed + one real elevation request completes end-to-end |
| 5 | **First real order o1 completed** through the bus by real roster lanes | Lead lane (driving) + roster; Lead monitors and files evidence | o1 done with its evidence on the board; sustained work released |

## Operating rules (binding, from Nox)
1. One mission, large batches, self-claiming successor items; no lane waits silently on Nox.
2. Maker never checker; fresh checker who did not build it for every proof.
3. State on disk; report to the board per completed scope item (completions and genuine named blockers only).
4. Care fence: collaboration in the owner's shared setup, never force-closing/hijacking his live sessions/apps; single writer.
5. The consolidated install stays healthy throughout; desktop profile untouched.
6. No deadline talk to the owner.

## Batch order
1. (running) o1 via the lead lane; Team B Phase-A designs (comms + Duet bridge + Owner Elevation + face integration plan).
2. Face build (Team B + Team A tokens + Team D containment) — checker-gated per component.
3. Comms build → wake proof. 4. Duet bridge build → wake-and-ack proof (Lane 6 gate). 5. Owner Elevation build → Lane 6 gate +
one real elevation. 6. o1 evidence rolled up; mission done-when audit with a fresh checker.

## o1 CLOSURE (Nox ruling Option C, 2 Nov 16:43 ET) — read the record, not the stored state
o1 is **cancelled administratively** (owner op, reason cites the closure record). The harness has no `cancelled` state: o1 and t5 are
stored as **`failed`** — read that as administrative closure, NOT as a failed deliverable. Evidence: a **fresh Claude check met all four
rubric criteria on the commits** (43/43 tests, fences clean, export surface client [apply, inject] / root [apply]); the **route gap** is a
harness gap (rubric final task + checkflow skipping final tasks + tier-2 verdict refusing tier 1), with **Option B** (roster starts a
fresh Checker for rubric final tasks; verdict route accepts tier-1 final through it) scheduled as the next package under a liveness gate.
Record: `docs/harness-build/finish-2026-11-02/O1-CLOSURE-RECORD.md` (also `O1-CONSOLIDATED-REPORT.md`: stall map, spend $2.1452 over
1,211 calls, o1-attributed $1.1461). Corrections carried: post-163 "one chain" provenance is wrong; 22/84 checker mutations survive
(panel gating/SSE refusal rest on reading, not a guarding test); nothing type-checked, bundled or rendered (that is leg 1 of this
mission). **The rendered face + integration remain the mission's leg-1 deliverable; the harness route fix is Option B's package.**
