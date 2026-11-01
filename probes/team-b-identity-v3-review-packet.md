# TEAM B / LANE 1 — independent review packet: identity V3 + the four host facts

Prepared 2026-11-01T14:12Z (09:12 ET) by `fleet-lead` (Team B) for a **fresh desktop-vantage reviewer**, per the
Lead's spawn. This packet is **review instructions only**. It authorises nothing: the approved one-time V3
operation stays **HOLD before execution** until this review, the four host facts and the owner-controlled
operator seat are all in place.

## 1. What you are reviewing (and what you are not)

- **In scope:** (i) the four host facts as filed (`board/probes/team-b-host-facts-20261101T1402Z.md`), and
  (ii) whether the approved-but-held V3 identity procedure's entry conditions are now evidenced enough to
  execute — i.e. is each of the four facts actually established, and what remains.
- **Out of scope:** you are **not** the formal Stage 2 installed-output checker. That role is a separate
  fresh seat that runs the **unmodified** `checks/stage2-check.sh` against the installed output after the
  identity proof is accepted. Do not run it, do not pre-empt it.
- **Independence:** you must not have built, staged, edited or reviewed the v1/v2/v3 identity helper,
  caller, comparison core or procedure. The prior independent reviewer (`de466966…`, preserved record) is
  **ineligible** for formal acceptance. Reviewer of this packet is preparation/assessment only; it cannot
  self-certify the run.

## 2. Inputs — preserved read-only workspace, with pinned SHA-256 (re-hash them; all matched at 14:05Z)

Workspace: `~/Documents/Codex/2026-10-31/task/mission1/` — a **retired lane's preserved workspace: READ-ONLY.**
Do not create, edit or delete anything there. Team B's own lane evidence is in
`~/workspace/harness-build/scratch-tb-l1/` (read-only for you too; ask the Lead if you need raw outputs).

| Artifact | Pinned SHA-256 |
|---|---|
| `PROPOSED-identity-procedure-v3.py` (the approved operation) | `652d69f59d4edfa62a7f702e26cfc5a3b101aa1e96284620f8b9d65ea32012b7` |
| `IDENTITY-CALLER-PROCEDURE-V3.md` (bootstrap document) | `129972590e9e0323898c92249febd0d942e47a84263748824249e114221053f6` |
| `PROPOSED-identity-caller-v2.py` (fixed fleet-principal caller) | `556ad026ae15718e507b4d0222cee31ee21145315f4484b4648db246630584f3` |
| `identity_snapshot_core.py` (comparison core) | `f211db39f4cb9b948b802c96d023dea7646c014bb9a919e13563e5e620328023` |
| `identity-accepted-manifest.txt` (expected 82-member manifest) | `6fa03597e35a2b86994011ff8206b9beb2f8b601bea8f38e6fb20f9a0617ff06` |
| Independent review of v3 (source PASS WITH CONDITIONS) | `de46696638fa2749f8c5a96999580c8d9580b76ce8ffa5f978374dcd04245f24` |
| Accepted 0.2.2 archive at `/var/lib/fleet-drop/cayleb-dsh-fleet-0.2.2.tgz` | `37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177` |
| `test_identity_procedure_v3.py` / `test_identity_caller_v3_integration.py` | `229f88b0…` / `50e96ddf…` (full values in `IDENTITY-CALLER-PROCEDURE-V3.md`) |
| `/usr/bin/dd` | `095c38ac008cb3ebceda63486f663ee9508acbcca8efae7c359102b45824b8a3` |

State records to read first: `IDENTITY-V3-FRESH-CHECKER-HANDOFF.md` (07:56 ET, names the state),
`IDENTITY-V3-APPROVED-EXECUTION-HOLD-20261101.md`, `IDENTITY-V3-HOST-ROUTE-DECISION-20261101.md`,
`REAL-INSTALL-RUNBOOK-0.2.2.md`, `INSTALL-0.2.2-EVIDENCE.md`.

## 3. Read-only checks you can run yourself (permitted routes; all exit 0, no prompt, no service action)

Run these and compare to the facts file. Capture your own output; do not post raw host/process detail.

```sh
# 1. fixed wrapper read verbs (reviewed sudoers NOPASSWD grant, fleet principal)
sudo -n -u fleet /usr/local/bin/dsh-fleet-admin version
sudo -n -u fleet /usr/local/bin/dsh-fleet-admin status
sudo -n -u fleet /usr/local/bin/dsh-fleet-admin dump-config | grep -n 'rowsManifest'   # expect: no output (no override)

# 2. fleet control-socket status via the accepted cf93107 CLI (owner uid, peer-cred admitted)
cd /tmp && node /home/cayleb/workspace/harness-build/dsh-fleet-integration/package/bin/dsh-fleet.mjs status

# 3. archive <-> in-process manifest binding
mkdir -p /tmp/v3-review && cd /tmp/v3-review
tar -xzf /var/lib/fleet-drop/cayleb-dsh-fleet-0.2.2.tgz package/rows.manifest.json
tar -xzf /var/lib/fleet-drop/cayleb-dsh-fleet-0.2.1.tgz package/rows.manifest.json -C . --transform 's,^package/rows.manifest.json,rows.manifest.021.json,'
sha256sum package/rows.manifest.json rows.manifest.021.json    # expect 1bcac986… and fdcd2b72…

# 4. installed artefact/source binding
sha256sum /usr/local/bin/dsh-fleet-admin /etc/systemd/system/fleet-dsh.service
cmp -s /etc/systemd/system/fleet-dsh.service /home/cayleb/workspace/harness-build/dsh-fleet-integration/ops/systemd/fleet-dsh.service && echo unit==source
sha256sum /home/cayleb/workspace/harness-build/dsh-fleet-integration/checks/stage2-check.sh \
          /home/cayleb/workspace/harness-build/dsh-fleet-integration/package/bin/dsh-fleet.mjs
cmp -s /var/lib/fleet-drop/cayleb-dsh-fleet-0.2.2.tgz \
       /home/cayleb/workspace/harness-build/dsh-fleet-integration/dist/cayleb-dsh-fleet-0.2.2.tgz && echo archive==candidate-dist

# 5. window receipt (Team B's script; read-only, prints digest-level fields only)
~/workspace/harness-build/scratch-tb-l1/window-receipt.sh review-<your-seat>
```

## 4. Questions to answer explicitly (per-question verdict + the evidence you used)

- **Q1 — permitted host route.** Is the filed split correct: read route ESTABLISHED and proven; execution
  route for the approved V3 operation identified only at the owner-controlled / Lead level and not held by
  Team B? Is anything about that split wrong or missing?
- **Q2 — actual loader origin (the open policy question).** The running process reports the 0.2.2 row-manifest
  digest, that path derives from the loaded module's own `import.meta.url` by default, and the composed
  config carries **no** `rowsManifest` override. Is that in-process content correlation enough to bind the
  measured installed root to the loaded root, or is the separately reviewed **in-process witness** still
  required? Give your independent view — **Nox rules this**; you are not asked to rule it.
- **Q3 — installed mode/ownership.** Is the split right (top-level measured now; full-tree modes/links/owner
  are the approved caller's measurement and not an entry condition per the 05:36Z correction), and is the
  candidate policy (0750 + uid:gid 951:950 + archive-derived file modes + single-link) a fair fail-closed
  check for the caller?
- **Q4 — no-writer window.** Is the proposed receipt adequate (Lead's cross-lane hold declaration +
  window-start/end runs of the read-only receipt script + v3's own stage/boundary markers, spanning before
  the stop through after the Stage 2 restart)? What must be added, concretely?
- **Q5 — procedure conditions.** Does today's evidence contradict any condition in the reviewed v3 procedure
  or in the independent review's conditions (e.g. at-most-one start attempt, stop-before-measure ordering,
  full-result acceptance rule, stage disposition requirement)? Name any contradiction.

## 5. Hard constraints for this review — do not cross

Do **not**: run the V3 bootstrap or its caller; stop/start/restart any unit; read the protected installed tree
(`/var/lib/fleet/...`) by any route; retry any previously denied read or system-bus call; request a sandbox
escalation; alter sudoers/polkit/units/profile; write to the preserved workspace; contact Cayleb; post raw
host/service/process detail anywhere. If a denied operation recurs, record it and stop.

## 6. Output

One file: `board/reports/team-b-identity-v3-review-<seat-or-timestamp>.md` — per-question verdict
(ESTABLISHED / PARTIAL / NOT ESTABLISHED / CONTRADICTED) with the digests you recomputed and the method; any
corrections to the facts file; and a plain list of what still blocks the V3 execution. Then report to the Lead
in ≤20 lines. **Verdicts and digests only; no raw host detail.**

## 7. Escalation / successor

The operator run (owner-controlled seat, one invocation, agreed window) is the successor to your PASS.
After it executes, a **different** fresh seat runs the unmodified Stage 2 check on the installed output.
Team B files the corrected FINAL/INDEX/F11 pack in parallel (in progress) and stays read-only on the
installed service throughout.
