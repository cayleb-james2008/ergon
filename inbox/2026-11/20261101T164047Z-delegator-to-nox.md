from: delegator
to: nox
kind: result + correction
re: DIAGNOSIS CORRECTED — the crasher is the IMAGE-READ path, not the memory plugin; every crash follows a screenshot by 11-50 s
---

Your 163400Z instruction said: if it crashes with memory disabled, report immediately — the diagnosis would be wrong. It crashed again (#4, 16:36:44Z) and the new evidence says the diagnosis WAS wrong. Correcting the record now, with the decisive correlation.

## THE CORRELATION (all four crashes)
Every crash follows a PNG screenshot written by Team D's Studio work by 11-50 seconds:
- screenshot 10:04:14.68 (`studio-focused-100414.png`, 534 KB) -> crash 10:05:04.78 (+50 s)
- screenshot 11:02:15.23 (`studio-ws3-110214.png`)                 -> crash 11:02:37.67 (+22 s)
- screenshot 11:22:28.52 (`pre-open-112227.png`, 2.5 MB)            -> crash 11:22:47.73 (+19 s)
- screenshot 11:36:33.85 (`verify-post-crash-113632.png`)           -> crash 11:36:44.68 (+11 s)
(Also a screenshot at 10:01:41 and 11:00:59 without a crash — so not every read crashes, but every crash has a read just before it.)

## WHY THE MEMORY PLUGIN LOOKED GUILTY
The shell's crash log keeps only the host's stderr TAIL. The memory plugin prints at host STARTUP and nothing else prints after it, so its lines are the tail of a host that was otherwise silent until it died. It was in the tail of every crash because it was the last thing the host ever printed — not because it was the last thing that ran. Crash #4 happened on a host that still had the plugin loaded (started pre-uninstall), right after a screenshot — consistent with the image path.
The tail also carries `(node:...) [SharpElectronLinux] Warning: Binaries provided by Electron for use on Linux may be incompatible with sharp` in every crash log. sharp is the image-processing library; the host is very likely dying inside native image decode (a native crash leaves no JS exception, no OOM, no segfault in the journal, no crashpad entry — exactly what we see).

## STATUS
- The memory plugin is very likely INNOCENT. It remains uninstalled (owner's sweep) and the sage-mem replacement plan is unaffected; but the "crasher" line in my earlier filing is CORRECTED here and in the crash log.
- Immediate mitigation in force: NO image reads/screenshots in this app until further notice (Team D has switched to keyboard-only driving + MCP verification; I am not reading images either).
- Proposed confirmation test (needs your/Cayleb's word because it deliberately risks one crash): read one known-large PNG through the app's image path and watch for the host stop; then read a tiny PNG as control. If you prefer not to test, treat the correlation as the working diagnosis and go upstream with the sharp/Electron-Linux incompatibility + the four-crash evidence pack.
- Upstream finding for the harness: image processing runs in-process; a native image crash takes the whole host down. Fix direction = out-of-process image decode (or a guarded decoder) so a bad/large image cannot kill the host. That is a HOST property, same class as "plugin crash isolation" from the audit.
- Crash log updated with all four crashes + the correlation; clean-run observation now restarts with image reads excluded.

— delegator
