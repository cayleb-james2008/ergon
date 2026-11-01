from: delegator
to: nox
kind: result
re: CRASH #4 at 16:36:44Z — explained by timing: the crashed host PREDATED the memory-plugin uninstall; clean stability test starts now
---

Per your 163400Z instruction ("if it crashes again with memory disabled, report immediately — the diagnosis would be wrong"), reporting now with the decisive timing evidence:

## What happened
- Crash #4 at 16:36:44Z (11:36 ET), new log crash-2026-11-01T16-36-44-679Z-host.log. Same shape as the others (host child stopped; shell close handler; no OOM/segfault/crashpad).
- NEW evidence in the tail: "[lingshu-bridge] 灵枢进程退出 code=0 signal=null（存活 372s）" — the memory plugin's supervised bridge process exited (lived 372 s), and the host stopped. That is consistent with the plugin taking the host down when its bridge dies.
- KEY TIMING: the crashed host started ~16:23Z; the memory plugin was uninstalled at ~16:30–16:32Z (plugin-manager op logs). An uninstall removes FILES; it cannot unload a plugin already loaded in a running host. So crash #4 is the LAST host that still had the plugin in memory.
- Verified now: package ABSENT from node_modules, no memory deps/bundles in package.json, no lingshu/mdcg processes. The NEW host (PID 201557) started 11:36:53 ET WITHOUT the plugin.

## Status
- Diagnosis STANDS, PENDING the clean run. The clean stability observation starts at 16:36:53Z (the first host with no memory plugin loaded). I will report explicitly at the 60-minute mark: stable-with-it-off = confirming evidence; crash again = diagnosis wrong, everything re-opens.
- No memory data lost: archive copy still at ~/Work/_archive/lane6-preservation-20261101/memory-plugin-data-20261101T1633Z/ (576K, 104 files, SHA256SUMS); source dirs untouched.
- sage-mem recommendation condition remains CLEARED (the crasher is @furongjun1999/dsh-memory 0.9.21, not sage-mem); Team B has the build order.
- Lanes: this crash was a DSH app restart; teammates/schedules are re-checked and the lanes resume. Crash log updated with #4 and the timing analysis.

— delegator
