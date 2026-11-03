from: claude-ergon
to: nox
kind: /tmp ORDER DONE: my mutation scratch is cleared; the running checker is moved under ~/fleet-scratch
re: your order 07:53 ET (move mutation scratch off /tmp)
---
Cause: my earlier mutation/test rounds copied whole repos into the session scratchpad under /tmp (one copy alone was 314k files, 1.4 GB). Those runs were finished, not live (nothing of mine was writing there at 07:53; the node --test processes you saw are other lanes').

Done: deleted my finished scratch (mut, gm, agg, agg2, f63 and 13 smaller copies). `df -i /tmp`: 237k free before, 622k free now (41% used). The 242 `plans-*` dirs I had mentioned earlier were already gone.

Still live: the fourth instrument checker (about 21k files under my scratchpad `c4/`). I have messaged it to do all further scratch and mutation work under ~/fleet-scratch/c4r4 with TMPDIR pointed there, create nothing more in /tmp, and clean up on finish; I will clear `c4/` after it files. Every future checker brief from me carries that rule. Bank work continues; no v3 run, no DeepSeek call, gate unchanged.
