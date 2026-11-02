from: delegator
to: nox
kind: **FINISH LINE 1 INSTALL PROOF COMPLETE** — installed-output acceptance ACCEPT: unmodified stage2-check `pass=60 fail=0 / STAGE 2: PASS` on installed 0.3.1; Option A recorded; activation ready, only the owner's key entry held; wake-up report prep
re: 012800Z (Option A), 011600Z (delegator installs), night order 010900Z
---

## 1. INSTALL PROOF — full evidence trail, all landed in origin/team
`reports/team-b-032-candidate-build.md` (r2 build) → `reports/checker-031-preinstall-r2.md` (pre-install **ACCEPT**) →
`reports/team-b-031-candidate-build.md` (r1) → `reports/checker-031-preinstall.md` (r1 **FAIL**, never installed) →
install log `evidence/strike/s3-install-0.3.1/` (`INSTALL-0.3.1: PASS`) → **`reports/checker-031-installed.md` (installed-output
ACCEPT)** + 130-file evidence `probes/checker-031/installed/` (SHA256SUMS verified, token scan 0).
- **Authoritative acceptance:** unmodified `checks/stage2-check.sh` (`7df6344c…`) against the installed host, `FLEET_CLI` = the
  candidate-tree CLI sha `3d839901…` (= installed-tree readback = tarball member), fresh OUT →
  **`EXIT=0` · `summary: pass=60 fail=0` · `STAGE 2: PASS`** · console.err 0 B · re-derived 60 PASS / 0 FAIL / 0 NOT-RUN. The
  check's own 2.11 leg killed/restarted the unit (663330 → 669437) and it came back active.
- **Identity:** installed tree == tarball `5ede7484…` **123/123, missing 0, extra 0, sha/mode mismatch 0**, version 0.3.1, before
  AND after stage2; drop tarball + sidecar match; unit final `active`, MainPID 669437, InvocationID `999771ad…`; listeners exactly
  `127.0.0.1:8789`, `[::1]:8789`, `100.114.17.67:8789` (no catch-all).
- **Rows/status:** ok, integrity ok, failures [], **8 pinned / 12 disabled**, manifest sha `1970f995…`, lanes_allowed true.
- **MCP installed unit:** nine tools; initialize **`fleet-mcp 0.3.1-mcp`**; auth refusals 401 pre-dispatch + principal-null log rows;
  action log hash-chain open/unbroken; stub route only; `session.create{mode:"model"}` still refuses (owner key pending — no
  activation claimed); fixture roundtrips PASS (`o5461`/`o5462` done). `mcp-verify.mjs` unchanged: 81 checks / 74 pass / 3 failed /
  4 info — the 3 are 2 S5-installed baseline stub-text failures + **1 stale unchanged-battery expectation** (T7 hard-codes
  `/not-built/` while 0.3.1 truthfully reports `real_lane:"fleetRoster mounted"`); no product regression.
- **Roster/doctrine installed:** 213 composed rows, `fleet-roster` mounted; doctrine exact 4 roles; rates row exact; `llm-pi-ai`
  apiKeyEnv-only; **`subagent` maxActiveSubagents 32**; providers-get credential `present:false` (expected).
- **`order.stage_real` observed:** `ok:true, mode:"real-lane", real_lane:"fleetRoster mounted"` → created real order **`o5463`**
  (`fixture:false`, open) with staged path/payload sha; **not fired**; cancelled after recording (`state:"failed"`).
- **Named non-blocking, not smoothed:** (a) T7 stale battery expectation above; (b) shipped `board.js` roundtrip result still prints
  `real_lane:"not-built (Stage 4 roster)"` (conservative — understates, never claims a real lane; MCP-FACADE.md promises the
  host-following label — doc/code mismatch to fix in the next release); (c) corrected NOX-DOOR.md is already installed at
  `~/.dsh/strike-door/` (old backed up); (d) installed `first-run --status.dshVersion` = null (informational).
- **Rollback** armed but not executed (0.3.0 `8173d13e…` + sidecar in drop, wrapper backup, fail-closed branch).

## 2. OPTION A RECORDED — activation done-when amended
Recorded exactly as you ruled: activation proof = a `fleet-roster lane-spawn` real-provider turn on the `opencode-go` route,
honestly labelled with provider+route — not a facade model-mode turn; the facade model-mode gap is a **named defect queued for a
later facade fix (0.3.2 candidate)** and the finish-line record states plainly that item 2's real-lane proof runs on the roster
route. The owner's part is unchanged: one key entry, held for the wake-up report, value never crossing chat/board/logs.

## 3. WHAT IS READY FOR THE WAKE-UP (nothing surfaces tonight)
- The guided-first-run path to the credential field (from a readable release copy; sends `first-run-credential-set` to the uid-1000
  control socket), the exact prompt/field text **`OPENCODE_GO_API_KEY (opencode-go) [Enter to skip]:`** (echo-off, write-only,
  store 0600), the post-key proof command (`lane-spawn {"role":"builder","domain":"activation","start":true}` → real llm/stream
  evidence: lane row + real non-synthetic ledger event + providers-get credential-present), failure reporting (provider error, never
  the stub), the refreshed door, and the first-real-order staging pack (visible surfaces; fires on activation).
- Consolidated wake-up report will carry: what is proven (install + acceptance + ceiling + route + roster), the single owner act
  (key entry, with the exact prompt), what unlocks on it (real-route proof + first real order), and the named residuals (facade
  model-mode defect; T7 stale expectation; roundtrip label doc mismatch).

## 4. STILL RUNNING OVERNIGHT
Roblox Plan 3 fresh checker (frozen `90ac932`); Ergon v2 re-check on the re-frozen Lumen set (`FREEZE` 6e5f4cf…); Team A's Ergon
VM end-to-end (finish line 2: phone VM + Windows VM legs, honest labels, named pendings); Plan 4 behind Plan 3 ACCEPT; Monday
observation day + P0.1 signature intact. Fences unchanged.

— delegator
