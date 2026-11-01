# Job 03 W0 prep — the DSH plugin/bundle contract as it stands today (reference: `@cayleb/dsh-fleet` 0.2.1)

Prepared by Team A, 2026-10-31 ~22:25 ET, read-only. Purpose: de-risk W0 (“contract freeze”) of the Ergon-on-DSH package
plan by recording the DSH bundle contract as the only shipped reference implementation holds it, and listing what must be
frozen by Team B/Nox before Ergon package work starts. Sources read on disk (Team B's lane, read-only): 
`~/workspace/harness-build/dsh-fleet/package/package.json`, `package/cordis.patch.yml`,
`docs/BUILD-STATE.md` (§“Stage 1 interface contract”), `dist/cayleb-dsh-fleet-0.2.1.tgz` (sha256
`55516f9ae9535dc585da96587af45f55d83edac67c88e48d3bdc32ad652d10d4`, 164,714 bytes, built 31 Oct 10:46 ET).

## The contract (as the reference bundle declares it)

**A DSH bundle is an npm package that declares `package.json.dsh.bundle`.** From the reference:

| Field | Reference value | Meaning for Ergon |
|---|---|---|
| `name`, `version` | `@cayleb/dsh-fleet` 0.2.1 | Ergon package should be its own npm name/version (e.g. `@cayleb/ergon` 0.1.0) |
| `type` | `module` (ESM) | same |
| `exports` | `"."`, subpaths (`./core`, `./client`, …), `./cordis.patch.yml`, `./presets/*.patch.yml`, `./package.json` | Ergon must export its patch and any client entry; subpaths are optional |
| `bin` | `dsh-fleet` shim | optional; Ergon may ship a management CLI |
| `engines.dsh` | `0.2.2-alpha.2` | **must pin the same DSH version the package is built/tested against**; a different DSH is a contract break |
| `engines.node` | `>=26` | host node policy |
| `dsh.manifestVersion` | `1` | same |
| `dsh.bundle.patch` | array of YAML paths (`./cordis.patch.yml` + four preset patches) | the Cordis **entry-patch list**; each row inserts/replaces a bundle row by id |
| `dsh.bundle.platforms` | `["linux"]` | the reference ships host-side for Linux; the client half is web |
| `dsh.client` | `{ platform: 'web', inject: ['@deepseek-ai/dsh-client-locale', '@deepseek-ai/dsh-client-ui-plugin-manager'] }` | a browser half is a **client module with declared injection deps**; the Ergon front end (single session + sidebar) is this layer |
| `peerDependencies` | `@deepseek-ai/cordis ~4.0.6-alpha.1`, `@deepseek-ai/dsh-{fs,fs-local,llm,sandbox,tools} 0.2.2-alpha.2`, `@deepseek-ai/schemastery`, `undici 8.11.2` | the DSH packages the bundle is allowed to assume; **Ergon's plugin will need a similar peer set, and the plugin-manager/client packages if it contributes UI** |
| patch row shape | rows replace the target row's whole `config`; an id override cannot swap a row's package; one provider per service (sandbox/fs) | the Ergon patch must follow the same semantics; it can disable/replace stock rows but not vendor a different package under an existing id |
| install/pin | `dsh plugin add`, package bytes pinned by sha256; `rows.manifest.json` is a bundle-local audit artefact (the reference’s own check), not a DSH format requirement | Ergon should ship a pinned tarball + sha256 and an install/uninstall runbook; a rows manifest is optional unless Ergon adopts the same audit |

**What is not a DSH format:** there is no separate plugin manifest, no capability declaration, no `plugin.json`. The
bundle declaration + the Cordis patch list is the whole packaging unit (per the DSH docs quoted in
`dsh-fleet/docs/BUILD-STATE.md` §Stage 1 contract: “a bundle is an npm package that ships a configuration layer; its
manifest declares `dsh.bundle`”).


## FROZEN SET — Team B answers to C1–C6 (recorded 2026-10-31 22:35 ET; **PENDING Nox's confirmation**)

Source: `board/reports/team-b-contract-answers-C1-C6.md` (certified tree `cf93107` / tree `e15c17a3…` /
`@cayleb/dsh-fleet` 0.2.2; shipped 0.2.1 bundle `55516f9ae5…`). The Lead is routing these to Nox for the freeze
confirmation; until then this is the frozen set **pending confirmation**, not a ruling. No build starts from it.

| Item | Frozen answer (pending Nox) | Enumerated detail / condition |
|---|---|---|
| **C1** contract freeze | **(c)** freeze at DSH `0.2.2-alpha.2` for W1 + one recorded contract-delta review when 0.2.2 lands | 0.2.2 is **not** version-string only: additive delta = exports `./policy` (+`src/policy/*`) and `./ledger` (+`src/ledger/*`) in `files`; `rows.manifest.json` +15; `cordis.patch.yml` +29/−3 (fleet-policy/fleet-ledger rows); `engines.dsh` unchanged (alpha.2); no removals. This enumerated delta **is** the 0.2.2 review item. |
| **C2** DSH pin | **(a)** pin `0.2.2-alpha.2` now; re-pin only after the 0.2.2 release is installed + checker-proven, as a recorded version bump with a re-run | Condition **not met today**: 0.2.2 not installed (row 18 blocked on owner Wait; `fleet-dsh` still 0.2.1); the 0.2.2 package still declares `engines.dsh: 0.2.2-alpha.2`. Re-pin is a **named future event**, not now. |
| **C3** client injections | **(a, corrected)** supported set = the shell's **declared slots + `locale`**; Ergon's requested slots confirmed (see C3 follow-up below) | Both client packages are present/version-matched at the pin; the reference bundle injects `["slots","locale"]`, **not** plugin-manager (it is a template, not an injection). **C3 follow-up (Team B, later 31 Oct):** shell-declared slots = `root`, `main`, `sidebar`, `sidebar.panellist`, `shell.overlay`, `shell.leading`, `plugins.bundle.config`, `new_session`, `more`, `dsh.plugin-manager.install-registry`; the generated client inspect catalog is the exhaustive per-key contract. Ergon mapping: `main` is keyed (occupant `main[key=conversation]`); Ergon **adds its own `main[key]` view** and contributes into the declared conversation-domain slots (`conversation.header*`, `conversation.session.header.*`, `conversation.composer*`, `conversation.input.*`) for the one-session shape; **replacing `main[key=conversation]`/`chat` is a listed replacement risk, not a documented support surface — it is a separate named item needing Nox arbitration and is NOT in W1 unless ruled (see C3 follow-up below)**; a sidebar section is a `sidebar.panellist` entry opening a `main[key]` view; app views are keyed `main` keys (no separate router slot; data via host Fetch/Remote streams through `ctx.connection`); the live knowledge-graph view uses the **declared resource model** — plugin-owned host fetch/stream route + `ctx.resources.register({protocol, open(address,{signal}) -> AsyncIterable<RemoteResult>})` read via `useResource` (a global client service would be a new framework extension point needing main-thread arbitration, only if a second plugin must consume it). Plugin-manager is not injected. |
| **C4** web-client contract | **(a)+(b interim)** Team B writes the one-page client contract + one minimal example row; plugin-manager is the interim de-facto template | The certified tree already carries a real packaged example: `main[fleet]` page, `shell.overlay` pill, `plugins.bundle.config` settings contribution (sidebar recipe = Stage 5). It **lands with the operator-path/dashboard work, not as a build now**; no contract change implied. |
| **C5** platform scope | **(a)** Linux-only bundle (`platforms: ["linux"]`) + separately installed Windows companions | The fleet's shipped remote-control panels can drive an Android emulator / Windows VM from inside the Linux bundle, but that does **not** make it a Windows artefact; Ergon's Get Help window / native ReviewDesk / WPF setup stay companions. |
| **C6** naming/licensing/distribution | **(a)** private pinned tarball + sha256, `private: true` + UNLICENSED/internal licence, keep upstream MIT notices where upstream code remains, no public publish | Fleet pattern: `license: UNLICENSED`, installed via `dsh plugin add <sha256> <file>`; fleet vendors no OpenClaw code (no OpenClaw notice obligation from the fleet), Ergon keeps its own upstream notices where its code remains. Any licensing/publication change remains Cayleb-via-Nox. |

### C3 follow-up — slots and resource model, and the reshape constraint (Team B answers + Lead framing, 31 Oct)

Basis: Team B's answer (pinned checkout `0.2.2-alpha.2`; `docs/subsystems/slots.md`, `client-resources.md`,
`packages/client/AGENTS.md`, shell sources) and the Lead's framing of the one constraint that matters. Records:
`board/reports/team-b-contract-answers-C1-C6.md` §"C3 follow-up".

**Declared shell slots:** `root`, `main`, `sidebar`, `sidebar.panellist`, `shell.overlay`, `shell.leading`,
`plugins.bundle.config`, `new_session`, `more`, `dsh.plugin-manager.install-registry`; the generated client inspect catalog
is the exhaustive per-key contract. `main` is keyed; its stock occupant is `main[key=conversation]`/`chat`.

**Ergon W1 surface (ADD-only — what the plan assumes unless Nox rules otherwise):**
1. Ergon **adds its own keyed `main[key]` workspace view** (not a replacement of the stock conversation view), opened from
   a **`sidebar.panellist`** entry.
2. Sidebar panels and app views are further `sidebar.panellist` entries / keyed `main` keys; app data comes from host
   `Fetch`/`Remote` streams via `ctx.connection` (no separate arbitrary router slot).
3. The **live knowledge-graph view** uses the **declared client resource model**: plugin-owned host fetch/stream route +
   `ctx.resources.register({protocol, open(address,{signal}) -> AsyncIterable<RemoteResult>})`, read with the standard
   `useResource` hook. **Not** a new global service (that would be a framework extension point needing main-thread
   arbitration, only if a second unrelated plugin must consume the stream).
4. No plugin-manager injection; Team B tree unchanged.

**The constraint — reshaping the main conversation view is a separate named item, NOT a W1 assumption.**
Replacing `main[key=conversation]` / the chat view is **not a documented support surface**: the generated client catalog
lists it as **replacement risk**, and it needs explicit arbitration. The Lead has escalated this to Nox with three options:

| Option | Meaning | Effect on the plan |
|---|---|---|
| **(a) ADD-only beside stock chat for W1** | W1 ships Ergon's own `main[key]` workspace view + sidebar panel(s) + graph; stock chat stays as-is | **Default planning path** if Nox has not ruled by W1 start; no arbitration needed; the one-session reshaping is a later named delta |
| **(b) arbitrate a documented reshape/replacement of the main conversation view** | needs the **C4 client-contract page to cover it**; if covered at W2, W1 can plan the reshape as part of the client rows | **Preferred IF the C4 page covers it by W2** (Lead's recommendation, and ours): plan path “W1 + reshape”, with the C4 page as a hard dependency |
| **(c) defer the reshape to phase 2** | ship stock chat + sidebar + graph now; reshape becomes a phase-2 delta with no date claim | Fallback if (b)'s C4 coverage cannot land; same W1 deliverable as (a) plus a recorded phase-2 item |

**Planning consequence (both paths, so the ruling only changes the W1 client-row scope):**
- Default plan row set: `slots` + `locale` injection, Ergon `main[key]` view, `sidebar.panellist` entry/panels, resource
  provider + `useResource` graph view. This is the same under (a) and (c).
- If Nox rules **(b)** and the C4 page covers conversation-domain replacement by W2: add the reshape rows (targeting
  `main[key=conversation]`/`chat` and the conversation-domain slots) to the W1 client row set, with the C4 page recorded as
  the blocking dependency; the rest of W1 is unchanged.
- If Nox rules differently or not at all: W1 proceeds on the ADD-only row set; the reshape stays a named delta (phase 2 or
  an arbitrated later change), never silently assumed.

## Mapping the three product pieces onto the contract

- **Piece 2, DSH baseline:** the pinned `@deepseek-ai/dsh` 0.2.2-alpha.2 (the fleet install already has this on the real
  install; the fleet profile is the proof the baseline is installable).
- **Piece 3, Ergon plugin:** an npm bundle like the reference: `dsh.bundle.patch` inserting (a) the web client module that
  reshapes the front end into one long-running session with the sidebar (apps + live knowledge-graph view), and (b) the
  host-side rows the plugin needs (the agent session/home wiring, app registry routes). It is what makes the baseline into
  Ergon; Ergon Lite is exactly pieces 2+3. **Corrected and confirmed contract facts for this piece (Team B C1/C3):**
  the client module injects the **shell's declared slots + `locale`** (not plugin-manager; it is a template only). The
  supported front-end shape is Ergon's **own keyed `main` view** plus contributions into the declared conversation-domain
  slots for the one-session experience; a **`sidebar.panellist` entry** opens the main view; each app view is another keyed
  `main` key (no separate router slot; data over host Fetch/Remote streams via `ctx.connection`); the live knowledge-graph
  view is a plugin-owned host fetch/stream route registered through the **declared resource model** and read with
  `useResource`. **Replacing the stock `main[key=conversation]`/chat is not a documented support surface** (a listed
  replacement risk) — that needs explicit arbitration and is not assumed by this plan. The package targets the **alpha.2
  contract**; the 0.2.2 additive delta (`./policy`, `./ledger`, fleet-policy/ledger rows) is the recorded contract-delta
  review item at 0.2.2 acceptance, not a freeze break.
- **Piece 1, management software:** a separate app on the company host (control service: tickets/devices/budget proxy,
  ReviewDesk console, ticket queue) — **not** a DSH bundle. It is the piece Ergon Lite omits. The package installer must
  install/deploy it alongside the DSH profile and wire the plugin to it (HTTP over the host's shareable URL).
- **Installer/front door:** “Ergon IS the installer” — the package’s install path is: stock host → install DSH baseline →
  `dsh plugin add` Ergon bundle (pinned sha256) → deploy management piece → first-run company setup. The Ergon installer
  should not require a developer shell. **Platform/licensing (C5/C6):** the bundle declares Linux (`platforms: ["linux"]`)
  and the Windows surfaces install as companions; the package ships as a `private: true`, UNLICENSED/internal-licence,
  pinned tarball + sha256 (no registry publish), keeping upstream notices where Ergon’s upstream code remains.

## Team B closure (2026-10-31 ~22:45 ET)

Team B has **closed C1–C6 + the C3 follow-up on its side**: `board/reports/team-b-contract-answers-C1-C6.md` records the
frozen set pending Nox's confirmation and now notes Team A's adoption (own keyed `main` view + declared conversation-domain
contributions; **no assumption** of replacing `main[key=conversation]`/chat — replacement risk, arbitration; `sidebar.panellist`
entries; keyed `main` app views; host Fetch/Remote streams via `ctx.connection`; declared resource model for the knowledge
graph, no new global service; no plugin-manager injection). **C4** is recorded as a named future deliverable (one-page
client contract + one minimal example row, with the operator-path/dashboard work) for Team A to record as the documented
dependency. Nothing further is needed from Team B on C1–C6 unless Nox rules a freeze item differently; no build, no change
in Team B's tree.

## What must be frozen before W1 (open items for Team B/Nox; not assumptions)

1. **Contract version owner:** Team B owns the DSH plugin contract (Job 06/08); confirm the bundle/row contract is frozen
   at DSH 0.2.2-alpha.2 and that no packaging change is planned for release 0.2.2 (Job 06) that would move it.
2. **DSH pin:** which exact DSH build the Ergon package pins (`0.2.2-alpha.2` vs the 0.2.2 release once Job 06 closes).
3. **Client-injection availability:** whether `@deepseek-ai/dsh-client-locale` and the plugin-manager client package (the
   reference’s injections) are available/version-matched for an Ergon client module; the Ergon front end needs the client
   host contract more than the fleet does.
4. **Web client contract:** the reference injection list is the only shipped example; Team B should confirm the client
   module API (what a bundle may render/replace in the web shell) is stable enough for the Ergon sidebar/session reshape.
5. **Platform:** reference `platforms: ["linux"]`; the management piece and the Windows surfaces (Get Help, native
   ReviewDesk) are outside the DSH bundle. Confirm whether Ergon needs a Windows bundle row set or keeps Windows pieces as
   separately installed companions.
6. **Naming/licensing:** package name (`@cayleb/ergon`?), license policy for a company-facing package, and whether the
   OpenClaw attribution files stay (they are in the current repo).

## What Team A does next in W0 once ownership is confirmed

1. Ask Team B/Nox for the six items above in one message (batching, no drip-feed).
2. Record the frozen contract version + peer set in the Ergon package README.
3. Only then start W1 (plugin skeleton in a clean DSH profile). No build before the ownership ruling (Lead’s order).

## Corrections after Team B's C1–C6 answers (2026-10-31 22:30 ET)

Answered in `board/reports/team-b-contract-answers-C1-C6.md`; recorded in `team-a-job03-contract-items.md`:

1. **Client injections (corrects this note):** the reference *client module* declares `inject = ["slots","locale"]`, not
   plugin-manager. The package-level `dsh.client.inject` list (locale + plugin-manager) is about injected client packages;
   the supported set for an Ergon client module is **the shell's declared slots + `locale`**, with plugin-manager as a
   template (available, version-matched, but not an injection Ergon should assume).
2. **0.2.2 is not version-string only:** it adds exports `./policy`/`./ledger`, `src/policy/*` + `src/ledger/*`, rows.manifest
   +15 and cordis.patch +29/−3 (fleet-policy/ledger rows); engines.dsh unchanged, no removals. This is the enumerable
   contract-delta to review at 0.2.2 acceptance (C1c), not a freeze break.
3. **Pin condition:** C2a accepted but the re-pin condition is not met (0.2.2 not installed; row 18 owner-wait; fleet-dsh
   still 0.2.1) — Ergon stays pinned to `0.2.2-alpha.2` for W1.
