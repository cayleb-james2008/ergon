# R5 — Desk task: ticket round trip and a real fleet order (pass 1)

- Step: R5 — Desk task: ticket round trip and a real fleet order
- Pass: 1
- Owner: driver (Team A / ergon-lead); maker never checker — a fresh-context checker re-runs and signs
- Operator action: operator raises a ticket from an employee device, works it on the management screen (claim, reply, receipt), and confirms the employee sees the resolution; the warehouse-board task is worked as a fleet order on the platform
- Expected: the ticket moves queued -> working -> resolved with a stored receipt, and the employee device reads exactly the host record; the fleet order reaches done with its task done+banked and every op id recorded
- Actual: 23/23 hard checks held; 0 named pending; 4 recorded finding(s)
- Word: **clean**
- Evidence files: `R5/tickets-round-trip.json`, `R5/fleet-order-ops.json`, `R5/fleet-order-readback.json`
- Timestamp (UTC): 2026-11-01T23:08:06Z .. 2026-11-01T23:08:07Z
- Blocker / finding: none

## Checks
- PASS a1 ticket stored queued from the phone device, duplicate false — ticketId=c060a8cc-e551-4426-8e9b-c60ee7e4ba7d state=queued duplicate=false
- PASS a1 host record: state queued, device teo-phone, the exact reviewed preview — state=queued device=lf-teo-phone subject=Which line for the Harlow Legal job, C or D?
- PASS a2 operator queue lists the new ticket first (newest first) — first=c060a8cc-e551-4426-8e9b-c60ee7e4ba7d new=c060a8cc-e551-4426-8e9b-c60ee7e4ba7d total=100
- PASS a3 worker credential claims the ticket and gets a real lease — lease=issued (64 hex) leaseUntil=1793625000000
- PASS a3 ticket reads back working with the claiming worker — state=working worker=agent-sam-akintola
- PASS a4a worker reply refused (agent cannot reply on current code — F4 recorded) — HTTP 403 worker_route_denied (SCENARIOS.md F4: only the operator token can reply)
- PASS a4b operator (Sam, the person) posts the reply — replyId=lf-1-lf51-mvyfxuw4-d45a83-a4 duplicate=false
- PASS a5a agent resolved-finish outcome recorded against SCENARIOS.md F3 (409 remote_binding_required unless the device is remote-bound) — HTTP 409 remote_binding_required
- PASS a5b operator resolves the ticket with the repair receipt — state=resolved
- PASS a6 device read equals the host record: resolved, Sam's reply, the receipt summary — state=resolved replies=1 receipt=Told the warehouse lead: line C.
- PASS b1 ticket stored queued from marcus-desktop — ticketId=6cbaf01e-2bb9-4bf8-b22e-615df82cd1dd state=queued
- PASS b2 ticket resolved with the reply and receipt, and Marcus's device reads both — host=resolved device=resolved replies=1
- PASS b3 the management record names the worker who handled it — worker=sam-akintola
- PASS both tickets have a product audit trail (stored -> claimed -> reply_added -> resolved) — audit rows=8: stored,claimed,reply_added,resolved,stored,claimed,reply_added,resolved
- PASS B1 fixture order created open on the installed plugin — order=o3286 state=open duplicate=false
- PASS B2 lead claims the order and closes its pre-flight — preflight=closed
- PASS B3 order decomposed to one board-shaped task — task=t4162 order_state=in_progress
- PASS B4 builder claims, posts evidence and submits the task as built — state=built evidence_post=7449
- PASS B5 a checker runs and records a pass verdict — checker=checker:selftest-chk-e8546ac7c19d verdict_post=7451 task_state=done
- PASS B6 the order rolls up to done through its own done-when — state=done waited=517ms
- PASS B6b the board task is done and banked, and the order's final task is done+banked — task=done/banked=true final=done/banked=true
- PASS B7 every board post the pipeline wrote is readable (evidence + verdict + order-created + report) — posts=13: order-created,preflight,preflight-answers,decomposition,evidence,built,verdict,task-done,report,built,verdict,task-done,order-done
- PASS every fleet op answered ok and is recorded — ops=12, refused=none

## Findings recorded (not a failure of this step)
- product limit F4 — agent ticket-worker credential cannot post a reply — observed HTTP 403 worker_route_denied
- product limit F3 — agent ticket-worker credential cannot finish resolved without a remote binding — observed HTTP 409 remote_binding_required
- product limit F6 — the ticket record carries a free-text worker and replies have no author; Dana and Sam share one operator token — observed worker=sam-akintola reply-authors=0
- warehouse board app — The warehouse-board app source lives only on the offline work PC (PACKAGE-PENDING). This order is a real fleet order carrying the board-shaped task; the board state it names is the fixture's scripted board (see R1: board.scriptedState=true). Named stand-in: the board app, never counted clean.

## Notes
- Fleet order ops (all on /run/fleet/control.sock, peer uid 1000): order-create -> order_claim (selftest lead) -> preflight_raise -> preflight-close -> task_decompose -> task_claim (selftest builder) -> evidence_post -> task_submit_built -> check-start -> verdict (selftest checker) -> lesson_propose (bank the task) -> report_file

## Recorded metadata
```json
{
  "orderId": "o3286",
  "taskId": "t4162",
  "verdictId": "7451",
  "evidenceId": "7449",
  "ops": 12
}
```
