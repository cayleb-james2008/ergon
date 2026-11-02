# Phase A · Mission item 1 — Visible face: integration plan

**Status:** plan only; nothing built, no app reload, no image read, no fixture shown as real. **Owner:** Team B leads the build; Team A supplies the
design-pack token contract; Team D supplies the Studio containment recipe; Claude lanes execute harness-sensitive client pieces per the standing rule.
**Contract sources (binding):** `board/reports/team-a-fleet-face-token-contract.md` (tokens/floors only, pinned set digest `ea68e398…` at issue,
polish re-freeze pending — the builder pins the digest it builds against, **no hand-typed hex**), `board/reports/team-d-studio-instances-design.md`
(§5 read-only status contract, owner-boundary refusals), DSH `packages/client/AGENTS.md` + `docs/subsystems/slots.md` (slot/props discipline).

## 1. What exists in the installed plugin today (verified)

- The client half is a **hand-built Stage-1 skeleton** (`src/client/client.js`, 3.5 kB): one settings section (`plugins.bundle.config`, key = package
  name, renders `#fleet-settings` read-only placeholder), an **inert** `main` registration `key: 'fleet'` (renders `null`), an **inert**
  `shell.overlay` pill, and **no `sidebar.panellist` entry** (deliberately not registered in Stage 1 to avoid an empty sidebar row).
- `dsh.client`: `platform: web`, inject `@deepseek-ai/dsh-client-locale` + `@deepseek-ai/dsh-client-ui-plugin-manager`. The bundle is loaded via
  `window.__ModuleLoader__.load({id, factory})`; exports only `apply`/`inject` (contract discipline).
- **No data channel**: no `@Remote` surface, no `ctx.remote.<ns>`; the plugin server side is the MCP facade (POST-only, nine tools, loopback/tailnet,
  one bearer token) and the control socket. Client plugins in this codebase read host data through Typert-generated `ctx.remote.<namespace>` methods
  and `ctx.remote.$on(...)` host events (see `packages/client/ui-plugin-manager`), which requires the Typert generation pipeline — the fleet
  package does not have one.
- The one live app: consolidated `fleet` profile GUI on `127.0.0.1:3092` (web profile), owner's Desktop app is a separate `desktop` profile; the
  face must render **in the owner's app** = the app he is using. That app currently runs the `desktop` profile, not the `fleet` web profile → the
  face has to reach him through whichever surface he actually uses; this needs an explicit decision (see G-D1).

## 2. Contract compliance (non-negotiable build rules)

| Rule | Enforcement |
|---|---|
| Own `main[key]` only (`fleet`), own `sidebar.panellist` entry `{ id: 'fleet', order, label: () => t('panel') }` | slot names declared exactly; no other plugin's key touched; no `ctx.slots.register` on undeclared domains |
| Tokens only, no hand-typed hex | import generated `tokens.css`; self-check `grep -rE '#[0-9a-f]{3,8}\b\|\brgba?\(\|\bhsla?\('` over built face (excluding the generated token file) = **0 hits**; every `var(--…)` defined |
| Unslop fences | depth = stack (`--surface-1/2/3`), no gradient text/buttons, no glows/text-shadow, no emoji icons, no pulse/ping/loops, no `transition-all`, no autoplay, reduced motion + forced colors honoured |
| Live state only, never fixture-as-real | every row carries provenance: `fixture: true` orders labelled as fixture; stale states carry `as of HH:MM`; no demo data in the shipped face |
| Own channels, no ctx in components | components get props only; reactive facts via inject `hooks`/declared store; no `useSyncExternalStore` in components |
| Panel content is read-only where the fleet is read-only | face never writes grants/policy; actions are explicit and audited |

## 3. Data path — the open engineering decision (G-D5, top risk)

Options for getting live fleet state into the client half:

- **P1 (recommended, supported): add a Typert-generated Remote surface** — a small server module in the fleet plugin declaring `@Remote` methods
  (`faceSnapshot()`, `facePanels()`), plus a stream (`mode:'stream'`) for live lanes/meters, and `ctx.remote.$on('fleet/changed')`. Client injects
  `remote.fleetFace` and calls generated methods. Needs: TypeScript build for that surface + the DSH client bundle pipeline (the fleet client is
  currently hand-bundled plain JS). Cost is real but it is the sanctioned channel and keeps the no-CORS posture.
- **P2 (not recommended): loopback JSON from the existing facade + CORS** — fetch `:8790` from the app origin, adding an allow-origin for the app
  and a browser token. Violates the "no CORS" posture of the facade, needs Lane 6 review, and duplicates a data surface.
- **P3 (fallback only): static snapshot at plugin load** — server writes a JSON snapshot the client bundle imports at load; stale between reloads;
  acceptable only for a first visual milestone, never as the shipped face.

**Recommendation: P1.** Until it exists, no panel may claim live data.

## 4. Panels and their state sources (all read-only unless named)

| Panel | Source of truth | Renders | Actions (phase 1: none or gated) |
|---|---|---|---|
| Fleet section (sidebar) | control `status` (integrity/rows/lanes_allowed/pid), `roster-get` lanes, `order-list` + `board-get` o1, `usage-feed`/ledger day-cap (meters), action-log chain head | lanes with role/model/state/last-seen; orders with state + age + block reason; meters (spend today vs cap, ledger events); integrity word + manifest sha short | none in phase 1; wake/steer come after A |
| Agent-activity view | **A comms registry** (`participants.list`): participant, kind, session, state `alive/busy/parked/stopped/stale`, current work (order/task), queue depth, last wake/prod | "which sessions are the agents', what they're working on, what's queued"; needs-you states in words + one-shot signal (no loops) | none; navigate to session |
| Phone-VM panel | local VM supervisor status (AVD `rig-phone`: state, adb transport, last action result, frozen v2 mobile comp URLs); frames streamed through P1 stream | booted/offline, transport ok, last round-trip result, live frame (owner sees it; **agents never read images**), actions used | phase 1: start/stop/status only, each audited |
| Windows-VM panel (monitor-level) | QEMU/RDP supervisor: pid, guest state, RDP protocol probe result (TPKT/X.224), session state, marker (`?step=done`), disk sizes; display stream (monitor-level) | installed/bootable/driveable-via-RDP words, marker timeline, live display thumbnail | phase 1: start/stop/status; no blind input |
| Studio instances | Team D contract §5: `fleet/fleet.json` + per-cell `status.json` (≤2 s), actions by name (`start/stop/cycle/focus/run`), refusals with reason | cells with state, pid/window title, MCP port (loopback), account label, play count, sealed-place match, owner-boundary enforced, last refusal | phase 1: actions proxied through Team D's launcher (post-build), all audited; never signals unregistered pid |

**Meters definition (so "meters" is not decoration):** today's spend vs ledger day cap, priced events count, per-role spend, and the hard-ceiling
remaining — all from the ledger/usage feed, with the accounting word `priced`/`unpriced` where true.

## 5. Delivery plan (component-wise, checker-gated; maker never checker)

1. **Face skeleton (no data):** `sidebar.panellist` entry + a `FleetPanel` with the section/card/list primitives, tokens only, empty-state wording,
   settings section upgraded from placeholder; visual self-check + token grep. Land with P1 skeleton stubbed (fixtures forbidden in shipped build).
2. **P1 data surface** (TS remote module + client inject) — this is the gating engineering item; small, proven with one `faceSnapshot()` call in a
   dev profile first (scratch DSH home, never the live app).
3. **Fleet section + meters** driven by live state (roster/orders/ledger); agent-activity view after A core exposes the registry.
4. **VM panels:** phone (existing rig supervisor → status file + frame stream), Windows (RDP monitor-level). Both need their supervisors to expose a
   stable status contract; no image reads by agents; renders go to the owner.
5. **Studio cells:** Team D's launcher + status contract behind the panel; owner-boundary refusal wording shown; containment proof before any real
   launch (Team D §7).
6. **One coordinated reload** of the owner's app to pick up the client bundle — owner-coordinated window, never a forced reload (B6); before that,
   all builds are verified in a scratch profile/HMR dev loop.
7. Owner grades the rendered visuals; face is never called done by us.

## 6. Gap report (named)

| ID | Gap | Severity | Close |
|---|---|---|---|
| G-D1 | The face must render **in the owner's app**, but that app runs the `desktop` profile while the fleet host is a separate `fleet` web profile — the client plugin lives in the fleet bundle, so it renders in the `fleet` GUI (:3092), not necessarily in the app he watches. Decision needed: load the fleet client into the app the owner uses vs present the fleet GUI | critical | Nox/Lead ruling + packaging (client module availability under the desktop profile or a same-app panel host) |
| G-D2 | No `sidebar.panellist` entry registered; `main['fleet']` inert | high | stage 1 of §5 |
| G-D3 | No live data channel (P1 not built); client bundle hand-bundled without Typert generation | critical | §3 P1; needs build-pipeline work |
| G-D4 | Client bundle update needs app reload (B6) | high | owner-coordinated window; HMR/dev loop in scratch profile first |
| G-D5 | Token pack mid-polish; pinned digest will change | medium | pin build digest; re-verify on re-freeze |
| G-D6 | VM panels have no stable supervisor status contract in the plugin's hands yet (rig state lives in shell/VM tooling) | high | define status JSON contract (same shape style as Team D §5) + supervisor process |
| G-D7 | Studio cells blocked on Team D launcher build + owner login acts (each cell's Studio login is owner-only) | high | Team D build + owner acts named |
| G-D8 | Empty/fixture honesty: no fixture data may render as real; `fixture:true` labelling must survive into the UI | high | provenance field in every row + face shows the word |
| G-D9 | No agent-activity data until A registry lands | medium | sequence after A core |
| G-D10 | Owner visual grade cannot be delegated; no image reads by agents | inherent | owner grades rendered visuals in his app |

**Named block B0 (shared with the index):** o1 — whose spec *is* this face — is `blocked` on the dead work tree + commit gate. The face build should
supersede o1's t1–t5 once Lead/Nox rule (a) writable git work tree visible to builders, re-staging tasks, or (b) per-lane artefacts accepted.
