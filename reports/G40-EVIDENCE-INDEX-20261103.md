# G40 EVIDENCE INDEX — one page for Nox's C5/A4 finalisation (Lead record-keeping, 2 Nov 19:16 ET)
Every element below is on the board/shared repo or in the fleet DB with ids; this page only indexes them. **Nothing here is a new claim.**

## 1. Authority and trigger
- Nox's 23:20Z state brief: Team E hosts G40; the probe order is created once the delegator's h5 ack **and** Team E's readiness mail land.
- **h5 ack**: delegator, 19:05:16 ET (bus h4/h5/h6 all acked; h7 = hold confirmation sent to nox).
- **Team E readiness mail**: `inbox/2026-11/20261103T000604Z-team-e-to-nox-g40-readiness.md` (posted in origin/team, commit `2b631004`).

## 2. The probe
| element | value | where |
|---|---|---|
| order | `o8` "G40 PROBE (throwaway): lane-shell confinement, read-only" (tier 1, one task) | fleet DB; board post 207 |
| creation | 19:08:53 ET by `cayleb-local` (owner-op route, finish lane) | post 207 |
| lead claim/preflight/decompose | 19:09:07 / no questions / 19:09:24 → `t21` (+ harness final `t22`) | posts 208, 209 |
| lane | `builder:1`, session `fleet-builder-dfe0f0d5…2e3e9bed`, opencode-go/deepseek-v4.1-flash, workspace `$DH/fleet/work/lanes/builder_1-2e3e9bed` | roster; post 210 |
| shell scope | `dsh-subprocess-3671044-ef352d528e30.scope` (transient `systemd-run` under `user@1000/.../app.slice`) | probe output |
| evidence post | post **210** — full 10-line `g40-output.txt` text + commit + hashes | board `o8` |
| commit | **`cbee8da1c6ea7a908d821bbf3bac52271c44c646`** (root commit, clean tree) | post 211 |
| artifact hashes | `g40.sh` **`1691395b…7e8437d2`**, `g40-output.txt` **`77be24a8…e5e80e1`** | post 211; Team E byte-check |
| checker | `checker:cf4da7e76dab9`, **verdict PASS 5/5** round 1, 19:10:39 ET (verdict id 14) | posts 212/213 |
| task states | `t21` **done**; `t22` open (done-when `true`; the harness finisher closes it) | fleet DB at 19:16 ET |

## 3. The determination — CASE C (confined), two independent readings
- **Probe author (finish lane `001300Z`):** all three tests failed (`systemd-user-dir-writable=1`, `wayland-socket=1`, `control-socket=1`),
  `/run/user` absent (ENOENT), `/run` ro over a 1 MiB empty tmpfs, `/home` ro ⇒ **case C**.
- **Independent observer (Team E / Lane 6):** `reports/team-e-g40-observation-20261103.md` (posted in origin/team, commit `9c924bd1`) —
  byte-checked the artifact against post 210, sha-pinned hashes match the built record, commit exists/clean, **host-side contrast** (host has
  `/run/user/1000`, `wayland-1`, `fleet/control.sock`; the lane's namespace had none) ⇒ **case C**, **no disagreement**.
- **Limits both state:** network confinement **not probed** by the seven lines; no live `/proc` capture (transient scope); DSH-native builder
  lane class only (no engine lanes run). The C determination covers the three capabilities + mounts, **not network**.

## 4. The L4-4b rider fact (for Nox's ruling; nothing applied)
The lane's cgroup is a **transient `dsh-subprocess-<fleet MainPID>-*` scope under `app.slice`**, not inside the fleet unit's own cgroup
(`.../dsh-fleet-consolidated.service`). A `denyCgroupUnits` entry for the fleet unit would **not** match the lane shell; only a pattern on the
scope family would. The owner's app uses the same family (`dsh-subprocess-<host PID>-*`), so unit membership alone cannot distinguish a lane
from an owner-app subprocess — **the PID in the name does**.
Doc correction found: the real control socket is **`/run/user/1000/fleet/control.sock`**; leg-4's design text still says `/run/fleet/control.sock`.

## 5. What is NOT claimed
No network-confinement result; no engine-lane result; no C5/A4 finality (Nox's); no application of the L4-4b rider; `t22`/order close is the
harness finisher's, not this index's.

— delegator (record-keeping), 19:16 ET
## 6. DISPOSITION (record update, 19:21 ET — the finish lane's, not mine)
- The finish lane recorded Nox's G40 rulings in its PA-9 branch (commit `8698e60`): **C5/A4 markers changed from NOT FINAL to
  FINAL** (`comms.control-channel-trusted` still default **false**; comms tests 77/77). **L4-4b's `denyCgroupUnits` prong: nothing
  applied**; L4-A proceeds without it.
- Team E's observation **agrees** with the probe author's determination — no mismatch, nothing reopened. The two named limits stand:
  network not probed (one more read-only line in a fresh throwaway order, if Nox wants that leg closed), and no live `/proc` capture
  (the scope was transient; the committed artifact is the evidence). Backend scope: `bash-sandbox` only.
- `o8`'s final task/order closure remains the harness finisher's; the finish lane is watching it and will report if it does not close.

## 7. NETWORK LEG (o9, G40-NET) — observed 19:19–19:22 ET; both throwaway orders closed administratively
- **o9** created 00:19:22Z; builder:1 ran the connection-attempts-only script; built commit `47d0e913…71be` (parent `cbee8da`, clean tree);
  fresh checker **PASS** 00:21:35Z (verdict 15); `t23` done, `t24` open → then **closed administratively by the owner op** (stored `failed`;
  harness artefact, not a failed deliverable).
- **Probe output:** `net:[4026533376]` (distinct from host `4026531833`); `/proc/net/dev` lists **only `lo`** with zero counters; one in-ns
  listener `127.0.0.1:3128`; **all seven connects `exit=1`** — 127.0.0.1:8790/3092/8791/9333/8799/53 and 1.1.1.1:443; none `0`, none `124`.
- **Observer (Team E, `team-e-g40n-observation-20261103.md`):** direct-reachability **C (confined)** — isolated net ns, loopback-only, no host
  service or public internet reachable; **host-side contrast** (host listeners exist on :8790/:3092/:8791/:9333 yet failed from the lane;
  :8799 and :53 have no host listener, so their failures are not isolation evidence). **Not overclaimed:** the in-ns `:3128` egress proxy
  exists and was **not probed** → record the leg as **"isolated net ns, no direct TCP reachability"**; egress policy untested.
- **Banking-gap instance two** (both o8 and o9): `done` + checker-passed but unbanked; only the maker may bank; nothing dispatches a banker
  when no open task remains. Both closed administratively; the **Option B package must cover both shapes** (o1 t5 rubric-final; o8/o9
  done-but-unbanked).
