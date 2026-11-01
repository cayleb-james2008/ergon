# TEAM E / LANE 6 — BATCH 3 REPORT: Paperclip/Pantheon move-removal EXECUTED (post-checker)
**Date:** 2026-11-01 10:00–10:03 ET · **Seat:** upkeep-lead · **Authorisation:** checker PASS (`board/reports/team-e-paperclip-preservation-checker-verdict.md`) + Lead explicit go (09:59 ET) + three pre-actions
**Status:** move-removal done, never deleted; all pre- and post-checks pass; nothing marked DONE beyond the checker's own PASS (this was execution of a checker-approved plan).

## 1. Pre-actions required and completed before `--apply`
1. **D7 marker path fixed:** `CHECKER-PASS.md` written at `$PRES/CHECKER-PASS.md` (`/home/cayleb/Work/_archive/lane6-preservation-20261101/paperclip-pantheon/CHECKER-PASS.md`) — exactly the path `remove-paperclip.sh` tests; receipt §7 correction appended as §8.
2. **D1 `~/.paperclip` decision recorded** (`DECISIONS.md`): **LEAVE IN PLACE — 3,250,983,090 B (3.1 GiB), 84 `paperclip-*.sql.gz` DB dumps 13–18 Oct**, outside both moved trees; checker recommended, Lead concurred. D1 explanation accepted: 22.1 GB inventory = 19.28 GB trees + 3.25 GB `~/.paperclip` = 22.53 GB.
3. **D8 process gate fixed in the script:** `pgrep` replaced by a `/proc` `cwd`/`exe`/`root`/`cmdline` scan; gate output before apply: **"0 consumers"** (own shell excluded), `PASS no live consumer`, `PASS service inactive`.
Also recorded checker corrections D2–D6 in receipt §8 (stored 6.75 GiB vs raw 7.73 GiB; "files" = git list entries; main archives include nested worktrees; unit copies 1/8; skills tar 11/13 links with the 2 runtime-based ones preserved by the move).

## 2. Execution (10:01:47 ET / 15:01:47Z, `remove-paperclip.sh --apply`)
Order: re-hash preservation (25/25 PASS) → source preconditions (16,185,660,895 + 3,094,249,713 B) → `/proc` gate 0 consumers → service inactive → `~/.paperclip` reported kept → **move** (not delete) both trees to `/home/cayleb/Work/_archive/lane6-retirement-20261101/` → unlink 16 archived symlinks (3 `~/.local/bin` + 13 `~/.claude/skills`) → `daemon-reload` → write `REMOVAL-RECORD.txt` → verify.

## 3. Post-apply verification (all run after the move)
| check | result |
|---|---|
| trees under retirement archive, byte counts **identical** to pre-move | `16185660895` / `3094249713` — exact match |
| `REMOVAL-RECORD.txt` written | yes — includes authorisation, marker, bytes, symlink count, kept-in-place decision, reversal instructions |
| 16 symlinks | **gone=16, still_present=0** |
| `~/.paperclip` untouched | 3.1 G, 84 dumps present |
| preservation archive integrity after the move | `sha256sum -c` **PASS 25/25** (move did not touch it) |
| units | paperclip-pantheon disabled/inactive; 7 dependent units static/inactive; `forge-local-worker.timer` active but **0 refs** to the tree (checker-verified) |
| live consumers of old paths | 0 (the single `/proc` cmdline hit is this verification shell's own command string) |
| desk/fleet spot-check | chrome-bridge, openbell-kalshi-poll, outis-control, fleet-dsh all **active** — untouched |
| disk | 586 G free |

## 4. Estate state now
- Removed-from-service (moved, reversible): `~/.local/share/paperclip-pantheon` 16.19 GB + `paperclip-studio-lane` 3.09 GB → `/home/cayleb/Work/_archive/lane6-retirement-20261101/` (that dir also holds the earlier dsh-tauri removal 547 MB).
- Preservation copy (checker-passed, untouched by the move): `/home/cayleb/Work/_archive/lane6-preservation-20261101/paperclip-pantheon/` (7.0 GiB + `CHECKER-PASS.md`).
- Reversal: `mv` both trees back, then relink from `manifests/symlinks.tsv`; skills payload archive covers 11/13, the 2 runtime-based skills return with the tree (never were deleted).
- 16 dangling symlinks cleaned from `~/.local/bin` and `~/.claude/skills`; unit files remain in `~/.config/systemd/user/` (disabled/static, untouched).
- `~/.paperclip` 3.1 GB stays by named decision.

## 5. Successors
- OpenCode timer migration still blocked on the Lead's fleet-provider clearance marker (fence gate BLOCKED, 7 clearances).
- Needs-ruling and parked: outis-pilot (Nox), herdr (root), Talos (wallet-adjacent), Muse cache, Codex extension-host :9222, `/dev/kvm` 0660.
- Nothing else executable on Lane 6 without a ruling.
## 6. Limits
No credential file read; no deletion of data; no service started/stopped except `daemon-reload` (config reload only). Moved bytes verified equal. The checker's PASS remains the certification for preservation; this report only records the execution and its post-checks.

## 7. DISPOSITION DONE (Lead post-check PASS, 10:18 ET)
The Lead's independent post-check ran all 8 legs and **PASSED** the paperclip removal: byte-exact moves (16,185,660,895 / 3,094,249,713, delta 0, same-device rename, inodes preserved), reversible, nothing lost — 16/16 symlinks gone with 0 dangling, `~/.paperclip` untouched (3,250,983,090 B / 84 dumps), preservation archive 25/25 re-hashed clean, services clean, 0 consumers. **This disposition is DONE by the maker-never-checker rule.**
**Residual launcher archived (authorised):** `~/.local/share/applications/paperclip-pantheon.desktop` was a dead launcher pointing into the moved tree (outside the 16-link scope). Moved at 2026-11-01T15:23:30Z to `/home/cayleb/Work/_archive/lane6-retirement-20261101/paperclip-pantheon.desktop`; `update-desktop-database` re-run; verified absent from `~/.local/share/applications/`; a scan showed **no other** .desktop entries referencing the moved trees. Restore line appended to `/home/cayleb/Work/_archive/lane6-retirement-20261101/REMOVAL-RECORD.txt`: `mv /home/cayleb/Work/_archive/lane6-retirement-20261101/paperclip-pantheon.desktop ~/.local/share/applications/ && update-desktop-database ~/.local/share/applications`.
**Successor:** consolidated needs-ruling dossier for Nox at `board/reports/team-e-needs-ruling-dossier-20261101.md` (N1 outis-pilot, N2 herdr, N3 Talos dead units, N4 Muse cache, N5 Codex extension-host :9222, N6 /dev/kvm 0660). OpenCode timer migration remains blocked on the fleet-provider clearance marker.
