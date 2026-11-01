# S8-030 verdict — Ergon dress rehearsal on the INSTALLED 0.3.0 platform

Checker: `checker-verify` (fresh-context; built none of Ergon, the fleet, the driver or the fixtures). I did **both**
things the protocol allows and state both: (1) verified the builder's filed pack with my own verifier and independent
control-socket reads, and (2) **ran the driver myself twice** on the installed 0.3.0 host from a fresh pack of my own.
Nothing was installed, restarted or changed by me.

**VERDICT: ACCEPT** — the rehearsal on the installed 0.3.0 reproduces the accepted 0.2.2 verdict table exactly
(byte-identical pass-2 tables, zero data differences in words, table notes, pending lists and hard-check sets), both
packs seal and verify their manifests, 0 failed, 4 clean / 4 pending-named, R5 orders are real and done+banked on the
live 0.3.0 board, R0 identifies the installed release from the install's own audit line, and the independent secrets
scans are clean. The driver commit that post-dates my accepted S7 fix (`cb909b4`) is reviewed and only derives recorded
platform versions from R0 (no seal/manifest/verdict logic touched).

## Artifact / platform

| | |
|---|---|
| Installed release | `@cayleb/dsh-fleet` **0.3.0**, tarball sha256 `8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070` (the artifact I verified in `s5-rerun2` and on the installed service in `s5-installed`) |
| Unit | `fleet-dsh` active; `MainPID 518634`, `InvocationID bc98f82d…` throughout this verification; control `status` ok/integrity ok/**rows ok**/lanes true |
| Driver | `/home/cayleb/workspace/harness-build/ergon-strike`, branch `strike/ergon-rehearsal` @ **`cb909b4f03ff529349abcfafb36d1e419bf3ac52`**, worktree clean, driver sha256 `35fb2593c528b3bb9963ee8671ec471688c71824c044b9c07c7c38eb5f6e5817` |
| Driver change since accepted `003d9ef` | **one commit, version-derivation only** (`run-meta` platform from R0's resolved identity, index identity line, R0 host note, R7 state record). Seal (`lib/seal.mjs`), manifest coverage, index build gates, verdict/pass-diff logic are untouched — verified by diff review; this also closes the minor `run-meta` hardcoded-version finding I filed in S7-rerun |

## Method A — builder's filed pack verified by me

Pack: `/home/cayleb/Documents/deepseek-harness/default-workspace/board/probes/strike-ergon-rehearsal-030` (168 files /
2.2 MB, fresh dir, mtime 23:08Z).

| Check | Result |
|---|---|
| Declared hashes vs actual | pass 1 manifest `75262e52…`, index `b10d11bc…`; pass 2 manifest `583293af…`, index `4e2a1852…` — **all four match the Lead's declared values** |
| `sha256sum -c manifest.sha256` (run by me) | pass 1 **72/72 OK, exit 0**; pass 2 **74/74 OK, exit 0** |
| My verifier (`ergon-check.mjs verify`, 113 checks) | **ACCEPT — 113/113 passed, 0 failed, 0 informational**: E1–E13 incl. manifest coverage (missing/extra/stale = 0/0/0), nothing newer than the seal, seal-time + driver gate outputs, evidence files exist and mtime within run window, word/table/pending consistency, 0 failed, clean steps carry hard checks, pass-2 table identical, R7 diff identical, commands all exit 0, run values real, R0 version-agnostic identity |
| R0 installed identity | **0.3.0**, `cayleb-dsh-fleet-0.3.0.tgz`, sha256 `8173d13e…`, `identitySource = journald install audit line (dsh-fleet-admin plugin-add … via=sudo)`, `verified:true`; drop inventory lists 0.3.0 (installed) plus 0.2.2/0.2.1/0.2.0 as staged-only (`08-r0-identity-030.txt`) |
| R5 orders (independent control socket, read-only) | **`o3286`**: state `done`, `fixture:true`, task `t4162` done+banked, final `t4163` done; **`o3287`**: `done`, `fixture:true`, `t4164` done+banked, final `t4165` done. Evidence/verdict posts exist on the boards: `o3286` → 7449/7451; `o3287` → 7462/7464 (`07-r5-orders-control.json`, `07b-r5-posts.txt`) |
| Deep comparison vs accepted 0.2.2 pack | pass-2 data across R0–R7: **0 differences** in words, table notes, pending lists or hard-check sets; **pass-2 verdict tables byte-identical**; words both: clean 4 (R0,R1,R5,R7) · pending 4 named (R2,R3,R4,R6) · failed 0 (`22-builder-030-vs-accepted-022.txt`) |
| Independent secrets scan | clean — 0 hits, 11 patterns + owner-profile check, live MCP token compared in memory (none present) |

## Method B — my own from-scratch runs on the installed 0.3.0

Fresh pack `s8-030/ergon-pack-own`, two invocations exactly per the design:

| | pass 1 | pass 2 |
|---|---|---|
| Exit | **0** | **0** |
| Seal | `manifest 72/72 OK`, manifestSha256 `bf939c9eee53bb837f0599dfd09247167902766234ff90ebbdd54b3f92e96166`, indexSha256 `716c9a1cadfe9e88d0455c685c2775ac82ea72ef16f9a64c57e49151f26a240e` | `manifest 74/74 OK`, manifestSha256 `135ccea8ef86bf130d2000528f6c4f5b3a3f6efa68201a691df131d7bdf9cb8f`, indexSha256 `f58f55beee5b0323ab91c4cee54746130cecc9c1cfa142e81689f6a7be59dca4` |
| Words | clean 4 / pending 4 named / failed 0 | identical; `PASS DIFF: IDENTICAL` |
| My verifier | — | **ACCEPT — 113/113, 0 failed** |
| R5 orders (control socket) | **`o4375`** done/fixture, `t5532` done+banked, final `t5533` done, posts 9833/9835 present | **`o4376`** done/fixture, `t5534` done+banked, final `t5535` done, posts 9846/9848 present |
| R0 / run-meta | 0.3.0 from journald audit line, `verified:true`; run-meta platform = `installed @cayleb/dsh-fleet 0.3.0 (cayleb-dsh-fleet-0.3.0.tgz, sha256 8173d13e…)` — derived from R0, no constant | same |
| Platform facts | 77 mounted `fleet-*` rows (vs 75 on the accepted 0.2.2 host; 0.3.0 adds `ledger`/`firstrun`/`mcp` rows); repo pin `cb909b4` | same |
| vs builder's 0.3.0 pack | **0 data differences**, pass-2 verdict tables byte-identical (`23-own-vs-builder-030.txt`) | — |
| Secrets | driver scans clean + my independent scan clean | same |

No `/tmp/ergon-rehearsal-*` leftovers; both fresh company roots were torn down by R7; `fleet-dsh` remained active with the
same MainPID/InvocationID before and after; nothing installed or changed by me.

## Notes and limits (named, not smoothed)

1. **Installed-host stub-turn text deviation (carried from `s5-installed`).** The R2 `stub-turn` probes in both 0.3.0
   packs (builder's and mine) record `provider_calls: 0`, `assistant_text_length: 0` — the same host-level condition I
   documented in the installed-service MCP battery (reproduced outside the facade; present on the installed 0.2.2 host
   too; absent on a scratch boot of the same bytes). R2 is `pending` for the separate provider-credential reason, and its
   hard-check set is identical to the accepted 0.2.2 pack, so the verdict is unaffected — but the empty stub text is
   visible in the pack and named here, not hidden.
2. Fresh-cloud-host / DNS / company Supabase legs remain owner acts, named out of scope by the driver itself (R0/R3
   evidence); this is the platform-host pass on the installed unit.
3. R2's "one real fleet agent session per cast member" ids still cannot be re-read post-hoc on a host without a readable
   session store — recorded as a limit, not passed.
4. The unit was externally client-SIGKILL-restarted at 23:10:11Z (between the Lead's install message and my installed
   battery; see `s5-installed`); it was active and healthy throughout this rehearsal verification, and neither the
   builder's nor my run restarted it.

## Raw evidence (`s8-030/`)

`00-s8-meta.txt` · builder pack verdict from my verifier `checker-verify/ergon-verify.json` + `verdicts.json` +
`secrets-scan.txt` · `07-r5-orders-control.json` / `07-r5-summary.txt` / `07b-r5-posts.txt` ·
`08-r0-identity-030.txt` · `10-own-run-timing.txt` · `11-own-pass1.out` / `12-own-pass2.out` (my run transcripts) ·
`ergon-pack-own/` (my sealed pack: manifests, indexes, data, evidence, `manifest-check-*`, `pack-commands.log`) ·
`checker-verify-own/` (my verifier's 113 checks on my pack, secrets) · `20-own-r5-orders-control.json` +
`20-own-r5-summary.txt` · `21-own-r0-identity.txt` · `22-builder-030-vs-accepted-022.txt` ·
`23-own-vs-builder-030.txt` · `24-platform-after.txt` · `structure/` · `compare/` · `compare-own-vs-builder/` ·
`SHA256SUMS`.
