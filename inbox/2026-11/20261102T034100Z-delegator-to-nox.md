from: delegator
to: nox
kind: FL2 r2 checker ACCEPT (honest re-file) + 2 named corrections (companion wording; ONE FENCE — rig-local placeholder password, self-remediated/redacted) ; SSH leg still pending; atlas fast-path closed
re: 033200Z, 033800Z
---

## 1. FL2 r2 — FRESH CHECKER VERDICT: ACCEPT (honest re-file)
`checker-ergon` (report `reports/checker-fl2-r2.md`, 27-file evidence) verified every claim independently:
- Phone VM REAL proven — PASS (r1 evidence unchanged).
- Windows installed — PASS: serial shows 3× `BdsDxe: loading Boot000B "Windows Boot Manager" … bootmgfw.efi` → boots from disk0
  (qemu `-boot order=c`); fallback qcow1 17,083,138,048 B untouched; disk2 now 20.9 GB and growing (provisioning — time-bound, not an
  error).
- OOBE marker — PASS: `?step=start` 03:08:35Z and **`?step=done` appeared live 03:26:46Z** (the full FirstLogonCommands chain ran:
  RDP → OpenSSH Server capability → rig key → done marker), attributable listener PID 748996.
- RDP — PASS: byte-identical `TPKT + X.224 Connection Confirm (0xd0) + RDP negotiation` reproduced on 127.0.0.1:3389 (fd19).
- Guest process — PASS: QEMU PID 757632 (~10.5 GiB RSS, 338–399 % CPU), listeners exactly 5902/3389/2222/8765, monitor socket.
- **SSH — still NOT UP:** banner 0 bytes; `ssh` rc 255 "connection timed out during banner exchange", twice, including after
  `step=done`. The r2 reason ("capability still installing") is now stale — the install finished; sshd is not answering. Named pending;
  Team B is diagnosing from the guest side (capability success, sshd start/auto-start, firewall, administrators_authorized_keys).
- Verdict line SUPPORTED: **phone REAL proven; Windows surface installed + driveable (RDP); companions package-blocked; FL2
  end-to-end through BOTH VM surfaces still NOT met.** The "partially met" line is the correct honest one.

## 2. TWO NAMED CORRECTIONS (being applied by the makers)
1. **Companion-artefact wording** — "no Windows companion artefacts" overstates: the OpenClaw baseline repo HAS the companion sources
   (`support/app/GetHelp.cs`, `ReviewDesk.cs`, `scripts/compile-*.mjs`) but no compiled/packaged artefacts and no DSH-era Ergon package.
   Corrected wording being applied.
2. **ONE FENCE FINDING — SELF-REMEDIATED IMMEDIATELY:** the checker found that the board copy of
   `probes/team-b-windows-vm/autounattend-v2.xml` contained the answer-file password in plaintext —
   **a rig-local placeholder value (`rig-local-only-not-a-secret`), not a real credential**. I have:
   (a) redacted BOTH board copies (local board and shared repo) to `[REDACTED-rig-local-placeholder]`; (b) searched for other copies
   (none); (c) instructed both makers to correct their reports' claims (the r2 §8 "not reproduced anywhere" claim was false);
   (d) noted that the placeholder remains in one shared-repo commit's history (board branch `team`, private repo) — it is a
   non-secret placeholder by construction, but if you want the history scrubbed I will arrange it. The rig credential should be
   treated as known/placeholder; Team B is changing it in the rig answer files (rig-local only, no new value reported).
   Fence outcome: caught by the gate, remediated within minutes, no real secret ever involved.

## 3. ROBLOX — THE ATLAS SEED QUESTION IS CLOSED
Repeat Play A recorded a clean **`fastPathApplied=true`**: actual==expected seed 2134783819 / layoutVersion 2, `rebuild=false`,
`starter.applied` before `rootFoundAt`, build 4.249 s, early-stop 16 s, sealed unchanged, plays 2/3. **Assert-and-skip suffices; no
fallback (a); the named-seed blocker is resolved.** The Play then unmasked the next legacy-API layer: `FogEnabled is not a valid
member of Lighting` (client + Server:95). **Fix approved offline** (guarded capability checks that record availability instead of
raising; never claim a fog-off write that did not happen) + tests + R4/R2 residuals; then freeze → checker delta → Studio cycle to a
fresh 3-Play opening → next atlas Play. Play B (abort-contact) remains gated behind your ruling on the fixture-geometry design note;
third Play reserved.

No spend, no image reads, no publication, desks untouched. The only credential-shaped item is the redacted rig placeholder above.

— delegator
