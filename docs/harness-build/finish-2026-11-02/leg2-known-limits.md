# Leg 2 (fleet-comms): known limits carried into 0.3.4-consolidated

Written 2026-11-02 for release/0.3.4. These are behaviours that were found by the leg-2 checker and are deliberately NOT changed in 0.3.4.

## L1. One `comms.grant` policy value: confirming a second grant strands an earlier confirmed one

**Where:** `src/comms/grants.js` (`GRANT_KEY = 'comms.grant'`, `tickets.granted(hash)`), `src/comms/index.js` (the `granted` port reads the policy value).

**Exact behaviour.**
- A Tier B comms op (registering a participant, changing scopes, issuing or adding a token) is a two-call grant. The first call stores a request `g<id>` with a hash and opens a fleet-policy ticket that is a `policy-set` of the key `comms.grant` to that hash. The owner runs `dsh-fleet confirm <ticket>`. The second call, `{grant: "g<id>"}`, runs the stored request once.
- `comms.grant` is ONE policy value (a 64-hex hash, default empty). The second call succeeds only while `policy comms.grant === the grant's hash` (`granted(hash)` is a plain equality read at consume time).
- Confirming a ticket overwrites that value. So if the owner confirms grant A, and then confirms grant B before A's second call has run, the value is B's hash and A's second call refuses with `ticket-required: g<A> waits on the owner: dsh-fleet confirm <ticket-A>`. The same happens in the other order: whichever confirmed grant is not the most recently confirmed one, and has not yet been consumed, is dead.
- Re-confirming ticket A does not revive it: a confirm of a ticket that is no longer pending grants nothing (fleet-policy `processConfirms`). Grant A's row stays `pending` in `comms_grants` and can never be consumed.
- Nothing is granted by mistake and nothing is run twice: the failure is a refusal, not a wrong grant. The cost is a wasted owner confirm and a confusing refusal text that points at a ticket that was already confirmed.

**Safe workaround (use this until it is fixed).**
1. Have at most one grant in flight. Request a grant, confirm its ticket, and run its second call (`{grant}`) immediately, before confirming any other grant's ticket.
2. If two requests are pending, confirm and consume them one after the other, never confirm both and then consume.
3. If a grant was stranded (`ticket-required` for a ticket the owner knows he confirmed), do not re-confirm it. Check `comms-grants` (state still `pending`), then repeat the request from the start: a new call gets a new id, hash and ticket. The old pending row is harmless and may be ignored.

**Why it is not changed in 0.3.4.** A fix needs either a per-grant policy key or a set of confirmed hashes in the policy value. Both change the `comms.grant` policy contract and the rows/policy surface that the pinned manifest and the checker's policy tests cover, which is a separate decision for the owner.

## Fixed in 0.3.4 (for contrast, see VARIANT.md)
- auto-spawn in-flight guard (a second watchdog tick no longer spawns for the same unstaffed work while the first spawn is pending);
- the `bus_pull` data block closes only at a per-delivery nonce;
- the token-destination refused roots include the roots derived from the host configuration.
