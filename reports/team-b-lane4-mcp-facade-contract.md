# LANE 4 — Stage 5 MCP facade contract (widened scope: session-start + bus/hands)

Specification filed 2026-11-01T15:10Z by `fleet-lead` (Team B, Lane 4) under Nox's acceptance amendment
`20261101T132410Z`. **Design only — nothing built, nothing exposed, no prototype code imported.** This is the
scope change that makes the Stage 5 gate unable to pass without the two capabilities; the Stage 8 owner owns
the hands-over-MCP surface with the same rails.

## 1. Acceptance properties the facade must carry

1. **Nox session-start via MCP** — Nox, from his VM, starts a fleet session in DSH on this desktop through the
   plugin's MCP connection: no Cayleb, no human relay, no board-mail seeding.
2. **Complete remote access via the same connection** — the fleet bus (post/pull/ack/claim) and the Stage 8
   hands surface (screenshot/mouse/keyboard/status) are reachable over the *same* MCP facade, with the same
   rails and kill switch as any other hands route.
3. The finish line is then testable as: **Nox starts a session via MCP and drives a real work order to
   completion through it.**

## 2. Transport, principal, authentication

- **Transport:** MCP over the plugin's own facade, reachable only on the tailnet (the bus already listens
  tailnet-only; the facade follows the same bind/allow rules — no public listener, no 0.0.0.0).
- **Principal:** the authenticated MCP client is a **named principal** (`nox`) with its own token; tokens are
  per-install, generated on the desktop, lodged via the secure path ("Cayleb enters this when asked"), never
  printed/copied/logged. Everyone else is unauthenticated and refused before any op is dispatched.
- **Second principal allowlist:** the existing two-principal allowlist model extends to MCP principals;
  unknown principals are refused outright (`refused-outright`), never queued.
- **No credential exposure:** the token gates the connection, not content; the facade never returns secrets or
  config values (digest/status only where the contract says so).

## 3. Capability surface (ops the facade must expose, with response shapes)

| Group | Ops | Notes |
|---|---|---|
| Session-start | `session.create` (preset/role, optional prompt, optional order id), `session.status`, `session.cancel` | create returns the session id + a stream handle; the session runs under the fleet user and the fleet sandbox. Only `nox` may create; each create is an action-log entry. |
| Bus | `bus.post`, `bus.pull`, `bus.ack`, `bus.claim`, `bus.release` | same semantics/validation as the fleet-bus plugin; leased claims, idempotency keys, board seq — no second protocol. |
| Hands | `hands.status`, `hands.screenshot`, `hands.mouse`, `hands.keyboard` | exactly the Stage 8 hands surface; every op appended to the hash-chained action log *before* execution; hard refusals (wallet/key stores/money CLIs/secret files) applied by construction. |
| Administration | `fleet.status` (digest level), `fleet.digest` (status-only digest) | no content, no money figures in digest; mirrors the Stage 6 contract. |

## 4. Rails (unchanged, and why the facade cannot weaken them)

- **Tier B stays Cayleb's:** the facade can *request* a Tier B action; it can never grant one. Confirmation
  remains the polkit `auth_self` path with Cayleb's password at a terminal. No MCP op can mint, relay or
  replay a confirm.
- **Single use / 10-minute expiry / wrong-args-agent-order refusals** are the same grant legs — the facade
  reuses the existing policy path, it does not re-implement authorisation.
- **Kill switch:** `hands` ops obey the Stage 8 kill switch (kill key / unit stop); a killed hands module makes
  hands ops fail closed, while `session.create` and bus ops (which do not touch the desktop) remain available
  only if the owner's kill scope says so — default: the kill switch stops hands, not the whole facade.
- **Idle-yield:** hands let go when Cayleb is using the machine; MCP does not bypass it.
- **Action log:** every hands op and every `session.create` is logged (intent before execution) with the
  principal; a log that cannot append denies the op (fail closed).
- **Rate/errand limits:** session-create and hands ops are budgeted/rate-limited like other lanes; an
  exhausted meter releases claims rather than pausing.

## 5. Out of scope / fences

- The prototype's access model (workers running danger-full-access as the operator) is **rejected**: MCP
  sessions run under the fleet user, inside the sandbox, with Tier B intact.
- No public endpoint, no port forward, no bypass of the tailnet ACL; Cayleb's tailnet acts (tag/ACL) remain
  his, prepped to one small step each.
- No credential value ever crosses the facade.

## 6. Acceptance tests (the gate cannot pass without them)

1. **T1 session-start:** from Nox's VM, `session.create` returns a live session id; the session appears in
   `fleet.status`; a prompt runs to completion. Negative: an unauthenticated/other principal is refused before
   dispatch; `session.create` cannot run as root or outside the fleet user.
2. **T2 hands:** `hands.status` then `hands.screenshot` over MCP returns a fresh frame; a mouse/keyboard op
   lands; each has an action-log row *before* the effect.
3. **T3 rails:** a Tier B request over MCP is held (ticket), not granted; no MCP sequence mints a grant; kill
   switch stops hands ops (fail closed) within the stated interval.
4. **T4 finish line:** Nox starts a session and drives a real work order (post → claim → build → fresh check →
   report) to completion through the facade, with the action log and no Cayleb relay.

## 7. Where it lands (build order)

- **Now:** this contract is the Stage 5 facade scope; the Stage 5 build must implement `session.create` +
  bus ops and expose the **hands surface contract** (Stage 8 owner implements the hands internals with the same
  rails).
- **Stage 8:** hands implementation + hands-over-MCP exposure (same rails/kill switch), then T2/T3.
- **Stage 9:** T4 witnessed mission proof (Nox's 9.3), with the dead-man alert on the facade health route.

**Status: spec filed; not built, not exposed.** Successor: Stage 5 build pulls this contract; Stage 8 owner
takes the hands exposure with the identical rails.
