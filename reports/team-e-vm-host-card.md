# TEAM E / LANE 6 — VM NODE HOST CARD (host facts only; Lane 4 owns the rig build)
Prepared 2026-11-01 09:10 ET by upkeep-lead, per Lead ruling (e). Read-only probes; nothing installed or changed.
Consumer: Lane 4 (surfaces lead — rig build + sidebar panel exposure). Rehearsal requirements come from Team A.

## 1. Host identity
| fact | value | evidence |
|---|---|---|
| host | `outis-desktop`, Linux 7.2.8-9-omarchy x86_64 | `uname -r` |
| CPU | AMD **Ryzen AI MAX+ 395** w/ Radeon 8060S — 16 cores / **32 threads**, 1 socket, AMD-V exposed | `lscpu` |
| RAM | **62 GiB visible** (physical 128 GiB; ~64 GiB is iGPU carve-out per solved RAM mystery), **42 GiB available** now | `free -h` |
| disk | 1.9 TB btrfs (`/dev/mapper/root[/@home]`, zstd:3), **594 GiB free**, 69% used | `df -h /home` |
| uptime | 3d 18h — fleet/kalshi/tripwire all live; treat this as a production host | `uptime -p` |

## 2. Virtualization capability — READY at host level
| capability | status | evidence |
|---|---|---|
| KVM | **present & usable**: `/dev/kvm` crw-rw-rw- root:kvm; cayleb can read+write (mode 666) | `ls -la /dev/kvm` |
| CPU virt | AMD-V (`svm`) exposed, 32 threads | `lscpu`, `/proc/cpuinfo` |
| QEMU | **qemu 11.1.2** installed (`qemu-base`, `qemu-system-x86_64`, qemu-block/chrdev/display modules) | `qemu-system-x86_64 --version` |
| UEFI/Secure Boot | **edk2-ovmf installed** incl. `OVMF_CODE.4m.fd`, `OVMF_CODE.secboot.4m.fd`, `OVMF_VARS.4m.fd` | `/usr/share/edk2-ovmf/x64/` |
| TPM 2.0 emulation | **swtpm installed** (libtpms, tpm2-tss) — Windows 11 TPM requirement can be met legitimately | `command -v swtpm` |
| libvirt | **NOT installed** (no `virsh`, no libvirtd) | `systemctl is-active libvirtd` → inactive; `command -v virsh` → none |
| GUI front-ends | **GNOME Boxes 50.0 (flatpak, user)** and **virt-manager 5.1.0 (flatpak)** installed; no VM images exist yet (`~/.var/app/org.gnome.Boxes` = 52K) | `flatpak list` |
| Android tooling | **NONE installed** — no `adb`, `emulator`, `sdkmanager`, no SDK dir | `command -v` probes |

## 3. Resource budget proposal (fits current headroom; do not exceed)
- **Windows Pro VM (minimal, debloated):** 8 vCPU, **8–12 GiB** RAM, **80 GiB** qcow2 (sparse; ~35–45 GiB actual after install), virtio disk/net, OVMF + swtpm TPM 2.0, UEFI Secure Boot on.
- **Android emulator node:** 4–6 vCPU, **6–8 GiB** RAM, **30 GiB** AVD, AOSP system image, KVM-accelerated.
- **Combined ceiling:** ≤16 vCPU, ≤20 GiB RAM, ≤160 GiB disk allocated (leaves ≥400 GiB free and ≥20 GiB RAM headroom for the live desk/fleet). Run one at a time during weekday market hours; both only off-hours.
- Do **not** put VM images on `/tmp` (tmpfs 32G, 14G used). Default locations are fine on `@home` (btrfs zstd compresses images poorly — disable compression on the image dir if copying is slow).

## 4. Licensing requirements — legitimate at build time
| item | requirement | legit build-time route | status |
|---|---|---|---|
| Windows guest | Needs a real Windows licence (Pro/Enterprise; Home lacks RDP/domain features) | **(a) Windows 11 Enterprise Evaluation** — free official 90-day ISO from Microsoft, legitimate for build/test, expires, no production use; or **(b) Cayleb's existing retail Pro/Enterprise key** (not evidenced on this host); or **(c) new retail purchase** (spend → Cayleb's pick, not ours) | **no Windows install media on host**: only `~/.winvm-scratch/answer.iso` (67 KB, label ANSWER) and `apps.iso` (65 KB, label APPS) — no Win ISO/WIM/ESD, no key file. Do not treat OEM keys from other machines as compliant. |
| Windows virtio drivers | `virtio-win.iso` (Fedora project, free) for virtio disk/net/balloon; or fall back to SATA+e1000 (slower, no extra media) | download once, verify sha256 | absent |
| TPM 2.0 / Secure Boot | emulated with **swtpm** + OVMF secboot (both installed) — no Microsoft licence needed for emulation | host-ready | READY |
| Android emulator | SDK/emulator free; **AOSP system image** needs no licence acceptance; **Google Play system image** requires accepting Google Play terms and is only for compatible devices → use **AOSP** for build-time | user-level SDK install, no root | needs install (~2–3 GB) |
| VM software | Boxes/QEMU/virt-manager are open source; no licence | — | READY |

## 5. Isolation options (build-time posture)
1. **GNOME Boxes (flatpak, already installed, no root)** — sandboxed front-end, user-mode NAT networking; fastest path to the minimal Windows VM. Its images live under `~/.var/app/org.gnome.Boxes/` (sandbox).
2. **virt-manager + libvirt** — better hooks (snapshots, SPICE config, per-VM network) but **requires root package install** (`libvirt`, `qemu-desktop` group, `virt-install`) and a systemd enable — a Cayleb window / root step, not Lane 6.
3. **Network:** each VM on user-mode NAT with **no host folder sharing, no clipboard, no USB redirection**; Windows guest egress-only; emulator `adb` bound to `127.0.0.1`.
4. **Display for Lane 4's sidebar panels:** expose per-VM display over **loopback only** (SPICE unix socket preferred; VNC on `127.0.0.1:<port>` if a panel needs a TCP endpoint). Never bind 0.0.0.0 / the tailnet.
5. **Hardening gap found (root fix, reported not performed):** `/dev/kvm` is mode **666** — any local account can use KVM. Recommend udev rule → `root:kvm 0660` plus cayleb in `kvm` group (cayleb is **not** currently in the kvm group). This is a host security item, not a blocker.
6. **Snapshots/backups:** keep the Windows base image pristine; clone per test. btrfs snapshots available on `@home`.

## 6. What Lane 6 supplies vs what Lane 4 owes
- **Supplied (this card + live probes):** virtualization readiness, exact budget numbers, licence/media status, TPM/OVMF/Secure Boot readiness, isolation and panel-exposure constraints, `/dev/kvm` gap, scratch/media inventory.
- **Lane 4 owes:** the rig build itself (VM creation, Android emulator install, panel surface) and any root step request routed through the Lead → Cayleb.
- **Owner acts (one step each, when reached):** (1) create/download the official Windows 11 Enterprise Evaluation ISO and confirm the licence route (Cayleb's pick if he uses an existing vs new licence); (2) any root/pacman install if virt-manager+libvirt is chosen. Nothing is asked tonight.

## 7. Scratch that must be preserved until the rig exists
- `~/.winvm-scratch` (345 MB): `answer.iso` (ANSWER), `apps.iso` (APPS), OCR/screenshot artifacts from an earlier VM session — planned VM dependency, **keep in place** (HOLD item H9). No credential reads performed.
