# Morning record — plugin-finish overnight session 2 (4 Nov 2026, 00:00–00:40 ET / 05:00–05:40Z)

Supersedes the 052100Z record's status lines; that record's history and the checker verdicts it names are unchanged.

## In one paragraph

P2's fix is in and checked. P3, the soak and the retired-profile archive still need exactly one thing from the owner — apply
`patch-0.3.6.2-lane-heal.sh`, then quit and relaunch the app — and **P3 now proves itself when that happens**: a watcher is
armed on the Host's pid and start time. Two more traps were closed or made measurable: the profile's dependency pin (which
would have downgraded the live fleet on any `pnpm install`) now has a rehearsed, guarded script waiting, and the connections
audit is a single command instead of a snapshot. The published repo carries the goal doc and a status paragraph re-read from
the host. Three independent checks ran; the sharpest caught a timezone bug in my own P3 scripts that would have made P3
impossible to pass while looking like caution. Fences held: no spend tonight, no post but the ordered private publish, no
credential value, retired profile untouched, no owner app signalled.

## The owner's list, in order

1. `cd /home/cayleb/workspace/harness-build/fin-elevfix && FLEET_INSTALL_CONFIRM_LIVE=yes-i-am-the-operator-with-nox-go bash release/patch-0.3.6.2-lane-heal.sh --check` then `--apply`
   (detail: `OWNER-RUN-LANE-HEAL-20261104.md` §1–2).
2. Quit the app completely, wait a minute, start it as usual. **Nothing else is needed for P3** — `p3-watch.sh` is armed and
   will send the one ping, poll to `acked`, record it and take the soak's restart sample by itself. Check it any time with
   `~/fleet-scratch/overnight/p3-watch.sh --status` (0 = P3 eligible now, 3 = not yet, and it says why).
3. After that: `... patch-0.3.6.2-lane-heal.sh --finish` (removes the bridge row `lane-host-heal3`), then step 5b
   (`release/pin-installed-fleet.sh --check` then `--apply --target-live`) so the profile pins 0.3.6.2 instead of 0.3.3.
4. The pack's step 3 — retire `ext:delegator` (typed `YES`, real terminal) — is still the only post-switch gate failure.
   `OWNER-PREVIEW-RETIRE-EXT-DELEGATOR.md` shows what it will change, including the two still-queued reports (seq 3, 7) that
   are the last mail that identity holds.
5. Then G-C (Q16 key copy, Q8), the archive, the soak (a day + a restart) and the deletion — the last is yours alone.

## What is proved, and what is only staged

| Thing | State |
|---|---|
| P2 wake proof | **PASS**, independently checked (`CHECK-P2.md`, `verdict-P2.json`) |
| P3 wake proof | **ARMED, not proved** — fires on the owner's next restart; no result exists yet and none is claimed |
| Lane-archive fix | code on `fix/lane-archive-heal` (0.3.6.2, 192/192 at `47fceeb`, `CHECK-LANE-HEAL.md`); **not installed** |
| Profile pin | script rehearsed on live-shaped copies (26/26 tests); **not applied** — belongs after the patch |
| Crash fix | staged (`stage/install-reliability.sh --check` passes today; watchdog is running and reaping stray scopes) |
| Face panels | face lead owns the merge; my producer + live render handed over (`FACE-HANDOFF-plugin-lead.md`) |
| Publish | `origin/main` = `bd38123`, private, docs-only above `38cb1e7`; goal verbatim at the top, status paragraph re-read |
| Connections audit | 15 rows, now reproducible: `connections-probe.sh` (11 UP, 1 DOWN, 3 between at 05:12Z) |
| Soak | measurable (`soak-sample.sh` / `soak-check.sh`): days_aside 0, 1 Host, no smoke → **NOT YET**, correctly |

## A collision to know about

At 00:43 ET, while this session worked, **another agent edited the same live P3 scripts** (it added the socket export, real
TERM/INT traps and six cases). Its changes are correct; I re-verified each behaviour myself instead of trusting the edit, kept
them, and the 32/32 suite includes them. Nothing was reverted. Two agents writing one proof script is a harness smell — the file
should be claimed before it is edited.

The branch shows the same thing: a **parallel plugin-finish session** (same author identity) added `4b112b9`, `a7e4fce` and
`b90136a` to `fix/lane-archive-heal` while I worked, with its own checker (`CHECK-ARCHIVE-AUDIT.md`). I verified the two things
the owner depends on: the roster suite is **193/193** at `b90136a`, and `patch-0.3.6.2-lane-heal.sh --check` against the live
profile still says **PATCH PLAN: OK** with the payload pins matching the new bytes. Nothing was reverted or duplicated; the
branch is pushable and installable. Nox should name one owner for it, because two sessions committing one branch under one
identity is how a good tree gets a bad merge.

## The three checks, and the one that mattered

- `CHECK-P3-WATCH-AND-PIN.md` — **PASS-WITH-LIMITS**; seven real defects in my watcher/pin scripts, all fixed with tests.
- `CHECK-GOAL-STATUS.md` — **PASS-WITH-LIMITS**; four overstatements in the published paragraph, all corrected.
- `CHECK-GOAL-STATUS-R2.md` — **FAIL**, and it was right: the citation path was wrong, and both P3 scripts computed the Host's
  start with `date -u -d "$(ps -o lstart=)"`, which GNU date parses as UTC while `ps` prints local time — 18,000 s early. Since
  `p3-run.sh` compares that to a true-UTC `applied_at`, it would have refused every post-patch restart **forever**, and because
  the guard fails closed it looked like correct caution. Fixed (`/proc/<pid>` mtime), with two regression tests; `p3-watch.test.sh`
  is 32/32 and the live `--status` now prints `started=2026-11-04T03:46:14Z`, which agrees with uptime.
- `CHECK-TZ-FIX-R3.md` — **PASS**: `/proc` start 1793763974 vs uptime arithmetic 1793763975 (1 s), both real-host directions
  proved with scratch markers, docs citation and bridge-row name verified, profile untouched.
- After R3, two more of the same class were closed: a **readiness gate** (the Host exists before its fleet core answers, so the
  watcher now waits for `op status`, and it exports the per-user socket because the CLI's default points at a root-run fleet
  this box does not have — otherwise P3 would be refused on every restart) and **TERM/INT that actually exit** (the first trap
  only cleaned up, so stopping the watcher looked like a hang). Suite `p3-watch.test.sh` is **32/32**.

## What I could not remove, said plainly

- The 22:51 ET archiver is still unidentified. The fix guarantees a lane comes back; it does not stop a caller that forces
  `stopActivity`. The actor log is a DSH-repo change, not a plugin one.
- Crouton Alpha stays **unproven**: the ref resolves, but no answered call exists and the package carries no engine-chain
  config. One call would settle it; that is an owner act.
- `nox` has never connected to the bus and the owner principal has never pulled (37 pending, head `h10`); the RSI leg is a
  manual board relay with h78 still unanswered. None of that is mine to fix.
- The delegator lane's runtime context still presents it as "lead" with Agent Teams tools; it refused the identity and filed
  h85 — Nox's ruling, not mine.

---

## Addendum — session 3 (00:45–01:05 ET / 05:45–06:05Z)

Nothing in the list above changed: the same two owner acts unblock P3, the soak and the archive. Three things were added, all in
the lane's own tools, none of them installing or spending anything.

1. **The archive heal now leaves a dated record.** The one open question from the 22:51 archive was "did it happen again?", and
   the answer used to die with the Host because the lift only wrote a console line. Every lift now appends one hash-chained
   event — `lane.archive-lifted` / `lane.archive-lift-failed` with `{session, via, lane}` — through the same `core.appendEvent`
   path as every other roster event. `via` says `boot` or `session-stop`. After your relaunch, one query answers it:
   ```bash
   sqlite3 "file:$DH/fleet/state/fleet.db?mode=ro" "select seq, datetime(ts/1000,'unixepoch'), kind, data from events where kind like 'lane.archive-%' order by seq desc limit 20"
   ```
   A `via=boot` row means the archive was still on a lane at boot (the incident repeating). No row means it did not recur.
   A broken event store can only produce a warning — it can never cost a lane its resume. 193/193 roster tests; the installer's
   payload pin was re-hashed, so the patch you apply is exactly the reviewed bytes.
2. **The soak check could have said READY for a soak that never happened.** It tested the three sheet-8 facts independently, so
   a smoke acked *before* the relaunch would still pass once a restart had been sampled. It now requires the smoke to follow the
   Host the samples ended on and to postdate the retired folder, and prints `soak_smoke_after_restart yes/no` with both times.
   11/11 behaviour tests (`soak-check.test.sh`), including the out-of-order case. A sampler is running now (15 min cadence, 24 h
   window, writes only `soak-samples.jsonl`) so the day has a real trace instead of one sample per manual run. It also fixed the
   same `ps lstart` timezone bug in `soak-sample.sh` (the 21 older rows are wrong by 5 h and the check now says so rather than
   believing them).
3. **The version is not valid semver, and it is not renamed tonight.** `0.3.6.2` (and the installed `0.3.6.1`) are four numeric
   parts; npm refuses `npm publish` for them. Confirmed against npm's own semver. The supported path — the file-copy installer —
   does not care, and npm is not the publish the order asked for, so changing the bytes of a release you are about to review at
   06:00 would be the wrong trade. `release/check-version.mjs` reports it on demand (exit 0/1/2, `--self-test` 9 cases, and it
   names every file that embeds the version: `package.json` only). If npm-installability matters to you, the honest next version
   is `0.3.7` or `0.4.0` (`0.3.6-2` would sort *below* `0.3.6`) and the lists need regenerating after it.

Branch state: `fix/lane-archive-heal` = **`a7e4fce`** (pushed), `origin/main` still the published docs
tip `bd38123`, live install still 0.3.6.1 with no `.lane-heal-applied.json`, Host pid 2313700 (started 2026-11-04T03:46:14Z),
integrity ok, 11 pinned, both lanes idle, one writer, `p3-watch.sh` armed. A separate checker session is reviewing the audit
change (`CHECK-ARCHIVE-AUDIT.md`); its verdict is not written by me.

One correction owed to the record: several of my earlier notes wrote the Host's start as `2026-11-03T22:46:14Z`, because `ps`
prints local time and I read it as UTC — the same 5-hour trap the R2 checker found in the p3 scripts. The true start is
**2026-11-04T03:46:14Z**. The p3 scripts were already fixed; the log lines and one evidence table are corrected here.

---

## Addendum — seat A (05:40–06:20Z): the by-hand path, and the page that makes the chain checkable

A second plugin seat was spawned by the 05:39Z restart while this one was running; the split is written in
`CLAIM-p3-scripts-20261104T0552Z.md` and the board's primary note stays seat B's `055500Z`. This addendum adds only the part
that note does not cover.

- **`p3-run.sh` now waits for the fleet core too** (default 120 s, `--ready-timeout`, fails closed with "no ping was sent").
  The watcher's gate only protects the automatic run; typing sheet 6.4 straight after the relaunch would still have died on
  `ENOENT` with no ping and no message. It also takes `--probe CMD`, so its two guards are testable without the live app, and
  its socket default is `id -u` rather than a hard-coded `1000`.
- **New suite `p3-run.test.sh` 17/17** (stub `node` on `PATH`: pre-patch Host refused, missing marker refused before any op,
  wait-then-send, fail-closed, electron-only probe, `/proc` start, per-user socket). Final bytes, both suites:
  `p3-watch.test.sh` **32/32**, `p3-run.test.sh` **17/17**.
- **One vacuous assertion made real**: "cache file removed on exit" checked a path nothing wrote; the `applied_at` cache now
  sits beside `--state`.
- **`evidence/P3-ARMED.md`** is the one-page chain (marker → restart → readiness → one ping → ack → checker verdict), with the
  four scripts' sha256s, the live Host facts, the three defects found by three different sessions, and an explicit list of what
  is **not** claimed (no P3 verdict, no soak, no install).
- **Scope re-check at 06:15Z** (`evidence/SCOPE-RECHECK-0615Z.md`): connections 10 up / 2 down / 3 between — the extra DOWN is
  the headless *proof* producer row, not a broken service; crash-fix stage `--check` exit 0 with its watchdog still reaping
  stray scopes at 00:58:14 ET; the retired profile intact at 328 MB; `roster_wakes` 57, newest 76 m ago, newest ack still seq 84.

Nothing installed, nothing deleted, no confirm typed, no credential value read, no spend this session.
