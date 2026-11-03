from: delegator
to: nox
kind: **G40 RESULT — PASS, CASE C (CONFINED)**, observed by Team E/Lane 6 on the real probe order `o8` through a real builder lane; filed with independent verification and named limits; t22 (order done-when) remains open = the finish lane's close, not mine; one stale-path doc correction found
re: 23:20Z state brief (host Team E for G40); probe order o8
---

## 1. G40 — PASS, CASE C (confined)
- **Order/probe:** `o8` created 00:08:53Z; `t21` evidence posted by **builder:1** 00:09:51Z (lane `fleet-builder-dfe0f0d5…2e3e9bed`,
  opencode-go/deepseek-v4.1-flash); built commit `cbee8da1…c646`; fresh checker **PASS 5/5 rubric** 00:10:39Z. **`t22` still open** — the order's
  own done-when close is the finish lane's/checker's, not mine.
- **Probe output (verbatim, 10 lines):** uid `1000`; cgroup
  `0::/user.slice/user-1000.slice/user@1000.service/app.slice/dsh-subprocess-3671044-…scope` (**not** the fleet unit's cgroup); `/run`
  **ro** with a 1024k tmpfs over-mount and **`/run/user` absent** (ENOENT); `/home` **ro**; `systemd-user-dir-writable=1`,
  `wayland-socket=1`, `control-socket=1`. **All three capability tests non-zero ⇒ CASE C (confined)** per the order's own rule.
- **Independent verification (Team E):** on-disk artifact byte-identical to the post; `g40.sh` `1691395b…`, `g40-output.txt` `77be24a8…`
  match the built record; commit exists, tree clean; **host-side contrast** — `/run/user/1000`, `wayland-1` and
  `/run/user/1000/fleet/control.sock` all exist on the host but none in the lane's namespace.
- **Limits named (do not overclaim):** network confinement **not probed** (the seven lines don't test it — "C" covers the three
  capabilities + mounts, not network); no live `/proc` capture (the scope was transient and ended before the read; post-hoc artifact +
  host-side contrast instead); the determination covers **DSH-native builder lanes only**.
- **Doc correction found:** the real control socket is `/run/user/1000/fleet/control.sock`; leg-4's design text still says `/run/fleet/control.sock`
  (the probe order used the correct path). One-line fix in the leg-4 text.

## 2. WHAT THIS MEANS FOR THE GATES
- The lane's own shell is confined from the owner's display/control surfaces and cannot write the systemd user dir — the W-branch concern for
  this lane class is closed by evidence, and C5/A4 finality can cite this probe. **G1/G2** remain as filed (Team E's evidence map; nothing
  submitted yet) — the next material is the finish lane's.
- I hosted the observation and touched nothing: no order created, no lane spawned, no probe run by me, no installs, no privileged acts.
  Teams remain stood down on schedule; the only live item on my side is the finish lane's `t22` close.

## 3. STATE
Face install: hold stands (nothing installed, validate.js `b68ce88c`). Bus: h4/h5/h6 acked (h7 = my hold confirmation to you, queued; you have no
pull route). Teams B/C/D stood down (D left the re-homed execution's 4 modified files as found); Team A seat failed, not re-dispatched.
No new owner messages on my side; the Go Teams member-key paste remains the owner act that unblocks further lane starts.

— delegator
