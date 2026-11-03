# claude-finish -> Nox: W7 delegator leg NOT closed — DSH app is down after the resume click

Facts (all read-only, 09:49 ET):
- 09:33 I clicked Resume once on the goal card (owner's one-act permission). Card went Blocked Goal -> Ongoing Goal, "Continuing goal 09:33", agent "Thinking". Nothing else touched.
- ext:delegator last_seen_at is still 1793682967161 (about 11 h old); the W7 watcher has logged nothing (armed, no ping sent). So no delegator bus check-in happened.
- The app is now GONE (no DSH window). coredumpctl shows electron SIGTRAP dumps at 09:37:47, 09:38:11-13 (five), 09:40:52, 09:44:26. Cause NOT established; the timing follows my resume by 4-11 minutes, so a link to the resumed goal is possible but unproven. (Earlier crash before my click: no OOM line, 0 agent kill hits.)
- Someone else is relaunching it over Tailscale SSH (/tmp/dsh-relaunch{,2,3}.log, pnpm run start:desktop). The attempts at 09:47, 09:47 and 09:49 fail at once with "Missing X server or $DISPLAY" (ozone x11), with DISPLAY=:0 and WAYLAND_DISPLAY=wayland-1 set by that command; they are not app crashes under load. Not mine; I have not touched them.
- Host pressure: load average 23, 41 GB of 64 GB used, 33 stray `node ... dsh-desktop-host/lib/index.js` processes alive.

Consequences:
- W7 delegator leg stays OPEN. The owner's permission was one act on one stuck instance, and it is used; I will not click, relaunch or resume anything again without a fresh word.
- The goal state is durable in the session log; whether it is active or paused after the crashes I have not read, and I will report it if you want it.
- No claim that the resume "worked": it started a turn that did not reach the bus before the app died.
Awaiting: who owns the relaunch (the display env of that SSH command is the likely defect), and whether you want me to read the session log goal state read-only.
