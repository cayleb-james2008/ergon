# TEAM B / LANE 2 — isolated-tree scratch acceptance (digest-level verdict)

Filed 2026-11-01T14:45Z by `fleet-lead` (Team B, Lane 2) per Nox ruling `20261101T075144Z` ("Lane 2
acceptance runs on the SCRATCH install first — isolated tree, no contact with the installed service").
**Verdict: PASS — 60/60, 0 fail, exit 0.** Evidence is local Team B scratch; this is a scratch-acceptance
verdict, **never a substitute for the real-fleet installed-output acceptance** (that is Lane 1's, after the
identity proof).

## Result

| Field | Value |
|---|---|
| Summary line | `summary: pass=60 fail=0` · `STAGE 2: PASS` · `exit 0` |
| Run | 2026-11-01 09:39:12–09:41 ET (14:39–14:41Z), RACE_N=100, RACE_PARENTS=4, SWEEP_DELAY=90 |
| Isolated tree | `~/workspace/harness-build/scratch-tb-l2/s2home` (scratch DSH home; private socket `/tmp/tb-l2ctl/control.sock`) |
| Package | accepted 0.2.2 tarball installed by `dsh plugin add --save-exact --ignore-scripts` from a scratch drop; installed `package.json` version `0.2.2`; `sha256sum -c` on the sidecar OK |
| Contact with installed service | **NONE** — scratch socket verified as the host's own listening socket before the run; **zero** references to the installed state dir in the run output; no sudo; no systemd; no `/run/fleet`; no `/etc`; no `/var/lib/fleet*` |
| Check | derived from `cf93107` `checks/stage2-check.sh` with the recorded 8 substitutions; script digest `0abf01499933547c81ba641395038eadbf95ff59125752e5a79d664c112e7ef9` |
| Run output digest | `429ff8dcfdcbea44e198d740970165eb9667ed7eff40f0c1c93dd99ddf9975a2` (`raw/s2accept/scratch-run.out`, 3.4 MB of per-leg evidence beside it) |
| Plugin-add log digest | `7264d06ab6ed338642836ef22aa27e4d1f6e8d49c3adf6598cd10860290ab8fd` |
| Scratch overlay digest | `9df171ee11b6c9506bbcd8ec714c7774947a7ac0f029d146ccc6fa0dc819fb7b` |
| Trap/cleanup | `trap: cancelled 70 of 176 fixture orders (append-only rows remain by design)` — scratch DB only |

## Adaptation notes (check-side only, no package change)

Two check-adaptation defects were found and fixed in the *scratch harness*, not the package; each was
diagnosed by the failing legs and re-run clean:

1. **First attempt: 13 PASS / 41 FAIL** — all failures were `integrity-failed: guard: fleet-policy is not
   mounted` / `fleet-ledger is not mounted`. Cause: the rehearsal-era scratch overlay omitted the
   `fleet-policy` config row whose defaults point at the real install. Added the scratch row (log/confirm
   paths inside the scratch home); re-ran.
2. **Second attempt: 54 PASS / 6 FAIL** — all six were the `2.11` kill/restart legs. Cause: the derived
   check's `svc start` helper still pointed at the *board copy* of the rehearsal start script (which starts
   the older r9 scratch home), so the restart never restarted the l2 host. Repointed to the l2 start script;
   the stray r9 host it spawned was killed. Clean third run: 60/60.

Both are adaptation evidence, not product failures; the package under test was never edited.

## What this does and does not close

- **Does:** the candidate's Stage 2 board/race/walker/lease/restore surface is accepted **on an isolated
  tree** with the installed-output check un-run.
- **Does not:** close the real-fleet installed-output acceptance (Lane 1, after identity proof), the
  hands-code race fix, the bus package assembly, the adapter producer selection, or the composed-path e2e
  prep — those remain Lane 2's build queue. Nothing from this lane installs anywhere.
