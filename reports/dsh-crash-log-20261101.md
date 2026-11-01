# DSH app crash log — 1 Nov 2026 (for the performance diagnosis workstream, Nox 154800Z/160600Z)

Two crashes in one morning under fleet load. Logged by the delegator session; state recovered from disk both times.

## Crash 1 — ~15:05Z / 10:05 ET
- What was running: 5 durable team leads (ergon, fleet, trading, roblox, upkeep) + 3 in-flight checker subagents (DST adjudication, Ergon PR #10, trading G0/G1) + the delegator; active lane work was trading DST/design review and Ergon PR checking.
- Session/store state: the DSH app instance was torn down and a new one started (GUI port changed 45305 -> 37593). The session resumed; teammates and the 5 scheduled bookings survived (same session id); the 3 in-flight subagent checkers were LOST with no verdict files.
- Recovery: re-spawned the lost checkers; no lane work lost (all lane state on disk).

## Crash 2 — ~16:04Z / 11:04 ET
- What was running: 5 team leads + 2 in-flight checkers (trading G2 build, Roblox tool re-check) + the delegator; lane work was Roblox Studio in-window work resuming, trading G2 checker, and the V3 repair evidence review.
- Session/store state: app restarted again (GUI port changed 37593 -> 41967). Session resumed; teammates and all 5 scheduled bookings survived; the 2 in-flight checkers were LOST with no verdict files.
- Recovery: re-spawned both checkers; post-restart verification run (service/bundle/window state below).

## Post-restart verification (after crash 2)
- Lane 1 operator window: CLOSED at 15:47:40Z (window-end receipt 311d4760…), 16 minutes before crash 2 — nothing was half-open; nothing resumes half-open.
- Temporary sudoers/polkit rule: STILL INSTALLED (verified via `sudo -l` from an NNP-free transient unit: the 14-shape root alias and the fleet caller line are present). The owner removal relay has NOT been executed yet; the one-liner is outstanding. No window risk, but the rule must not persist silently.
- fleet-dsh: active/running, InvocationID e44cc343… (the V3-run restart; unchanged by the crash), NRestarts=0, integrity ok via the control socket (fleet-core pid 172431, uptime ~19 min, DB head seq 14829). Installed bundle untouched.
- Session-only armed items: 5 scheduled bookings still active (Monday 01:00/07:45/09:05/16:00 ET + GitHub 2-day cadence); re-spawned checkers as above; RESUME note current; board sensor alive (Nox ack 16:03:59Z).
- Not lost: any lane state (all on disk). Lost: subagent/child work in flight (no partial outputs) — the same crash-durability gap recorded after crash 1 and the reason Nox's write-first Effects/Outbox pattern matters.

## Pattern notes for the diagnosis
- Both crashes occurred with 7-8 concurrent child agents (5 leads + 2-3 subagent checkers) and long-lived sessions; neither was triggered by a specific lane action.
- The app restart preserves the session store, teammates and schedules; it does NOT preserve in-flight subagent turns or session-child processes (e.g. the armed DST runner in crash 1's predecessor).
- Recommendation for the diagnosis: treat "child-agent count x session age" as the leading hypothesis axis; the harness's own scheduling/durability workstream should assume session-child work is not crash-durable.

## Crash-log evidence (read 11:30 ET from the desktop app's own crash logs)
Log dir: /home/cayleb/.local/opt/deepseek-harness-official/apps/desktop/.desktop-build/development/electron-user-data/logs/
- crash-2026-11-01T15-05-04-783Z-host.log (10:05 ET), crash-2026-11-01T16-02-37-667Z-host.log (11:02 ET), crash-2026-11-01T16-22-47-729Z-host.log (11:22 ET).
- All three: source=host, phase=running, app=@deepseek-ai/dsh-desktop 0.2.2-alpha.2, electron 44.7.0, node 24.21.0.
- Error line in all three is identical in shape: "dsh desktop host stopped: <host stderr tail>", where the tail is the memory plugin's startup output:
  "[lingshu-bridge] [mdcg-mcp] 路径：记忆真源 root=.../.dsh-memory/data/mdcg · 身份/凭据根 aux=~/.mdcg ... [mdcg-mcp] 自报: ... @furongjun1999/dsh-memory ... [lingshu-bridge] [mdcg-mcp] 代校验：延迟导入 527 项 / 目标模块 52 个 / 已解析 191 项 / 可选缺失 25 项 · 代 b63d1f8a9d07".
- Stack is the shell's ChildProcess close handler (main.js:9837 maybeClose) — i.e. the SHELL did not crash; the host server child process stopped and the shell recorded its stderr tail.
- No renderer error console captured. No OOM, no segfault, no kernel kill in the journal window (14:50-16:30). No Electron crashpad dumps (Crashpad dir empty). No host JS exception captured.
- Plugin operation history in the logs is 31 Oct churn (install/remove/enable), context only.
- Memory plugin's own logs: @furongjun1999/dsh-memory 0.9.21 (latest published; npm time.modified 31 Oct 13:15Z); ~/.mdcg/sustain/heartbeat.md_cg.stamp present; mdcg _access.log shows an RRF recall at 16:21:39Z, ~68 s before crash 3 (and activity at 16:04:24Z after crash 2's restart).
- Interpretation: the recurring factor is the memory plugin (lingshu-bridge/mdcg-mcp) at host startup; the host exits silently later with no kernel-level cause. Strongest next test = temporarily disable that plugin and observe; alternative = capture the host's full stderr/exit code (the shell keeps only the tail).

## ACTION TAKEN (11:31 ET, Cayleb's decision): memory plugin disabled
- `plugin_manager set_bundle @furongjun1999/dsh-memory enabled=false` → result: stage=enable, enabled=false, changed=true, application=applied, warnings=[].
- Rationale: the only recurring content in all three crash-log error tails is this plugin (lingshu-bridge/mdcg-mcp); no OOM/segfault/crashpad/JS exception evidence exists. This is the controlled test.
- Observation plan: watch for the next ~60 min of normal work. If no crash → the memory plugin is the probable cause; report upstream (0.9.21 is the latest published) and decide keep-disabled / wait-for-fix. If it crashes again → the plugin is exonerated; next step is full host stderr/exit-code capture (the shell keeps only a tail) and the perf workstream continues.
- Reversal: `plugin_manager set_bundle @furongjun1999/dsh-memory enabled=true` (one call, no reinstall; package still installed at 0.9.21).
- Side effect: the memory tools (md_cg/flymemory-backed recall) are unavailable while disabled; the session and fleet work are unaffected.

## 11:34 ET — CULPRIT CONFIRMED BY THE OWNER
- Cayleb confirms the memory plugin caused the crashes; the disable STANDS (no re-enable, not even for a test, until a replacement is chosen and checker-passed through the normal gates).
- Plugin identified for the record: `@furongjun1999/dsh-memory` 0.9.21 ("灵枢 Lingshu / mdcg-mcp / lingshu-bridge"), npm package, profile bundle `@furongjun1999/dsh-memory`, entry row `include:furongjun1999-dsh-memory`. Uninstalled by the owner at 11:30:31 ET (plugin-manager operation log) — the package is now ABSENT from the profile; the disable-by-Lead at 11:31 was overtaken by the uninstall.
- Stability-with-it-off: observation in progress (app up since ~11:23; no crash yet). If it crashes again with the plugin gone, the diagnosis is wrong and everything re-opens.
- Memory data preserved (copy): ~/Work/_archive/lane6-preservation-20261101/memory-plugin-data-20261101T1633Z/ (576K, 104 files, SHA256SUMS.txt); source dirs also untouched in place.

## CRASH #4 — 16:36:44Z (11:36 ET) — explained: the crashed host PREDATED the uninstall
- New crash log: crash-2026-11-01T16-36-44-679Z-host.log (shell pid 193022).
- Error tail (same shape) now ENDS with a new line: "[lingshu-bridge] 灵枢进程退出 code=0 signal=null（存活 372s）" — the memory plugin's supervised bridge process exited, and the host stopped. That is consistent with the plugin taking the host down when its bridge dies.
- KEY TIMING: the crashed host started ~16:23Z (11:23 ET); the memory plugin was uninstalled at ~16:30-16:32Z (11:30-11:32 ET). Uninstalling removes the FILES; it cannot unload a plugin already loaded in a running host. So crash #4 is the last host that still had the plugin in memory — NOT evidence that the diagnosis is wrong.
- Verified now: package ABSENT from node_modules; no memory deps/bundles in package.json; no lingshu/mdcg processes. The NEW host (PID 201557) started 11:36:53 ET WITHOUT the plugin — the clean stability test starts at 16:36:53Z.
- Therefore: diagnosis stands PENDING the clean run. If this clean host crashes again, the diagnosis is wrong and everything re-opens (reported to Nox immediately with this nuance).

## 16:40 ET — DIAGNOSIS CORRECTED: the image-read path, not the memory plugin
- Every one of the four crashes follows a PNG screenshot written by Team D's Studio work by 11-50 s:
  10:04:14.68 studio-focused-100414.png -> crash 10:05:04.78 (+50 s);
  11:02:15.23 studio-ws3-110214.png -> crash 11:02:37.67 (+22 s);
  11:22:28.52 pre-open-112227.png (2.5 MB) -> crash 11:22:47.73 (+19 s);
  11:36:33.85 verify-post-crash-113632.png -> crash 11:36:44.68 (+11 s).
  (Screenshots at 10:01:41 and 11:00:59 did not crash, so not every read crashes — but every crash has a read just before it.)
- Every crash tail carries `[SharpElectronLinux] Warning: Binaries provided by Electron for use on Linux may be incompatible with sharp`. sharp = image processing; a native crash inside image decode leaves no JS exception, no OOM, no segfault in the journal, no crashpad entry — exactly the observed profile.
- The memory plugin was in every tail only because it prints at host STARTUP and nothing else printed afterwards; it was the tail of a silent host, not the last thing that ran. Memory plugin likely INNOCENT (remains uninstalled per owner sweep; sage-mem replacement plan unaffected).
- Mitigation in force: NO image reads/screenshots in this app until further notice (Team D switched to keyboard-only driving + MCP verification; Lead reading text only).
- Confirmation test (needs owner/Nox word because it risks one crash): read one known-large PNG through the app's image path, then a tiny PNG as control. Otherwise treat the correlation as the working diagnosis and escalate upstream: image processing is in-process; out-of-process/guarded image decode is the host-level fix.
- Upstream class: HOST property — same class as plugin crash isolation (no component fixes it).
