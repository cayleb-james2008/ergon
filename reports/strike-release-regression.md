# Release regression pass — 0.3.0 integrated tree (Lead), 2026-11-01T22:40–23:00Z

Purpose: before the install, run the package's own unit suite on the integrated release tree and A/B it against the 0.2.2 tree
that closed Stage 2, so a merge defect cannot reach the installed output.

## Environment note (why the raw suite is noisy here)
The package's unit suite expects the composed DSH runtime (peer packages @deepseek-ai/* and dev tools). In the bare source
worktree those peers are absent, so whole test files fail at import (`ERR_MODULE_NOT_FOUND`) — 10+ files, environmental.
With the peers installed from the published 0.2.2-alpha.2 packages in a temp copy, the suite runs but a further set of tests
that need the real install paths (/var/lib/fleet, roots owned by fleet) or kernel/namespace features fails the same way on the
*accepted 0.2.2 tree* — i.e. environment-sensitive tests exist on the baseline too and are not the acceptance gate (the stage
checks are, and those passed 60/60 on the installed output).

## A/B result (same temp environment, same command)
| Tree | tests | pass | fail |
|---|---|---|---|
| 0.2.2 accepted tree (cf93107) | 456 | 416 | 40 (47 unique entries incl. whole-file failures) |
| 0.3.0 integrated tree | 578 | 534 | 43 (49 unique entries) |
| Normalised diff (timings stripped) | — | — | **no new failure class**: the delta is netns/consumer/module-missing noise (`@deepseek-ai/dsh-web`, netns), and a file-level rename of the same sandbox consumer test |

Dependency-satisfied subset on the 0.3.0 tree: 556/558 pass; the single failure is the sandbox-importing finisher test
(`ERR_MODULE_NOT_FOUND @deepseek-ai/dsh-sandbox` — peer not installed in that copy).
New-code suites: core + firstrun + mcp = 220/220 pass (after the two fixes below).

## Two REAL defects the pass found in the shipped artifact, both fixed
1. **`bin/rows-manifest.mjs` (a SHIPPED file) mishandled the new extra bundle-patch file.** The generator treated every
   `dsh.bundle.patch` entry after the first as a preset patch, so `mcp.row.yml`'s inserted row was about to be pinned as an
   exact-insert row. That is wrong for a host service row: insert-pins require the mounted config to match the manifest exactly,
   so a profile layer re-pointing the row (the scratch overlays, a legitimate port/path change) would fail boot integrity —
   precisely the class the design avoids by pinning only mount rows by name. Fix: only `./presets/*.patch.yml` pin inserted rows;
   other bundle patch files join the bundle patch array and contribute their disables/inserts without pinning. Regenerating
   `rows.manifest.json` with the fixed generator produces a **byte-identical** file to the shipped one (7 pinned, 12 disabled) —
   proof the manifest was already correct and the generator was the defect.
2. **`src/core/test/cli.test.mjs` encoded the pre-0.3.0 pure-shim contract** and failed on the intentional `first-run` dispatch.
   Updated to the documented contract: only the literal command `first-run` is dispatched to `src/firstrun/index.mjs`; every other
   command imports fleet-core's CLI; plus a behavioural check that `first-run --help` exits 0 and prints the first-run usage.
   (Tests are not shipped in the tarball.)

## Corrected artifact
- `cayleb-dsh-fleet-0.3.0.tgz` sha256 **d03ea2ac8163985732baf1b64a907f698cd62cd41c5ed96be5d72b6d99394aab**, 104 members
  (101×0644 + 3×0755), `@cayleb/dsh-fleet 0.3.0`. Only `bin/rows-manifest.mjs` content differs from the previous build
  `b60019a0…`; `rows.manifest.json` is unchanged.
- Scratch gate on the corrected tarball: PASS — profile composes 214 = 196 + 18 rows, host `status` ok/integrity ok/lanes
  allowed, `first-run --status` = 104 files / 0 missing / 0 extra vs the tarball, `mcp-selftest` 15/15 PASS.
- Staged for the wrapper at `/var/lib/fleet-drop/cayleb-dsh-fleet-0.3.0.tgz`; the install script's expected digest updated;
  the wrapper-change review doc notes the final digest (member count and modes unchanged, so its normalise review stands).
- Checkers re-running their S4A (package-alone install) and S5 (MCP battery) proofs against this exact digest.
