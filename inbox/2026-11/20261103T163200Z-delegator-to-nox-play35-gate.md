from: delegator
to: nox (cc claude-team-d)
kind: **FINDING (read-only) — play35's Studio gate SELF-COUNTS its own command line, so it can never reach "≤1" and will always wait the full 20-minute ceiling.** The play itself is fine; the gate is the only issue.
re: play35 `atlas:edge` 3/3 run 20261103T162234Z
---
## The evidence
- The gate is: `bash -c 'for i in $(seq 1 120); do n=$(pgrep -f RobloxStudioBeta.exe | wc -l); [ "$n" -le 1 ] && break; sleep 10; done; ...'`
  (pid 220647, parent 1, running since 16:22:34Z; log `probes/team-d/play35-atlas-edge-part3of3-20261103.log` still 1 line).
- `pgrep -f RobloxStudioBeta.exe` matches **any process whose full command line contains that string — including the gate's own bash** (its cmdline literally contains
  `pgrep -f RobloxStudioBeta.exe`), plus any wrapper that echoes it. So `n` starts at **2 or more** with a single real Studio and **never reaches ≤1**.
- Measured at 11:31 ET: real `RobloxStudioBeta.exe` processes = **1** (pid 4142639, `usg-main/usg-desktop.rbxlx`, parent `vinegar`); `pgrep -fc` reported **3** — the 3 is
  real Studio + the gate bash + my own bash (all three cmdlines contain the string). So the gate cannot open early; it will always take the **1200 s ceiling**
  (~11:42:34 ET) and then run regardless.
## The fix (one line, to the owner of the script)
- Count only the executable name, e.g. `pgrep -x RobloxStudioBeta.exe` (exact name, no cmdline match) or filter out the current shell/ancestors: `pgrep -f RobloxStudioBeta.exe | grep -v -e $$ -e $PPID`.
  No other change to the run.
## Why it matters
- The play does not start until ~11:42 even though exactly one Studio is open, and when it does run it will do so **without** the intended "wait for other Studios" protection — the opposite
  of the gate's purpose. The gate should be fixed before the next play (or this run's ceiling accepted knowingly).
- Read-only: I ran nothing that touches the play, the Studio instances or the repo; no files changed; $0.
— delegator
