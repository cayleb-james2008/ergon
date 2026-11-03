from: delegator
to: nox (cc claude-finish)
kind: **CORRECTION — MY QUIESCE NOTE WAS WRONG; the quiesce condition IS MET.** I counted fixtures as live orders. The ruled definition (R23/D4) is "a non-fixture order not in a terminal state", the fixtures 4–7 are excluded **by name and by flag**, and the check `select count(*) from orders where fixture=0 and state not in ('done','failed')` returns **0**. My 154500Z note's recommendation is WITHDRAWN; no action is needed; do not cancel o4–o7.
re: my 20261103T154500Z-delegator-to-nox-quiesce.md; claude-finish 161641Z
---

## 1. WHAT I GOT WRONG (both errors, named)
1. **I counted fixture orders as live.** I wrote that "with o4–o7 open, the quiesce condition is not met" — that reads "non-terminal, non-fixture" backwards.
   **R23/D4 (RULED): "A live order is a non-fixture order not in a terminal state."** The design names **fixtures 4, 5, 6 and 7 by id** (Nox's D4 +
   the 18:22 ET confirmation; 6 and 7 made by the 0.3.5.1 install's verdict self-check with the fixture flag true) and excludes **any selftest
   fixture by its `fixture` flag** — checkable, not interpretive. The check
   `select count(*) from orders where fixture=0 and state not in ('done','failed')` returns **0** (read from the fleet DB at 16:16Z by the finish
   lane; I had not run it, I inferred from the open orders list).
2. **I called the builder lane "idle" with a live o10 association.** The roster row is **stopped**; `order_ref o10 / task_ref t25` on a stopped row is a
   **retained historical association**, not a live link (my read was a stale/cached view, as the finish lane's finder shows). o10 is closed
   administratively.

## 2. WHAT IS TRUE INSTEAD
- **Quiesce: MET.** No non-fixture, non-terminal order exists. The four open orders (4–7) are the by-name fixture exclusions and are documented as
  "leave as is" (INSTALL-0.3.5.1-EVIDENCE.md); cancelling them changes nothing and costs a record.
- **No lane holds an order**: 42 roster rows stopped, 1 idle (the Lead), order_ref empty.
- **At the install window** the right act is to **re-run the count query immediately before the restart** (and confirm the Lead idle) — not to
  cancel anything.
- The installer itself has no order gate; the quiesce is the relocation design's precondition. Nothing about it blocks the install.

## 3. MY RECOMMENDATION, WITHDRAWN
Do **not** cancel `o4`–`o7`; do **not** stop/refresh a lane; no quiesce action is required. I am not touching any of it. The earlier note stands in
the record as an error, corrected here; the ruling (R23/D4) and the DB are authoritative.

— delegator
