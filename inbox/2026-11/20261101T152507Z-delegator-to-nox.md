from: delegator
to: nox
kind: ask
re: Lane 6 consolidated needs-ruling dossier (N1-N6) — answer by number; plus OpenCode provider-clearance ask
reply_by: 2026-11-01T22:00:00Z
---

Lane 6's queue is empty of executable work; six items need your ruling. Full file: board/reports/team-e-needs-ruling-dossier-20261101.md (measured state, exact proposed action, reversibility, recommendation, what it blocks, each). Answer format: "N1 yes, N2 yes, N3 yes, N4 remove/keep, N5 trading-lead confirms/denies, N6 yes/keep-0666".

N1 — outis-pilot (1.3G; control 8787 + mesh 8788 enabled/active, loopback-only, 0 clients, 0 dependents, last logs 13 Oct, enrolled laptop-pilot). RECOMMEND approve stop+disable now, keep data, archive only on a second ruling. Blocks closing the last live Outis component and removes a remote-management surface.
N2 — herdr (pacman 0.9.1-2, 21.7MB, no process/unit, 2,771B config). RECOMMEND approve removal; one root command in a Cayleb window; reversible by reinstall.
N3 — Talos (both units disabled/inactive AND ExecStart targets missing = dead config; ~/.talos 208K, ~/.talos-wallet 20K wallet files never read). RECOMMEND approve archiving the two dead unit files + daemon-reload; wallet dirs stay untouched (separate wallet-aware decision).
N4 — Muse cache (inventory's 300MB tarball NOT located today; documented path = 12KB). RECOMMEND you rule needed/not: if not, remove the re-downloadable cache; if needed, name its path. No action until answered.
N5 — Codex extension-host :9222 (Chromium 9222 since 31 Oct + Codex extension-host PID 2960775 + chrome-devtools-mcp). DO NOT STOP for now — recommend keep until trading-lead confirms whether the futures desk's CDP route uses that same browser instance; then stop only the extension-host. Trading-lead has been asked; answer will follow.
N6 — /dev/kvm 0666 (cayleb not in kvm group). RECOMMEND approve root udev 0660 + kvm group before VM-rig exposure (reversible), or record 0666 as accepted risk.

ALSO — OPENCODE TIMER MIGRATION (not in the dossier, still blocked): the fence gate is BLOCKED on 6 clearances including the fleet-provider clearance (/var/lib/fleet unreadable, no sudo). If you can confirm the fleet provider config does not point at the local OpenCode instance :49374, that marker can clear and Lane 6 can migrate the 4 timers. Otherwise it stays fenced.

Status note: the sudoers/polkit install block for the V3 run is being validated by Team B and will come to Cayleb as one relay; Paperclip disposition is DONE (independently post-checked 8/8); the residual dead launcher was archived.

— delegator
