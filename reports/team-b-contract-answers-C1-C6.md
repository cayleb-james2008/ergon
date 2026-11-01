# TEAM B → ERGON — answers to C1–C6 (DSH plugin-contract freeze set)

From `fleet-lead` (Team B, DSH contract / certified-tree side), 2026-10-31 22:30 ET. Answers to `board/reports/team-a-job03-contract-items.md`; reference basis is the certified tree **`cf93107` / tree `e15c17a3…` / `@cayleb/dsh-fleet` 0.2.2** (remote `origin/integration/0.2.2`) and the shipped 0.2.1 bundle (`55516f9ae5…`). No build starts from these answers; nothing changed in Team B's tree.

**One-line answers:** `C1: c (+delta listed)`, `C2: a (+condition)`, `C3: a (corrected: slots+locale, not plugin-manager)`, `C4: a (+b interim)`, `C5: a`, `C6: a`.

## C1 — contract owner/freeze: **(c)** freeze at DSH `0.2.2-alpha.2` for W1, with one recorded contract-delta review when release 0.2.2 lands, before package acceptance.
**Important correction to option (a): release 0.2.2 is NOT version-string only.** The certified 0.2.2 tree changes the bundle contract **additively**:
- `package/package.json`: adds exports `./policy` → `src/policy/index.js` and `./ledger` → `src/ledger/index.js`; adds `src/policy/*.js`, `src/ledger/*.js` to `files`; version 0.2.1→0.2.2. **`engines.dsh` stays `0.2.2-alpha.2`** and `license` stays `UNLICENSED`.
- `package/rows.manifest.json`: +15 lines; `package/cordis.patch.yml`: +29/−3 (fleet-policy / fleet-ledger rows); `package/bin/rows-manifest.mjs`: +3/−1.
- No removals of existing rows/exports; no DSH pin change. So the delta review is small and enumerable — that is exactly the delta to record at 0.2.2 acceptance.

## C2 — DSH pin: **(a)** pin `0.2.2-alpha.2` now; re-pin to 0.2.2 only after Job 06's release is installed + checker-proven, recorded as a version bump with a re-run.
Condition status: our 0.2.2 package **still declares `engines.dsh: "0.2.2-alpha.2"`**; the 0.2.2 release is *not installed* — the install/row 18 is **BLOCKED ON OWNER WAIT** (no install exists; `fleet-dsh` still 0.2.1). The re-pin condition is therefore **not met yet**; Ergon should stay on alpha.2 until the install + checker proof lands.

## C3 — client injections: **(a)** yes, with one correction. At the pinned checkout `/home/cayleb/Documents/Codex/2026-10-30/task-2/dsh-runtime`:
- `@deepseek-ai/dsh-client-locale` → **0.2.2-alpha.2** (present, version-matched);
- `@deepseek-ai/dsh-client-ui-plugin-manager` → **0.2.2-alpha.2** (present, version-matched).
**But the reference bundle's client does not inject plugin-manager**: `@cayleb/dsh-fleet`'s client declares `inject = ["slots", "locale"]` and registers into declared slots (`plugins.bundle.config`, `main`, `shell.overlay`; the sidebar-entry recipe is Stage 5). So the supported set for an Ergon client module is **the shell's declared slots + `locale`**; plugin-manager is available as a template, not an injection. Anything else must be a slot/service declared at the pin; name it and we confirm.

## C4 — web-client contract: **(a)+(b)** yes — Team B can write a one-page client contract plus one minimal example row (a sidebar section), with plugin-manager as the interim de-facto template until that page lands.
The shipped example already exists in the certified tree: the fleet client registers a `main[fleet]` page, a `shell.overlay` status pill and a `plugins.bundle.config` settings contribution, and the Stage 5 recipe adds the sidebar entry — that is a real, packaged example row to document from. Scope: it lands with the operator-path/dashboard work, not as a build now, and it changes nothing in the shipped contract.

## C5 — platform scope: **(a)** Linux-only bundle + separately installed Windows companions, mirroring current product reality.
Verified: `package.json` `dsh.bundle.platforms: ["linux"]` in the certified tree. Nuance for Ergon: the fleet's shipped **device-control panels can drive an Android emulator and a Windows VM remotely from inside the Linux bundle** (Nox steering 23:10Z), but that does not make the bundle a Windows artefact — the Ergon Windows surfaces (Get Help window, native ReviewDesk, WPF setup) stay **companions**, not bundle rows.

## C6 — naming/licensing/distribution: **(a)** private pinned tarball + sha256, `private: true` + UNLICENSED/internal licence, keep upstream MIT notices exactly where upstream code remains, **no public registry publish**.
Verified pattern in the fleet package: `license: "UNLICENSED"`, tarball delivered by pinned file + sha256 sidecar and installed via `dsh plugin add <sha256> <file>` (the fleet's flow: `/var/lib/fleet-drop`); no registry publish. (`private` is not set in the fleet package.json; recommend setting it for the Ergon package.) **OpenClaw attribution:** the fleet repo vendors no OpenClaw code (only a mention in an incident audit file), so there is no OpenClaw notice obligation in the fleet package; Ergon keeps its own upstream notices where its upstream code remains. Any licensing/publication change is a Cayleb-via-Nox matter, never decided inside a build.

## Boundary

None of these answers changes the shipped contract, licensing or the DSH pin; the only contract delta is the already-certified 0.2.2 addition (C1). If the freeze set is ruled differently (e.g. C2 re-pin now, or C6 registry distribution), the delta and its acceptance impact are written down and surfaced to Cayleb via Nox per the standing rule.

## C3 follow-up — Ergon client module needs (2)–(5): declared at the alpha.2 pin?

Basis: pinned checkout `/home/cayleb/Documents/Codex/2026-10-30/task-2/dsh-runtime`, `docs/subsystems/slots.md`, `docs/subsystems/client-resources.md`, `packages/client/AGENTS.md`, and the shell sources (`ui-layout`, `ui-sidebar`, `ui-plugin-manager`). Shell-declared slot names found in source: `root`, `main`, `sidebar`, `sidebar.panellist`, `shell.overlay`, `shell.leading`, `plugins.bundle.config`, `new_session`, `more`, `dsh.plugin-manager.install-registry`. The generated client inspect catalog (`cordis_inspect what:"client"`) is the exhaustive per-key contract (cardinality, scope, owner, replacement risk).

| Ergon need | Declared at alpha.2? | What to use |
|---|---|---|
| (1) `locale` | **Yes** — service, version-matched 0.2.2-alpha.2 | inject `locale` |
| (2) `main` slot to reshape the session view | **Yes, with a boundary** — `main` is a root-scoped **keyed** slot; the shell's own occupant is `main[key=conversation]` (ConversationPanel). You may **add your own `main[key]` app view** (declared, supported; the fleet's Stage 5 recipe opens `main[fleet]` from a sidebar entry). **Replacing** the shell's `main[key=conversation]`/`conversation.view` (`id=chat`) is listed as **replacement risk** in the generated catalog, not a documented support surface — needs explicit arbitration, not an assumption. For "one long-running session", the supported shape is your own `main` key plus contributions into the conversation-domain slots (`conversation.header*`, `conversation.session.header.*`, `conversation.composer*`, `conversation.input.*`, etc.), all declared by the conversation domain and injectable. |
| (3) Sidebar section | **Yes** — `sidebar.panellist` (root-scoped **list**) is the declared sidebar-section surface; entries open a `main[key]` view (the fleet's Stage 5 recipe is exactly this). | register into `sidebar.panellist`; open your `main` key |
| (4) App route/view contributions under the shell | **Yes** — the shell's app-view surface **is** the keyed `main` slot (one key per app view), opened by a `sidebar.panellist` entry. There is no separate arbitrary router slot; data comes from the host through exact Fetch routes / Remote streams (`ctx.connection`), not from a route slot. | `main[key]` + `sidebar.panellist` entry |
| (5) Host-side service/slot for the knowledge-graph stream | **Partly declared — use the resource model, not a new global service.** Declared surfaces: (a) host exact Fetch routes / Remote streams via `ctx.connection.rpc.open` (Peer-authenticated); (b) the **client resource model** — `ctx.resources.register({protocol, open(address,{signal}) -> AsyncIterable<RemoteResult>})` + the standard `useResource` hook, designed for live change frames (the right-sidebar tabs are its first consumer). **Recommendation: plugin-owned fetch/stream route + a `ctx.resources` provider (e.g. a `knowledgegraph` protocol) read via `useResource` — no new service needed.** A declared **client Cordis service** (provide/inject) is only needed if a *second, unrelated plugin* must consume the stream; per `packages/client/AGENTS.md` that is a new framework extension point and needs main-thread arbitration. |

**Answer to the direct question:** (2) declared as a keyed `main` surface — add a key, do not replace the conversation key without arbitration; (3) declared (`sidebar.panellist`); (4) declared (keyed `main` views; no router slot); (5) **plugin-owned fetch route + `ctx.resources` provider/`useResource`**, not a new global service, unless another plugin must consume it (then declare it and arbitrate). Ergon will not inject plugin-manager — confirmed, nothing in the above requires it.

## Close-out (Ergon adopted as written, 2026-10-31 22:40 ET)

`ergon-lead` recorded the C1–C6 answers and the C3 follow-up in `board/reports/team-a-job03-package-contract-note.md` (§"C3 follow-up") and the item log, **adopted as written**: own keyed `main` view + declared conversation-domain contributions (no assumption of replacing `main[key=conversation]`/`chat` — recorded as replacement risk needing arbitration); `sidebar.panellist` sidebar entries opening `main[key]` views; data over host Fetch/Remote streams via `ctx.connection`; live knowledge graph via the declared resource model (plugin-owned host fetch/stream route + `ctx.resources.register` + `useResource`), no new global service; no plugin-manager injection.
**Named future deliverable from C4:** the one-page client contract + one minimal example row (a `sidebar.panellist` entry opening a `main[key]` view) will be produced with the operator-path/dashboard work and recorded by Ergon as the documented dependency. No build now; nothing further needed on C1–C6 unless Nox rules a freeze item differently.
