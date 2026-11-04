# plugin-finish → Nox: the installer now has a suite, and three suites' rules are actually pinned

**2026-11-04T09:00Z** · addendum to `20261104T075000Z`. Nothing addressed to me since; no new order. P3/soak unchanged and
owner-gated (marker absent). `fix/lane-archive-heal` is now `f9e0ee2` (pushed); `origin/main` deliberately untouched.

## 1. The four commands the owner types had no automated test — its first run found a defect

`release/patch-lane-heal.sh` was only ever verified by my own hand rehearsal. Now `release/patch-lane-heal.test.sh` **29/29**
rehearses check → apply → finish → rollback on throwaway copies of today's live 0.3.6.1 state (a copy, never the live home; the
test skips with a reason if the live install is no longer 0.3.6.1, i.e. if the owner has applied it).

What it proved, in owner terms:

* `--check` writes **nothing** (tree hash identical before/after) and prints the plan;
* `--apply` lands exactly the three reviewed bytes, writes `applied_at` as UTC, bumps the version to 0.3.6.2;
* `--finish` **only removes lines** — 0 added, and the removed block is the 13-line bridge row, so the finished
  `cordis.patch.yml` is the pre-bridge file;
* `--rollback` restores the three files and `package.json` byte-identically, and a second `--rollback` is idempotent.

The defect: `--finish` deletes the bridge module, so `--rollback` **deliberately** does not put the row back (a row pointing at a
missing module would break the profile) — but the header promised "put the bridge back if it was removed" and the tool printed
neither. It now prints `bridge row: NOT restored (--finish removed it; …)` or `restored cordis.patch.yml (bridge row included)`.
Commit `f9e0ee2`.

## 2. Three suites pinned their rules only by mutations I ran by hand

`soak-check`, `soak-sample` and `connections-probe` had their mutations executed once, typed into the evidence and discarded:
nothing would fail if the rule was deleted tomorrow. Each now has in-suite mutation cases under the three conditions — mutant is
`chmod +x`, suite honours a subject-path override, and the failure must be **the case that names the rule** (as an ERE, because
the `/1000` parse mutant legitimately fails the epoch fixtures before reaching the ISO case). Counts: **25/25, 17/17, 7/7**.

Writing them found one more flake of tonight's signature family: the probe suite used fixed random ports, so its own mutation run
(a second copy of the suite) could collide, and a bind failure read as `000` — the test blaming the probe for the port clash it
created. Listeners bind port 0 now.

Named rather than quietly fixed: **`p3-watch` still has no in-suite mutations.** Its rules were mutated externally by two
independent checkers and by the parallel seat; adding cases to that file would make me a second writer on the surface
`CLAIM-p3-scripts-20261104T0552Z.md` gave to the other seat.

## 3. Numbers

Battery on the final bytes: `soak-check` 25 · `soak-sample` 17 · `soak-supervisor` 35 · `connections-probe` 7 · `watchdogs-arm` 29 ·
`p3-watch` 33 · `p3-run` 22 · `probe` 11 · `workspace-retire` 29 = **208**, plus `patch-lane-heal` 29 in the repo = **237 cases,
0 failures**. Roster suite 193/193 on `f9e0ee2`.

## 4. Owner

Unchanged: apply the patch, quit, relaunch, then `--finish`; P3 fires by itself. New: none of those commands is untested, and the
rollback now tells you what it did and did not undo. Evidence `evidence/TEST-NONVACUITY-20261104.md`,
`evidence/WATCHDOG-STRAY-UNITS.md`, `evidence/WORKSPACE-RETIRE.md`.
