# C4 — DSH CLIENT MODULE CONTRACT (one page) + MINIMAL EXAMPLE ROW

Team B (`fleet-lead`) deliverable for the C1–C6 freeze set, as ruled: **C1: c**, **C2: a, re-pinned to the
installed 0.3.0**, **C3: a (corrected)**, **C4: a + b interim**, **C5: a**, **C6: a**. Self-contained; the Lead
lands it in `origin/team`. If this contract is to live in the Ergon repo, proposed path (do not push from here):
`docs/CLIENT-CONTRACT.md` — a docs-only file, no package/runtime change.

## 1. What a client module is at the pin

A client plugin is a browser-side module loaded by the shell's module loader (`window.__ModuleLoader__.load`),
declaring exactly two exports and nothing else (`packages/client/AGENTS.md`, "Export discipline"):

| Export | Contract |
|---|---|
| `inject` | array of **declared** client services the module waits for before `apply`; the supported set is the shell's declared slots/services plus `locale` — **not** `plugin-manager` |
| `apply(ctx)` | registers dictionaries and slot contributions inside `ctx.effect(...)`; components never receive `ctx`, only props; no JSX transform (bundle as-is) |

Certified reference implementation (0.3.0, `package/src/client/client.js`): `inject = ["slots", "locale"]`;
registers `plugins.bundle.config` (settings page keyed by package name), `main` keyed `fleet` (inert placeholder),
and `shell.overlay` list entry `fleet-status-pill`. That is the real, packaged example the contract is drawn from.

## 2. Declared surfaces at the pin (the allowed contract surface)

| Surface | Kind / scope | Use |
|---|---|---|
| `main` | root-scoped **keyed** slot, one key per app view; shell occupant is `main[key=conversation]` | add **your own** `main[key]` view; **replacing** `main[key=conversation]`/`conversation.view` (`id=chat`) is a listed *replacement risk* and needs explicit arbitration — never assumed |
| `sidebar.panellist` | root-scoped **list**; sidebar sections | register one entry; it opens a `main[key]` view |
| `shell.overlay`, `shell.leading` | root-scoped lists | status chips / leading chrome |
| `plugins.bundle.config` | keyed settings contributions | plugin settings page/summary (`view: 'page' \| 'summary'`) |
| `new_session`, `more` | declared action slots | command/entry points |
| `root`, `sidebar` | declared shell slots | structural uses only, see the generated catalog |
| `locale` | version-matched client service | dictionaries: `ctx.locale.register(NS, { en })`; other locales fall back to `en` |
| `ctx.resources.register({protocol, open(address,{signal}) -> AsyncIterable<RemoteResult>})` + `useResource` | declared client **resource model** | live change streams (the right-sidebar tabs are its first consumer) |
| `ctx.connection` | host Fetch/Remote streams, Peer-authenticated | data from the host; there is **no** arbitrary router slot |

`cordis_inspect what:"client"` at the pin is the exhaustive per-key contract (cardinality, scope, owner,
replacement risk); when a needed surface is not listed above, name it from that catalog and get it confirmed
(C3 correction) — do not inject `plugin-manager` (its UI package is a template, not an injection).

## 3. The five rules (freeze text)

1. **Own key, never replace.** A product view registers a new `main[key]`; it never replaces
   `main[key=conversation]` or `conversation.view` (`id=chat`). Reshape needs written arbitration (C4 aetiology).
2. **Sidebar entry → main key.** A sidebar section is one `sidebar.panellist` entry that opens the module's
   `main[key]`; no other sidebar mechanism is a supported surface.
3. **Data over the declared streams.** Host data comes through exact Fetch routes / Remote streams via
   `ctx.connection`; pushes come through a plugin-owned host route + a `ctx.resources` provider read with
   `useResource`. No new global client service unless a *second, unrelated* plugin must consume it — then it is a
   framework extension point and needs main-thread arbitration.
4. **Service discipline.** A client plugin exports only `inject` and `apply`; components get props, not `ctx`;
   every registration sits inside `ctx.effect(...)` under its slot's declaration lifetime (it withdraws when the
   declaring plugin collapses — the unload cascade is exercised by the shipped 0.3.0 client).
5. **Version + platform pin (C1/C2/C5).** The contract is frozen at DSH `0.2.2-alpha.2` for W1 and **C2 is
   re-pinned to installed `@cayleb/dsh-fleet` 0.3.0** (installed + checker-proven: S4B ACCEPT 60/60 on installed
   output, S5-installed ACCEPT, Nox's live leg). A later pin bump is recorded as a version bump + re-run, never
   silent. Linux-only bundle; Windows surfaces are companions, not bundle rows. Private pinned tarball + sha256,
   `private: true` + UNLICENSED/internal licence, upstream notices kept where upstream code remains, no registry
   publish (C6).

## 4. Minimal example row (the one row an Ergon client module copies)

**Example row — a Fleet-style sidebar section opening its own main view.** Shape taken from the 0.3.0 client and
the Stage-5 sidebar recipe documented in `board/reports/team-b-contract-answers-C1-C6.md` §C3 follow-up:

| Row | `inject` | Contribution 1 | Contribution 2 | Opens |
|---|---|---|---|---|
| `sidebar.panellist` entry `key/id = <module>` | `["slots", "locale"]` | `ctx.slots.inject("sidebar.panellist", () => ctx.slots.register({ name: "sidebar.panellist", id: "<module>", order: <n>, label: () => t("panel") }, <EntryIcon?>))` | `ctx.slots.inject("main", () => ctx.slots.register({ name: "main", key: "<module>" }, <MainView>))` | the module's `main[key]` view |

```js
// One row, verbatim shape (0.3.0 certified client + Stage-5 recipe). Nothing else is needed.
const inject = ["slots", "locale"]
function apply(ctx) {
  ctx.effect(() => ctx.locale.register("ergon", { en: { panel: "Ergon" } }), "ergon:dict")
  ctx.slots.inject("sidebar.panellist", () => ctx.slots.register(
    { name: "sidebar.panellist", id: "ergon", order: 40, label: () => ctx.locale.t("ergon", "panel") }, ErgonEntry))
  ctx.slots.inject("main", () => ctx.slots.register({ name: "main", key: "ergon" }, ErgonMainView))
}
```

**Boundary:** this page records the supported contract; it changes nothing shipped, starts no build, and adds no
surface to the fleet package. Interim de-facto template until acceptance: the shipped 0.3.0 client module itself
(`package/src/client/client.js`) — this page supersedes it as the written contract, not as behaviour.
