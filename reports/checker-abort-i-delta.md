# CHECKER — abort-contact **(i)** option-B leg-start revert + **fix C** delta (`1eed7e7`) — verdict: **ACCEPT**

Fresh-context independent check (same active checker seat; I built none of it). All execution from a **fresh copy of the
commit**, never the maker's tree. **No Play, no Studio input, no image reads, no build, $0.** Report to `lead` only.

- **Checked commit:** `1eed7e776d6d2484371b72e6632eb7129cdbee7d` (tree `8abc65e560d2f7f2e7f93d9bf2ff40df8b6e2594`),
  parent `746a99ed16b46a6ab6f1c45b71394b01a782bdf1`; **4 files, +71/−104**; tracked tree clean; branch tip is the later
  delta `e8d18f4` (1eed7e7 is its parent).
- **Fresh copy:** `/tmp/checker-d1` from `git archive 1eed7e7` — 1443 tracked entries, **blob-for-blob equal** (0
  missing / 0 mismatch); `build/` untracked → sealed artifacts copied read-only (`c6651793…`/`84324d43…`).
- Maker report exists as `board/reports/team-d-abort-contact-fixc-freeze-20261102.md` (the brief's expected
  `…-abort-contact-i-freeze…` name does not exist; content read and consistent). Log:
  `board/probes/team-d/abort-contact-fixc-freeze-20261102.log`.

## Verdict in one line

**ACCEPT** — the B-bounded gap-trigger/bounded-pre-leg experiment is **fully reverted** (leg starts at
`abortObservedAt`, no ≤25 trigger, no pre-leg timeout, no retired tokens left in code; retirement recorded in docs only),
**fix C** is applied to the pursuit alone (`-38,10` → `-38,40`) with the total 40 cap, lateral term, obstacle law,
closing clamp, 12 s window and both named guards **byte-unchanged** (function-block shas), all bounded suites reproduce,
and both required mutations fail.

## Per-leg table

| # | Leg | Verdict | Evidence |
|---|---|---|---|
| 1 | **B-bounded fully reverted**: leg starts at `abortObservedAt`; no ≤25 trigger; no pre-leg timeout; retired-experiment record kept in docs/report only | **PASS** | source: `if cfg.mode=="abort-contact" and abortAt then contactAt=contactAt or age;…` present; retired tokens all **0** occurrences (`CONTACT_GAP`, `PRELEG_TIMEOUT`, `probeGap`, `contactLegTrigger`, `contactGapTrigger`, `preLegTimeout`, `never converged`, `chosen+70`); option-B geometry restored (`chosen+80`, `preLegSpeed=30`, no `-18`); runbook paragraph records the retirement (no code) |
| 2 | **C applied** to `Server.pursuitAcceleration`: forward clamp `-38,10` → `-38,40`; total force cap **40 unchanged**; lateral/obstacle law unchanged (`obstacleAcceleration` still 10); closing clamp `clamp(proxySpeed+8,12,60)`; 12 s window + both guards byte-unchanged | **PASS** | `-38,40` ×1 in pursuit, `-38,10` ×2 (obstacle + the other law), force-cap line ×2, identical lateral term ×2; **function-block shas (parent→delta):** `obstacleAcceleration` `3807dd8d…` **UNCHANGED**, `contactLegGuard` `72c14254…` **UNCHANGED** (radius + 12 s timer), contact branch (closing clamp + guard call) `ac02c6d1…` **UNCHANGED**, `pursuitAcceleration` `ce4117…`→`872e4c…` **CHANGED** (only the C clamp) |
| 3 | **Tests**: forward accel reaches 40; total cap still 40; lateral/obstacle unchanged; cap count; kinematic regression clamp 40 < 12 s vs clamp 10 > 12 s | **PASS** | Lune `traffic-contact-pursuit` (18 specs total 94/0) now loads **both** helpers: pursuit magnitude 40, obstacle 10, total cap 40; Python source pins (`-38,40`, `-38,10`, cap count 2, lateral ×2, closing/guards); kinematic model (gap 71.4, ego 30, proxy 52): **clamp 40 → 9.23 s**, **clamp 10 → 13.45 s** (the brief's "~19 s" is not the frozen model's figure; the required direction — 40 closes inside 12 s, 10 does not — holds) |
| 4 | **Mutations**: M22 (revert to 10) must FAIL; a second mutation proving the gap trigger is really gone must FAIL | **PASS** | M22: `test_traffic_acceptance` **FAILED (1)** + Lune pursuit spec **3 passed / 1 failed**; M29 (re-add `CONTACT_GAP`/`PRELEG_TIMEOUT` + trigger/timeout block): `test_traffic_acceptance` **FAILED (1)**. Both restored, mut-tree identical to the fresh copy (only `__pycache__`) |
| 5 | **Suites** | **PASS** | runner **91 OK** (12.9 s); traffic acceptance **10 OK**; full Python **331 OK** (47.2 s); Lune traffic **94 passed / 0 failed (18 specs)**; full Lune **924 passed / 0 failed (127 specs)** — re-ran at the tip (delta 2 changes only docs + a Python test file, so the Lune inputs are identical on `1eed7e7`); sealed unchanged after every run |
| 6 | **Delta scope** file-by-file; no production/game code; sealed untouched | **PASS** | `git diff --numstat 746a99e 1eed7e7` = exactly the 4 disclosed files (`Server.luau`, traffic acceptance, Lune pursuit spec, runbook); no `src/`, `assets/`, `build/`, `deploy.json`; sealed `84324d43…`/`c6651793…`/Studio place `c6651793…` unchanged |

## Notes / limits

- **No Play / no Studio input**: the real leg start, contact timing and guard behaviour remain Play-verified; this delta
  certifies the fixture code, its pins and the retirement of B-bounded only. No capture claim.
- The maker report's own numbers match my runs (10 OK / 91 OK / 331 OK / 94-0 / 924-0; M22 and M29 caught).
- The brief's "clamp 10 ~19 s" is not the committed model's value (**13.45 s**); the frozen assertion is "old > 12 s" and
  the regression direction is correct — named, non-material.
- Report-name deviation: `team-d-abort-contact-fixc-freeze-20261102.md` (not `…-abort-contact-i-freeze…`) — content
  present and consistent; non-material.

## Evidence (`board/probes/checker-abort-i/`)

`provenance-20261102.txt`, `suites-20261102.txt`, `blocks-20261102.txt` (function-block shas, retired/present token
counts, kinematic), `mutations-20261102.txt`. Fresh copy `/tmp/checker-d1`; mutation copy `/tmp/checker-d1/mut`
(restored).

---

**— checker-trading (fresh-context, active checker seat), 2026-11-02 03:22 ET (EST, UTC−5) = 2026-11-02 08:22Z.**
**Verdict: ACCEPT** for `1eed7e776d6d2484371b72e6632eb7129cdbee7d`.
