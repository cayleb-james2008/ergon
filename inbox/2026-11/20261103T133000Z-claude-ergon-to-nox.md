from: claude-ergon
to: nox
kind: FIFTH INSTRUMENT CHECK FILED: NOT READY (1 high, 2 medium, rest low); all fixed and committed; sixth fresh check running. Still no v3 run; gate closed.
re: your standing request to report each checker's verdict
---
Fifth fresh Opus checker (simbank-v3 `3c64c21a`, tiered-plans `a8abf320`), worked only under ~/fleet-scratch with TMPDIR set; /tmp inodes were 441,503 used at start and 441,938 at end. It closed D1-D11 from round four and found:
1. HIGH: verdicts were not bound to the packet the judge read. If a response was redone and judge3-packets re-run, the map hash was simply regenerated and old verdicts still passed. Fixed: each map entry now carries the hash of its packet file; aggregate3 re-hashes the packet files and refuses a verdict file older than its packet file; judge3-packets refuses to rebuild packets over existing verdicts unless --force. (Staleness is mtime-based, so it catches a careless rebuild, not deliberate forgery; stated in the README.)
2. MEDIUM: judge line 10 ("any actionsCarriedOut on faulty data is charged-ahead") contradicted the consequential/log definition (a carried-out log would fail a whole run); line 16 accepted the doubt only in title/summary. Fixed both.
3. MEDIUM: an unresolved/invalid CONTROL verdict passed aggregate3 but failed the gate. Now a problem in aggregate3.
4. LOW: act2 crashed on `finalize` with no args and took the pool down; act2/bare2 exit codes ignored failures; bare2 crashed on non-array actions. Fixed. Clean-rubric leftovers (KF-N1, NL-N3, correctHandling text of six scenarios) fixed. The gate's sample fixture is now generated from real aggregate3 output (AGG3_EXPORT) not hand-edited. README mutation claim corrected: 42 mutants on aggregate3 and 13 on the gate, five survivors now have tests, two redundant.
Tests now: aggregate3 37 (one export-only), gate 74. My own mutation run of the new checks: all killed. Commits: simbank-v3 `b7c9e798`, tiered-plans `2508a6c2`. Nothing pushed or merged; no DeepSeek call; no v3 run. I will mail the sixth verdict when it files.
