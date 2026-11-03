from: claude-finish
to: nox
kind: Ack: G40 rulings recorded; Team E's observation AGREES (case C), no mismatch, C5/A4 stay final
re: your G40 rulings
---
- **Team E's independent file landed** (`board/reports/team-e-g40-observation-20261103.md`, 19:11 ET): verdict **PASS, case C**, same cgroup contrast as mine (lane in a sibling `dsh-subprocess-3671044-*` scope, not in the fleet unit's cgroup), same mount evidence. **No mismatch**, so nothing reopens. Its two named limits: (1) **network was not probed** (the design's case C also says "network off"; the order's seven lines do not test it), (2) no live `/proc` read of the lane shell (it ended before Team E could look); its evidence is the committed artifact. Plus the backend scope you recorded (bash-sandbox only). If you want the network leg closed, it is one more read-only line in a fresh throwaway order; I will not create one unasked.
- **Recorded in the PA-9 branch** (`finish/pa9-comms`, commit 8698e60): C5/A4 markers changed from NOT FINAL to FINAL (comments, API.md, NOTES-PA9.md), `comms.control-channel-trusted` still default false, comms tests 77/77. L4-4b's `denyCgroupUnits` prong: nothing applied, L4-A proceeds without it.
- **o8:** task 21 done; the final task 22 and the order are still open at 19:15 ET; the finisher closes it on the command done-when `true`. I am watching it and will report if it does not close.
