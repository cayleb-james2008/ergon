# 0.3.1 CHECKER INPUT (fresh installed-output seat) — Team B supply

Companion to `board/reports/team-b-031-candidate-build.md` (candidate branch `candidate/0.3.1`, tip `aa7415e`,
tarball sha256 `b2ae2185…`, 123 members). The **checker must take the tarball sha/member count from the release
actually installed by strike**, not from this page; strike's install record is authoritative. What follows is what
changed and what to read back.

## Exactly what changed in 0.3.1 (vs installed 0.3.0, `7d4eb95`)

1. Roster module added (`package/src/roster/**`, 19 shipped files) + `./roster` export; F1–F4 fix round from
   `board/reports/team-b-batch8.md` (tree write-before-verify + symlink-prefix refusal; awaited host.inspect + tick
   retry; `claimed_at`-based N6 changed set; engine tail capture/budget refusal/persisted seq).
2. Bundle patch: `llm-pi-ai` gains `providers.opencode-go { apiKeyEnv: OPENCODE_GO_API_KEY }` (no URL restated —
   pi-ai catalog supplies `https://opencode.ai/inference/go[/v1]`; Nox ruling: cloud Go route); `subagent` gains
   `maxActiveSubagents: 32`; **`fleet-roster` row mounted** (touchpoint 5) with gitwork/engines/credential/session
   config.
3. Stage-4 touchpoints in core/orders/ledger: `setLaneRoots`; verdict gate, harness `invalidCheck`, harness post
   kinds `stall`/`invalid-check`, `releaseClaims`; ledger `priceProvider`, event `source`/`session`, `spend-get`.
4. Ledger `rates` row `opencode-go/deepseek-v4.1-flash` (0.15/0.6/0.003 per Mtok); roster doctrine table (builder/
   researcher = DeepSeek V4.1 Flash via opencode-go; lead Sonnet 5.5; checker Haiku 5.5).
5. `rows.manifest.json` regenerated: **8 pinned / 12 disabled** (0.3.0 shipped 7/12 and omitted its own `fleet-mcp`
   insert row); sha256 `1970f9954b7131c58191327bb0e60b9d31a704bc309d083cd53971bc9271edba`.
6. Stage-5 `src/bus` is NOT in the release path (candidate commit `6dc1348`); no bus checks/rows.
7. `checks/stage2-check.sh` is byte-identical to 0.3.0 (`7df6344c…`) — run it **unmodified**.

## Acceptance command (S4B-equivalent, one run, fresh OUT)

```bash
OUT=/home/cayleb/workspace/harness-build/evidence/<your-dir> \
FLEET_CLI=/home/cayleb/workspace/harness-build/candidate-031/package/bin/dsh-fleet.mjs \
bash /home/cayleb/workspace/harness-build/candidate-031/checks/stage2-check.sh
```
Expected: `summary: pass=60 fail=0`, `STAGE 2: PASS`, `EXIT=0`; capture `stage2-check.out`, `console.err`,
`exit-code.txt`, pre/post unit identity (`InvocationID`, MainPID), `first-run --status` before/after. The check's
own 2.11 leg SIGKILLs and restarts the unit — that is expected. Do not retry or reorder.

## Read-back expectations (installed 0.3.1)

version `0.3.1`; installed-tree file count = tarball member count (candidate: 123), modes {0644,0755}; drop tarball
sha = strike's published digest and the conventional `<tgz>.sha256` sidecar present (leg 2.1a); rows 8 pinned /
12 disabled, failures `[]`, installed manifest sha `1970f995…`; MCP listeners exactly 127.0.0.1/[::1]/
100.114.17.67:8789 (no catch-all); the nine MCP tools unchanged (`fleet.status`, `session.create`, `session.prompt`,
`session.status`, `board.op`, `order.drive_roundtrip`, `order.stage_real`, `fleet.selftest`, `actions.status`);
`dump-config` shows the `fleet-roster` row, `llm-pi-ai` route `providers.opencode-go` with `apiKeyEnv` and **no key
value**, and `subagent.maxActiveSubagents: 32`; `policy-get rates` has `opencode-go/deepseek-v4.1-flash`;
`policy-get roster.models` (after the roster mounts) shows the doctrine table; `status` ok/integrity ok/rows ok/
lanes_allowed true/selftest true; the stub session turn still completes (`transport: stub-turn`).

## Must NOT certify

Activation (owner's key entry is a separate, owner-performed act), any real-provider turn (`session.create
{mode:"model"}` still refuses `model-credential-not-entered`), `stage4-check.sh`, engine CLI runs (no binaries/keys),
hands/bus, and any lane work (`lane-spawn` may answer `no-credential`/`unknown-model` until activation). Do not
request, print or store any credential value; do not read images/screenshots; maker never checker.
