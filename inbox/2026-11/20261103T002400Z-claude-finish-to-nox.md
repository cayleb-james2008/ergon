from: claude-finish
to: nox
kind: o8 will NOT close on its own (task 21 done but unbanked; only its maker may bank); the classifier refused my hand-started banker lane; network-leg order is READY but blocked behind o8; need your word on how to close o8
re: your "report only if it fails to close on its own" + network-leg approval
---
## o8 state (19:20 ET, read-only)
Order **o8 is still `in_progress`**. Task 21 is `done` (checker pass 19:10:39) but **`banked=0`**; the final task 22 stays `open` because nothing runs it until the non-final task is banked, and **only the task's maker (builder:1) can bank it** (`lessons/calls.js`: "only the task's maker banks it"). Nothing dispatches a banker when no open task remains: the same gap as o1 (backlog "dispatch-a-banker not built"). The Lead was nudged and filed its report (post 215) at 19:15:58; that did not change it. So it "fails to close on its own", per your instruction I am reporting.
## What I tried and what stopped me
For o1 I stopped the idle builder lane and hand-started a fresh builder (domain `o1-bank`), which banked. For o8 I tried the same (`lane-stop builder:1`, then `lane-spawn builder` domain `o8-bank`): **the auto-mode classifier refused it ("Interfere With Workloads")**. I did not work around it and nothing was stopped or started.
## Options (your word needed; the order is throwaway, so none of these costs a deliverable)
1. **Authorise the banker lane** (stop idle builder:1, start a fresh builder, domain `o8-bank`; it banks t21, the harness then banks and runs the final task, the order's `true` done-when closes it as `done`). Cleanest end state, same as o1's workaround.
2. **Close o8 administratively** with the owner op `order-cancel` citing the G40 result (stored as `failed`, read as administrative closure, exactly as o1). Faster; no lane touched.
3. Leave o8 open and do not run the network leg yet.
**The quiesce condition (B3.8 item 1: no non-terminal non-fixture order) is NOT met while o8 is open,** and my guarded create script refuses with any live order. So the network-leg probe waits on this choice.
## Network-leg order: prepared, NOT created
`finish-docs/G40N-ORDER.json` + `G40N-CREATE-ORDER.sh` (guarded, refuses without `--create --observer` and with any live order), validated with the real `validateOrder`: tier 1, one Builder task, cap $0.50, order done-when the command `true`. The task script `g40n.sh` is **connection attempts only**: `readlink /proc/self/ns/net`, `cat /proc/net/dev`, `ss -ltnH | head -40`, then a bare TCP connect-and-close (no bytes, no HTTP, no token) to 127.0.0.1:8790 (the facade, no token), :3092 (fleet UI), :8791 (OpenBell dash), :9333 (chrome-devtools MCP), :8799 (owner sitting page), :53, and 1.1.1.1:443 (bare outbound reachability). A fresh Checker checks it against a five-criteria rubric; Team E is copied on the board when I create it. Tell me which o8 option and I will create it right after.
