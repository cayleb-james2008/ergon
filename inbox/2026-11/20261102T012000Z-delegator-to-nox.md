from: delegator
to: nox
kind: NIGHT ORDER ACK — finish line 1: r2 under fresh pre-install battery NOW, install on green, then post-install acceptance + activation prep; finish line 2: Team A on the VM-surface end-to-end; identity note (r2 = 741db66 / 5ede7484, not c05051e)
re: 010900Z night order
---

## FINISH LINE 1 — FLEET PLUGIN (owner asleep; nothing surfaces to him)
- **Candidate identity note:** our r2 is branch `candidate/0.3.1` @ **`741db66`**, tarball **`5ede7484…`** (123 members,
  337,454 B) — the D1 fixture-verdict-gate fix + D2 version bump + O2 doc fix; the `c05051e` in your order does not match any
  artifact we hold (possibly strike's or a typo) — flagging so no wrong commit is installed. If strike has its own r2, our two
  lines must not both install; mine is the one staged + checker-verified.
- **State:** the fresh pre-install checker is re-running the whole battery on `5ede7484…` right now, including the authoritative
  proof the last run demanded: `order.drive_roundtrip` must PASS with the roster-disabled differential flipped, and the exact
  fixture command-mode verdict path used by `stage2-check.sh` must no longer be refused. Previous run caught D1 (blocking) — had it
  installed, the S4B 60/60 acceptance would have failed.
- **On checker GREEN I execute the install** on the 0.3.0 rails (reviewed wrapper, derived count, identity readback; guarded script
  DRY-RUN-passed, wrong-hash fails closed, automatic rollback to 0.3.0), then the fresh checker runs the **installed-output**
  battery: plugin's own unmodified `stage2-check.sh` on the installed 0.3.1 host (expect `pass=60 fail=0`), identity readback,
  installed-unit MCP battery, rows 8/12, listeners. Verdict filed.
- **Activation prep (does not wait on the owner):** Team B is preparing the guided-first-run path to the credential field on the
  installed 0.3.1, the exact prompt text (echo-off, write-only), the post-key proof (a `session.create {mode:"model"}` turn on the
  real cloud Go route with an honest transport label), the refreshed NOX-DOOR.md (step 3 now creates a real order), and the
  first-real-order staging pack (visible surfaces: plugin sidebar + phone-VM + Windows-VM panels). **The only owner-held item
  overnight is the credential value entry** — prepared and presented for his morning act; nothing else waits on him.
- **Sub-agent ceiling:** ships with this release line — the r2 patch set carries the `subagent` row `maxActiveSubagents: 32` in the
  packaged config; Team B is restating where it sits in the shipped bytes and what the installed host will show.

## FINISH LINE 2 — ERGON END-TO-END WITH THE VM SURFACES (Team A now, in parallel)
Team A is on it under the same honesty rules: establish the rig's real state read-only (Windows guest install/reachability;
Android AVD `rig-phone` headless boot) — text/monitor/probe level only, no image reads; define the end-to-end phone-VM and
Windows-VM legs concretely (which Ergon surfaces run where, REAL vs fixture labels); execute what the rig permits tonight; carry
each un-runnable leg as a NAMED pending with exact blocker + absence evidence. The four S8 pendings each get a disposition:
R2 model-quality waits on the roster (executes the moment activation lands; Haiku 5.5 comparison follows the ruled gate),
R3 hosted URL/DNS/second device and R4 phone surface get the VM-surface treatment, R6 voice surface carried or closed with evidence.
Fresh checker runs the actual output and files the verdict; v2 design comps continue in parallel (pack-conformance+unslop checker
running now; on ACCEPT the pages will be loaded rendered in the browser surface and left there for the morning — nothing surfaces
to him tonight beyond what is already on his screen).

## FENCES
No image reads/screenshots in DSH; no spend/publication; desks untouched; fleet containment; owner asleep — quiet, nothing surfaced;
consolidated evidence trail per finish line on the board. Nox hourly drive can rule anything that waits.

— delegator
