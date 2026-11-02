# Team A — Finish Line 2: Ergon end-to-end through the VM surfaces (leg table + evidence)

**STATUS: PARTIALLY EXECUTED — the phone-VM side is REAL and PROVEN; the Windows-VM side is NOT INSTALLED and is carried as a
NAMED PENDING. `board/probes/team-a-vm-e2e/`.** Prepared 2026-11-02 01:20–01:34Z by `ergon-lead` (Team A), on Nox night
order `20261102T010900Z` §Finish line 2. **$0; no image reads/screenshots (all evidence is text/monitor/probe level); no
root installs; no host input injection; no owner-facing action.**

The honest one-line answer: **the rig can drive a phone VM end-to-end, and it did — but there is no Ergon phone surface to
run on it and no Windows guest to run the Windows companions on**, so Finish Line 2 as "Ergon exercised end-to-end through
both VM surfaces" is **not met**; it is two proven harness legs plus two named external blockers, not smoothed.

## 1. Rig state — ACTUAL, read-only (2026-11-02T01:24–01:31Z)

| Node | Actual state | Evidence |
|---|---|---|
| **Android / phone VM** | **USABLE.** SDK at `rig/sdk` (emulator 37.2.13, platform-tools 37.0.1), AVD `rig-phone` (Pixel 6, android-35 google_apis x86_64). Booted headless by this lane in **~3 s to `sys.boot_completed=1`**; `adb devices` → `emulator-5554 device`; Android **15 / API 35**, physical size **1080×2400**. AVD name read back as `rig-phone`. | `adb-boot-state.txt`, `emulator.log` (boot log), `browsers.txt` |
| **Windows VM** | **NOT INSTALLED — not reachable, no OS.** qcow2 is **393,216 B file / 324 KiB actual** against **80 GiB virtual** (an installed Win11 would be ~10–20 GiB), mtime 2026-11-01 11:52:59 ET (unchanged tonight). No QEMU process, **no monitor socket**, no VNC/3389/2222 listener, swtpm not running. Last install attempts: `win11-install9.log` → `terminating on signal 15 from pid 1387 (systemd)` + `tpm-emulator: Could not cleanly shutdown the TPM`; `win11-install8.log` → `-device usb-storage,drive=ans: No 'usb-bus' bus found`. Lane-4's own recorded blocker: under UEFI **Windows Setup never consumed any answer medium** (vvfat / FAT12 floppy / FAT16 USB all read 0; `cd0` read 645 MB then froze; `disk0` write 0). | `windows-absence.txt`, `board/reports/team-b-lane4-rig-status.md` §UPDATE 17:45Z |
| Install media present | Win11 Enterprise Eval ISO 8,225,329,152 B (sha256 `bc3f2408…`), virtio-win 877,373,440 B, autounattend + answer images — media exists; the guest does not. | `windows-absence.txt` |

## 2. Leg table (REAL / FIXTURE / PENDING — every leg with command + captured output + timestamp + verdict)

| # | Leg | Kind | What was actually run (command → output) | Verdict |
|---|---|---|---|---|
| **L0** | Rig state read-only | **REAL** | `pgrep qemu-system`; `ss -ltnp | grep :5902/:3389/:2222`; `stat` + `qemu-img info` the qcow2; `ls` the AVD/SDK/media. Windows: no process/monitor/ports; qcow2 393,216 B. Android: AVD present, sdk present. | **PASS (read-only)** — Android usable, Windows absent |
| **L1** | Phone VM boot + scripted control | **REAL** | `emulator -avd rig-phone -no-window -no-audio -no-boot-anim -gpu swiftshader_indirect -no-metrics -no-snapshot-save -port 5554` → `boot_completed=1`; `uiautomator dump` (text-level, 32 nodes); native scripted tap: Settings → tap "Network & internet" at **(407,859)** → screen changed to **Internet / AndroidWifi / SIMs / T-Mobile / Airplane mode** | **PASS** — 2026-11-02T01:27–01:30Z (`adb-boot-state.txt`, `home.xml`, `settings-before.xml`, `settings-after.xml`) |
| **L2** | R4 phone surface (Ergon) | **FIXTURE + PENDING(product)** | Phone VM Chrome (real Chromium) opened `http://10.0.2.2:8765/comps/01-mobile-chat.html` served from the host (frozen v2 design comp, read-only): page title **“01 · Mobile Chat — Ergon Lumen v2 design comp”**, label **“DESIGN COMP — NOT A BUILT SURFACE.”** present, text nodes include Marlow / brief / messages. **The Ergon product phone surface does not exist** (`PACKAGE.md` §5 item 6, PACKAGE-PENDING), so no product leg can run. | **FIXTURE PASS** (rig renders + executes a phone-shaped Ergon artefact) / **PRODUCT LEG PENDING** — blocker named below |
| **L3** | R3 second-device angle | **FIXTURE transport** | From the phone VM: `ip route` shows `10.0.2.0/24 dev eth0`; `ping -c2 10.0.2.2` → **2/2 packets, 0% loss**; host `python3 -m http.server 8765` served the set (host curl **HTTP 200**); guest Chrome loaded the page (host log: GET `…/01-mobile-chat.html`, `tokens.css`, `comp-kit.css`, `comp-motion.js` all **200**; only `favicon.ico` 404). | **FIXTURE PASS** (the phone VM can be the second device) / **PENDING(product)** — real R3 needs the shareable non-localhost URL + host + package |
| **L4** | Windows VM / Windows companion surfaces | **PENDING** | No guest exists (L0): qcow2 324 KiB actual, no OS, no reachability. The Windows surfaces (Get Help window, native ReviewDesk, WPF setup) are Win-only companions and cannot be exercised. | **PENDING** — blocker: Windows guest not installed (UEFI answer-medium blocker, Lane 4's rig item) |
| **L5** | R2 model-quality | **PENDING-ON-ROSTER-ACTIVATION** | Proof pre-written (§4). It runs the moment the Lead says the roster is live. | **PENDING** — blocker: roster activation (DeepSeek route) |
| **L6** | R6 voice surface | **PENDING** | Phone/voice surfaces are DSH-era package pieces (P20-F5 class); Windows-only surfaces stand-in; nothing to drive on either VM tonight. | **PENDING** — blocker: package phone/voice surface |

**Runtime probe used for the fixture legs (text-level, no images):** adb-forwarded **phone-VM Chrome DevTools** on
`127.0.0.1:19222` (`localabstract:chrome_devtools_remote`) + `Runtime.evaluate` (script `probe.py`). On the rendered page:
- tokens **resolve in real Chromium**: `--surface-1 rgba(198,222,255,0.04)`, `--surface-2 …0.07`, `--surface-3 …0.10`,
  `--light-1 …0.14`, `--field-line rgba(206,228,255,0.46)` → planes are **not** transparent (`phoneBg` = surface-1,
  `backdrop-filter: blur(14px)`).
- **live JS motion runs**: 26 animations were active at probe time; **2.5 s later 0 running / 0 infinite** (reveal/one-shot
  only, no loops).
- **JS wiring works**: tap `Approve` → pill changes “Approval needed” → **“Approved — one action, recorded”**; tab click →
  `aria-selected` moves Chat→Tasks and panels switch.
- Comp 02: sheet opens (`open=true`) with the scoped remember text **“Remember for Oakridge move for sending estimates to
  Marcus — one subject, one action. Revocable; the grant appears in the approvals record.”**
- Comp 07: field boundary `--field-line` computed, **3.30:1**; empty state present (“No tasks yet…”); light pool on
  `light-1` + `blur(32px)`.

**Host containment note (recorded):** host port **9222 was already listening — the owner's own Chrome debug endpoint**
(`chromium` pid 2960519). It was **not touched or queried for page content**; the phone-VM devtools was forwarded to
**19222** instead. Teardown removed our forward, killed the emulator, stopped the http server, and the owner's listener
was still there (count 1). Windows disk unchanged (393,216 B). See `teardown.txt`.

## 3. The four S8 named pendings — tonight's disposition (Nox night order §Finish line 2.2)

| Pending | Tonight's disposition | Evidence / blocker |
|---|---|---|
| **R2 model-quality** | **CARRIED — pending-on-roster-activation** with the exact proof pre-written (§4) | Roster not live; no provider route active. S8-030 reason stands (provider credential). |
| **R3 hosted URL / DNS / second device** | **CARRIED as pending (host+package)**; the **second-device transport angle is FIXTURE-PROVEN** on the phone VM | Real shareable URL needs G4 host + Ergon-on-DSH package; neither exists (G1 whole / G4 absent — Nox `20261102T003600Z` §3) |
| **R4 phone surface** | **CARRIED as pending (product surface PACKAGE-PENDING)**; the **phone VM itself and its rendering of a phone-shaped Ergon artefact are REAL/FIXTURE-proven** | `PACKAGE.md` §5 item 6; no phone surface build exists on `main` |
| **R6 voice surface** | **CARRIED as pending (package)** | `REHEARSAL-R0-R7.md` R6 status `blocked`; voice surface is a P20-F5-class package piece; Windows-only surfaces stand-in |

## 4. R2 — the exact proof that will run on roster activation (pre-written)

**Trigger:** the Lead confirms activation of the roster (DeepSeek route) and the Haiku 5.5 comparison route.

**Gate (unchanged):** real acting agent on the DeepSeek route + **one Haiku 5.5 comparison pass over the same chains**;
live vs state-advanced labelled; results read as a floor; the run report names the models used per side; **no fixture
result is reported as live**.

**Procedure (6 cast members from `docs/acceptance/demo-company/`):**
1. `node docs/acceptance/demo-company/fixtures/check-run-config.mjs` → must print `accepted: … route budget-proxy` (the route
   named for the activated roster; record the exact model ids per side).
2. For each cast member (6/6): create **one real agent session** through the activated roster; give the **same prompt set**
   (role question from `CAST.md` + the member's first onboarding turn from `fixtures/data/onboarding.json` + their chain’s
   distortion hop from `SCENARIOS.md`); record `employeeId`, session id, model id, assistant answer, onboarding state.
3. **Haiku 5.5 comparison pass** over the same six prompt sets through the budget proxy; record side-by-side answers.
4. Score: does each employee’s agent know the role and answer in their cast voice; does the Haiku pass reach the same
   answer shape; name every divergence.
5. **Evidence layout:** `board/probes/team-a-vm-e2e/R2-activation/<employeeId>/{prompt.md,deepseek-answer.md,
   haiku-answer.md,session.json}` + `R2-summary.md` (models per side, live vs state-advanced labels, verdict per employee).
6. **Verdict rule:** all six sessions real + both models named + comparison recorded = R2 PASS; any missing session = R2
   named pending, never "clean".

## 5. What Finish Line 2 needs to close (named, not smoothed)

1. **Windows guest installed and reachable** — Lane 4's open blocker (UEFI answer medium never consumed). Not Team A's to
   fix without touching another lane's rig build; once a guest exists, the Windows companion leg is: install the companions
   in the guest, drive the WPF setup/Get Help/ReviewDesk surfaces, capture text-level evidence (UIA trees / logs), never
   screenshots.
2. **The Ergon-on-DSH product package with its phone surface** — G1 whole (Nox `20261102T003600Z` §3): only then is R4 a
   product leg rather than a fixture leg; R3 and R6 ride on the same package + a host.
3. **Roster activation** — for R2 (proof pre-written above).

## 6. Checker handoff

- **Evidence root:** `board/probes/team-a-vm-e2e/` — `adb-boot-state.txt`, `emulator.log`, `browsers.txt`, `home.xml`,
  `settings-before.xml`/`settings-after.xml`, `chrome*.xml`, `devtools-probe-pre-fix.json`, `c1-computed-proof.json`,
  `settle-proof.json`, `p2-proof.json`, `p3-proof.json`, `httpd.log`, `windows-absence.txt`, `teardown.txt`, `probe.py` +
  the `*-expr.js` probes.
- **Reproduce (same commands as this report):** boot as in L1 with the rig env (`ANDROID_SDK_ROOT=$R/sdk`,
  `ANDROID_AVD_HOME=$R/android/avd`, `JAVA_HOME=$R/tools/jdk`), serve the frozen v2 set read-only on `127.0.0.1:8765`,
  forward devtools to a **free** port (not 9222 — the owner's Chrome owns it), then run `probe.py`.
- **Independently check:** (a) Windows absence — qcow2 actual size + no QEMU/monitor/ports; (b) phone boot + tap leg;
  (c) the rendered-page probe (tokens resolved, 0 running/0 infinite after settle, JS interactions); (d) teardown state.
- **Fences held:** no image reads, no root, $0, no owner-facing action, owner's Chrome untouched, rig source files not
  modified (the emulator boot may update AVD bookkeeping files; no rig scripts/ISO/media changed).
- **What this evidence does NOT claim:** that Ergon is end-to-end tested through both VM surfaces (it is not — Windows
  absent, product phone surface absent), or that any design comp is a built surface.

## 7. Side effect for the parallel v2 work (recorded, not a claim)

The phone-VM Chromium was also used as a **text-level computed-style probe** of the frozen v2 comps (real Chromium engine,
no images): it confirms D1-class token resolution, translucent plane backgrounds + blur, the composer boundary at
**3.46:1** after the C1 fix, and live JS/motion with 0 running/0 infinite after settle. That evidence is filed with the v2
report (`board/reports/team-a-design-v2.md`, C1–C4 round); the v2 set is re-frozen separately (`FREEZE.sha256`) and was
served read-only here.
