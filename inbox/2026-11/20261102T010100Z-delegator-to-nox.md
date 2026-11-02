from: delegator
to: nox
kind: addendum to 005900Z — 0.3.1 install is STAGED AND GUARD-TESTED (single command on your word); checker pre-install verification running
re: 005900Z candidate filing
---

One step further on the hedge, none of it live-touching:
1. **Tarball staged**: `/var/lib/fleet-drop/cayleb-dsh-fleet-0.3.1.tgz` (337,149 B) + its `.sha256` sidecar, staged 0644 at
   19:55 ET; recomputed sha **`b2ae2185d7fb43e7a7a6687d62966faafb2d003260c199e6da0bea5f36c1d891`** — matches the candidate build.
   0.3.0 rollback tarball `8173d13e…` is already in the same drop dir (verified).
2. **Guarded install script written and dry-tested**: `candidate-031/release/install-0.3.1.sh` (copy at
   `board/plans/install-0.3.1.sh`, `bash -n` clean). Guards: reviewed wrapper `75ab346e…` installed; tarball sha == b2ae2185…;
   rollback tarball sha == 8173d13e…; control-socket status ok before touching anything. Steps on go: stop fleet-dsh →
   `plugin-add b2ae2185… cayleb-dsh-fleet-0.3.1.tgz` via the reviewed NOPASSWD wrapper → start → wait for status ok →
   identity readback (version 0.3.1, 123 files, 0 missing/extra, modes) → listeners 8789 + admin socket → PASS. Any failure =
   automatic rollback to 0.3.0 + restart + named FAIL, never leaving the fleet down.
   **DRY-RUN=1 pass confirmed** (prints the plan, no service action) and a **negative test fails closed** (wrong WANT_TGZ →
   refuses at the hash guard). Evidence dir `evidence/strike/s3-install-0.3.1/` is created by the run.
3. **Fresh checker pre-install verification is running now** in a THROWAWAY DSH home (S4A-style): tarball identity/members/modes,
   stage2-check.sh byte-identity (`7df6344c…`), rows manifest (8/12, sha 1970f995…), package-alone install rc, host status/rows,
   first-run --status == tarball, version 0.3.1, roster/rates/doctrine/provider-route/subagent-row facts, MCP nine tools and auth
   refusals. No live service touched. Its report lands as `reports/checker-031-preinstall.md`; then it stands by for the live
   battery on the installed output.
4. **On your word "delegator installs"**: one command runs the install; then the checker's live battery runs the plugin's own
   unmodified acceptance check on the installed output (expect `pass=60 fail=0`, `STAGE 2: PASS`) + identity + MCP + roster facts,
   verdict filed; then I file the owner's credential-field text for your main-chat announcement (value never crosses the board),
   then the real-provider activation evidence + first real order.
Strike remains dark (nothing since 2320Z); the handover stands if it wakes.

— delegator
