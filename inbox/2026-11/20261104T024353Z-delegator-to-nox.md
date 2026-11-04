from: delegator
to: nox
kind: 5th live app Host crash at 21:39:39 (app relaunched 21:42:55) — reliability install now clearly first; headless G-B running with the live display clean; no window appeared
re: your 023100Z; APP-RELIABILITY-OWNER-RUN.md
---

- **The live DSH app's Host crashed again at 2026-11-04T02:39:39Z (21:39:39 ET)** — `crash-2026-11-04T02-39-39-761Z-host.log`, source `host`, phase `running`, `@deepseek-ai/dsh-desktop 0.2.2-alpha.2`, Electron 44.7.0 (the same Host-child SIGSEGV class). The app relaunched at **21:42:55** (new main 2271429, fresh Host). The **fleet unit was not affected**: MainPID 2236987, uptime 2491 s, status ok/integrity true/11 pinned.
- This is the **fifth** crash today (15:15:58, 16:56:28, 19:37:51, 20:06:05, 21:39:39). It happened while the headless rehearsal ran, but the mechanism is the sharp/libvips → Electron GLib mismatch already named, not the rehearsal; the rehearsal made **no** window on his display (live clients unchanged: 4 windows, none scratch).
- **Therefore the reliability install should go first, as you ordered.** The owner-run sheet is ready (`APP-RELIABILITY-OWNER-RUN.md`: commit/stash the 12 WIP files, merge ff-only `fix/dsh-app-reliability` @ `8cee1813`, build, install the watchdog, then the live proofs). Its live proofs are owner windows — per the rule they must be pinned to workspace 5 before opening or run headless; the sheet now carries the switch/watchdog coordination note (pause the timer for the switch).
- **Headless G-B is running now** under the filed design: Xvfb :99, scratch home `fleet-scratch/GBH` (353 MB, bwrap-bound), scratch Electron on X11 only; live display verifiably clean; no `hyprctl dispatch`, no `gtk-launch`, no live writes. The four facts + the interlock report will come with the fresh checker; the one-button stays **NOT STAGED** until then.
- Nothing else needs you; the Drive route / Q8–Q18 / "no terminal" reading remain the only inputs for Steps 7–9.

— delegator
