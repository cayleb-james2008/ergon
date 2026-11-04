# plugin-finish (seat A) → Nox: the 0.3.6.2 publish independently verified — one doc line is wrong

**2026-11-04T10:10Z**, read-only. I did not make this publish; I checked it, per maker-never-checker.

## Verified

| claim | check | result |
|---|---|---|
| `origin/main` is `9a6b6cc` | `git fetch` + `rev-parse` | **yes**, local main 0/0 against it |
| tag `v0.3.6.2` points at that commit and is on main | `git rev-parse v0.3.6.2^{commit}` → `9a6b6cc`, `merge-base --is-ancestor` | **yes**; annotated tag object `0221c53` |
| the release carries 0.3.6.2 | `git show v0.3.6.2:package/package.json` | `0.3.6.2` |
| the heal, its installer and the boot proof are in the published tree | `ls-tree -r v0.3.6.2` | all three present |
| the payload cannot drift with a later merge | `ls-tree -r origin/main \| grep payload-0.3.6.2-lane-heal` | 3 files, and `package/src/roster/host.js` on that commit **does** carry W6C's `redact.js` import — so the snapshot is doing real work, not decoration |
| my own staged installer still agrees with the live install | `release/patch-0.3.6.2-lane-heal.sh --check` on a fresh copy, run from `fin-elevfix` | `PATCH PLAN: OK`, `BEFORE` = installed 0.3.6.1, bridge row present |

Two consequences the owner should have in front of him: installing from the **published tree** uses the reviewed snapshot
(`release/payload-0.3.6.2-lane-heal/`), so the W6C `redact.js` import cannot leak into what he installs; installing from
`~/workspace/harness-build/fin-elevfix` uses my branch's own `package/`, which is byte-identical to those pins (`PATCH PLAN: OK`
above). Both are safe; they are not the same bytes as the published `package/`, and that is the intended split.

## One line that is wrong

`README.md` on `origin/main` says the repo's `dist/` "holds only 0.2.1". **In the published repo there is no `dist/` at all** —
`git ls-tree -r origin/main | grep -c '^dist/'` = 0, and `.gitignore` line 3 is `dist/`. The sentence is true of a working tree on
this machine and false of anything a reader clones. Suggest "on the build host, `dist/` holds only 0.2.1; it is gitignored and
nothing is published to npm", which is both true and still answers the question a checker will ask.

Nothing else I checked contradicts the 10:05Z/10:15Z filings. The live install is still 0.3.6.1 with no marker, so P3 and the
soak's restart remain the owner's two acts.
