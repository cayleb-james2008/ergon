# SSH SETTLED PROBE #2 — RDP live, SSH still silent after ~2 h settle (Lead-run; fleet-lead seat failed before filing)

2026-11-02 01:31 ET, read-only, no reinstall/restart/blind input; guest and both disks preserved. Closed the "settle then re-probe"
hypothesis from the first settled probe (03:37–03:38Z): the guest has now been up and idle for hours and **SSH still does not answer**.

## Probe evidence (commands + outputs, 01:31 ET)
- Listeners (QEMU hostfwd, unchanged): `127.0.0.1:3389` (RDP), `127.0.0.1:2222` (SSH hostfwd), `127.0.0.1:5902` (VNC),
  `0.0.0.0:8765` (marker listener) — all LISTEN; guest QEMU process running.
- **RDP re-confirmed, protocol level:** X.224 handshake →
  `03 00 00 13 0e d0 00 00 12 34 00 02 2f 08 00 02` = TPKT + **Connection Confirm (0xd0)** + RDP negotiation.
- **SSH still silent:** `timeout 8 socat - TCP:127.0.0.1:2222,connect-timeout=4` → **0 bytes** (no banner); key round-trip
  `ssh -p 2222 -i win/agent_key -o BatchMode=yes cayleb@127.0.0.1 'hostname & ver'` → `Connection timed out during banner exchange`
  (rc 255). The QEMU hostfwd accepts the TCP connection, the guest does not serve SSH.
- Guest/disk state: `disks/win11-eval-2.qcow2` **31,230,590,976 B** (grew from ~20.9 GB — Windows servicing/updates still writing),
  fallback `disks/win11-eval.qcow2` **17,083,138,048 B** untouched (mtime Nov 1 21:13); marker log has the earlier `?step=start`
  (03:08:35Z) and `?step=done` (03:26:46Z).

## Interpretation (honest)
- The FirstLogonCommands completed (`?step=done`), so the OpenSSH capability command returned — but **no serving sshd is reachable**.
  The "still servicing, settle then re-probe" hypothesis is now **disproven**: two hours of settle changed nothing.
- Guest-side cause remains unverified because there is no text channel into the guest (no agent, Windows writes nothing to serial,
  VNC is video-only). Candidate causes (unverified): `Add-WindowsCapability` silently failed (servicing/Windows Update state), sshd
  not started/auto-start not set, firewall rule absent, or the admin key path not accepted for an Administrators account.
- **Final state for the morning report: Windows = installed + OOBE first logon + RDP re-confirmed (driveable via RDP);
  SSH/OpenSSH = unavailable with this exact blocker.** No reinstall/restart/blind input without a fresh Nox filing; a status-reporting
  reinstall is the deterministic close but is not needed for the driveable-surface claim.

## Fence / seat note
Read-only probe run by the Lead after the `fleet-lead` seat failed mid-task with no closing message (the task was a read-only probe;
no state was changed by that failure). Guest left running; no credential values; no image reads; $0.
