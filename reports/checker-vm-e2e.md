# Checker — Team A Finish Line 2 evidence (`team-a-vm-end-to-end.md`)

**Seat:** independent checker (fresh context; built none of this work). **Subject:** `board/reports/team-a-vm-end-to-end.md`
+ evidence `board/probes/team-a-vm-e2e/` (27 files, 256 KB), claims dated 2026-11-02T01:20–01:34Z.
**Checked:** 2026-11-02T01:36:05Z → 01:45Z by this seat. **No image reads/screenshots, no root, no host-Chrome/Studio
touch, no rig-image writes; $0.** The only active step taken was the permitted headless AVD re-boot (torn down).

**VERDICT ON THE FILING: ACCEPT (honest filing).** The evidence supports the report's bottom line — **two proven legs
(phone REAL + phone-VM fixture) plus two named external blockers (Windows guest not installed; Ergon phone/voice
product surface absent), not smoothed** — and its explicit "end-to-end through both VM surfaces is NOT met" is correct.
I independently re-ran the phone REAL leg (headless boot + scripted Settings tap) and re-read every cited file. Three
specific sub-claims are **UNVERIFIED** for lack of captured output (ping 2/2, host curl 200, DevTools forward port
19222) and a few minor imprecisions are named in §5; none changes the bottom line. **Current-state caveat:** a
concurrent Windows install attempt started at 01:43:14Z (after Team A's window) — Team A's absence claims were true at
capture and at my 01:37Z re-read, but the rig's Windows node is now live again (guest still not installed).

`VM_E2E_VERDICT: ACCEPT (honest filing)` · `PHONE_REAL_LEG: PASS (independently re-run)` · `FIXTURE_LEGS: PASS (3 sub-claims UNVERIFIED)` ·
`WINDOWS: PENDING — guest not installed (verified)` · `FILING_CLAIM "two legs + two blockers": SUPPORTED` ·
`CHECKED_AT: 2026-11-02T01:45Z` · `CHECKER: checker-vm-e2e`

---

## 1. Per-claim table

| # | Claim (report) | Verdict | Evidence / reason |
|---|---|---|---|
| 1a | Windows guest NOT installed: qcow2 **393,216 B** file / **324 KiB actual** / **80 GiB virtual**, mtime `2026-11-01 11:52:59 ET` | **PASS** | Independently re-read: `stat` 393,216 B, mtime identical; `qemu-img info` virtual 80 GiB, disk size 324 KiB (`checker-vm-e2e/00-rig/P1-windows-recheck.txt`) |
| 1b | mtime unchanged tonight | **PASS (with note)** | mtime matches the evidence file and predates Team A's window; no earlier baseline is captured in the evidence itself, but Lane 4's disk-growth record is consistent with it and no write occurred during the Team A window |
| 1c | No QEMU process, no monitor socket, no VNC/3389/2222, swtpm down | **PASS AT CAPTURE — wording imprecise; state now changed** | At my 01:37:50Z re-read: no qemu, no monitor socket, no 5902/3389/2222, swtpm not running. **But** Team A's own `windows-absence.txt` shows a qemu process — the *phone* emulator (`-avd rig-phone`); the "no QEMU" wording means "no Windows QEMU". The file's swtpm line is self-referential (pgrep matched the evidence command itself). **Current state:** a concurrent Windows install attempt started **01:43:14Z** (PID 698517, monitor socket, VNC 127.0.0.1:5902, swtpm PID 698500) — after Team A's window; qcow2 still 393,216 B / mtime unchanged, so the guest is still not installed (`00-rig/P4-concurrent-windows-attempt.txt`) |
| 1d | Last logs: signal-15/systemd + usb-bus failure | **PASS** | Live log tails match evidence verbatim: `terminating on signal 15 from pid 1387 (systemd)` + `tpm-emulator: Could not cleanly shutdown the TPM`; `-device usb-storage,drive=ans: No 'usb-bus' bus found` |
| 1e | Phone VM USABLE | **PASS (independently re-run)** | Headless boot: `sys.boot_completed=1`, `emulator-5554 device`, Android 15 / API 35, 1080×2400, AVD `rig-phone`; clean teardown (`01-phone/P2-avd-recheck.txt`) |
| 2a | Headless AVD boot → boot_completed=1, Android 15/API 35, 1080×2400, emulator-5554 device | **PASS** | Original `adb-boot-state.txt` matches; my independent re-run reproduced every value |
| 2b | Scripted native tap Settings → "Network & internet" at **(407,859)** → Internet/SIMs/Airplane-mode screen (text-verified) | **PASS** | Original `settings-before.xml` = Settings screen with the "Network & internet" row; parsed independently: (407,859) is inside the clickable row `[0,770][1080,1001]`; `settings-after.xml` contains `Internet`, `AndroidWifi`, `SIMs`, `T-Mobile`, `Airplane mode`, `Hotspot & tethering`. My re-run of the same tap produced the same screen texts (`01-phone/P1-xml-assertions.txt`, `P2-avd-recheck.txt`) |
| 2c | `uiautomator` dump text-level, **32 nodes** | **PASS** | `home.xml` = 32 nodes |
| 2d | Boot "~3 s" to boot_completed | **UNVERIFIED as exact figure** | No timestamped capture; emulator log records snapshot load `3804 ms`; my re-run booted fast (consistent), but the precise ~3 s figure is an inference, not evidence |
| 3a | Phone-VM Chrome rendered the **frozen v2** mobile comp, served read-only from the host | **PASS** | `page-before.xml`: title `01 · Mobile Chat — Ergon Lumen v2 design comp`, label `DESIGN COMP — NOT A BUILT SURFACE.`, URL bar `10.0.2.2:8765/...`; the served file `board/probes/team-a-design-v2/comps/01-mobile-chat.html` sha256 `66c5edbe…` matches the **current** `FREEZE.sha256` (manifest digest `ea68e398…` = report §7 claim); title/label present in the file; tokens/blur values match the probe (`02-fixture/P1–P3`) |
| 3b | Title + "DESIGN COMP — NOT A BUILT SURFACE" text verified | **PASS** | Page XML + both probe JSONs (`labelPresent:true`) |
| 3c | Transport: `10.0.2.2` ping 2/2 + HTTP 200 | **PARTIAL — HTTP 200 PASS; ping 2/2 UNVERIFIED** | `httpd.log` shows the page + `tokens.css` + `comp-kit.css` + `comp-motion.js` all **200**, only `favicon.ico` 404, and the guest Chrome URL bar proves the guest→host path. **No capture of `ip route` or `ping -c2` exists anywhere in the evidence** — that specific claim is unverified (the transport itself is proven by the real GETs) |
| 3d | "host `curl` HTTP 200" | **UNVERIFIED as a distinct claim** | No curl output captured; `httpd.log` 200s exist but cannot be attributed between host curl and guest Chrome (both appear as 127.0.0.1 through the emulator NAT) |
| 3e | Tokens resolve in real Chromium | **PASS** | `c1-computed-proof.json` tokensResolved = `rgba(198,222,255,0.04/0.07/0.10/0.14)` + `--field-line rgba(206,228,255,0.46)`, identical to `tokens.css` lines 9–28 |
| 3f | `backdrop-filter: blur(14px)` | **PASS** | Probe `phoneBlur:"blur(14px)"`; `tokens.css` `--blur-panel:14px` and `comp-kit.css` uses it |
| 3g | 26 animations → 0 running / 0 infinite after 2.5 s | **PASS** | `c1-computed-proof.json` `animationsNow:26`; `settle-proof.json` (2500 ms wait) `running:0, infiniteIterations:0`. Minor coupling note: the two figures come from two different page loads (`?c1c4=2` vs `?settle=1`); both facts are individually true |
| 3h | Approve → pill changes "Approval needed" → "Approved — one action, recorded" | **PASS** | `pillBefore/pillAfter` in `c1-computed-proof.json` and `devtools-probe-pre-fix.json` |
| 3i | Tab click moves `aria-selected` Chat→Tasks and panels switch | **PASS** | `tabBefore:["true","false","false"]` → `tabAfter:["false","true","false"]`; `panelsAfter:[true,false,true]` in the probe |
| 3j | Sheet opens with the scoped remember wording | **PASS** | `p2-proof.json`: `sheet.open:"true"` + exact quoted text `"Remember for Oakridge move for sending estimates to Marcus — one subject, one action. Revocable; the grant appears in the approvals record."` |
| 3k | Comp 07: field boundary **3.30:1**, empty state, light pool `light-1` + `blur(32px)` | **PASS** | `p3-proof.json`: token `rgba(206,228,255,0.46)`, contrast 3.3 pass; empty state text present; `bg rgba(198,222,255,0.14)`, `blur(32px) saturate(1.15)` |
| 3l | Composer boundary 3.46:1 after the C1 fix (§7) | **PASS** | `c1-computed-proof.json`: border token = `--field-line`, composited stack, contrast 3.46 ≥ 3, pass |
| 3m | Phone-VM DevTools used **19222** | **UNVERIFIED (specific port)** | `probe.py` takes a port argument and the JSON proofs are real `Runtime.evaluate` responses, so a forwarded DevTools session existed — but no forward command/port is captured anywhere in the evidence |
| 4a | R4 phone surface = PACKAGE-PENDING; fixture only | **PASS (substance); cited file not found** | No product phone surface exists; corroborated by board records (`team-a-job03-package-assembled.md`: "phone/voice surface (PACKAGE-PENDING)"; `team-a-pr10-checker-verdict.md`). The cited `PACKAGE.md §5 item 6` is not present at any path reachable from this seat → citation UNVERIFIED |
| 4b | R3 real shareable URL: host + package absent | **PASS as pending-with-blocker** | Consistent with the board's G1/G4 absence; the Nox `20261102T003600Z §3` citation is not independently readable from this seat (authority citation, not evidence) |
| 4c | R6 voice = package pending | **PASS (substance); cited file not found** | Corroborated by the P20-F5-class phone/voice PACKAGE-PENDING records; the cited `REHEARSAL-R0-R7.md` is not present at any path reachable from this seat → citation UNVERIFIED |
| 4d | R2 model-quality: proof pre-written, runs on activation | **PASS (pre-written plan only)** | §4 procedure is present and explicit (6 cast members, gate, evidence layout, verdict rule); nothing executed — correctly labelled pending-on-roster-activation |
| 4e | All Windows companions blocked (guest not installed) | **PASS** | Guest absence verified (1a/1c) |
| 4f | **"End-to-end through both VM surfaces is NOT met"** | **PASS (honest)** | Windows absent + product phone surface absent; report states this plainly in the header and §5/§6 |
| 5a | Host `:9222` owner Chrome untouched | **PASS (with note)** | Owner listener still `chromium pid 2960519` on 127.0.0.1:9222 at every read; no 9222 content/query in the evidence; a negative can't be proven from absence, but nothing indicates use |
| 5b | Phone-VM DevTools used 19222 (containment) | **UNVERIFIED (same as 3m)** | No forward/port capture; no evidence of host-Chrome use either |
| 5c | Teardown complete | **PASS (re-verified)** | No emulator/qemu, no adb device, no 8765 server now; owner listener count 1. (A defunct qemu zombie was briefly visible at 01:37Z then reaped; the Windows qemu at 01:43Z is a **different lane's** new attempt, not Team A's) |
| 5d | Windows disk unchanged | **PASS** | 393,216 B and mtime `2026-11-01 11:52:59.451108133 -0500` identical at every read, including after the concurrent attempt started |
| 5e | $0 | **PASS (no spend evidence)** | No model calls/purchases in the evidence; not independently auditable |
| 5f | No image reads/screenshots | **PASS** | 0 image files in evidence; probes are `Runtime.evaluate`/text/monitor only |
| 5g | No root | **PASS (with note)** | No sudo/pkexec/root in evidence scripts or commands; history not auditable |
| 5h | No host input injection | **PASS** | No such commands in evidence |

## 2. Independent re-runs performed (not just re-reads)

- **Phone REAL leg re-run** (`01-phone/emulator-recheck.log`, `P2-avd-recheck.txt`): booted the same AVD headless with
  the rig env; `boot_completed=1`, `emulator-5554 device`, release 15 / SDK 35, 1080×2400, `rig-phone`; launched Settings,
  verified `(407,859)` sits in the clickable "Network & internet" row, tapped it, and got the same
  Internet/SIMs/Airplane-mode screen; killed the emulator, confirmed no qemu/adb residue.
- **Rig re-read** (`00-rig/`): qcow2 size/mtime/virtual/actual, install media sizes, install-log tails, AVD/SDK versions,
  monitor/VNC/ports/swtpm state.
- **Frozen-set consistency** (`02-fixture/`): current `FREEZE.sha256` verifies (`sha256sum -c` all OK), digest matches
  report §7; served comp file sha matches the manifest; tokens/blur values ground the probe.
- **XML/text assertions parsed independently** (`01-phone/P1-xml-assertions.txt`): node counts, tap-point containment,
  settings-after texts, page title/label/URL/Marlow/tabs.
- **Containment re-verified** (`03-containment/`): host `:9222` still the owner's chromium; no rig listeners; disk unchanged.

## 3. What I could not verify (plainly)

1. `ip route` / `ping -c2 10.0.2.2 → 2/2, 0% loss` — no capture in the evidence (transport itself is proven by real HTTP GETs).
2. "host curl HTTP 200" — no curl output; 200s are in `httpd.log` but attribution to host vs guest is not separable.
3. DevTools forward port **19222** — no forward/port capture (the DevTools session itself is evidenced by the proof JSONs).
4. Exact "~3 s" boot duration — no timestamped capture.
5. Cited `PACKAGE.md §5 item 6` and `REHEARSAL-R0-R7.md` — not present at any path reachable from this seat; the
   substance of both pendings is corroborated by other board records.
6. Nox order/ruling citations — authority citations, not evidence; not independently readable from this seat.

## 4. Findings / imprecisions (none changes the bottom line)

- **F1** "No QEMU process" (Windows row) is imprecise: Team A's own evidence shows the *phone* emulator's qemu running;
  the intended claim (no Windows guest QEMU) is true at capture. The evidence file's swtpm line is self-referential.
- **F2** "FAT16 USB all read 0" slightly overstates Lane 4's own record: at the 240 s sample `ans read 0` while the VM
  was "still in boot/loader phase" (Lane 4 UPDATE 17:45Z), i.e. not yet a conclusive "never consumed"; the blocker
  substance (no answer medium consumed in any completed attempt) is supported.
- **F3** The 26-animations figure and the 0-running/0-infinite settle figure come from two different page loads
  (`?c1c4=2` vs `?settle=1`); both are true, but the report's sentence reads as one probe.
- **F4 — current-state caveat (operational):** at **01:43:14Z** a **new Windows install attempt** started
  (qemu PID 698517, monitor socket, VNC 127.0.0.1:5902, swtpm PID 698500, serial log shows UEFI DVD boot). This is
  *after* Team A's window and after my 01:37:50Z re-read; the qcow2 is still 393,216 B with unchanged mtime, so the guest
  is still not installed — but the "no QEMU/monitor/VNC/swtpm" half of claim 1 is no longer the current state. The Lead
  should treat claim 1 as time-stamped at 01:24–01:31Z and expect the Windows node to keep changing while Lane 4 retries.

## 5. Verdict on the filing

**ACCEPT.** The filing is honest and well-scoped: it does **not** claim end-to-end completion (it says the opposite in the
header), it separates REAL / FIXTURE / PENDING legs, it names both external blockers with exact evidence, and its
numbers, sizes, mtimes, log lines, page text, token/blur/motion/interaction probes and teardown claims check out against
the files — with the phone REAL leg independently reproduced. The three unverified transport/port specifics and the minor
imprecisions above are exactly the kind of detail a maker should have captured; they do not affect the verdict that the
phone side is proven and the Windows/product side is blocked, so the "two proven legs + two external blockers, not
smoothed" framing is supported.

## 6. Evidence

`board/probes/checker-vm-e2e/` — `00-rig/` (windows re-read, AVD/SDK, concurrent-attempt caveat, final disk),
`01-phone/` (XML assertion parse, independent boot+tap re-run log, before/after XMLs), `02-fixture/` (comp file, freeze
consistency, CSS-vs-probe), `03-containment/` (containment re-read, fences), `SHA256SUMS` (verified).
Team A evidence root re-read: `board/probes/team-a-vm-e2e/` (27 files, all present).

`VM_E2E_VERDICT: ACCEPT (honest filing)` · `UNVERIFIED: ping 2/2 · host curl 200 · DevTools port 19222 · exact ~3s boot · cited PACKAGE.md/REHEARSAL-R0-R7 paths` ·
`CURRENT_STATE: Windows install attempt running since 01:43:14Z (guest still not installed)` ·
`CHECKED_AT: 2026-11-02T01:45Z` · `CHECKER: checker-vm-e2e`
