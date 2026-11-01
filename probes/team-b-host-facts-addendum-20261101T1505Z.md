# TEAM B / LANE 1 — host-facts ADDENDUM after the independent reviewer's HOLD

Filed 2026-11-01T15:05Z by `fleet-lead`. Digest level (verdicts, digests, method). Supersedes nothing; adds to
`board/probes/team-b-host-facts-20261101T1402Z.md`. Reviewer verdict: `board/reports/fleet-checker-identity-v3-review.md`
(HOLD execution; artifact PASS-with-conditions). **The V3 run is NOT invoked. No service action was taken in this
batch.**

## FACT 1 — permitted host route: now proven NEGATIVE for the approved operation (decisive)

Probes run 2026-11-01T14:54:34Z as the operator seat's user (`cayleb`), non-retried, side-effect-free shapes
(no state change: `stat`/`sha256sum /dev/null` reads; `mkdir`/`dd`/`chmod` invoked with no arguments so a
permit would have produced only a usage error):

| Shape (as the V3 procedure invokes it) | `sudo -n` result |
|---|---|
| `/usr/bin/stat` (harmless read) | **`sudo: a password is required`** (rc 1) |
| `/usr/bin/sha256sum /dev/null` (harmless read) | **`sudo: a password is required`** (rc 1) |
| `/usr/bin/mkdir` (no args) | **`sudo: a password is required`** (rc 1) |
| `/usr/bin/dd` (no args) | **`sudo: a password is required`** (rc 1) |
| `/usr/bin/chmod` (no args) | **`sudo: a password is required`** (rc 1) |
| `sudo -n /usr/bin/systemctl status fleet-dsh` | **`sudo: a password is required`** (rc 1) |
| `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin version` | **works** (rc 0, `0.2.2-alpha.2`, NOPASSWD fixed verb) |

**Verdict: the approved V3 operation is UNRUNNABLE under the current approval.** The procedure refuses to run
as root, and it needs bare non-interactive `/usr/bin/sudo` for five fixed root command shapes plus
`sudo --user fleet -- /usr/bin/env …` for the caller. Root password-based sudo is in play; only the fixed
wrapper verbs are NOPASSWD. A password-based caller invocation *after* the stop is exactly the failure mode the
reviewer named (unit down while a human types), and granting the shapes would be a sudoers/polkit change — a
security change not authorised by the approval. **Per the Lead's ruling: not routed around; taken back to the
owner.** The route options (all owner decisions, none taken): (a) owner-authorised sudoers addition for exactly
those five fixed shapes (security change; new approval); (b) operator at a terminal typing up to six prompts
(reviewer's safety analysis says the post-stop caller prompt is the hazard — not recommended); (c) re-scope the
measurement to permitted interfaces only (no such route exists today).

## FACT 3 — layout caveat recorded (reviewer condition 4)

The caller *requires* a non-symlink leaf directory and single-link files; those are **preconditions of
measurability, not measurements**, and **no fact asserts them**. The approved measure cannot pre-check them from
this seat (they are read by the caller after the stop). Therefore: **a post-stop block on a symlink leaf, a mode
mismatch or `nlink != 1` is the EXPECTED fail-closed outcome, not an anomaly** — the attempt would be consumed
and the result read as "the measurement was taken and did not match", never retried. Candidate policy remains
0750 / uid:gid 951:950 / archive-derived modes / single-link, judged by the independent checker as an installer
contract.

## FACT 4 — window receipt hardened (v2, reviewer condition 3) — done and tested

`~/workspace/harness-build/scratch-tb-l1/window-receipt-v2.sh` (v1 preserved untouched):
1. **Fails closed:** if `active_state`, `sub_state`, `n_restarts`, `invocation_id`, `rows_manifest_sha256`,
   `integrity_ok`, the wrapper status read or the drop-area digest is missing/unparseable → prints
   `RECEIPT_INCOMPLETE:<field>`, exits 3, **emits no receipt**. Tested: missing hold declaration, missing
   procedure result, stale/non-fresh procedure invocations each fail closed with a specific reason.
2. **Tied to the procedure's markers:** window-end requires `--procedure-result <v3-result.json>` and accepts
   it only if the procedure's `baseline.InvocationID` differs from `after.InvocationID`, the receipt's own
   invocation equals `after.InvocationID`, and the baseline invocation appears in the enumerated restart events.
3. **Restart accounting:** every `Started` event in the window is enumerated from the unit journal with its
   `_SYSTEMD_INVOCATION_ID`; the rule recorded in the receipt is "every restart attributable to the procedure
   or to Stage 2, none unexplained" — not a fixed +1 count (Stage 2 kills/restarts the unit itself).
4. **Names its own blind spot:** every receipt carries `not_covered`: the compared root's bytes/modes/links
   (unreadable from this seat) and root-side mtime/ctime (protected read, not authorised). It is accompanied by
   a `--hold-declaration <file>` whose **sha256 is embedded**, naming every held writer; the declaration itself
   is the Lead's, hash-referenced in both receipts.
5. Baseline-v2 receipt taken: `2026-11-01T14:51:30Z`, invocation `2ff92d0b…`, `n_restarts=0`,
   rows manifest `1bcac986…`, drop digest `4055db47…`, receipt digest `23a453b9…` (v1 baseline stayed valid).

## FACT 2 — unchanged; Nox's policy ruling is the only path

The reviewer's independent view agrees with the filed one: the in-process correlation binds the loaded
bundle's **declarative layer** (manifest + mounted row field values), **not its code bytes**; the documented
resolution order can select a second 0.2.2 copy before the measured path, and such a copy would hash the same
manifest. Three options are with Nox (accept with narrowed wording + named residual / require the reviewed
in-process witness / narrow the claim to what V3 actually supports). **Team B does not call it "loader origin"
and does not choose the policy.**

## Status

V3: approved, **HOLD**, not invoked. Reviewer's four blocking conditions: FACT 1 now proven **negative** (owner
decision needed — route unrunnable as approved); FACT 3 caveat recorded; FACT 4 hardened and tested; FACT 2 with
Nox. The formal Stage 2 installed-output checker remains a distinct fresh seat and must judge the whole private
result, not an exit code; preserved-stage removal is a separate owner-authorised act; the operator must be
physically present for the whole window.
