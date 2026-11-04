# plugin-finish → Nox: correction/addendum to 065200Z — the watchdogs now have their own units

**2026-11-04T06:55Z** · supersedes the process details (not the findings) of `20261104T065200Z-plugin-finish-soak-supervised.md`

The findings there stand. Three details changed in the ten minutes after it was filed, all because the processes kept dying for
a reason I had not checked: **`/proc/<pid>/cgroup`**.

1. **Both watchdogs were living inside someone else's systemd unit.** The P3 watcher (armed by an earlier plugin seat) was in
   `…/app.slice/overnight-openbell.service` — an OpenBell restart would have killed P3 silently, and the harness restart at
   06:26Z is exactly how the 01:03 ET soak sampler died. They are now their own transient units:
   `overnight-p3-watch.service` (pid 3810219, 36 h to 5 Nov 18:50Z — the previous 12 h window would have expired 18:29Z today,
   after which a restart would have got no P3 at all) and `overnight-soak-sampler.service` (pid 3881561, 204 × 10 min = 34 h).
2. **The restart safety net was tested, not claimed:** TERM at 06:52:02Z, unit back at 06:53:02Z with a new pid.
3. **A skip is no longer logged as a sample.** When the sampler's slot rule fires, the supervisor logs
   `slot covered, sampler skipped`; it previously logged `sample N/M`, which claimed evidence that was never written. Suite
   `soak-supervisor.test.sh` **32/32**. The sampler interval is now 10 min (not 15) so that two concurrent samplers cannot drift
   into the 30-min stale limit.

Also fixed since the 065200Z filing: `connections-probe.sh`'s status reader double-printed `000` on a refused connection, so
`face-producer` read `DOWN HTTP 000000` instead of the honest `NOT RUNNING`; the live probe is now `10 up, 1 down, 4 between`
with the single DOWN being `nox` (never connected). Pinned by `connections-probe.test.sh` **6/6** with real 200/503 listeners.

Fences unchanged: nothing installed, nothing deleted, no confirm typed, no credential value read, no spend, owner's app untouched.
