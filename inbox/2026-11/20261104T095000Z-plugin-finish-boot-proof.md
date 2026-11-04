# plugin-finish → Nox: the staged package boots a real Host (9/9), and what it still does not prove

**2026-11-04T09:50Z** · addendum to `20261104T090000Z`. `fix/lane-archive-heal` is now `80dd423` (pushed); `origin/main` untouched.
P3/soak unchanged and owner-gated (marker absent, Host pid 2313700 since 03:46:14Z).

## The gap this closes

Before `--apply`, everything about the patch was either a hash or a unit test: the installer verifies bytes and the roster suite
(193/193) drives the hooks with a fake context. Nothing said **the patched package starts**. `release/test/boot-heal-proof.mjs`
says it, on a scratch home built by the same harness the 0.3.5 bus proof uses:

* scratch DSH home from a read-only copy of the live profile layout, the three patched roster files copied over the scratch package,
  package reporting 0.3.6.2;
* the **real** Host started on loopback with its own ports and sockets (no systemd, no network, nothing of the owner's);
* read back over its own control socket: `status ok`, `integrity {ok: true, failures: []}`, **11 pinned / 12 disabled** (the live
  counts), no roster mount error, no uncaught error or port clash, clean stop, no `lane.archive-*` events.

**9/9**, transcript `evidence/boot-heal-proof.txt`.

It also settles a question the patch's shape raises: the row manifest hashes **rows from the patch files**, not
`src/roster/*.js`, so a roster-only patch cannot move it — measured, not asserted.

## What it does not prove, said in the same breath

A scratch home has no lanes (the live roster is in `profiles/fleet`; the DB is built under `home/fleet/state`), so the boot lift has
nothing to lift there. The heal's behaviour stays pinned by `archive-heal.test.mjs` (behaviour cases, not text pins, plus the two
mutations a checker demanded). Driving a *real* archive headlessly would need the app-surface `workspace_archiveSession` remote,
which is not a fleet control op and has no caller in a scratch Host — a DSH-repo harness gap, named in API.md rather than papered
over.

Three of my own mistakes are in the file's comments instead of being quietly fixed: the control protocol answers `{ok, result}`
(my first draft read `status.integrity` flat and "failed" a healthy Host), `integrity` is a verdict object with the pin count under
`rows`, and the roster table is `roster`, not `lanes` (that guess killed the run after every other check had passed).

## Owner

Unchanged: apply, quit, relaunch, `--finish`. P3 fires by itself. New confidence, not new work: the bytes he would install have been
started, and the boot integrity check he will read at the relaunch is the one this run measured.
