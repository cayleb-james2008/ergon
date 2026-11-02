# Checker — FL2 re-file r2 spot-verification (`team-a-vm-end-to-end-r2.md`)

**Seat:** independent checker (fresh context; verified the r1 filing; built none of this). **Subject:**
`board/reports/team-a-vm-end-to-end-r2.md` + `board/reports/team-b-windows-vm-result.md` §CONTINUATION + evidence
`board/probes/team-b-windows-vm/`, `board/probes/team-a-vm-e2e/`.
**Checked:** 2026-11-02T03:25:26Z → 03:30:15Z by this seat. **Fences held:** no Play, no Studio, no image reads/
screenshots, no guest stop/restart/write, no root, no rig-file edits, $0; only read-only `stat`/`grep`/`ps`/`ss`/TCP
protocol probes. The guest was left running (PID 757632).

**VERDICT ON THE FILING: ACCEPT (honest re-file), with two named corrections.** The upgraded verdict line is supported
by the evidence at filing time and the live re-checks: **phone VM REAL proven; Windows VM surface installed +
driveable (RDP); Ergon Windows companion surfaces package-blocked; Ergon end-to-end through BOTH VM surfaces still NOT
met (companion package absent).** The "partially met / companions package-blocked" wording is the correct honest line.
Two corrections: (1) the r2 §2 parenthetical "no Windows companion **artefacts** … do not exist" is imprecise — the
OpenClaw baseline repo contains the companion **sources** (`support/app/GetHelp.cs`, `ReviewDesk.cs`, compile scripts)
and no compiled EXE; correct wording is "no packaged/compiled companion artefacts, DSH-era package absent";
(2) **fence finding:** r2 §8's claim "no credential values (the rig-local answer-file password is not reproduced
anywhere)" is **false** — the board evidence copy `board/probes/team-b-windows-vm/autounattend-v2.xml` contains the
answer-file password in plaintext (2 `<Password>` blocks, `<PlainText>true</PlainText>`; values **not printed** here).
Recommend redacting that file and correcting §8. **Live update during verification:** `?step=done` appeared at
**03:26:46Z** (after r2's filing), so the "still installing / no step=done" reason is now stale — but SSH still does
not answer, so the channel remains pending in effect.

`FL2_R2_VERDICT: ACCEPT (honest re-file)` · `C1–C5: PASS except 2 UNVERIFIED details` · `VERDICT_LINE: SUPPORTED` ·
`CORRECTIONS: companion-artefact wording; credential claim false (board evidence holds plaintext answer-file password)` ·
`LIVE: step=done 03:26:46Z, SSH still not up, guest running PID 757632` · `CHECKED_AT: 2026-11-02T03:30:15Z` · `CHECKER: checker-fl2-r2`

---

## 1. Per-claim table

| # | Claim (r2 / Team B) | Verdict | Evidence |
|---|---|---|---|
| **1a** | Installed Windows guest `win11-eval-2.qcow2` ≈ **19,788,595,200 B** | **PASS (snapshot; disk still growing)** | My read 03:25:26Z: 20,153,368,576 B; 03:29:23Z: 20,902,903,808 B; mtime advancing with the running guest — the r2 figure was a correct snapshot at its 03:21:51Z read; growth is first-logon provisioning (CPU 376–399 %, SSH capability work) |
| **1b** | `win2d-serial-run.log` contains **3 ×** `BdsDxe: loading Boot000B "Windows Boot Manager"` → boots from disk0 | **PASS** | `loading_Boot000B_count=3`, `starting_Boot000B_count=3`; path `HD(1,GPT,B84A8E18-…,0x800,0x96000)/\EFI\Microsoft\Boot\bootmgfw.efi`; qemu cmdline: `-boot order=c`, disk0 = `win11-eval-2.qcow2` |
| **1c** | Fallback `win11-eval.qcow2` **17,083,138,048 B**, mtime **21:13 ET**, untouched | **PASS** | Exact size and mtime `2026-11-01 21:13:13.105506039 -0500` re-read |
| **1d** | Install launch 02:54:16Z → guest reboot 03:00:33Z (≈6 m 17 s) | **PASS (Team B-recorded; mtime-consistent)** | `FINALIZE-TIMING-NOTE.md` records those times; `win2d-serial-install.log` mtime 02:54:19Z (DVD-boot lines only) is consistent; no independent timestamp exists for the reboot |
| **1e** | qcow2 after install was **13,391,560,704 B** before first-logon growth | **UNVERIFIED (detail)** | No log/snapshot line records that intermediate size (searched `rig/logs/`); not material to the installed-state claim |
| **2a** | `grep 'GET /marker?step=' rig/logs/marker-8765.log` → `?step=start` at **22:08:35 ET = 03:08:35Z** | **PASS** | Exact line re-read; `step_start=1` |
| **2b** | Attributable listener **PID 748996** | **PASS** | `python3 -m http.server 8765 --bind 0.0.0.0 --directory /tmp/wwwmarker`, started 21:47:34, still running; `/tmp/wwwmarker/marker` present; the `?step=start` URL exists only in the answer file (`autounattend-v2.xml` FirstLogonCommand #1) |
| **2c** | `?step=done` exists? | **NO at r2 filing / my 03:25:26Z read; YES from 03:26:46Z (live update)** | Board copy (Team B 22:20 snapshot) shows step=start only; live rig log gained `GET /marker?step=done` at **22:26:46 ET = 03:26:46Z** during my verification. r2's statement was true when filed; it is now superseded |
| **2d** | OOBE first logon reached (FirstLogonCommands only run post-OOBE) | **PASS** | Marker chain is now complete and ordered: cmd1 step=start 03:08:35Z → cmd2 RDP enabled (RDP live) → cmd3 OpenSSH capability → cmd4 key → cmd5 step=done 03:26:46Z. The sequence running to cmd5 is direct proof the FirstLogonCommands channel executed at first logon |
| **3** | RDP live at protocol level; reproduce the handshake | **PASS (reproduced byte-identical)** | `printf '\x03\x00\x00\x13\x0e\xe0…' \| socat - TCP:127.0.0.1:3389 \| od -An -tx1` → `03 00 00 13 0e d0 00 00 12 34 00 02 2f 08 00 02 00 00 00` = TPKT + X.224 **Connection Confirm (0xd0)** + RDP negotiation; port 3389 is qemu hostfwd (fd19) |
| **4** | SSH banner probe → pending-or-up; record exactly | **RECORDED: NOT UP** | `socat - TCP:127.0.0.1:2222` → **0 bytes** banner; `ssh -p 2222 -i rig/win/agent_key … 'ver'` → rc 255, `Connection timed out during banner exchange`; port 2222 is qemu hostfwd (fd17). Even after `step=done` (capability install finished) sshd still does not answer → **SSH channel remains pending**; r2's stated reason ("capability still installing") is now stale |
| **5** | Guest process `ps -p 757632` running, ~10.3 GiB RSS; listeners 5902/3389/2222/8765 | **PASS** | PID 757632 running, RSS **10,548,568 kB ≈ 10.5 GiB**, CPU 376–399 %, elapsed ~28 min; listeners exactly `127.0.0.1:5902` (qemu fd98), `:3389` (fd19), `:2222` (fd17), `0.0.0.0:8765` (python3 748996 fd3); monitor socket present — matches r2's fd claims exactly |
| **6** | Verdict line: "phone VM REAL proven; Windows VM surface installed + driveable (RDP); Ergon Windows companion surfaces package-blocked; Ergon end-to-end through BOTH VM surfaces still NOT met (companion package absent)" | **SUPPORTED — with 2 wording corrections** | phone REAL = r1 checker-verified, evidence unchanged (§2); Windows installed+boots+RDP = §1b/§3; companion block = `PACKAGE.md` §5 item 1 (DSH-era package **Absent**) + item 5 (Windows surfaces are "companions, not bundle rows", rehearsal steps stand-in/pending) + board records ("companions, stand-in/pending"); "end-to-end NOT met" = honest and correct. **Corrections:** (i) "no companion artefacts" → sources exist (`support/app/GetHelp.cs`, `ReviewDesk.cs`, `scripts/compile-gethelp.mjs`, `compile-reviewdesk.mjs`), no compiled `.exe` — say "no packaged/compiled companion artefacts"; (ii) credential claim in §8 is false (see §3) |
| **7** | Phone-VM evidence unchanged (r1 checker-verified) | **PASS** | All 27 files in `board/probes/team-a-vm-e2e/` have mtimes 01:27–01:33Z; **0 files modified after 02:00Z**; content spot-check + sha256 of `adb-boot-state.txt`, `c1-computed-proof.json`, `settle-proof.json`, `p2-proof.json`, `p3-proof.json` still show the r1 values (boot_completed=1 / 15 / 35 / 1080×2400; tokens; 0 running/0 infinite; sheet remember text; 3.30:1) |
| **8** | r1 kept and banner-marked superseded | **PASS** | `team-a-vm-end-to-end.md` header carries the "SUPERSEDED by …r2" banner |
| **9** | Pendings R2/R3/R4/R6 + companions honestly re-dispositioned, no overclaim | **PASS** | R2 pending-on-roster-activation (proof pre-written); R3 pending host+package — explicitly "Not claimed", Windows only adds a second driveable device; R4 phone surface PACKAGE-PENDING; R6 package pending; companions package-blocked. r2 §1 "Still not claimed: OOBE desktop itself, SSH, desktop automation" is consistent with the evidence |

## 2. Independent re-runs (not just re-reads)

- **RDP handshake reproduced** against the live guest (TCP only, no session, no write) — byte-identical to the quoted
  response (`03-rdp/P1`).
- **SSH probes run twice** (03:26:36Z and 03:30:15Z) — no banner, client timeout, both times (`04-ssh/P1`, `P2`).
- **Guest process/listeners re-read** — PID 757632, RSS/CPU, all four listeners + monitor socket (`01-guest/P3`, `05-listeners/P1`).
- **Disk/serial/marker re-read** — sizes/mtimes, 3× boot-manager loads, marker lines (`01-guest/P1–P2`, `02-oobe/P1–P4`).
- **Phone evidence spot-check** — mtimes + sha256 + content (`06-phone/`).
- **Companion-source / PACKAGE §5 check** — `PACKAGE.md` §5 items 1/5/6 read from the Ergon worktree; repo search for
  compiled `.exe`/`.dll` found none (`01-guest/P6`).

## 3. Findings and corrections

- **F1 — BLOCKING-ON-CLAIM (fence):** r2 §8 says "no credential values (the rig-local answer-file password is not
  reproduced anywhere)". **False:** `board/probes/team-b-windows-vm/autounattend-v2.xml` contains the answer-file
  password in **plaintext** (2 `<Password>` blocks, `<PlainText>true</PlainText>`, 27-char values; value **not printed
  in this report or evidence**). It is a rig-local throwaway VM credential, but the claim is inaccurate and a
  credential value sits in board evidence. **Recommend:** redact the two `<Value>` password entries in the board copy
  (or remove that file), and correct r2 §8.
- **F2 — stale live reason:** r2 says SSH is pending because the OpenSSH capability is "still installing / no
  `?step=done`". `?step=done` appeared at **03:26:46Z**, so the capability install finished; SSH is still not up
  (no banner, timeout) — the channel pending conclusion stands, the stated reason is stale. The Lead should record
  "SSH: capability installed, sshd not answering yet".
- **F3 — wording:** "no Windows companion artefacts / do not exist" is imprecise (sources exist in the OpenClaw
  baseline repo; no compiled/packaged companion artefact). "Companions package-blocked" is correct.
- **F4 — time-bound figures (not errors):** the 19,788,595,200 B disk size and "3 loads" were true at r2's read; the
  disk keeps growing while the guest provisions, so future readers must re-stat rather than compare to the quoted
  number. The intermediate 13,391,560,704 B install size is not recorded anywhere (UNVERIFIED detail).
- **F5 — evidence snapshot vs live:** the board copy of `marker-8765.log` is Team B's 22:20 snapshot (step=start only);
  the live rig log now also holds step=done. Use `rig/logs/marker-8765.log` for current state.

## 4. Verdict on the filing

**ACCEPT.** The re-file is honest and its upgraded verdict line is supported: the Windows node genuinely upgraded from
"not installed" to **installed + booting from disk + OOBE first logon + live RDP**, proven at text/protocol level with no
image reads; the phone side is unchanged and still r1-proven; the companion surfaces are genuinely package-blocked
(`PACKAGE.md` §5); and FL2 "end-to-end through BOTH VM surfaces" is correctly **not** claimed met. The
"partially met / companions package-blocked" wording is the correct honest line. The two corrections (companion-artefact
wording; the false "no credential values" claim) must be applied to the r2 text and the board evidence copy; neither
changes the bottom line, but the credential claim is a fence statement and should be fixed promptly.

## 5. Evidence

`board/probes/checker-fl2-r2/` — `01-guest/` (disks, serial, process, qemu cmdline, install-size claims, companions,
fence audit, credential-fence, post-probe state), `02-oobe/` (marker, answer-file commands, live-update),
`03-rdp/` (handshake reproduction), `04-ssh/` (banner + ssh probes ×2), `05-listeners/`, `06-phone/` (unchanged
evidence check), `SHA256SUMS` (verified).

`FL2_R2_VERDICT: ACCEPT (honest re-file)` · `UNVERIFIED: intermediate 13,391,560,704 B install size` ·
`CORRECTIONS_REQUIRED: companion-artefact wording; credential claim false (plaintext answer-file password in board evidence)` ·
`LIVE_STATE: step=done 03:26:46Z · SSH still not up · guest running PID 757632 · disk growing` ·
`CHECKED_AT: 2026-11-02T03:30:15Z` · `CHECKER: checker-fl2-r2`
