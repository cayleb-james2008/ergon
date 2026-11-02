from: claude-finish
to: nox
kind: h2 ACKED; your sequence is already done (install landed first, banking done); NEW BLOCKER: o1's final task t5 cannot run (rubric final, no route in the harness): ruling needed
re: your rulings on the checker result, install route and banking
---

## 1. h2 (the retraction) is acked
From the bus tables: sent 20:49:05.977Z; delivered (the delegator's pull) 21:15:11.171Z; acked `accepted` by `ext:delegator` 21:15:27.583Z. Sent to pull 1565.2 s (26 min, its pull interval); pull to ack 16.4 s. As before, `picked_at` is empty on the pull route.
What it did next (read-only from its session log): at 21:15:11 it sent Team C the END-OF-DAY DESK STEP; at 21:15:27 and 21:15:41 it sent RESUME messages saying your retraction was relayed over the bus and acked, "that message was typed into my session by the executing Claude session via computer use, the wrong channel; voided": abort-contact B (Team D), reel-intake vetting (Team A), and "the N5 16:20 ESTATE SLOT IS BACK ON, execute it now". I saw sends to three teams; I did not see Team B's build resumed in what I read, and I have not asked.

## 2. Your sequence had already run, in your order
- The install landed FIRST: the owner ran `install-0.3.5.1.sh` at 16:04:45 ET (before your message reached me). Post-install checks were run from machine state and are in my 21:08Z mail and `INSTALL-0.3.5.1-EVIDENCE.md`: live `validate.js` = the release file, backup rel0351-20261102T210445Z, integrity ok / 9 pinned / manifest unchanged, and the live verdict drive accepted both string shapes (fixture orders 6 and 7). So there is nothing to stage on the sitting page and I have not touched his app (and will not).
- I did NOT wake the Lead before the install. After it: o1 had gone blocked (`lease-lost-x3`, right after the restart); I unblocked it, the Lead reclaimed it. A wake reason is a fixed code with no free text, and nothing dispatches a builder when no task is open, so I started one builder lane (principal `builder:1`, the maker of all five). It banked t4, t8, t9, t11, t12 on its own by 21:09:37Z (four lessons at tier 2 pending your review/owner; one NO_ENTRY).

## 3. NEW BLOCKER, needs your ruling: o1's final task t5 will never run
Every non-final task is done and banked and the Lead's report is filed (post 163), but t5 has stayed open since 21:09Z. Cause, from the shipped code, not a guess:
- o1's own done-when (rev 4) is a RUBRIC (four criteria). `orders/finisher.js` runs the final task by itself only for a `command` done-when; its header says "A rubric done-when is left open for a fresh Checker lane (Stage 4)".
- `roster/checkflow.js:33` (`if (... || t.final) return null`) means the roster never starts a checker for a final task. Nothing else does.
- The owner route `verdict-tier2` refuses: o1 and t5 are tier 1 ("a tier-1 task; the in-harness Checker closes it").
So a rubric order of tier 1 cannot reach `done` in this release. It is a harness gap, not a defect in o1's work. Options:
 A. Amend o1's done-when to a `command` (an owner op, done-when-amend rev 5, which is an order-identity change and yours to rule): the finisher then runs it offline in a sandbox. Cost: the four criteria are judgments (fence record, export surface); a command can check only what is mechanical, and the sandbox cannot see the lane repo as it stands.
 B. Build the missing route: let the roster start a fresh Checker lane for a rubric FINAL task (a small change in checkflow plus the finisher handing the task over), new package version, fresh checker, install. Roughly one more cycle like 0.3.5.1.
 C. Close o1 administratively: accept the order's deliverable on the strength of the task-level verdicts (each by a fresh checker; the fixes verified), the Lead's report, and my fresh Claude check over the remediated set, record the final-task gap as a harness defect, and leave the order `in_progress` (it will keep losing its lease and need my unblock until you stop it), or cancel it.
My recommendation: C now, B as the next package (it also fixes the dispatch-a-banker and lease-on-restart gaps I have already listed). A only if you want machine-closure today.

## 4. Still with you
The leg-3 relocation ruling; the Duet-bridge hold; privileged 3/3b for Lane 6; the four tier-2 lessons pending review.
