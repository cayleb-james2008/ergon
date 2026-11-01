# Job 03 — driven rehearsal spec (prep only; runs once the package + host exist)

Prepared by Team A, 2026-10-31 ~22:30 ET. This is the step-for-step rehearsal script for Cayleb's actual RHL setup, from
the non-technical operator's view: **fresh host, no developer shortcuts, no pre-seeded state.** It is prep; every step is
marked with what is runnable today and what is blocked (package absent, work-PC board, Windows surfaces, host purchase).
Product shape being rehearsed: apps hosted on ONE machine per company (dedicated cloud host default), everyone uses them
through a shareable non-localhost URL, nobody installs anything to use an app; the personalised agent's home is the
company cloud host; Ergon itself is the installer.

## Rules for the rehearsal (bind every step)

- **Fresh host:** a never-used machine/VM for the pass; no shortcuts from a developer clone, no pre-seeded company, no
  copied state from a previous run. Each pass is one continuous run from an empty host to the close-out.
- **Operator's view:** the actions are the ones the non-technical operator (Cayleb at RHL) actually takes. Developer-only
  commands are not allowed in the pass; team-side provisioning is documented separately and done before the clock starts.
- **Nothing pre-seeded:** if a step needs a company, cast, ticket or data, the rehearsal creates it in the pass.
- **Evidence per step:** timestamped capture of the operator's screen/URL/command output, the expected and actual result,
  and a pass/fail/pending word. No secrets in evidence; no credential value is ever shown or stored in the pack.
- **Cayleb's phone steps are his**, one at a time, each a single mechanical instruction (e.g. "open this link on your
  phone and tap the button"). The team does not touch his phone.
- **Honesty:** a step that cannot run because a piece is missing is recorded `pending` with the missing piece named —
  never `clean`, never skipped silently. Windows-only surfaces are `stand-in` or `pending`, never `clean`.

## The pass (R1–R7) and per-step evidence

**R0 — Pre-pass (team, before the clock).** Fresh host imaged (Ubuntu 24.04, single-tenant CPX22-class), no company data;
the package tarball hash recorded; the company Supabase project created and the connection config placed by the secure
route (no secret shown to the operator); DNS/URL name ready. *Evidence:* host fingerprint, package SHA, absence of company
state, project id. *Blocked today:* package absent, host purchase pending.

**R1 — Fresh host → company setup.** Operator opens the install link on the host's console as instructed; runs the setup
exactly as a first-time user would; creates the company (name, timezone, sites/roles as prompted). *Expected:*
setup completes, company page visible, no developer shell needed. *Evidence:* per-prompt screens, final company screen,
installer log path + tail. *Runnable today:* no (package).

**R2 — First personalised agents.** Operator adds the first employees (real RHL names/roles per the deployment list) and
onboards each one through the product's onboarding conversation; the assistant draft appears and the operator confirms it.
*Expected:* each employee has a working assistant that knows their role; onboarding ends `active`. *Evidence:* one
screenshot/record per employee, assistant answer to a role question, onboarding state. *Runnable today:* partially — the
management piece + onboarding protocol exist on current code, but not in the package shape.

**R3 — Shareable URL from a second device.** Operator opens the **shareable non-localhost URL** from a second device (a
laptop/tablet, not the host's own browser) and signs in with the device approval flow. *Expected:* the same company and
assistant visible from the second device; URL is a real name, not `localhost`/raw IP. *Evidence:* URL bar capture from the
second device, device approval record, page identity. *Blocked today:* hosted-URL package + host.

**R4 — Phone enrolment.** Operator (or Cayleb) enrols a phone: open the same link on the phone, "Add this phone", manager
approves; the phone appears as an approved device. *Expected:* phone listed, assistant answers from the phone while the
desktop is off. *Evidence:* phone screen captures (no personal content), approved-device list, desktop-off test result.
*Cayleb's act:* the phone taps; one instruction at a time.

**R5 — A desk task.** Operator does a real desk task end to end: raise a ticket from an employee device, work it on the
management screen (claim, reply, receipt), and confirm the employee sees the resolution. *Expected:* the ticket moves
through the queue with the right states and the receipt; the employee view updates. *Evidence:* ticket ids/states/receipts,
employee-side screen. *Runnable today:* yes on current code (demo company rehearsal proves the mechanics), not in the
package shape.

**R6 — The voice/message flow.** Operator uses the voice button on a device to dictate a message/ask; the assistant answers;
the voice-control badge appears after the optional in-app setup step. *Expected:* dictation → answer round trip; the badge
reads "Voice control activated"; typing still works. *Evidence:* screen captures, transcript of the two turns (no personal
data), badge state. *Blocked today:* phone/voice surfaces are DSH-era package pieces (P20-F5 class) — pending on current
code.

**R7 — Close-out and second pass.** Record the state (company page, employee list, tickets, URL, host logs), then destroy
the host and **run the whole pass a second time from scratch** (done-bar: "end-to-end tested multiple times"). *Expected:*
second pass completes clean with the same evidence shape. *Evidence:* both passes' packs, diff of results, host teardown
record.

## Evidence pack layout (per pass)

`pass-<n>/R0-pre/ … R7-close/` each with: `screen-*.png`, `notes.md` (operator action → expected → actual → word),
`logs/` snippets, `url.txt`, `host-fingerprint.txt`; a top `index.md` with the pass verdict per step and the named
blockers. Secrets: none. The pack goes to the fresh-context checker, who re-runs the acceptance battery against the final
package — a rehearsal nobody drove does not count.

## Current status of the pass (31 Oct, pre-package)

| Step | Status today | Blocked on |
|---|---|---|
| R0 pre-pass | blocked | package absent; host purchase |
| R1 company setup | blocked | package absent; host |
| R2 first agents | partially runnable on current code (not package shape) | package |
| R3 shareable URL | blocked | package + host |
| R4 phone enrolment | blocked | package (phone surface) + host |
| R5 desk task | runnable on current code | package for the real pass |
| R6 voice/message | blocked | package (phone/voice surface); Windows-only surfaces stand-in |
| R7 second pass | blocked | the above |

When the Lead/Nox confirm package ownership and the owner buys the host, this spec becomes the run sheet for W4. The board
fold-in (R2/R5 context) waits on the `RHLCO-WKS-09` window; Windows-only steps stay stand-in/pending until a Windows seat
is available.
