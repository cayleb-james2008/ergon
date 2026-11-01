# Wrapper change review — `dsh-fleet-admin` normalise member count (0.3.0)

**VERDICT: ACCEPT**

- Checker: `checker-install` (independent, fresh context; read-only review — did not run the relay, did not install anything, did not fix anything)
- Date: 2026-11-01 17:13–17:17 EST, host `outis-desktop`, as `cayleb` (uid 1000)
- Artifact under review: `/home/cayleb/workspace/harness-build/strike-release/ops/dsh-fleet-admin.new`
- Review package read in full: `strike-release/release/WRAPPER-CHANGE-0.3.0.md` (sha256 `3adc9a354496c47578ad1f12892d3c81fd0f59778eab8a1c69017a91e6e78047`)

## Inputs verified (hashes all match the Lead's)

| Item | sha256 | Match |
|---|---|---|
| installed wrapper `/usr/local/bin/dsh-fleet-admin` (0755 root:root, 17569 B) | `eb0ad02480282b9b89341beb991ce38e4b6ffcace5c755d945dbcf6f0f4fb888` | yes |
| reviewed new `ops/dsh-fleet-admin.new` (18741 B) | `75ab346e8b314c25b1da630c78ded154efa5669aff62c474afa7da2d53c01209` | yes |
| 0.3.0 tarball `dist/cayleb-dsh-fleet-0.3.0.tgz` | `5c91f430b0c9219e802211779eb81eab030f4218a17fa3bb24dfba39f18d1fd5` | yes |

Tarball facts re-confirmed independently: **104 regular members** (101× `0644`, 3× `0755`), `package/package.json` = `@cayleb/dsh-fleet 0.3.0`, no directory members. (Lead noted a pending one-line MCP fix post-dates this tarball; count/modes logic is unaffected — re-confirmed on the tarball actually present.)

## (1) Diff — only the normalise heredoc changed

- `diff -u` full output: **exactly one hunk**, `@@ -67,12 +67,29 @@`.
- Stripping the `PYNORM` heredoc body from both files: **227 non-heredoc lines each, byte-identical**. Only the heredoc body differs (36 → 53 lines).
- The **only** changed lines outside the heredoc body: none. The heredoc invocation line (`<<'PYNORM' || die "normalise: failed (..."`) is identical.
- `bash -n` clean on both old and new wrappers (rc 0).
- Verb-list/usage text: the `die "verbs: ..."` line is byte-identical between old and new; all `die "plugin-add <sha256> <file.tgz in $DROP>"`-style validation strings are identical (implied by the non-heredoc identity, checked explicitly).

New heredoc block adds: non-empty member list; `package/package.json` must exist, parse as JSON, have `name == "@cayleb/dsh-fleet"` and a non-empty string `version`; every member a clean relative path; the count is derived and printed. Unchanged: mode whitelist {0644,0755}, missing-member refusal, regular-file check, de-link by copy+rename, dirs 0750, post-verify of files **and** dirs, fail-closed `SystemExit` on any mismatch.

## (2) Simulated-pnpm test — re-run independently (my own harness, not the wrapper's code)

State built to match the wrapper's documented pnpm post-install state: every one of the 104 files written with its umask-masked mode (`0644→0640`, `0755→0750`) and hard-linked into a separate store dir so each inode has **nlink=2**; package dirs left as pnpm leaves them.

- **Main case** (dirs 0750): extracted heredoc block `python3 -I -S -B <block> <tgz> <root/package>` → **rc=0**; stdout `normalise-archive: @cayleb/dsh-fleet 0.3.0 104 regular members` then `normalise: 104 files single-link with archive modes; package dirs 0750`; stderr empty. My independent assert over the resulting tree: 104/104 files regular, **nlink=1**, **exact archive mode**, **sha256 matches archive bytes**, 0 missing, 0 extra, **0 dirs wrong mode**.
- **Variant** (dirs start 0755): rc=0, dirs become 0750, assert clean.
- Pre-state spot check shows `nlink=2` and masked modes before the run, so the test is a real de-link, not a no-op.

## (3) Negative controls — fail closed

- **Foreign package name**: archive rewritten with `package/package.json` `name` = `evil-package` → **rc=1**, stderr exactly `normalise: archive package name is 'evil-package', not '@cayleb/dsh-fleet'`. The simulated installed tree is **byte-identical before/after** (the identity refusal happens before any tree write).
- **Member missing from installed root** (`src/core/core.js` deleted) → **rc=1**, stderr `normalise: missing member src/core/core.js: [Errno 2] No such file or directory: '...'`.
- **The reported blocker reproduced**: the *installed* (0.2.2) block run against the same simulated 0.3.0 tree → **rc=1**, `normalise: expected 82 members, got 104`. The change targets exactly the observed failure.

## (4) `bash -n` and usage/verb list

`bash -n` rc=0 on the new wrapper. Verb list text identical. No new verb, argument or audit-line change (non-heredoc identity proves it; the relay's later claim that it "prints the current/new verb lists" is separately addressed below).

## (5) Sudoers / polkit — untouched by this change

What I can read as cayleb:
- `sudo -n -l` (effective policy): the `(fleet) NOPASSWD:` set is exactly `version, status, dump-config, probe-access, init-profile, install-dsh 0.2.2-alpha.2, integrity-drill arm, integrity-drill restore, plugin-add *, plugin-remove *, restore-test, restore-test *, restore-live *` — matching `ops/sudoers/dsh-fleet` in the release tree.
- Metadata: `/etc/sudoers` mtime **2026-10-25 06:42:17** (pre-strike), `/etc/sudoers.d` mtime **2026-11-01 13:58:22.904**, `/etc/polkit-1/rules.d` mtime **2026-11-01 13:58:22.913** — both the setup timestamp, i.e. before the 17:11 wrapper work.
- The relay script's only mutating operations are the two `sudo /usr/bin/install -o root -g root -m 0755` lines (old→dated backup, new→target). It contains no sudoers or polkit write.

What I **cannot** read as cayleb (stated, not guessed): content of `/etc/sudoers` (`-r--r----- root:root`), listing/contents of `/etc/sudoers.d` and `/etc/polkit-1/rules.d` (both `0750`, permission denied). Byte-identity of those files is not provable from my seat; the evidence above (effective policy + mtimes + relay source) is what a non-root checker can establish.

## Observations (none blocks ACCEPT)

1. **Relay cosmetic mismatch:** `owner-relay-wrapper.sh` prints "verbs before/after" via bare `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin`. No sudoers entry matches a bare invocation, so both lines print `sudo: a password is required` (confirmed, rc=1) instead of a verb list. `|| true` keeps the install running; the line is cosmetic and the text will read identically before/after anyway. The review package's claim that it prints the verb lists is inaccurate.
2. The normalise block still does not reject an archive with **extra** members outside `package/` by count (it checks archive→installed direction and clean paths; a foreign prefix degrades to a missing-member refusal) — this is pre-existing behaviour, unchanged by this patch, and the archive is hash-pinned by the caller.
3. The derived count is printed to stdout during `plugin-add` — harmless, and useful in the install log.
4. The installed wrapper is still the old one and no `.pre-0.3.0-20261101` backup exists yet — expected, the owner relay has not run.

## Evidence

`/home/cayleb/workspace/harness-build/evidence/strike/checker-verify/wrapper-review/` — 34 files + `SHA256SUMS` (`sha256sum -c` all OK). Key: `00-full-diff.txt`, `01-heredoc-strip-proof.txt`, `normalise-heredoc.py` (extracted new block), `old-normalise-heredoc.py`, `sim-pnpm.py`, `assert-normalised.py`, `mk-bad-archive.py`, `run-simulated-test.sh`, `logs/16-full-console.txt` (whole suite), `logs/10..15-*` per case, `logs/17-old-block-vs-030.txt`, `logs/20-sudoers-polkit.txt`, `logs/30-final-state.txt`, `README.md`.

Fences respected: no root actions, no installs, no relay run, no modification of the reviewed artifact or its tree, no secret values, no image reads.

---

## Addendum 2026-11-01 17:29 ET — tarball superseded; logic re-confirmed on the current build

The release tarball was replaced after this review: it is now `b60019a0f3611c83fb13a374444c46b05092df89737f803661408fd7151a0b10` (the post-MCP-fix build) instead of the `5c91f430…` build reviewed above. The reviewed new wrapper is unchanged (`75ab346e8b314c25b1da630c78ded154efa5669aff62c474afa7da2d53c01209`). I re-ran the count/modes logic against the current tarball (independent of the S4A install run):

- current tarball: **104 regular members, 101×0644 + 3×0755**, `package/package.json` = `@cayleb/dsh-fleet 0.3.0` — identical count/modes class as the reviewed build, so the wrapper's logic is unaffected;
- simulated-pnpm case (masked modes, nlink=2, dirs 0750) + reviewed block → **rc=0**, `normalise-archive: @cayleb/dsh-fleet 0.3.0 104 regular members`, independent assert **104/104 exact archive modes, 0 missing/extra, dirs 0750**;
- negative control foreign package name → **rc=1** `normalise: archive package name is 'evil-package', not '@cayleb/dsh-fleet'`;
- negative control member missing from installed root → **rc=1** `normalise: missing member src/core/core.js: …`;
- the **installed (0.2.2) block** on the current 0.3.0 archive → **rc=1** `normalise: expected 82 members, got 104` (the blocker is still real; the fix is still required).

Evidence: `evidence/strike/checker-verify/wrapper-review/logs/31-current-tarball-recheck.txt` and `logs/32-current-tarball-sim.txt` (manifest `SHA256SUMS` refreshed, 36 files, all verify). **Verdict unchanged: ACCEPT.**

## Addendum 2 2026-11-01 17:46 ET — wrapper facts re-confirmed on the CORRECTED tarball `d03ea2ac…`

The artifact was corrected after the review; the wrapper's count/modes logic was re-confirmed on the digest that will actually be installed: `d03ea2ac8163985732baf1b64a907f698cd62cd41c5ed96be5d72b6d99394aab` — **104 regular members, 101×0644 + 3×0755, `package/package.json` = `@cayleb/dsh-fleet 0.3.0`**; reviewed new wrapper unchanged (`75ab346e…1209`).

Re-run on this artifact: simulated-pnpm state + reviewed normalise block → **rc=0**, `normalise-archive: @cayleb/dsh-fleet 0.3.0 104 regular members`, independent assert **104/104 exact archive modes, 0 missing/extra, dirs 0750**; negative controls still fail closed (`archive package name is 'evil-package'…`; `missing member src/core/core.js…`). Independent delta vs the previous build: only `bin/rows-manifest.mjs` content changes; `rows.manifest.json` unchanged.

Evidence: `evidence/strike/checker-verify/wrapper-review/logs/33-corrected-tarball-sim.txt` (manifest refreshed, all verify). **Verdict unchanged: ACCEPT.**

## Addendum 3 2026-11-01 18:03 ET — wrapper facts re-confirmed on the FINAL tarball `8173d13e…`

Final artifact `8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070`: **104 regular members, 101×0644 + 3×0755, `@cayleb/dsh-fleet 0.3.0`**; reviewed wrapper unchanged `75ab346e…`. Re-ran the normalise logic on this digest: simulated-pnpm state (masked + nlink=2) → block rc=0, `normalise-archive: @cayleb/dsh-fleet 0.3.0 104 regular members`, independent assert **104/104 exact archive modes, 0 missing/extra, dirs 0750**; negative controls fail closed (bad name; missing member). Evidence: `evidence/strike/checker-verify/wrapper-review/logs/34-final-tarball-sim.txt` (manifest refreshed, all verify). **Verdict unchanged: ACCEPT.**
