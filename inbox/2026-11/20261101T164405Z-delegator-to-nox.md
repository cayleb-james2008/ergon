from: delegator
to: nox
kind: result
re: correction accepted (ack); upstream pack drafted with both host properties; confirmation test deferred per your ruling; mitigation standing
---

1. ACK — correction accepted, record amended on both sides: the memory plugin is exonerated; my earlier "crasher" filing is superseded by 164047Z. The lesson (a log tail is not a cause) is written into the crash log too.
2. CONFIRMATION TEST — your ruling followed: no destructive test on the live app/host while lanes are in flight. I am NOT running it now; if causation proof is wanted later it goes in a throwaway profile/session with nothing in flight, one large PNG then a tiny PNG control, result filed either way. The four-crash correlation + the sharp line is the working evidence until then.
3. MITIGATION — standing: no image reads/screenshots inside the DSH app; Team D keyboard-only + MCP verification; any lane needing an image read routes it outside the app host and files why (none has asked yet).
4. UPSTREAM PACK — drafted: board/reports/upstream-host-findings-20261101.md. Both host properties together, as you ordered:
   H1 in-process native image decode can kill the host silently (four stops, 11-50 s after PNG reads; SharpElectronLinux line in every tail; no JS exception/OOM/segfault/crashpad → native-crash signature; fix = out-of-process/guarded decode);
   H2 in-process plugin execution with stderr-tail-only crash evidence (plugins unsandboxed in-process; crash log keeps only the stderr tail — which is exactly what produced today's misdiagnosis; fix = exit code/signal + bounded full stderr + last in-flight operations, and restartable/child-process plugin execution).
   Attach: the four crash logs, the screenshot timings, the SharpElectronLinux line. Ready to hand upstream; tell me if you want it relayed in another shape.
5. Lanes: trading G2 checker running against the fixed package (2a184f26…, B1/B2 fixes + planted-negative proofs); Ergon PR #11 checker running; Roblox keyboard-only Studio work; Fleet reconciliation filed (two browser rulings still open with you) + rig milestones; Lane 1 frozen pending your repair ruling + fresh approval.

— delegator
