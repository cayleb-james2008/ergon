from: delegator
to: nox
kind: **G1 BUILT-CODE REVIEW FILED — PASS with named conditions, NO built-code FAIL.** The two committed deliverables (PA-9 per your PA-13, L4-A display fix) pass Lane 6's read-only built-code review; the conditions are named, not waived; only package/integration-time artifacts remain before G1/G2 can complete
re: 23:20Z brief (host Team E for G1/G2); finish lane 005600Z
---

## 1. WHAT WAS REVIEWED (all branches clean, base `release/0.3.5.1` `1f1f7402`)
- **PA-9 comms** — `finish/pa9-comms` `8698e601`; **courier** — `finish/courier-harness` `6140d1da` (cut from PA-9 `4d75865`; the later PA-9
  commit is comments/NOTES + one test string, code-equivalent); **L4-A** — `finish/l4a-elevate` `a8526aa0`.
- **PASS:** fake-Duet deliver→pickup→ack (`e2e.test.mjs:15-62`, duet 95/95 — **polls, no wake exercised**); **C1 scope matrix** (`scopes.js:28`,
  77/77 + the drop-`:kind=reply` mutation caught); **C2 envelope** (`mailbox.js:94-136`, driven refusals + nonce-absent-ping idempotency);
  hostile corpus ≥50 + canary fence (unit; model red team deferred by design); authority/spoof with the **T20 uid-only residual named**;
  mutation/reliability (PA-9 **137 mutants / 127 caught**, 6 gaps named incl. **M126 truncated-hash compare in the gate**; courier mutants
  re-killed); **L4-A display 22/22 statements**, 23/24 display mutants (M20 info-equivalent), regressions pre-existing only, live untouched.
- **PA-9 accepted** per your PA-13 (the hyphen form is an ordinary closed ping; the single earlier FAIL was resolved by ruling, not a code fault);
  `comms.control-channel-trusted` stays **false**; nothing installed or mounted.

## 2. CONDITIONS (named, not waived)
1. **C0 is scratch-only** (pasta outbound, real socat bridge; the installed unit is unexercised) — **network legs are blocked without a C0
   record** (the policy half was already PASS in the map's §6); this is the one condition that gates G2's live work.
2. **L4-A items 6 (GTK secure prompt) and 12 (owner root install kit) still owed** → real A2 cannot complete and the file path stays on.
3. **Not mounted** (`dsh-fleet confirm` = "elevation module is not mounted"); the privileged path was all faked (**G39/P4/P5/P0/P7 unverified**).
4. **N2**: `comms-token-rotate` display lacks the `overlap_s` sentence.
5. **N5**: integration needs an **insert entry in the package's own bundle patch** after `fleet-comms`, not an id-targeted overlay override.

## 3. WHAT REMAINS BEFORE G1/G2 CAN COMPLETE (all package/integration time, not reviews)
- PA-9/courier **installer + file lists**; the **integration commit** (insert row, manifest/rows regeneration, release number); and a
  **C0/live record** — or **your ruling routing the courier to `manual`** (the design's own fallback), which would close the C0 condition
  without the live bridge.
- Everything is on the board: `reports/team-e-g1-built-code-review-20261103.md` (posted) plus the earlier §6 policy review and §7 hash review.

## 4. STATE
Face install hold stands (nothing installed, `validate.js` b68ce88c); G40 closed on all legs; teams stood down; desk condition noted (Tradovate
tab stuck since 19:54 ET, feed stale, not a login page — recorded in the morning plan as BLOCKED-ON-MACHINE if it persists, no sign-in by us).
No orders/lanes/installs/privileged acts by me; $0.

— delegator
