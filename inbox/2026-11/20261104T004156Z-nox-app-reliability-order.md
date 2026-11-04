# NOX ORDER — DSH APP UNATTENDED RELIABILITY (owner ask, 3 Nov 2026 ~19:41 ET)

Owner: "figure out why these restarts and crashes keep happening; I need a permanent fix — when I'm not here I won't be able to restart the app."

Evidence from Nox's live checks tonight (19:39–19:42 ET):
- The fleet Host backend did NOT stop: dsh-fleet-consolidated up since 15:07:56, MainPID 383250, NRestarts=0, ports 3092/8790/8796 listening, no errors in its journal since 18:00.
- The failure is at the APP layer: the Electron window showed "DeepSeek Harness is unavailable" while the Host was healthy; a session appeared paused; an OS popup said the desktop host stopped.
- FOUR app instances were alive at once (scopes app-deepseek-ai-dsh-desktop-1444786, -1471865, -266214, -3926702) — duplicate instances pile up across launches and never die. Prime suspect for the disconnect/paused symptoms: windows and sessions split across stale instances.
- Ruled out tonight: /tmp (44% bytes, 42% inodes), RAM (37G available).

Work order (DeepSeek side; Crouton-first per the engine order; fresh checker on the fix):
1. Root-cause the app<->Host disconnects and the session pauses from the app and Host logs; name the mechanism, not a guess.
2. Permanent fixes, in scope: single-instance enforcement (a new launch replaces or focuses the live one; stale scopes are reaped), app-side auto-reconnect to a healthy Host without an owner restart, a user-level watchdog that relaunches the app when its window dies or wedges (Nox's proven systemd-run relaunch route is the interim manual version — mechanise it), and session-pause recovery so a paused session resumes its goal after an app restart.
3. Prove unattended: kill the app window, watchdog restores it and the session continues — checker-verified, evidence filed.
Report as one consolidated filing. — Nox
