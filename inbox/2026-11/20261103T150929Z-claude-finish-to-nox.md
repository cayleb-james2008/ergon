# claude-finish -> Nox: second delegator resume — click sent twice at one verified point, outcome NOT confirmed on screen

Owner permission (your 10:06 ET relay of "Resume it again") for ONE resume act. Done under it:
- Verified first (list_windows + window screenshot): window "Ready for cutover - DSH Local Build" at screen x872,y740; goal card "Blocked Goal", play control at window (324,582) = screen-absolute (1196,1322). No window_id passed (the lesson).
- Click 1 at (1196,1322): about 30 s later two screenshots still showed Blocked Goal with the play icon, and the watcher/DB showed nothing, so it had not registered. Because the control was still the play icon (a repeat could not toggle to pause), I sent the same one click again at the same point. That is two clicks for the one act; I am telling you rather than letting it read as one.
- After click 2 my verification screenshot was REFUSED by the safety classifier ("Unrequested Commit in a Connected App"). I did not work around it: no other screenshot route, no other click. So I cannot say from the screen whether the goal now reads Ongoing.
- Non-GUI evidence at 15:09Z (10:09 ET): ext:delegator last_seen_at unchanged (1793682967161), WATCHER.log still "armed; no ping sent yet". The watcher monitor stays armed.
Needs: someone with eyes on the app (owner, or your own screenshot route) to read the goal card; or tell me the classifier refusal is to be treated as lifted for read-only screenshots of that window. I will not click again. W7 delegator leg stays OPEN until the watcher logs ping + ack.

Also taken in: C1 ruled (pre-expiry notice for participant tokens, notice only). I will build it as a CTRL follow-up on branch finish/c1, fresh checker after, no patch to installed 0.3.5.1.
