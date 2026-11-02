# Finish mission — batch 3: face P1 built + proven in a scratch fleet host; Duet courier (B2) built; 0.3.5 comms integrated

**Filed:** 2026-11-02T16:30 ET (21:30Z) · **Author:** fleet-lead (Team B) · **Status:** built + tested offline; **not installed or restarted on the live host**
(standing fence: no installs/restarts without an explicit ruling). Worktree `~/workspace/harness-build/finish-b-prep`, branch `finish/batch2-prep`,
commits `5008080` (face + duet) and `d42a5ff` (trace); 33/33 unit tests pass; `node --check` clean on every source.

## 1. Face — built against the FINAL Lumen freeze and proven to render/serve in a scratch fleet profile

**Token pin:** Team A final freeze **`2a8cc21bd5f516198ea82a97efc40d3ddff1f7820309fd33b2175ae946670dbe`** (`sha256 FREEZE.sha256`,
`board/probes/team-a-design-v2/`); `tokens.css` sha `19af9c3c…`; face delta `--depth-shade` only. Tests enforce: no literal hex/rgba/hsl in face
code, every `var(--token)` defined by the frozen file, exact pin recorded in `prep/face/tokens/FREEZE.md`.

**Deliverable:** `@cayleb/dsh-fleet@0.3.6-face-prep` (built from the installed 0.3.5 source package):
- tarball `~/workspace/harness-build/finish-b-prep/face-dist/cayleb-dsh-fleet-0.3.6-face-prep.tgz`, sha256 **`e8655231bfa16e15bf7c7c869ce8317c6f11a80d6b3000c74e66c58d62f8329b`**;
- `rows.manifest.json` sha **`8adc1d6b657ea1dd6c9371f089250e0ae16c2624fb57d6058e46c47e956f1777`**, **10 pinned / 12 disabled** (adds `fleet-face` as a mount;
  generator run twice, byte-identical);
- profile overlay `…/face-dist/consolidated-face-overlay.patch.yml`, sha `74e763b0…` (live overlay path-rewritten for the scratch home);
- builder `prep/face/build-face-variant.sh` (parameterised; no core logic changed — client bundle, one server route module, one insert row, manifest regen).

**What the face contains:** own `main['fleet']` page, `sidebar.panellist { id:'fleet', order:40 }`, settings section, status pill (live/partial word, no
colour-only signal); eight panels — fleet status (integrity/lanes/manifest), lanes, orders (fixture rows carry the FIXTURE word), meters (explicit
"not wired", never zero-invented), agent activity, phone VM, Windows VM, Studio cells. Live data comes from the **P1 same-origin route**
`GET /fleet-face/snapshot` registered by `src/face/plugin.mjs` through `ctx.webServer` (no CORS, no Typert generation needed); the client refetches every
5 s and renders an explicit **"live data not connected yet / partial"** state when the route or a section is absent. Fixtures exist only in the dev
harness (`dev/harness.html`, labelled FIXTURE) and are never imported by the shipped bundle (asserted by test).

**Scratch proof (no live writes; DOM-free, no image reads):** fresh scratch DSH home + runtime, profile from the web template, variant installed,
live 0.3.5 overlay path-rewritten. Results: `--dump-config` rc=0, 62,602 B, `fleet-face` row exactly once, `@cayleb/dsh-fleet/face` composed once,
`fleet-comms` once; host boot with **no errors**; control status ok; **`GET /fleet-face/snapshot` → HTTP 200, `live:true, errors:{}`** with sections
`status/lanes/orders/meters/activity` (and `vms/studio: null` = honest not-wired); the web index boot graph contains
`@cayleb/dsh-fleet/client.js&rev=f194692c1dde`; the host-served bundle (10,442 B) contains the face markers. Host stopped, ports closed.

**Named blocker:** the live fleet profile (:3092) still runs the stock 0.3.5 client. Wiring the face there needs a profile install + fleet-host restart —
**explicit ruling required** (no installs/restarts under the standing fence). It is one command + the dump gates above once allowed; the artifact is ready.

## 2. Duet courier — B2 wording implemented and tested (no wake/push claim)

`prep/duet/drive-mailbox.mjs`: Google Drive REST v3 adapter over an injected `fetch` (hosts pinned to `www.googleapis.com`; multipart create for the
handoff JSON + the `ACK <id> <nonce>` stub; list/read stubs; no listener, outbound HTTPS only). `prep/duet/proof.mjs`: the B2 proof shape —
`delivered_at`, `picked_at`, `acked_at`, `pickup_latency_ms`, `ack_latency_ms`, **`measured_wait_ms`**, `wake_claimed:false`, out-of-order and
bad-timestamp refusals. 10 tests (4 new) cover the adapter contract, host pinning, nonce/ACK content and the proof shape. Existing courier core
(`pump/collect/sweep`, fence/oversize/replay/unknown-handoff refusals) unchanged.
**Named blocker:** real Drive hookup needs the owner's Google consent + a mailbox folder (owner sitting); no account or network was touched.

## 3. Comms integration note — integrate with 0.3.5, do not rebuild

The installed 0.3.5 `src/comms` is authoritative: `fleetComms` (`who`, mailbox `pull/ack/status`, registry `comms-list`, presence, wake to lanes only,
pull-only participants never signalled), six facade tools `bus.*`, seeded `cayleb-local`/`nox`. Our `prep/comms/*` modules are **superseded** — kept only
as reference/conformance (token scopes matrix, T1/T2/T3 transport semantics, watchdog scoping tests) and for the face's data contract. The face consumes
`fleetComms.who()` in the snapshot; no duplicate bus, registry or token stack will be built. Two integration facts to carry: lane wake already exists in
0.3.5 (`wake.js`, coalesced FLEET-WAKE, max 4 signals, daily cap), and owner/delegator mailboxes are pull-only until the owner issues tokens (Tier B
blocked by the helper gap, B4 — unchanged).

## 4. Elevation status

A+ spec + Lane 6 brief remain on file (`prep/elevation/LANE6-BRIEF.md`, `owner-approval.mjs` with helper-proof-only grants, one-shot consume, argv
hygiene, 4 tests). Privileged helper build + install + first run stay **held for the owner's home sitting** (Nox ruling); no privileged action taken.

## 5. Incident (self-reported, restored)

While closing a scratch listener I killed the wrong PID: `openbell-dash.service` (pid 1481) owns 127.0.0.1:8791 — the scratch host had failed to bind that
port and had continued with MCP unavailable. Restored immediately with its own unit: `systemctl --user start openbell-dash.service` → **active/running,
listening 127.0.0.1:8791 + tailnet, NRestarts 0**, log clean (`OpenBell dashboard: http://127.0.0.1:8791`). Read-only dashboard; no data loss. The scratch
host never bound 8791 (it showed only its web port), so no other conflict existed.

## 6. Named blockers (no ruling assumed)

1. **Live face wiring**: install the `0.3.6-face-prep` artifact into the fleet profile + restart the fleet host → explicit ruling needed; artifact + gates ready.
2. **Duet Drive hookup**: owner Google consent + mailbox folder (owner sitting).
3. **Owner Elevation**: privileged helper install/first run = owner sitting; Lane 6 gate precedes the build (Claude-lane helper).
4. **VM/Studio panels**: still honest not-wired — they need the phone/Windows supervisors and Team D's containment launcher as rows (next build slice, after rulings).
5. **Meters**: needs the ledger read port in the snapshot (spend today / priced events); today it renders "not wired", never 0.
