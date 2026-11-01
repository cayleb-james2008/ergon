# IDENTITY V3 — RUN RECORD, FAILURE DIAGNOSIS AND RULING RECOMMENDATION (Lane 1)

Filed 2026-11-01T15:57Z by `fleet-lead` (Team B, Lane 1). Digest level. **The one-time approval is CONSUMED; the
run was executed once, fail-closed, and must not be retried.** No service action was taken by Team B; all facts
below are from the preserved result, the valid window receipts, and a no-privilege reproduction.

## 1. Verdict

**Installed-identity: FAILED (fail-closed) for this install.** The procedure executed exactly once
(15:46:23–15:47:40Z), reached the caller, and the caller **refused**: the installed package tree does not match
the accepted archive under the candidate's exact-mode contract. Identity is **not** established, the formal
Stage 2 check **did not run**, and **no acceptance is claimed**. This is a genuine installer-contract finding,
not a runner failure: fail-closed behaviour worked as designed.

## 2. Run facts (receipt digests reproduced and verified against the Lead's numbers)

| Item | Value |
|---|---|
| Baseline receipt | `af2f99fc9536dfa4a8ea424b45724d6cd04171343791c0c366b15ce683a7fd10` (invocation `2ff92d0b…`, NRestarts 0) |
| Window-start receipt | `3843b35474df4416265ba3dcd0573a46bb361768cec967bd5439a62ddb72c08d` (hold sha `56016059…`) |
| Window-end receipt | `311d476066c443e044c158bba262eb105daf7da78fe0f93d5aea84d813d18ca7` (post invocation `e44cc343…`, NRestarts 0; hold sha `56016059…`; procedure markers baseline `2ff92d0b…` → after `e44cc343…`) |
| Result summary (preserved, `identity-cf93107-v3-procedure-summary.json`, sha256 `0635d280…`) | `outcome=BLOCKED`, `reason="fixed command /usr/bin/sudo --user fleet failed with exit 1; no alternate route"`, `restoration=RESTORED_ONCE`, `stage_disposition=CREATED_STAGE_PRESERVED_OWNER_INSPECTION_REQUIRED`, `observation_validated=false`, `loader_origin/no_writer/runtime_identity=NOT_ESTABLISHED`, `formal_stage2=NOT_RUN` |
| Caller diagnostic (panorama, sha256 `e56fea39…`) | `{"outcome":"BLOCKED","reason":"package tree failed comparison: full file mode mismatch"}` |
| Observation file | 0 B (caller failed before writing it); bootstrap.err 0 B |
| Service restored | invocation `e44cc343…` active/running, `NRestarts=0`; hold interval closed by window-end receipt |

## 3. Preserved stage — inspection (owner-authorised disposition still his)

`/run/dsh-fleet-id-cf93107` exists, root-owned 0755 with three root-owned 0444 files; their digests **match the
pinned values exactly** (caller `556ad026…`, core `f211db39…`, manifest `6fa03597…`). Contents are the reviewed,
non-secret source already on the record. Inspection therefore finds **no unexpected content**; the stage adds
nothing after this record. Recommendation: a one-line owner-authorised removal (not by us; nobody rm -rf's an
unconfirmed stage) — record retained here and in `scratch-tb-l1/raw/v3-run/stage-digests.txt`.
**Disposition either way:** if the owner removes it, record "stage removed by owner <time>"; if he does not
act, the stage remains in `/run` (tmpfs) and clears on reboot — that is not a security regression (the three
0444 files are reviewed source whose digests are recorded above, no secrets) but it is recorded as
"stage left to tmpfs clearing" rather than silently dropped.

## 4. Failure diagnosis — ROOT CAUSE FOUND AND REPRODUCED

The caller requires each installed file's mode to equal the accepted archive's tar member mode exactly
(`identity_snapshot_core.py:169`; ownership is checked first and passed, the failure is the **mode** leg).
The authorised install path runs the fixed wrapper, whose code sets **`umask 027`**; the sanctioned installer
then creates the installed files masked by that umask.

**No-privilege reproduction (same sanctioned `dsh plugin add`, same accepted tarball, same DSH/pnpm):**

| Installer umask | Files compared | Mode diffs vs archive |
|---|---|---|
| **022** (Lane 2 scratch acceptance) | 82 | **0** |
| **027** (the wrapper's umask; same `dsh plugin add`) | 82 | **58** — `0644 → 0640` and `0755 → 0750` (both execs), installed package root `0750` |

So the installed tree's modes are deterministically `archive_mode & ~0o027`. This explains the mismatch with no
protected read and no retry; it is a property of the **documented installer path**, not of the package bytes.
The exact byte comparison never completed because modes are checked before/with content.

## 5. Ruling recommendation (to the Lead; Nox rules policy)

**(a) — record FAILED, do not accept a post-hoc narrowing of this run, carry the finding into the release work,
and require a fresh approval for any second attempt.** Concretely:

1. **Record** identity FAILED for installed 0.2.2; Mission 1 stays open; Stage 2 installed-output check un-run.
2. **Repair choice (release work, not this lane's to execute):**
   - **Preferred: installer fix** — make the authorised install reproduce archive modes (e.g. wrapper installs
     the plugin with umask 022, or a fixed post-add mode normalisation to the archive's 82 member modes while
     keeping directory modes 0750). Then the original strong claim (archive-identical modes) survives unchanged
     and only the operation needs re-approval.
   - **Alternative: contract amendment** — a reviewed v4 procedure comparing against documented
     install-normalised modes (`archive_mode & ~umask`), with the residual named and the independent checker
     ruling the installer contract. This weakens the claim and must be an explicit Nox decision, not a
     quiet post-hoc fix.
   - **Do not** run option (b) on the current install: the reviewer's condition says the checker must assess the
     installer contract; narrowing it after a fail is retro-fitting.
3. **Second run: FRESH OWNER APPROVAL REQUIRED.** The approval was for **one invocation** of procedure
   `652d69f5…`; it is consumed. A second attempt is a new operation — and after either repair it involves
   changed reviewed bytes (v4) and/or a new install act. A fresh approval, a fresh independent review where the
   procedure changes, and a new operator relay (sudoers/polkit re-install) are all required. The current
   sudoers/polkit removal is correct; nothing is left installed.
4. **FACT 2** is moot for this consumed attempt (the run never reached acceptance); it stays open with Nox only
   for any future claim wording.
5. **Preserved stage:** inspection complete at digest level (contents match pinned); recommend owner-authorised
   removal in one line, after which no privileged artifact remains.

## 6. What Team B can do next without any new approval (no service action)

- Prove the installer-fix hypothesis in scratch: umask 022 install → 82/82 archive-identical modes (already
  shown once); add the wrapper-shaped variant (umask 022 + dirs 0750) and record it as the proposed fix's
  evidence for review.
- Draft the v4 diff (if Nox prefers the contract-amendment path) as a proposal only — not staged, not run.
- Continue the corrected FINAL/INDEX/F11 pack and the sessions' other lanes.

**Bottom line: the fail-closed result is a real, diagnosed installer-contract finding (umask 027 mode
normalisation); identity is failed for this install; no retry under the consumed approval; a second run needs a
fresh owner approval after the repair/policy decision.**

## 7. Stage disposition (recorded 2026-11-01T17:55Z, per Nox/Lead)

The preserved stage `/run/dsh-fleet-id-cf93107` is recorded as **left to tmpfs clearing on reboot** — three
root-owned 0444 files whose digests match the pins (`556ad026…`, `f211db39…`, `6fa03597…`; recorded in
`scratch-tb-l1/raw/v3-run/stage-digests.txt`), no secrets. It is not removed by Team B; the owner may direct
earlier removal as a separate owner-authorised act. This satisfies the "record either way" rule.

## 8. Attempt 2 — pre-run preservation and invocation contract (2026-11-01T18:40Z)

- **Bootstrap harness verified:** `scratch-tb-l1/v3-bootstrap.sh` sha256 `95868da0…` is **byte-identical** to the
  reviewed bootstrap block in `IDENTITY-CALLER-PROCEDURE-V3.md` (19 lines, same sha); it gates the procedure source
  as a single-link regular file with sha256 `652d69f5…` (verified on disk: nlink=1, mode 644) before
  `exec(compile(...))`, runs with the exact `env -i PATH=/usr/bin:/bin LC_ALL=C` + `-I -S -B`, and redirects
  stdout/stderr to the fixed preserved-workspace filenames. Invocation runs in an NNP-free transient user unit.
- **Run-1 evidence preserved BEFORE the attempt** (the bootstrap overwrites the fixed filenames): new run dir
  `scratch-tb-l1/v3-runs/run2-20261101T184036Z/failed-run1/` holds the consumed attempt's files byte-for-byte
  under their failed-run names — summary `0635d280…` (verified before copy), observation `e3b0c442…` (0 B),
  diagnostic `e56fea39…` (92 B), bootstrap.err `e3b0c442…` (0 B); manifest `MANIFEST.txt` sha256 `1619cef5…`.
- **Collect step after the attempt:** `v3-run2-evidence.sh collect 20261101T184036Z` copies the new run's four
  files into `…/run2-20261101T184036Z/run2-evidence/` with a manifest; run-1 copy stays untouched.
- **Unit-name patterns (proposed):** preflight `v3-preflight-<YYYYMMDDTHHMMSSZ>`, release act
  `v3-release-<YYYYMMDDTHHMMSSZ>`, the single V3 invocation `v3-run-<YYYYMMDDTHHMMSSZ>`; the operator reports the
  exact `Running as unit:` line so it is bound into the receipts/record.

## 9. Attempt-2 preconditions cleared (2026-11-01T19:00Z) — deletions recorded, stage removed, CLEAN stage A

**Deletion of the two no-clobber output originals** (authorised by the Lead; preserved copies untouched):
- BEFORE: `identity-cf93107-v3-observation.json` 0 B sha256 `e3b0c442…`; `identity-cf93107-v3-diagnostic.txt`
  92 B sha256 `e56fea39…`; pre-delete `cmp` proved both byte-identical to the preserved copies in
  `v3-runs/run2-20261101T184036Z/failed-run1/` (manifest `1619cef5…`, run-1 summary `0635d280…` verified).
- DELETED: both originals (`rm`, no sudo; cayleb-owned). AFTER: the preserved workspace contains only
  `identity-cf93107-v3-procedure-summary.json` (664 B, `0635d280…`, truncate-on-open by the bootstrap) and
  `identity-cf93107-v3-bootstrap.err` (0 B, same). Preserved copies unchanged (`e3b0c442…`, `e56fea39…`,
  manifest `1619cef5…`).
- Reason: the procedure `prepare()` opens OUT/ERR with `O_EXCL` — existing files would block attempt 2.

**Stage removal (owner root act, authorised earlier-removal under Nox 181500Z):** `/run/dsh-fleet-id-cf93107` is
**absent** (observed 19:00Z); its digests are recorded in `raw/v3-run/stage-digests.txt` and §7. This is the
authorised earlier removal required by attempt 2, not a deviation.

**Clean stage A §C pre-flight** (NNP-free unit `v3-preflight-20261101T190015Z.service`, read-only): **PREFLIGHT-PASS,
rc=0** — all 14 approved `sudo -n -l` shapes PASS with no prompt; fleet caller argv PASS; C3 non-tty root read
performed (stat ran, "No such file" = permission success); stage/outputs absent; polkit parse clean; service read
active/running. Log sha256 `49024cba…` (internal receipt `e27d5e67…`), path
`scratch-tb-l1/raw/preflight-v3-20261101T190015Z.log`. Two UNTESTED informational lines only (root-only drop-in
paths; behaviour proven by the release act's password-free stop / C1).

## 10. Stage B release act — GREEN (2026-11-01T19:00:51Z)

Unit `v3-release-20261101T190051Z.service` (NNP-free transient; log sha256 `f230b3ea…`,
`scratch-tb-l1/raw/stage-b-release-20261101T190051Z.log`), rc=0:
- `systemctl --no-ask-password stop fleet-dsh` rc 0 (polkit rule works password-free); inactive confirmed.
- `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin plugin-add 37e3a542… cayleb-dsh-fleet-0.2.2.tgz` rc 0 —
  **wrapper normalisation step ran on the real install: "normalise: 82 files single-link with archive modes;
  package dirs 0750"** (the fix under test). Archive hash unchanged `37e3a542…`.
- `systemctl start fleet-dsh` rc 0; unit active/running, **new InvocationID `6ac541f5c8214faa90e97f7af983123e`**,
  NRestarts=0; installed wrapper `eb0ad024…` (byte-identical to the reviewed proposed wrapper); drop area 6
  entries unchanged.
- Post-release read-only checks: wrapper `eb0ad024…`, archive `37e3a542…`, unit active — all as expected.

## 11. Window opened — hold declared, baseline + window-start receipts (2026-11-01T19:01:05–06Z)

- **Hold declaration** (Lead's; no-writer interval from window-start through the Stage 2 final restart):
  `scratch-tb-l1/raw/hold-declaration-v3-run2.md`, sha256 `21d6b0febafb05e5d4025bafea12a550185ace87176b420eac037e19b3300e75`.
- **Baseline receipt** `2026-11-01T19:01:05Z`: receipt_sha256 `48b86ce3034201749476e60f5cd9c1bced7e78b68d095ea5313f7577d29c8e6e`;
  invocation `6ac541f5…`, NRestarts 0, rows manifest `1bcac986…`, drop digest `4055db47…`, wrapper `eb0ad024…`,
  unit `b44c05d0…`, hold sha embedded.
- **Window-start receipt** `2026-11-01T19:01:06Z`: receipt_sha256
  `999f3ebdb2cc8046b0be818fd1ca4ea85f587a55a53d58414bd413a937cb8ede`; same invocation, hold sha embedded.
- Window is open for the **single** V3 attempt 2; no retry if BLOCKED.

## 12. Attempt 2 RESULT — CONDITIONAL_DISK_OBSERVATION; evidence collected; window-end receipt

**Correction to §9 (recorded honestly):** the note "summary/bootstrap.err are truncate-on-open, so they do not
block" was WRONG. The bootstrap runs `set -euC` (noclobber), so its `exec > …-procedure-summary.json` and
`exec 2> …-bootstrap.err` FAIL if those files already exist. The Lead's first bootstrap attempt
(`v3-run-20261101T190207Z`, 20 ms, rc 1) failed exactly this way (`cannot overwrite existing file`), **before any
procedure code ran** (no stage, no service action, no output). The originals were then removed (preserved copies
verified byte-identical) and the single procedure invocation ran. Correct precondition for any future attempt:
**all four preserved-workspace output filenames must be absent** before the bootstrap (or run the bootstrap from a
clean workspace), not just the two `O_EXCL` outputs.

**Executed run:** unit `v3-run-20261101T190246Z.service`, rc=0 (310 ms). Result
`identity-cf93107-v3-procedure-summary.json` 607 B sha256 `9a45faeb…`:
- `outcome=CONDITIONAL_DISK_OBSERVATION`, `reason=null`, `observation_validated=true`,
  `restoration=NEW_INVOCATION_RECORDED`, `stage_disposition=CREATED_STAGE_PRESERVED_OWNER_INSPECTION_REQUIRED`,
  `stage_device_inode=[26,112370]`, `formal_stage2=NOT_RUN`, `loader_origin/no_writer/runtime_identity=NOT_ESTABLISHED`.
- baseline `InvocationID 6ac541f5c8214faa90e97f7af983123e` (MainPID 304045) → after
  `51c53101c03c4dad8b287fea8ceac849` (MainPID 305549), active/running.
**Private observation** `identity-cf93107-v3-observation.json` 12,163 B sha256 `ed75242f…`: target
`/var/lib/fleet/dsh/profiles/fleet/node_modules/@cayleb/dsh-fleet`; root device/inode/mount `32/1839011/32`;
**82/82 file records**; archive `37e3a542…`, caller/core/manifest match the pins; policies:
`file_mode_policy=EXACT_ARCHIVE_MODES_REQUIRE_INSTALLER_CONTRACT_CONFIRMATION`,
`directory_policy=CANDIDATE_REQUIRES_INDEPENDENT_CONFIRMATION`; limits: `no in-memory proof; reads may update
access times`; timestamp `2026-11-01T19:02:46.907769+00:00`. Diagnostic 0 B, bootstrap.err 0 B.
**Collection:** `v3-runs/run2-20261101T184036Z/run2-evidence/` (manifest `e7aea045…`: summary `9a45faeb…`,
observation `ed75242f…`, diagnostic/err `e3b0c442…`); attempt-1 copies untouched.
**Window-end receipt** 19:03:29Z: receipt_sha256 `f857da8e7c8335532fbd261b54e18dab245fe912c1e9c74818a233ceaf969be9`;
service invocation `51c53101…`, NRestarts 0; starts in window=1, attributed=1, unattributed=0; baseline proof
`6ac541f5`; hold sha `21d6b0fe…` embedded. The hold remains in force until the Stage 2 check's final restart.

## 13. Hold-close instrument for the Stage 2 restart (prepared, not run) — 2026-11-01T19:05Z

The Stage 2 check can kill/restart fleet-dsh, so after its final restart the current invocation is a **third**
invocation (not the V3 procedure's `after`). The strict window-end receipt (`f857da8e…`, already taken right after
the V3 run) correctly enforces `after == current`; it cannot be reused for the Stage-2-spanning close.
Prepared `window-receipt-v4-stage2span.sh` sha256 `b131fbab…` (runner target changed in a scratch dry-run only):
same hardened collection (fail-closed required fields, procedure baseline/after bind, in-window restart
enumeration, hashed hold declaration, stated blind spot), plus `--stage2-span` mode:
- requires current invocation ≠ procedure baseline and ≠ procedure `after` (i.e. a genuine Stage 2 restart),
  attributes it explicitly in `stage2_restart` and in the restart lists, keeps `unattributed` visible;
- without the flag the strict window-end behaviour is unchanged.
Dry-run (synthetic procedure result only, no service action): window-close rc 0 with the current invocation
distinct from baseline/after, hold sha embedded, receipt `f7376093…` (`raw/window3-window-close-20261101T190515Z.json`);
the same synthetic result under strict window-end fails closed (`RECEIPT_INCOMPLETE:receipt-not-the-procedure-after-invocation`, rc 3).
To be taken on the Lead's signal when the Stage 2 run reaches its final restart; hold remains in force.

## 14. Stage note (2026-11-01T19:08Z) — attempt-1 stage removed before the run; NEW stage preserved

- The owner's `sudo rm -rf /run/dsh-fleet-id-cf93107` removed the **attempt-1** stage **before** the run — that is
  why the stage-A preflight was clean (`C4-stage-absent` PASS at 19:00:15Z).
- The stage currently in `/run` is the **NEW** stage created by the successful run at **14:02 ET
  (19:02Z)**: three root-owned 0444 files, mtimes 14:02, digests verified now against the pins — caller.py
  `556ad026…`, identity_snapshot_core.py `f211db39…`, manifest.txt `6fa03597…`.
- Disposition stands as `CREATED_STAGE_PRESERVED_OWNER_INSPECTION_REQUIRED`; it is preserved, **not removed by
  Team B**, and will clear on reboot (tmpfs). Owner may direct earlier removal as a separate authorised act.

## 15. Stage 2 settled — settling window-close receipt, hold released, anomaly-3 reconciled (2026-11-01T19:15Z)

**Settling window-close receipt (stage2-span):** 19:15:40Z, receipt_sha256
`5e4b87f3bbfe364cddef57e441d21237c79cacde8c19df977ac5cdf6fe2ad5e1` — service active/running, invocation
`b2178a052bad41a6891d1da53e248b94` (MainPID 313468, the Stage 2 check's own 2.11 restart; ≠ V3 baseline
`6ac541f5…` and ≠ V3 after `51c53101…`), NRestarts 0; starts_in_window=2, **attributed=2 (V3 after + Stage 2
restart), unattributed=0**; baseline proof `6ac541f5…`; hold sha `21d6b0fe…` embedded; journal bound
window-start−7d. This closes the declared interval **with the Stage 2 restart inside it**.
**Hold released:** the Lead released hold `21d6b0fe…` at 19:15Z (its declared endpoint — the Stage 2 final
restart — is reached). Release is the Lead's act and is recorded here; the hashed hold declaration file is
unchanged (it is cited by the receipts) and no writer action occurred after the release.

**ANOMALY 3 reconciliation — PROVEN CONSISTENT (same run, two layers):** the checker flags `failed-run1/` as
"mixed-vintage" because summary `0635d280…` says BLOCKED "fixed command /usr/bin/sudo --user fleet failed exit 1;
no alternate route" while diagnostic `e56fea39…` says "package tree failed comparison: full file mode mismatch".
Evidence that they are the **same attempt at two layers**, not mixed vintages:
1. **mtimes within 0.314 s of each other** (preserved with `cp -p`): bootstrap.err 10:46:52.695 ET, observation
   10:46:52.722, diagnostic 10:46:52.986, summary 10:46:53.009 — one run, and the summary closes last, exactly as
   the bootstrap's stdout redirect flushes on exit; observation is 0 B precisely because the caller refused before
   writing it.
2. **Code paths:** the caller raises `Blocked("package tree failed comparison: " + str(exc))`
   (`PROPOSED-identity-caller-v2.py:223`) and prints `{"outcome":"BLOCKED","reason":str(exc)}` to **stderr**
   (`:254`) — that stderr IS `ERR_NAME` / `…-diagnostic.txt`. The procedure's `call()` runs that caller as a fixed
   command; its `_run` raises `Blocked("fixed command … failed with exit " + rc + "; no alternate route")`
   (`PROPOSED-identity-procedure-v3.py:182-183`), `run()` wraps it as `Blocked: …`, and the summary records exactly
   that string. So the summary is the **procedure-level** record (caller command exited 1) and the diagnostic is
   the **caller-level** reason (why it refused). Same event, two layers — consistent, as the Lead read it.
3. **No other run shares those files**: attempt-1's four originals were created and closed inside that 0.3 s
   window; the attempt-2 outputs are the separate 607 B/12,163 B files dated 14:02 ET with their own manifest.
Corrected wording (checker package §3c): `failed-run1/` is one attempt's two-layer evidence, **not** a mixture of
runs; the only "mixed vintage" is that both a procedure-level and a caller-level artefact exist, which is by design.
