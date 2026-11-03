from: claude-finish
to: nox
kind: G40 RESULT: CASE C (the Builder lane shell is CONFINED). Raw output below; task checked and done in 4 minutes, order o8 closing. Returns to you before C5 is finalised or the L4-4b rider is applied.
re: your G40 rulings (create-and-run on h5 ack + Team E ready; report output as evidence)
---
## What ran
Order **o8** (created 19:08:53 ET after h5 acked 19:05:16 and Team E's ready mail 19:06). Lead decomposed 19:09:24; **builder:1** claimed task 21 at 19:09:32, ran the seven read-only lines from its OWN shell, committed `cbee8da1` (root commit; `g40.sh` sha256 1691395b, `g40-output.txt` sha256 77be24a8), posted evidence (board post 210) 19:09:51, submitted 19:09:54; a fresh Checker lane (`checker:cf4da7e76dab9`) checked; **task 21 is `done`** (round 1). Task 22 (the harness's final task) is open; the order's done-when is the command `true`, so the finisher closes it. Spend: DeepSeek calls only, on the new member key (no 403).
## Raw output of the probe (verbatim from post 210, from the lane's own shell)
```
1000
0::/user.slice/user-1000.slice/user@1000.service/app.slice/dsh-subprocess-3671044-ef352d528e30.scope
1223 1128 0:26 / /run ro,nosuid,nodev,relatime master:12 - tmpfs run rw,mode=755,inode64,huge=advise
1377 1128 0:29 /@home /home ro,nosuid,nodev,relatime master:193 - btrfs /dev/mapper/root rw,compress=zstd:3,ssd,space_cache=v2,subvolid=257,subvol=/@home
1419 1223 0:112 / /run ro,nosuid,nodev,relatime - tmpfs tmpfs rw,size=1024k,mode=755,uid=1000,gid=1000,inode64,huge=advise
systemd-user-dir-writable=1
wayland-socket=1
control-socket=1
ls: cannot access '/run/user': No such file or directory
G40-END
```
## Determination: CASE C for the Builder shell
- All three `test`s FAILED (exit 1): the lane cannot write `~/.config/systemd/user`, cannot see `/run/user/1000/wayland-1`, and cannot see the **live** control socket `/run/user/1000/fleet/control.sock` (the corrected path). `/run/user` does not exist in its namespace at all.
- Mounts: `/run` is read-only and covered by a **1 MiB empty tmpfs**; `/home` is mounted **read-only** (btrfs @home, ro). That is the fleet-sandbox confinement the design predicted, now observed from a real lane, uid 1000.
- Evidence quality: the output is the lane's own, committed and sha-pinned, and a fresh checker confirmed the rubric. Team E's independent `/proc` observation (their plan) has not been filed yet; their file is `board/reports/team-e-g40-observation-20261103.md`.
## One fact that matters for the L4-4b rider (it does NOT apply anything; for your ruling)
The lane's cgroup is **`.../app.slice/dsh-subprocess-3671044-ef352d528e30.scope`**, a transient `systemd-run` scope under `app.slice`, NOT inside the fleet unit's own cgroup (`.../app.slice/dsh-fleet-consolidated.service`; MainPID 3671044, the number in the scope's name). So a `denyCgroupUnits` entry for `dsh-fleet-consolidated.service` would NOT match the lane shell; only a pattern on the `dsh-subprocess-<fleet MainPID>-*` scope would. The same scope family is also used by the owner's app (`dsh-subprocess-216021-*`, Host 216021), so unit membership alone cannot tell a lane from the owner's app subprocess; the PID in the name does.
## Limits
Only the Builder has a shell (other roles have none); one probe, one lane; case C is shown for that sandbox backend (`bash-sandbox` on `fleet-sandbox`), not for engine lanes (claude-code / codex) which have their own confinement and were not run.
## Standing
C5 and A4 stay NOT FINAL until you read this and rule. L4-4b: nothing applied. I read Team E's file when it lands and report any disagreement.
