from: delegator
to: nox
kind: Roblox Play A — starter fix WORKS in Play, fast path WINS, trace proven live; next one-line legacy bug found + fix approved (repeat Play A planned); Windows driveable verification in progress
re: 023306Z run 5; 024500Z
---

## 1. ROBLOX — PLAY A (fresh 3-Play opening): starter fix works in Play; fast path wins; next legacy bug is one line
- **Opening:** `studiocycle cycle` (close-discard/reopen/fit) → sealed place `c6651793…` unchanged through close+open; new Studio
  pid 759088, window 0x558c7fa45500, Edit, **0/3 Plays**; preflight rc 0, place==build==`c6651793…`.
- **Play A** (`run atlas:scout --seed 2134783819`, one Play, 23 s — the 45 s bound was never hit): exit 1 fail-closed, collector
  phases prepared/runtime/cleanup, rejections [], sealed BEFORE/AFTER unchanged (`84324d43…`/`c6651793…`), `verify-clean` clean,
  plays **1/3**.
- **What it proves (the fix works):** the starter now requires the modules and calls `apply` on the module table; the
  `World.dailySeed` patch **wins the ordering race**; the trace is retained on all three paths (`result.json.worldTrace` ==
  `world-trace.json` == runtime `report.worldTrace`, starter trace present) — **C1+C2 proven IN PLAY**, not just in tests.
  Trace: `rootFoundAt=24.889, worldReadyAt=30.231, actualSeed=2134783819, actualLayout=2, expected 2134783819/2, rebuild=false,
  starter={applied:2134783819}` — identity evidence is conclusive that the fast path won (the `fastPathApplied` field itself is the
  pre-check default `false`; a clean `true` needs the next fix + one repeat Play).
- **New failure (exact, one line):** runtime arrived before view-001 → `AtlasWorldSeed:66: the world root is required` —
  `requireApplied`'s legacy `type(root)=="table"` assert rejects the Roblox **Folder Instance** (`userdata`); the same assert
  existed in the old `ensure` and was masked by the starter crash. **Fix approved** (offline): Instance-compatible root check
  (`root ~= nil` + Instance-level assertion; named fail-fast + seed/layout checks unchanged), a Lune spec with a REAL
  `@lune/roblox` Instance root (Folder accepted; nil/wrong object still fails closed), plus R1 (the two dropped C1 ordering
  assertions, test-only). Then freeze → checker delta → **repeat Play A once** (opening 1/3 → 2/3, 3rd reserved) to record
  `fastPathApplied=true`. **Play B (abort-contact) stays gated behind the fixture-geometry ruling.**
- The superseded "rebuild hang" reading stays withdrawn and is recorded; Plan 4's assert-and-skip is correct and now reachable.

## 2. WINDOWS VM — DRIVEABLE VERIFICATION IN PROGRESS
Install #2b finished finalize and the run stage is up (QEMU PID 757632; VNC 5902, hostfwd RDP 3389 / SSH 2222; marker listener on
8765). AutoLogon/FirstLogonCommands should fire on this boot; verification is text-level (marker `step=start`/`step=done`, RDP
X.224, `ssh -p 2222 -i rig/win/agent_key <admin>@127.0.0.1 'ver'`). No marker hit or SSH banner yet at the time of this filing —
the guest may still be in OOBE/first-logon servicing. On driveable, the FL2 verdict is re-filed with the Windows leg upgraded and a
fresh checker spot-verifies before any "met" claim; on failure the exact blocker is filed. OOBE desktop stays unclaimed either way
(no image reads).

## 3. REPORT REFRESH
The wake-up report was refreshed at 02:42Z per your run 5; the Roblox section will be updated with Play A's result (starter fix
works; fast path wins; next one-line fix; plays accounting) before the morning consolidated report. Native residuals unchanged.

No spend, no credential values, no image reads, no publication, desks untouched.

— delegator
