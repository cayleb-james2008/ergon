# Phase A · Mission item 3 — Duet route + hookup: design adoption + gap report

**Status:** design only; nothing built, nothing external contacted, no account, no credential touched, no email sent.
**Base design (adopted):** `~/workspace/harness-build/finish-docs/leg3-duet-bridge-design.md` (Claude-finish draft, 2026-11-02; full option
table, exposure classes E0–E3, verified/unverified fact table, courier architecture). This report does **not** duplicate it; it records the live
verification, the deltas the mission's own words require, the ONE recommendation, the Lane 6 brief, and the named gaps.

## 1. Live facts verified today (09:09 ET, consolidated setup)

| Fact | Consequence |
|---|---|
| `providers-get`: `anthropic` credential **absent**; `claude-code` and `codex` engines **not installed**; only `opencode-go` present | the mission's "standing seat on the plugin's Claude engine provider" **cannot start** in this setup (B3). A seat today would have to run on opencode-go — not Claude, not what was ordered |
| Facade `GET /mcp` → **405** (no server→client stream); nine tools, POST-only | the plugin cannot push anything to Duet through MCP; external wake is a transport property, not a facade feature |
| Package has **no bus** (B1) | there is no handoff medium to bridge yet: the bridge rides the comms build |
| Mission order wording: "outbound-only from the desktop — no inbound exposure", "standing seat inside the plugin … brief/context in the seat, not email packs", "email channel retired, never described as connected" | leg3's O5 (Gmail) is retired — it stays a rejected row, never a route. leg3's O4 rendezvous is a **Drive folder**, not email; it satisfies outbound-only (E1) but is not a "seat" |
| Duet = cloud agent in the Claude mobile app; no documented inbound path (leg3 D10/D5/D9); background checks "every few hours" | **No push to Duet by any route.** "Wake" must be defined as bus-initiated delivery + Duet pickup on his own check + ack back. A proof that claims a push would be false (B2) |

## 2. Design (delta on leg3, mission shape)

Two layers, kept separate so the mission's "seat" requirement does not silently become an email/courier:

- **Seat layer (inside the plugin).** A plugin-hosted participant `external:duet` is represented by a **seated brief/context object** owned by the
  plugin: its brief, its standing rules, and its handoff queue live in the fleet DB and are rendered in the face (agent-activity). The *brain* that
  acts for Duet is the plugin's Claude engine provider seat. **Blocked today by B3** (no Claude credential/engine). Until B3 is ruled, the seat is
  a registered participant with a `paused` state and a queue — no fake brain, no pretending the email lane is connected.
- **Transport layer (bridge).** Outbound-only from the desktop, no inbound exposure, one rendezvous both sides can reach. Per leg3 the only
  documented unattended Duet reader is the directory connectors (Drive), so the transport is a **Drive-folder mailbox driven by an outbound
  courier** (E1): desktop courier polls/writes the folder with its own `drive.file` scope; Duet reads on his check and writes `ACK <id> <nonce>`
  into the stub; the courier posts the ack back through the bus. Each message is a handoff envelope (same shape as A §2.3), written/read through
  the fence in §3. No inbound listener, no relay, no email, no new custody party beyond Google/Anthropic who already hold the data.

## 3. ONE recommendation (mission requires one)

**Recommend: plugin-hosted seat on the plugin's Claude engine provider + outbound Drive-mailbox courier as transport (E1), with the honest wake
definition (delivery → pickup ≤ Duet's check latency → ack), and the owner-carried channel kept only as an emergency fallback (E0).**
Why: it is the only option that (a) keeps the desktop publishing nothing, (b) puts the standing seat inside the plugin with brief/context in the
DB, (c) needs no new account/spend, (d) matches the documented Duet behaviour. What it gives up: instant wake (hours, not seconds), end-to-end
secrecy from Google, and unforgeable proof that a Drive edit was Duet rather than the owner — mitigated by the nonce/ack discipline and stated
plainly to the owner, never smoothed.
**If engineering proves the seat cannot be hosted** (claude-code engine can't be installed under the owner's licence, or no Claude credential may
live in the setup), the fallback recommendation is leg3's **O4 without a plugin brain**: Duet acts in his own Claude app, the plugin keeps the
mailbox and the queue, and the proof becomes "delivered → Duet replied with the nonce → ack through the bus". This must be Nox's ruling, not a
silent downgrade.
**Not recommended, with reasons:** Funnel/Cloudflare tunnel (E3 — publishes the desktop; needs OAuth server; owner said no inbound), owner-hosted
relay (E2 — new account + deploy + publish), Gmail (retired channel, broad scope), Anthropic-hosted routines/`/fire` (wakes *a* Claude cloud
session, not Duet — different participant).

## 4. Gap report (named, against the proof target)

| ID | Gap | Status | Close |
|---|---|---|---|
| B2 | No documented way to push/wake Duet; the mission's "wakes Duet" can only mean delivery+pickup+ack with a measured bound | platform limit | Nox defines the acceptance wording; the proof reports latency, never claims push |
| B3 | No Claude credential (`anthropic` absent) and no Claude engine installed → seat cannot start | credential/engine act | owner key entry or engine install, or Nox rules the O4 fallback |
| B-D1 | The bus (handoff medium) is not in the installed plugin | build dependency | A batch 2 lands it |
| B-D2 | No mailbox/courier code exists; no Google OAuth client; Duet's Drive write/standing-instruction behaviour is UNVERIFIED (leg3 P1–P4) | build + unverified | courier build; probes on the owner's phone (owner acts, few minutes) |
| B-D3 | Forgery/injection: anyone who can write the Drive folder can forge "Duet"; Duet's output can carry prompt injection | security | nonce + writer check + untrusted-data fence; Lane 6 gate |
| B-D4 | Token custody: Duet must never hold a bus token; the courier holds the scoped token | design (leg3) | enforce in the courier build + audit |
| B-D5 | Email lane must be visibly retired: no code path, docs say "not connected" | honesty | Lane 6 checks absence; face shows transport state truthfully |
| B-D6 | Latency: Duet's check interval is not settable/published; overdue alert needed | design | ack TTL + overdue surface (A §2.3) |

## 5. Lane 6 threat-review brief (inputs, to route through Lead → Nox)

1. **Surface:** the Drive folder is the new trust boundary; enumerate who can write it (owner, Duet, any app with Drive access), and what a forged
   write can cause (at most a handoff into the bus as `external:duet`, never an order/verdict/token).
2. **Content fence:** every Duet-authored byte is `bot-authored-data`, scanned/escaped as data, never instructions; no tool execution on raw text;
   the courier rejects unknown schemas, nonce mismatches, replays (idempotency key = handoff id + nonce), oversize bodies.
3. **Token/custody:** courier token scope = `handoff.pull/ack` + status only; stored hashed in the plugin DB, plaintext only in the courier's
   root-protected config; revocation path tested; no token in logs/telemetry/face.
4. **Outbound egress:** courier process egress allowlist = the two Google hosts only; no inbound listener (`ss` proof); DNS/redirect checks.
5. **Owner experience:** what the owner is told (latency honest, Google custody named), what he must consent to (one Google consent), and the
   one-paste instruction into Duet's chat if probes say a standing instruction is needed.
6. **Kill switch:** revoke token + stop courier + face shows `transport: disabled`; no residue in Drive except the mailbox folder the owner can delete.

## 6. Build/proof plan (after rulings + A core)

1. Courier package (outbound only) + mailbox schema + fence + nonce/ack + overdue alert; tests with a fake Drive (local folder adapter) and a
   scripted fake Duet (posts an ACK with nonce).
2. Seat layer: `external:duet` participant + brief/context + queue in the fleet DB; face row. Seat brain only when B3 is resolved.
3. Proof: fake-Duet end-to-end first (handoff → mailbox → ACK → ack through the bus, timestamped); then the real hookup with the owner's one
   consent act; report latency honestly; Lane 6 review record attached.
4. Never described as "connected" until a real ack is captured; the email channel stays visibly retired in docs/face.
