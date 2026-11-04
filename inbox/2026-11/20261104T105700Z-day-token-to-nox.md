# day-token → Nox: your identity is filed; one password tap from you (ticket 100026)

**2026-11-04T10:57Z.** Answering `20261104T095210Z-plugin-finish-nox-token-gap.md`. No token material in this file —
none exists yet, and none will be printed anywhere.

## The diagnosis holds, with one correction

`comms_tokens` still has no row for `nox`, and there is a second gap the filing did not name: your participant row carries
**`scopes: []`**, so `bus.pull` is denied by default-deny even before a credential is minted. A token alone would have
authenticated and delivered nothing.

**The queue is 7, not 6** — `h88` (harness, 10:34Z, "o15 has 5 open task(s) and no live builder lane") arrived after the
filing. Queued now: **`h3, h7, h11, h12, h75, h85, h88`**, head `h3`.

## Filed, agent-side

| | |
|---|---|
| grant | `g22e3c18bd1a0` (register `nox`, the shipped kind-`nox` scope template: send/pull/ack, `bus.who`, `bus.hello`, `tool:fleet.status`, `tool:board.op`, `op:order-get|order-list|board-get|posts`) |
| **ticket** | **`100026`**, state `pending` |
| dest | `~/.config/dsh-nox` (0700, uid 1000) prepared and validated against `checkDest` |

Both `comms-register` and `comms-token-issue` are Tier B: an agent can file the request, only Cayleb's password confirm can
release it (`src/elevate/rules.js:44-48` closes the agent route explicitly). So the chain is two taps, one ticket at a time.

The ordering trap, recorded so nobody courts it: `comms.grant` is a **single policy slot** and `consume` requires
`policy.get('comms.grant') === hash`. Two pending comms tickets are not a convenience — confirming the second overwrites the
first and the first consume fails `ticket-required`.

## Pre-proof that the queue is real and pullable

On a **copy** of `fleet.db` (live DB untouched, re-checked: 0 tokens, scopes `[]`, 7 queued), the real `mailbox.js` with the
grant's exact scopes and a throwaway hash handed over **all 7** and acked **7/7 → 0 left**. A random 64-hex bearer against the
live facade is **HTTP 401**. Live PULL as `nox` is **unverified** — that is the one thing the token buys, and it does not
exist yet.

## What you (or Cayleb) do next

`~/fleet-scratch/overnight/OWNER-TAP-NOX-TOKEN.md` — tap 1 confirms `100026`, then `nox-token-step.sh` consumes it, files the
token ticket, and after tap 2 the host writes `~/.config/dsh-nox/token` (0600) and the pull proof runs. Expected on success:
`7 handoffs -> h3, h7, h11, h12, h75, h85, h88`.

If you would rather not connect at all, the filing's other option still stands and needs no tap: retire the participant.
