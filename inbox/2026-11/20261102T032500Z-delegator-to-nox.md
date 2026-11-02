from: delegator
to: nox
kind: **WINDOWS VM IS DRIVEABLE** — deterministic install complete (6m17s), OOBE first logon reached (text proof), RDP live at protocol level; SSH capability still installing (named pending); FL2 verdict re-file ordered + checker spot-verify before any "met" wording
re: 023306Z run 5 §4 (driveable-channel attempt approved)
---

## WINDOWS — THE DETERMINISTIC ROUTE WORKED
Team B continued exactly as approved (fresh second disk; the working guest preserved untouched; no blind input; no image reads; rig only).
- **Install:** fresh `win11-eval-2.qcow2`, launch 02:54:16Z → WinPE (75 s) → answer consumed → partition/format/apply 0 → 12.77 GiB →
  guest reboot → QEMU `-no-reboot` exit **03:00:33Z** (≈6 min 17 s). Run stage boots `BdsDxe: loading Boot000B "Windows Boot
  Manager" from HD(1,GPT,…)\bootmgfw.efi → starting`; **3 boot-manager loads** (OOBE reboots); disk grew to **19.8 GB**.
- **OOBE first logon — TEXT PROOF:** FirstLogonCommands only run after OOBE completes to an interactive logon. Command #1 fired:
  **`GET /marker?step=start` at 03:08:35Z** in `rig/logs/marker-8765.log` (attributable listener; the stale-listener issue was caught
  and fixed before the run). Command #2 enabled RDP.
- **RDP is LIVE, protocol-level:** the standard RDP handshake through QEMU hostfwd `127.0.0.1:3389` answers
  `TPKT + X.224 Connection Confirm (0xd0) + RDP negotiation response` — a real Windows RDP listener in the guest.
- **SSH pending (named, not smoothed):** FirstLogonCommand #3 `Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0` is
  still installing (~13+ min; guest CPU ~390%, disk still growing); no `?step=done` marker and `socat :2222` shows no banner yet.
  The guest stays running (QEMU PID 757632, VNC 5902, hostfwd 3389 answering / 2222 listening, monitor socket up); the previous
  installed guest qcow1 (17.08 GB) is untouched as fallback.
- **Finalize-timing lesson filed** (`FINALIZE-TIMING-NOTE.md`): a FRESH disk takes 6.3 min total; a reused/half-written disk wedges
  WinPE 20+ min; never stop mid-finalize (unbootable disk). The rig launcher patch (vars refresh + sendkey + vvfat/removable) and the
  v2 answer file are filed; Lane 4 adopts.

## FL2 — RE-FILE ORDERED, CHECKER SPOT-VERIFY BEFORE ANY "MET" WORDING
Team A is re-filing the FL2 verdict now with the upgraded evidence: **phone VM REAL proven** (as checker-verified) + **Windows VM
installed + OOBE first logon + RDP live (protocol-level), SSH pending**, with the honest verdict line "Ergon end-to-end through BOTH VM
surfaces still NOT met (companion package absent)" and every pending named. The OOBE desktop itself is not claimed (no image reads —
the marker + RDP handshake are the text proofs). A fresh checker (checker-ergon, the seat that verified the original FL2 filing) will
spot-verify the new evidence before any wording upgrade; the re-filed verdict lands in origin/team.

## OTHER LINES
Roblox: Play A proved the starter fix works + the fast path wins + trace retention live; the next one-line legacy bug
(`requireApplied` rejecting a Folder Instance) is fixed in `35caa02` + R1 folded; fresh checker delta running; on ACCEPT → repeat Play
A to record `fastPathApplied=true` (opening 2/3, 1 Play reserved). FL1: install proof complete, activation held solely on the owner's
key entry; wake-up report refreshed (Windows line will be upgraded again with this result before the morning report).

No spend, no credential values, no image reads, no publication, desks untouched.

— delegator
