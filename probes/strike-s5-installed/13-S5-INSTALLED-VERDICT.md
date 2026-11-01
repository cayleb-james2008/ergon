# S5 INSTALLED verdict — MCP battery against the live fleet-dsh service (installed 0.3.0)

Checker: `checker-verify` (fresh-context; built none of this artifact or the fleet). My unchanged battery
(`probes/mcp-verify.mjs`, the same file used on the scratch boots) run against the **installed unit's own listeners**, with
the token read by the probe itself over the real admin socket. Nothing was restarted or changed by me.

**VERDICT: ACCEPT (with one named installed-host deviation)** — every mandatory battery item passes on the live
installed service except the strict non-empty stub-text check. `session.prompt` returns a **completed** turn with its
transport named (`stub-turn`) and the **named ledger-gate reason** (`stub-provider-selftest-only`) — satisfying the
Lead's stated OR — but the assistant text is **empty** and `provider_calls` is 0 on this installed host. That condition is
reproducible **outside the MCP facade** through the fleet's own `stub-turn` control op and was already present on the
installed 0.2.2 host before this artifact, while a scratch boot of the same final bytes returns `S3STUB:` text — so it is
an installed-profile/runtime condition, not a regression of the installed 0.3.0 artifact. Recorded, not smoothed.

## Target (live installed service)

| | |
|---|---|
| Release | `cayleb-dsh-fleet-0.3.0.tgz` sha256 `8173d13ebee542b328ffe0b4a3476e27ffa561e46172949d54f9757ddd4e1070`; the staged `/var/lib/fleet-drop/cayleb-dsh-fleet-0.3.0.tgz` hashes to the same bytes |
| Unit at run | `fleet-dsh` **active**, `MainPID 518634`, `InvocationID bc98f82dc22a4053b41f75f3a9e10231`; unchanged before/after my battery |
| Listeners :8789 | `127.0.0.1`, `[::1]`, `100.114.17.67` — **exactly** the three; no `0.0.0.0`/`::` catch-all |
| Sockets | `/run/fleet/mcp-admin.sock` present, `/run/fleet/control.sock` present |
| Boot integrity | `ok:true`, chain ok, sandbox ok, **rows ok** (`pinned 7`, `disabled 12`, `failures:[]`, facet manifest sha `1bcac986…` == the shipped `rows.manifest.json`), guard ok, `lanes_allowed:true`, `policy.selftest:true`, control `peer_uid:1000` |
| Binds vs netfilter | facade `mcp.bind` = `127.0.0.1`, `::1`, `100.114.17.67`; identical to `ss`; every bind is inside the unit's allowlist (`127.0.0.0/8`, `::1/128`, `100.114.17.67/32`, `100.110.154.92/32`); unit also denies `100.64.0.0/10`, `0.0.0.0/8`(LAN ranges) and `::/0`; no catch-all |
| Audit | journald `dsh-fleet-admin: plugin-add cayleb-dsh-fleet-0.3.0.tgz sha256=8173d13e… via=sudo` at 23:07:09Z |

Service-restart note: between the Lead's install message (MainPID 510804 / InvocationID baf7bf93) and my battery the
unit was **client-request SIGKILL-restarted at 18:10:11 local (23:10:11Z)**, journal-visible, coming back active
(MainPID 518634 / InvocationID bc98f82d). It was not me — my scripts capture before/after and never restart; the restart
is consistent with the parallel acceptance check the Lead mentioned. The battery ran entirely on the post-restart
process, with the same MainPID/InvocationID before and after.

## Battery result — 85 checks, 83 passed, 2 failed (both strict stub-text checks), 4 informational

`mcp-evidence/mcp-verify.json`, transcript `07-mcp-probe.log`. Decisive items:

| Test | Installed-service result |
|---|---|
| **T1** auth | missing token → HTTP 401 `-32001`, no result envelope; wrong token → 401 `-32001`; action log gained `refused` rows `missing-token` + `invalid-token` with `principal:null`; **zero intent rows** (refused before dispatch); `WWW-Authenticate: Bearer` |
| **T2** both legs | loopback and tailnet both initialize 200 + `Mcp-Session-Id` + `serverInfo fleet-mcp`; `notifications/initialized` 202; `tools/list` = exactly the nine contracted tools, identical over both legs |
| **T3** sessions | `session.create` OK (`fleet-mcp-…`, mode stub, model `deterministic-stub (fleet-selftest-stub)`); `session.prompt` → **`status:"completed"`, `transport:"stub-turn"`, `ledger_gate:"stub-provider-selftest-only"`, `turn_agent` named, `turns:1`**, but **`assistant_text:""`, `provider_calls:0`**; `session.status` → `found:true`, turns 1, `last_transport:"stub-turn"` (same empty text). `mode:"model"` → `model-credential-not-entered` refusal, no session id. **See the deviation section.** |
| **T4** roundtrips | two chains on the live installed board: `o4373`/`t5528` (final `t5529`) and `o4374`/`t5530`, 12 posts each, final state `done`; **re-read via `/run/fleet/control.sock`**: order `o4373` state `done`, `fixture:true`, task `t5528` done+banked, final task `t5529` done; labels `fixture-validated` / `real_lane: not-built (Stage 4 roster)` |
| **T5** stage_real | named refusal `lanes-unavailable`, `real-lane-staged`, staged path under `/var/lib/fleet/state/mcp/staged/`, **`payload_sha256` equal to my own recomputation**, non-fixture payload, done-when executed (`true`), **no order row created** (order-list has no matching title) |
| **T6** action log | every tool call has intent-before-outcome (`ref_seq`), principal `cayleb-local`, args stored only as names+digest; `actions.status` reports `open:true`, `broken:null`, chain `jsonl-hash-chain`, seq 60 at the end; every call paired 9/9 |
| **T7** source gate | runtime sources loopback+tailnet CIDRs; bind exact (above); unit netfilter verified; live foreign-source refusal not proven (needs second host/ns/root) — stated |
| **T8** limits | over-limit body → **413 `-32004`** (+`payload-too-large` rows); malformed JSON → **400 `-32700`**; batch → **400 `-32600`** |
| **T9** rate | **429 `-32005`** with `retry-after: 59` (after 43 further requests in the window); `rate-limited` refused row logged |

Whole-evidence token scan: **0 hits** across 63 files. Token was read only via the admin socket; never printed or stored.

## The named deviation (why it doesn't block ACCEPT, and exactly what it is)

- **Observed on the installed service:** `session.prompt` completes, transport `stub-turn`, ledger gate
  `stub-provider-selftest-only` named, but `assistant_text` empty and `provider_calls` 0. The Lead's acceptance wording
  is "a non-empty assistant text **or** a named ledger-gate reason": the named gate reason is present, so the criterion
  is met; the deterministic echo itself (`S3STUB:<your text>`, per the tool description) is not produced on this host.
- **Not the MCP facade's doing, and not new in this artifact:**
  1. the fleet's own `stub-turn` control op (outside MCP) returns the same `completed` / empty text / `provider_calls:0`
     on this host (`13-control-stub-turn-repro.json`);
  2. the same empty result is recorded in the **installed 0.2.2** era by my own accepted S7 re-run's R2 stub-turn probes
     (before the 0.3.0 install; `provider_calls: 0`, `assistant_text_length: 0`) and by the builder's 0.3.0 pack R2 probes
     (`14-prior-install-stub-turn.txt`);
  3. a **scratch boot of the same final bytes** returns real `S3STUB:` text (`provider_calls:1`) — e.g. `s5-rerun2`.
  So the difference is the installed fleet profile/runtime, not the installed artifact. Recommendation (follow-up, not a
  release blocker): compare the installed fleet profile's agent/llm row config with the shipped template to find why the
  stub adapter is not reached on the installed host.
- **Recorded as failed, not hidden:** my probe marks the strict non-empty-text checks FAIL (T3.prompt, T3.status); this
  verdict accepts under the Lead's OR criterion and names the failure explicitly.

## Stated limitations (readable scope)

- `/var/lib/fleet/log/mcp-actions.jsonl` is `fleet`-owned and **not readable by uid 1000** (`ls`/read refused). A raw
  **full-file** token scan is therefore **not possible from this seat**. The readable scope is the admin socket's
  `log_tail` (100 rows; 60 present) + MCP `actions.status`: **no bearer token in that scope**, log `open:true`,
  `broken:null`, token file mode 600 (mode read from the admin stat; value never printed).
- Live foreign-source refusal not proven (only runtime allowlist, exact binds, and unit netfilter — all verified).
- T3 stub-text deviation as above.

## Raw evidence (`s5-installed/`)

`00-run-log.txt` · `01-installed-identity.txt` (unit, listeners, sockets, audit line, direct-read refusals) ·
`02-listeners.txt` · `03-control-status-before.json` · `04-boot-integrity.json` · `05-bind-vs-netfilter.json` ·
`07-mcp-probe.log` · `08-control-status-after.json` · `09-action-log-readable-scope.json` /
`09-action-log-readable-scope.log` · `10-direct-log-read.txt` · `11-evidence-token-scan.txt` ·
`12-installed-stub-turn-criterion.json` · `13-control-stub-turn-repro.json` + `.log` ·
`14-prior-install-stub-turn.txt` · `mcp-evidence/mcp-verify.json` + `raw/*.json` · `run-s5-installed.sh` ·
`SHA256SUMS`.
