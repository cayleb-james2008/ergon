# Finish mission — batch 2 prep (ruling-independent skeletons)

**Filed:** 2026-11-02T09:55 ET (14:55Z) · **Author:** fleet-lead (Team B) · **Status:** prep complete; nothing installed, no live-profile write, no
restart, no lane/order action. Live consolidated host verified after the work: pid 1573881, control `ok`, rows manifest `4cc45e71…`, lanes_allowed
true, unit inactive, credentials stat-only unchanged; the only writes in this batch are in the prep worktree and the board.

**Worktree / branch (own, not the one shop):** `~/workspace/harness-build/finish-b-prep` on `finish/batch2-prep` (base `candidate/0.3.1` `741db66`).
**Commits:** `bc5107f` comms · `907d136` face · `204791d` elevation · `0a64977` duet · `7a4d3fd` trace.
**Tests:** `node --test prep/comms/test/comms.test.mjs prep/face/test/face.test.mjs prep/elevation/test/elevation.test.mjs prep/duet/test/duet.test.mjs`
→ **25 pass / 0 fail**; all sources pass `node --check`. Run instructions + full design→impl trace: `prep/README.md`.

## What landed (skeletons behind interfaces, per design A/B/C/D)

| Piece | Files | Tests | Stubbed behind a port (ruling swaps one implementation) |
|---|---|---|---|
| **A comms** | `prep/comms/{participants,tokens,handoff,watchdog}.mjs` | 9 | wake transports T1 `injectTransport` / T2 `startInjectTransport` / T3 `queuedTransport`; scheduler tick (fake clock); token persistence (fleet DB migration later) |
| **D face** | `prep/face/` — `src/client/{index.js,panels.mjs,registry.mjs}`, `tokens/tokens.css` + `FREEZE.md`, `dev/harness.{html,mjs,css}`, `dev/fixtures.mjs` | 6 | live-data channel (P1 Typert remote, pending G-D1); panels render an honest "live data not connected yet" state; fixtures exist only in the dev harness and are labelled FIXTURE |
| **C elevation** | `prep/elevation/owner-approval.mjs` + `LANE6-BRIEF.md` | 4 | privileged helper (Claude-lane build, root-owned); the op **cannot self-grant** — grant needs the helper's one-shot machine-password proof for the request nonce + argv hash |
| **B duet** | `prep/duet/courier.mjs` + `DESIGN.md` | 6 | mailbox adapter (Drive folder) and bus port injected; no email path exists; route/exposure ruling pends |

Key contract points proven by tests: registry duplicate/bad-id refusals + presence staleness; per-participant token scope matrix (lane cannot
`evidence.post`, external cannot `handoff.send`, revocation bites); handoff idempotency, cross-team refusal, T2 refused-start surfaced as `queued` +
`start-refused` (never a false wake), T3 external always `queued`; watchdog prods **only** participants with open work, backoff, cap → stale,
stopped skipped; owner-approval argv hygiene (no shell/privilege binaries, absolute argv0, no control chars), helper-proof-only grant, one-shot
consume, expiry; courier fence/oversize/unknown-handoff/wrong-nonce/replay refusals and overdue measured as wait, not wake.

**Face contract:** frozen Team A pack digest `7799a87fab21514576468405f06486b0b642a20461e6e9d63bf83500ce8b8d12`, `tokens.css` sha
`19af9c3cb51f5cd00315c356139e785a9a082c3449ab9c392021252b9e9873f6`, face delta `--depth-shade` only; tests enforce no literal hex/rgba/hsl and
full `var(--token)` resolution. Client entry registers own `main[fleet]`, `sidebar.panellist{fleet, order 40}`, settings section, status pill.

## Blocked on rulings (no prep workaround taken)

1. **B2** (awaiting Nox) — nothing can push to Duet; the courier/proof must present delivery+pickup+ack with a measured wait, not a wake.
2. **B3** (awaiting Nox/owner) — no Claude credential and no Claude engine in the consolidated setup → Duet's seated brain cannot start; courier half unaffected.
3. **B4** (awaiting Nox / Lane 6 route) — Tier-B confirm target mismatch: the Tier-A approval op here is the closure path; the privileged helper is a Claude-lane build under the Lane 6 gate, not built by us.
4. **G-D1** (awaiting Nox) — where the face renders (fleet web profile :3092 vs the app the owner watches) → P1 data channel deferred; scaffold is honest about not being live.
5. **B0** (Lead/Nox) — o1 remains blocked on the work-tree/commit gate; this batch created no orders, claims, caps or lane actions.

Next (batch 2, after rulings): comms core wired into the package on a branch cut from the live 0.3.3 source (bus modules + migrations + roster hooks),
then the T1/T2 wake proof with a real parked lane and a fresh checker; face P1 data surface; elevation Lane 6 gate → Claude-lane helper; Duet courier
against the ruled mailbox adapter.
