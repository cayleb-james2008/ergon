# S5 verdict — independent MCP acceptance, release tarball 0.3.0

Checker: `checker-verify` (fresh-context; built none of this artifact or the fleet). Maker never checker: I did not edit
or "fix" anything; the only builder-origin input is the release tarball bytes and the shipped row contract. The primary
verdict is my own client, not the builder's `mcp-selftest` or gate script.

**VERDICT: ACCEPT** — on a scratch host booted from the exact release bytes, over **both** loopback and the host's
tailnet address, every mandatory check in my battery passed. Named deviations and limits are listed at the bottom —
none of them is smoothed over, and none contradicts an accepted claim.

## Artifact under test

| | |
|---|---|
| Tarball | `/home/cayleb/workspace/harness-build/strike-release/dist/cayleb-dsh-fleet-0.3.0.tgz` |
| SHA-256 | `b60019a0f3611c83fb13a374444c46b05092df89737f803661408fd7151a0b10` — equals the Lead-declared digest |
| Members | 104 (tar entries; no directory entries) |
| Installed profile | throwaway `DSH_HOME=/tmp/cv-s5.tbqAwQ/dsh`, profile `fleet` (copy of the shipped web template), plugin `0.3.0` |
| Installed-tree identity | `diff -rq <tar>/package ≈ <profile>/node_modules/@cayleb/dsh-fleet` → **exit 0, zero differing lines** (byte-identical to the tarball) |
| Composed profile | 15 `fleet-*` rows incl. `fleet-mcp`; scratch overlay only redirects paths/port (my own overlay, not the builder's) |
| Listeners :18889 | `127.0.0.1`, `[::1]`, `100.114.17.67` — exact binds, no `0.0.0.0`/`::` catch-all |
| Platform state | control `status`: ok/integrity/lanes true, failures `[]`, policy selftest on, peer_uid 1000 |

Run: 2026-11-01T22:22:28Z → 22:22:55Z. Commands: `bash s5/run-s5.sh` (its full log and every raw call are in
`s5/`). The installed `fleet-dsh` service was NOT touched (it still runs 0.2.2 without the MCP row; see limits).

## Independent probe result — `probes/mcp-verify.mjs` (my client, no builder code)

87 checks · **84 passed · 0 failed** · 3 informational. Summary `verdict: PASS`; `tailnet_established: true`.

| Test | Result (machine-checked) |
|---|---|
| **T1** unauth/wrong-token | missing token → HTTP 401, JSON-RPC `-32001`, **no result envelope**; wrong token → 401 `-32001`; action log gained exactly `refused` rows `missing-token` + `invalid-token` with `principal:null`; **zero intent rows** (refused before dispatch). `WWW-Authenticate: Bearer` present. |
| **T2** initialize + tools/list, both legs | both legs HTTP 200, `Mcp-Session-Id` issued, `serverInfo.name=fleet-mcp`, protocol `2025-03-26`, `notifications/initialized` 202; tool names exactly the contracted nine: `actions.status, board.op, fleet.selftest, fleet.status, order.drive_roundtrip, order.stage_real, session.create, session.prompt, session.status`; every tool has description + inputSchema; both legs identical. Tailnet leg reached `100.114.17.67:18889`. |
| **T3** stub session | `session.create` → `sessionId=fleet-mcp-…`, `status:created`, `mode:stub`, model label `deterministic-stub (fleet-selftest-stub)`, provider `fleet-selftest-stub`. `session.prompt` → `status:completed`, `turns:1`, **`transport:"stub-turn"`**, `turn_agent:"builder:selftest-…"`, `ledger_gate:"stub-provider-selftest-only"`, `turn_model:"claude-haiku-5-5"`, `provider_calls:1`, non-empty assistant text (starts `S3STUB:`). `session.status` → `found:true`, turns 1, same model label, `last_transport:"stub-turn"`, read-back of the same assistant text. `session.create mode:"model"` → named refusal `model-credential-not-entered`, no session id, no faked model call. |
| **T4** `order.drive_roundtrip` | first pass: order `o1` / task `t1` / final `t2`, 12 posts; `final_state` order done, task done, verdict pass; **independently re-read over the control socket**: `order-get o1` → state `done`, `fixture:true`, task `t1` done+banked, final task `t2` done; every reported post id exists on the board; order appears in `order-list`. Second independent pass: `o2`/`t3`, also verified — different ids, so no planted one-shot. Labels honest: `mode:"fixture-validated"`, `real_lane:"not-built (Stage 4 roster)"`. |
| **T5** `order.stage_real` | no roster mounted → named refusal `lanes-unavailable`, `details.mode:"real-lane-staged"`, staged path under the scratch state dir, **`payload_sha256` equals my recomputation of `sha256(canonical(payload))`**, payload is the non-fixture order form with the sent title/goal/done-when, the done-when command was executed by me (`true`, exit 0), **no order row created** (order-list has no order titled with the staging marker) and no execution claim. |
| **T6** action log | every tool call this probe made has an **intent** row before its **outcome** row (`ref_seq`=intent seq), ordered call-for-call, `principal:"cayleb-local"`, args stored only as names + sha256 digest; outcome `ok`/`error` matches reality (`lanes-unavailable` for stage_real). Whole log (readable on scratch, 24 rows, sha256 `d0e9331096ccddbf44e8a1d9db519529464e82b49fb97f0e9bb04e5a3c196e33`): every line parses, **full hash chain links and recomputes**, **no bearer token anywhere in the file**, no argument value I sent, `actions.status` reports `open:true`, `broken:null`, `chain:"jsonl-hash-chain"`. |
| **T7** source gate (partial, stated) | `fleet.status.mcp.sources` = loopback `127.0.0.0/8` + tailnet `100.64.0.0/10` + `::1`; `mcp.bind` = the three exact hosts; `ss` shows no catch-all; unit netfilter for the (future) installed service reads `IPAddressAllow=localhost 100.110.154.92 100.114.17.67` with deny-all-LAN/global-v6 (recorded; it governs the installed unit, not the scratch host). A live foreign-source request could not be produced from this seat (EADDRNOTAVAIL on a local-bridge source) — **recorded as not proven, never as a pass**. |
| **T8** transport limits | body over 262144 → **413 `-32004`**; malformed JSON → **400 `-32700`**, no dispatch; batch → **400 `-32600`** (refused, one message per POST); `payload-too-large` and `parse-error` refusals appended to the action log. |
| **T9** rate limit | inside one minute the request after the 60/min allowance → **429 `-32005` with `retry-after: 60`** (observed after 43 further requests in this run, window already partly used); `rate-limited` refusal appended with `principal:null`. |

Whole-log and evidence secret scans: bearer token **0 hits** in the action log; **0 hits** across the 61 evidence files
(compared in memory; the token was never printed, stored or transmitted off-host).

## Raw evidence (all under `s5/`)

`00-run-log.txt` (every step + PASS/FAIL) · `01-environment.txt` · `02-installed-tree-diff.txt` (identity) ·
`03-composed-profile.yml` · `05-control-status-before.json` · `06-listeners.txt` · `07-mcp-probe.log` (my probe transcript,
auth redacted) · `08-control-status-after.json` · `09-mcp-actions.jsonl` + meta (log copy, 24 rows, sha256) ·
`10-whole-log-token-scan.txt` · `11-evidence-token-scan.txt` · `12-host-log.txt` · `mcp-evidence/mcp-verify.json` (all 87
checks, every expected/actual) · `mcp-evidence/raw/*.json` (each call's raw result, incl. both roundtrips + control-socket
reads) · `SHA256SUMS`.

## Deviations and limits — named, not smoothed

1. **Stub echo differs from the tool description.** The description says `S3STUB:<your text>`; the observed assistant
   text was `S3STUB:Current runtime context. …` — the stub echoed the session's injected runtime-context turn rather than
   the caller's literal text. The turn is real, model-free, correctly labelled (`transport:stub-turn`,
   `ledger_gate:stub-provider-selftest-only`) and `session.status` reads back the same text. Recorded as an observed
   behaviour deviation from the description; it does not affect the acceptance criteria (transport + non-empty text).
2. **Batch refusal is not in the action log.** A JSON-RPC batch is refused 400 `-32600` (correct) but no `refused` row is
   appended, while malformed JSON and over-limit bodies do get rows. Recorded as observed; the log contract names bad
   token / foreign source / rate limit / admin-socket refusals, not batches, so this is not treated as a violation.
3. **Ping probe duplication.** The over-limit request produced two `payload-too-large` refused rows for one client
   request (server-side read-body path logs on the too-large path; observed count 2). Harmless, recorded exactly.
4. **Installed-service leg NOT RUN.** The running `fleet-dsh` unit is 0.2.2 and has no `fleet-mcp` row (install blocked
   on the owner act per the Lead). This verdict covers the release bytes on a real host booted from them, including the
   tailnet bind — but it does **not** prove the installed service. The same battery must be re-run against the installed
   unit once it is 0.3.0; the installed-service acceptance remains **queued**, not passed.
5. **Foreign-source live refusal is not proven** (only the runtime allowlist/bind inspection + unit netfilter). Needs a
   second host/namespace or root.
6. Reported `turn_model:"claude-haiku-5-5"` on the stub transport. This is the ledger's priced model name for the
   synthetic call, not a provider call made with a credential (`provider_calls:1` through the stub route, gate reason
   `stub-provider-selftest-only`); recorded so nobody misreads it as a real model call.

**ACCEPT** stands on points proven above; item 4 is the only acceptance scope left open.
