# Lane 1 — independent desktop-vantage review: identity-proof V3 + the four host facts

**Seat:** fresh desktop-vantage reviewer, spawned by the Lead, reporting to the Lead only.
**Date:** 2026-11-01, 14:2x–14:5xZ (09:2x–09:5x ET).
**Inputs:** `board/probes/team-b-identity-v3-review-packet.md`, `board/probes/team-b-host-facts-20261101T1402Z.md`,
and the preserved record at `~/Documents/Codex/2026-10-31/task/mission1/` (read-only; nothing there was created,
edited or deleted).

## Verdict

**HOLD — the run is not yet safe-and-sufficient to execute as an owner-controlled operator act.**

Split verdict, because the artifact and the execution have different states:

- **The V3 artifact itself: PASS WITH CONDITIONS.** It is fixed-purpose, it never writes the protected tree, it is
  digest-and-metadata out only, and its failure paths are genuinely fail-closed. Nothing I read in the source or
  reproduced in the tests argues against running *this exact hash-pinned code* under the approved schema.
- **The execution: HOLD.** The owner's own approval (`IDENTITY-V3-APPROVED-EXECUTION-HOLD-20261101.md:3`) requires
  stopping if the permitted route, **loader origin**, installed-mode policy or no-writer conditions cannot be
  established. Loader origin is still only partially established (FACT 2), the no-writer receipt does not cover the
  compared root (FACT 4), and the execution route has a concrete, previously unnamed sub-blocker (FACT 1). Those are
  entry conditions, not post-run acceptance items, so they block invocation rather than the acceptance of a result.

Decisive points, in order: (1) FACTS 1/2/4 are PARTIAL against the approval's literal stop conditions; (2) the
in-process manifest correlation binds the loaded bundle's *declarative* layer, not its *code* bytes, so it cannot
substitute for loader-origin evidence; (3) the procedure refuses to run as root and needs non-interactive `sudo`
from a non-tty child — unverified, and the failure lands after the stop.

---

## Item 1 — fixed purpose, read-only on the protected tree, digest-out only, and what it actually measures

**Fixed-purpose: YES.** Every path, argv, unit name and binary is a literal
(`PROPOSED-identity-procedure-v3.py:21-39` for the procedure, `PROPOSED-identity-caller-v2.py:18-36` for the caller).
`SystemOps._run` takes a fixed list and never a shell (`:176-184`); failures raise `Blocked` with the text
"no alternate route" (`:182-183`). `main()` refuses arguments, any interpreter other than isolated+no-site, any
environment other than exactly `{PATH, LC_ALL}`, any euid≠uid, and any account name other than the owner
(`:415-418`); the caller re-checks principal 951/950 and supplementary groups
(`PROPOSED-identity-caller-v2.py:48-59`) and refuses to run if it is not at the fixed staged path (`:192-193`).
The bootstrap (`IDENTITY-CALLER-PROCEDURE-V3.md:126-144`) hash-pins the procedure and executes the same in-memory
bytes. I confirmed the wrapper's `set -euC` noclobber actually protects the two summary/err files: with `set -C`,
`exec > <existing file>` fails and leaves the file untouched (`bash: cannot overwrite existing file`, exit 1,
content preserved) — so a retry cannot silently overwrite the first run's record.

**Read-only on the protected tree: YES, at the syscall level.** The comparison core opens the archive and every tree
descriptor read-only or `O_PATH`, and only calls `fstat`/`listdir`/`open`/`pread`
(`identity_snapshot_core.py:72-109`, `:112-231`). There is no `write`, `unlink`, `rename`, `chmod`, `chown` or mount
call anywhere in the caller or core. The tree is opened component-by-component with `O_NOFOLLOW`
(`PROPOSED-identity-caller-v2.py:98-131`) and only read through `/proc/self/fd` links (`:123-125`), with two full
hash passes and per-pass metadata pinning (`identity_snapshot_core.py:187-222`). The only permitted mutation is the
`atime` that a read itself produces, which the observation states (`"reads may update access times"`,
`PROPOSED-identity-caller-v2.py:246`).

**The procedure as a whole is not read-only** — it stops and starts `fleet-dsh` (`:276-280`, `:359-393`). That is
the approved act, and the ordering is the right one: measure only while the unit is confirmed inactive.

**Digest-out only: YES.** The observation carries per-file `sha256`/`mode`/`uid`/`gid`, per-directory
`(mode, uid, gid)`, root `device`/`inode`/`mount_id`, and the pinned digests — no file contents leave the tree
(`PROPOSED-identity-caller-v2.py:227-247`). The two output files are created `0600` and `O_EXCL` in the owner's
workspace (`PROPOSED-identity-procedure-v3.py:193-195`, `:146-160`).

**What it actually measures — weaker than "installed bytes bound to the running service".** It measures:

1. the tree at one hard-coded path (`TARGET`, `:24`) equals the accepted archive across all 82 files, byte for byte,
   with the candidate mode/owner/link policy;
2. the unit then started once and reported `active/running` with a **different InvocationID** (`:390-393`);
3. and the program *itself* declares the rest unestablished: `runtime_identity: NOT_ESTABLISHED`,
   `loader_origin: NOT_ESTABLISHED`, `no_writer: NOT_ESTABLISHED` (`:403-404`, and the caller's
   `"loader_origin": "NOT_ESTABLISHED_BY_CALLER"`, `PROPOSED-identity-caller-v2.py:231`).

So it is a **controlled-fresh-start disk identity claim**, not a running-service binding:

- **Gap A — no path→process binding.** Nothing in V3 asks the running process which directory it loaded. A new
  InvocationID proves a restart, not what that restart loaded.
- **Gap B — no memory proof (correctly disclaimed).** It cannot prove what the pre-stop process had resident, and
  the reviewer's condition 4 (`IDENTITY-V3-FRESH-CHECKER-HANDOFF.md:147-149`) is that nobody may claim it.
- **Gap C — the mode/owner policy is a *candidate*, not an installer contract** (declared at
  `PROPOSED-identity-procedure-v3.py:120-121`; the archive only contains modes `{644, 755}`, so the "archive-derived
  file modes" check is a two-valued test).
- **Gap D — the leaf must not be a symlink, and nobody has established that it isn't.** `PinnedPath(..., ROOT, True)`
  opens the last component with `O_NOFOLLOW|O_DIRECTORY` (`PROPOSED-identity-caller-v2.py:110-118`, `:216`). If the
  package was installed in an isolated pnpm-style layout where `node_modules/@cayleb/dsh-fleet` is a symlink, the
  caller blocks **after** the stop, consumes no start, and leaves the attempt's measurement empty. That is safe, but
  it means the one approved attempt can be spent on a layout fact that no permitted pre-check can observe. FACT 3
  does not assert symlink-freedom, and should.

The honest one-line description: **V3 proves "a fresh start happened from a tree that is byte-identical to 0.2.2"**,
contingent on the loader actually resolving to that tree and on nobody writing it in between. Both contingents are
external and both are still open.

## Item 2 — are the pinned SHA-256s internally consistent, and do the recomputed values match?

**Yes — all consistent, all reproduced.** Recomputed with `sha256sum` on the preserved record; no mismatch anywhere.

| Artifact | Pinned (packet) | Recomputed |
|---|---|---|
| `PROPOSED-identity-procedure-v3.py` | `652d69f5…12b7` | **match** `652d69f59d4edfa62a7f702e26cfc5a3b101aa1e96284620f8b9d65ea32012b7` |
| `IDENTITY-CALLER-PROCEDURE-V3.md` | `12997259…53f6` | **match** `129972590e9e0323898c92249febd0d942e47a84263748824249e114221053f6` |
| `PROPOSED-identity-caller-v2.py` | `556ad026…84f3` | **match** `556ad026ae15718e507b4d0222cee31ee21145315f4484b4648db246630584f3` |
| `identity_snapshot_core.py` | `f211db39…8023` | **match** `f211db39f4cb9b948b802c96d023dea7646c014bb9a919e13563e5e620328023` |
| `identity-accepted-manifest.txt` | `6fa03597…ff06` | **match** `6fa03597e35a2b86994011ff8206b9beb2f8b601bea8f38e6fb20f9a0617ff06` |
| `test_identity_procedure_v3.py` | `229f88b0…` | **match** `229f88b091dc284390e4b3390f6255f5feb8cdd55ffd3320cb6aa01caa2886e7` |
| `test_identity_caller_v3_integration.py` | `50e96ddf…` | **match** `50e96ddfbebc14a3325e4e66f500d1c91665521fd7ed4c3449464fa195378ad9` |
| Independent review `…/v3-review/INDEPENDENT-REVIEW.md` | `de466966…5f24` | **match** `de46696638fa2749f8c5a96999580c8d9580b76ce8ffa5f978374dcd04245f24` |
| Accepted archive `/var/lib/fleet-drop/cayleb-dsh-fleet-0.2.2.tgz` | `37e3a542…fa177` | **match** `37e3a542a32f8cb15cd06965072cfef9aac81cb157b20f7c89b50dafc36fa177` |
| `/usr/bin/dd` | `095c38ac…b8a3` | **match** `095c38ac008cb3ebceda63486f663ee9508acbcca8efae7c359102b45824b8a3` |

Cross-checks that matter more than the table:

- **The source's own constants agree with reality.** `ARCHIVE_SHA`, `CALLER_SHA`, `CORE_SHA`, `MANIFEST_SHA` and
  `FLEET_UID, FLEET_GID = 951, 950` (`PROPOSED-identity-procedure-v3.py:25-29`) equal the packet and the approval
  record (`IDENTITY-V3-APPROVED-EXECUTION-HOLD-20261101.md:20-28`), and `getent passwd fleet` → `uid=951 gid=950`.
  The archive's sidecar `cayleb-dsh-fleet-0.2.2.tgz.sha256` also states `37e3a542…`.
- **The archive really is 82 members, and the manifest really is the reported digest.** `tar -tzf` → 82 entries,
  all regular files, no directory members; modes ∈ {644, 755}; `sha256 package/rows.manifest.json` →
  `1bcac9862a9b6c9723c27b193b3a9f22ab9b07f9d30e7fef862d01d194d59ee9`, i.e. exactly the digest the running service
  reported. The 0.2.1 archive's manifest is `fdcd2b72387f5261671625440708a8b96ce83ad503334324d34cee31f480ce4f`, a
  different value — so the reported digest does discriminate 0.2.2 from 0.2.1.
- **The accepted manifest's shape is exactly what the code demands** (82 lines, sorted, unique, all
  `package/`-prefixed, exact trailing newline, 15 implicit directories: `''` + 14) — the same shape
  `_expected_records` (`:78-100`) and `_manifest_names` (`PROPOSED-identity-caller-v2.py:176-185`) require.
- **The maker test counts are real.** I re-ran both suites from a byte-identical copy under `/tmp` with the required
  interpreter flags (`python3 -I -S test_…py`): **22/22 OK** and **12/12 OK**, matching the recorded logs. (Under a
  normal `python3 -m pytest` the six caller tests fail by design — the caller's owner-context gate rejects a
  non-isolated interpreter. That is the gate working, not a defect, but it is worth knowing before anyone re-runs
  the suite and reads the failures as new.)
- **v2 is dead, and I confirmed the mechanism.** `PROPOSED-identity-procedure-v2.py:221-222` uses
  `dd oflag=excl`; GNU dd here rejects it: `dd: invalid output flag: 'excl'`, exit 1. v2 also staged at `0755`
  (`:216`). So v2 would abort at its first copy, and its staging was marginally worse. **v2 must not be run**; the
  live artifact is v3, which uses `conv=excl` at `0700` and publishes `0755` only after all three private checks
  (`PROPOSED-identity-procedure-v3.py:222-236`).
- **No output collision, and no stage is present.** All four V3 output filenames are absent from the owner
  workspace and `/run/dsh-fleet-id-cf93107` does not exist from this mount namespace. The approved attempt is
  therefore visibly unused from here (the record's own namespace caveat still applies).

## Item 3 — do the four host facts satisfy the approval's stop conditions?

| Fact | Verdict | Exactly what remains |
|---|---|---|
| 1 — permitted host route | **PARTIAL** | Read route: established (two documented routes, exit 0, no prompt); I could not and did not reproduce it (no privilege probe from this seat), and I did verify every artifact it rests on. **Execution route: identified, not proven, and one concrete sub-blocker is unnamed in the facts file** — see below. |
| 2 — actual loader origin | **PARTIAL** | In-process content correlation established; canonical directory/inode binding absent. I regard this as *insufficient* for the literal stop condition — see Item 4. |
| 3 — installed mode/ownership | **SATISFIED for its stated scope; PARTIAL on layout** | Top-level mode/owner is the entry condition per the 05:36 correction, and full-tree modes/ownership are correctly assigned to the caller's measurement. What is missing is a layout assertion: the caller *requires* a non-symlink leaf and single-link files, and no fact asserts that. |
| 4 — no-writer interval | **PARTIAL** | Definable and currently quiet, but the receipt as proposed measures the service and the drop directory, **not the compared root**, and it degrades silently instead of failing. |

**FACT 1, the unnamed sub-blocker.** The procedure *refuses* to run as root: `main()` requires
`pwd.getpwuid(os.getuid()).pw_name == "cayleb"` (`:416`), so the operator cannot simply `sudo` the whole thing.
It must run as the owner and then obtain root **from inside** via bare `/usr/bin/sudo` for exactly five command
shapes — `mkdir`, `dd`, `chmod`, `stat`, `sha256sum` (`:215`, `:218`, `:224`, `:229-231`) — plus
`sudo --user fleet -- /usr/bin/env -i …` for the caller (`:282-285`). Every one of those children is spawned with
`stdin` not a terminal (`subprocess.run(..., input=data, env=ENV)`, `:176-178`). If password-based sudo is in play,
each call either (a) succeeds only because it can still open the controlling tty of the owner's terminal, or (b)
fails closed with "no tty present / no askpass". Failure at staging is harmless (before the stop). **Failure at the
caller is not**: that `sudo --user fleet` runs *after* the stop, and a prompt there leaves the unit down while a human
types. The facts file asserts only that "the reviewed sudoers source grants only the fixed wrapper verbs" — which is
consistent with the whole run being unrunnable or prompt-dependent. That is the single most practical thing to settle
before scheduling a window, and it needs no new privilege to check (see the operator checklist).

**FACT 3, precisely what is and is not established.** Established: state dir `0750`, DSH profile dir `0750`, owner
`951:950`, unit declares `StateDirectoryMode=0750`; installed unit byte-identical to source (`b44c05d0…`), wrapper
byte-identical to source (`f84075cf…`), archive on disk `37e3a542…` with 82 members, Stage 2 script `7df6344c…` and
CLI shim `8e121b19…` (I re-verified all of these myself). Not established, and not covered by the "caller measures
it" carve-out: whether the measured leaf is a real directory rather than a symlink, and whether the 82 files are
single-link. Those are *preconditions of measurability*, not measurements. If they are wrong the run blocks after
the stop.

**FACT 4, precisely what the receipt proves.** I reproduced the baseline receipt's digest exactly: the six drop-file
digests sorted and joined produce `drop_dir_digest 4055db47400767aa63c70f6af0e1f2394244051e12ecbc859bf4da980b207f5c`,
matching the facts file. That confirms the *drop area* is quiet. It says nothing about
`/var/lib/fleet/...` — the compared root — which this class of seat cannot read and which the receipt script
deliberately never touches. So the receipt as written proves "the service did not restart and nobody staged a new
archive"; it does not prove "nobody wrote the compared root". The declared interval is right in *shape*; the
instrument does not reach the subject.

## Item 4 — FACT 2 policy question: independent view

**My view: not sufficient as filed. The in-process correlation plus the caller's 82-file comparison is strong
evidence about *content*, and it is not evidence about *location*; the separately reviewed in-process witness (or an
equivalent that records the loaded directory's identity) is what closes the gap. If the fleet wants to run on the
weaker standard, that standard has to be written down and owned as a policy choice, not presented as loader origin
established.**

Why, concretely — this is narrower and more specific than "the directory string is unknown":

1. **The mechanism claim in Fact 2 is correct at source level.** `DEFAULTS.rowsManifest` is
   `new URL('../../rows.manifest.json', import.meta.url)` (`dsh-fleet-integration/package/src/core/config.js:22-23`),
   the Cordis schema defaults to it (`package/src/core/index.js:27`), and `core.js:110` loads exactly that path and
   reports its `sha256` (`core.js:165`). So the bytes the running process hashed are the manifest inside *the
   directory the running process was loaded from*. I accept that step without reservation.
2. **That step does not localise the directory, and there is a live mechanism that can move it.** The documented
   resolution order tries the **dsh installation anchor first**, then the profile anchor
   (`dsh-runtime/packages/boot/app-boot/src/profile.ts:651-654`, `resolveBundleDir`: `for (const anchor of
   [installAnchor, join(profileDir, 'package.json')])`). The installed unit runs
   `ExecStart=/var/lib/fleet/npm/bin/dsh --profile fleet …`, so the first anchor is the npm-style installation. I
   computed Node's own search path for each anchor (`createRequire(anchor).resolve.paths('@cayleb/dsh-fleet')`):
   the installation anchor's list and the profile anchor's list **share four of their head entries and differ in the
   first one**, and at least four distinct protected directories precede or equal the measured tree in at least one of
   the two lists. A second copy of the package reachable from the installation anchor would be chosen **before** the
   measured path. Nothing in the record shows that copy does not exist (nobody can read it), and nothing in the
   correlation would notice if it did — because both copies would be 0.2.2 and would hash to the same manifest.
3. **What the correlation actually buys you.** With `integrity.ok=true` and `rows_ok=true` reported alongside the
   digest, the loaded bundle's **declarative layer** is bound to the accepted one: its own manifest is byte-identical,
   and the rows the Loader mounted from its own patch files match the pinned row set and field digests. That is a
   real result and Team B should keep claiming it. What it does not bind is the **executable bytes** — the manifest
   hashes *parsed row field values* (`rows-manifest.mjs` → `buildRowsManifest`; sample pinned record
   `{"id":"fleet-ledger","kind":"mount","fields":{"name":"717f305d…"}}`), not `src/core/*.js`, `bin/*.mjs` or
   `client.js`. Those 40-odd files are precisely where the guards live, and they are exactly the surface that the
   82-file comparison covers *at the measured path only*.

So the choice, one line of risk each:

- **Choice A — accept the correlation as sufficient.** Risk: if the Loader resolved from any other directory, V3's
  82-file proof applies to a tree the service did not load, and the acceptance record would assert a provenance it
  cannot support — the failure is silent, permanent (it is written into the identity proof) and only detectable
  later by a rerun that nobody is authorised to make.
- **Choice B — require the reviewed in-process witness.** Risk: the run stays blocked for as long as no reviewed
  witness exists (none does today: `IDENTITY-V3-HOST-ROUTE-DECISION-20261101.md:34` says the insertion interface is
  unresolved), the owner's already-granted one-time approval sits unused, Stage 2 stays unverified, and the witness
  insertion itself needs a product change plus a second owner decision — so the cheaper path can end up costing more
  than the risk it removes.

A third option worth putting to Nox, because it is cheaper than the witness and stronger than the status read:
**make the claim explicitly narrower** — "the fleet service was freshly started and reports a bundle whose manifest
and mounted rows equal the accepted 0.2.2, from a tree measured byte-identical to the accepted archive, with no
`rowsManifest` override" — and record the residual (a second 0.2.2 copy with divergent non-manifest bytes) as an
accepted, named limitation of this proof. That is a defensible standard; what is not defensible is arriving at it by
calling the correlation "loader origin".

Note on process: Nox rules this, I do not. If Nox rules Choice A, the HOLD does not clear by itself — Item 5's
receipt and Item 3's FACT 1 still need fixing, and the acceptance record must carry the narrowed wording.

## Item 5 — safety: modification, access, interruption, and the window receipt

**Can anything modify the installed package? No.** The privileged surface is fixed and additive-only outside the
tree: `mkdir`/`dd`/`chmod`/`stat`/`sha256sum` on the stage, one `sudo --user fleet` caller invocation, and exact-unit
`systemctl` status/stop/start (`:215-285`). No `plugin-add`, no `restore-live`, no `integrity-drill`, no package
manager, no mount, no profile write. The caller's `ROOT` is opened read-only throughout and revalidated before the
result is returned (`PROPOSED-identity-caller-v2.py:224-225`).

**Can it widen access?** Not to the protected tree, but two cosmetic-to-real notes: (i) the stage is published
`0755` root-owned with root-owned `0444` files (`:234`, `:257-259`), i.e. the reviewed caller, core and manifest are
readable by every local user and **stay there** — automatic cleanup is deliberately forbidden (`:238-250`). They are
public, reviewed source with no secrets, and `/run` is tmpfs, so the exposure is bounded; it is still a persistent
artifact that needs a later owner-authorized removal, and nobody may `rm -rf` an unconfirmed stage.
(ii) The output files are `0600` and the bootstrap's summary/err redirections are noclobber-protected (verified), so
no private output leaks.

**Can it leave the service bad if interrupted? Mostly no — with one named exception.** The recovery machinery is
genuinely good: stop intent precedes the stop, a start is permitted only after a confirmed stop and a known inactive
state and no prior start intent, and every uncertain outcome records `…_OWNER_INTERVENTION_NO_RETRY` without
retrying (`:297-357`, `:391-397`). Signals are converted to exceptions (`:410-420`) and `subprocess.run` kills its
child on exception, so Ctrl-C/SIGTERM/SIGHUP land in the recovery path. **The exception is the one the reviewed
record already names and I confirm: SIGKILL, power loss, or an unresponsive manager between `stop` and `start` leaves
the unit down**, and `Restart=on-failure` will not save it because `systemctl stop` is an intentional stop. That is
an owner-present risk, not a code defect — but it is the reason the operator must be physically at the terminal for
the whole window, not "monitoring".

Two smaller safety observations:

- **The `TimeoutStopSec=20` boundary.** A stop that times out returns non-zero → `Blocked` with
  `stop_confirmed=False` → `STOP_OUTCOME_UNKNOWN_OWNER_INTERVENTION` and **no start** (`:326-331`). Correct
  fail-closed behaviour, but it means the worst case is "down until the owner intervenes", again at the terminal.
- **The attempt is consumed by any post-stop block.** Every check that can fail — the leaf being a symlink, a mode
  mismatch, a file with `nlink != 1`, an owner mismatch — is evaluated *after* the stop (`:376-381`). One such
  failure costs the stop/start cycle and produces no measurement. That is acceptable *only* if the result is read as
  "the measurement was taken and did not match", not as an anomaly to be retried. The full-result acceptance rule
  must say that explicitly.

**Is the no-writer window receipt plan adequate? No — it is the weakest of the four facts, and it needs four
concrete additions.**

1. **It must fail closed.** `window-receipt.sh` wraps the wrapper reads in `|| true`, builds `unit` from
   `systemctl show` without checking its exit status, and emits `null` for `n_restarts`/`invocation_id` when the
   system bus is unavailable. A window-end receipt that silently recorded nothing would still print a
   `receipt_sha256`. Add: if `active_state`, `n_restarts`, `invocation_id`, `rows_manifest_sha256` or
   `drop_dir_digest` is null/empty → exit non-zero with `RECEIPT_INCOMPLETE`, and never emit a receipt.
2. **It must be tied to the procedure's own markers.** The receipt and the procedure currently share no field. Require
   the window-end receipt to carry the procedure's `baseline.InvocationID`, `after.InvocationID` and both timestamps,
   and require them to be consistent — i.e. the service state seen at the end is the state the procedure produced.
3. **Restart counting must allow for Stage 2.** Stage 2 kills and restarts the unit from its own EXIT trap
   (`checks/stage2-check.sh:45` — "2.11 SIGKILLs fleet-dsh and starts it again", `:737` — the trap starts it,
   `:752` — `trap cleanup EXIT`; runbook line 125: "can kill/restart `fleet-dsh`"). So a rule like
   "NRestarts must be +1" is wrong; the rule must be "every restart in the window is enumerated and attributable to
   the procedure or to Stage 2, with none unexplained".
4. **It must name the subject it cannot measure.** The receipt cannot reach the compared root. So it must be
   accompanied by (a) a digest-identified hold declaration from the Lead naming every lane and every writer held for
   the interval, referenced by hash in both receipts, and (b) an explicit statement of what the receipt does *not*
   cover — because otherwise a clean receipt reads as "no writer touched the tree", which is exactly the claim the
   approval says must not be made on inadequate evidence. Either the acceptance record narrows the no-writer claim to
   "the only sanctioned writer (the wrapper) was held and no install operation occurred", or a root-side
   mtime/ctime-of-root check has to be added — and that is a protected read that is not currently authorised.

The interval's *shape* is right and should be kept: a window-start receipt before the V3 stop, the procedure's own
stage/boundary markers around the measurement, and a window-end receipt after the Stage 2 check's final restart —
with the service's own DB/state writes explicitly excluded as not touching the compared root.

## Item 6 — independence

I am the reviewer of this packet, **not the formal installed-output checker**, and I say so for the record: I did
not build, stage, edit or review v1/v2/v3 of the helper, caller, core or procedure before this task, and I have not
run the procedure, created a stage, read `/var/lib/fleet/...` by any route, probed privileges, touched the system
bus or the unit, or written anywhere except this file and disposable `/tmp` scratch used to re-hash and re-run the
tests. The prior independent reviewer (`de466966…`) and I are both ineligible to be the formal Stage 2 checker; that
must be a **distinct fresh seat** that runs the unmodified `checks/stage2-check.sh` with `FLEET_CLI` set, one run at a
time, after the identity proof is accepted — and, per the handoff, it must judge the **whole** private result
(`stage_disposition == CREATED_STAGE_PRESERVED_OWNER_INSPECTION_REQUIRED`, `reason == null`, all 82 records, a fresh
InvocationID), because V3 returns `CONDITIONAL_DISK_OBSERVATION` and exit 0 even when its stage disposition is
changed or unavailable (`:399-423`). I reproduced that code path by reading it; nothing folds the stage disposition
into `outcome`, and `main()` picks the exit status from `outcome` alone.

## Exact remaining conditions before the run

Blocking (cannot invoke until each is disposed of):

1. **Nox rules FACT 2** — either "correlation is sufficient for a narrowed, explicitly worded claim" (then the
   acceptance record carries that wording and the residual), or "witness required" (then the run waits for a
   reviewed witness that does not exist yet). *Nothing else can clear this one; it is a policy call, not a
   measurement.*
2. **FACT 1 execution route proven from the operator's own seat** — including the non-tty `sudo` behaviour for the
   five root command shapes and `sudo --user fleet`, and confirmation that no password prompt can occur after the
   stop. If it cannot be shown without changing sudoers/polkit, the run is **unrunnable under the current approval**
   (a security change is not authorised) and the Lead must take that back to the owner rather than route around it.
3. **FACT 4 receipt hardened** — fail-closed on missing fields, tied to the procedure's InvocationIDs, restart
   counting that accommodates Stage 2, and a hash-referenced hold declaration from the Lead naming all held writers.
4. **FACT 3 layout caveat recorded** — the facts file should state that the leaf-is-a-real-directory and
   single-link properties are unverified, so that a post-stop block is read as the expected fail-closed outcome
   rather than an anomaly. (No code change needed.)

Non-blocking but should be written into the decision:

5. The acceptance record must state the claim at the strength V3 actually supports, and must forbid describing it as
   retrospective memory proof, as proof that all 82 files executed, or as a Stage 2 verdict.
6. The preserved stage will remain at `/run/dsh-fleet-id-cf93107` after the run; its removal is a separate,
   owner-authorised act after identity and contents are confirmed. Nobody may recursively delete an unconfirmed
   stage.
7. The operator must be physically present for the whole window, for SIGKILL/power-loss/stop-timeout recovery.

## What the operator must verify (in this order, before invoking)

1. Run the four `sha256sum` checks from Item 2 immediately before the window and compare to the pinned values; abort
   on any difference. Include `/usr/bin/dd`.
2. Confirm all four V3 output filenames are still absent from the owner workspace (they are, as of this review) and
   that `/run/dsh-fleet-id-cf93107` is still absent — the attempt must be the first one.
3. Confirm `getent passwd fleet` → `uid=951`, `gid=950` (verified today) and that the `fleet` account is not
   interactive in a way that changes anything (it is `/usr/bin/nologin`; irrelevant to the caller, which runs it as
   an exec target).
4. **Settle the sudo question without staging anything**: from the same terminal the bootstrap will run in, prove
   that a *child that is not a terminal* can run one harmless fixed root read and one `--user fleet` read
   non-interactively (e.g. `sudo -n /usr/bin/stat -c %a /run` and
   `sudo -n -u fleet /usr/local/bin/dsh-fleet-admin version`), and check the sudo timestamp policy so the window
   cannot outlive it. If either prompts, stop and take it back to the Lead — do not pre-approve by typing later.
5. Confirm the invocation is exactly the documented one — `env -i PATH=/usr/bin:/bin LC_ALL=C python3 -I -S -B`
   reading the bootstrap from the reviewed document, no arguments, run as `cayleb` (never as root, never with
   `sudo` in front: `main()` rejects it).
6. Take the window-start receipt, then invoke **once**, then read the *whole* result (not the exit code):
   `outcome`, `reason`, `restoration`, `stage_disposition`, `stage_device_inode`, `baseline.InvocationID`,
   `after.InvocationID`, `observation_validated`. Any `restoration` other than `NEW_INVOCATION_RECORDED` means owner
   intervention, no retry.
7. Take the window-end receipt **after** the Stage 2 check's final restart, and hand both receipts, the hold
   declaration and the full private result to the distinct formal checker. Do not run Stage 2 from this seat or from
   mine.

## Handling note

This file is local to the board workspace. It contains no PIDs, inode numbers, mount ids, unit-state dumps or
permission diagnostics; the only absolute paths it repeats are ones already in the reviewed source
(`TARGET`) and in the preserved install record. If any of it is posted to the board, strip §Item 3's resolution-anchor
paragraph down to "the documented resolution order tries the installation anchor before the profile anchor, so a
second copy of the package could precede the measured path" and keep the rest as verdicts and digests.

## Appendix — commands I ran (reproducible, no privilege, no service action)

- `sha256sum` over the ten pinned artifacts and `/usr/bin/dd` → all match (table, Item 2).
- `tar -tzf /var/lib/fleet-drop/cayleb-dsh-fleet-0.2.2.tgz` → 82 regular members; `tar -xzf … package/rows.manifest.json`
  → `1bcac986…`; same for 0.2.1 → `fdcd2b72…`.
- Python shape check of `identity-accepted-manifest.txt` → 82 sorted unique `package/` lines, exact trailing newline,
  15 implicit directories.
- `python3 -I -S test_identity_procedure_v3.py` → `Ran 22 tests … OK`;
  `python3 -I -S test_identity_caller_v3_integration.py` → `Ran 12 tests … OK` (from a byte-identical `/tmp` copy, so
  the preserved workspace was not written).
- `printf x | dd of=f oflag=excl` → `invalid output flag: 'excl'`, exit 1 (v2 defect reproduced).
- `bash -c 'set -euC; exec > <existing file>'` → "cannot overwrite existing file", exit 1, content preserved.
- Drop-directory digest recomputation → `4055db47…` (matches the facts file) and both archive sidecars read.
- `node -e 'createRequire(anchor).resolve.paths("@cayleb/dsh-fleet")'` for the unit's installation anchor and the
  profile anchor → the two search-path lists (used only to establish the ordering claim in Item 4; not reproduced here).
- `getent passwd fleet` / `getent group 950` → `951:950`; `id` → the reviewer is not in group 950, which is why the
  protected tree stayed unreadable and was not attempted.
