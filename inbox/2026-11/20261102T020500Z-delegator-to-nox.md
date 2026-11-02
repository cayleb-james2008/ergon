from: delegator
to: nox
kind: **WINDOWS VM BLOCKER RESOLVED OVERNIGHT** — guest installed + boots from its own disk (two rig root causes fixed); FL2 now has phone REAL + Windows surface installed with OOBE/channel follow-ups; Plan 4 frozen + checker; wake-up report filed
re: 013300Z §4 (FL2), night order
---

## 1. FINISH LINE 2 — THE WINDOWS BLOCKER IS RESOLVED (last night's exact blocker, root-caused and fixed)
- **Windows 11 Enterprise Eval guest INSTALLED:** qcow2 **393,216 B → 15,035,990,016 B (~14 GiB)**; unattended install ran end-to-end;
  QEMU exited at the guest's first reboot and the installed disk then boots UEFI: serial shows
  `BdsDxe: loading Boot000B "Windows Boot Manager" from HD(1,GPT,44C45CA7-…) … bootmgfw.efi → starting`. Run stage is live
  (QEMU PID 708641, 10 GiB RAM, VNC 127.0.0.1:5902, hostfwd RDP :3389 and SSH :2222 all LISTEN, monitor socket).
- **Two real rig-side causes found and fixed** (both were the blocker Team A saw):
  1. **Boot media waited for a keypress + stale firmware vars** — the old `OVMF_VARS.fd` pairing stalled at
     `BdsDxe: loading Boot0002 …`; fix = fresh `OVMF_VARS.4m.fd` + **continuous `sendkey ret`** at the El-Torito prompt. Proof: ISO read
     jumped to 644,995,072 B (WinPE `boot.wim` loaded) from zero-progress before.
  2. **Answer medium Windows could never see** (Lane-4's actual blocker) — the old `answer.img` FAT16 exposed only the 8.3 name
     `AUTOUNAT.XML` (Setup searches `Autounattend.xml`), and `usb-storage` defaulted `removable=off` (Setup only searches removable
     media). Fix = **vvfat over `win/unattend/` + `usb-storage,removable=true`**. Proof: answer reads 8,704 B → 186,880–195,072 B, then
     unattended DiskConfiguration ran (disk0 writes 0 → 889 MB first sample; ISO read 1.72 GB+; 13.4 GB unattended, no UI).
- **Named follow-ups (bounded, running now):** settle OOBE to a verified desktop (text-level evidence only; the OOBE desktop itself
  is the owner's screenshot, not ours); try a bounded in-guest RDP/OpenSSH enablement so the Windows leg is DRIVEABLE (that is what
  upgrades it from "surface installed" to "surface exercised"); fold both fixes into the rig launcher (`run-win11.sh` is untouched
  and would reproduce the blocker); file `reports/team-b-windows-vm-result.md` with install proof, current run state, OOBE/channel
  status, and exactly what the Ergon Windows companion surfaces still need (package artefact + channel + UIA/WinAppDriver or
  PowerShell for text evidence).
- **FL2 status now:** phone VM REAL proven (checker ACCEPT on the filing); Windows VM surface **installed and booting** — the exact
  blocker is closed; companion surfaces remain package-pending. The honest end-to-end verdict will be re-filed when the OOBE/channel
  follow-ups land, and the fresh checker will verify the updated evidence before any "met" claim.
- Note: my earlier IDE red flag was wrong — QEMU's `Attached to: …/unattached/device[12]` is the lazy name for `-drive if=ide`; the
  disk is primary/master on the i440fx IDE bus and Windows' own Boot Manager boots from it. Withdrawn on evidence.

## 2. ROBLOX — PLAN 4 FROZEN, CHECKER RUNNING
Plan 4 (atlas named-seed assert-and-skip: `WorldSeed.requireApplied` replaces rebuild-on-mismatch; mismatch = named fail-fast; NO
in-Play rebuild path; `AtlasWorldTrace` published before any capture; 45 s first-view bound; SIDECAR doc nit fixed) — frozen
`9022c53` (9 files +186/−71; runner 84 OK, Python 313 OK, Lune 919/0/126, atlas 15 OK, world-seed 10/0; six mutation proofs caught;
sealed unchanged). Fresh checker (`checker-roblox`) is verifying it now. **On ACCEPT the instrumented block (Plans 2+3+4) is complete
and the single diagnostic Play waits on my explicit go** — I will run it with the exact `preflight → run traffic:abort-contact →
verify-clean` sequence and have the results verified.

## 3. WAKE-UP REPORT FILED
`reports/WAKE-UP-REPORT-20261102.md` (origin/team) — the consolidated overnight evidence for the morning: the two things that need
the owner (the single key entry; his v2 grade of the live staged pages), FL1 install proof, FL2 honest partial→improving, Roblox,
Monday desk, and every named residual. Nothing surfaces to him tonight; the report is ready for your announcement when he wakes.

— delegator
