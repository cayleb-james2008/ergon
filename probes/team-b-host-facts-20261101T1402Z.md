# TEAM B / LANE 1 — the four host facts (digest level)

Filed 2026-11-01T14:02Z (09:02 ET) by `fleet-lead` (Team B, Lane 1). For Nox via the Lead.
Method: **read-only permitted routes only.** No service action, no stop/start, no protected-tree read, no
privilege probe, no sudo escalation requested, no retry of any previously denied operation. Raw outputs stay
in Team B's private scratch (`~/workspace/harness-build/scratch-tb-l1/raw/`, mode 0600) — this file is
verdict + digests + method only, per the 04:20Z digest-only ruling.

## FACT 1 — permitted host route

**Read route: ESTABLISHED (proven by running it).** Two documented read-only routes, both exit 0, no
authentication prompt:
- (a) the fixed admin wrapper's `version` / `status` / `dump-config` verbs, run as the fleet principal through
  the reviewed sudoers NOPASSWD grant — `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin <verb>`;
  wrapper installed bytes = reviewed source digest `f84075cf1ab0f1df71bcbf8dc2af3bb883e60f432d4c8d2478db9def0db3be91`;
  `status` output digest `dfc42801b81b0ef5387bd383627597869c1eb3c861a362e5fb8341a58ee269c1`.
- (b) the fleet control-socket status call through the accepted `cf93107` CLI as the owner uid (peer-cred
  admitted), reply `ok=true`. Method: `node <cf93107>/package/bin/dsh-fleet.mjs status`.

**Execution route for the approved v3 identity operation: IDENTIFIED, NOT HELD BY TEAM B, NOT PROVEN FROM THIS
SEAT.** The reviewed operation needs (i) fixed root-controlled staging, (ii) a fixed read as the fleet
principal, and (iii) exact-unit stop/start. The reviewed sudoers source grants only the fixed wrapper verbs —
it does not grant the root stage commands or an arbitrary fleet-principal caller. This session's approval
prompts are disabled, so an escalated exec is auto-rejected and Team B must not request one. The documented
route in the preserved record is **one approval-reviewed escalated exec of the exact hash-pinned bootstrap**
(owner-present if authentication is required; the owner's OS authentication and the stop/start risk are
Cayleb's acts, arranged through Nox). Verdict: the route exists at the Lead / owner-controlled operator
level, not at Team B lane level. No route was invented.

## FACT 2 — actual loader origin

**Verdict: PARTIAL — in-process content correlation ESTABLISHED; canonical directory/inode binding NOT
ESTABLISHED.**

- At the status call the running `fleet-core` reported `integrity.ok=true`, rows `pinned=7`, `disabled=12`,
  `rows.manifest_sha256 = 1bcac9862a9b6c9723c27b193b3a9f22ab9b07f9d30e7fef862d01d194d59ee9`.
- That digest is the sha256 of `package/rows.manifest.json` **inside the accepted 0.2.2 archive**
  (`37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177`, 82 members). The verified 0.2.1
  archive's manifest is a different digest (`fdcd2b72387f5261671625440708a8b96ce83ad503334324d34cee31f480ce4f`).
- Mechanism (this is the new part): the candidate source defaults the manifest path to
  `new URL('../../rows.manifest.json', import.meta.url)` — i.e. **relative to the loaded module's own
  URL** — and only a composed-config `rowsManifest` override could move it. The composed fleet-profile
  config, read through the permitted `dump-config` verb, sets **no `rowsManifest` override**. Therefore the
  bytes the running process hashed were read from its own loaded package root, and they are the 0.2.2 bytes.
- **Remaining gap (named, not smoothed):** the exact canonical directory string / device+inode of that root,
  and coverage of later dynamic loads, are not exposed by any existing supported interface (the wrapper
  `status`/`dump-config` and the control-socket `status` do not carry it; the DSH host inventory returns
  specifier/state/version, not the selected directory). Closing it needs either the separately reviewed
  in-process witness insertion (a new owner decision + product change; not ready to run) or an explicit
  checker ruling that this in-process manifest correlation plus the approved caller's candidate-bound
  82-file comparison is sufficient binding. **Team B does not choose that policy — it names it.**
- Corroboration: `NRestarts=0` and one invocation since the install start (uptime consistent with the
  recorded 03:46:27Z restart); no profile/config write since install.

## FACT 3 — installed mode/ownership

**Verdict: TOP-LEVEL ESTABLISHED; full-tree modes/links/ownership are the approved caller's measurement
(explicitly not an entry condition per the 05:36Z correction).**

- Fixed wrapper `status` verb (permitted route): service state directory `0750`, owner the fleet principal
  (resolved `uid:gid = 951:950`); DSH profile directory `0750` same owner; static account resolution
  `951:950`. Unit declares its state directory mode `0750`.
- Installed unit file byte-identical to accepted `cf93107` source (`b44c05d082410741c43e5f6cce941911a030946d084f7c3efdacdaa3045efe15`);
  installed wrapper byte-identical to accepted source (`f84075cf…`); accepted archive on disk matches
  `37e3a542…` and holds 82 members; cf93107 Stage 2 script `7df6344c…`, CLI shim `8e121b19…` re-verified.
- Not established here: the 82 members' individual modes, single-link layout and ownership at the installed
  root. The caller's policy (0750 / 951:950 / archive-derived modes / single-link) is the fail-closed
  candidate check the independent checker must assess as an installer contract.

## FACT 4 — no-writer window

**Verdict: WINDOW DEFINABLE AND CURRENTLY QUIET; the receipt is declarative + measurable, not enforceable by
Team B alone.**

- Writer inventory for the compared root: (i) owner/root through the fixed wrapper verbs
  (`plugin-add`/`plugin-remove`/`integrity-drill`/`restore-live`) — the only sanctioned install/profile
  writers; (ii) the fleet service itself (mounts its profile; writes its DB/state, not the package root);
  (iii) root/manual edits; (iv) the package manager, invoked only by the wrapper. Rule W makes Lane 1 the
  only lane permitted to touch this surface.
- Quiet baseline receipt `2026-11-01T135748Z`, receipt digest
  `2fdbd1e8ab3a7d9367e5b54791eaf607606548c4c39824cc81be033dab0bba6d`: `active/running`, `n_restarts=0`,
  rows manifest `1bcac986…`, drop-directory digest `4055db47400767aa63c70f6af0e1f2394244051e12ecbc859bf4da980b207f5c`
  (6 entries, no new staging since install), wrapper / unit / archive digests as above.
- Caveat: the wrapper's audit `logger` tag has **no readable journal entries at all** in this seat's view, so
  that is not evidence of absence; the install-operation record is the preserved install evidence, not the
  journal.
- What the receipt must add at execution time: the Lead's hold declaration for the interval (all lanes;
  named writers held), a window-start and window-end receipt from the same read-only script, and the
  approved procedure's own stage/boundary markers. Receipt script (portable, permitted routes only):
  `~/workspace/harness-build/scratch-tb-l1/window-receipt.sh <label>` — prints digest-level fields and the
  private receipt path. The interval must span before the v3 stop through after the Stage 2 check's final
  restart; the service's own DB/state writes are expected and do not touch the compared root.

## Pinned-input re-verification (before any approved run) — ALL MATCH TODAY

Procedure `652d69f59d4edfa62a7f702e26cfc5a3b101aa1e96284620f8b9d65ea32012b7`; bootstrap doc
`129972590e9e0323898c92249febd0d942e47a84263748824249e114221053f6`; caller `556ad026ae15718e507b4d0222cee31ee21145315f4484b4648db246630584f3`;
comparison core `f211db39f4cb9b948b802c96d023dea7646c014bb9a919e13563e5e620328023`; expected 82-member
manifest `6fa03597e35a2b86994011ff8206b9beb2f8b601bea8f38e6fb20f9a0617ff06`; archive `37e3a542…`;
independent review `de46696638fa2749f8c5a96999580c8d9580b76ce8ffa5f978374dcd04245f24`; `/usr/bin/dd`
`095c38ac008cb3ebceda63486f663ee9508acbcca8efae7c359102b45824b8a3`. v3 procedure tests 22/22 and
caller/integration 12/12 are recorded in the reviewed log with pinning intact.

## What this unlocks / what is still needed

1. Lane-1 brief correction: "stage the identity-proof helper **v2**" is superseded by the preserved record —
   the **v2 procedure failed independent review and must not be run**; the live approved artifact is **v3**
   (approved by Cayleb, HOLD before execution), which embeds the unchanged v2 comparison core. Team B stages
   **v3 + these four facts** for the fresh desktop-vantage reviewer; not v2.
2. Operator (Lead / owner-controlled seat) runs the exact hash-pinned v3 bootstrap **once**, in an exclusive
   window, with window-start/end receipts.
3. Fresh eligible checker (did not build or review the candidate or the helper) verifies the complete v3 result
   under the mandatory full-result acceptance rule and these facts, then runs the **unmodified** Stage 2 check
   on the installed output, one run at a time.
4. Nox rules the Stage 2 gate; Team B files the corrected FINAL report, INDEX correction and F11 Block I
   amendment.
