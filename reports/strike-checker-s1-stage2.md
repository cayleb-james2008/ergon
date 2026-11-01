# Strike S1 — Stage 2 acceptance check against the installed output

**Verdict: ACCEPT**

- Checker: `checker-install` (independent, fresh context; did not build or modify the artifact)
- Date: 2026-11-01, 16:43:22 → 16:45:59 EST (157 s), host `outis-desktop`, ran as `cayleb` (uid 1000)
- Done-when met: plugin's own Stage 2 check ran to **EXIT=0** with **60 PASS / 0 FAIL / 0 NOT-RUN**, against the running installed host; service independently verified back up afterwards.

## Command (exactly as the check header requires)

```
OUT=/home/cayleb/workspace/harness-build/evidence/strike/s1-stage2-check \
FLEET_CLI=/home/cayleb/workspace/harness-build/dsh-fleet-integration/package/bin/dsh-fleet.mjs \
bash /home/cayleb/workspace/harness-build/dsh-fleet-integration/checks/stage2-check.sh
```

## Exact result lines (verbatim)

```
summary: pass=60 fail=0 (evidence: /home/cayleb/workspace/harness-build/evidence/strike/s1-stage2-check)
STAGE 2: PASS
```
```
EXIT=0
trap: cancelled 70 of 1082 fixture orders (append-only rows remain by design)
```

Counts re-derived from the raw output: `^PASS ` = **60**, `^FAIL ` = **0**, no `NOT-RUN`, no `SKIP`, `console.err` empty (0 bytes), `stage2-check.out` byte-identical to `console.out`.

## Provenance / hashes

| Item | Value |
|---|---|
| Check script `checks/stage2-check.sh` sha256 | `7df6344c0846491e1130e9fb4c198aeed01de480b2ca47b03d9a5c3529fcca95` — **matches the expected hash** |
| Tree | `/home/cayleb/workspace/harness-build/dsh-fleet-integration` |
| Branch / commit | `integration/0.2.2` / `cf93107bc6e31ed265332b005d75b63d8acb300e` |
| Tree state | clean before **and** after the run (`git status --porcelain` empty; script + package.json hashes unchanged after) |
| `package/package.json` sha256 | `ef93187aec289d70bd29446c66c5b63abe44f78f4d2342871af2338274b4f52c` (package `@cayleb/dsh-fleet` version `0.2.2`) |
| Tarball the check selected (newest non-`-s1tamper` in `/var/lib/fleet-drop`) | `cayleb-dsh-fleet-0.2.2.tgz` |
| Tarball sha256 | `37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177` — matches its `.sha256` sidecar; **byte-identical (`cmp`)** to the accepted tree's `dist/` tarball |
| Installed package root (wrapper target) | `/var/lib/fleet/dsh/profiles/fleet/node_modules/@cayleb/dsh-fleet` — **not readable as cayleb** (parent `/var/lib/fleet` is `0750 fleet:fleet`) |

The check's own output confirms the tarball provenance it used: `tarball cayleb-dsh-fleet-0.2.2.tgz (sha256 matches sidecar); ./store -> ./src/orders/standalone.js`.

## Post-run service proof (independent of the check)

```
systemctl is-active fleet-dsh   -> active (rc 0)
ActiveState=active  SubState=running  User=fleet
MainPID=427587      InvocationID=9e3929d0d99c45ca98ff0d01fd275eb6
{"op":"status"} -> ok:true, result.integrity.ok:true, integrity.failures:[], pid:427587,
                   protocol:1, events:42653, policy.selftest:true, control.accepted:181 rejected:0
```

Pre-run baseline: MainPID `415240`, InvocationID `66d3f8b4eb3f40f59d1d39aaab5e1cc4`. The check's 2.11 leg SIGKILLed and restarted the unit (`MainPID 415240 -> 427587`, up again at +4 s); the unit stayed up, so the check's EXIT trap did not need to start it (no `trap: starting` line appears). InvocationID change proves the restart actually happened. No manual intervention was needed.

## Installed identity facts, read-only, from a cayleb seat

- `systemctl show fleet-dsh -p User,MainPID,InvocationID,ActiveState,SubState,ExecStart` — as above; `ExecStart` = `/var/lib/fleet/npm/bin/dsh --profile fleet --no-open --host 127.0.0.1 --port 3091` (pid 427587).
- `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin version` → `0.2.2-alpha.2` (rc 0). **This is the DSH app pin, not the plugin version** — do not read it as the plugin version.
- `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin status` → `uid=951(fleet) gid=950(fleet) groups=950(fleet)`; `/var/lib/fleet` and `/var/lib/fleet/dsh` are `drwxr-x--- fleet fleet`; then the same DSH pin (rc 0).
- **NOT readable from cayleb (recorded, not guessed):** `/var/lib/fleet` and its subdirs (`dsh`, `state`, `npm`, `log`) are `0750 fleet:fleet`; the installed package's `package.json` is `Permission denied`; `/run/fleet/` is not listable (the control socket itself is `srw-rw-rw-` and answers). `/var/lib/fleet-drop` is `0755 cayleb:cayleb` and readable.
- Available-but-not-run: a proposal-only identity snapshot caller is staged at `/run/dsh-fleet-id-cf93107/` (root 0444 `caller.py` + `identity_snapshot_core.py` + a 82-member `manifest.txt`; archive sha256 `37e3a542…`, the same tarball). Its header says **"PROPOSAL ONLY ... Do not ... run on the installed tree without the separate reviewed owner authorization"** — this checker did **not** run it; that authorisation is the owner's/Lead's call (relevant to S4's authorised read path).

## Anomalies / limitations (nothing softened)

1. **No failure, no NOT-RUN, no SKIP.** 60/60 PASS; verdict is unambiguous.
2. **The check exercises the running installed host via the control socket; it does not hash the installed tree's bytes** (it cannot from cayleb — `0750 fleet:fleet`). The installed-tree-equals-tarball comparison is therefore *not* established by S1 and must not be claimed from it; that is S4's job.
3. `dsh-fleet-admin version` prints the DSH pin `0.2.2-alpha.2`, not the plugin version; plugin `0.2.2` is evidenced by the tarball/package.json above. Anyone reading "0.2.2-alpha.2" as the plugin version would be wrong.
4. Duration was **157 s**, not "tens of minutes": the check is simply fast on this host (the 2.11 sweep-wait dominated). Not a defect; recorded so the number is not mistaken for a truncated run.
5. `run-timestamps.txt` contains a duplicated `start_epoch` line — cosmetic artifact of my wrapper (`start_epoch=1793569402` twice); both values identical, nothing else affected.
6. The check leaves append-only fixture rows by design and cancelled 70 of 1082 fixture orders at exit; the one non-fixture probe was cancelled inside leg 2.17. Expected per the check's own documented side effects.

## Evidence pack

`/home/cayleb/workspace/harness-build/evidence/strike/s1-stage2-check/` (13 MB, 49 files + `SHA256SUMS`; all files verify with `sha256sum -c`).

Key files: `stage2-check.out` (full output), `console.err` (0 bytes), `exit-code.txt`, `run-timestamps.txt`, `00-meta-before.txt`, `01-tarball-provenance.txt`, `10-identity-baseline.txt`, `20-post-run-service-proof.txt`, `30-pass-table.txt` (all 60 PASS lines verbatim), `ctl.jsonl` (every control request/response), per-leg `2.*`/`inv-*.jsonl`, `cleanup-cancel.jsonl`, `40-evidence-readme.md`, `SHA256SUMS`.

Spot hashes: `stage2-check.out` = `e018fbdd068441338ae1796f6884f1feaf4fbaec849909180d8651b0072e3f0b`; `00-meta-before.txt` = `84a6bba66e29fdd16ea1e2b8b2092a6efab69029fd6ab6e714eb8c8102cdc811`; `30-pass-table.txt` = `f49e5b496d9e752c382154a7ba03044abe1d38fda3b5ce74d4867cfbde113ced`.

## Fences respected

Did not modify the check, the tree, the installed profile, or the fleet DB; no spend/publication; no secret values recorded; no image reads/screenshots; the proposal-only identity caller was not executed; maker ≠ checker.
