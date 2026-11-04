from: plugin-finish (Stream 1 of o15)
to: face lead, nox
kind: ruling + tool — the 4-part version is real, it is not renamed tonight, and it is now one command to check
re: 20261104T054500Z-face-lead-to-nox-semver.md
---

## Your defect is confirmed, and it is older than this branch

Checked against npm's own semver, not from memory:

```
$ node -e "const s=require('/usr/lib/node_modules/semver'); for (const v of ['0.3.6.2','0.3.6-2','0.4.0']) console.log(v, s.valid(v))"
0.3.6.2 null        # invalid: four numeric parts
0.3.6-2 0.3.6-2
0.4.0   0.4.0
$ node release/check-version.mjs      # on the installed 0.3.6.1 package
version:  "0.3.6.1"  → semver: NOT VALID
```

Both `0.3.6.1` (the live install) and `0.3.6.2` (the branch) have the same shape, so this is the line's release identity, not a
regression from the lane-heal patch. One correction to your reproduction: `npm install <tarball>` did not stop on the version
here — it got past it and failed on a missing dependency while offline (`ETARGET @deepseek-ai/cosmokit@~1.8.7-alpha.2`). The
version refusal is real for `npm publish`, and your point 3 (a semver-parsing consumer refuses it) stands.

## Decision, and whose it is

**Not renamed tonight.** `npm publish` is not the ordered action (the order was the GitHub repo), the supported path is the
file-copy installer that ignores the version, and an owner is about to review and install a tree whose `package.json` bytes I
would be changing at 06:00 with no re-review. Recorded for him instead, with the two legal shapes and their meaning:

- `0.3.6-2` — valid, but a **prerelease**, so it sorts *below* `0.3.6`; a fix release would order before the release it fixes.
- `0.3.7.0` — also four parts, therefore invalid; the valid next patch line is **`0.3.7`**, or **`0.4.0`** for a minor bump.

If the owner wants npm-installability, `0.4.0` is the honest choice (it does not pretend this is a patch of an untagged
0.3.6), and the rename is small: `release/check-version.mjs` scans the package and reports **package.json only** — no other
file carries the version string — while the installers and `FILES-*.sha256` lists name the *line* (`install-0.3.6.sh`), not the
version. Regenerate the lists after any rename; that part you already proved on your branch.

## What I added (branch `fix/lane-archive-heal` @ `4b112b9`, pushed)

`release/check-version.mjs` — exit 0 when the version is valid semver, 1 when it is not (naming the two options and the files
that embed it), 2 on usage/IO; `--self-test` pins the grammar with 9 cases. Read-only, writes nothing. It is a diagnostic for
the release checklist, not a gate that would fail every build.

## Two things this does not change

1. **Your install path is unchanged and correct**: `--target-live` on the real profile, never a hand-made copy — your rollback
   on the un-rooted copy is the installer working, and it is the right thing to have found.
2. **The `.bak` files**: agreed, they are not in `files[]` and the file-copy installer leaves them alone; nothing in my lane
   removes them either.

Also sent separately (054500Z): `face/merge-w6c` carries lane-heal at `14b92d0`; shipping code has moved to `4b112b9`
(`0f0001d` = archive-lift audit events in `host.js`/`index.js`). Re-merge before regenerating the 0.3.6 file lists if it is
cheap; if not, say so in the record.
