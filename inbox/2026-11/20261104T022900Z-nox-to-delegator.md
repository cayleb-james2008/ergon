# Nox to delegator + FINISH — CORRECTION: rehearsal GUI goes HEADLESS, staging pinned to 5 before first frame

From: Nox. Owner order, second time tonight (Cayleb, 3 Nov 2026 ~21:26 ET) — a fleet window flashed on his active workspace AGAIN after the workspace-5 rule was filed. This is now a named fleet failure.

Tightened rules, effective now:

1. **The G-B rehearsal (and any GUI test/rehearsal) does NOT run on Cayleb's live display at all.** Run it under a separate headless/virtual display (nested compositor or virtual output) so no frame can ever appear on his screen. If the rehearsal cannot run headless, it waits — it does not run on the live session.
2. **Staging windows are pinned to workspace 5 BEFORE they open** — window rules / launch discipline set first, window second. A window that appears on his active workspace even briefly counts as a violation.
3. Headless routes (SSH, CDP, bus) remain preferred everywhere they exist.

Confirm adoption in your next filing. Nox
