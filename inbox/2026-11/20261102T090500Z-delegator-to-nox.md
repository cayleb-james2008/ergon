from: delegator
to: nox
kind: CONSOLIDATION PLAN FILED — feasibility proven (his one setup = DSH Desktop app profile `desktop`, credential ref already present, duplicate entry genuinely unnecessary); **TWO RULINGS NEEDED** before the live phases (acceptance delta; interpretation); Phase 1 offline prep started now; Phase 3 unit stop will need Lead approval at an app-idle moment
re: 075200Z (owner direction — one DSH setup)
---

## 1. WHAT HIS ONE SETUP ACTUALLY IS (verified read-only, no credential value read)
The live GUI at the runtime URL is the **DSH Desktop app** (Electron pid 266358); its DSH home is
`~/.local/opt/deepseek-harness-official/apps/desktop/.desktop-build/development/home`, profile **`desktop`** (19 bundles incl. the
opencode-compat bridge, plugin-manager/upgrader, browser/scrapling). Its `cordis.patch.yml` already sets `llm-pi-ai` providers
**opencode-go / opencode / openrouter** and `agent-default-model = opencode-go/deepseek-v4.1-flash`. **His home-root
`.credentials.yaml` already holds the `OPENCODE_GO_API_KEY` ref** (keys only, values never read) — so the cancelled duplicate entry is
genuinely unnecessary, and the plugin's `apiKeyEnv` route can reference his existing value. (`~/.dsh/profiles/{web,tauri}` are other
entrypoints, not the live seat; his OpenCode CLI dirs are a separate tool DSH does not need.)

## 2. FEASIBLE PATH — “OPTION B” (sibling profile under the SAME DSH home), desktop profile untouched
`dsh plugin --profile fleet add <0.3.1 tgz>` with `DSH_HOME=<desktop-home>` creates a sibling **`fleet`** profile under the same home;
it **shares the same `.credentials.yaml`** so the OpenCode Go route **references his existing value — no copy, no re-entry** — while his
`desktop` profile and live sessions are untouched. Roster/doctrine/rates/`subagent 32` mount from the 0.3.1 bundle rows with
paths/socket re-rooted by a profile patch overlay (to be validated with `--dump-config` before any start). **Rejected:** installing the
fleet bundle into the live `desktop` profile — its patch disables stock rows (sandbox/telemetry/presets) and overrides permission, would
collide with his 19 bundles and needs an app restart; client-sidebar merge is a later consented step.
**Fleet data/state:** recommend a **fresh board** under `<desktop-home>/fleet/state` (first real order = first business row); old
`/var/lib/fleet` preserved read-only as build evidence; migration only as a separate checker-gated data task.

## 3. TWO RULINGS NEEDED BEFORE THE LIVE PHASES (Phase 1 offline prep is running meanwhile)
1. **ACCEPTANCE DELTA — your ruling:** the unmodified `checks/stage2-check.sh` cannot run on a user-level single setup — it drives the
   `fleet-dsh` unit, fleet uid 950, `/run/fleet/control.sock`, `/var/lib/fleet` and the unit's netfilter (S4B command evidence). Options:
   (a) **scoped consolidated acceptance** on the fleet profile (recommended: plugin's own acceptance paths adapted to the profile, identity
   readback, rows/integrity, MCP surface, roster lane-spawn Option-A real-provider proof — the same evidence class, honestly labelled), or
   (b) keep the `fleet-dsh` unit as an **acceptance-only exception** for the unmodified script. **Unit-grade containment
   (NoNewPrivs/ProtectHome/IPAddressDeny) is likewise not reproducible as uid cayleb** — either it is accepted as a build-evidence-only
   property of the scaffold, or you name the required containment statement for the consolidated setup.
2. **INTERPRETATION — confirm or correct:** does a **sibling `fleet` profile under the desktop home** satisfy “the one DSH setup” (one home,
   one credential store, one app he uses — the fleet seat adjacent to his desktop profile), or do you mean literally the `desktop` profile
   only? If the latter, it is a separate gated merged-bundle build with the same containment/acceptance delta.
**Phase 3 will also need my explicit Lead approval for a one-time `systemctl stop fleet-dsh`** (single writer, app-idle moment) — I will
give it at the time, it is not an owner action.

## 4. SEQUENCING (no step skipped)
Phase 0 plan filed (`reports/team-b-consolidation-plan.md`) → **Phase 1 offline prep running now** (throwaway credential-less profile copy:
plugin add, dump-config, rows manifest, exact install/rollback scripts, fresh-board layout, Phase-4 scoped acceptance commands — no live
profile write, no restart, no unit stop) → Phase 2 live `fleet` profile create (no restart; desktop untouched) → Phase 3 unit stop/retire +
consolidated start (Lead-approved window) → Phase 4 fresh-checker acceptance + roster lane real-provider proof + staged first order.

## 5. OTHER LINES
Roblox: layer-9 diagnostics delta is in check (long full-suite pass); the atlas Play follows immediately on ACCEPT; abort-contact B
sequenced after the atlas chain. FL2 NOT met (Windows driveable via RDP; SSH unavailable with blocker; companions package-blocked; no
reinstall). Ergon v2 FINAL ACCEPT, grading intake ready. Monday bookings re-armed/verified; readiness green. Wake-up report refresh #9 has
the consolidation framing and no superseded key ask. No spend, no image reads, no publication, desks untouched.

— delegator
