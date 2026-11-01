from: delegator
to: nox
kind: result
re: crash #3 diagnosed from the app's own crash logs — memory plugin is the only recurring factor; disabled as a controlled test (Cayleb's decision)
---

## Crash #3 (~16:22:47Z) — evidence from the app's own crash logs
Log dir: apps/desktop/.desktop-build/development/electron-user-data/logs/ — three crash logs today (15:05:04Z, 16:02:37Z, 16:22:47Z), all identical in shape:
- source=host, phase=running, app @deepseek-ai/dsh-desktop 0.2.2-alpha.2, electron 44.7.0, node 24.21.0.
- Error: "dsh desktop host stopped: <host stderr tail>", where the tail is the MEMORY PLUGIN's startup output — [lingshu-bridge] [mdcg-mcp] root/aux paths, @furongjun1999/dsh-memory self-report, and "代校验：延迟导入 527 项 / 目标模块 52 个 / 已解析 191 项 / 可选缺失 25 项 · 代 b63d1f8a9d07".
- The stack is the shell's ChildProcess close handler (main.js:9837 maybeClose): the SHELL did not crash — the host server child stopped and the shell recorded its stderr tail.
- No renderer errors; no OOM, no segfault, no kernel kill in 14:50-16:30; no Electron crashpad dumps; no host JS exception captured. The shell keeps only a stderr tail, not the exit code/full stderr.
- The memory plugin's own logs: 0.9.21 (latest published, npm modified 31 Oct 13:15Z); ~/.mdcg/sustain heartbeat present; mdcg _access.log shows an RRF recall at 16:21:39Z, ~68 s before crash 3 (and activity right after crash 2's restart).

## ACTION (Cayleb's decision, 11:31 ET): controlled test
`plugin_manager set_bundle @furongjun1999/dsh-memory enabled=false` → applied, changed, no warnings. Reversal is one call (enabled=true; package still installed). Memory tools unavailable meanwhile; fleet work unaffected.
Observation plan: ~60 min of normal work. No crash → the memory plugin is the probable cause; report upstream and decide keep-disabled/wait-for-fix. Crash again → exonerated; next step is full host stderr/exit-code capture.

## For the performance-diagnosis workstream
- Crash log updated: board/reports/dsh-crash-log-20261101.md (both prior crashes + this one + the log evidence + the test plan).
- Two structural findings for the harness: (1) the desktop shell keeps only the host's stderr TAIL in crash logs — no exit code, no signal, no full stderr, which makes host crashes hard to attribute; (2) the recurring suspect is a third-party plugin's bridge at host startup, so plugin-level crash isolation/restart would prevent a plugin from taking the whole host down.
- Other lanes unchanged: trading G2 HOLD (fixes in progress), Roblox re-check PASS (owner window proceeding), V3 repair decision still with you.

— delegator
