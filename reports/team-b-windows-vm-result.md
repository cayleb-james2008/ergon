# TEAM B — WINDOWS-VM RESULT (installed + booting; OOBE/channel status; launcher patch)

2026-11-02 02:00–02:12Z, follow-up to `board/reports/team-b-windows-vm-attempt.md`. **Result: Windows 11 Enterprise
Eval is INSTALLED on the rig qcow2 and boots via UEFI "Windows Boot Manager".** The guest is **left RUNNING** (see §3).
No image reads, no root, no host changes outside the rig, $0.

## 1. Install proof (text/disk level)

| Fact | Value |
|---|---|
| qcow2 | `rig/disks/win11-eval.qcow2`: **393,216 B → 17,078,550,528 B (≈15.9 GiB actual, 80 GiB virtual)**; was 324 KiB actual / unchanged mtime before the attempt |
| Install evidence | unattended answer consumed (answer-media reads 186,880–195,072 B; vvfat + `removable=true`), disk partitioned/formatted, image applied: disk0 writes 0 → 889 MB → 13+ GiB; ISO read 1.72 GB+ |
| First reboot | QEMU exited 01:57:20Z on the guest's first reboot (`-no-reboot` in the install stage) = WinPE apply/specialize handoff complete |
| Disk boot | serial (`logs/win11-serial-run2.log`): `BdsDxe: loading Boot000B "Windows Boot Manager" from HD(1,GPT,44C45CA7-14C4-4967-8BDC-D8D586A48CB6,0x800,0x96000)/\EFI\Microsoft\Boot\bootmgfw.efi`, then `starting` — **the installed OS boots from disk0** |
| OOBE reboots observed | **2 boot-manager loads** in run2 (OOBE/specialize reboots rode through; run stage has no `-no-reboot`) |
| Root causes fixed (details in the attempt report) | F1 stale/foreign OVMF vars + missed El-Torito "press any key" (`sendkey ret`) — ISO read went 0 → 645 MB (WinPE); F2 answer media 8.3-only FAT16 (`AUTOUNAT.XML`, not `Autounattend.xml`) + `usb-storage` default `removable=off` — fixed with vvfat over `win/unattend/` + `removable=true` |

## 2. Current run state (live at 02:12Z)

| Item | Value |
|---|---|
| QEMU | **PID 708641** (run stage on the installed disk, `-boot order=c`), **RSS ≈ 10.3 GiB**, host load accepts it (62 GiB total) |
| Transports | VNC `127.0.0.1:5902` LISTEN; QEMU monitor unix socket `rig/win/monitor.sock`; hostfwd `127.0.0.1:3389` (RDP) and `127.0.0.1:2222` (SSH) LISTEN (QEMU side; guest services still off — see §4) |
| Serial | `rig/logs/win11-serial-run2.log` (2 × Boot Manager + OVMF noise only; Windows does not write serial) |
| OOBE status | **not verified to a desktop.** Disk still being written (17.08 GiB at 02:11Z, slow growth = first-logon servicing), 2 boots, no crash. There is **no text channel** into the guest (no QEMU guest agent installed; Windows writes nothing to serial), so "desktop reached" cannot be claimed from this seat — see §5. |

## 3. Guest left RUNNING (explicit)

Left up so the surface is inspectable and the OOBE can finish: **QEMU PID 708641**, qcow2 in use, ~10.3 GiB RSS.
Stop cleanly with the monitor `quit` command on `rig/win/monitor.sock` (or `kill 708641`); swtpm exits with it. No
reboot is needed to resume: relaunch the `run` stage as in the patched launcher.

## 4. Guest-service enablement (RDP/OpenSSH) — NAMED PENDING, no honest channel yet

Attempted bounded, without image reads: blind keyboard injection via the QEMU monitor (Win+R + base64-encoded
PowerShell to enable `fDenyTSConnections=0`, the Remote Desktop firewall group, and the OpenSSH server capability),
**twice**, with a host-visible HTTP marker (`http://10.0.2.2:8765/marker`) as the success signal for "the guest shell
ran". Result: **no guest request reached the host marker, and neither `127.0.0.1:3389` (RDP X.224) nor
`127.0.0.1:2222` (SSH banner) answers at protocol level** — the hostfwd listeners are QEMU's, the guest services are
not enabled. Conclusion: keyboard injection cannot be verified while the OOBE screen is unknown, so the service
enablement is **carried as a named pending with the exact missing step**:
- confirm the OOBE/desktop with a visual seat (this is the owner's screenshot, not ours), **or**
- for the next install, bake enablement into the answer file's `oobeSystem` → `FirstLogonCommands` (RDP registry +
  firewall group + `Add-WindowsCapability OpenSSH.Server`, then `Start-Service`), so the guest is driveable with no
  blind input at all. **Recommended** — it makes the surface deterministic and removes the whole channel problem.
Then verify from the host: RDP X.224 handshake on 3389 and `SSH-2.0-OpenSSH_for_Windows_*` banner on 2222, followed by
a text command round-trip (`ssh`/RDP client) — the usable channel for the companion surfaces.

## 5. What the Ergon Windows companion surfaces still need

1. **Guest at a verified desktop** (OOBE complete; auto-logon account exists in the answer file) — needs the visual
   confirmation or the FirstLogonCommands re-run above.
2. **A guest automation channel**: RDP (then UIA/WinAppDriver against the WPF ReviewDesk / Get Help / setup) or
   OpenSSH + PowerShell for text-level evidence; both are one-time in-guest enablements (commands above), RDP being
   the natural desktop transport.
3. **The companion artefacts themselves** — the Ergon Windows package (Get Help window, native ReviewDesk, WPF setup)
   does not exist yet (G1 whole; PACKAGE §5), so even a perfect channel has nothing to drive.
4. **Panel transport** per the rig contract: loopback VNC 5902 is live today; RDP 3389 once enabled.
Evidence discipline: UIA trees / logs / text command output, never screenshots.

## 6. Launcher patch (so the next run cannot reproduce the blocker) — filed, `run-win11.sh` NOT edited in place

`board/probes/team-b-windows-vm/` contains `run-win11.sh.orig`, **`run-win11.sh.patched`** and
**`LAUNCHER-PATCH.diff`** (82-line diff). The patched launcher: refreshes OVMF vars on `install --refresh-vars`, runs
the `sendkey ret` keypress loop for the El-Torito prompt, attaches the answer medium as **vvfat over
`win/unattend/`** with **`usb-storage,removable=true`**, adds serial logs, and for `run` adds the RDP/SSH hostfwd and
drops `-no-reboot`. Lane 4 owns adoption; strike-release untouched.

## 7. Fences / changes

- Rig files touched: `win/OVMF_VARS.fd` (distro template; backup `win/OVMF_VARS.fd.pre-attempt10`), logs
  `win11-attempt12/13`, `win11-serial-attempt12/13`, `win11-run1/2`, `win11-serial-run1/2`; `/tmp` helper scripts;
  host marker HTTP server on `127.0.0.1:8765` (temporary, can be stopped). `win/run-win11.sh` left untouched.
- No image reads/screenshots were used; no root; no host changes outside the rig; $0.
