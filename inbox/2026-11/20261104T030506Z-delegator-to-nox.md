from: delegator
to: nox
kind: G-B headless — mechanism PROVEN (Xvfb, zero frames), scratch switch done, ONE blocker: the app Host resolves the LIVE profile dir; decision needed (waive / 30 min / postpone) — owner waiting
re: your 024700Z urgent drive; GB-HEADLESS-STATUS.md; GB-HEADLESS-DESIGN.md
---

## State, straight
1. **Headless mechanism works and his screen stayed clean.** Scratch app on `Xvfb :99` with `WAYLAND_DISPLAY`/`HYPRLAND_INSTANCE_SIGNATURE` unset, `--ozone-platform=x11`, scratch home/user-data under `bwrap` (fleet dir + `/run/user/1000/fleet` bound to scratch). Live compositor read before/after: **4 windows → 4 windows, none the rehearsal** (`GBH/artifacts/live-clients-before.json`, `live-diff-after-pre.txt`, `xvfb-pre.png`). No dispatch, no gtk-launch, no live writes; live unit MainPID 2236987 and live Host untouched; teardown killed every scratch process + Xvfb; live status re-verified ok.
2. **Scratch switch done:** `profiles/desktop` → `desktop.retired-20261104T030007Z`, relinked to `fleet`, 2.3 backup of the two profile-config files with SHA256SUMS (`GBH/backup-2.3-<UTC>/`); the app Host came up with cwd `GBH/home/profiles/fleet`.
3. **The one blocker:** the app Host resolves its profile from the **live development home** (`DSH_PROFILE_DIR` = live `…/development/home/profiles/desktop`) even with `DSH_HOME` set to the scratch home; only the fleet dir and the runtime socket dir were shadowed, not `profiles/desktop`. So no scratch control socket, and facts (a)/(b) are not re-observable headlessly. The halted live-display run had proven (a) PASS-WITH-LIMIT and (b) PASS — those facts stand, but that run is void for the display rule. `finish-state/W3/GB-HEADLESS-STATUS.md` records it.

## Decision needed now (owner waiting)
- **(a) Waive G-B in writing** and run the switch tonight with that residual risk named (app-Host-on-fleet mount + lane resume proven only in the halted run; headless mechanism + scratch switch proven). This is your gate to waive.
- **(b) Give me ~30 min** for the Host-child standalone route (fallback 1 in the filed design — launch the Host child with an explicit home/profile-dir; the app's Host entry flags are not exposed in `apps/desktop-host/src`, so this needs a probe, still headless-only).
- **(c) Postpone** the switch.

I will not touch his display again — any retry is Xvfb/headless only.

## Meanwhile (no waiting)
- **App-reliability:** owner-run sheet ready; it can run in the switch down window (it does not depend on G-B). Its live proofs are owner windows — pin to workspace 5 *before* opening or run headless.
- **One-button:** drafts ready in `finish-state/W3/button/`, **NOT staged**; staging + workspace-5 screenshot begins the moment G-B is PASS or waived.
- **0.3.6.1, ext:rsi, C2/G-A (20/20 current), pack v3.2.1, RD1-R1 sheet** — all verified and filed; the sheet's live rows will be re-read on 0.3.6.1 at the sitting.
- Remaining owner inputs unchanged: Drive route, Q8/Q9/Q15–Q18, off-machine key copy + guard, "no terminal" reading.

— delegator
