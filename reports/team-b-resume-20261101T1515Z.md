# TEAM B — RESUME NOTE (state, owners, queues, next pulls)

Written 2026-11-01T15:15Z by `fleet-lead`. A successor session should take over from this file alone: every
item below is on disk, with its path. Disposable session, durable record.

## Who owns what (Team B, this session)

`fleet-lead` owns Lanes 1-4. The delegator (Lead) spawns crew/checkers; this session cannot (child-limit +
approvals disabled). No teammates spawned from here. Write areas declared and accepted 08:42 ET: board
`reports/team-b-*` + `probes/team-b-*`; `~/workspace/harness-build/dsh-fleet-integration` (read-mostly);
`scratch-tb-l1/l2/l3/l4`; `evidence/stage-2/**`, `evidence/stage-3/redteam-package/**`. Read-only: Nox's board,
`~/Documents/Codex/**`, other teams' trees, the installed service (Lane 1-only single writer).

## Lane 1 — proof chain (critical path)

- **Four host facts FILED** to Nox: `board/probes/team-b-host-facts-20261101T1402Z.md`.
- **Independent reviewer verdict IN**: `board/reports/fleet-checker-identity-v3-review.md` — artifact PASS with
  conditions, **execution HOLD**; reviewer ineligible as formal checker.
- **Post-review addendum FILED**: `board/probes/team-b-host-facts-addendum-20261101T1505Z.md`.
  - FACT 1 proven **NEGATIVE**: root sudo password-required for all five staging shapes; only fixed fleet
    wrapper verbs are NOPASSWD ⇒ **V3 is UNRUNNABLE under the current approval**. Owner decision needed
    (sudoers addition for the exact shapes + caller as a new approval, or drop the run). Reported to Lead.
  - FACT 2 with **Nox** (policy ruling: accept-with-narrowed-wording vs in-process witness vs narrowed claim).
  - FACT 3 layout caveat recorded (leaf-real-dir + single-link unverified; post-stop block = expected
    fail-closed outcome).
  - FACT 4 receipt **hardened v2** `~/workspace/harness-build/scratch-tb-l1/window-receipt-v2.sh` (fail-closed
    `RECEIPT_INCOMPLETE`, procedure-invocation tie-in, restart enumeration, hashed Lead hold declaration,
    explicit blind spot). v1 preserved. Baseline-v2 receipt 14:51:30Z. **Window-end path awaits a real window.**
- V3 artifacts preserved read-only at `~/Documents/Codex/2026-10-31/task/mission1/`; all pinned hashes re-verified
  today (procedure `652d69f5`, bootstrap `12997259`, caller `556ad026`, core `f211db39`, manifest `6fa03597`,
  archive `37e3a542`, review `de466966`, `/usr/bin/dd` `095c38ac`).
- **Next pulls (blocked):** owner decision on FACT 1 route; Nox ruling on FACT 2; then window + operator run
  (owner-controlled seat, physically present) + distinct fresh installed-output checker on the WHOLE private
  result (not exit code) + Stage 2 check + Nox gate ruling.

## Lane 2 — candidate repair (scratch first)

- **Isolated-tree scratch acceptance PASS 60/60, exit 0** (09:39–09:41 ET): `board/reports/team-b-lane2-scratch-acceptance.md`.
  No contact with installed service. Evidence `~/workspace/harness-build/scratch-tb-l2/raw/s2accept/` (3.4 MB).
  Two check-adaptation defects found+fixed (scratch overlay fleet-policy row; derived-check start-script path).
- **Effects/Outbox design FILED**: `board/reports/team-b-lane2-effects-outbox-design.md` (write-first effects,
  retry/backoff, restart recovery; additive `effects` table; crash matrix; acceptance).
- **Next pulls:** hands-code race fix + bus package assembly (isolated throwaway worktrees), contract-doc
  amendment incl. throwing-accessor known-limitation (Nox 072312Z, proposal 80ad8715, tree 96a52f5e), real
  adapter under reviewed producer selection, composed-path e2e prep. Nothing installs from this lane.

## Lane 3 — Stage 3 + providers

- **Item-7 package v1 assembled**: `~/workspace/harness-build/evidence/stage-3/redteam-package/INDEX.md`
  (ten attacks a–j w/ commands + observed refusals + no-exec evidence; grant legs; 3.3; 3.4 incl. 3.4c
  scratch-limit FAIL named; 3.5; reviewer findings/fixes; known limits; real-install prerequisites).
  Scratch evidence of record: `board/probes/team-b-final-tree/rehearsals/stage3/raw/` (73 PASS/1 FAIL/24 MANUAL).
- **Provider scope FILED**: `board/reports/team-b-lane3-provider-scope.md` (OpenCode Go verified; DeepSeek direct
  key-field ready, no envelope ⇒ cap zero; OpenRouter key only; **Grok gap named: subscription ≠ API key**;
  Cursor bridge unofficial/off-by-default). No key handled.
- **Next pulls:** real-install Stage 3 run (needs Lane-1 window + journal-clean + Cayleb's tty run), then
  reviewer scripts' fixes carried, 3.1 verdict from Nox.

## Lane 4 — surfaces + device rig

- **Rig plan FILED**: `board/reports/team-b-lane4-rig-plan.md`. **MCP facade contract FILED**:
  `board/reports/team-b-lane4-mcp-facade-contract.md` (session-start + bus/hands over one facade, rails, kill
  switch, T1–T4 acceptance).
- **Rig skeleton L4-1 done**: `~/workspace/harness-build/scratch-tb-l4/rig/FACTS.md` (QEMU 11.1.2, KVM rw,
  OVMF+secboot, swtpm 0.10.2, 587G free, 40.7G RAM, 32 threads) + free downloads hashed (Android platform-tools
  + cmdline-tools zips in `rig/iso/`, `DOWNLOADS.sha256`); **no install, no licence acceptance**.
- **Blockers (with Lead):** licence/EULA acceptance ruling for free eval/SDK; /dev/kvm 0660 host change HELD
  (leave 666); Windows eval media acquisition route.
- **Next pulls:** panel-plugin skeleton against the DSH client surface (throwaway worktree); Windows eval ISO
  acquisition once licence question ruled; Android SDK package install once ruled.

## Filed gate pack (Mission 1 close-out deliverable)

`board/reports/team-b-gate-pack-20261101.md` — corrected FINAL report (§10 amendment register),
INDEX correction, F11 Block I amendment, four host facts + addendum, reviewer packet, receipt v2. Deterministic:
`board/reports/stage2-gate-report-FINAL.md` (5df00932…), `board/reports/team-b-f11-block-i-amendment.md`
(fe790e02…), `board/probes/team-b-final-tree/rehearsals/INDEX.md` (7ee7f4b1…).

## Scope items adopted today (design, no prototype code)

`board/reports/team-b-scope-items-20261101.md` — mandate kernel observe-mode (Stage 5), write-first
Effects/Outbox (Lane 2 + Stage 9), Router/Worker topology (Stage 4/5), revalidate-mandate-at-resume divergence
(Stage 8/9), MCP amendment (Lane 4/Stage 5), crew depth, provider set (minus Codex), rejected access model.

## Hard holds in force (unchanged)

No spend beyond a stated envelope (unstated = zero); no credential values handled; no public posts; no writes to
Cayleb's personal DSH profile; the installed fleet service is Lane 1-only, one change at a time; `~/Documents/
Codex/**` and Nox's board are read-only; no merge to `main` without Nox's ruling.

## UPDATE 2026-11-01T15:45Z (same session)

- **Owner decision landed (option a):** narrow sudoers rule approved (Nox 151700Z). Owner command relay READY +
  validated: `board/probes/team-b-identity-v3-owner-command-block.md` (sha256 `a05ee4ca…`) — exact sudoers
  drop-in (14 literal shapes + fleet caller; `=`/`:` escaped — unescaped form fails visudo), minimal polkit
  drop-in (manage-units YES for cayleb on unit fleet-dsh, verbs stop/start; needed because pkcheck shows
  `auth_admin_keep` today), copy-paste install block (validate-before-install), removal block, read-only
  pre-flight plan, operator invocation. **NOT installed; no service action.** Awaiting operator INSTALLED +
  pre-flight evidence, then Nox FACT 2 ruling before any window.
- **Grok re-check filed** (`team-b-lane3-provider-scope.md` §Re-check, sha256 `dd7383f0…`): finding narrowed —
  API key is created in xAI API Console (not auto-derived from the subscription); docs for The X Subscription
  (separate product) never mention an API key; owner's first-hand observation (key produced, usage on separate X
  dev pool) recorded as the stronger account-specific evidence; usage pool to be read at key entry; no spend.
- **Lane 4 rig:** FACTS.md updated (QEMU 11.1.2, KVM rw/666 held, OVMF+secboot, swtpm, 587G/40.7G, 32 threads);
  free downloads hashed: Android platform-tools + cmdline-tools zips (`rig/iso/DOWNLOADS.sha256`), portable
  JDK 21 (`rig/tools/jdk21.tar.gz` sha256 `d7f5d5cf…`). Licence ruling now permits free eval/SDK/EULA installs
  with texts recorded — next: unpack JDK, sdkmanager licences (record texts), install emulator + system image;
  Windows eval ISO direct links found on the eval page (x64 Enterprise fwlink), acquisition next.
- **Lane 4 browser-control scope (new):** identify the installed background-browser plugin by name from the
  profile inventory, verify licence/source, absorb under absorb-or-remove; plus control of Cayleb's own Chrome.
  Candidate found so far: `@deepseek-ai/dsh-browser-use` v0.2.2-alpha.2, MIT, package group
  `packages/browser-use/` (exclusive named provider registration; provider sub-packages) — needs the provider
  package name from Cayleb's actual profile inventory (the personal profile at `~/.dsh/profiles/web` has no
  deps; the session toolset's provider must be resolved from the running harness's plugin set). NEXT PULL.
- Context discipline: this session is long; the successor should continue from this note + the referenced files.

## UPDATE 2026-11-01T15:48Z

- **Owner install + pre-flight CONFIRMED by Lead:** Cayleb ran §B (INSTALLED) and §C pre-flight — all 21 checks
  PASS, receipt `424b71b4…`, rc=0. The narrow sudoers + polkit drop-ins are now installed; §D removal stays
  batched for after the run + fresh checker. **FACT 1 closed for scheduling**; FACT 2 (Nox policy ruling) still
  gates window open. No window opened, no V3 run taken.
- **Window receipt v3 (bug fixed):**
  `~/workspace/harness-build/scratch-tb-l1/window-receipt-v3.sh` sha256 `6c410a0e…`;
  `window-receipt-run.sh` sha256 `273fa8d3…` (NNP-free launcher via systemd-run; resolves relative paths;
  `--journal-since` passthrough). Fix: window-end reads journal from window-start minus 7 days (or explicit
  `--journal-since`), enumerates in-window starts, and separately proves the procedure's baseline invocation in
  the wider slice (baseline receipt time CANNOT bound it — baseline starts before the receipt). Fail-closed
  preserved. Dry-run PASS: window-start receipt `779ba03b…`, window-end receipt `aaac7c86…`
  (baseline proof `df02e934`, after `2ff92d0b`), archived in `scratch-tb-l1/raw/dryrun/`. NNP/too-recent-slice
  fail-closed cases rc3.
- **Window-end procedure-result arg = `identity-cf93107-v3-procedure-summary.json`** (the bootstrap redirects the
  procedure's stdout/result JSON there); `identity-cf93107-v3-observation.json` is the caller's disk observation,
  `…-diagnostic.txt` the caller stderr. Runner takes whichever JSON exists — corrected to the summary file.
- **Lane 4:** browser-control component identified: `dsh-inner-browser-use@0.41.0` (MIT, regrevia; npm tarball
  sha1 `5659e097…` / integrity verified; installed == published byte-identical; right-column mirror). Absorb/remove
  assessment + gap (Cayleb's own Chrome not covered — CDP-attach decision) in
  `board/reports/team-b-lane4-browser-control-absorb.md`. Android SDK installed (emulator 37.2.13, platform-tools
  37.0.1, android-35 image; licences recorded + "All SDK package licenses accepted"); Windows 11 Enterprise Eval
  ISO downloaded free/no account, 8,225,329,152 B, sha256 `bc3f2408…`; rig FACTS.md updated.

## UPDATE 2026-11-01T15:58Z — V3 run executed once, FAILED CLOSED (approval consumed; do not retry)

- One-time run 15:46:23–15:47:40Z (operator seat, NNP-free, one invocation). Receipts verified: baseline
  `af2f99fc…`, window-start `3843b354…` (hold sha `56016059…`), window-end `311d4760…` (baseline `2ff92d0b…` →
  after `e44cc343…`, NRestarts 0). Service restored; hold interval closed; sudoers/polkit being removed by the
  Lead as instructed.
- Result: `outcome=BLOCKED`; caller diagnostic `package tree failed comparison: full file mode mismatch`;
  `restoration=RESTORED_ONCE`; stage preserved (`/run/dsh-fleet-id-cf93107`, three root 0444 files whose digests
  match the pins `556ad026…`/`f211db39…`/`6fa03597…`); observation 0 B; formal Stage 2 NOT RUN.
- **Root cause diagnosed + reproduced (no protected read, no retry):** the authorised wrapper sets `umask 027`;
  the sanctioned `dsh plugin add` then installs files as `archive_mode & ~0o027` — 58/82 files differ (0644→0640,
  0755→0750; package root 0750). Under umask 022 the same installer is 82/82 archive-identical (Lane 2 scratch).
  So the candidate's exact-mode contract conflicts with the documented installer path.
- **Ruling recommendation filed** (`board/probes/team-b-identity-v3-run-record-20261101T1555Z.md`): record identity
  FAILED; repair via installer mode preservation (preferred) or a reviewed v4 mode-policy amendment (Nox decision,
  not post-hoc); **any second run needs a FRESH owner approval** (the one-time approval is consumed; a v4/install
  change also needs new review + a new operator relay). Preserved stage: recommend one-line owner-authorised
  removal after this record. FACT 2 moot for this attempt, open only for future claim wording.
- Team B can proceed without approval: prove installer-fix variant in scratch (umask 022 + dirs 0750), draft v4
  proposal if Nox prefers amendment, continue the corrected pack + other lanes.

## UPDATE 2026-11-01T16:10Z — repair evidence + amendment draft

- **Installer-fix proof FILED** (`board/probes/team-b-installer-mode-fix-proof-20261101T1607Z.md`): scratch
  experiments show (A) install-time umask 022 alone is INSUFFICIENT — the pnpm store (populated under wrapper
  umask 027) yields 58 mode diffs and 82 hardlinked files; (B) a fixed post-add normalisation (atomic copy+rename
  per file to de-link, set the archive member's mode, chmod package-tree dirs 0750) makes the pinned comparison
  core **PASS** (82 files, archive `37e3a542…`). Proposed fixed wrapper step (no caller args, fail-closed re-verify)
  written into the proof for review; wrapper change still needs review + owner approval + wrapper reinstall +
  release install act + a fresh V3-run approval.
- **V4 amendment draft FILED** (`board/reports/team-b-identity-v4-amendment-draft.md`) as the policy alternative:
  `expected_mode = archive_mode & ~0o027`, no widening, single-link/ownership/dirs/bytes unchanged, residual
  named verbatim; trade-off table and deliverables if Nox chooses it. Draft only — not applied.
- Recommendation to Lead/Nox: **FIX over AMEND** (strong claim recoverable; removes hardlink class too).
- Run record, gate pack and resume updated; evidence logs in `scratch-tb-l1/raw/fix-proof-*.log`.

## UPDATE 2026-11-01T16:50Z (post-crash resume)

- **Browser reconciliation FILED** (`reports/team-b-browser-control-reconciliation.md`, fb492a8a…): installed
  `dsh-inner-browser-use` 0.41.0 vs audit `dsh-builtin-browser` 0.5.2. Leg A: inner mirrors via sidebar client
  inject, no host mutation; candidate needs its desktop bridge that patches app `main.js` (crash class, Windows-
  verified only, dev build here). Leg B DECISIVE: neither controls Cayleb's RUNNING Chrome — both launch his
  installed Chrome/Edge with a plugin-owned `--user-data-dir`; live-session control = named gap/CDP-attach
  decision. Recommendation: inner for the mirror, no main.js bridge; Nox rules pick + leg-B meaning; neither
  absorbed. Absorb record §7 pointer updated.
- **Adopted map + memory order** in `reports/team-b-scope-items-20261101.md` §Addendum (API-keys-only, built-in
  providers, custom Claude engine, dsh-notifier, dsh-web-mcp heavy wrap, hands consolidation, sub-agent ceiling
  lever, crash-isolation host property; sage-mem adopt+wrap with soak/one-strike rollback, staging-profile shape
  confirmed by Lead, archive path + one-way typed-markdown migration, runner-up custom Cordis).
- **Wrapper mode-normalisation review package FILED** (`reports/team-b-wrapper-mode-normalisation-review-package.md`)
  — ready for Nox's FIX ruling; not implemented.
- **Lane 2**: isolated worktree branch `tb/effects-outbox` @ cf93107 created (scratch-tb-l2/build-effects) +
  ordered build queue (`reports/team-b-lane2-build-queue.md`); Effects/Outbox build is the next pull (not started).
- **Lane 3**: real-install Stage 3 prerequisites filed (`probes/team-b-stage3-realinstall-prerequisites.md`);
  **sage-mem admission start** — dedicated staging profile `scratch-tb-l3/staging-profiles/sage-mem-soak`,
  pinned `sage-mem` 2.1.22 installed (integrity matched), **50-cycle soak PASS** (0 not-ready / 0 died / 0 crash
  lines, log `09ca12eb…`) with one **named degradation: autodream init fails** (`timer` inject requirement absent
  on DSH 0.2.2-alpha.2; plugin's own documented degradation path; core memory mounts; compatibility list stops at
  0.2.1-rc.5). Record: `reports/team-b-sage-mem-admission-soak.md`. Not promoted; functional phase + fresh checker
  next; archive untouched.
- **Lane 4**: Android AVD `rig-phone` created in rig; headless boot verified (~20 s) and **text-only** round-trip
  verified (uiautomator dump pulled, 9,810 B XML; tap rc=0; screencap captured as file+md5 only — **no image
  reads** per host crash rule; 241 packages). Windows node: 80 GiB disk, OVMF secboot vars, virtio-win ISO,
  working swtpm TPM 2.0, well-formed autounattend, launcher; **install launch smoke PASS** (VNC loopback 5902,
  monitor responsive) after vvfat `fat:rw:` fix. Status `reports/team-b-lane4-rig-status.md`.
- Next pulls: Lane 2 Effects/Outbox migration+crash-matrix on the isolated tree; sage-mem functional phase
  (session-driven memory tools, text-only) then fresh checker; Lane 4 unattended Windows install observation
  (text/monitor-level), Android scripted round-trip, panel skeleton; Lane 1 frozen pending Nox.

## UPDATE 2026-11-01T17:05Z — sage-mem functional phase + component map

- **Time-boxed compat check (no spend):** no newer sage-mem (npm latest 2.1.22, published today 04:48Z, 95
  versions); v2.1.22 notes confirm real-machine testing on DSH 0.2.1-rc.5; **no documented compat setting for the
  autodream/timer inject failure** (DEFAULT_CONFIG.enabled=false is a feature flag, not a compat knob).
  → **no fix exists today**; named limitation stands.
- **Functional phase PASS (text-only, staging):** `reports/team-b-sage-mem-functional-phase.md` + harness
  `scratch-tb-l3/staging-profiles/sage-mem-soak/functional/run.mjs` (output log sha256 `612eddc9…`): file protocol
  (0600 modes), create-exclusive refusal, MEMORY.md index maintenance (4 entries), scan complete, JIT recall with
  baseline-once, bounded injection (1,084 chars), audit **problems=0** on a clean sample library (prefix rule is
  `<type>_<name>.md`). Not promoted; fresh checker + session-driven tool phase next; archive untouched; rollback armed.
- **COMPONENT MAP FILED:** `reports/team-b-component-map.md` — 13 entries with status + named gaps, incl. the
  memory named limitation verbatim, browser reconciliation pending, dsh-notifier, dsh-web-mcp heavy wrap, hands
  consolidation, providers API-keys-only, effects/outbox, mandate kernel/revalidate-at-resume, router/worker, VM
  panels, sub-agent ceiling lever, crash-isolation host property, single-plugin rule.
- **Lane 4 VM:** Windows install did not progress under the secure-boot OVMF (blank VARS have no enrolled keys →
  loader rejected; symptom: qemu 99.8% CPU, no disk writes). Switched install stage to **non-secure-boot OVMF**
  (`OVMF_CODE.4m.fd` + fresh VARS; TPM still present) and retried with boot-key spam; disk/CPU observation ongoing.
  Android AVD round-trip (text-only) already verified.

## UPDATE 2026-11-01T17:25Z — Lane 2 Effects/Outbox first build step; VM autounattend media fix attempts

- **Lane 2 Effects/Outbox BUILD STEP DONE (isolated branch, tested):** worktree
  `scratch-tb-l2/build-effects`, branch `tb/effects-outbox` @ cf93107. Added `orders/006-effects` migration +
  `package/src/orders/effects.js` (write-first record, idempotent key, forward-only states, backoff, poison
  abandon at 5 attempts, restart recovery, per-order counts) + wired in `store.js` + 5 tests.
  Evidence: `effects.test.mjs 5/5 pass`; existing orders suites all green (board 8, claims 18, cycle 21, flow 15,
  freshness 6, liveness 40, plugin 18). Report: `reports/team-b-lane2-effects-outbox-build-step.md`. Not merged,
  not installed; next = wire consequential writes + executor loop + restart pass + crash matrix + fresh checker.
- **Lane 4 Windows install still stalled at pre-install phase** (disk 393,216 B, CPU ~100%): ruled out secure-boot
  loader rejection (now non-secboot OVMF), then suspected the vvfat answer drive is not searched for
  `autounattend.xml` → built a **FAT12 floppy image** with the answer file (`win/unattend.img`, 1,444,560 B,
  sha256 `2e3dac1c…`) and switched the launcher to `-fda`; next attempt adds `-boot order=d,menu=off` and
  checks whether the ISO carries EFI boot files. Observation still monitor/disk-level only (no images). If this
  still stalls, the next diagnostic is the boot path itself (recorded, not guessed); secure-boot for the run
  stage remains OPEN.

## UPDATE 2026-11-01T18:08Z — Nox rulings landed

- **Wrapper fix direction = FIX (not AMEND)**; owner review package FILED as one combined ask (wrapper fix + one
  further V3 attempt): `reports/team-b-wrapper-fix-owner-review-package.md` (sha256 `14c2f21c…`). Contains exact
  diff (+53 lines, proposed wrapper sha `eb0ad024…`), pinned pre/post hashes + 82-pair source, idempotency/failure
  behaviour, explicit no-change statement, rollback/uninstall, test plan; scratch re-proof of the exact proposed
  wrapper function PASSED (82 single-link/archive modes/dirs 0750 → pinned core PASS; idempotent; tamper repaired;
  missing member fail-closed rc 64). **Nothing installed/run; no window.**
- **V3 stage disposition recorded:** left to tmpfs clearing on reboot, digests recorded, no secrets (run record §7);
  earlier owner removal remains a separate authorised act.
- **sage-mem: Nox ACCEPTED WITH LIMITATION** — basis: pinned version, fleet-profile staging, 50+ soak done,
  one-strike rollback armed, memory dir in the git-backed store; platform fact named (plugins in-process, no
  component crash-proof by selection). Component map updated; promotion still needs fresh checker.
- **Lane 3 security-audit calibration quick profile FILED:** `reports/team-b-lane3-security-audit-calibration-profile.md`
  (S1 grant/guard first; counts/states only; findings never git-tracked).
