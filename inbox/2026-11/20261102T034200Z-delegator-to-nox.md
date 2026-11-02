from: delegator
to: nox
kind: WINDOWS SETTLED PROBE — RDP re-confirmed 6/6 (wording upgrade condition (b) MET); SSH still pending with exact blocker; wake-up refresh #3 DONE; FL2 stays NOT met; FogEnabled fix committed awaiting Team D's freeze report for the delta
re: 033600Z §4 (settled probe condition)
---

## 1. SETTLED PROBE — RDP RE-CONFIRMED 6/6; WINDOW WORDING CONDITION (b) MET
- **RDP re-confirmed 6/6** clean back-to-back probes at **03:37:36 / :46 / :56, 03:38:06 / :16 / :26Z** → each 19-byte reply
  `030000130ed00000123400022f080002000000` = TPKT + X.224 **Connection Confirm** + RDP negotiation. Combined with the checker's
  spot-verify ACCEPT of r2 (condition (a)), the morning wording is now allowed to say **"Windows: installed + OOBE first logon +
  RDP re-confirmed — driveable via RDP"**.
- **SSH still pending, exact blocker filed:** TCP to `127.0.0.1:2222` connects (QEMU hostfwd → guest accepts) but **0 bytes of SSH
  banner in 6/6 probes** (03:37:36–03:38:26Z) and the key round-trip at 03:38:40Z timed out during banner exchange. The `?step=done`
  marker fired 03:26:46Z (so the OpenSSH capability command returned), but **no serving sshd**. Guest-side causes unverified (no text
  channel): capability truly installed? sshd running/auto-start? firewall rule? admin key path? The guest was still servicing
  (CPU ≈300 %, disk growing) — "settle then re-probe" is a hypothesis, not a pass.
- **Compliance:** no reinstall, no restart, no further blind input (in-flight sender stopped 03:37:26Z, not repeated, no guest-visible
  effect); **QEMU PID 757632 and both disks preserved** (qcow2 20,948,844,544 B; qcow1 17,083,138,048 B untouched); listeners 5902 VNC /
  3389 RDP answering / 2222 hostfwd no SSH banner / 8765 marker.
- **Disposition proposal (no action taken):** SSH is **not required** for the driveable-surface claim — RDP is the driveable channel and
  the companion automation path (UIA/WinAppDriver or PowerShell over RDP) does not need sshd. I propose carrying SSH as a named pending
  with the exact blocker above; if you want it closed, the two honest routes are (i) periodic post-settle re-probes (read-only, no
  approval needed — Team B can keep watching) and/or (ii) a future reinstall whose answer file reports guest status to the proven marker
  channel (needs your filing approval; not needed for FL2). **Your call — I will not run a reinstall without your word.**
- **Team B corrections filed:** fence (the rig-local placeholder password WAS present in the board copy of the answer file; both board
  copies redacted; rig answer files rotated to a new unnamed placeholder; report states exactly that) and companion wording (no "met"
  claim; companions package-blocked). FL2 r2 checker ACCEPT + r3 corrections stand; **FL2 remains NOT met** (companions package-blocked).

## 2. WAKE-UP REPORT REFRESH #3 — DONE
`reports/WAKE-UP-REPORT-20261102.md` refreshed again: Windows section now states installed + OOBE first logon + **RDP re-confirmed
(driveable via RDP)** with **SSH pending + exact blocker**; Roblox section carries named-seed RESOLVED (assert-and-skip), FogEnabled fix
in flight, fixture A+C ruled and sequenced; FL2 line unchanged "NOT met (companions package-blocked)"; only two owner items (key entry,
v2 grade). It is ready for the 08:00–09:30 ET consolidated report; I will refresh once more if anything material lands before then.

## 3. ROBLOX — FOG FIX COMMITTED, FREEZE REPORT PENDING
Team D committed the ratified FogEnabled fix (`a231e3f`: guarded legacy fog reads/writes — capability() pcall + nil-is-absent,
readFog() records availability/absence, Server:95 tiles-fogoff guarded, no unconditional `Lighting.FogEnabled`; Studio 0.745 removed
the property). Awaiting its freeze report (suites + sealed + repro block) to launch the fresh checker delta; then the fresh 3-Play
opening + one atlas Play, then the ruled A+C fixture work, per your sequence.

No spend, no image reads, no publication, desks untouched.

— delegator
