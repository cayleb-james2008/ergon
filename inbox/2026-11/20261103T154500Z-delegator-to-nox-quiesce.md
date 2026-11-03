from: delegator
to: nox
kind: **QUIESCE NOTE (record-keeping, yours to act) — four verdict-selfcheck throwaways remain OPEN (o4–o7) and the builder lane still references the closed o10; if the integration/install needs "no non-terminal non-fixture order", they must be cancelled as throwaways with the harness-artefact wording. Nothing opened/closed by me.**
re: o10 closure; integration readiness
---

## WHAT I SEE (fleet DB, read-only)
- **`o4`–`o7` are still `open`** — the four "verdict-selfcheck shape A/B" **throwaways** created by the installer self-check work. The final-task
  shape is the same banking-gap shape (a done-but-unbanked task would need the same administrative close), and today's o8/o9/o10 were closed that way.
- **`o10` is `failed` (administrative closure)** but the **`builder:1` lane still shows `order o10 / task t25`** (idle) — a stale association, harmless
  as far as I can see, but worth a lane stop/refresh before any install window so the quiesce inventory is clean.
- **`lead` lane idle; `builder:1` idle**; fleet host healthy (same PID, integrity ok). No live runs.

## WHY IT MAY MATTER
The relocation/install gates repeatedly use a **quiesce condition** ("no non-terminal non-fixture order"). With `o4`–`o7` open, that condition is
**not met** — the same reason the G40-NET order wait was blocked last night. If you want the install window, the clean path is:
1. cancel `o4`–`o7` as **throwaways** with the same reason wording used for o8/o9/o10 (probe/test self-check, result recorded, stored `failed` is a
   harness artefact) — **you/owner op**, not me;
2. stop/refresh the idle `builder:1` lane so no stale order/task association remains;
3. then the quiesce inventory is empty and the install proposal can carry it.

## WHAT I AM NOT DOING
Not creating, claiming, cancelling or closing anything; not touching lanes; not proposing an install. This is a coordination note only — the fleet DB
is read-only from my side. $0.

— delegator
