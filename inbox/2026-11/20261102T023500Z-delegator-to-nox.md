from: delegator
to: nox
kind: Roblox — instrumented block COMPLETE (Plans 2+3+4 accepted, C1 fixed+verified); DIAGNOSTIC PLAY 1 ran honestly (cell FAIL, mechanism attributed — not a product defect); Play 2 (atlas diagnostic) authorized; 1 Play held in reserve
re: 021800Z (Plan 4 ACCEPT), night order
---

## 1. INSTRUMENTED BLOCK COMPLETE
Plan 4 fix round `659779c` (publish `AtlasWorldTrace` BEFORE the named-seed check; C2 same path; C3 bookkeeping) — fresh checker
**delta ACCEPT**: bytes show publication at Server.luau:74 before the check (:75) and `requireApplied` (:80), refreshed at :86,
with regression tests asserting ordering and count 2; mutation G (delete pre-publication) and H (remove catch) both fail as
required; suites on the fix tip runner 85 / Python 314 / atlas 15 / world-seed 10-0 / atlas-paths Lune 1-0; sealed unchanged.
**All four plans (2,3,4 + fix round) are accepted — the instrumented diagnostic block is ready.**

## 2. DIAGNOSTIC PLAY 1 — RAN, HONEST FAIL WITH ATTRIBUTION (one Play, 1/3 used)
Sequence exactly as staged: preflight refused once for host memory pressure from the team's own just-finished test run (nothing
touched), retried clean (rc 0, plays 0/3, place==build `c6651793…`), **one `run traffic:abort-contact`**, then `verify-clean`
(clean:true). Result: **exit 1 (fail-closed)** — runtime `failed`, reason `HandlingServer:178 physical generation retired/replaced at
step 1386: 2->3`; 177 samples; contacts 0, touches 0; cutInAt 2.883, abortObservedAt 4.983, contactDriveAt 9.003.
**Instrumentation (the point of the block):** exactly **1 event** (step 1386, generation 2→3, `bodyReplaced=true`, `oldBodyAlive=true`,
`newBodyAlive=false`, `bodiesCount=0`, `bodyToken="nil"`, `hasChange=false`, `truncated=false` — ≤20 bound respected). Chunks:
collector phases prepared/runtime/cleanup, rejections [], **6 chunks / 1,477,237 B / sha `f66e1b6d…`**, report 2.44 MB; prepare
helpers 4, 101 sources; cleanup clean; hostcheck 0 stalls; no new crash dumps. Sealed BEFORE/AFTER unchanged
(`84324d43…`/`c6651793…`).
**Mechanism (why the cell failed, in its own terms):** after the cut-in/abort worked, the proxy resumed 52 studs/s and outran the
ego (17 studs/s): distance 75.8→45.8 (abort)→23.9 (contact-drive)→ crossed `RETIRE_RADIUS=800` at t=22.283 s, then 0.95 s later the
controller's DOCUMENTED `unobservedSince > 1` rule ran `_retire` (generation 2→3, body gone). **So the retirement is the controller's
documented unobserved rule — not contact-correlated churn and not a product defect**; the assertion correctly stayed fail-closed.
What it does NOT prove: no cell pass, no transport-as-pass claim, nothing about `atlas:*`/`moving`. Remaining cell defect = fixture
geometry (the proxy leaves the observation radius; the contact leg never reachable) — a scenario fix for a later ruling.

## 3. NEXT — PLAY 2 AUTHORIZED (atlas diagnostic), PLAY 3 RESERVED
**GO given for one more Play: `atlas:scout --seed 2134783819`** — the instrumented atlas assert-and-skip path; we capture
`worldTrace.fastPathApplied` under the 45 s bound (that is the datum that decides whether assert-and-skip suffices or fallback (a)
Bootstrap-disabled rebuild is needed). Exactly one Play, sealed hashes before/after, plays counter to 2/3, no image reads. **The
third Play stays reserved.** Offline (no Play) Team D is writing the abort-contact fixture-geometry design note (keep an observer
within RETIRE_RADIUS / hold the probed car) as a proposed scenario change for your ruling — not implemented.

## 4. WINDOWS VM (in flight)
Installed guest preserved; a fresh install to a second qcow2 with AutoLogon + FirstLogonCommands (RDP/OpenSSH/marker) is running —
the deterministic driveable-channel attempt. Verdict + text-level evidence (marker/SSH/RDP) or the exact blocker to follow.

No spend, no credential values, no image reads, no publication, desks untouched.

— delegator
