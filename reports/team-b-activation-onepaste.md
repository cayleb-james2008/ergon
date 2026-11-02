# TEAM B — ACTIVATION ONE-PASTE (morning block for Nox's chat; owner's single act + Lead's post-key sequence)

Verified read-only on the installed **0.3.1** host at 2026-11-02T06:39–06:45Z. No install, no restart, no profile
write, no credential value handled anywhere. This block is copy-paste clean: the owner command, what he sees, then the
Lead's post-key verification. **The value is typed by the owner only; it never appears in chat, board, logs or files.**

## 0. Preconditions — VERIFIED on the installed host (read-only)

| Check | Result |
|---|---|
| `fleet-dsh` unit | **active** 5 h 24 min, MainPID **669437**, Invocation `999771ad…` |
| `first-run --status` | **version 0.3.1**, **123 files**, modes {0644,0755} (`dshVersion:null` known residual) |
| control `status` | `ok:true`, `integrity.ok:true`, rows **8 pinned / 12 disabled**, `failures:[]`, `lanes_allowed:true`, `selftest:true` |
| `providers-get` | route **opencode-go** present (llm), models `["deepseek-v4.1-flash"]`, credential **present:false** (no key yet — expected) |
| `dump-config` | **fleet-roster row mounted** (checksRoot/gitRoot/engineRoot/credentialStore/sessionRoots), **subagent `maxActiveSubagents: 32`**, **llm-pi-ai `providers.opencode-go { apiKeyEnv: OPENCODE_GO_API_KEY }`** (no value) |
| credential entry path | non-TTY run prints `status: credential-entry-required` + the field list and **exits 0 without reading stdin or writing a value** — path works read-only |

## 1. The owner's ONE act (copy-paste; his terminal, this desktop)

```bash
node /home/cayleb/workspace/harness-build/candidate-031/package/bin/dsh-fleet.mjs first-run --credentials
```

What he sees, in order:
1. the field list header (official fields only; Cursor is not prompted);
2. exactly this prompt — **`OPENCODE_GO_API_KEY (opencode-go) [Enter to skip]:`** → he pastes/types the key (input is
   **not echoed**, not even stars) and presses Enter;
3. **`accepted (not echoed)`**;
4. `DEEPSEEK_API_KEY (deepseek) [Enter to skip]:` etc. — he presses **Enter to skip** every other field;
5. final summary: `status: written` and `stored: OPENCODE_GO_API_KEY via control-op` — exit 0.
   (Ctrl-C anywhere cancels the whole entry, nothing stored. Non-TTY anywhere → `status: credential-entry-required`,
   exit 0, nothing stored. Alternative owner route: fleet web UI → **Models** at `http://127.0.0.1:3091`, same field.)

## 2. Lead runs immediately after the owner says "done" (no owner involvement)

```bash
CLI=/home/cayleb/workspace/harness-build/candidate-031/package/bin/dsh-fleet.mjs
export DSH_FLEET_SOCKET=/run/fleet/control.sock
# 1. key stored + route live (value never printed)
node "$CLI" op providers-get            # expect opencode-go credential.present = true, models ["deepseek-v4.1-flash"]
node "$CLI" op policy-get '{"key":"roster.models"}'   # doctrine table (lead sonnet-5-5; builder/researcher opencode-go/deepseek-v4.1-flash; checker haiku-5-5)
node "$CLI" op roster-get               # cap 5
# 2. REAL provider-route proof: one roster lane on the working engine
node "$CLI" op lane-spawn '{"role":"builder","domain":"activation","start":true}'   # -> lane.principal builder:1, provider opencode-go
node "$CLI" op lane-get '{"id":"builder:1"}'          # lane state; provider/model names
node "$CLI" op ledger-get '{"agent":"builder:1"}'     # expect a REAL (non-synthetic) event, provider opencode-go, model deepseek-v4.1-flash, priced cost
node "$CLI" op lane-stop '{"id":"builder:1"}'         # release the lane after the proof
# 3. First real order (do NOT drive it yet; staging creates the real order on a roster host)
#    payload: reports/team-b-activation-package.md §2, with checkpoint_at REPLACED by a real future ISO/epoch
#    (or omitted -> defaults to now+24h). Template sha256 of the JSON as written: b9b7cd01…
#    Fire through the door's MCP client:
node ~/.dsh/strike-door/mcp-probe.mjs --url http://100.114.17.67:8789/mcp \
     --token-from-admin-socket /run/fleet/mcp-admin.sock \
     --call '{"name":"order.stage_real","arguments":{<the payload with a REAL checkpoint_at>}}'
#    expect: {ok:true, mode:"real-lane", real_lane:"fleetRoster mounted", order:{id:"oN", state:"open"}}
node "$CLI" op board-get '{"kind":"order","id":"oN"}'  # read back fixture:false, tasks []
```

## 3. Failure paths (report honestly; never substitute the stub)

- **Wrong/expired key or provider error:** `lane-spawn … start:true` yields a lane error / the provider error names
  itself; the ledger records **no** real event. Report the provider error verbatim; do **not** proceed to the order
  fire and **never** cite a `transport:"stub-turn"` / `model:"deterministic-stub"` turn as the route proof. The owner
  re-runs step 1 (each run re-prompts every official field; Enter skips) and the Lead repeats step 2.
- **Non-TTY / wrong terminal:** `status: credential-entry-required`, exit 0, nothing stored — the owner re-runs in a
  real terminal (or uses the web UI Models page).
- **Roster ops `lanes-unavailable`:** integrity/lanes gate is closed (check `status`); do not fire orders.
- **`order.stage_real` refusal:** `bad-args` for a missing/replaced checkpoint or a malformed done_when (fix payload);
  `lanes-unavailable` would mean the roster row is missing (it is mounted — verified).
- **Do not** print, echo, log or board-file the key; the value moves only owner → fleet credential store.

## 4. Fence / state notes

- The staged payload: **no Team-B staged file exists** — `/var/lib/fleet/state/mcp/staged/` is fleet-owned (uid 1000
  cannot read it; the S5 checker's earlier staged payload is that seat's, not rerun here). Our first-order payload is a
  report template that `order.stage_real` validates; **`checkpoint_at` must be a real future timestamp at fire time**
  (the template placeholder would be refused `bad-args`), after which `order.stage_real` would accept it (roster row
  mounted, schema fields present). **Not fired.**
- `NOX-DOOR.md` installed at `~/.dsh/strike-door/NOX-DOOR.md` is **byte-identical to the corrected version** filed at
  `board/reports/team-b-nox-door-corrected.md` (step 3 creates a real order on a roster host).
- No credential value appears in this file; no owner contact; $0.
