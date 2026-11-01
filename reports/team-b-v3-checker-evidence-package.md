# V3 RUN + FORMAL STAGE 2 — evidence package for the fresh installed-output checker

Prepared 2026-11-01T18:12Z by `fleet-lead` (Team B, Lane 1) so the Lead can have it ready when he spawns the
independent seat. **Maker-side inventory only; the checker runs the unmodified check itself and issues its own
verdict.** No evidence is pre-judged here.

## 0. Checker eligibility and authority

- A **distinct fresh seat** that did **not** build or review the candidate, the walker change, the identity helper
  or the V3 procedure. The procedure reviewer (`de466966…`) and the identity-V3 reviewer are ineligible.
- The checker runs the **unmodified** `checks/stage2-check.sh` against the **installed output**, `FLEET_CLI` set,
  **one run at a time**, owns the run exclusively, and preserves full private logs.
- It must judge the **whole V3 private result**, not the exit code (see §3).

## 1. Candidate / release identity (frozen)

| Item | Value |
|---|---|
| Candidate commit / tree | `cf93107bc6e31ed265332b005d75b63d8acb300e` / `e15c17a3c93919de6b627e791a469dd9ddeefb28` |
| Package version / package.json sha | 0.2.2 / `ef93187aec289d70bd29446c66c5b63abe44f78f4d2342871af2338274b4f52c` |
| Accepted archive | `/var/lib/fleet-drop/cayleb-dsh-fleet-0.2.2.tgz`, sha256 `37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177`, 82 members |
| Package file list | `board/reports/team-b-package-file-list-0.2.2.txt` (82 archive paths) |
| Completeness decision | Nox `20261101T034100Z` (two-way derivation, 82 members, DECIDED COMPLETE) |
| Unmodified check script | `checks/stage2-check.sh` sha256 `7df6344c0846491e1130e9fb4c198aeed01de480b2ca47b03d9a5c3529fcca95`; CLI shim `8e121b19dbf82bcccc4521991df18386a788b83a936f32b1bf2d9036dff0d114` |
| Walker change review | `board/reports/fleet-checker-walker-review-ee14591.md` (SAFE and CORRECTLY SCOPED; three known limits carried) |
| Formal Stage 2 PASS (candidate) | `board/reports/fleet-checker-stage2-signoff.md` + `board/reports/fleet-checker-certification-cf93107.md` (rows 1–10, 14, 16, 19) |
| FINAL gate report + corrections | `board/reports/stage2-gate-report-FINAL.md` (§10 amendment register) |
| Repo state | `integration/0.2.2` pushed/remote-verified; `main` unmoved (no merge — Nox ruling pending) |

## 2. Install / wrapper fix (this release act)

- Wrapper fix package (approved by owner 184800Z): `board/reports/team-b-wrapper-fix-owner-review-package.md`
  (proposed wrapper `eb0ad024…`; installed wrapper hash to be recorded **after** the relay).
- Relay install block: `board/probes/team-b-v3-owner-relay-block-20261101.sh` (owner-executed; install only).
- Release act (executed by the operator/Lead after relay): stop → `dsh-fleet-admin plugin-add 37e3a542… cayleb-dsh-fleet-0.2.2.tgz` (normalising wrapper) → start; post-checks: installed wrapper hash, unit
  active/invocation, archive hash unchanged.
- Prior install record: `board/probes/team-b-identity-v3-run-record-20261101T1555Z.md` (first install + failed
  identity attempt; the tree state the checker will see is produced by THIS release act).

## 2b. Drop-in installation notes the checker must see (recorded 2026-11-01T18:20Z, before the run)

1. **Comment-only divergence from the approved §A text (not an anomaly).** The owner-executed relay block
   (`board/probes/team-b-v3-owner-relay-block-20261101.sh`, sha256
   `9ef5aedb28fdad9538bd2a81cfe15b471f73a714dd7c24a0117bb243635382b6`) installs the two approved drop-ins with
   **fewer comment lines** than the §A text in the command block doc (`88691743…`): sudoers carries 1 of the 2
   header comments, polkit 0 of the 2. After stripping comments and blank lines, the **rule bodies are
   byte-identical to the approved §A** —
   sudoers normalised sha256 `0b52b187625d0686c5ecece7ef4ce0f92b0cb2572fec69eb7a2d226ec3789d27`,
   polkit normalised sha256 `a3978674213b850a398d12cb1f1c7a1c951944f4effac27af3f118ebfbcb881c`.
   Verified by `validate-relay-block.sh` (heredoc-aware; sha `7e5d5739…`, result RELAY-BLOCK-CONFIRMED) and
   `visudo -cf` both before and after installation. **No rule, mode, path or option differs** — comments are inert;
   the checker should not read this as a post-hoc change.
2. **Install order (recoverable, nothing runs).** The relay installs the fixed wrapper first, then validates and
   installs the drop-ins. A hypothetical `visudo -cf` failure after the wrapper step would leave the fixed wrapper
   installed with no drop-ins: harmless and recoverable (re-run the drop-in half; nothing is executed by the relay,
   the service is untouched). `visudo -cf` runs before install and after install, plus a whole-config `visudo -c`.
3. **What this does not authorise:** the relay is install-only. No service action, no run, no V3 invocation, no
   credential handling.

## 3. The V3 run result (the identity chain the checker judges)

| Item | Value / path |
|---|---|
| Preserved result files | `~/Documents/Codex/2026-10-31/task/mission1/identity-cf93107-v3-procedure-summary.json` (result), `…-diagnostic.txt` (caller stderr), `…-observation.json` (caller disk observation), `…-bootstrap.err` |
| Window receipts | `~/workspace/harness-build/scratch-tb-l1/raw/` — window3-baseline / window-start / window-end receipts (v3 script `6c410a0e…`, runner `273fa8d3…`); hold declaration digest carried in each |
| Pinned procedure inputs | procedure `652d69f5…`, bootstrap doc `12997259…`, caller `556ad026…`, core `f211db39…`, manifest `6fa03597…`, archive `37e3a542…`, review `de466966…` |
| Acceptance rule (mandatory) | `stage_disposition == CREATED_STAGE_PRESERVED_OWNER_INSPECTION_REQUIRED`; `reason == null`; observation validated; caller/core/manifest/archive digests and all 82 file records match; fresh invocation; baseline≠after; the four host facts / loader-origin claim at the strength the record supports (any narrowed wording ruling from Nox recorded) |
| Four host facts + reviewer verdict | `board/probes/team-b-host-facts-20261101T1402Z.md`, addendum `…1505Z.md`, reviewer `board/reports/fleet-checker-identity-v3-review.md` |

## 3b. Attempt-2 evidence (collected 2026-11-01T19:03Z) and the noclobber correction

- **Executed run:** unit `v3-run-20261101T190246Z.service`, rc 0. Summary 607 B sha256 `9a45faeb…`
  (`outcome=CONDITIONAL_DISK_OBSERVATION`, `reason=null`, `observation_validated=true`,
  `restoration=NEW_INVOCATION_RECORDED`, `stage_disposition=CREATED_STAGE_PRESERVED_OWNER_INSPECTION_REQUIRED`,
  `formal_stage2=NOT_RUN`, baseline `6ac541f5…` → after `51c53101…`). Observation 12,163 B sha256 `ed75242f…`
  (target the fixed package root, root dev/inode/mnt `32/1839011/32`, 82/82 file records, archive `37e3a542…`,
  caller/core/manifest pinned). Diagnostic/err 0 B.
- **Collection:** `~/workspace/harness-build/scratch-tb-l1/v3-runs/run2-20261101T184036Z/run2-evidence/` (manifest
  `e7aea045…`); attempt-1 failed-run copies untouched in `failed-run1/` (manifest `1619cef5…`).
- **Window receipts v3:** baseline `48b86ce3…` (19:01:05Z), window-start `999f3ebd…` (19:01:06Z), window-end
  `f857da8e…` (19:03:29Z; invocation `51c53101…`, starts-in-window 1 / attributed 1 / unattributed 0, baseline
  proof `6ac541f5…`). Hold declaration sha `21d6b0fe…` embedded in all three.
- **Honest record — one bootstrap-level failure before the run:** unit `v3-run-20261101T190207Z` failed in 20 ms
  (`cannot overwrite existing file`) because the bootstrap runs `set -euC` (noclobber) and its `exec >`/`exec 2>`
  targets already existed. **No procedure code ran** (no stage, no service action, no output). The originals were
  removed (preserved copies verified) and the single procedure invocation then ran. Correction to this package's
  §2b/§3 wording: for any future attempt **all four preserved-workspace output filenames must be absent** before
  the bootstrap (the two `O_EXCL` outputs AND the two noclobber redirect targets).
## 3c. ANOMALY-3 reconciliation and settling close (recorded 2026-11-01T19:16Z, after the checker's PASS)

**ANOMALY 3 (failed-run1/ "mixed-vintage") — reconciled as two layers of ONE run, not a mixture.** This wording in
§3b (and any reading of §3's "attempt-1 copies") is corrected to: `failed-run1/` holds **one attempt's two-level
evidence**. Proof: (i) mtimes preserved by `cp -p` are within 0.314 s — bootstrap.err 10:46:52.695 ET, observation
10:46:52.722, diagnostic 10:46:52.986, summary 10:46:53.009 (summary closes last as the bootstrap's stdout
redirect flushes; observation 0 B because the caller refused before writing it); (ii) code path — the caller
raises `Blocked("package tree failed comparison: …")` (`PROPOSED-identity-caller-v2.py:223`) and prints the
BLOCKED JSON to **stderr** (`:254`), which is the diagnostic file, while the procedure's `_run` raises
`Blocked("fixed command … failed with exit 1; no alternate route")` (`PROPOSED-identity-procedure-v3.py:182-183`)
and the summary records that string — procedure-level outcome vs caller-level reason, same event; (iii) no other
run's files exist with those digests (attempt-2's outputs are the separate 607 B/12,163 B files with their own
manifest). **The executed run is untouched.**

**Settling window-close receipt (Stage 2 restart inside the hold interval):** 19:15:40Z, receipt
`5e4b87f3…` — invocation `b2178a05…` (Stage 2's own restart; ≠ V3 baseline/after), starts_in_window=2,
attributed=2, unattributed=0, baseline proof `6ac541f5…`, hold sha `21d6b0fe…` embedded. **Hold `21d6b0fe…` was
released by the Lead at 19:15Z** (declared endpoint reached); stage preserved; nothing written after release.

**Stage 2 evidence (fresh checker's own output):** `scratch-tb-l1/stage2-run-20261101T190916Z/` — `stage2-check.out`
sha256 `8d52347d…` (`PASS` lines 60, `FAIL` lines 0, `STAGE 2: PASS`), `EXIT.txt` = `EXIT=0`, `run-meta.txt`
(script `7df6344c…`, FLEET_CLI `8e121b19…`, pre-run invocation `51c53101…`), 13 MB of per-leg artefacts.
## 4. What the checker needs to be told (checklist)

1. Exact candidate/tree/archive above; verify the checkout at `cf93107`, script/shim hashes before running.
2. Read the whole V3 result + receipts + hold declaration; apply the mandatory acceptance rule; do not accept the
   exit code alone; do not retry or reorder anything.
3. Confirm the installed service is up from its permitted context **before** the check; confirm it is the post-
   release invocation; confirm the tree state measured by V3 is the tree the check exercises.
4. Run the unmodified Stage 2 check **once** in its own exclusive window with `FLEET_CLI` set and a private output
   directory; preserve full logs, exit code, cleanup and final service state.
5. Judge and report: per-leg PASS/FAIL/UNVERIFIED with the actual commands+results; name any failing leg; known
   limits carried (walker review §5, Stage 3 `3.4c` is Job-07, five-high dispositions per Nox 034100Z).
6. Do not merge, do not publish, do not edit logs, do not treat this package as a verdict.

## 5. Not in the package (deliberately)

- No maker verdict, no pre-baked PASS, no edited logs, no credentials, no service identity beyond what the
  permitted vantage shows. Nox retains the final Stage 2 / Mission-1 gate ruling.
