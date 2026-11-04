from: plugin-finish (Stream 1 of o15)
to: nox
kind: status — session 3: the archive heal now records itself, the soak check can no longer be fooled, and three checker defects are closed
re: 20261104T052600Z-plugin-finish-to-nox.md (this adds; it does not supersede the morning record's owner list)
---

## Where the lane stands (unchanged where it matters)

Live at 05:55Z: Host pid 2313700, started 2026-11-04T03:46:14Z (22:46:14 EST Nov 3 — my earlier `…22:46:14Z` notes were the
`ps` local-time trap, corrected in the record), `op status` ok, integrity ok, 11 pinned / 12 disabled, one writer, installed
0.3.6.1, **no `.lane-heal-applied.json`**. `delegator` and `lead` idle. P2 stays PASS. **P3, the soak and the archive still need
the same two owner acts**: apply `patch-0.3.6.2-lane-heal.sh`, then one quit-and-relaunch.

## What changed tonight (branch `fix/lane-archive-heal` @ `ae2519a`, pushed)

1. **The heal records itself.** Every archive lift appends one hash-chained event — `lane.archive-lifted` /
   `lane.archive-lift-failed {session, via, lane}` — via `core.appendEvent`. `via` = `boot` | `session-stop`. After the
   relaunch, one SQL answers the question nobody could answer on 3 Nov: did the 22:51 archiver run again?
   `select seq, datetime(ts/1000,'unixepoch'), kind, data from events where kind like 'lane.archive-%'`. A broken event store
   produces a warning, never a lost resume; `core/test/api.test.mjs` pins both kinds as appendable.
2. **The soak check could have passed a soak that never happened.** It tested the three sheet-8 facts independently, so a smoke
   acked before the relaunch still read READY once a restart was sampled. The smoke must now follow the Host the samples ended
   on and postdate the retired folder; `soak_smoke_after_restart yes/no` prints both times. 11/11 behaviour tests
   (`soak-check.test.sh`), including the out-of-order case. `soak-sample.sh` also carried the same `ps lstart` TZ bug (21 legacy
   rows are 5 h early); fixed to `/proc/<pid>` mtime, and the check now flags those rows instead of believing them. A sampler
   runs 96 × 15 min under pid 2914548.
3. **The version is not valid semver — recorded, not renamed.** `0.3.6.2`/`0.3.6.1` are four numeric parts, so `npm publish`
   refuses them (confirmed against npm's semver; the file-copy installer is unaffected). `release/check-version.mjs` reports it on
   demand, names the two legal shapes and every file that embeds the version (`package.json` only). Decision: no rename tonight —
   the owner is about to review those exact bytes, and npm is not the ordered publish. Next version, if wanted: `0.3.7`/`0.4.0`
   (`0.3.6-2` sorts *below* `0.3.6`).

## Checks

**`CHECK-ARCHIVE-AUDIT.md`: `ARCHIVE AUDIT: PASS-WITH-LIMITS`**, then three re-check rounds; every finding closed with a test, no
new defect at the end:

| Finding | What it was | State |
|---|---|---|
| D1 | `API.md`/`host.js` named "`op events`", a control op that does not exist | **CLOSED** — both name the events table and the query |
| D2 | the sink guard was unpinned: deleting it kept 193/193 green (the caller's own catch masked it) | **CLOSED** — the test calls `liftArchive` directly and requires `true`; the mutant fails 9/10 |
| D3 | docs/tests promised a warning nothing observed (silent catch stayed green) | **CLOSED** — a collector requires one warning per lost event and matches the text; the silent-catch mutant fails |
| N1 | installer header quoted a stale test count | **CLOSED** — it names the suite instead |

Payload hashes were re-pinned after every change to the three shipping files, and `--check` was re-run against a **fresh copy of
the live profile** each time (`PATCH PLAN: OK`). Nothing was applied, installed, published or spent. The live profile still
hashes to the 0.3.6.1 bytes; the patch is staged.

## Still not mine to do

The owner's list in the 054000Z morning record (plus its session-3 addendum) is unchanged. Two coordination items are filed and
open for the face lead: `051500Z`-style note at **054500Z** (the face branch carries lane-heal at `14b92d0`; shipping code is at
`ae2519a` — re-merge before regenerating the 0.3.6 lists if cheap) and **055200Z** (the semver decision and the tool).
