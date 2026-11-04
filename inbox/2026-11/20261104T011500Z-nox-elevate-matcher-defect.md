# DEFECT (Nox, 2026-11-04T01:15Z): Tier B confirm can never pass on this host — sudo journal MESSAGE has leading spaces the matcher rejects

Priority: FINISH-critical. Every Tier B confirm fleet-wide is blocked (RSI W9 registration in flight tonight; the relocation close-out owner sheet and Duet G2 also depend on confirms).

## What happened (live, owner present, twice)

- D1 elevate kit installed by the owner from the root-owned copy and PROVEN: /usr/local/libexec/fleet-elevate, 6 entries, root-owned, no setuid; journal line by root: `installed /usr/local/libexec/fleet-elevate manifest_sha256=43b3e12b7f5c54d87685889f2ed2b67856dde3f27f1fe8c5382144ce3c835867 kit_sha256=f364350f640d8ae2840f66cfe2d7e2a25c7d20e85c9a06e82a2a74e316eb56c9` (kit hash matches Lane 6's record). proof.sh p0 ALL PASS.
- Ticket 100022 (comms-register ext:rsi, grant g050059880507) confirm run by the owner: YES typed, secure prompt opened, password entered, prompt closed. Elevation e3 shows `done` in `dsh-fleet elevate list`.
- The confirm CLI then reported: `no confirmation: no sudo journal line for exactly this command; the ticket is still pending`. The follow-up `comms-register` with the grant still returns `ticket-required: g050059880507 waits on the owner`.

## Root cause (verified by Nox from the journal itself)

sudo DID run the exact approved command, as root, at 2026-11-03 20:03:08 local:

```
journalctl -o json -t sudo (that minute):
_EXE=/usr/bin/sudo  _TRANSPORT=syslog  SYSLOG_IDENTIFIER=sudo
MESSAGE="  cayleb : PWD=/ ; USER=root ; COMMAND=/usr/bin/timeout -k 5 10 /usr/bin/true tb:08506ec9a558"
```

Note the TWO LEADING SPACES in MESSAGE. `src/elevate/journal.js` `successRegExp` anchors `^<user> : ...` with no whitespace tolerance, so the genuine line never matches. Everything else matches the design: _EXE, _TRANSPORT, identifier, PWD=/ (= CONFIRM_CWD), timeout 10, exact command. journal.js's own header flags this shape as `[unverified: that sudo logs this shape here; G39, P4, P5]` — on this host it does not.

Likely same class: proof.sh kit's journal check also reported FAIL for the install line (which exists, root-written, exact hash) during the same sitting.

## Fix shape (lane's call, stated for the record)

Tolerate sudo's leading whitespace in the matcher (trim MESSAGE or allow leading spaces in successRegExp) — without weakening the anchors that matter (_EXE / _TRANSPORT / SYSLOG_IDENTIFIER are journald-supplied and unforgeable by uid 1000; the exact command + PWD + time window stay exact). Needs: fix, fresh-checker pass incl. a negative test (a forged uid-1000 line with the same text must still NOT match — the forge proof in the L4A sitting), and a package the owner installs. The owner has done his part twice; do not ask him to re-confirm until the fixed build is installed.

## State left behind

- Ticket 100022 still pending; Tier B approval window runs to 2026-11-05T00:46:45Z. If the window lapses before the fix ships, Nox re-requests (1a) and the owner re-confirms once, on the fixed build.
- RSI sheet steps 2–5 wait on this fix. Nothing else about W9 is blocked.
- D1 is DONE and proven; the L4A sitting's remaining proof steps (confirm + forge) ride the same fix.
