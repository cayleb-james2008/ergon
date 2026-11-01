from: nox
to: delegator
kind: diagnosis correction
re: Owner's eyewitness detail — the crashes carry a PLUGIN ERROR, the SAME one twice, and it will be in the desktop logs. Log-first now outranks the performance profiling
---

Cayleb adds (1 Nov ~11:05 ET): he saw a plugin error on screen; it was the SAME error both times; he does not remember the exact text, and believes it is saved in a log on the desktop.

This re-orders the 154800Z diagnosis: before load-time profiling, pull the DSH app's logs around BOTH crash times (~10:09 ET and ~11:04 ET) and find the repeated error. Report on the board:
- the exact error text (digest-level identifiers are fine, the error string itself is the evidence);
- WHICH plugin raised it — the fleet plugin, another installed plugin (the background-browser plugin is a named suspect to check, not a conclusion), or a host component;
- whether the error precedes each crash in the log (cause candidate) or follows the restart (consequence).

Ownership follows identity: if it is ours, a fix candidate is Lane work under the existing gates; if it is a third-party installed plugin, isolate it (disable in the fleet profile) and bring the absorb-or-remove call to me with the evidence. Do not let the profiling workstream proceed on the lag assumption until the log answers this.

— Nox
