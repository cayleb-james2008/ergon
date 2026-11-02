# TEAM D — CONTAINED STUDIO INSTANCES (plugin sidebar), design v1 — 2026-11-02

Mission: Nox "FINISH THE PLUGIN" (123400Z), item 1. Owner of this layer: Team D (**instance/rig layer only**); Team B
builds the sidebar UI against the contract in §5. **Design phase only — no build, no Play, no image reads, $0**, mission care
fence binds. Next: checker review of this design → build → containment proof.

## 0 · One-paragraph shape

Fleet Studio = **N isolated "studio cells"** under one fleet root. Each cell has its own **writable Vinegar data root**
(so its own wine prefix → its own Roblox Studio login/settings/logs), its own **place working copy** (distinct filename →
distinct window title and a distinct Studio process), its own **loopback MCP port**, its own **cache/plays/latch/state**
and its own **agent-team namespace**. The owner's personal Studio is never opened, targeted or signalled by the launcher:
actions are addressed by **registry (instance id + pid + window id + place hash)**, and anything outside that registry is
**refused, not best-effort**. A cell can crash without touching another cell or the owner; recovery is the existing
`studiocycle` recipe, per cell.

## 1 · Measured rig facts this design builds on (outis-desktop, 2026-11-02)

| Fact | Value |
|---|---|
| Studio runtime | Flatpak `org.vinegarhq.Vinegar` 1.9.4, Wine 11.0, Studio `0.745.0.7451137` |
| Process shape | `bwrap → vinegar Z:\…\data\usg\usg-desktop.rbxlx → wineserver → RobloxStudioBeta.exe <place> → RobloxCrashHandler.exe`; `FLATPAK_ID=org.vinegarhq.Vinegar`, `HOME=/home/cayleb` inside the sandbox |
| Vinegar data root | `~/.var/app/org.vinegarhq.Vinegar/data/vinegar/{appdata,versions,prefixes/studio,kombucha*,settings.reg}`; **`prefixes/studio` = 2.2 GB**, single prefix today |
| Sealed place | `build/test.rbxlx` == `~/.var/app/…/data/usg/usg-desktop.rbxlx` (sha `c6651793…`); release `84324d43…`; the place path is a **command-line argument** to Studio |
| Tooling today | `winctl` (Hyprland window ops; builtin target = `class ^steam_proton$`, title `usg-desktop\.rbxlx`; configurable via `WINCTL_CONFIG`), `studiocycle` (per-window close/reopen/fit, sealed-hash guarded, discard autosaves), `vinegar_run.py` + `vinegar_lib/checks.py` (**hardcoded** `~/.cache/usg`, `prefixes/studio`, `usg-desktop.rbxlx`, `MCP_PORT=13469`, plays counter, latch) |
| Vinegar capabilities found in the shipped binary | strings include **`XDG_DATA_HOME`**, `launching new studio instance`, `delete-prefix`, `delete-studio` — the Go launcher honours `XDG_DATA_HOME`, which is the likely multi-cell switch |
| MCP | Master Claude's Studio MCP is loopback HTTP (today 13469); one server per Studio owner; currently no separate listener when idle |

## 2 · Isolation architecture (the "studio cell")

Fleet root (outside the repo, owner-writable): `~/.var/app/org.vinegarhq.Vinegar/data/fleet/` — visible/writable inside the
flatpak sandbox (the app-data dir is already mounted), so no extra `--filesystem` grants are needed.

```
fleet/
  fleet.json                  # schema 1, per-cell registry, capacity, owner-boundary config
  cells/<slug>/               # one contained Studio instance
    cell.json                 # id,label,slug,createdAt,accountLabel,placeSha256,sealSha256,windowTitle,mcpPort,cellState
    place/<slug>.rbxlx        # working copy, copied from the sealed release; per-cell hash recorded
    data/                     # XDG_DATA_HOME for this cell  -> vinegar/{prefixes/studio,settings.reg,...}
      vinegar/versions -> ../../../../vinegar/versions  (read-only symlink to the shared Studio install)
      vinegar/kombucha* -> ../../../../vinegar/kombucha* (read-only symlink to the shared Wine build)
      vinegar/settings.reg  (template copy; per-cell edits are allowed and isolated)
    cache/usg/                # plays counter, latch, run state (per-cell)
    state/{status.json,actions.jsonl,crashes.jsonl,locks/}
  shared-template/            # prefix template + place template used to create cells (never mutated by a cell)
```

**Launch recipe (per cell):**

```
flatpak run --env=XDG_DATA_HOME=<cell>/data \
            --env=DSH_FLEET_CELL=<slug> --env=DSH_FLEET_ROOT=<fleet> \
            org.vinegarhq.Vinegar '<cell>/place/<slug>.rbxlx'
```

- **Own login:** the writable per-cell data root gives each cell its own `prefixes/studio` → its own Roblox Studio auth
  store, settings, logs, autosaves. Shared `versions/` and `kombucha/` stay read-only symlinks (no 200 MB re-download, no
  duplicated Wine runtime); a cell's prefix is created from the template (reflink copy where the filesystem supports it,
  otherwise `cp -a`; 2.2 GB/cell — capacity check required, §8).
- **Own Studio connection:** each cell has its own Studio process, its own play counter file and its own MCP port; MCP
  allocation is a registry-owned **loopback-only port range** (e.g. 13470–13499), never 13469 (the owner's port), verified
  with `ss` as `127.0.0.1:<port>`.
- **Own game/place:** the working copy is a byte copy of the sealed release (`sha256` recorded in `cell.json`); the
  canonical seal (`build/release.rbxlx`) is **never** written by the fleet and its hash is re-checked after every action.
  The place filename is the cell slug → Studio's window title contains it (`<slug>.rbxlx`) → addressable per cell.
- **Own agent team:** the cell's `course`/run workspace is the instance id namespace; `vinegar_run.py` gets an
  **instance-state contract** (`USG_CELL_ROOT`, `USG_CACHE`, `USG_PLACE`, `USG_MCP_PORT`, `USG_WINCTL_TARGET`) instead of
  the hardcoded owner defaults; each cell has one lock and one Play at a time; a global scheduler admits at most one Play
  across all cells at a time (GPU/Wine stability — see §8).
- **Window addressing:** per-cell `WINCTL_CONFIG` target = `{class: ^steam_proton$, title: ^<escaped slug>\.rbxlx$}`;
  `winctl` must additionally match the registered **pid** and refuse if 0 or >1 windows match. The owner target
  (`usg-desktop.rbxlx`) is in a separate config and is never loaded by the fleet launcher.

## 3 · Cell lifecycle & state machine

`new → provisioning (template copy) → stopped → starting (studio) → ready(edit) → playing (run) → stopping → stopped`;
error states `degraded` (MCP/window missing, retryable) and `quarantined` (crash budget exhausted). All transitions append
to `state/actions.jsonl` (who/what/when/prev/new + evidence paths); `cell.json` is written atomically (rename) and its
sha256 is logged. Status is **read-only** for Team B

Every action class is explicit and deny-by-default: `provision`, `start`, `stop`, `cycle`, `focus`, `run` (Play),
`status`, `destroy`. Each action first (re)validates the **cell identity tuple** (cell id, slug, place hash, pid, window id,
MCP port) and the **owner boundary** (§6); a mismatch is a named refusal, never a retry on a different target.

## 4 · Crash containment & recovery

- Supervisor (launcher process) watches per cell: Studio pid liveness, window presence, MCP health (`tools/call` probe),
  Edit/Play state, plays counter, place hash, crash-dump directory (`<cell>/data/vinegar/prefixes/studio/…/logs/crashes/reports`).
- On crash / MCP loss: mark cell `degraded`, record dump path+size+crash class in `crashes.jsonl`, run the per-cell
  `studiocycle` recipe (close if a stale window exists **and it is this cell's registered window**, discard autosaves,
  reopen the cell place, fit 2200×1000), re-validate place hash, then `ready`.
- Retry budget: 2 automatic recoveries per cell per hour, then `quarantined` (no auto start; surfaced to the sidebar with
  the crash evidence). Crash containment is structural: window kill is by **registered window id/pid**, place restore is by
  **cell place hash**, so no other cell and never the owner can be hit.
- Known crash class to expect and report honestly: intermittent userland Wine Studio shutdown minidumps (4 seen in the
  atlas chain; sealed unchanged each time) — the supervisor must classify `crash, post-runtime, dump-size, no kernel GPU
  evidence` and not loop.

## 5 · Sidebar contract (Team B builds UI; Team D owns the data)

Read-only status file, updated ≤2 s: `fleet/fleet.json` (cells, capacity, owner-boundary state) and per-cell
`state/status.json`:

```
{ "schema":1, "cell":"<slug>", "state":"ready|playing|degraded|quarantined|stopped|provisioning",
  "pid":…, "window":{"id":…,"title":…,"size":[w,h]}, "mcp":{"port":…,"loopback":true,"healthy":true},
  "accountLabel":"…", "place":{"path":"…","sha256":"…","matchesSeal":true},
  "plays":{"used":n,"max":3}, "lastAction":"…","lastError":null,
  "crashes":{"count":n,"last":{"at":"…","dump":"…","size":…,"class":"wine-shutdown"}},
  "updatedAt":"…" }
```

Actions are files/commands with the same names as §3 (`start`, `stop`, `cycle`, `focus`, `run`); the launcher returns
`{ok:true|false, refusal, reason, evidence}`. Team B never parses logs; it renders state + last refusal reason.

## 6 · Hard boundary vs the owner's personal Studio (detection + refusal)

A Studio window/process is the **owner's** unless it matches a registered cell tuple. The launcher:
1. enumerates windows via `winctl`/Hyprland and processes via `/proc`; classifies owner vs cell;
2. **refuses** any action whose target is not an exact registered tuple (pid + window id + title `^<slug>\.rbxlx$` +
   place path under `fleet/cells/<slug>/place/` + env `DSH_FLEET_CELL=<slug>`);
3. **refuses** to inherit the owner's defaults: `usg-desktop.rbxlx`, `prefixes/studio` (owner), `MCP_PORT 13469`,
   `~/.cache/usg` owner cache, default winctl target — any of these appearing as a cell target is a **named hard error**;
4. treats **ambiguous or duplicate** matches (same pid, same title, two windows) as refusal, never "pick one";
5. never kills by name (`pkill RobloxStudioBeta`), never signals an unregistered pid, never closes a window it did not
   start, never writes the owner place or the sealed `build/` files;
6. logs every refusal with the evidence tuple; the sidebar shows `ownerBoundary: enforced` plus the last refusal.
A guard test suite (fake window table + fake `/proc`) must prove each refusal rule and must be green before any real
launch is allowed.

## 7 · Containment proof plan (fresh checker, no image reads)

Static/offline:
1. **Unit tests with fakes**: two cells (fake procfs, fake Hyprland client table, fake XDG tree) — provisioning paths,
   distinct data roots/prefix/place/port; status schema; atomic writes; lock contention; action/refusal matrix.
2. **Owner-boundary battery**: owner window present in the fake table; every action on it refused with the named reason;
   duplicate/ambiguous match refused; attempts to reuse owner place/port/cache refused; `pkill`-style API absent from the
   launcher (grep/pin).
3. **Recovery simulation**: kill the fake cell pid → supervisor marks degraded, invokes cycle once on the registered
   window only, revalidates place hash, `ready`; owner pid untouched; crash budget → quarantined.
4. **Real-host, no images** (build phase, two disposable cells): `ps -eo pid,args,environ` shows distinct
   `XDG_DATA_HOME`/`DSH_FLEET_CELL`/place per cell; `ss -ltn` shows the MCP port(s) bound to **127.0.0.1 only**; each cell's
   prefix/place/cache hashes recorded and different; `winctl` lists one owner window + N cell windows with distinct titles;
   `sha256sum build/release.rbxlx build/test.rbxlx` unchanged; owner Studio pid/uptime/window id unchanged throughout.
5. **Cross-checks**: a cell Play via the runner runs on the cell place (place hash bound), plays counter is per-cell, and
   the owner cache/latch files are untouched (hash before/after). Sealed artifacts never written (`find -newer` check).

## 8 · Top risks & open questions

1. **Isolation depends on `XDG_DATA_HOME` being honoured inside the flatpak sandbox** (binary strings suggest yes; unproven).
   Probe P1 in build: launch one test cell with a redirected data root, verify prefix/logs/place land under the cell and the
   owner prefix is untouched. Fallback: outer `bwrap` bind of a per-cell prefix over the fixed path (bwrap is present;
   nesting with flatpak must be probed) or a second flatpak **user** (needs owner approval — last resort).
2. **2.2 GB prefix per cell** (plus a Wine prefix is not designed for relocation): disk and creation time; use reflink
   (`cp --reflink=auto`) where supported, capacity gate (e.g. ≤ 4 cells), lazy provisioning, and a shared read-only
   template. Login isolation *requires* a per-cell prefix — no cheaper way to get distinct logins.
3. **Window identity ambiguity on XWayland**: identical class/title across cells is possible if slugs collide or Studio
   truncates titles; require slug uniqueness, escaped regex, exact pid match, and refusal on multi-match; never use the
   default `studio` target.
4. **MCP port machinery**: per-cell ports must be assignable (MCP currently hardcoded 13469 in `checks.py`); the runner's
   instance contract is the risky refactor — keep owner defaults unchanged and add env overrides reviewed by checker.
5. **Resource pressure / crash storms**: one Studio ≈ 1–2 GB RAM + GPU; cap cells (suggest 3–4), a global one-Play-at-a-
   time scheduler, memory guard before start, crash budget/quarantine. Owner may want more cells than the rig sustains —
   capacity is surfaced, not silently exceeded.
6. **Login/account handling**: each cell logs in once, manually/owner-approved, inside its own prefix; the plugin must not
   store credentials, tokens or cookies, and must show `accountLabel` only. Switching accounts = new cell (isolated).
7. **Owner Studio already running**: cells must coexist; the boundary is target-tuple based, so no global "single Studio"
   assumption may leak into the launcher; the owner's `usg-desktop.rbxlx` remains the only place the runner treats as
   canonical sealed.

## 9 · Deliverables & sequence

design (this) → **checker review** (boundary rules, isolation assumptions, proof plan) → build: `tools/studio_fleet/`
(launcher, supervisor, cell schema, instance contract additions to `checks.py`/`vinegar_run.py` behind env defaults, per-cell
winctl/studiocycle configs, guard/unit tests) + sidebar contract docs for Team B → containment proof (§7) → Team B UI
integration. Hard fences: no build of the game, no publication, no credential storage, no image reads, owner Studio never
touched, $0. Open to Team B: polling vs IPC (start with file polling), capacity cap value, whether cells may use distinct
Roblox accounts or the same account signed into multiple cells (login isolation supports both).
