# Tier A / hand-placed confirm ledger (executing Claude session)

Rule (Nox, 2 Nov 09:24 ET): each hand-placed grant needs its own explicit manager ruling from Nox until the Stage 7 factor exists. No standing authority.

## 1. Ticket 100002: o1 budget raise +0.15 USD
- Ruling: Nox, 09:24 ET, "Authorized. Hand-place the confirm for this one Tier A ticket only"; ruled cap 1.00 USD; this ticket raises o1 0.50 -> 0.65 (within the cap). Never a larger sum than the Lead's own request.
- Ticket: id 100002, class spend, tier A, tool budget_raise_request, agent lead, order o1, args_sha256 357d56848bc0ec1879c133ad5637a272f6dd9e37b04abe1349d55225846efc31, created 14:23:24Z.
- Duplicate 100003 (the Lead's identical retry, same hash) voided via owner op ticket-void at 14:25:16Z (event 199) so one raise cannot become two.
- Confirm file placed 2026-11-02T14:25:16Z in <DH>/fleet/state/confirms/100002.json, fields: ticket, pkexec_uid 1000, at, via "hand-placed by the executing Claude session under Nox manager ruling (not the polkit factor)", provenance nox-manager-ruling-20261102T1425Z, scope. Consumed by the poller within 5 s (dir empty).
- Journald: `logger -t dsh-fleet-confirm` line "ticket 100002 confirmed by hand-placed file under nox-manager-ruling-20261102T1425Z (not polkit ...)".
- Plugin audit: event 202 policy.grant approver `cayleb-local:polkit`, issued 14:25:16Z, expires 14:35:16Z. LIMITATION: the plugin hard-codes this approver label and ignores extra confirm-file fields, so its own chain does NOT show the true provenance. This ledger, the journald line and the board filing are the provenance record. Owner Elevation build item: record approver provenance in the grant row.
- Raise executes on the Lead's identical repeat call within the grant window.

## 2. Ticket 100004: o1 budget raise +0.15 USD (the Lead's re-request after 100002 lapsed)
- Ruling: Nox, 09:49 ET, "Authorized - same terms as the lapsed grant ... this ticket only, for exactly the requested amount up to the approved $1.00 cap".
- Ticket: id 100004, class spend, tier A, tool budget_raise_request, agent lead, order o1, +0.15 (o1 0.50 -> 0.65), args_sha256 d72a79a94bb639e9ff0a0b89a4248bb28499ecc3eab51d56edc4427e39e1a26*, created 14:44:10Z.
- Guarded script verified the ruled facts before placing (pending / spend / A / o1 / lead / 0.15; budget+0.15 <= 1.00). Duplicate 100005 (Lead's identical retry) voided at 14:50:09Z.
- Confirm file placed 2026-11-02T14:50:09Z (provenance nox-manager-ruling-20261102T1450Z); consumed (dir empty). Journald line written. Plugin event 304 policy.grant approver `cayleb-local:polkit` (label hard-coded by the plugin; true provenance is this ledger + the file fields + journald + the board filing). Grant window to 15:00:10Z.
- o1 unblocked at 14:50:2xZ (event 305, post 25) with instructions for the Lead. The Lead stayed `idle`: nothing wakes a lane (no lane-wake op in 0.3.3). The raise executes only on the Lead's identical repeat call, so it is PENDING EXERCISE at the time of writing.

## Ticket 100006 (standing ruling): o1 budget raise +0.15 USD
- Authority: Nox standing ruling 09:54 ET (o1 only, exactly +0.15, cap <= 1.00, honest provenance, ledgered, reported after). Facts verified by script before placing: pending / spend / A / budget_raise_request / o1 / lead / 0.15; budget 0.5 + 0.15 <= 1.00. args_sha256 d72a79a94bb639e9ff0a0b89a4248bb28499ecc3eab51d56edc4427e39e1a269.
- Confirm placed 2026-11-02T15:41:52Z (provenance nox-manager-ruling-20261102T1541Z); consumed by the poller; journald line written. Plugin audit shows the hard-coded approver label cayleb-local:polkit (true provenance: this ledger, file fields, journald, board report). Same-hash duplicates voided first.
