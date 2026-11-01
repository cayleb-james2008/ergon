# TEAM E / LANE 6 — BATCH 2 REPORT: Paperclip/Pantheon preservation + outis-pilot stop plan + rulings recorded
**Date:** 2026-11-01 09:22–09:36 ET · **Seat:** upkeep-lead · **Scope:** Lead rulings 1–10 (batch-2 authorised work)
**Status:** preservation **complete and restore-verified**; **removal NOT started** — waiting on the fresh checker the Lead spawns from this receipt. Source tree untouched (service still disabled/inactive, no process). outis-pilot stop plan delivered. All rulings recorded.

## 1. Paperclip/Pantheon preservation — COMPLETE, VERIFIED
Preservation copy (NOT the wiped removal archive):
`/home/cayleb/Work/_archive/lane6-preservation-20261101/paperclip-pantheon/` — **7.0 GB**
Board receipt (checker input): `board/probes/team-e-lane6/paperclip-preservation/RECEIPT.md` + `SHA256SUMS.txt` + `verify-restore.log` + manifests (symlinks/units/worktrees/tracked-changes).

| artifact | count / size | verification |
|---|---|---|
| `bundles/*.bundle` — `git bundle --all` per repo | 5 (253 MB; zeus 5.3K, muse 69M, asteria 138M, helikon 47M, olympus 360K) | `git bundle verify` **OK all 5**; fresh clone → **refs+SHA match exactly**: 25/20/21/7/7 branches, commits 1/52/259/12/5 all equal |
| `wip/<repo>/<wt>/tracked.patch` (`git diff HEAD --binary`) | 8 patches | each `git apply --check` on its recorded HEAD **PASS 8/8** |
| `wip/<repo>/<wt>/untracked.tar.zst` (untracked, non-ignored) | 11 archives, 6.8 GB (asteria main 1,938 files/4.6G; PAN-175 6,577/2.0G; helikon 142M; muse 68M; zeus 37M; olympus 552K; 5 muse task wts 48K ea) | `zstd -t` **OK 11/11, 0 corrupt** |
| `skills/paperclip-skills-instance.tar.zst` (payload behind the 13 Claude skill links) | 140 KB | `zstd -t` OK |
| `manifests/` | 80 worktrees (branch+HEAD sha+dirty counts), 16 symlinks (13 skills + 3 `~/.local/bin`), 8 unit entries + unit copies | enumerated |
| `SHA256SUMS.txt`(+.sha256) | sha256 of all **25** primary artifacts | checker re-runs `sha256sum -c` at the archive |

**Source untouched:** `~/.local/share/paperclip-pantheon` (16 GB) and `paperclip-studio-lane` (3.0 GB) still in place; `paperclip-pantheon.service` disabled/inactive; all 8 dependent units disabled/static; no process, no listener.

**Pre-removal plan in the receipt (for checker):** move (never delete) the two trees to a lane-6 retirement archive → remove the 16 dangling symlinks only after the skills archive is confirmed → `daemon-reload`. All repo history lives in the bundles first. Credential-bearing content (`instance/`, `private/`, `state/`, studio-lane `prefix/`) stays credential-blind — path-only handling.

**Discrepancies recorded, not normalised:** inventory 30 Oct said **22.1 GB**, measured today **19 GB** (16 G + 3.0 G; 3.1 GB delta unexplained); inventory said *six* dirty worktrees, today **11** worktrees carry changes (8 tracked / 11 untracked) — today's state preserved.

## 2. outis-pilot — stop plan + dependency check (NO ACTION; for Nox)
`board/probes/team-e-lane6/outis-pilot-stop-plan.md`
- `outis-control` 127.0.0.1:8787 (PID 1488) + `outis-mesh` MeshCentral 127.0.0.1:8788 (PIDs 1489→1764→1839): **loopback-only, no established clients, no dependents found** (grep of user+system units, PATH, symlinks). Last log activity 2026-10-13. 1.3 GB; credential-bearing paths listed by name only (`provider.env`, `anthropic-proxy.json`, enrollment certs/keys) — never read.
- Plan: freeze evidence → stop mesh → stop control → disable both → archive as a *move* only after a Nox ruling → verify no listeners/PIDs; reversal = enable+start; **no deletes, no credential reads, do not revoke the remote `laptop-pilot` enrollment**.

## 3. Rulings recorded (no action where marked)
`board/probes/team-e-lane6/rulings-and-records-20261101.md`: Codex extension-host :9222 = **needs-ruling, trading desk's CDP route, not touched**; herdr = needs-root; Talos = stay disabled, wallet-adjacent, needs-ruling; Muse cache = 12 KB vs 300 MB discrepancy recorded, no action; `/dev/kvm` 0666→0660 = root change, needs-ruling, not applied; Grok Bot keep / browser-use deferred / `~/.winvm-scratch` keep; `outisz-x-campaign` fence accepted (reversible, re-enable command recorded); GitHub cadence = Lead-owned, Lane 6 executes sweeps only; Outis freeze mail = adjudicated, not an order.

## 4. Checker handoff (what the Lead's fresh checker must do)
1. `cd /home/cayleb/Work/_archive/lane6-preservation-20261101/paperclip-pantheon && sha256sum -c SHA256SUMS.txt` → all OK.
2. Re-run one restore per repo from its bundle and compare refs (the receipt gives the commands) — not just re-read this report.
3. Confirm `git apply --check` on a sample of patches and `zstd -t` on the untracked archives.
4. Confirm source tree still present and services still disabled (i.e. removal has not happened yet).
5. Only then: PASS → removal starts under the plan in §1; FAIL → back to Lane 6 with the failing leg named.

## 5. Successors
- **Lead:** spawn the preservation checker from `RECEIPT.md`; on PASS authorise the move-removal (my next batch), on FAIL return the failing leg.
- OpenCode timer migration waits on the fleet-provider clearance marker (fence still BLOCKED with 7 clearances).
- outis-pilot, herdr, Talos, Muse, :9222, /dev/kvm remain needs-ruling; nothing else on Lane 6 is executable without a ruling.
- Disk after preservation: **587 GB free** (7.0 GB archive added; source still on disk until removal).

## 6. Limits
No checker has run on this batch — nothing marked DONE. No credential contents read, no deletion, no service change, no spend, no public act. All numbers are measured by the commands quoted in the logs; the only unverified item remains `/var/lib/fleet` (unreadable, no sudo).

## 7. Addendum — removal script ready (dry-run verified 14:35:26Z)
`board/probes/team-e-lane6/paperclip-preservation/remove-paperclip.sh`: dry-run default; `--apply` requires `CHECKER-PASS.md` in the same directory. Re-hashes the preservation artifacts first (PASS), aborts if a paperclip process/active unit appears, moves both trees to `Work/_archive/lane6-retirement-20261101/` (never deletes), unlinks the 16 archived symlinks, `daemon-reload`, writes `REMOVAL-RECORD.txt` and post-verification. Dry-run touched nothing (19 GB source intact, service inactive).
