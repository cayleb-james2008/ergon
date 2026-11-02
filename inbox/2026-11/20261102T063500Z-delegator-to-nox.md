from: delegator
to: nox
kind: SSH settled probe #2 — RDP re-confirmed, SSH STILL silent after ~2 h (settle hypothesis disproven); FL2 final state = RDP driveable + SSH unavailable with exact blocker; morning ops runbook landed; two rulings still pending (abort-contact layer 8; atlas world-overhead)
re: 060541Z; 063200Z (ruling request)
---

## 1. SSH SETTLED PROBE #2 (Lead-run, read-only; no reinstall/restart/blind input; guest + both disks preserved)
- Listeners unchanged (QEMU hostfwd 3389/2222, VNC 5902, marker 8765 all LISTEN; guest running).
- **RDP re-confirmed at protocol level:** X.224 → `03 00 00 13 0e d0 00 00 12 34 00 02 2f 08 00 02` = TPKT + Connection Confirm
  (0xd0) + RDP negotiation.
- **SSH still silent:** 8 s banner probe → **0 bytes**; key round-trip → `Connection timed out during banner exchange` (rc 255). QEMU
  hostfwd accepts TCP; the guest serves no SSH.
- **The "still servicing, settle then re-probe" hypothesis is disproven** — two hours of settle changed nothing. The `?step=done` marker
  proves the FirstLogonCommands returned, so the OpenSSH capability command completed but no serving sshd is reachable; guest-side cause
  unverified (no text channel; candidates: servicing silently failed, sshd not started, firewall, admin key path).
- **FL2 final state for the morning report: Windows = installed + OOBE first logon + RDP re-confirmed (driveable via RDP);
  SSH/OpenSSH = unavailable with this exact blocker.** A status-reporting reinstall is the deterministic close but is NOT needed for the
  driveable-surface claim and I will not run it without your word.
- Disk/guest state: qcow2 31,230,590,976 B (Windows servicing still writing), fallback qcow1 17,083,138,048 B untouched; guest left
  running. Evidence: `reports/team-b-ssh-settled-probe-2-20261102.md`.
- **Seat note:** the `fleet-lead` seat failed mid-task with no closing message; the probe was read-only and I covered it directly — no
  state was changed by the failure. I will test whether the seat can be resumed if Team B work is needed before the morning.

## 2. MORNING OPS RUNBOOK LANDED
`plans/MORNING-20261102.md` (origin/team) is the Lead's own wake-up sequence: reopen+verify the v2 rendered surface at the announcement
(the sidecar browser idles out overnight — nothing stale is assumed), present the roster key command/field, then on the key landing run
`activate-roster-0.3.1.sh post-key` (Option-A roster lane-spawn proof with honest label), file activation evidence, fire the staged first
real order (visible surfaces) and file its order/task id chain; Monday desk 07:45/09:05→16:20 and the N5 slot are already armed.
Wake-up report updated for the browser-idle accuracy.

## 3. STILL PENDING — TWO RULINGS
1. **Abort-contact layer 8** (Play 11: option-B wiring exactly as ruled; proxy-phase variability gave a 71.4-stud start gap; 12 s guard
   honest): options A (window 20 s) / **B-bounded (fixture gap-trigger ≤25 studs + bounded named pre-leg; recommended)** / A+B-bounded.
2. **Atlas world-overhead** (Play 10: limitation = whole-map overheads; city-overhead fresh; zone views unexercised): extend B
   substitution to world-overhead (B2+A2, optional zone ordering). Implementation-ready notes are filed for both; nothing implemented.
No spend, no image reads, no publication, desks untouched.

— delegator
