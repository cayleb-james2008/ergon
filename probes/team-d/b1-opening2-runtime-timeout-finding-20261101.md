# B1 opening 2 — real finding: fixture runtime never reports; Studio crashes (2026-11-01 14:02–14:08 ET)

Resume after the owner reopen (new Studio pid 302763, fresh 3/3, place `usg-desktop.rbxlx`, window fitted 2200×1000
floating — precondition confirmed in Edit: viewport 1837×744, Edit capture 1837×744, ratio 1.0). Standard resume done:
MCP verify → re-sync from B1 tree (`{"synced":101}`) → verify-clean clean → `--resume`.
Log: `b1-queue-opening2-resume-20261101.log`. Sealed hashes unchanged before/after (`84324d43…`, `c6651793…`).

## Result — stopped by the stop rule (same step failed twice; no retry)

```
{"fixture":"traffic:abort-contact","exit":1,"error":"TimeoutError: collector did not receive runtime within 155s"}
{"fixture":"traffic:abort-contact","exit":5,"error":"TimeoutError: collector did not receive runtime within 155s"}
{"queue":"stopped","entry":"traffic:abort-contact","exit":5,"resume":"--resume retries this entry"}
```

- **Play 1 (`20261101T190213Z-traffic-abort-contact`)**: LOCKED→PREFLIGHT→STAGED→PREPARED→PLAYING (t=4.5 s)→RAN
  (t=7.2 s); play state server `running,server,players:1` / client `running,player,scripts:true`. Prepare report:
  `bootstrapDisabled:true, helpers:2, httpEnabledForRuntime:true, verifiedSources:101`. Collector received **prepared**
  only — **no runtime phase**; after 155 s timeout the abort path ran clean (`stop:ok, cleanup:ok, verifyClean:ok`,
  collector `clean:true`). No crash dump for this Play.
- **Play 2 (`20261101T190501Z-traffic-abort-contact`)**: same timeout; during the wait the MCP session died
  (`Disconnected('no response to tools/call within 60s')`), abort could not run, `newCrashDumps:1`, **exit 5**.
- **Studio is gone**: `no RobloxStudioBeta process` (status/verify-clean exit 2 “no Studio running”). Plays counter
  2/3 for pid 302763 — the crash ended the session.
- Crash evidence: Windows minidumps **14:07 = 93af6366…dmp 97,638,464 B** and **13:59 = ee5db613…dmp 70,497,944 B**;
  `coredumpctl`: **wineserver SIGSEGV pid 302699 at 14:00:41** (the reopen attempt); Studio log
  `…logs/0.745.0.7451137_20261101T190042Z_Studio_B9882_last.log` (last lines: client disconnect, RCC
  `DisconnectServerShutdown`, then the process died ~14:07:54).

## What it means (not yet attributable)

- **Edit-side** prepare/cleanup both worked and verified 101 sources; the Play started and both server and client
  reported running with scripts alive. The missing piece is the fixture's **runtime helper POST** (installed in Edit,
  expected to run in Play and call the collector). That is the first real native fixture on the candidate and is
  exactly the class the never-run set exists to catch.
- Candidate explanations, not yet discriminated: (a) runtime helper script errors in Play (product/test finding —
  module or scenario flow); (b) Play-side HttpService cannot reach the collector (environment); (c) Wine Play
  instability ate the helper before it could post (Play 2 then crashed). No agent image was read; console text was not
  captured by the runner before the crash.
- **Studio stability trend**: previous session crashed after 3 Plays, this one after 2 (plus wineserver SIGSEGV on the
  reopen attempt). The batched 3-Plays-per-open plan will likely need more reopens than the nominal 11; record, do not
  alter the ceiling without the Lead.

## Proposed next diagnostic (awaiting Lead ruling; no retry made)

ONE diagnostic Play (1 of 3) after the relaunch, designed to separate (a)/(b)/(c) without images:
1. start Play, wait ~10 s, `get_console_output` (text) and save it — helper script errors/prints appear here;
2. from the Play **Server** datamodel, `execute_luau` a minimal HttpService POST to the collector URL with a probe nonce,
   then read the collector/console again — separates HTTP-path failure from helper-script failure;
3. stop Play, verify-clean; keep everything as evidence.
Optionally (tooling only, no game code): make the runner dump `get_console_output` on a runtime timeout so the next
failure is self-explaining.
