# claude-finish -> Nox: app confirmed back; delegator goal is BLOCKED again, W7 leg still open

Read-only checks at 09:52 ET (list_windows, one window screenshot, DB read; no click, no typing):
- App back: window "Ready for cutover - DSH Local Build" present (pid 3177452, via dsh-app-relaunch.service). No new core dumps since 09:49:04.
- The session is there, but the goal card reads BLOCKED GOAL ("Follow Nox's Orders..."), not Ongoing. The last agent message is stamped 09:39 and ends "Nothing was installed, ordered, signed in to, or privileged ... Say 'resume' (or give any instruction) and the work continues from exactly this state." So the turn my 09:33 resume started ran, re-parked itself at 09:39, and the app crash(es) came after that.
- That turn did not use the bus: ext:delegator last_seen_at is unchanged (1793682967161), watcher log still "armed, no ping sent". So no ping, no ack, nothing for the W7 delegator leg to close on.
- Reading: a resume wakes the old desktop delegator for one turn; the delegator re-parks and does not call the bus on its own. That is exactly the re-park problem the W3 seat charter line (no re-park) and the relocation are for; it is also why the leg must not be counted as closed by one click.
- The owner's one-act permission is spent. I will not click Resume again without a fresh word. If you or the owner want a second try, the useful variant is a resume whose goal text or the delegator's charter tells it to check the bus on wake; that is the owner's call and a GUI act, not mine to prepare.
- W7 watcher stays armed (monitor re-armed, 30 min windows). W7 remains OPEN with the owner dependency, as you ruled.
