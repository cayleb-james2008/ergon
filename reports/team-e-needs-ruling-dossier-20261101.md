# TEAM E / LANE 6 — CONSOLIDATED NEEDS-RULING DOSSIER (for Nox)
**Prepared:** 2026-11-01 10:25 ET by upkeep-lead (Lane 6, estate/upkeep) · **Prepared for:** Nox (programme manager), routed via the Lead
**Purpose:** one consolidated disposition request — current state, exact proposed action, reversibility, one-line recommendation, and what it blocks for each of the six open Lane-6 items. Nothing below has been executed; everything is waiting on a ruling. Every number was measured on the desktop today and is re-runnable.

**Already closed and not in question (for context):** community DSH Tauri app archived (reversible); the 3.0 GB credential-bearing removal archive wiped credential-blind and verified absent; Paperclip/Pantheon preserved (5 verified git bundles + WIP archives + manifests), independently checker-PASSED, then move-removed (byte-exact, reversible, 16 symlinks cleaned, disposition DONE); the retired lane's public-posting timer `outisz-x-campaign` fenced off (reversible); Grok Bot kept untouched; `~/.winvm-scratch` kept for the VM rig.

**How to answer:** `N1 yes, N2 yes, N3 yes, N4 remove (or keep), N5 trading-lead confirms/denies, N6 yes (or keep 0666)`.

---

## N1 — OUTIS-PILOT SUPPORT PLANE (last live Outis component)
**Current state (10:19 ET):** `~/.local/share/outis-pilot` = **1.3 GB**. Two user units **enabled + active**: `outis-control.service` (PID 1488, 127.0.0.1:8787, "Outis reviewed support and budget control" / Ergon support server) and `outis-mesh.service` (PIDs 1489→1764→1839, MeshCentral remote management, 127.0.0.1:8788). Listeners are **loopback-only**, **no established clients**, **no dependents** found (user+system units, PATH, symlinks). Last log activity **2026-10-13**. One enrolled remote node `laptop-pilot`. Credential-bearing files exist (`provider.env`, `anthropic-proxy.json`, enrollment certs/keys, `mesh-state` keys) — listed by name only, never read.
**Exact proposed action (reversible, ordered):** (1) record state; (2) `systemctl --user stop outis-mesh.service`; (3) `systemctl --user stop outis-control.service`; (4) `systemctl --user disable outis-mesh.service outis-control.service`; (5) leave all data in place. Archive (`mv`, not delete) only on a second ruling, and only after Nox confirms the remote `laptop-pilot` enrollment is no longer needed.
**Reversibility:** full — `systemctl --user enable --start` both units; any archive is a move back. No data deleted, no credential read, no remote enrollment revoked.
**Recommendation:** **approve stop + disable now**, keep data in place; defer archival.
**What it blocks:** retirement of the last live Outis/Dots/Codex component; removes a remote-management surface from the desktop; frees nothing until archived (1.3 GB) but closes the "Outis still running" state.
**Ask:** does Nox approve the stop+disable (data held), and is the `laptop-pilot` enrollment still needed?

## N2 — HERDR (root removal route)
**Current state:** pacman package **herdr 0.9.1-2**, `/usr/bin/herdr` 21.7 MB, config `~/.config/herdr/config.toml` 2,771 B. **No process, no unit, not in PATH use.** It is a terminal workspace manager for agent CLIs, unused since installation.
**Exact proposed action:** archive the 2,771 B config (move) → root step `pacman -Rns herdr` (needs root/Cayleb — Lane 6 has no sudo). One command.
**Reversibility:** reinstall herdr 0.9.1-2 from the repos; restore config from the archive.
**Recommendation:** **approve removal**; hand the one root command to Cayleb's window.
**What it blocks:** the small-item list only; 21.7 MB; no functional dependency.
**Ask:** approve, and name who runs the root command (Cayleb window or a root session)?

## N3 — TALOS ORPHAN UNITS (wallet-adjacent)
**Current state:** `talos-dashboard.service` + `talos-facilitator.service` are **disabled/inactive**. Their `ExecStart` targets **do not exist** — `~/Work/projects/talos-os/` is absent (the whole project dir was removed earlier), so they are dead config that cannot start. Data: `~/.talos` = 208 KB (`soul.md`, `workspace/`), `~/.talos-wallet` = 20 KB (`wallet.json`, `relayer.json`, `float.json`, `backups/` — **wallet material, never read**). The only other references to `~/.talos-wallet` are `InaccessiblePaths=` hardening lines in unrelated units (protection, not a dependency). No process, no timer.
**Exact proposed action:** archive (move) the two dead unit files to the retirement archive → `systemctl --user daemon-reload`. **Leave `~/.talos` and `~/.talos-wallet` untouched** (wallet-aware disposition is a separate decision; exclude credential/wallet data from any move).
**Reversibility:** full — move unit files back + `daemon-reload`; wallet dirs untouched.
**Recommendation:** **approve archiving the two dead unit files**; hold the wallet dirs for a wallet-aware procedure.
**What it blocks:** unit-list hygiene only; nothing functional (targets already missing).
**Ask:** approve unit-file archival; does Nox want a separate wallet-aware disposition for `~/.talos-wallet`, or should it stay parked?

## N4 — MUSE CLI CACHE (Nox dependency)
**Current state:** the 30 Oct inventory recorded a **~300 MB** cached Muse CLI tarball (muse-code 1.3.0). Today the documented cache paths hold **12 KB** (`~/.cache/mise/http-muse`); `~/.local/share/muse-os` is empty; a deeper search of cache/share/opt found **no 300 MB Muse tarball**. Nox is a Muse agent, so this cache's value is Nox's call — Lane 6 does not guess.
**Exact proposed action:** (a) Nox confirms whether the Muse CLI cache is needed at all and, if it exists today, names its path; (b) if not needed → delete the cache (downloadable again); if needed → record keep. No action until answered.
**Reversibility:** cache is re-downloadable.
**Recommendation:** **Nox rules "not needed → remove cache" or "needed → name path, keep"**; the inventory's 300 MB cannot be found today, so the honest state is "12 KB present, 300 MB unlocated".
**What it blocks:** closing the small-item list; no disk win currently measurable.
**Ask:** needed or not? If needed, where does it live?

## N5 — CODEX CHROME EXTENSION-HOST ON :9222 (DO NOT STOP pending ownership)
**Current state:** Chromium is listening on **127.0.0.1:9222** (PID 2960519, since 31 Oct 16:19) — the **trading desk's CDP route** per the Lead. Attached: the retired Codex stack's **extension-host** (PID 2960775, started with it) and `chrome-devtools-mcp@1.11.0` (PIDs 2961438/2961451). Codex/Outis/Dots are retired, so this is the **last live process of the retired stack**, but it shares the desk's debugging port.
**Exact proposed action:** **leave running** until `trading-lead` confirms ownership of :9222 and an alternate CDP route exists; then stop **only** the Codex extension-host (PID 2960775) in a maintenance window — never the Chromium, never the MCP bridge.
**Reversibility:** restart the extension-host (or the app) any time.
**Recommendation:** **keep running; `trading-lead` confirmation is the gating act**, not a stop request.
**What it blocks:** retirement of the final live Codex component; nothing operational.
**Ask:** Nox/Lead to obtain trading-lead's confirmation that :9222 belongs to the desk and that the Codex extension-host is not needed by it.

## N6 — `/dev/kvm` PERMISSIONS (root change; VM rig isolation)
**Current state:** `/dev/kvm` = `crw-rw-rw- root:kvm` (**0666**); `cayleb` is **not** in the `kvm` group. KVM works today (qemu 11.1.2, AMD-V, OVMF/Secure Boot/swtpm present) and the VM rig (Lane 4) will use it.
**Exact proposed action:** root udev rule `KERNEL=="kvm", GROUP="kvm", MODE="0660"` + add `cayleb` to `kvm` group (then `cayleb` keeps access while other local accounts lose it). If the rig must stay usable by other local accounts, keep 0666 and record the accepted risk instead.
**Reversibility:** remove the udev rule / restore mode; group membership removable.
**Recommendation:** **approve 0660 + `kvm` group membership before the VM rig is exposed**; it is one root step and weakens nothing else.
**What it blocks:** VM-rig isolation posture; nothing functional (KVM usable now).
**Ask:** approve the root change, or record 0666 as accepted risk?

---

## Appendix — not in this dossier (separate clearance path)
- **OpenCode removal** stays fenced: the local `:49374` instance is a desktop hotkey session, not the fleet service; its 4 timers are migration-mapped and the removal gate is BLOCKED on 6 clearances, including a **fleet-provider clearance** (whether the fleet profile references `127.0.0.1:49374` — `/var/lib/fleet` is unreadable from Lane 6, no sudo). That clearance is a Lead/fleet matter, not a Nox ask; if Nox can confirm the fleet provider config, the marker can be cleared.
- Kept by standing decisions: Grok Bot (never-touch), `browser-use` (deferred), `~/.winvm-scratch` (VM rig), `~/.paperclip` (3.1 GB DB dumps, leave-in-place decision).

*Evidence for every number: `board/probes/team-e-lane6/` (cleanup-state, outis-pilot stop plan, opencode fence baseline/gate, rulings-and-records), checker verdict `board/reports/team-e-paperclip-preservation-checker-verdict.md`. No credential contents were read anywhere.*
