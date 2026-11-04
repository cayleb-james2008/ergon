# plugin-finish → Nox: one more code fix on the published tip (a stranded commit, verified and pushed)

**2026-11-04T10:16Z** (measured; the original stamp and filename said 11:00Z, typed ahead of the clock — corrected here). This adds to `20261104T104000Z-plugin-finish-delta-pass-and-dist-fix.md`; that filing said `ff46cf2` was
the final docs tip, and it no longer is. Corrected here rather than edited there.

## What landed after it

The parallel seat committed `9a4b053` ("fix(installer): the merged-install path can warn about a Host that predates the
install") and then stopped — no plugin seat process was alive (`ps`: only the ergon seat and this one), so the commit would have
sat unpushed and invisible to the owner. Two things were true and both mattered:

1. **It is a real code defect, not a doc tweak.** On path A (the whole-release install, which writes no
   `.lane-heal-applied.json`) there is no `applied_at`, so the "the running Host predates the patch" warning could never fire,
   and `--finish` would quietly remove the bridge row under a Host older than the install — taking its rebound sockets until the
   next start. That is the exact hazard the warning exists for, on the path the owner actually takes. It now falls back to the
   newest mtime of the three patched files, names that evidence in the message, and says so when it has neither marker nor mtime.
2. **It was unverified.** I checked it rather than trusting it: suite **62/62**, case 5e pins both directions (a Host 120 s
   before the install warns and names the evidence; 120 s after does not), and I rebuilt the 7e mutation myself — silencing the
   fallback fails exactly those two assertions with **zero** `reviewed payload directory is missing` lines. Then `verify-publish.sh`
   **8/8** at the new tip, which reads the tree out of `git archive origin/main` and runs the release gates against those bytes.

Pushed as a fast-forward: **`origin/main` = `9a4b053`**, `0/0` against local. `git diff v0.3.6.2 origin/main` touches README, the
manifest and the installer + its suite, and **zero files under `package/`** — so the tag still names the release bytes.
`dist/` remains gitignored (nothing published to npm).

## State

- `origin/main` = **`9a4b053`**; tag **`v0.3.6.2` = `9a6b6cc`**; `package/` identical between them.
- Installer suite **62/62**; the release-wide test block is **314 cases, 0 failures** (252 in the lane folder + 62 in the repo).
- Live install **0.3.6.1**, `.lane-heal-applied.json` absent → the owner's install + relaunch are still the two acts. P3's watcher
  (60 h window) and the soak sampler are armed and running; `p3=no-marker` is the honest state.
- Disclosure: `NEXT.json` was truncated by a bad write in this session (`open(…,'w').write(dict)`) and rebuilt from the measured
  facts in `plugin-progress.md`. Nothing measured was lost; the seat's exact wording was. Its own note says so.
