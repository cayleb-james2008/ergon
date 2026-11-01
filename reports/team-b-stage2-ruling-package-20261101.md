# STAGE 2 RULING PACKAGE — installed-output PASS + V3 disk-identity acceptance (for Nox)

Prepared 2026-11-01T19:18Z by `fleet-lead` (Team B) at the Lead's instruction, after the fresh independent
checker's verdict. **Maker-side consolidation only — Nox rules.** Verdict language is the checker's; nothing here
widens it.

## 1. Verdict being submitted for ruling

| Item | Verdict |
|---|---|
| **Formal Stage 2, installed output** | **PASS — 60/60 legs, exit 0**, run by a fresh independent checker (no history/build/review of this work) against the installed output, unmodified `checks/stage2-check.sh` (`7df6344c…`), `FLEET_CLI` per the evidence package, one run at a time, exclusive ownership. |
| **V3 identity result** | **ACCEPTED under the mandatory whole-result rule as `CONDITIONAL_DISK_OBSERVATION` — disk-identity strength only.** No wider claim (not retrospective memory proof, not proof all 82 files executed, not a Stage-2 verdict — that was this check). |
| **Narrowing ruling** | **Not required** for this acceptance: `loader_origin`, `runtime_identity`, `no_writer` are `NOT_ESTABLISHED` **by design** in the reviewed procedure (summary lines :403-404), and the checker judged the whole private result on that basis. Fact-2 policy question remains open only for any future, wider claim. |
| **Wrapper fix** | Installed (`eb0ad024…`, byte-identical to the reviewed proposed wrapper) and exercised: the release act's `plugin-add` ran the normalisation step on the real install (`normalise: 82 files single-link with archive modes; package dirs 0750`). |

## 2. Per-leg summary (from the checker's own output — not re-derived by the maker)

- `stage2-run-20261101T190916Z/stage2-check.out` sha256 `8d52347d…`: **`PASS` lines = 60, `FAIL` lines = 0,
  `summary: pass=60 fail=0`, `STAGE 2: PASS`**; trap: `cancelled 70 of 1082 fixture orders (append-only rows remain
  by design)`.
- `EXIT.txt` = `EXIT=0` (`418a5c17…`); `run-meta.txt` (`1b10cce2…`) records: fresh checker uid 1000,
  `script=checks/stage2-check.sh sha=7df6344c…`, `FLEET_CLI=…/package/bin/dsh-fleet.mjs sha=8e121b19…`,
  pre-run invocation `51c53101…` / MainPID 305549.
- Script hash **identical before and after** (`7df6344c…`); CLI shim **byte-identical to installed
  `bin/dsh-fleet.mjs`** (`8e121b19…`); all named hashes re-verified by the checker; **82 records re-derived**
  against the accepted archive (`37e3a542…`) and manifest (`6fa03597…`).
- The check's own 2.11 restart produced invocation `b2178a05…` (MainPID 313468); the V3 window is undisturbed —
  its `after` `51c53101…` ended after the 19:03:29Z window-end receipt.
- `loader_origin` / `runtime_identity` / `no_writer` = `NOT_ESTABLISHED` by design (:403-404) — no narrowing
  ruling needed.

## 3. Evidence paths

| Evidence | Path / digest |
|---|---|
| Stage 2 run (checker's private output, 13 MB, per-leg artefacts) | `~/workspace/harness-build/scratch-tb-l1/stage2-run-20261101T190916Z/` — out `8d52347d…`, `EXIT.txt` `418a5c17…`, meta `1b10cce2…` |
| V3 run-2 private result (collected) | `scratch-tb-l1/v3-runs/run2-20261101T184036Z/run2-evidence/` — manifest `e7aea045…` (summary `9a45faeb…`, observation `ed75242f…`, diagnostic/err `e3b0c442…`) |
| Window receipts (v3) | baseline `48b86ce3…` (19:01:05Z) · window-start `999f3ebd…` (19:01:06Z) · window-end `f857da8e…` (19:03:29Z; invocation `51c53101…`) · settling window-close `5e4b87f3…` (19:15:40Z; invocation `b2178a05…`, starts-in-window 2 / attributed 2 / unattributed 0, hold sha embedded) |
| Hold | declaration sha `21d6b0fe…`; **released by the Lead at 19:15Z** (declared endpoint — Stage 2 final restart — reached). Recorded; hashed file unchanged. |
| Run record / checker package | `board/probes/team-b-identity-v3-run-record-20261101T1555Z.md` (`f062a6ec…`); `board/reports/team-b-v3-checker-evidence-package.md` (`60b9030b…`) |
| Preserved stage (NEW, from the successful run) | `/run/dsh-fleet-id-cf93107` — three root 0444 files, mtimes 14:02 ET, pins `556ad026…`/`f211db39…`/`6fa03597…`; disposition `CREATED_STAGE_PRESERVED_OWNER_INSPECTION_REQUIRED`; clears on reboot, earlier removal = separate owner act |

## 4. Anomalies — all disclosed, all reconciled, none material

1. **Drop-in comment-only divergence (inert, disclosed before the run).** Relay block `9ef5aedb…` installed the
   approved drop-ins with fewer comment lines than the §A text; after stripping comments/blanks the **rule bodies
   are byte-identical** to the approved §A (normalised sudoers `0b52b187…`, polkit `a3978674…`), `visudo -cf`
   clean before/after. Recorded in the checker package §2b pre-run.
2. **Install-order note (inert).** Fixed wrapper lands before drop-in validation; a hypothetical visudo failure
   would leave the fixed wrapper with no drop-ins — recoverable, nothing runs. Recorded pre-run.
3. **Noclobber bootstrap failure (no execution).** Lead's first bootstrap attempt (`v3-run-20261101T190207Z`,
   20 ms, rc 1) failed on `set -euC` redirects (`cannot overwrite existing file`); **no procedure code ran** (no
   stage, no service action, no output). Originals removed (preserved copies verified); one procedure invocation
   then ran. Correction recorded: all four output filenames must be absent before a bootstrap. This is not a retry
   of a procedure attempt.
4. **Checker ANOMALY 3 — `failed-run1/` "mixed-vintage": PROVEN CONSISTENT, same run two layers.** Summary
   `0635d280…` records the procedure-level outcome (fixed caller command exited 1, "no alternate route"); diagnostic
   `e56fea39…` records the caller-level reason ("package tree failed comparison: full file mode mismatch").
   Evidence: mtimes within 0.314 s at 10:46:52.695–53.009 ET (preserved by `cp -p`; summary closes last as the
   bootstrap's stdout flushes; observation 0 B = caller refused before writing it); code paths
   `PROPOSED-identity-caller-v2.py:223,254` (raises `package tree failed comparison…`, prints BLOCKED JSON to
   stderr = diagnostic) vs `PROPOSED-identity-procedure-v3.py:182-183` (raises `fixed command … failed with exit
   …; no alternate route` = summary reason); no other run shares those digests. Correction to the package wording
   recorded (§3c): one attempt's two-level evidence, **not** a mixture of runs.

## 5. Named limits carried into the ruling (as written)

- **Disk-identity strength only** — no retrospective process-memory proof, no proof that all 82 files executed, no
  claim about later dynamic loads; `loader_origin` / `runtime_identity` / `no_writer` NOT_ESTABLISHED by design.
- Walker-change known limits (B5 review): aggregate envelope no longer walker-asserted (replacement = composed
  spend regressions); pinned R6-10 test is a fixture-path guard, not a product regression; cite the review file.
- Stage 3 `3.4c` journald leg remains a **Job-07** item; five-high dispositions are as Nox ruled (034100Z); the
  desktop composed-host coverage boundary is named, not a defect.
- **No merge to `main`** — Nox's ruling; `cf93107` remote-verified, `abfb842` excluded.
- Preserved V3 stage remains until reboot or an owner-authorised removal.

## 6. What Nox is asked to rule

1. **Record the formal Stage 2 PASS** on the installed output (checker verdict above), with the disk-identity
   condition of the V3 acceptance and the named limits carried.
2. Confirm no narrowing ruling is required for this acceptance (fact-2 policy stays open only for future wider
   claims).
3. Release the remaining Mission-1 closure items: the **merge ruling** for `cf93107` into `main` (release owner's
   step, no force-push) and the stage disposition as recorded.
