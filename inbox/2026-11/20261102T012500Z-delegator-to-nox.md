from: delegator
to: nox
kind: ACTIVATION PACKAGE READY + **ONE RULING NEEDED** (shipped facade cannot do a real-provider session turn — propose amending the done-when to the roster-lane proof); door refreshed; first real order staged; installed-host facts verified (subagent 32 mounted, opencode-go route, roster row)
re: 011600Z + night order 010900Z; install 012000Z filing
---

## 1. INSTALLED HOST — VERIFIED FACTS (all read on the live 0.3.1, MainPID 663330)
- `dsh-fleet-admin dump-config` (as fleet): **`maxActiveSubagents: 32`** mounted; **`opencode-go: apiKeyEnv: OPENCODE_GO_API_KEY`**
  route present (no URL); **`- id: fleet-roster`** row mounted. Sub-agent ceiling confirmed shipped in `package/cordis.patch.yml`
  lines 240-242 as a patched BASE row (not in the pinned manifest — rows stay 8/12, integrity unaffected; no profile write needed).
- `providers-get`: `opencode-go` kind llm, models `["deepseek-v4.1-flash"]`, **credential present:false** (owner act, expected).
  `policy-get roster.models` answers the policy rows (day_cap 5, lease 900s, goldfish 3 reviewers, etc.).
- Rows ok 8 pinned/12 disabled, manifest sha `1970f995…`; listeners three addresses; `first-run --status` 0.3.1/123 files.
- Files landed in origin/team: `reports/team-b-activation-package.md`, `reports/team-b-nox-door-corrected.md`.

## 2. **RULING NEEDED — the shipped facade cannot run a real-provider session turn**
The activation checklist's real-route proof was going to be `session.create {mode:"model"}` on the facade. Team B found in the
shipped 0.3.1 bytes that **`session.create {mode:"model"}` is an unconditional `model-credential-not-entered` throw**
(`src/mcp/session.js:245`; same in 0.3.0) — entering the owner's key does NOT change it. **So "a facade session turn on a real
provider route" cannot be met as shipped.** Two honest routes:
- **(A) AMEND the done-when** (my recommendation): activation proof = the **roster-lane real-route turn** —
  `fleet-roster lane-spawn {"role":"builder","domain":"activation","start":true}` → a real `llm/stream` call through the
  `opencode-go` route, evidenced by the lane row + a real (non-synthetic) fleet-ledger event + `providers-get` credential-present.
  The facade's stub turn stays labelled `stub-turn` and is never presented as a model/provider proof. This is achievable the
  moment the owner enters the key, with no further build.
- **(B)** Have the release line wire facade model-mode (patch `session.js`) in a 0.3.2 — a build beyond tonight's release; then
  the facade turn becomes provable too.
Please rule A or B. Everything else for activation is ready tonight: the guided-first-run path (`node
/home/cayleb/workspace/harness-build/candidate-031/package/bin/dsh-fleet.mjs first-run --credentials` from a readable release
copy, sends `first-run-credential-set` to the uid-1000 control socket), the exact prompt text re-confirmed in the shipped bytes
**`OPENCODE_GO_API_KEY (opencode-go) [Enter to skip]:`** (echo-off before the prompt, write-only, store 0600), the full checklist,
and the failure path (report the provider error, never cite the stub). **The owner's single act remains the key entry; it is HELD
for the consolidated wake-up report — nothing surfaces tonight.**

## 3. DOOR — REFRESHED AND INSTALLED
`~/.dsh/strike-door/NOX-DOOR.md` replaced with the corrected 0.3.1 version (old kept as `NOX-DOOR.md.pre-0.3.1-20261102`, 0600):
step 2 labelled stub; **step 3 `order.stage_real` now CREATES a real non-fixture order** (`mode:"real-lane"`,
`real_lane:"fleetRoster mounted"`), roundtrip stays fixture-validated, the real-route story and the facade limit are stated, token
discipline unchanged. Ready for you to use when the fleet door is re-issued.

## 4. FIRST REAL ORDER — STAGED, FIRES ON ACTIVATION
Staging pack filed (payload for the plugin's visible surfaces — own sidebar section + phone-VM + Windows-VM sidebar panels,
design-pack + unslop fences, rubric done-when, 90 min, budget 0.5) + the exact `order.stage_real` call and expected result:
**one real order id `o<seq>`, state `open`, `fixture:false`; no task ids until a real Lead lane decomposes** (board property);
maker = builders, harness-made fresh Checker per round, goldfish reviewers on the large-order path, maker never checker. Nothing
driven before activation.

## 5. LIVE BATTERY
`checker-031` is running the installed-output battery now (unmodified `stage2-check.sh` on the installed host → expect
`pass=60 fail=0` / `STAGE 2: PASS`, identity 123/123, rows, installed-unit MCP, roster facts, `order.stage_real` observed not
fired). Verdict → immediate filing.

— delegator
