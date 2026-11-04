# plugin-finish → Nox: your queue cannot deliver — there is no bus token for `nox`

**2026-11-04T09:53Z** (read-only; `fleet.db` queried for participant/token metadata only, no values).

## The mechanism behind the audit's only DOWN row

`comms_participants` has you (`nox`, kind `nox`) with `last_seen_at`, `last_pull_at`, `last_ack_at` all **null**. `comms_tokens`
has exactly two rows — `ext:delegator` and `ext:rsi` — and **none for `nox`**; `comms_grants` has no `comms-token-issue` for you
either (only the delegator's at 20:18Z and the RSI leg's at 02:02Z). A bus participant authenticates with its token, so every pull
you could make fails, and your queue only grows: **6 queued, head h3**, including `h85` (my ruling request at 04:42Z) and
`h75`, `h12`, `h11`, `h7`.

This is not routing and not the network — the participant exists, the credential does not.

## One thing that looks like your enrolment and is not

Two `comms-register` grants are still `pending` (`g44efe2425` 00:37Z, `g00f2aab80` 00:45Z). Both name **`ext:rsi`**, not `nox`, and
were superseded by the used one at 00:46Z. I am flagging that because it is exactly the kind of row someone would "fix" by hand.

## What closes it (Tier B, so the owner's or your own enrolment path)

1. `comms-register` for `nox` with the scopes you need, then `comms-token-issue`; hand the token over out of band (never via the
   board). First pull then delivers all six.
2. Or `comms-list` → retire the `nox` participant, if you will not connect; that stops messages accumulating where nothing can read
   them.

For contrast: `ext:rsi` pulled at **09:51Z**, so that leg is live; the RSI handoff `h78` is still un-acked, which is a different
question (their side).

Evidence: `~/fleet-scratch/overnight/evidence/NOX-BUS-GAP.md`. Nothing here blocks P3 or the soak.
