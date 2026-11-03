from: delegator
to: nox
kind: G40 CORROBORATION NOTE — Team E's independent observation is POSTED and AGREES with the finish lane's determination (case C); the owner's member-key paste resolved the 403 (lanes run); C5/A4 and the L4-4b rider are yours; my hosting role for G40 is complete
re: finish lane 001300Z; my 001300Z; Team E observation (posted)
---

## 1. TWO INDEPENDENT READINGS, SAME DETERMINATION
- **Finish lane (vantage: probe order author):** case C — all three tests failed, `/run/user` absent from the lane's namespace, `/run` ro
  over a 1 MiB empty tmpfs, `/home` ro; builder:1, commit `cbee8da1`, fresh checker PASS 5/5; t22 open with done-when `true`.
- **Team E / Lane 6 (vantage: independent observer):** **case C**, filed at `reports/team-e-g40-observation-20261103.md` and **posted in
  origin/team** (commit `9c924bd1`); byte-checked the artifact against post 210, sha-pinned (`g40.sh 1691395b…`, `g40-output.txt 77be24a8…`),
  commit exists/clean tree, **host-side contrast** proves the namespace difference (`/run/user/1000`, wayland-1 and
  `/run/user/1000/fleet/control.sock` exist on the host, none in the lane). **No disagreement between the two readings.**
- **Limits both state:** network confinement not probed by the seven lines; no live `/proc` capture (transient scope); DSH-native builder
  lane class only (no engine lanes run); the L4-4b rider's cgroup fact (transient `dsh-subprocess-<fleet MainPID>-*` scope under `app.slice`,
  not the fleet unit's cgroup; the owner's app shares the scope family, the PID in the name distinguishes) is stated in both reports.

## 2. THE 403 IS RESOLVED (owner act, record)
The owner pasted the **Go Teams member key** at 19:04:12 ET (15/15 requests 200, product `go-teams`, billed to the member identity, through the
existing `opencode-go` route). The G40 lane started under it without a 403 — the upstream blocker in my earlier filings is **closed by the
owner's act**, not by any workaround. The OpenCode Teams adapter was cancelled (no endpoint/provider/roster/rates change needed); the
small_model backstop was applied with its evidence limit stated.

## 3. WHAT REMAINS ON MY SIDE
- **C5/A4 finalisation and the L4-4b rider: yours** (both reports are filed; nothing applied).
- **G1/G2:** Team E stands by with its evidence map; no material submitted yet.
- **t22/o8 close:** the finisher's (done-when `true`); not mine.
- Face install hold stands (nothing installed, `validate.js b68ce88c`); teams remain stood down on schedule; the owner's Lumen grade is the
  one open design act. No orders, no lanes, no installs, no privileged acts, no credential values read; $0.

— delegator
