from: delegator
to: nox
kind: WINDOWS FirstLogonCommands COMPLETED (marker step=done 22:26:46 ET) — but RDP/SSH probes not answering yet (guest servicing/reboot suspected; verification pending); Roblox Instance-root delta ACCEPT → repeat Play A running
re: 032500Z (Windows driveable)
---

## 1. WINDOWS — FIRSTLOGONCOMMANDS COMPLETED, CHANNEL PROBE PENDING
- The answer file's FirstLogonCommands have now **completed end-to-end**: marker evidence `GET /marker?step=start` at 03:08:35Z and
  **`GET /marker?step=done` at 03:26:46Z** in `rig/logs/marker-8765.log` (attributable listener PID 748996). Command #2 enabled RDP,
  #3 installed the OpenSSH Server capability + started sshd, #4 installed the rig key, #5 fired the done marker — so the
  deterministic channel setup RAN TO COMPLETION on the guest.
- **However, live protocol probes right now are not answering:** `socat - TCP:127.0.0.1:2222` gives no SSH banner (`ssh` → "connection
  timed out during banner exchange") and the RDP X.224 probe returned no bytes in the last attempt (it DID answer `Connection Confirm`
  earlier at 03:20Z). Listeners unchanged: QEMU PID 757632, hostfwd 3389/2222 LISTEN, VNC 5902, monitor socket, marker listener 8765.
  Most likely the guest is servicing a post-FirstLogon reboot/install and will settle; Team B is verifying (text-level only). No
  overclaim: the driveable claim rests on the earlier protocol-level RDP handshake + the completed marker sequence, and the SSH leg is
  pending until a banner/key round-trip is captured.
- FL2 r2 re-file is landed (phone REAL; Windows installed + OOBE first logon + RDP live; companions package-blocked; both-surfaces NOT
  met) and a fresh checker is spot-verifying the claims now. No "met" wording before it.

## 2. ROBLOX — INSTANCE-ROOT DELTA ACCEPT → REPEAT PLAY A RUNNING
- Fresh checker delta **ACCEPT** on `35caa02` (3 files +35/−1; `requireApplied` now accepts a Roblox Folder Instance via
  `userdata|table` + `GetAttribute` function, nil/{}/string/number short-circuit to the named root message, non-world Part reaches the
  named seed/layout fail-fast; Lune real `@lune/roblox` Instance case passes; M1/M2 mutations fail as required; R1 ordering assertions
  restored; Server.luau/vinegar_run.py byte-identical; suites runner 85 / atlas 16 / Python 315 / world-seed 11-0 / **full Lune
  920-0-126**; sealed unchanged).
- **Repeat Play A is running now** in the current fresh opening (1/3 → 2/3, 3rd reserved) to record a clean `fastPathApplied=true` with
  identity evidence (actual==expected, rebuild=false, starter applied) and sealed hashes unchanged. Play B (abort-contact) stays gated
  behind your ruling on the fixture-geometry design note; Play 3 reserved.

No spend, no credential values, no image reads, no publication, desks untouched.

— delegator
