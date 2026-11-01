# LANE 4 — rig status (scratch, no host changes)

Updated 2026-11-01T17:15Z by `fleet-lead`. Root: `~/workspace/harness-build/scratch-tb-l4/rig/`. No apt/systemd/
profile changes; /dev/kvm left at 666 per ruling.

## Verified working
- **Android node:** SDK installed (emulator 37.2.13, platform-tools 37.0.1, `system-images;android-35;google_apis;x86_64`),
  licences accepted + texts recorded (`logs/android-licences-accepted.log`, "All SDK package licenses accepted");
  AVD **`rig-phone`** created inside the rig (`android/avd/`, Pixel 6 profile); **headless boot verified** —
  emulator booted in 27.9 s and `adb devices` showed `emulator-5556 device` (`logs/emulator-first.log`).
- **Windows node:** Win11 Enterprise Eval ISO (8,225,329,152 B, sha256 `bc3f2408…`, direct Microsoft link, no
  account/sign-in), 80 GiB qcow2 disk, OVMF secure-boot vars copy, `virtio-win.iso` (877 MB, sha256 `303f7ae4…`),
  `swtpm` TPM 2.0 (daemon works; socket + pid recorded), `win/unattend/autounattend.xml` (well-formed; local
  account, disk layout, EULA/oobe skip) and `win/run-win11.sh` (stages install/run).
- **Launch smoke PASS:** QEMU boots the install media with TPM + VNC on loopback (`127.0.0.1:5902` listening,
  monitor socket responsive) — vvfat answer-drive fixed (`fat:rw:`).
- **Panel plan:** device panels ship as a client-plugin surface over the existing sidebar (contract in
  `team-b-lane4-mcp-facade-contract.md`); Windows/Android guests are driven through their own loopback transports.

## Open / next
- Observe the unattended Windows install to completion (needs a panel/screenshot path; install is unattended so
  a later boot check suffices), then record guest reachability; Android: scripted tap/screenshot round-trip.
- Panel-plugin skeleton against the DSH client surface (throwaway worktree); MCP facade build waits on Stage 5.
- Named gaps unchanged: Cayleb's **running** Chrome control (CDP-attach decision); disk/media budget vs host card.

## UPDATE 2026-11-01T17:08Z — Windows install root-caused and RUNNING

- **Root cause of the stalled boot found:** the install stage used `OVMF_CODE.secboot.4m.fd` with a **blank
  VARS file** (no enrolled Secure-Boot keys) → the Windows loader was rejected; symptom was qemu at ~100 % CPU
  with **zero disk writes** for minutes. (Observation was monitor/disk-level only; **no screenshots/images read**.)
- **Fix:** install stage switched to **non-secure-boot OVMF** (`OVMF_CODE.4m.fd` + fresh `OVMF_VARS.4m.fd`; swtpm
  TPM 2.0 still attached — Win11 needs TPM + Secure-Boot *capable*, not enabled). Boot-key spam during the
  "press any key" window.
- **Result:** Windows Setup is now running unattended — disk grew 197,888 → 393,216 B within 90 s (qemu ~97 % CPU),
  i.e. the installer has started writing. Install continues; completion check is disk growth + reboot-to-disk +
  guest reachability (all text/monitor-level, no image reads). Secure-boot enablement for the `run` stage is a
  **separate decision** (requires key enrollment / signed loader policy), recorded here as an open item.

## UPDATE 2026-11-01T17:12Z — panel skeleton staged; install retry with Win11 bypasses

- **Panel skeleton created (staged, not installed):** `~/workspace/harness-build/scratch-tb-l4/rig/panel-skeleton/`
  — `package.json` (dsh bundle + client manifest), `cordis.patch.yml` (insert row `rig-panels`), `client.js`
  (two `sidebar.right.pane.tab` registrations: "Windows VM (rig)", "Android (rig)", polling a loopback rig
  bridge `/status` + embedding `/<kind>/view` when up; text-only fallback card), `README.md` (bridge/bundle/
  acceptance plan + constraints). Validated: JSON OK, YAML OK, `node --check` OK. Digests: client.js
  `34af4b19…`, cordis.patch.yml `c32ecdf0…`, package.json `1ae0148f…`.
- **Windows install stall #2 diagnosed:** the first run (secure-boot OVMF blank VARS) was rejected by firmware;
  the second run (non-secure-boot OVMF) wrote the partition table then stalled — the expected cause is Windows 11
  Setup's requirement checks (Secure Boot absent) waiting for a human. **Fix applied:** autounattend now registers
  `LabConfig` bypasses (`BypassTPMCheck`, `BypassSecureBootCheck`, `BypassCPUCheck`) in the windowsPE phase
  (autounattend sha256 recorded), then a fresh install attempt with boot-key spam. Observation continues at
  monitor/disk level only (**no screenshots/images**).
- **Secure-boot enablement for the run stage remains OPEN** (recorded; no key enrollment, no switch without a ruling).

## UPDATE 2026-11-01T17:35Z — Windows answer-media route: vvfat/floppy ruled out, USB attempted

Block-level evidence from the QEMU monitor (no screenshots/images): the VM **does** boot and read the install
media — `cd0` read **644,995,072 B / 10,424 ops**, then froze; `disk0` write stayed **0**; the vvfat answer drive
read **0**, and after switching to a FAT12 floppy image (`unattend.img`) the floppy also read **0** — i.e. Windows
PE never consulted either answer medium, so Setup sat at its first interactive screen (matches the frozen cd0
reads + moderate CPU).
- Ruled out: secure-boot OVMF (loader rejection earlier), then vvfat and legacy floppy as autounattend carriers
  under UEFI.
- Next attempt: a real **FAT16 USB mass-storage** answer volume (`win/answer.img`, 64 MiB, `mkfs.fat -F 16`,
  `autounattend.xml` written at root via pyfatfs with a pkg_resources shim under `rig/tools/pylib`; attached as
  `usb-storage`). Evidence being collected: `ans` reads > 0 (answer file consumed) and `disk0` writes growing;
  then reboot-to-disk/guest reachability.
- Secure-boot enablement for the run stage stays **OPEN** (no key enrollment without a ruling).

## UPDATE 2026-11-01T17:45Z — install media diagnosis (open blocker)

- The USB answer volume initially failed to start (`No 'usb-bus' bus found`), fixed by adding `-device qemu-xhci`;
  VM relaunched and running (monitor responsive). At 240 s: `cd0 read 66,654,208 B`, `disk0 write 0`,
  `ans read 0` — still in the boot/loader phase at that sample (earlier attempts reached `cd0 645 MB` before
  freezing at the first interactive Setup screen without reading any answer medium).
- **Named open blocker (no guessing):** under UEFI, Windows Setup has not yet consumed an answer medium in any
  configuration tried (vvfat read 0, FAT12 floppy read 0, current USB FAT16 attempt still in boot phase). Next
  diagnostics, in order: (1) let the current boot reach the Setup phase and re-sample `ans`/`disk0`; (2) if `ans`
  stays 0, the answer file must ride a device Windows PE enumerates as a *removable FAT volume at boot* — next
  option is embedding it via a partitioned USB image (MBR + FAT16 partition, not a superfloppy) or an ISO rebuild
  (needs an ISO tool not installed); (3) if neither works, the install leg is filed BLOCKED-ON-MEDIA and the rig
  stays at "Android verified / Windows media not booting unattended".
- No images read at any point; observation is monitor/block stat + disk size only.
