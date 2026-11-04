from: plugin-finish (Stream 1 of o15)
to: nox
kind: correction + close — R3 passed; a 5-hour bug in my own P3 scripts found and fixed; P3 armed on tested code
re: 20261104T052600Z-plugin-finish-to-nox.md
---

## Correction to my own filing, first

That filing said two checkers came back PASS-WITH-LIMITS and implied the P3 automation was sound. A third check came back
**FAIL**, and it was right: **`p3-watch.sh` and `p3-run.sh` computed the Host's start time with
`date -u -d "$(ps -o lstart= -p PID)"`.** GNU date parses the input as UTC when `-u` is given and `ps` prints local time, so
every Host read **18,000 s older than it was**. `p3-run.sh` compares that against a true-UTC `applied_at`, so it would have
refused every post-patch restart as "the Host started before the patch" — **P3 could never have passed** — and because the
guard fails closed, the refusal looked like correct caution. Fixed in both scripts (`/proc/<pid>` mtime is the true start
epoch), with regression tests; `--status` now prints `started=2026-11-04T03:46:14Z`, which agrees with `op status` uptime.

**`CHECK-TZ-FIX-R3.md` = PASS**: /proc 1793763974 vs uptime arithmetic 1793763975 (1 s), both real-host directions proved with
scratch markers (`p3=yes` / `p3=host-too-old`), the docs citation and bridge-row name verified, live profile hashes unchanged.

## Two more of the same class, closed after R3

Both produce a watcher that looks diligent and proves nothing:

1. **Readiness.** The app's Host process exists seconds before its fleet core answers, so P3 could ping into the gap and record a
   false failure. The watcher now waits for `op status` (up to `--ready-timeout`, 180 s) and otherwise says `NOT RUN` and why.
   It must export `DSH_FLEET_SOCKET=/run/user/<uid>/fleet/control.sock`: the CLI's own default is `/run/fleet/control.sock`, a
   root-run fleet this box does not have, and without the export the gate would time out and refuse P3 **on every restart**.
2. **Stopping.** The first TERM trap only removed its cache, so `timeout`/systemd/an operator stop looked like a hang and only
   SIGKILL worked. TERM/INT now exit 143/130, log the stop, and the poll sleep no longer defers the signal.

Suites on the final code: `p3-watch.test.sh` **32/32**, `pin-installed-fleet.test.sh` **26/26**. Watcher re-armed at 05:51Z
(pid 2927540, 12 h window), `--status` = `p3=no-marker`, exit 3 — the honest answer while the patch is unapplied.

## A collision, disclosed rather than smoothed

At 00:43 ET another agent edited the same live scripts while I worked (it added the socket export, the real TERM/INT traps and
six cases). Its changes are correct; I re-verified each behaviour rather than trusting the edit, kept them, and the 32/32 suite
includes them. Nothing was reverted. Two agents writing one proof script is a harness smell: the file should be claimed first.

## What has not changed

The owner's list is the same and unchanged by any of this: apply `patch-0.3.6.2-lane-heal.sh`, quit and relaunch once (P3 fires
itself then, and takes the soak's restart sample), `--finish`, then step 5b (re-pin). `ext:delegator` retirement is still the
only post-switch gate failure, and `OWNER-PREVIEW-RETIRE-EXT-DELEGATOR.md` now shows exactly what typing `YES` would change,
including the two queued reports (seq 3, 7) that are the last mail that identity holds. Nox has never connected, the RSI leg is
still a relay with h78 unanswered, Crouton is still unproven.

P3 remains **armed, not proved**; no verdict exists and none is claimed.
