# PLUGIN-FINISH → Nox: addendum to `20261104T055500Z` — the by-hand P3 path, and the exact log lines for the owner

From: plugin-finish seat A. **Read `20261104T055500Z-plugin-finish-to-nox.md` first: it is the primary note** (R3 PASS, the
`ps lstart` 5-hour defect, the readiness and stop fixes, the collision). This adds only what that note does not cover, so the
morning reader gets one story and one set of numbers, not two competing ones.

## What is new in this addendum

1. **`p3-run.sh` got the readiness wait too, so the by-hand path is safe.** The watcher's gate only helps the automatic run;
   sheet 6.4 typed straight after the relaunch would still have hit `ENOENT` on the first op with `set -e` — no ping, no
   message, and the evidence would read as if P3 had been attempted. It now waits (default 120 s, `--ready-timeout` to change,
   `0` to skip) and fails closed with "no ping was sent".
2. **`p3-run.sh` is testable without the live app**: `--probe CMD` overrides the Host probe, so its two guards can be driven
   by behaviour instead of by hope. New **`p3-run.test.sh` 17/17** with a stub `node` on `PATH`: refuses a Host that predates
   the patch, refuses a missing marker before any op runs, waits then sends, fails closed when the fleet never answers, and pins
   the electron-only probe, the `/proc` start source and the per-user socket. The socket default is now `id -u`, not a
   hard-coded `1000`.
3. **One vacuous test is no longer vacuous**: "cache file removed on exit" checked a path that was never written. The
   `applied_at` cache now lives beside `--state`, so the assertion actually tests the cleanup (and the evidence dir stops
   collecting `.p3-watch.applied.*` files).
4. **The owner sheet says what success looks like**, line by line (`host changed` → `soak: sampled` → `ready: waiting up to
   180s for /run/user/1000/fleet/control.sock` → `ready: the fleet core answered after Ns` → `run #1 …` → `acked`), so a slow
   start can be told apart from a failure without reading the script. `OWNER-RUN-LANE-HEAL-20261104.md`, step 5.

## State, exactly

- Both suites on the final tree: **`p3-watch.test.sh` 32/32**, **`p3-run.test.sh` 17/17**. One watcher, pid 2927540, armed
  05:51Z, window to ~17:51Z, `--status` → `p3=no-marker` exit 3.
- P3 is **armed, not proved**. No verdict exists and none is claimed; the checker writes `verdict-P3.json` from the DB and the
  lane log after the owner's restart.
- Two plugin seats are live (the 05:39Z restart resumed a session that was already running). The split is written down in
  `~/fleet-scratch/overnight/CLAIM-p3-scripts-20261104T0552Z.md`: one seat on `p3-watch*`, one on `p3-run*` + the docs. Nothing
  was reverted by either; the suites above were run against the combined tree.
- Unchanged: nothing installed, no confirm typed, no credential value read, no spend this session, the owner's app untouched.

**For the delegator:** the resume loop started a second plugin seat on a mission that already had a live one, and both wrote
the same progress file for ~20 minutes. The roster already refuses a second live lead (`lead-exists`), so this is the spawning
layer, not the roster: check for a live seat (pid or session) before resuming, and give each stream its own scratch filename.
