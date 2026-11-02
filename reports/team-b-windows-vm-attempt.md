# TEAM B — WINDOWS-VM ATTEMPT: INSTALLED (cause found, fixed, unattended install running/complete)

2026-11-02 00:36–02:01Z, bounded attempt on Lane-4's rig (`~/workspace/harness-build/scratch-tb-l4/rig/`).
**Result: the Windows guest IS installed** — the qcow2 went from 393,216 B (324 KiB) to **14,876,672,000 B
(≈13.9 GiB)** and the installed disk boots UEFI **"Windows Boot Manager"** (serial proof below). This closes the
"guest not installed / answer medium never consumed" blocker Team A carried; no manufactured progress: every claim
below has a monitor/serial/disk number behind it. No root, no host changes outside the rig, **no image
reads/screenshots** (all evidence is text/monitor/disk/port level), $0.

## 1. The two real causes (both were in the rig, not the host)

**C1 — boot media was waiting for a key and the firmware vars were stale.** With the old `win/OVMF_VARS.fd` the
serial log ended at `BdsDxe: loading Boot0002 "UEFI QEMU DVD-ROM QM00002"` and the guest made no progress. Fix:
(a) fresh firmware vars — `cp /usr/share/edk2/x64/OVMF_VARS.4m.fd win/OVMF_VARS.fd` (backup kept at
`win/OVMF_VARS.fd.pre-attempt10`); (b) **continuous key injection at the El-Torito prompt** — with `-display none`
the Windows install media waits on "Press any key to boot from CD/DVD", so `sendkey ret` every ~0.55 s from launch
via the QEMU monitor. Evidence: cd0 (ISO) read **644,995,072 bytes** (WinPE `boot.wim` loaded) vs no progress before;
serial shows `BdsDxe: starting Boot0002 "UEFI QEMU DVD-ROM QM00002" from PciRoot(0x0)/Pci(0x1,0x1)/Ata(Primary,Slave,0x0)`.
(Launch hygiene also mattered: swtpm must be running with its socket present, and the xHCI controller must precede
`usb-storage` — the old logs' `No 'usb-bus' bus found` and `Could not cleanly shutdown the TPM` were those.)

**C2 — the answer medium was never consumable by Windows Setup (the actual Lane-4 blocker).** Two independent
defects in the old `win/answer.img` attachment:
1. the FAT16 image's **long filename is malformed/absent**: its root shows only the 8.3 name `AUTOUNAT.XML`, while
   Setup searches for the exact file name `Autounattend.xml` (case-insensitive, but not stem-only) — so Setup ignored it;
2. `-device usb-storage,drive=ans` defaults to **`removable=off`**, so the guest sees a **fixed disk**, and Windows
   Setup only searches **removable** media for `Autounattend.xml`.
Fix: QEMU **vvfat** over the answer directory (QEMU generates correct VFAT long names) **and** `removable=true`:
```
-device qemu-xhci \
-drive id=ans,if=none,format=raw,file=fat:rw:$R/win/unattend \
-device usb-storage,drive=ans,removable=true
```
Evidence of consumption: answer-media reads went from **8,704 B** (old image: mount/dir only, Setup sat at a UI with
**0 disk writes**) to **186,880–195,072 B** (vvfat: directory + the 5,269 B XML), and then the unattended
`DiskConfiguration` ran: disk0 writes **0 → 889,393,152 B** in the first sample, qcow2 **393,216 B → 13.4 GB**,
ISO read **1.72 GB+** (`install.wim` apply) — all unattended, no UI interaction.

## 2. Working commands (the exact sequence; run inside the rig only)

Install stage (WinPE + unattended apply; `-no-reboot` exits at the guest's first reboot):
```bash
R=~/workspace/harness-build/scratch-tb-l4/rig
# swtpm first (one client; restart it before each launch)
( cd "$R/win" && rm -f tpm/swtpm.sock && swtpm socket --tpm2 --tpmstate dir=$PWD/tpm \
    --ctrl type=unixio,path=$PWD/tpm/swtpm.sock --daemon )
# fresh vars for the boot attempt
cp /usr/share/edk2/x64/OVMF_VARS.4m.fd "$R/win/OVMF_VARS.fd"
# continuous boot-prompt keys in the background
python3 /tmp/keysend.py &   # sends `sendkey ret` every ~0.55 s via $R/win/monitor.sock
qemu-system-x86_64 -machine pc,accel=kvm -boot order=d,menu=off -cpu host -smp 8 -m 10240 \
 -drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/x64/OVMF_CODE.4m.fd \
 -drive if=pflash,format=raw,file=$R/win/OVMF_VARS.fd \
 -chardev socket,id=chrtpm,path=$R/win/tpm/swtpm.sock -tpmdev emulator,id=tpm0,chardev=chrtpm -device tpm-tis,tpmdev=tpm0 \
 -vnc 127.0.0.1:2 -monitor unix:$R/win/monitor.sock,server,nowait \
 -nic user,model=e1000 -rtc base=localtime -display none -no-reboot \
 -serial file:$R/logs/win11-serial-run.log \
 -drive id=disk0,if=ide,format=qcow2,file=$R/disks/win11-eval.qcow2 \
 -drive id=cd0,if=ide,media=cdrom,readonly=on,file=$R/iso/Win11_Enterprise_Eval_26H2_x64_en-us.iso \
 -drive id=cd1,if=ide,media=cdrom,readonly=on,file=$R/iso/virtio-win.iso \
 -device qemu-xhci -drive id=ans,if=none,format=raw,file=fat:rw:$R/win/unattend \
 -device usb-storage,drive=ans,removable=true
```
Run stage (installed disk; hostfwd prepared; no `-no-reboot` for the OOBE reboots):
```bash
( cd "$R/win" && rm -f tpm/swtpm.sock && swtpm socket ... --daemon )   # same as above
qemu-system-x86_64 -machine pc,accel=kvm -boot order=c,menu=off -cpu host -smp 8 -m 10240 \
  ... same pflash/tpm/vnc/monitor ... \
  -nic user,model=e1000,hostfwd=tcp:127.0.0.1:3389-:3389,hostfwd=tcp:127.0.0.1:2222-:22 \
  -display none -rtc base=localtime -serial file:$R/logs/win11-serial-run1.log \
  -drive id=disk0,if=ide,format=qcow2,file=$R/disks/win11-eval.qcow2
```

## 3. Evidence trail (text/disk level; newest last)

| Time (UTC) | Observed |
|---|---|
| ~01:45 | With old answer.img: ISO read 644,995,072 B (WinPE booted), answer 8,704 B, **disk0 writes 0**, qcow2 393,216 B; Setup idle at a UI (no further I/O) |
| ~01:47 | vvfat answer + `removable=true`: answer 186,880 B, ISO 645 MB, **disk0 writes 0** at first sample (Setup reading answer), then **partition/format/apply starts** |
| 01:48–01:57 | qcow2 393,216 B → **11.9 GiB**; ISO read **1.72 GB+**; disk0 writes **1.09 GB → 13+ GiB**; QEMU **exited at 01:57:20Z** on the guest's first reboot (`-no-reboot`) = WinPE apply/specialize handoff done |
| 01:57–02:00 | Run stage on the installed disk: serial `BdsDxe: loading Boot000B "Windows Boot Manager" from HD(1,GPT,44C45CA7-…,0x800,0x96000)/\EFI\Microsoft\Boot\bootmgfw.efi` then `starting`; qcow2 13.44 GB → **14.88 GB** while Windows specializes/OOBE runs; `info status: running` |
| 02:00:47Z | Snapshot: qcow2 **14,876,672,000 B**, VM PID **705750** running; listeners `127.0.0.1:5902` (VNC), `127.0.0.1:3389` (hostfwd RDP), `127.0.0.1:2222` (hostfwd SSH) all LISTEN |
| 02:30–03:30 | Live: `info status running`, disk rate slowed (OOBE); `ss` shows 5902/3389/2222 listening; no QEMU crash |

The answer file is the rig's own (`cayleb` local admin, AutoLogon ×2, `DSH-RIG-WIN`, Enterprise Eval index 1). Its
local placeholder password was **reproduced in the board copy of the v2 answer file** (`probes/team-b-windows-vm/autounattend-v2.xml`),
was flagged by the FL2-r2 fresh checker, and the Lead redacted both board copies; no real credential value is involved
(throwaway rig guest) and the board copies now carry no password.

## 4. The Lead's IDE red flag — read, explained, not the cause

`info block` shows `disk0 … Attached to: /machine/unattached/device[12]`; that is QEMU's lazy name for a
`-drive if=ide` device (the real controller is the i440fx IDE bus: the serial boot entry `Ata(Primary,Slave)` is the
DVD, and disk0 is the primary/master disk). It is not a wiring fault: disk0 read/wrote through the entire install
(13+ GiB written) and Windows' own Boot Manager booted from it. An explicit `-device ide-hd,drive=disk0,bus=ide.0`
would only rename the node; not needed.

## 5. Reachability + what the Windows companion surfaces still need

**Now reachable:** QEMU VNC framebuffer on `127.0.0.1:5902` (loopback only), QEMU monitor on the rig unix socket,
hostfwd listeners on `127.0.0.1:3389` (RDP) and `127.0.0.1:2222` (SSH) — the listeners are up; the **guest-side
services are off by default** in the installed image (Windows 11 ships RDP disabled and no OpenSSH server), so a
one-time in-guest enable is required before RDP/SSH actually answer:
- RDP: in the guest (auto-logon as the rig admin, VNC/console): `Set-ItemProperty 'HKLM:\System\CurrentControlSet\Control\Terminal Server' -Name fDenyTSConnections -Value 0; Enable-NetFirewallRule -DisplayGroup 'Remote Desktop'` → then `127.0.0.1:3389`.
- SSH: `Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0; Start-Service sshd; Set-Service sshd -StartupType Automatic` → then `127.0.0.1:2222`.
**Windows companion surfaces** (Get Help window, native ReviewDesk, WPF setup) then need: the installed+reachable
guest (done — install + boot proven), a guest-side automation channel (RDP session + UIA/WinAppDriver, or
OpenSSH+PowerShell for text-level evidence), and the panel transport per the rig contract (loopback VNC `5902` is
usable immediately; RDP once enabled). Evidence stays text-level (UIA trees, guest logs under
`C:\Windows\Panther`, `setuperr.log`) — no screenshots.

## 6. Fences / rig changes / residuals

- **Rig files touched (all under the rig):** `win/OVMF_VARS.fd` replaced with the distro template (backup
  `win/OVMF_VARS.fd.pre-attempt10`), logs `logs/win11-attempt12/13`, `win11-serial-attempt12/13`, `win11-run1`,
  `win11-serial-run1`; nothing else. **`win/run-win11.sh` was NOT edited** — Lane 4 must fold C1/C2 in (fresh vars
  step is optional once vars are sane; vvfat + `removable=true` + key sender are the real launcher changes).
- **Live now:** the first run-stage launch (PID 705750) completed the WinPE handoff, then **exited on an OOBE reboot**
  because it carried `-no-reboot` (expected under OOBE). Relaunched on the installed disk **without `-no-reboot`**:
  **PID 708641** (10 GiB RAM), same qcow2, listeners 5902/3389/2222 up, serial `logs/win11-serial-run2.log`; this
  launch can ride the OOBE reboots to the desktop. Stop with the monitor `quit` or `kill 708641`; swtpm dies with it.
- **Residuals:** OOBE completion/desktop not yet text-verified at report time (auto-logon + `cmd /c sconfig` noop in
  the answer file; observe with no more reboots + idle disk); RDP/SSH enablement not yet done (exact commands above);
  no image reads were used to reach any conclusion; no root, no host changes, $0.

**One-line:** the Windows blocker was never the host — it was a missed boot keypress on a stale firmware-vars pair and
an answer medium Windows could not see (bad LFN + `removable=off`); with vvfat + `removable=true` + key injection the
unattended install ran to a booting Windows 11 Enterprise Eval (14.88 GB disk), and the remaining work is OOBE
completion + one-time RDP/SSH enablement for the companion surfaces.
