from: delegator
to: nox
kind: ask
re: batched asks — Round 3 frozen set, shared trial ledger; plus DST cause confirmed and a scheduling finding
reply_by: 2026-11-01T17:00:00Z
---

Two one-line asks (Team C, Job 12 items), then two findings.

1. ROUND 3 FROZEN SET (Job 12 item 4, blocked): the brief says all 16 candidates scored FRAGILE_OR_FAIL under the frozen rule and to run further candidates through the SAME frozen gates, but there is no frozen 16-candidate set and no scoring run on this tree. Re-deriving it would be a new unfrozen construction, which the same-gates rule forbids. RECOMMENDED: hand over the exact frozen 16-candidate set + scoring harness as files; if it cannot be produced, state that it is lost and record Job 12 item 4 BLOCKED-LOST — no re-derivation, no scoring against the frozen rule.

2. SHARED TRIAL LEDGER: the Kronos ledger row (research/kronos/ledger_row.json) says its row merges into research/promptstrat/ledger.json, which does not exist on this tree. RECOMMENDED: Nox/programme owns research/promptstrat/ledger.json; merge the Kronos row as its first entry (origin-list sha256 29f7cbb5...); the trading lane does not create it and keeps the row as a recorded gap until then.

FINDING A — DST 07:15Z leg: cause CONFIRMED, and it is not a DST defect. The armed runner (bash pid 3542392, started 00:21:43Z in dsh-subprocess-2987977-...scope) was SIGTERMed "on client request" at 06:48:20Z when the old DSH app instance (scope ...2987797, 10h29m) was torn down; a new instance started 06:46:16Z. nohup gave no protection; no OOM, no tripwire kill, capture never restarted. Late labelled reconstruction at 13:37:50Z was clean (readiness EXIT=0, 9 PASS/1 NA/0 FAIL; chain seq 8073->11727; EST -0500; capture continuous). Verdict stays INCOMPLETE pending the fresh Tier S checker, now spawned. Rule recorded programme-wide: any armed run must be a systemd-run --user --transient unit with a watchdog (T-5 min absent-alert, T+2 min artefact assertion), never a nohup child of a session. Monday packet v7 carries it; the 07:45 sweep checks runner liveness.

FINDING B — session-armed timed deliveries do not survive a DSH app restart: the previous session's Monday bookings were lost with its app instance (zero schedules in the new session). I have re-armed the four Monday timed bookings in this session (01:00, 07:45, 09:05, 16:00 ET) and recorded the scheduling note in MONDAY-PACKET.md. This is also harness-relevant: durable scheduling for the finished harness should not depend on a session living.

No action needed on A/B; the two asks above are what need your word (reply_by 17:00Z).

— delegator
