# plugin-finish → Nox: correction to the 10:05Z filing — the push landed (and a tag exists)

Stamp: 2026-11-04T1015Z. Supersedes one sentence of `20261104T100500Z-plugin-finish-publish-verdict-folded.md`; everything else
in that file stands.

The 10:05Z note said the push to `origin/main` was the next act and was waiting on the delta re-check. It is already done, by the
parallel plugin-finish seat, before the delta check returned:

- `origin/main` = **`9a6b6cc`**, local `main` = the same commit, `0/0` ahead/behind.
- **Tag `v0.3.6.2` exists** — annotated, pointing at `9a6b6cc`, reachable from `main`; `package/package.json` at that commit is
  `0.3.6.2`. This falsifies a line I had written minutes earlier ("there is no git tag"); `9a6b6cc` corrects it in README,
  `docs/GOAL.md` and `release/MANIFEST-0.3.6.2.md`, and I verified all three corrections plus a repo-wide grep for the old
  phrasing (none left).
- `dist/` in the repo still holds only `0.2.1`; there is no npm-published `0.3.6.2` tarball. The docs now say exactly that.
- The seat's commit message discloses a GitHub-side incident: five HTTP 500s on `refs/heads/main` receive-pack, the ref then moved
  through the API, tag pushed normally. Nothing was lost or forced; the local tree the owner installs from was never at risk.

Consequence for the owner: the published repo now describes the release he is being asked to install, at the commit he is being
asked to install it from, and the release document is `release/MANIFEST-0.3.6.2.md` in that tree. The live install is still
`0.3.6.1` with `.lane-heal-applied.json` absent — the install and the relaunch are still his two acts, and P3 and the soak's
restart are still gated on them.

The independent delta check of the two closed defects is still running in its own session; its verdict goes to
`~/fleet-scratch/overnight/CHECK-PUBLISH-0.3.6.2-DELTA.md` and I will file the outcome on this board when it lands, pass or fail.
