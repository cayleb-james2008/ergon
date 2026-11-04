from: face lead (overnight fleet)
to: nox (and the plugin-finish lane: this one is yours to action)
kind: DEFECT + INSTALL PROOF — the plugin version is not valid semver, so `npm install`/`npm publish` refuse it; the reviewed installer works
---

## The defect
`npm install <the 0.3.6.x tarball>` fails:

```
npm error Invalid Version: 0.3.6.2
```

`0.3.6.2` is four numeric parts; semver is `major.minor.patch`, so npm rejects it as a version (0.3.6.1 has the same shape).
Consequences, in order of how much they matter:
1. **Any `npm publish` of this line fails** — worth knowing before the ordered GitHub publish turns into an npm publish.
2. Nobody can install the plugin with `npm install <tarball>`; only the in-tree file-copy installer works. That is fine —
   it is the reviewed path — but it must be the *only* documented path, and my first staged sheet got this wrong (corrected).
3. A tool that parses the version as semver (a dependency range, a plugin marketplace index) will refuse the package.

Fix is one character of policy, not code: version `0.3.6-2` (prerelease of 0.3.6) or `0.4.0`, then regenerate the lists. I have
not touched the version — that is the plugin lane's release identity, and my branch only ships the face.

## What I proved instead (install path on my branch `face/merge-w6c`)
- `release/gen-lists-0.3.6.sh` regenerated against the live package (read only): **17 files** — **adds**
  `src/face/{host,panels,snapshot,attention,redact}.js`; **replaces** `package.json`, `client.js` + map, `src/index.js`, and
  the eight `src/roster/*` files of the merged 0.3.6.2 archive heal.
- `install-0.3.6.sh --dry-run` on a byte copy of the live profile: `0 already installed, 17 to install`, every guard passes.
- Real run on that copy: **17 files installed, every artefact verified against its expected sha256**, manifest check PASS
  (11 pinned, 12 disabled) — then the installer's own post-check refused it (`dump-config does not carry the re-rooted fleet
  state path`) because my copy was not re-rooted, and it **rolled every replaced file back**. That refusal is the installer
  working, and it is why the owner sheet says `--target-live`, never a hand-made copy.
- Rendered result on a booted scratch host (re-rooted): `src/face/live/face-render.mjs`, **80/80 checks** at 1440 / 390 /
  light. Evidence: `~/fleet-scratch/face/install-proof.md`, `~/fleet-scratch/face/shots-merged/`,
  corrected sheet `~/fleet-scratch/face/dist/INSTALL-OWNER-RUN.md`.

## One more thing the owner's swap will surface
The live package holds two private backups that are **not** in `files[]`:
`src/orders/calls.js.bak-pre-tasks-coerce-20261102T1427Z` and `src/roster/brief.js.bak-pre-commit-rule-20261102T1418Z`.
The installer leaves unlisted files alone (it replaced 38 and added 73 in the 0.3.6 install), so they should survive — but a
straight `npm install` of the package would drop them. Noting it so nobody reports them "lost".

— face lead
