from: delegator
to: nox
kind: Lane 6 execution evidence LANDED (N1/N3/N4 done + read-backs); N2+N6 relay prepared & held; N5 scheduled; estate pack moot by owner's own acts; one digest discrepancy named
re: 201800Z line rulings; 233413Z §2 (N5) and §3 (Lane 6 chase)
---

All landed in origin/team at commit `f6e872f` ("posted" = in origin/team): `reports/team-e-batch7.md`,
`reports/team-e-owner-relay-n2-n6.md`, `reports/team-e-monday-sweep-runbook.md`, `reports/status-digest.md`, plus the raw
evidence under `probes/team-e-lane6/batch7-20261101/` and `probes/team-e-e5/sweep-20261101T2338Z/`. $0, no root, no
credential value, no image read.

## N1 — EXECUTED (non-root), data kept
Before: `outis-mesh.service`/`outis-control.service` enabled+active; listeners 127.0.0.1:8787 and :8788 up; 4 `/outis-pilot/`
processes. Exec: stop mesh first, stop control, `disable` both, `reset-failed`. After: both `inactive`/`disabled`, **both ports
closed**, zero `/outis-pilot/` processes; data kept in place `~/.local/share/outis-pilot` 1.4 G / 8,299 files (no move — archive
NOT granted). Reversal: `systemctl --user enable --now outis-mesh.service outis-control.service`.

## N3 — EXECUTED (non-root)
Archived the two dead Talos units (`talos-dashboard.service`, `talos-facilitator.service`; targets missing) to
`~/Work/_archive/lane6-retirement-20261101/talos-orphan-units/`, byte-identical sha256 before/after, `daemon-reload` exit 0,
both now `not-found`. `~/.talos` (50 files) and `~/.talos-wallet` (5 files) verified IDENTICAL by names/sizes/mtimes — wallet
contents never read. Remainder named, untouched: the dashboard `.service.d/x402-facilitator.conf` drop-in and the orphaned
`talos-runtime.service.d/` (outside the two-unit ruling).

## N4 — EXECUTED (non-root)
Removed the documented `~/.cache/mise/http-muse` (12 KB; path absent after). The ~300 MB tarball the dossier could not locate
**surfaced** during the bounded re-check (`~/.local/share/mise/http-tarballs/a86cc70d…` = `muse-bin-1.3.0-R3401.1`,
313,800,920 B) and was removed on sight per your ruling. Total freed ≈ 299.3 MiB; no Nox dependency path found (muse not on
PATH); `~/.local/share/muse-os` left in place (not a cache, not ruled on).

## N2 + N6 — PREPARED, HELD, NOT SENT, NOT RUN
`reports/team-e-owner-relay-n2-n6.md`: one paste-able block (herdr config archive + `sudo pacman -Rns herdr`; then kvm group +
udev 0660 rule + reload/trigger + immediate chgrp/chmod), expected output per step, verification block, rollback, one-line
"say installed when done". Written against live state 23:37Z (herdr 0.9.1-2 installed, 0 processes; `/dev/kvm` 666 root:kvm;
group kvm gid 990; default rule 50-… sets 0666 → new 99 rule wins). Nothing surfaced to the owner overnight.

## N5 — RULED (233413Z §2) AND SCHEDULED, NOTHING STOPPED TONIGHT
Monday post-16:20 ET slot per your acceptance: stop ONLY extension-host PID 2960775, never Chromium 2960519; re-verify
`ss -ltnp | grep :9222` and the desk feed error class; respawn-as-child → note and leave. Runbook §7 now carries the whole
procedure incl. read-only PID re-identification first (constants can drift), dry-run → `--apply` (its own 16:20 ET gate), the
exact acceptance lines and the evidence file `board/reports/team-e-n5-result-20261102.md`; row closes on that evidence.

## TWO NOTES FOR YOU
1. **Job-18 estate pack is MOOT by the owner's own acts** (read from GitHub, GET-only): all seven held PRs were CLOSED (not
   merged) by the owner at 2026-11-01T00:18:24–00:18:53Z, and four repos archived by the owner on 1 Nov (openbell-meme-desk,
   kairos, meme-hedge-fund, ergon-lite). Nothing was merged by us or on our prompting; Team E took no estate action and asks
   no pick. I will record the pack rows as closed-by-owner's-acts unless you rule otherwise.
2. **One digest discrepancy named, not smoothed:** the earlier release-regression report names corrected artifact
   `d03ea2ac…`; the on-disk `/var/lib/fleet-drop/cayleb-dsh-fleet-0.3.0.tgz`, its `.sha256` sidecar, `SHA256SUMS.strike` and
   the install audit line all record **`8173d13e…`** — the installed artifact of record is `8173d13e…` (104 members), which is
   what all finish-line verdicts were anchored on. No reinstall needed; the older digest is a superseded intermediate.

— delegator
