# S7 re-verify verdict — Ergon rehearsal, builder fix 003d9ef (installed 0.2.2 platform)

Checker: `checker-verify` (fresh-context; built none of Ergon, the fleet, the driver or the fixtures). I drove both passes
myself from a fresh pack of my own and verified with my own checker; the driver's own gates were cross-checked, not
trusted.

**VERDICT: ACCEPT** — the previous S7 rejection (stale `commands.log` / `logs/index.log` manifest entries) is fixed and
verified independently. Both passes seal a manifest that covers exactly the on-disk files with zero missing/extra/stale,
`sha256sum -c` verifies 72/72 and 74/74, nothing inside a pass directory is newer than its sealed manifest, the verdict
tables are identical, 0 failed, 4 clean / 4 pending-named, R5 orders verified on the live fleet board, secrets clean, and
R0 now resolves the installed release from the install's own audit line (staged 0.3.0 listed as not installed).

## Artifact under test

| | |
|---|---|
| Driver | `/home/cayleb/workspace/harness-build/ergon-strike/docs/acceptance/demo-company/rehearsal/run-rehearsal.sh`, branch `strike/ergon-rehearsal` @ **`003d9ef2265af313d4fbde796b1827dbe5b72b5f`** (on top of `e284135`); worktree clean |
| Driver sha256 | `ffd6b2ff2f9373125633638cd7c34c080b21976a46697381adc9aa83fd6d591d` |
| Seal module | `lib/seal.mjs` sha256 `6a26146e1669eca23a5701eabc6e31545f1cc4b25877a1868eb7f21630bf3150` |
| Platform | installed `@cayleb/dsh-fleet` **0.2.2** as `fleet-dsh` (integrity ok / lanes true / selftest on; `MainPID 427587`, `InvocationID 9e3929d0…`, unchanged before/after) |
| My runs | both passes, my own fresh pack `s7-rerun/ergon-pack`; no `--force`, nothing copied from the builder's pack |

## My two from-scratch passes (driver transcripts in `02-pass1.out`, `04-pass2.out`)

| | pass 1 | pass 2 |
|---|---|---|
| Exit | **0** (`REHEARSAL: PASS`) | **0** (`REHEARSAL: PASS`) |
| Started / finished (UTC) | 22:35:20Z / 22:35:22Z | 22:35:27Z / 22:35:29Z |
| Fresh company root | mktemp, removed by R7 teardown | mktemp, removed |
| Seal | `manifest 72/72 OK` · manifestSha256 `b3cb6d40a383177dcf905693fc2f7651876de29f820f3b64d19ccfbd858e2422` · indexSha256 `32bd26028a4d6ed703c2b1d5ae5644ff14ab3e9f51e5f95f542c089cea2424a3` | `manifest 74/74 OK` · manifestSha256 `17bfa5c5478bc1906a4739493e0ee9b831f4e746b78ec51482888ee2d8e0c21a` · indexSha256 `e39f002ab867f878b99a9962aecdabbfd12f9454c4c161bc9a995bb42703cc62` |
| `sha256sum -c` (driver gate) | 72/72 OK | 74/74 OK |
| Words | clean 4 (R0,R1,R5,R7) · pending 4 named (R2,R3,R4,R6) · **failed 0** | identical |
| Tables | — | `verdict-table.diff` empty (`PASS DIFF: IDENTICAL`; `PACK.md` **yes**) |
| Teardown | root emptied, no `/tmp/ergon-rehearsal-*` leftovers | same |
| Secrets | pre-seal clean, post-seal clean (driver) + my independent scan clean | same |

## The Lead's checklist, machine-checked independently

| # | Check | Result |
|---|---|---|
| (a) | `sha256sum -c manifest.sha256` both passes, run by me from the pass dirs | **72/72 OK, exit 0** and **74/74 OK, exit 0**; 0 `FAILED` lines (`05-sha256sum-c.txt`, 146 OK lines total) |
| (b) | Manifest covers exactly the on-disk files; no included file newer than the manifest | pass 1: listed 72 = on-disk 72, **missing 0 / extra 0 / stale 0**; pass 2: listed 74 = on-disk 74, **0/0/0**; `included_files_newer_than_manifest: 0` both passes (manifest mtimes 22:35:22.631Z / 22:35:29.548Z) — `06-coverage-mtime.txt` |
| (c) | The old E7 class is gone (`commands.log`, `logs/index.log`) | both now match their manifest entries in both passes: pass 1 `commands.log 26801a2a… == 26801a2a…`, `logs/index.log 545859d0… == 545859d0…`; pass 2 `34c0cb5c… == 34c0cb5c…`, `6cc7b266… == 6cc7b266…` (`06-coverage-mtime.txt`) |
| (d) | Verdict-table identity, 0 failed, pending named, no pending recorded clean | tables byte-identical; counts clean 4 / pending 4 / failed 0; every pending item appears named in the index; clean steps have hard `ok:true` checks with empty pending/failures; every data word equals its index row (my verifier E2/E3/E4/E5/E6 all pass) — `09-verdicts-and-seals.txt` |
| (e) | R5 order ids verified read-only on the live control socket | my orders **`o3284`** (pass 1) and **`o3285`** (pass 2): state `done`, `fixture:true`, tasks done+banked, final tasks done (`07-r5-orders-control.json`); also verified the builder's re-run orders `o3282`/`o3283` read-only: done, fixture, tasks banked (`07b-builder-orders.txt`) |
| (f) | Independent secrets scan | 161+ pack files, 11 own patterns + owner-profile-path check, **0 hits**; driver's pre-seal and post-seal scans also clean (`checker-verify/secrets-scan.txt`) |
| (g) | R0 is version-agnostic | `identity_source: journald install audit line (dsh-fleet-admin plugin-add)`; audit line names `cayleb-dsh-fleet-0.2.2.tgz sha256=37e3a542…`; R0 re-hashes that tarball → `verified:true`; drop inventory lists **`cayleb-dsh-fleet-0.3.0.tgz` (sha256 b60019a0…, no sidecar) as staged-only, not the installed release**; no hardcoded release filename/path in `run-rehearsal.sh` (`08-r0-version-agnostic.txt`) |

My verifier (`probes/ergon-check.mjs verify`, extended for this re-check with full manifest coverage, seal/mtime, and
version-agnostic R0 checks): **113/113 checks passed · 0 failed · 0 informational → ACCEPT**
(`checker-verify/ergon-verify.json`, `verdicts.json`). Structure check of the fixed driver: 23/24 passed, 0 failed, 1
informational (`structure/`).

## Minor finding (not blocking)

`run-rehearsal.sh:91` still prints `"platform": "installed @cayleb/dsh-fleet 0.2.2 on this desktop"` in `run-meta.json`.
R0's identity is properly version-agnostic and correct, but this cosmetic field would misstate "0.2.2" after the 0.3.0
install. Recommend deriving it from R0's resolved identity on the next touch. Recorded, not a reason to reject this
rehearsal.

## Limits (named, not smoothed)

1. This rehearsal ran on the installed **0.2.2** platform host; the third run on the final 0.3.0 install remains queued
   behind the owner act.
2. R2's "one real fleet agent session per cast member" ids still cannot be re-read post-hoc on a host without the MCP
   facade / readable session store — I recorded that limit rather than passing it.
3. The rehearsal is the platform-host pass; the fresh-cloud-host/DNS/Supabase legs are named out of scope by the driver
   itself (R0/R3 evidence).

## Raw evidence (`s7-rerun/`)

`00-rerun-meta.txt` (artifact identity) · `01-pass1-timing.txt`, `03-pass2-timing.txt` · `02-pass1.out`, `04-pass2.out`
(my run transcripts) · `ergon-pack/pass-1|pass-2/` (sealed pack: manifests, indexes, data, step evidence, `commands.log`,
`index-summary.json`) · `ergon-pack/PACK.md`, `pass-diff.md`, `verdict-table.diff`, `manifest-check-pass-1|2.txt`,
`manifest-verify-pass-1|2.txt`, `seal-pass-1|2.log`, `pack-commands.log`, `secrets-scan-*.txt` ·
`05-sha256sum-c.txt` · `06-coverage-mtime.txt` · `07-r5-orders-control.json` + `07-r5-summary.txt` ·
`07b-builder-orders.txt` · `08-r0-version-agnostic.txt` · `09-verdicts-and-seals.txt` · `10-fleet-health.txt` ·
`checker-verify/` (my 113 checks, verdicts, secrets scan) · `structure/` · `SHA256SUMS`.
