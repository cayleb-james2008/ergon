# Team B — Consolidation Phase 2: live `fleet` profile created (no start)

**Filed:** 2026-11-02T09:28Z (04:28 EST) · **Author:** fleet-lead (Team B) · **Approval:** Lead (Phase 2 approved 091500Z after Nox ruled Option B)
**Scope actually performed:** create the sibling `fleet` profile under the one DSH Desktop home, install the pinned candidate into it, write its overlay + recovery key, prove it composes, and prove nothing else changed. **No start, no app restart, no credential re-entry, no unit change, no `/var/lib/fleet` or `~/.dsh` write, no spend.**
**Fence:** values of `.credentials.yaml` were never read (stat only); the recovery private key was generated locally and never printed; Phase 3 remains held.

## 1. One-line state

The consolidated profile exists at `<desktop-home>/profiles/fleet`, composes cleanly (`--dump-config` rc=0, 59,979 B, integrity-relevant rows all re-rooted), and is **not running**. The desktop app, the desktop profile and the `fleet-dsh` unit are byte-for-byte as they were before the work. Two pinned-row families remain for the Nox ruling (§6) before Phase 3.

## 2. Pre-record (taken before any write, 09:12:1xZ)

| Item | Before |
|---|---|
| `profiles/desktop/cordis.patch.yml` | `77476b79afe6d601a75f0b6e17b88683ba71cfb3e44cae30eb83047ead052a0c` |
| `profiles/desktop/package.json` | `ce364672e9bac6476f623eeaeec8a0d7f2bbe6a0bec38eee9d687377103e2594` |
| `.credentials.yaml` | mode 600, size 410, mtime 2026-11-01 09:30:40.226764748 -0500 (stat only) |
| `fleet-dsh` unit | active, InvocationID `999771adc0a84fa7b07d3e56fe97bc28`, NRestarts 0 |
| desktop app | Electron main pid 266214, started Nov 1 12:40:27; GUI `http://127.0.0.1:35645` answering (HTTP 401 auth gate) |

## 3. Actions

| # | Action | Result |
|---|---|---|
| 1 | `mkdir <desktop-home>/fleet` and `$XDG_RUNTIME_DIR/fleet` | both exist; runtime dir empty after |
| 2 | `age-keygen -o <desktop-home>/fleet/recovery.key`; derive `recovery.pub` | key 189 B mode 600; pub 63 B mode 644; neither printed |
| 3 | `dsh plugin --profile fleet --from-default-profile web` (init) | rc=0; fresh dump 44,801 B |
| 4 | `dsh plugin --profile fleet add <candidate-031 tarball>` | rc=0; pnpm log clean; `@cayleb/dsh-fleet file:…cayleb-dsh-fleet-0.3.1.tgz` |
| 5 | render `profile-overlay.yml` tokens → `<desktop-home>/profiles/fleet/cordis.patch.yml` | 57 lines, 4,327 B, sha256 `b1874b3ae7354d0bd89711ebf4c5a0a66558ff6225b39e4c41b1e8a0ed1b68e6` |
| 6 | static pin audit of overlay vs `rows.manifest.json` | PASS — see §5 |
| 7 | `DSH_HOME=<desktop-home> node <cli> --profile fleet --dump-config` | rc=0, 59,979 B, no start |

Installed package identity is the **unmodified** candidate r2: `rows.manifest.json` sha `1970f9954b7131c58191327bb0e60b9d31a704bc309d083cd53971bc9271edba`, `mcp.row.yml` sha `e32c2f060ec5e580f2cac7b524156d1b486bb44bb4255551b81e2b9fb0e141b9`, 123 package files. (Only the *copy* variant in Phase 1b has re-rooted values.)

Live profile files: `cordis.yml` `c300dcf2…`, `pnpm-lock.yaml` `b2700788…`, `pnpm-workspace.yaml` `ae7c5b68…`, `package.json` `a7d5b80a…` (bundles `@deepseek-ai/dsh-base`, `@deepseek-ai/dsh-web-app`, `@cayleb/dsh-fleet`), `cordis.patch.yml` `b1874b3a…`; 133 files total (123 package + 6 profile + 4 pnpm bookkeeping), 3 executable.

## 4. Composition proof (`phase2-dump-live.yml`)

17 fleet-plugin entries compose: plugin root `fleet` + **12 rows** (`fleet-core, fleet-orders, fleet-roster, fleet-lessons, fleet-policy, fleet-ledger, fleet-housekeeper, fleet-sandbox, fleet-egress, fleet-fsguard, fleet-firstrun, fleet-mcp`) + **4 preset inserts** (`preset-fleet-lead/builder/checker/researcher`).

Re-rooted values (all under the live desktop home / runtime dir):

| Row | Key | Composed value |
|---|---|---|
| fleet-core | fleetRoot / workRoot | `<DH>/fleet`, `<DH>/fleet/work` |
| fleet-housekeeper | recoveryFile | `<DH>/fleet/recovery.pub` |
| fleet-egress | logPath | `<DH>/fleet/log/egress.jsonl` |
| fleet-roster | credentialStore / sessionRoots | `<DH>/.credentials.yaml` (reference only), `[<DH>/sessions]` |
| fleet-policy | logFile | `<DH>/fleet/log/actions.jsonl` |
| fleet-sandbox | maskRoots / egressSocket | `<DH>/fleet/{dsh,state,log,work,backups,checks}`, `$RUNTIME/fleet/egress.sock` |
| fleet-lessons | skillDir | `<DH>/fleet/skills` |

Defaults confirmed: `maxActiveSubagents: 32`; roster engine `opencode-go` with `apiKeyEnv: OPENCODE_GO_API_KEY` (**env-var name only** — the dump contains no credential value and the credential file was never read).

## 5. Static pin audit

`rows.manifest.json` pins 8 ids: `fleet-ledger, fleet-mcp, fleet-policy, permission, preset-fleet-lead, preset-fleet-builder, preset-fleet-checker, preset-fleet-researcher` (plus 12 disabled ids).

- Overlay overrides only 8 ids: `fleet-core, fleet-sandbox, fleet-egress, fleet-lessons, fleet-housekeeper, session-query-sqlite, fleet-policy, fleet-roster`.
- Intersection with the pinned set is `fleet-policy` only, a **mount** row whose pinned field is `name` (its config is free); the overlay sets config only and the composed name stays `@cayleb/dsh-fleet/policy`.
- `fleet-mcp` is deliberately **not** overridden (overriding its pinned config is what fails Phase 1).

⇒ No pinned field is touched, so the rows check should pass at boot. The runtime verdict itself is **not claimed here** — it is deferred to Phase 3 start; the Phase 1b copy proved `integrity {ok: true, failures: []}` for this same overlay shape with a variant mcp row.

## 6. Findings feeding the held ruling

**F1 — pinned `fleet-mcp` row keeps unit paths.** Composed from the shipped config unchanged: `port 8789`, `bindSourceAllow [loopback, tailnet]`, `statePath /var/lib/fleet/state/mcp`, `logPath /var/lib/fleet/log/mcp-actions.jsonl`, `adminSocket /run/fleet/mcp-admin.sock`, `fleetRoot /var/lib/fleet`, `peerUid 1000`. At Phase 3 start this row would go **DEGRADED (EACCES)** unless the variant build (option a) is used — Phase 1 evidence, reproduced in the Phase 1b control run. The only unit-path leftovers in the whole dump are this row (4 fields) and F2 (4 lines).

**F2 — pinned preset rows keep unit skills path.** All 4 `preset-fleet-*` insert rows pin their config, which contains `customSkillDirs: [/var/lib/fleet/skills]` (dump lines 1422/1489/1544/1608). This cannot be re-rooted from the overlay without breaking the pin. Phase 1b session probe shows a preset session still **starts** and a stub turn completes with that path unreadable (no mount failure, no DEGRADED line) — so option (b) degrades skill availability only, not session creation; skill listing was not exercised.

Implication for the ruling: option (a) variant should re-root F1 **and** (for full parity) the 4 preset patch files, then regenerate the manifest with the package's own generator; option (b) accepts a degraded MCP row in the consolidated profile and keeps the unit as MCP/acceptance host, with presets loading no unit skills.

## 7. Post-record (09:24Z, after all work and after the Phase 1b copy runs)

| Check | Result |
|---|---|
| desktop `cordis.patch.yml` / `package.json` | byte-identical to §2 (`0ff76e08…` hash file matches before/after) |
| `.credentials.yaml` | mode 600, size 410, mtime Nov 1 09:30:40 (predates this work; never read) |
| desktop app | Electron main pid 266214 unchanged (started Nov 1 12:40:27); GUI 35645 HTTP 401 (alive) |
| `fleet-dsh` unit | InvocationID `999771ad…` unchanged, NRestarts 0, ActiveEnterTimestamp Nov 1 20:18:05 EST, unit file sha `b44c05d0…`; pid 669437 |
| listeners | only the unit's: 8789 on 127.0.0.1 / [::1] / 100.114.17.67, 3091 on 127.0.0.1; **nothing new** |
| consolidated profile process | none (no desktop-home `--profile fleet` process) |
| `$RUNTIME/fleet` | empty (no state created) |
| `/var/lib/fleet` | untouched (0750 fleet; only `ls` used) |
| writes under desktop home | only `profiles/fleet/**` and `fleet/**`, all 04:12; the 04:25 `storages/…projcache` mtime is the running app's own activity |

**Single-writer:** the unit is still the only writer. Phase 3 must not start the consolidated profile while `fleet-dsh` runs (it would also collide on 8789).

## 8. Phase 3 status and rollback

Phase 3 (stop/retire the unit → start the consolidated profile) is **HELD**: it needs Nox's ruling on F1/F2 and Lead approval at an app-idle moment. No partial cutover was attempted.

Rollback is trivial and touches nothing else: remove `<desktop-home>/profiles/fleet` and `<desktop-home>/fleet` (recovery keypair), and the now-empty `$RUNTIME/fleet`. The desktop profile, credentials, unit and runs are untouched and keep working.

## 9. Evidence (under `board/probes/team-b-consolidation/`)

| File | sha256 (head) | What it shows |
|---|---|---|
| `phase2-profile-init-dump.yml` | `3a947757…` (44,801 B) | fresh web-template dump before overlay |
| `phase2-dump-live.yml` | `b14de0c8…` (59,979 B) | composed live profile |
| `profile-overlay.yml` | `d240c00d…` | overlay source (tokens) |
| `phase2-plugin-add.log` | `9e7bf1bb…` | plugin add rc=0 |
| `phase2-credential-stat.txt` | `5f1bdfca…` | stat-only record of the credential file |
| `phase2-desktop-hashes-before.txt` / `-after.txt` | `0ff76e08…` (both) | desktop profile unchanged |
| `phase2-install-consolidated-profile.sh` | — | the scripted Phase 2 install (overlay + key + dump) |
| `phase2-install-consolidated-profile.sh` siblings | — | Phase 3 cutover + rollback scripts, **not run** |

Cross-reference: Phase 1 (`team-b-consolidation-phase1.md`), Phase 1b variant prototype (`team-b-consolidation-phase1b.md`), plan (`team-b-consolidation-plan.md`).
