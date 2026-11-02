# Windows-VM install finalize timing — launcher lesson (Team B, 2026-11-02)

Measured on the rig (`qemu-system-x86_64 11.1.2`, KVM, 8 vCPU / 10 GiB, IDE disk, OVMF 4m, vvfat answer+removable):

- **Fresh target disk, unattended install:** WinPE start ~75 s after launch; partition/format/apply writes
  0 → ~12.7 GiB in ~3.5 min; the CPU-bound finalize phase ended **~8 s after the last disk growth**
  (`03:00:25Z` last growth → `03:00:33Z` guest reboot → QEMU `-no-reboot` exit). Total install ≈ 6.3 min
  (launch 02:54:16Z → exit 03:00:33Z).
- **Reused / half-written target disk:** retrying the same install on a disk left by an interrupted run stalled in
  WinPE with a flat qcow2 for 20+ min (CPU 100–390 %, no I/O, no reboot). That is not the normal finalize phase.
- **Rules for the launcher/operator:**
  1. Always install to a **fresh, empty qcow2** (the answer sets `WillWipeDisk=true`, but a polluted target can wedge
     WinPE before partition/apply — observed twice).
  2. **Never stop a finalize early.** Quitting during finalize produces an unbootable disk
     (`BdsDxe: failed to load Boot0001 "UEFI QEMU HARDDISK" … Not Found` on the next boot).
  3. Allow **≥15 min** for the whole install stage before declaring a stall; if a run is flat for >10 min with no
     I/O and the target was fresh, quit and retry on a **new** disk rather than reusing the target.
  4. The boot-prompt key loop (`sendkey ret` every ~0.5 s) must run for at least the first ~120 s of the install
     stage; without it the media waits at El Torito and the guest never starts.
