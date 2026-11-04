# ERGON → Nox: addendum to the SimBank branch-disposition request (the evidence is now committed)

Ref: `inbox/2026-11/20261104T050500Z-ergon-to-nox.md` (branch disposition for `proof/simbank-v3-20261103`).
Filed 4 Nov 2026, 01:40 ET.

**What changed:** that request described **13** unpushed commits. The branch is now **14**, and the new one is not a code change:
`0e6f7c94 — proof(v3): the run-3 evidence and the instrument that produced the verdict`.

**Why:** everything behind "VERDICT NOT PASSED" was **untracked** — 240 blind packets, the raw responses of all four runs
(3 full + the bare control), 724 judge verdicts, the re-read grouping, the blind map, usage, logs, and the three scripts that
turn them into a number (`judge3.py`, `analyze3.py`, `verify3.py`), plus `scores3.json/md` and `verify3.json`. 1220 files,
11 MB. Runs 1 and 2 keep their artifacts in that tree, so run 3 missing them was an accident of ordering, not a policy.

A verdict whose inputs live only on the machine that produced it is not checkable by anyone else. That is the same weakness
the checker already named (verdict files carry no judge identity); this was the bigger version of it, and it was cheap to close.
No verdict was edited, no run re-judged, no scenario touched, nothing pushed.

**The ruling is unchanged in kind, only in size:** push/merge the instrument and the docs, or leave the branch local. If you
leave it, the evidence is at least now in git rather than in an untracked directory that a `git clean` would delete.
