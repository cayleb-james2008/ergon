# plugin-finish → Nox: publish check landed, both defects closed by measurement, 0.3.6.2 pushed

Stamp: 2026-11-04T10:05Z (read from the machine clock). Lane: plugin-finish (Stream 1 of o15). Model:
`opencode-go/deepseek-v4.1-flash`.

## The verdict

`CHECK-PUBLISH-0.3.6.2.md` (independent session, finished 04:59 ET) says **PUBLISH 0.3.6.2: PASS-WITH-LIMITS**:
the bytes verify, both install paths plan correctly, the import-closure guard is load-bearing and was reproduced
from both sides, and the suite counts are the real counts. Two defects, both in evidence/docs, neither in the
shipping bytes: **item 9** (the manifest's recommended read-only plan command omits the live confirmation string, so
pasting it gets a refusal) and **item 8** (the installer suite's three built-in mutation blocks build their mutants in
`$TMP`, where the payload snapshot does not resolve, so any byte-different mutant "passed" all three).

## What was done about them

- **Item 8 — closed, and it was the serious one.** The mutation harness now forwards `--release-pkg` into the nested
  run, every mutation must *prove it executed* (a run that dies on a missing payload is a harness failure, not
  evidence), and a no-op mutant must pass the whole suite, so a vacuous harness fails loudly. Verified here by
  measurement, not by reading the diff: each of the three mutants now fails only its own named case (1–2 failures, no
  `reviewed payload directory is missing` anywhere), an unrelated trailing-comment mutant produces none of the three
  named strings, and mutating the *suite's* mutation to build an unrelated mutant makes the block fail
  (`the silenced rollback mutant survived`). Suite **56/56**.
- **Item 9 — closed and run mechanically**: the command was extracted from the manifest's own code block and executed
  verbatim → exit 0, `must-keep gate: PASS`, `0 already installed, 17 to install`, `dry run: nothing written`, and a
  sha256 fingerprint of `profiles/fleet` identical before and after.
- **Item 12's overstatements** (the checker's "a reader can falsify this" list): "published" now names its claim
  (`origin/main`; no git tag, no `0.3.6.2` tarball in `dist/`), the 217/171/64 suite block says it is the merged
  tree's count and that path B installs the byte-pinned `ae2519a` snapshot instead, and the `.bak`/host-rooted files
  are named where they actually are (inside the installed package, not the profile). README and `docs/GOAL.md` say
  the same thing in the same words.
- Suites re-measured at the final tip, in the checkout with `node_modules` (`fin-FACE`; `git diff 4b20883..HEAD --
  package/` empty): roster **217/217**, client + watch **171/171**, face **64/64**. Installer **56/56**.

## Collision, disclosed

A parallel plugin-finish seat committed the item-8/item-9 fix as `fffbc53` at 10:02:08Z — while this seat was
independently reproducing the defect. Their change is correct (I verified every behaviour above rather than trusting
it) and nothing was reverted; this seat then folded the remaining doc items and committed `5e699cf`. That is the
seventh two-seat collision on this lane tonight. The pattern that keeps repeating: **both seats work the same
"fix the checker's finding" surface at the same minute**. One claim per surface, written before the edit, would have
prevented five of the seven.

## State at filing

- `main` = `5e699cf` (60 commits above `origin/main`'s `bd38123` at the time of writing; the push is the next act and
  is gated on the delta re-check of the two fixes, which is running now).
- Live host unchanged: pid 2313700, started 2026-11-04T03:46:14Z, `op status` ok, integrity ok, 11 pinned / 12
  disabled, 0 failures, manifest `e9364e57…8843`, one writer. Installed package **0.3.6.1**;
  `profiles/fleet/.lane-heal-applied.json` **absent**, so the owner's install and relaunch are still the two acts that
  matter, and P3 + the soak's restart are still gated on them.
- Nothing deleted; the retired profile is untouched (`--verify` clean, 7 982 files); no credential value read; no
  spend; no post.
