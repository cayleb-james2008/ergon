from: face lead (overnight fleet)
to: nox
kind: CLOSE-OUT — checker round 2 says PASS; the face is one artefact from the owner's window
re: my filings 20261104T052800Z (merge + live data + staged install) and 20261104T072500Z (findings closed)
---

## Checker round 2 (fresh session, pinned `opencode-go/deepseek-v4.1-flash`, did not build this)
**PASS.** Her four round-1 findings verified closed as product defects, each with a measurement, not an opinion:
- **HIGH-1 mobile evidence** — her five `m390` dumps now have five distinct hashes, each with its own panel heading; a negative
  control (the old one-drawer click) makes the gate exit 1 with exactly four FAILs showing Fleet text. So the gate now fails
  when navigation breaks.
- **MED-1 theme** — panel background measured `rgb(255,255,255)` in the light shell (body white) and `rgb(25,25,25)` in dark
  (body `rgb(21,21,23)`), 15 theme checks pass.
- **MED-2 Browser** — carries `data-fleet-lumen`, `box-shadow`/`backdrop-filter`/`filter`/`background-image` all `none`, card
  geometry and font identical to Fleet, pills 8 px not 999 px. One LOW (h1 tracking `normal` vs `-0.22px`) — closed since.
- **MED-3 staleness** — fresh **70 renders, 0 problems**, now including a light viewport.
She also confirmed the two smaller things I had fixed: the attention count no longer disagrees with its list (informational
flags moved to their own Notes group), and both headers print one timestamp format.
The two MEDIUMs she raised in round 2 were **plumbing, not product**: the Browser's own gate was measuring against a sheet the
panel no longer paints with (now green — 36 renders, 0 failures, min contrast 4.63, focus ring 2 px), and `dist/SHA256SUMS.txt`
did not match its tarball (regenerated; `sha256sum -c` says OK).

## Final state, verified by me on the settled commit `4b20883` (clean tree)
| Gate | Result |
|---|---|
| client + watch suites | **171/171** |
| host-face suite | **64/64** |
| `tokens --check` | the committed sheet is exactly what `tokens.json` generates |
| panel state matrix | **70 renders, 0 problems** (four panels, live/empty/loading/error/unconfigured/rehearsal/retrying, both host-rig states, dark **and** light viewports) |
| real shell (scratch host, headless Chromium) | **110/110 checks**, reads `/api/fleet/state 200`, `/api/fleet/events 200`, `/api/fleet/activity 200`; rig routes answer the probes' own 503 because this machine has no AVD and no Windows VM |
| artefact | `dist/cayleb-dsh-fleet-0.3.6.2.tgz` contains **exactly** the tree's `client.js` (`2474cbcb…be283`), and its `SHA256SUMS.txt` verifies |

## What is left, and it is one thing
**The owner's window.** Nothing is installed. The swap needs a Host restart, and the sheet is ready:
`~/fleet-scratch/face/dist/INSTALL-OWNER-RUN.md` → `release/install-0.3.6.sh --dry-run --target-live`, then the same without
`--dry-run`; it stops the unit, installs 17 files (5 added: the host half and its panel producer; 12 replaced: `client.js`,
`package.json`, `src/index.js` and the eight roster files of the merged 0.3.6.2 archive heal), starts it, self-checks, and
prints its own rollback line. `npm install` remains unusable on this package (`0.3.6.2` is not semver); the plugin lane now has
a semver checker, which is the right owner of that fix.

## Recorded, not hidden
1. **Two writers on one branch.** A second "FACE lead" session committed into `fin-FACE` throughout (`3787359`, `34f9e7e`,
   `b68194c`, `093e5f0`, `c458731`, `4b20883` …). Its commits are the same fixes, so the outcome is right, but the worktree had
   two writers and the checker's evidence went stale once because of it. Whoever finishes the face should own the last
   verification; two sessions typing into one worktree will produce exactly that defect again.
2. **Reference set still incomplete** — Linear (03) and Vercel (04) captures are login-walled, so no screen is called gate-PASS
   against the owner's full set; the checker and I both grade against Claude desktop 01/02 plus the measurable floors.
3. **Rig panels** read the host's own routes and say "No AVD attached" / "No Windows VM attached" — true on this machine, and
   the honest version of what used to be "not configured".

— face lead
