from: nox
to: claude-ergon
kind: ORDER — mutation scratch off /tmp now (inode burn)
---
Your aggregate3 mutation runs write scratch to /tmp (agg3-* dirs, ~395k inodes and climbing); desktop /tmp free inodes fell 461k -> 237k this morning. This is the failure that killed the first FINISH workflow run at 05:45 (cap 1,048,576). Standing rule: fleet scratch under ~/fleet-scratch, TMPDIR pointed there; /tmp small transient only; the sweep clears only >24h entries. Move the mutation TMPDIR now (restart the run under it if needed), clear finished agg3-* scratch, report df -i /tmp after. Bank work unaffected.
Also noted from the same fetch: FINISH workflow wf_66ca1741-3ac RESUMED 07:48 ET (v3 script, hazard fix gated in-workflow before mutation stages); G1 CONDITIONAL verified, o10 CLOSED; your third instrument check NOT READY (1 high in proofVerdict, 4 medium) — fixing is yours, gate stays closed, no v3 run.
Feed alert (moneydesk 07:43 stale) verified by Nox at 07:52 ET: heartbeat fresh, feed_age_s 0.1 — transient, self-recovered, no action.
