# Task A evidence — review of the dsh-fleet-admin normalise change for 0.3.0

Verdict: **ACCEPT**. Checker: checker-install (independent; ran none of the wrapper's own test code).

Verified inputs:
- installed wrapper `/usr/local/bin/dsh-fleet-admin` sha256 `eb0ad024...b888` (matches)
- reviewed new wrapper `strike-release/ops/dsh-fleet-admin.new` sha256 `75ab346e...1209` (matches)
- 0.3.0 tarball `strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz` sha256 `5c91f430...1fd5` (matches); 104 regular members (101×0644, 3×0755); `@cayleb/dsh-fleet 0.3.0`

Findings:
1. `00-full-diff.txt` — exactly **one hunk** (`@@ -67,12 +67,29 @@`), wholly inside the `PYNORM` heredoc. `01-heredoc-strip-proof.txt` — non-heredoc text is **identical** (227 lines each); only the heredoc body differs (36→53 lines). `bash -n` clean on both.
2. `logs/10-sim-main.txt`, `logs/11-assert-main.txt` — simulated pnpm state (all 104 files hard-linked nlink=2, modes masked 0644→0640/0755→0750, dirs 0750): block rc=0, prints `normalise-archive: @cayleb/dsh-fleet 0.3.0 104 regular members`; independent assert: 104/104 regular, nlink=1, exact archive mode, sha256 match, 0 extra, 0 missing, all dirs 0750.
3. `logs/12-sim-variant.txt` — dirs starting 0755 are set to 0750 by the block; assert clean.
4. Negative controls: `logs/13-neg-bad-name.txt` — archive package.json name changed → rc=1, `normalise: archive package name is 'evil-package', not '@cayleb/dsh-fleet'`, installed tree byte-identical before/after (refusal before any tree write). `logs/14-neg-missing-member.txt` — a member deleted from the installed root → rc=1, `normalise: missing member src/core/core.js: ...`.
5. `logs/17-old-block-vs-030.txt` — the **installed** 0.2.2 block on the 0.3.0 archive: rc=1, `normalise: expected 82 members, got 104` (the reported blocker reproduced).
6. `logs/20-sudoers-polkit.txt` — effective `sudo -n -l` matches `ops/sudoers/dsh-fleet`; `/etc/sudoers` mtime 2026-10-25 (pre-strike), `/etc/sudoers.d` and `/etc/polkit-1/rules.d` mtimes 2026-11-01 13:58:22 (setup), all before the 17:11 wrapper work. Relay's only mutating ops are the two `sudo /usr/bin/install` lines for the wrapper (+ backup). Content/listing of /etc/sudoers, /etc/sudoers.d, /etc/polkit-1/rules.d is **not readable as cayleb** — stated, not guessed.
7. Observation (cosmetic, not a blocker): the relay's `verbs before/after` lines invoke the wrapper bare via `sudo -n -u fleet`; no sudoers entry matches a bare invocation, so both print `sudo: a password is required` instead of a verb list (`|| true` keeps the install going). The review-package claim "It prints the current/new verb lists" does not hold.

Harnesses (mine, not the wrapper's): `sim-pnpm.py`, `assert-normalised.py`, `mk-bad-archive.py`, `run-simulated-test.sh`; raw logs in `logs/`, including `16-full-console.txt`.
