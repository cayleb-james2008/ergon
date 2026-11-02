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

---

# ADDENDUM — deterministic second-install pass (2026-11-02 02:13–02:38Z): NOT DRIVEABLE IN THIS PASS (named blocker)

Bounded pass per Nox/Lead order (fresh second disk + enriched answer file + patched launcher mechanics). **Result:
the install was launched and consumed the enriched answer, but did not reach OOBE/FirstLogonCommands inside the
time-box, so no marker/SSH/RDP evidence exists yet.** The previous installed guest (qcow1) is preserved untouched as
the fallback evidence. No image reads, no root, no host changes outside the rig, $0.

## What was built (ready for the next run)

- **Second disk:** `rig/disks/win11-eval-2.qcow2` (80 GiB sparse; qcow1 untouched at `rig/disks/win11-eval.qcow2`,
  17,083,138,048 B).
- **Answer v2:** `rig/win/unattend-v2/autounattend.xml` (copy filed at
  `board/probes/team-b-windows-vm/autounattend-v2.xml`): same unattended install as v1 **plus** five
  `FirstLogonCommands` — (1) host marker `http://10.0.2.2:8765/marker?step=start`, (2) enable RDP
  (`fDenyTSConnections=0`, TermService auto, Remote Desktop firewall group), (3) enable OpenSSH Server
  (`Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0; Start-Service sshd`), (4) install the rig's SSH
  public key as the admin `authorized_keys` (`C:\ProgramData\ssh\administrators_authorized_keys`, ACLs restricted),
  (5) marker `?step=done`. AutoLogon/account/OOBE settings unchanged from v1 (so OOBE should land on a desktop).
- **Rig SSH key:** `rig/win/agent_key` (private, never leaves the rig) + `agent_key.pub` (filed in the probe dir);
  intended verification was `ssh -p 2222 -i rig/win/agent_key -o StrictHostKeyChecking=no cayleb@127.0.0.1 ver`.
- **Launch mechanics:** patched-launcher sequence (fresh OVMF vars, continuous `sendkey ret`, vvfat answer over
  `win/unattend-v2/` + `usb-storage,removable=true`, `-boot order=d`, monitor+serial+VNC). A tiny host marker
  listener on `0.0.0.0:8765` ran throughout.

## What happened (evidence)

- WinPE booted (serial `BdsDxe: starting Boot0002 "UEFI QEMU DVD-ROM"`), answer consumed: **answer reads
  188,928 B**, **disk0 writes 12,648,665,088 B**, **ISO reads 8,593,821,696 B**, qcow2 0 → **12,477,792,256 B
  (11.9 GiB)**.
- Then the guest stayed in WinPE's CPU-bound finalize phase: qcow2 flat for **>8 min**, QEMU 148–250 % CPU,
  `info blockstats` idle on disk0/cd0/ans, **no reboot/exit**, serial unchanged (only the one Boot0002 start), so
  OOBE/FirstLogonCommands never ran: **marker hits = 1 (host's own curl only), no RDP X.224 answer on 3389, no SSH
  banner on 2222**.
- **Honest note:** an earlier run of this same install was stopped by me during this CPU-bound phase (monitor
  `quit`); booting that disk then gave `BdsDxe: failed to load Boot0001 "UEFI QEMU HARDDISK" … Not Found` — proof
  that the finalize phase must be allowed to complete before the disk is bootable. Do not stop it early.
- **Left running:** install #2b QEMU **PID 724993** (~9.7 GiB RSS) is still in that finalize phase, so a later seat
  can let it finish; the marker listener on `127.0.0.1/0.0.0.0:8765` is still up for the FirstLogon hits. The v2
  disk is not bootable yet; qcow1 remains the working installed guest.

## Exact next step (deterministic, unblocked)

1. Let PID 724993 finish (it should reboot → QEMU `-no-reboot` exits). If it does not within ~5 more minutes, quit
   it and re-run the install stage; do not boot a half-finalized disk.
2. Boot the installed disk: run stage (`-boot order=c`, no `-no-reboot`, hostfwd 3389/2222) and wait for
   OOBE/AutoLogon; the FirstLogonCommands then fire in order.
3. Verify text-level: `grep 'GET /marker?step=' /tmp/marker2.log` (expect `step=start` then `step=done`), RDP X.224
   answer on `127.0.0.1:3389`, and the SSH round-trip
   `ssh -p 2222 -i rig/win/agent_key -o StrictHostKeyChecking=no cayleb@127.0.0.1 'ver'` → Banner/Windows version.
4. If any FirstLogonCommand needs elevation it may fail silently (UAC); the marker steps tell us where it stopped.
   The fallback is the same enable commands run from the guest console (visual seat) — the deterministic route is
   still strictly better than blind input.

## Blocker named

**Bounded pass exhausted: Windows Setup's WinPE finalize phase on disk2 did not complete within the time-box (flat
disk, CPU-bound, no reboot), so OOBE/FirstLogon never ran and the guest is not driveable yet.** No blocker in the
rig mechanics: answer consumption, partition/format/apply and the launcher fixes are all proven in this pass; what
remains is simply the finalize duration. qcow1 (installed, booting) is preserved as the working fallback.

---

# CONTINUATION (2026-11-02 02:49–03:20Z) — **DRIVEABLE: Windows guest installed, OOBE first logon reached, RDP live** (SSH capability still installing)

Continued the deterministic route exactly as ordered: the stalled finalize was quit, the same install was re-run once
on a **fresh** target disk (`win11-eval-2.qcow2` recreated; qcow1 untouched), and the run stage was booted. No image
reads, no root, no host changes outside the rig, $0.

## Install (fresh disk, unattended) — completed

- Launch 02:54:16Z → WinPE boot, answer consumed, partition/format/apply 0 → 12.77 GiB → finalize → guest reboot at
  **03:00:33Z** (QEMU `-no-reboot` exit). Total ≈ 6 min 17 s. qcow2 after install **13,391,560,704 B**.
- Run stage on the installed disk: serial `BdsDxe: loading Boot000B "Windows Boot Manager" from
  HD(1,GPT,B84A8E18-1290-4A16-831E-447FB7E03C1E,0x800,0x96000)/\EFI\Microsoft\Boot\bootmgfw.efi` → `starting`;
  **3 boot-manager loads** (OOBE reboots), disk grew to **19.8 GB** during first-logon provisioning.
- **OOBE reached first logon** — proven by the FirstLogonCommands themselves: the answer file's command #1 fired →
  **`GET /marker?step=start` in `rig/logs/marker-8765.log` at 03:08:35Z** (attributable listener PID 748996, log
  `rig/logs/marker-8765.log`). FirstLogonCommands only run after OOBE completes to an interactive logon.

## Driveable evidence (text-level)

**RDP is LIVE** — the answer's command #2 enabled it, and the guest answers the standard RDP handshake through the
QEMU hostfwd. Exact command + output (2026-11-02T03:20:13Z):
```
$ printf '\x03\x00\x00\x13\x0e\xe0\x00\x00\x00\x00\x00\x01\x00\x08\x00\x03\x00\x00\x00' \
    | socat - TCP:127.0.0.1:3389,connect-timeout=3 | od -An -tx1
 03 00 00 13 0e d0 00 00 12 34 00 02 2f 08 00 02
 00 00 00
```
= TPKT + X.224 **Connection Confirm (0xd0)** + RDP negotiation response: a real Windows RDP listener on the guest,
reached over `127.0.0.1:3389` (QEMU user-net hostfwd). **SSH not yet up**: FirstLogonCommand #3
(`Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0`) is still installing at report time (~13 min;
guest CPU ~390 %, disk still growing, no `?step=done` marker, no SSH banner on 2222). Named, not smoothed.

Transport inventory (live): VNC `127.0.0.1:5902`, RDP hostfwd `:3389` **answering**, SSH hostfwd `:2222` LISTEN but no
banner, monitor `rig/win/monitor.sock`, marker listener `0.0.0.0:8765` (log `rig/logs/marker-8765.log`).
Guest left RUNNING: QEMU **PID 757632**, ~10.3 GiB RSS; qcow1 (previous installed guest) untouched at
**17,083,138,048 B** as fallback.

## Exact verification for the checker / next seat (no image reads)

```bash
cd ~/workspace/harness-build/scratch-tb-l4/rig
grep 'GET /marker?step=' logs/marker-8765.log      # expect step=start (03:08:35Z) and, after the SSH capability installs, step=done
printf '\x03\x00\x00\x13\x0e\xe0\x00\x00\x00\x00\x00\x01\x00\x08\x00\x03\x00\x00\x00' | socat - TCP:127.0.0.1:3389 | od -An -tx1   # RDP CC
socat - TCP:127.0.0.1:2222                          # expect SSH-2.0-OpenSSH_for_Windows_* once the capability lands
ssh -p 2222 -i win/agent_key -o StrictHostKeyChecking=no cayleb@127.0.0.1 'hostname & ver & $PSVersionTable.PSVersion.ToString()'
```
The admin user is **`cayleb`** (the answer file's local Administrator account). **FENCE CORRECTION (checker finding):**
this answer file holds a **rig-local placeholder password**, and that placeholder **was present in the board copy**
(`board/probes/team-b-windows-vm/autounattend-v2.xml`); the fresh FL2-r2 checker flagged it, and the Lead has since
redacted both board copies (local board + shared repo); no other copies were found. **The password was reproduced in
a board artefact before redaction — that is not claimed otherwise.** No real credential value exists anywhere in this
work: the value is a locally generated placeholder for a throwaway rig guest, and the board copies now carry no
password. The rig's public key is installed as the admin `authorized_keys`; private key `rig/win/agent_key` never
leaves the rig.

## Finalize-timing lesson (launcher doc)

Filed as `board/probes/team-b-windows-vm/FINALIZE-TIMING-NOTE.md`: fresh-disk install takes ~75 s to WinPE, ~3.5 min
of apply, finalize ends within seconds of the last disk growth (≈6.3 min total); **a reused/half-written target disk
can wedge WinPE for 20+ min** (flat disk, CPU-busy, no reboot) — always install to a fresh qcow2, never stop mid-
finalize (it yields an unbootable disk), and allow ≥15 min before calling a stall.

## FL2 implication (no "met" claim before the fresh checker)

The Windows leg upgrades from "surface installed/not driveable" to **installed + OOBE first logon + RDP live**, with
the SSH channel pending only on the OpenSSH capability install. The Ergon Windows companion surfaces still need the
companion artefacts themselves and a desktop automation seat (RDP now available; UIA/WinAppDriver or SSH/PowerShell
for text evidence). Fresh checker spot-verifies the commands above before any FL2 "met" wording.
