# S7 verdict — independent Ergon dress-rehearsal re-run (installed 0.2.2 platform)

Checker: `checker-verify` (fresh-context; built none of Ergon, the fleet, the driver or the fixtures). Maker never
checker: I did not edit the driver, the pack or the fixtures. I drove the rehearsal myself from the builder's branch and
then verified the pack with my own checker (`probes/ergon-check.mjs`), independently of the driver's own scans.

**VERDICT: REJECT** — narrow and actionable: the evidence pack's integrity claim fails. The pass indexes state "Every
evidence artefact's SHA-256 is in `manifest.sha256`", but in **both passes** two artefacts are listed with stale hashes
(`commands.log`, `logs/index.log`), so the manifest does not verify. Every rehearsal *claim* I could reproduce did
reproduce: both passes ran exit 0, 0 failed, 4 clean / 4 pending-named, identical verdict tables, R5 orders verified on
the live fleet board, R0 identity matches the released 0.2.2 tarball, secrets clean, fleet install untouched. The
rejection is about the pack being internally consistent enough to sign — not about the rehearsal content. Fix: build the
manifest **last** (after `commands.log`, `logs/index.log`, the final secrets scan and the R7 diff are final) or exclude
the two appended logs from the manifest and say so in the index; then re-run both passes.

## Artifact and platform

| | |
|---|---|
| Driver | `docs/acceptance/demo-company/rehearsal/run-rehearsal.sh` in `/home/cayleb/workspace/harness-build/ergon-strike`, branch `strike/ergon-rehearsal` @ `e284135264f4025a1527c6754261b23907620dea`; worktree clean; driver sha256 `85f66c1a23c59f7d62f0572bb5b81e6f7ff138e8275201286d08ad9c0bea50f9` |
| Platform | installed `@cayleb/dsh-fleet` **0.2.2** as `fleet-dsh`; control socket `/run/fleet/control.sock` (peer uid 1000); integrity ok / lanes true / policy selftest on. (0.3.0 is not installed; the rehearsal-on-final-install re-run remains queued — this is the platform that exists today, as the Lead stated.) |
| Runs | mine, from a fresh pack dir of my own (`s7/ergon-pack`), two invocations as the driver's design requires |
| Rules observed | read-only wrt the install: no restart, no install, no unit change, no owner-profile access; the only fleet writes are the driver's documented fixture orders that roll up `done` |

## My two from-scratch passes

| | pass 1 | pass 2 |
|---|---|---|
| Command | `bash docs/acceptance/demo-company/rehearsal/run-rehearsal.sh --pass 1 --out <fresh pack>` | same, `--pass 2 --out <same pack>` |
| Exit | **0** (`REHEARSAL: PASS`) | **0** (`REHEARSAL: PASS`) |
| Started / finished (UTC) | 22:22:55Z / 22:22:58Z | 22:23:36Z / 22:23:40Z |
| Fresh company root | `/tmp/ergon-rehearsal-20261101T222255Z-wqWAOs` (mktemp, removed by R7 teardown) | `/tmp/ergon-rehearsal-20261101T222336Z-KWQcWq` (mktemp, removed) |
| Index sha256 | `cff73dc1049c2d9540df43a584fe7f74b8cfe2921b5536357fb169551c1e8aeb` | `f5f012ecd90053a915d09657844e29a0041b3adc8e88281346a9ae24553b9e22` |
| Words | clean 4 (R0,R1,R5,R7) · pending 4 (R2,R3,R4,R6) · **failed 0** | identical |
| Pending named | R2 provider credential (owner act); R3 hosted URL/DNS/second device; R4 phone surface; R6 voice surface — each named and counted, never clean | identical |
| Verdict tables | — | byte-identical to pass 1 (`PASS DIFF: IDENTICAL`; `verdict-table.diff` empty; `PACK.md` says **yes**) |
| Teardown | root emptied, no `/tmp/ergon-rehearsal-*` leftovers | same |

Post-run fleet facts: `fleet-dsh` still `active`, `MainPID=427587`, `InvocationID=9e3929d0d99c45ca98ff0d01fd275eb6` —
unchanged from the pre-run baseline in the earlier S1 evidence pack (16:47Z). No restart, no install, no non-fixture
order. The only new rows are the two passes' fixture orders and their posts (append-only board).

## Independent verification of the pack (`ergon-check.mjs verify`)

81 checks · 79 passed · **2 failed** (the manifest integrity checks) · 0 informational. Machine report:
`s7/checker-verify/ergon-verify.json`.

Passed (my checks, not the driver's): both pass dirs + run-meta with real mktemp roots/commit/start times; index verdict
table rows R0–R7; every data word equals its index row; 0 failed; every `pending` item present and named in the index;
counts line equals per-step words; every clean step has hard `ok:true` checks with empty pending/failures; every
evidence file exists; all mtimes inside the run window (no pre-filled evidence); pass-2 table identical; R7 pass-2
carries `meta.passDiff.identical=true`; driver secrets scans present and clean; manifest exists; index-summary agrees;
R0 identity digest matches `cayleb-dsh-fleet-0.2.2.tgz` = `37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177`
(the published sidecar; I hashed the tarball myself); commands.log exists with the step invocations, all exit 0; R5
claimed order id exists on the live control plane; index names the per-run values (not a template).

**Failed — the only failures:**

| Check | Evidence |
|---|---|
| `P1.E7.manifest` | `pass-1/manifest.sha256` lists `commands.log` as `97799916…` but the file hashes to `4d38ad18…`; lists `logs/index.log` as `e3b0c442…` (the empty-file hash) but the file hashes to `e5b24a20…`. Both files are appended *after* index-build writes the manifest (commands.log gets the index-build/final-scan/rmdir rows; logs/index.log is still being written by node while the manifest is computed). |
| `P2.E7.manifest` | same two files, same cause (74 entries; pass-1 had 72). |

The index therefore makes a false integrity statement for 2 artefacts per pass. That is exactly the kind of
index-vs-files-vs-commands inconsistency a checker cannot sign. No other hash entry is stale: every step evidence file
listed in `evidence[]` verifies, and the manifest file's own digest matches `index-summary.json`.

## Independent reproductions of the pack's claims

| Claim from the pack | My independent check | Result |
|---|---|---|
| R0 installed identity 0.2.2 | `sha256sum /var/lib/fleet-drop/cayleb-dsh-fleet-0.2.2.tgz` + sidecar; compare to R0 `identityDigest` | matches `37e3a542…`, `published_matches:true` |
| R5 pass-1 fleet order `o3277` done/fixture | my `ctl.mjs order-get o3277` over `/run/fleet/control.sock` | `state:done`, `fixture:true`, task `t4144` done+banked, final `t4145` done, `report_post_id 7335`; posts `7327…7335` include `7331`,`7333`,`7335` |
| R5 pass-2 order `o3278` | same | `state:done`, `fixture:true`, tasks `t4146` done+banked, `t4147` final done |
| R5 claims a real warehouse-board task | readback + order row labels | fixture-scoped (`fixture:true`), `real_lane: not-built (Stage 4 roster)`; no real-lane execution claimed |
| R2 "one real fleet agent session per cast member" (6 sessions) | session ids recorded in `R2/fleet-sessions.json`; the installed 0.2.2 host has no MCP facade and its DSH session store is not readable as uid 1000 | **cannot be cross-checked post-hoc** — recorded as a named limit, not a pass |
| R3/R4/R6 pending pieces | index + data | correctly pending with named PACKAGE-PENDING pieces; never clean |
| No secrets in the pack | my own scan, 161 files, 11 patterns (+ owner-profile-path check; live MCP token compare unavailable because this host has no MCP facade) | **clean, 0 hits** (`s7/checker-verify/secrets-scan.txt`) |
| No pending step recorded clean | data-vs-index comparison + clean-step hard-check rule | none found |
| Fleet untouched | unit MainPID/InvocationID + control status + no leftover roots | unchanged / intact |

## Raw evidence (all under `s7/`)

`00-s7-meta.txt` · `01-pass1-timing.txt`, `03-pass2-timing.txt` · `02-pass1.out`, `04-pass2.out` (my run transcripts) ·
`ergon-pack/pass-1|pass-2/` (indexes, data, step evidence, manifest, commands.log) · `ergon-pack/PACK.md`,
`pass-diff.md`, `verdict-table.diff` · `checker-verify/ergon-verify.json` (81 checks) · `checker-verify/verdicts.json` ·
`checker-verify/secrets-scan.txt` · `structure/ergon-structure.json` (driver structure check: 18/18) · `SHA256SUMS`.

## Named limits (not smoothed)

1. The rehearsal ran on the installed platform host against **0.2.2** — the artefact the final 0.3.0 install would
   replace. The docs' "fresh cloud host + DNS + Supabase project" legs are named out of scope by the driver itself
   (R0/R3 evidence); this run is the platform-host rehearsal the Lead scoped, not a fresh-cloud-host pass.
2. R2's real fleet agent sessions cannot be re-read after the fact on a host without the MCP facade; I did not take the
   pack's session ids on trust (I simply cannot verify them) — named, not passed.
3. Pass 1 wall time was ~3 s, pass 2 ~45 s; the fixtures are small and the two passes are deterministic. The pack's own
   `commands.log` and my timing files record both.
4. The manifest defect above is the only rejection reason. With that fixed (manifest written after the last append, or
   the two logs excluded and documented), the same pack content would re-verify; I will re-run both passes and re-check
   from a fresh pack, not patch theirs.
