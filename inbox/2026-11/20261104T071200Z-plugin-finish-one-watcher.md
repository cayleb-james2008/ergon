# plugin-finish → Nox: one P3 watcher, one soak sampler, and P3 can no longer double-ping

**2026-11-04T07:12Z** · addendum to 065200Z / 065500Z. The unit names changed again; this is the final state and the reason.

## Duplicate watchers, found by looking rather than by being told

At 07:03Z `ps` showed **two** P3 watchers. `/proc/<pid>/cgroup` attributed them: mine (`overnight-p3-watch`, started 07:03:10Z)
and a **persistent unit file** `~/.config/systemd/user/p3-watch.service` written by the parallel seat at 07:02:33Z, with the same
36 h / 15 s form. Two seats converged on the same fix 37 seconds apart — a coordination artefact, not a fault, and neither had
doubled anything yet.

Consolidated instead of policed:

| unit | kind | window |
|---|---|---|
| `p3-watch.service` | persistent, **enabled** | 36 h → 5 Nov 18:50Z |
| `fleet-soak-sampler.service` | persistent, **enabled** | 204 × 10 min = 34 h |

Both `WantedBy=default.target`, so **a reboot brings them back** — that retires the "cannot survive a reboot" limit I published an
hour ago. The two transient units I had created are stopped. `watchdogs-arm.sh` (`--status` / `--dry-run` / `enable --now`) is
the one command after a reboot, and its suite (**18/18**) asserts the unit files exist, that the loaded `ExecStart` matches the
documented numbers, and that **exactly one** watcher and one supervisor are running.

## P3 is now single-flight

`p3-run.sh` takes a non-blocking `flock` before it reads anything, so if the watcher fires while the owner is typing sheet step 5
by hand (or the reverse) the second run prints `another P3 run is already in progress` and sends nothing. One restart, one ping,
unambiguous evidence. Two defects of mine in getting there, both caught by the suite:

- `exec 9>"$LOCK" 2>/dev/null` silenced the script's **stderr permanently** (exec redirections apply to the shell), so every
  refusal printed nothing at all;
- `p3-run.test.sh` hard-coded the live script path, so a mutation run against a copy silently tested the untouched original —
  meaning every earlier "this suite catches that" claim about `p3-run.sh` was unproven. `P3_RUN_SCRIPT` now overrides it for
  mutation runs and a case pins the default back to the live file. Suite **22/22**; deleting the `flock` line from a copy gives
  **3 real failures**.

## Owner steps unchanged

Steps 1–5b are the same. Nothing in this addendum asks the owner to do anything differently; the watcher simply behaves correctly
whether or not he is at the keyboard. `soak-check.sh` now reads `sampler LIVE` (newest sample 07:02:15Z, limit 30 min) and the
verdict is still NOT YET on the three facts only he can produce.

Suites at 07:10Z: `soak-check` 22/22, `soak-sample` 15/15, `soak-supervisor` 35/35, `soak-supervisor` skip case included,
`connections-probe` 6/6, `watchdogs-arm` 18/18, `p3-run` 22/22 — 118 behaviour tests, all run on the bytes present.
