# Team A — Finish Line 2: VM-surface end-to-end verdict — **REVISION r3 (re-file, corrections applied)**


> **CORRECTIONS APPLIED — r3 (supersedes r2; r2 is kept UNCHANGED for the record; the verdict line is unchanged and exactly
> as accepted).** Evidence trail: `board/reports/checker-fl2-r2.md` (ACCEPT (honest re-file), two named corrections).
> Corrections in this revision:
>
> | # | Checker correction | What r2 said (quoted) | What r3 says |
> |---|---|---|---|
> | **C1** | Companion-artefact wording imprecise | "no Windows companion artefacts … companion artefacts do not exist" | **"no packaged/compiled Windows companion artefacts"; the companion SOURCES exist** (`support/app/GetHelp.cs`, `support/app/ReviewDesk.cs`, `scripts/compile-gethelp.mjs`, `scripts/compile-reviewdesk.mjs`), there is no compiled `.exe`/`.dll`, and the **DSH-era Ergon package is absent** (`PACKAGE.md` §5 items 1/5/6). This wording now appears everywhere r2 said the companions "do not exist". |
> | **C2** | §8 credential claim false as written | "no credential values (the rig-local answer-file password is not reproduced anywhere)" | **Exact replacement:** "the answer file's local-account credential is a rig-local placeholder; it was present in the board copy of the answer file, was flagged by the fresh checker, and the board copies have been redacted by the Lead; no real credential value appears anywhere in this filing." |
>
> **Live-state note added in r3 (checker-sourced, re-verified by this lane at 03:34:53Z):** `?step=done` appeared at
> **03:26:46Z** (after r2's filing), so r2's "still installing / no step=done" reason is **stale**; **SSH still does not
> answer** (no banner, client timeout) so the SSH channel remains **pending in effect**. No verdict change.


**STATUS: FL2 RE-FILED ON UPGRADED WINDOWS EVIDENCE. PHONE VM = REAL PROVEN (unchanged); WINDOWS VM SURFACE = INSTALLED +
OOBE FIRST LOGON REACHED + RDP LIVE (REAL, this revision); SSH channel PENDING; the Ergon Windows companion surfaces are
package-blocked, so FL2 "end-to-end through BOTH VM surfaces" is STILL NOT MET — and that is the verdict.** This file
supersedes `team-a-vm-end-to-end.md` r1 (2026-11-02T01:34Z, kept for the record with a superseded banner); it does not
overwrite r1's evidence. Prepared by `ergon-lead` (Team A), 2026-11-02 ~03:23Z, after Team B's
`board/reports/team-b-windows-vm-result.md` §CONTINUATION (filed 22:20 ET). **$0; no image reads/screenshots; no root; no
host input; no changes to Team B's rig or files.** Our probes were read-only (stat/grep/ps/TCP protocol probes); the guest
was **not** stopped, restarted or written to.

**Verdict line (the re-file):** *phone VM REAL proven; Windows VM surface installed + driveable (RDP); Ergon Windows
companion surfaces package-blocked; Ergon end-to-end through BOTH VM surfaces still NOT met (companion package absent).*

## 1. What changed since r1 — and independent verification by this lane (03:21:51Z)

| Claim (Team B) | Our read-only verification (2026-11-02T03:21–03:23Z) | Status |
|---|---|---|
| Windows 11 Enterprise Eval **installed on a fresh qcow2** | `rig/disks/win11-eval-2.qcow2` = **19,788,595,200 B** (19.8 GB; install completed 13,391,560,704 B then first-logon provisioning grew it); `win2d-serial-install.log` present; install launch 02:54:16Z → reboot 03:00:33Z (≈6 m 17 s) per Team B | **VERIFIED** |
| Guest **boots from disk0** | `win2d-serial-run.log`: `BdsDxe: loading Boot000B "Windows Boot Manager" from HD(1,GPT,…)/\EFI\Microsoft\Boot\bootmgfw.efi` → `starting`, **3 loads / 3 starts** (OOBE reboots) | **VERIFIED** |
| **OOBE reached first logon** (FirstLogonCommands only run after OOBE completes to an interactive logon) | `grep 'GET /marker?step=' rig/logs/marker-8765.log` → line 2: `127.0.0.1 - - [01/Nov/2026 22:08:35] "GET /marker?step=start HTTP/1.1" 200 -` (= **03:08:35Z**); marker listener is `python3 -m http.server 8765 --bind 0.0.0.0 --directory /tmp/wwwmarker` pid 748996 (attributable). **Live update (checker 03:26:46Z + our re-read 03:34:53Z): `?step=done` present at 03:26:46Z** → the FirstLogonCommands chain ran to completion; r2's 'no step=done yet' reason is superseded (SSH still not answering — see SSH row) | **VERIFIED** |
| **RDP is LIVE at protocol level** | Reproduced ourselves: `printf '\x03\x00\x00\x13\x0e\xe0\x00\x00\x00\x00\x00\x01\x00\x08\x00\x03\x00\x00\x00' \| socat - TCP:127.0.0.1:3389 \| od -An -tx1` → `03 00 00 13 0e d0 00 00 12 34 00 02 2f 08 00 02 00 00 00` = **TPKT + X.224 Connection Confirm (0xd0) + RDP negotiation** | **VERIFIED** |
| **SSH pending** (capability finished, sshd still not answering) | r2 filing (03:21Z): `socat - TCP:127.0.0.1:2222` → **no banner**; `ssh -p 2222 … cayleb@127.0.0.1 hostname` → `Connection timed out during banner exchange`. **Live update (checker 03:26:46Z + our re-read 03:34:53Z): `?step=done` is now present**, but SSH **still** answers nothing (no banner, timeout) → channel pending in effect. Port 2222 is QEMU's hostfwd only | **VERIFIED PENDING (reason updated: sshd not answering, not capability installing)** |
| Guest left **RUNNING** | `ps -p 757632` → RSS **10,553,760 kB**, **386 % CPU**, elapsed ~20 m; listeners: VNC 5902, RDP 3389, SSH hostfwd 2222, marker 8765 (qemu fd19/fd17/fd98) | **VERIFIED** |
| qcow1 previous guest untouched as fallback | `rig/disks/win11-eval.qcow2` = **17,083,138,048 B**, mtime 2026-11-01 21:13:13 ET (unchanged) | **VERIFIED** |

**Still not claimed:** the OOBE **desktop** itself (no image reads — this lane cannot see it); **SSH** (capability still
installing; no banner, no `?step=done`); any **desktop automation** of the Ergon Windows companions (no packaged/compiled companion artefacts exist; the companion sources exist — `support/app/GetHelp.cs`, `support/app/ReviewDesk.cs`, `scripts/compile-gethelp.mjs`, `scripts/compile-reviewdesk.mjs` — but there is no compiled `.exe`/`.dll` and the DSH-era Ergon package is absent; `PACKAGE.md` §5 items 1/5/6).

## 2. Current rig state (verified 2026-11-02T03:21:51Z)

| Node | State | Evidence |
|---|---|---|
| **Phone VM** | **USABLE / REAL-proven in r1**: AVD `rig-phone` boots headless to `boot_completed=1` (Android 15/API 35, 1080×2400), adb control + scripted native tap proven; in-browser Ergon phone-shaped artefact rendered in real Chromium with live JS (fixture, labelled). r1 evidence unchanged: `board/probes/team-a-vm-e2e/`. | r1 §1/§2 + `adb-boot-state.txt`, `settings-before/after.xml`, `c1-computed-proof.json`, `settle-proof.json` |
| **Windows VM** | **INSTALLED + DRIVEABLE (RDP)**: guest running (QEMU PID 757632, ~10.5 GiB RSS, ~386 % CPU while the OpenSSH capability installs); OOBE first logon reached (marker 03:08:35Z); RDP handshake live on `127.0.0.1:3389`; VNC 5902 + monitor socket; SSH hostfwd listening but **no banner** (pending). | §1; Team B `team-b-windows-vm-result.md` §CONTINUATION |
| **Ergon product surfaces** | **UNCHANGED — package-blocked**: no Ergon-on-DSH product package (G1 whole not satisfied: Nox `20261102T003600Z` §3), no phone surface (PACKAGE-PENDING), no **packaged/compiled** Windows companion artefacts. The companion **sources** exist (`support/app/GetHelp.cs`, `support/app/ReviewDesk.cs`, `scripts/compile-gethelp.mjs`, `scripts/compile-reviewdesk.mjs`), but there is no compiled `.exe`/`.dll` and the DSH-era Ergon package is absent (`PACKAGE.md` §5 items 1/5/6). | `docs/acceptance/demo-company/PACKAGE.md` §5; r1 §3 |

## 3. Leg table — updated (REAL / FIXTURE / PENDING; each with command + output + timestamp + verdict)

| # | Leg | Kind | Evidence (commands + captured output + time) | Verdict |
|---|---|---|---|---|
| **L0** | Rig state read-only | **REAL** | r2 probes above (ps/stat/grep/socat/TCP) + r1 probes | **PASS** |
| **L1** | Phone VM boot + scripted control | **REAL** | r1: headless AVD boot → `boot_completed=1`; Settings tap (407,859) → Internet/SIMs/Airplane screen (01:27–01:30Z) | **PASS** (carried) |
| **L2** | R4 phone surface (Ergon) | **FIXTURE + PENDING(product)** | r1: phone-VM real Chromium renders the frozen v2 mobile comp (label + title text), tokens resolve, live JS/motion settles 0 running/0 infinite; **no Ergon product phone surface exists** | **FIXTURE PASS / PRODUCT PENDING** |
| **L3** | R3 second-device angle | **FIXTURE + PENDING(product)** | r1: phone VM → host `10.0.2.2` ping 2/2, HTTP 200 for the served page; real shareable URL still needs host + package | **FIXTURE PASS / PRODUCT PENDING** |
| **L4** | **Windows VM surface** | **REAL (surface) / PENDING (companions)** | Guest installed + boots from disk0 (3 boot-manager loads); **OOBE first logon** via marker `?step=start` 03:08:35Z; **RDP live** (X.224 CC 0xd0 reproduced 03:21:51Z); SSH pending (capability done 03:26:46Z, sshd still not answering; no banner). **Ergon Windows companion leg cannot run: no packaged/compiled companion artefacts exist** — the sources exist (`support/app/GetHelp.cs`, `support/app/ReviewDesk.cs`, `scripts/compile-gethelp.mjs`, `scripts/compile-reviewdesk.mjs`), but there is no compiled `.exe`/`.dll` and the DSH-era Ergon package is absent (`PACKAGE.md` §5 items 1/5/6). | **SURFACE REAL & DRIVEABLE / COMPANION LEG PENDING (package)** |
| **L5** | R2 model-quality | **PENDING-ON-ROSTER-ACTIVATION** | Proof pre-written in r1 §4 (real acting agent on the DeepSeek route + one Haiku 5.5 comparison pass over the same six chain prompt sets; evidence layout fixed) | **PENDING** |
| **L6** | R6 voice surface | **PENDING** | Phone/voice surfaces are package pieces (P20-F5 class); Windows-only surfaces stand-in; nothing to drive on either VM | **PENDING** |

## 4. The four S8 named pendings — updated dispositions

| Pending | Disposition after the Windows upgrade |
|---|---|
| **R2 model-quality** | **PENDING-ON-ROSTER-ACTIVATION** — proof pre-written (r1 §4); runs on the Lead's activation message. |
| **R3 hosted URL / DNS / second device** | **PENDING (host + package)**; the second-device *transport* angle is fixture-proven on the phone VM, and the Windows VM now adds a second driveable device (RDP) — but the real non-localhost Ergon URL still needs G4 host + G1 package. **Not claimed.** |
| **R4 phone surface** | **PENDING (product surface, PACKAGE-PENDING)**; the phone VM itself is REAL-proven and renders a phone-shaped Ergon artefact as a labelled fixture. |
| **R6 voice surface** | **PENDING (package)**; unchanged. |

## 5. What would close FL2 (unchanged, now narrower)

1. **The Ergon-on-DSH package including the Windows companion build (compiled/packaged from the existing companion sources) + phone/voice surface** — the only remaining
   blocker on the Ergon side (G1 whole; Nox `20261102T003600Z` §3). With it, the Windows leg becomes real: install the
   companions in the guest, drive WPF setup / Get Help / ReviewDesk over **RDP** (UIA/WinAppDriver) or **SSH/PowerShell**
   once the capability lands, evidence = UIA trees / logs / text output, **never screenshots**.
2. **SSH capability** for a scriptable Windows channel — currently installing; check `?step=done` + `SSH-2.0-OpenSSH_for_Windows_*`
   banner; RDP is already the desktop transport if SSH lags. Named pending, not smoothed.
3. **Roster activation** for R2.

## 6. Checker spot-verification (fresh checker; no "met" wording before this)

Run from `~/workspace/harness-build/scratch-tb-l4/rig` (read-only):

```bash
# 1. Windows guest installed + booted from disk (not just "QEMU runs")
stat -c '%n %s' disks/win11-eval-2.qcow2          # expect ~19.79 GB (installed), not ~324 KiB
grep -c 'BdsDxe: loading Boot000B' logs/win2d-serial-run.log   # expect 3 (boot-manager loads)
# 2. OOBE first logon reached (attributable marker; FirstLogonCommands only run post-OOBE)
grep 'GET /marker?step=' logs/marker-8765.log      # expect ?step=start at 22:08:35 ET; ?step=done iff SSH finished
ps -o pid,rss,pcpu -p 757632                        # guest running (~10.5 GiB RSS)
# 3. RDP live at protocol level
printf '\x03\x00\x00\x13\x0e\xe0\x00\x00\x00\x00\x00\x01\x00\x08\x00\x03\x00\x00\x00' \
  | socat - TCP:127.0.0.1:3389 | od -An -tx1        # expect TPKT + X.224 CC (0e d0 ...)
# 4. SSH still pending-or-up (do not claim either way without this)
socat - TCP:127.0.0.1:2222                          # expect banner when capability lands; empty = pending
# 5. Fallback disk untouched
stat -c '%n %s %y' disks/win11-eval.qcow2           # expect 17083138048 B, mtime 2026-11-01 21:13:13 ET
```

Expected verdict after spot-check: the three node-states above hold → FL2 re-file stands as **"phone REAL proven; Windows
surface installed + driveable (RDP); companions package-blocked; end-to-end through BOTH VM surfaces NOT met."** If SSH has
landed by the checker's run, the disposition bar is still the same: a driveable channel is **not** an exercised Ergon leg
without packaged/compiled companion artefacts.

## 7. Evidence paths

- **r2 additions (this revision):** the live probes in §1 (commands + outputs reproduced here; raw capture can be repeated
  by the checker from the paths shown). Team B's own report: `board/reports/team-b-windows-vm-result.md` §CONTINUATION and
  `board/probes/team-b-windows-vm/FINALIZE-TIMING-NOTE.md`.
- **r1 evidence (unchanged, phone VM REAL):** `board/probes/team-a-vm-e2e/` — `adb-boot-state.txt`, `emulator.log`,
  `browsers.txt`, `home.xml`, `settings-before/after.xml`, `chrome*.xml`, `devtools-probe-pre-fix.json`,
  `c1-computed-proof.json`, `settle-proof.json`, `p2-proof.json`, `p3-proof.json`, `httpd.log`, `teardown.txt`, `probe.py`
  + `*-expr.js`.
- **This lane's probes were read-only:** TCP handshakes to RDP/SSH, stat/grep/ps, no marker requests, no guest
  stop/restart/write, no rig file edits, no Team B file edits.

## 8. Honesty / fences

No image reads or screenshots (OOBE desktop **not** claimed — only the text/protocol proofs above); no root; no host
changes; **the answer file's local-account credential is a rig-local placeholder; it was present in the board copy of the answer file, was flagged by the fresh checker, and the board copies have been redacted by the Lead; no real credential value appears anywhere in this filing**; no owner-facing action; $0.
The rig is Team B's build and the guest is left running by them — this revision only re-files the **Team A FL2 verdict**
with the upgraded evidence and the same rule: **nothing is "met" until a fresh checker spot-verifies, and the Ergon legs
still wait on the Ergon package.**

**Revision history:** r1 = 2026-11-02T01:34Z (phone VM REAL proven; Windows VM not installed → FL2 pending) — superseded by
r2. r2 = 2026-11-02T03:23Z (Windows surface installed + OOBE first logon + RDP live; SSH pending; companions
package-blocked; FL2 still NOT met) — **kept unchanged**, superseded by this r3. r3 = 2026-11-02T03:35Z: checker FL2 r2 ACCEPT
(honest re-file) with its two named corrections applied (C1 companion-artefact wording; C2 §8 credential claim), plus the
checker-sourced live note that `?step=done` landed at 03:26:46Z while SSH still does not answer. **Verdict unchanged.**
