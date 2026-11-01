from: nox
to: delegator
kind: owner order
re: DSH PERFORMANCE — the app is badly laggy on Cayleb's machine; deep optimisation workstream opened (Lane 4 lead, Team B support). Diagnose with measurements FIRST
---

Cayleb's report (1 Nov ~10:42 ET): the DeepSeek Harness app on his desktop is very unoptimised — clicking a sub-agent to load its chat takes on the order of a couple of minutes, despite the machine having decent RAM. His order: deep-rooted optimisations in the plugin so the harness runs far faster and smoother. Today's app crash/restart may or may not be related — treat it as a data point, not a conclusion.

SEQUENCE — evidence before fixes, in one batch:
1. DIAGNOSE (measure, do not guess): where does the wait actually live? Profile on his machine: session/transcript store sizes per fleet session (are our sessions ballooning histories?), number of retained sessions, what loading one sub-agent chat actually reads, CPU/RAM/disk behaviour during a load, the fleet plugin's own contribution (panel polling/rendering, bus traffic, log volume), and the crash's correlation with store size if any. File the measurements on the board — numbers, not impressions.
2. OPTIMISE everything inside the plugin's reach, as candidates under the existing gates: fleet session/context hygiene (compaction and retention defaults so histories stop growing unbounded), plugin surfaces that render light and poll lazy, shipped config/default tuning, fan-out shapes that do not multiply transcript weight, log discipline. Each optimisation lands with a before/after measurement on the same load action.
3. HOST-BOUND GAP: whatever lag is proven to live in the DSH app's own renderer/loader — outside any plugin's reach — is filed as a named gap with the measurements attached (what action, how slow, what it reads), plus any upstreamable report text. A named, measured gap is an acceptable outcome; a shrug is not. Nobody attempts to rewrite the host app.

ACCEPTANCE SHAPE: performance becomes a standing property of the finished package, alongside minimalism — the fleet plugin must not make the host slower, and fleet working patterns must keep session weight bounded. Lane 4 owns it; fold progress into the consolidated digests, measurements first.

— Nox
