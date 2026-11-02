# TEAM B — CONSOLIDATION PLAN: fleet plugin into the owner's ONE DSH setup

Scope-first plan for Nox direction 20260752Z ("ONE DSH setup only; the fleet plugin consolidates into it; the duplicate
key entry is CANCELLED"). **Nothing has been installed, restarted or written into his setup by this plan.** Read-only
verification: 2026-11-02T06:39–06:55Z (UTC); no credential value was read or printed anywhere.

## 1. What "his one DSH setup" concretely is (verified read-only)

| Piece | Actual, from disk |
|---|---|
| The app he uses | **DSH Desktop app** (Electron, pid 266358 = the process serving the live GUI): Web GUI at `http://127.0.0.1:35645` (port owner is that Electron process). |
| Its DSH home | `~/.local/opt/deepseek-harness-official/apps/desktop/.desktop-build/development/home/` (the app passes `…/home/profile/desktop` to its host). |
| Its profile | **`desktop`** — the only profile under that home's `profiles/` (`profiles/desktop/{cordis.yml,cordis.patch.yml,package.json,node_modules,…}`). |
| Profile composition | 19 bundles incl. `dsh-base`, `dsh-web-app`, `@opencode-compat/dsh-bridge`, `cursor-opencode-provider`, `dsh-free-search`, `dsh-inner-browser-use`, `dsh-scrapling`, `dsh-plugin-upgrader`, …; `cordis.patch.yml` already overrides `llm-pi-ai` with providers **opencode-go / opencode / openrouter** (`apiKeyEnv` names only) and sets `agent-default-model = opencode-go / deepseek-v4.1-flash`. |
| His credential surface | home-root **`.credentials.yaml`** (mode 600): ref names present — `OPENCODE_GO_API_KEY`, `OPENCODE_API_KEY`, `OPENROUTER_API_KEY`, `client-connection/browser-session` (keys only; **values never read**). |
| His OpenCode CLI | separate tool at `~/.config/opencode/` + `~/.local/share/opencode/` (`auth.json` mode 600 present, value never read). It is **not** what DSH uses for the route; DSH already has its own `OPENCODE_GO_API_KEY` ref — so no copy/re-entry is needed from this side either. |
| Other profiles | `~/.dsh/profiles/web` (minimal base+web-app, empty patch) and `~/.dsh/profiles/tauri` (Tauri app links) — not the live GUI seat; they are other/old entrypoints. |
| Fleet scaffold (today) | `/var/lib/fleet` + systemd unit **`fleet-dsh`** (user `fleet`, uid 950, currently **active**, MainPID 669437) running `@cayleb/dsh-fleet` 0.3.1 installed in that profile. |
| Plugin artifact | `~/workspace/harness-build/candidate-031/dist/cayleb-dsh-fleet-0.3.1.tgz`, 337,454 B, 123 members, sha256 **`5ede748489e448c525b1b201cddb8ea5b5a4b9305e231328b49fd49a5653aef4`**. |
| Fleet data/state + door | `/var/lib/fleet/state` (board DB, roster rows), `/var/lib/fleet/{work,checks,gitwork,engines}`, door at `~/.dsh/strike-door/` (token via admin socket; no value in files). |

## 2. Recommendation — where the plugin lives in the single setup

**Option B (RECOMMENDED): add a dedicated `fleet` profile under the SAME DSH home the app uses**, i.e.
`<desktop-home>/profiles/fleet`, sharing the home-root `.credentials.yaml`. One DSH installation, one home, one
credential store; the owner's `desktop` profile (and his live sessions) are untouched.

- Install mechanism (verified from the 0.3.0/0.3.1 installs and the scratch evidence):
  `node ~/.local/opt/deepseek-harness-official/apps/cli/lib/bin.js plugin --profile fleet add <path>/cayleb-dsh-fleet-0.3.1.tgz`
  with `DSH_HOME=<desktop-home>`. `dsh plugin add` forwards to pnpm in the profile and **appends the bundle to
  `dsh.profile.bundles`** (proof: previous scratch profile `dsh.profile.bundles = [dsh-base, dsh-web-app,
  @cayleb/dsh-fleet]`). Then the fleet bundle's rows (`fleet-core`, `fleet-orders`, `fleet-policy`, `fleet-ledger`,
  `fleet-lessons`, `fleet-roster`, `fleet-mcp`, `fleet-firstrun`, `fleet-sandbox`, `fleet-fsguard`, presets) compose in
  that profile exactly as in the fleet unit; the 0.3.1 row set is already integrity-pinned (8 pinned / 12 disabled).
- **Option A (REJECTED, evidence): install into his live `desktop` profile.** The fleet bundle patch disables stock rows
  and overrides `permission`/presets (sandbox, fs-sandbox, telemetry rows, preset-standard/ptc/minimal/cordis,
  workspace-changes, ui-deliverables); his profile mounts 19 bundles incl. plugin-manager/upgrader and a heavily
  customised patch. Mounting the fleet bundle there would collide row-by-row and require an app restart, with real risk
  to his live app — against the care fence. Any later "fleet UI inside his app" work belongs to a separate, consented
  merge step (client row + selected board/MCP rows only), not this consolidation.
- **Option C (fallback): `~/.dsh/profiles/fleet`** (the minimal web profile tree) if Nox means the `~/.dsh` home rather
  than the desktop build home. Same bundle install; but the credential store would be `~/.dsh/.credentials.yaml`, which
  **does not exist** (verified absent) → the owner's existing `OPENCODE_GO_API_KEY` would then have to be entered again,
  which the direction explicitly cancels. So Option C is only viable if the fleet profile is pointed at the desktop
  home's store via config — unnecessary when Option B already shares it.

**How the roster/doctrine/rates/OpenCode-Go route mount:** exactly as shipped in 0.3.1's bundle rows — `llm-pi-ai`
`providers.opencode-go { apiKeyEnv: OPENCODE_GO_API_KEY }`, ledger `rates` row
`opencode-go/deepseek-v4.1-flash`, `fleet-roster` mounted with `roster.models` doctrine (lead Sonnet 5.5;
builder/researcher deepseek-v4.1-flash via opencode-go; checker Haiku 5.5) and `subagent.maxActiveSubagents: 32`
(headroom). Paths get a fleet-profile config overlay (not a row edit) so nothing writes outside the home:
`fleetRoot: <desktop-home>/fleet`, `dbPath/logDir/workRoot/checksRoot` under it, `controlSocket:
$XDG_RUNTIME_DIR/fleet/control.sock`, roster `gitRoot/engineRoot` under the fleet root, MCP `statePath` under the fleet
root. Each override is a **profile patch entry**, validated offline with `--dump-config` before any start.

**Credential reference (the point of the consolidation):** the fleet profile is under the same home, so the DSH
credential seam resolves `OPENCODE_GO_API_KEY` from the **same `.credentials.yaml` the owner already uses** — the
existing value is referenced, never copied, never re-entered. Pre-flight proof is `providers-get` in the consolidated
host showing the opencode-go route with `credential.present: true` (Boolean only; no value read). The duplicate fleet
`.credentials.yaml` under `/var/lib/fleet` is retired with the unit — no second key store, no second entry.

## 3. Fleet data/state — recommendation: FRESH BOARD (do not migrate)

- **Recommend fresh**: first start of the consolidated fleet profile creates a new board DB under
  `<desktop-home>/fleet/state`; the staged first real order becomes its first business row.
- **Why:** the existing `/var/lib/fleet` DB is fleet-uid-owned and unit-path-bound (work/checks/gitwork/engines,
  >1,000 fixture orders, selftest artifacts, sandbox roots); copying it into a user home would carry fleet-uid paths,
  fixture residue and a second writer's history into the acceptance surface for zero operational value. The old tree is
  preserved read-only as **build evidence** (0.3.0/0.3.1 proofs remain on the record); the unit is stopped, not deleted.
- Migration is only worth it if Nox wants the old board's history *inside* the single setup — that is a separate,
  checker-gated data task (export/import with path rewrite), not part of the first consolidation.

## 4. Single-writer discipline + care fence

- **One writer at a time:** the old `fleet-dsh` unit is the current writer; the consolidated fleet profile's first start
  needs a **Lead-approved stop of the unit first** (and the unit marked retired/disabled thereafter so two processes can
  never own a board). Prep steps write only to scratch/throwaway locations; the consolidated profile is created and
  composed **before** the stop, but not started.
- **Care fence:** the install and start never force-close, hijack or message his live app or sessions; the desktop
  profile's files are not edited (only a new sibling `profiles/fleet` is created inside the same home); the app restart
  is needed only for the *optional* future client-row merge, not for Option B; no image reads; no credential values; if a
  start conflicts with his session, stop and wait — no retries that touch his UI.
- **Acceptance run and the door** must use the consolidated control socket and MCP endpoint; the old unit's sockets must
  not be used while it is retired.

## 5. Acceptance re-proof on the consolidated setup (plan)

1. **Identity/rows readback:** `first-run --status` from the consolidated profile → version **0.3.1**, 123 files, modes
   {0644,0755}; control `status` → `ok/integrity.ok/rows 8-12/failures[]`; `dump-config` → fleet rows + roster + subagent
   32 + opencode-go route; `providers-get` → route present, credential present **true** after the shared store resolves.
2. **Real-provider proof (Option A lane):** `lane-spawn {role:"builder", start:true}` on the consolidated host → lane
   `builder:1` on `opencode-go/deepseek-v4.1-flash`; `lane-get` + **real non-synthetic `ledger-get` event** (priced by
   the new rates row); `lane-stop`.
3. **First real order:** `order.stage_real` with the staged payload (real `checkpoint_at`) → real `fixture:false` order,
   read back via `board-get`; not driven until a Lead decomposes it.
4. **MCP battery:** the door's own probe (`mcp-probe.mjs`, loopback + tailnet as available) — nine tools, auth refusals,
   action-log chain.
5. **Unmodified `checks/stage2-check.sh`: NAMED BLOCKER — cannot run in the single setup.** The script drives the
   **systemd unit** (stop/start/SIGKILL leg 2.11), the **`fleet` uid**, `/run/fleet/control.sock`, `/var/lib/fleet`
   layout and unit netfilter. A user-level profile has none of those. Options for Nox to rule: (a) accept a **scoped
   consolidated acceptance** that runs the same legs it can (rows identity, board, roster, provider, MCP, first order)
   and records the unit legs as N/A with evidence; or (b) keep the systemd unit as an explicit, documented exception to
   "one setup" solely as the acceptance/demo harness. Recommendation: **(a)** for the finish line, with the unit-based
   S4B proof retained as build evidence (already ACCEPT 60/60 on installed 0.3.0/0.3.1).

## 6. What genuinely cannot run (or degrades) in the single setup — named

1. **Unit-grade containment is lost:** `NoNewPrivs`, `ProtectHome`, `ProtectSystem=strict`, IPAddressDeny/Allow
   netfilter, `KillMode=control-group`, dedicated `fleet` uid. The plugin's sandbox/fsguard still run (bwrap works
   user-level; fsguard paths re-root to the fleet root), but the host-level guarantees the FL1 identity legs rely on are
   not reproducible as uid cayleb. This is the material acceptance delta and needs the Nox ruling in §5.5.
2. **Systemd restart/SIGKILL leg (stage2 2.11) and fleet-uid ownership checks** — N/A in the single setup.
3. **A second writer is possible if the old unit is left running** — prevented by the stop/retire rule (§4).
4. **The client/sidebar surface inside his live app is NOT part of this consolidation** — that is a deliberate later
   step (merged client row, consented app restart), because the fleet bundle cannot be mounted into the desktop profile
   safely (§2 Option A).
5. Nothing about the **model route** is blocked: `OPENCODE_GO_API_KEY` already exists in his setup's credential store
   (ref verified; value untouched), so the cancelled duplicate key entry is genuinely unnecessary.

## 7. Sequencing (prep now is safe; install only at a consented window)

- **Phase 0 (this file):** plan + read-only verification. ✅
- **Phase 1 — offline prep, touches nothing live:** stage the 0.3.1 artifact outside the live home; create a **throwaway
  copy of the profile tree without credentials** (never copy `.credentials.yaml`; a credential-less throwaway proves
  composition/rows only) and compose the fleet profile there: `plugin add <tgz>` → `--dump-config` → rows/manifest
  check → expected `fleet` profile patch (paths + socket) → `first-run --status` mechanism check. Write the exact
  install/rollback script + backup list (profile files are 600; backup before any live write).
- **Phase 2 — live profile creation (still no start):** with Lead approval, create `profiles/fleet` in the live home and
  install the bundle with `DSH_HOME=<desktop-home>`; `--dump-config` only; verify `providers-get` sees the shared
  credential ref (Boolean); **no service/app restart** (desktop profile untouched, so his app keeps running).
- **Phase 3 — single-writer cutover (consented window, app idle):** Lead-approved `systemctl stop fleet-dsh` + retire;
  start the consolidated fleet profile (user-level, background job) on the compacted config; identity/rows readback.
- **Phase 4 — acceptance + checker:** §5 steps with a fresh checker; then the owner's key entry is **not needed** (ref
  already present) — only a Lead-run `providers-get` presence check; then roster lane proof; then the first real order;
  FL2/finish-line closes on the consolidated setup, with the S4B unit proof retained as build evidence.
- **Rollback:** stop the consolidated profile, restore nothing in the desktop profile (untouched), re-enable/start the
  old unit (still installed, data intact) — a documented one-command reversal until the unit is deleted.

## 8. Named blockers / rulings needed before Phase 2

1. **Nox ruling on the acceptance delta** (§5.5): scoped consolidated acceptance vs keeping the unit as the acceptance
   harness — without it the finish line cannot be re-proved at FL1 strength on the single setup.
2. **Care-fence window + Lead approval** for Phase 3 (unit stop/retire) — not an owner action; owner is not needed and
   should not be disturbed.
3. **Explicit statement that a sibling `fleet` profile under the desktop home is "the single setup"** (my read of the
   direction; written here so it can be corrected before any write). If Nox means literally the `desktop` profile only,
   the merged-bundle work in §2 Option A becomes a separate gated build and the same §5.5/§6.1 deltas apply.
