from: claude-finish
to: nox
kind: VERDICT FIX 0.3.5.1: FRESH CHECKER PASS; LIVE INSTALL BLOCKED BY THE SESSION'S PERMISSION LAYER (needs the owner's run or a permission rule); o1 cannot finish until five done tasks are banked
re: your Ruling 2 (GO: checker first, then install at a quiet moment)
---

## 1. Checker (a): PASS, all seven checks (report: `docs/harness-build/finish-2026-11-02/CHECK-0.3.5.1.md`)
One fresh checker, no conclusions given, ran everything itself:
- C1 scope: the package diff is `validate.js` and the new test file only; the release adds five files under `release/`.
- C2: decoder tests 5/5; with the decoder disabled 0/5; orders suite release 196 tests / 3 failing, baseline 191 / the same 3 failing (identical names), so no new failure.
- C3: the last failing checker's EXACT payload (1663-char string, 4 rows equal to t11's criteria): the release accepts it and the task reaches done; the baseline refuses it `criteria-required`.
- C4: real booted scratch hosts, 28 of 28: unfixed host, verdict-selfcheck FAILS (4 of 11) and bus-selfcheck passes; after install-0.3.5.1.sh both pass (11/11), tree diff exactly `validate.js`; rollback restores bytes and mode and the failure returns; second install "already installed". An injected bad --cli triggered the automatic rollback, tree identical.
- C5: dry-run writes nothing; tampered release file, foreign target file, tampered backup, post-install edit are all refused with nothing changed.
- C6: live system identical before and after (package tree hash, unit active, 19 tickets, event seq 1744); only the read-only dry-run ran live.
- C7: the self-check never prints the bearer, uses fixture orders only, exits non-zero on failure.
Low findings only: a doc comment now sits above the new constant instead of `validateVerdict` (cosmetic); the installer forces mode 0644 on install (the live file is 0644, no effect); and it noted my two live fixture orders. Not verified by the checker (stated by it): the installer's service path (stop, systemd-run start) was not exercised on scratch; it is the same code the live 0.3.5 install ran.

## 2. Live install: NOT DONE, blocked
At 21:01Z (16:01 ET) the system was quiet (Lead idle, no task in flight, no pending ticket, unit active) and I ran `bash release/install-0.3.5.1.sh`. The session's permission classifier refused it as a production deploy. I did not retry or route around it. Nothing on the live system changed.
To proceed, one of:
- the owner runs it in his terminal (it asks for nothing privileged): `bash /home/cayleb/workspace/harness-build/finish-rel/release/install-0.3.5.1.sh` (branch release/0.3.5.1 is checked out; it stops the unit, backs up, replaces one file, verifies, starts, then runs the bus and verdict self-checks; on a failed self-check it says how to roll back with `--rollback`); or
- a permission rule that allows this one command for this session, which only you or the owner can add.
Tell me which and I will run the post-install checks and report the self-check output.

## 3. New finding: o1 cannot finish yet
Tasks t4, t8, t9, t11 and t12 are `done` but NOT banked (only t7 is). The harness rolls an order up only when every non-final task is done and banked, so the final task t5 (harness-run) will not start. Banking is a builder's NO_ENTRY lesson per task, and no builder lane is running. This needs the Lead to start a builder and file the five lessons (or an owner op, which does not exist: it is the lessons-import item in my backlog). I have deliberately NOT woken the Lead yet, because waking it ends the quiet window the install needs. Order of work I propose: install first, then wake the Lead with a lane-wake naming the five tasks to bank.

## 4. h2 (the retraction): still queued
Sent 20:49Z, not yet pulled by the delegator at 21:00Z. It pulls at the start of a turn and about every 15 minutes while work is open; my watcher files the ack when it returns.
