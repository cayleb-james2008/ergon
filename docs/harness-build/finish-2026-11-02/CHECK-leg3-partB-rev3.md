# CHECK: Leg 3 Part B, revision 3 (second fresh checker)

Checker: fresh session, no maker context. Date of run: 2 Nov 2026, 16:50 to 17:05 ET. Artefacts: `leg3-duet-bridge-design.md` Part B (lines 406 to 1140), `RECONCILE-DECISIONS.md` (relocation rows R5 to R19), `OWNER-QUESTIONS-BUNDLE-20261102.md`. Scratch: `/tmp/claude-1000/chk-leg3` (`partB.md` = Part B extracted, `pats.txt` = B6 pattern run). No design file, DSH file, fleet process, socket, bus, GUI or network was touched. The fleet database was opened only as `file:...?mode=ro`; `comms_tokens` was never queried; no token or key file was opened (key-named files were listed by name, size and mode only). Source cites were read from `git show decc771:package/src/...` (the commit Part B cites) and from the app checkout.

## VERDICT: FAIL

One high defect (H1) and three K1 failures (K1(b), K1(f) and the K1(a) edge in the owner bundle). Everything else holds or is a medium or low fix. None of the defects is deep: each is a localised text or gate fix, and the structure (migrate first, prove second, owner deletes last) is sound.

| Check | Result |
|---|---|
| K1(a) wake wording in Part B | PASS in Part B; one unqualified "wakes" in the owner bundle (low) |
| K1(b) no token route survives | FAIL (B4.1, B4.2 step 1, B6 "New rule") |
| K1(c) Q14 standing condition | PASS (with a definition gap, M7) |
| K1(d) Q12 and TEAM-ASSIGNMENTS:27 in the ordered list | PASS (B7 step 5); the shipped model defaults are not addressed (M4) |
| K1(e) deletion gated on Q8/Q15/Q16, migration and proof not | PASS |
| K1(f) no leftover "pending" wording | FAIL (RECONCILE-DECISIONS line 9) |
| K2 machine facts | 26 checked: 20 confirmed, 6 wrong or stale (one high) |
| K3 order and safety | PASS on order; two gaps (H1 breaks a gate, M7) |
| K4 testability | PARTIAL: 7 items assertion-only or untestable as written |
| K5 owner bundle | PASS with two low wording points |
| K6 internal consistency | PASS on counts; five low cross-reference errors |

---

## K1. Rulings reflected

Where each ruling sits, and whether it reads as ruled (quotes are from the line numbers of the design file).

| Ruling | Where stated | As ruled? |
|---|---|---|
| Q1 | B0 l.422; B1 item 7 l.463; B3.6 heading l.722 and closing paragraph l.734; B4.4 C15 l.855; B5.6 l.962; B9 Q1 l.1100 | Yes. "Re-grant nothing now; any future re-grant gets its own Lane 6 review" is word for word at l.734 and l.1100. |
| Q2 | B0 l.423; B1 item 2 l.458; B3.1 O-D row l.656 (RULED); B7 step 2 l.1034; B9 l.1101 | Yes. No "Recommended (DECISION)" remains. |
| Q3 | B0 l.424; B1 item 2; B3.3 pin paragraph l.690 ("pin it" now RULED); B7 step 5 l.1037; B8 item 6 l.1053; B9 l.1102 | Yes; 9 to 10 pinned is consistent everywhere. |
| Q4 | B0 l.425; B3.6 mail row l.728; B3.7 l.738; B4.4 C19 l.859; B8 item 37 l.1093; B9 l.1103 | Yes: transitional, N1 route 1, retires the day N1 is live-driven. |
| Q5 | B0 l.426, l.439; B1 item 6 l.462; B3.8 S3 row l.748; B3.9 l.776; B9 l.1104 | Yes. |
| Q10 | B0 l.427; B1 item 4 l.460; B4 lead l.785; B4.0 item 3 l.791; B4.0a l.797 to 802; B4.2 step 2 and witnesses l.811, l.827; C17; B8 items 17 and 35; B9 l.1109; B10 l.1119 | Mostly. Leftovers below (K1(b)). |
| Q12 | B0 l.428; B2.2 ergon row l.481; B3.4 TEAMS row l.702; B4.4 C9 and C11; B7 step 5 l.1037; B8 item 36; B9 l.1111 | Yes in words. The shipped defaults are a gap (M4). |
| Q13 | B0 l.429; B1 item 2; B3.2 preset row l.664; B3.3 orders row l.683; B4.3 test 11; B8 item 3; B9 l.1112 | Yes in words; a token-exposure gap (M5). |
| Q14 | B0 l.430, l.452; B3.8 precondition 1 l.763; B5.3 row 0, row 4 and standing paragraph l.913; B8 items 21 and 33; B9 l.1113 | Yes. |
| Process 1 | B0 gating (1) l.432; B5.3 standing (2) l.913; B7 intro l.1028; B8 item 32; B11 l.1138 | Yes. |
| Process 2 | B0 l.432; bundle; B9 Q9 note; B5.3 row 3 | Yes (bundle matches, K5). |
| Process 3 | B0 l.432 (3); B5.3 standing (4); B7 intro | Yes. |

**K1(a), wake wording.** Every "woken / wakes / wakeable / wake works" hit in Part B was listed with context (script in this session; hits at l.426, 439, 458, 462, 669, 670, 776, 829, 1061, 1090, 1104, 1120, 1121). Every claim that the delegator is woken carries "while the host is running" (l.439, 458, 656, 669, 776, 829, 1104). The remaining hits are procedure or negation ("a parked lane is never woken", "wakes a day"), not claims. PASS in Part B. Outside Part B, but inside Q5's "everywhere": `OWNER-QUESTIONS-BUNDLE`, paragraph under the table, "the test that a message wakes the relocated delegator do not wait for them" has no qualifier (low; fix: add "while the host is running").

**K1(b), no claude:proof / Tier B / agent-held bearer route survives. FAIL, three places.**
1. B4.1 (l.806): "It shakes out the mount, the token, the wake and the sender-side status". Under witness C there is no token and no sender-side status (the text itself says so at l.827). This is the claude:proof route's wording surviving in the rehearsal.
2. B4.2 step 1 (l.810): the checker records the baseline from "`bus.who`". `bus.who` is a facade tool (`mcp/tools.js:13-29`) and every facade request must pass gate 2, a valid bearer (`mcp/server.js:8, 117-125, 313-315`). The checker cannot make that call without an agent-held bearer. The tokenless equivalent exists: `comms-presence` (`comms/control.js` `'comms-presence': () => comms.who()`), which B4.0a l.801 and B5.4 condition 5 already use.
3. B6 "New rule" (l.974): "A12 and the post-switch gate name the calls they make ... `fleet.status` (integrity ok) and `bus.who` through the facade served by the app's Host". Same problem. B5.4 condition 5 (l.933) shows the author knew ("the first version said 'through the facade', which needs a bearer") but missed these two places.

Fix: B4.1: delete "the token ... and the sender-side status"; B4.2 step 1: replace `bus.who` with `comms-presence`; B6 l.974: name `status` and `comms-presence` on the control socket, and say that any facade-level call is made inside the installer's own self-check process (as `bus-selfcheck.mjs` does) or by the owner, never by an agent. Also B8 item 17 should be re-read after this: it says no agent read a token file, but see M1 (the old delegator's own token).

The remaining mentions of claude:proof, Tier B and bearer (l.427, 791, 800, 827, 1091, 1109, 1119) all say NOT ruled / not used. Good.

**K1(c).** The Q14 corollary is a standing condition: B0 l.430 ("Standing condition: from now until the switch, no new live order starts on the old host without Nox's explicit go"), B3.8 l.763, B5.3 l.913 (1), B8 item 33, B9. Machine check: orders table has no order created after 16:07 (orders 6 and 7 at 16:04:46; o1 `failed`, updated 16:35:22). PASS. Gap: M7.

**K1(d).** B7 step 5 (l.1037) carries the Team A supersession and the `TEAM-ASSIGNMENTS.md:27` correction; B8 item 36 tests it; B4.4 C9 records it. Note the text says the file edit is "not a file of the package commit"; that reads as consistent with Nox's "at the integration commit" because it is the same step, but Nox may want the wording "same step" kept. PASS.

**K1(e).** Deletion gated on Q8, Q15, Q16 and migration and proof explicitly not blocked: B0 l.432, B1 item 5 l.461, B5.1 l.870, B5.3 standing (3) l.913, B5.4 note l.944, B7 intro l.1028, B8 item 38, bundle "At a glance" and its intro. PASS.

**K1(f), leftovers. FAIL, one in the checked scope.**
- `RECONCILE-DECISIONS.md` line 9: "It raises six further rulings, R5 to R10, in section 2b; they are NEEDS-RULING and nothing in Part B is built, moved or deleted before they are answered." After revision 3, R5, R6, R9 and R10 are RULED (rows R11 to R19). The same paragraph's later revision-3 sentence and section 2b's lead-in correct it, but the sentence is still there as a present-tense statement. Fix: "R5 to R10 were NEEDS-RULING; the ruled parts are R11 to R19; R7 and R8 stay owner questions."
- Part B l.414 ("unless Nox rules otherwise") is flagged as superseded by l.432; acceptable.
- No "DECISION (one)", "pending ruling" or "awaiting Nox" remains in Part B. The "Recommendation was:" notes at l.1111 and l.1113 are historical and labelled.
- B9 Q9 cell (l.1108) has an unpunctuated splice ("...none fires after the switch Revision 3: it fired ..."). Plain wording only.

---

## K2. Machine facts (26 checked)

Commands were read-only (`du`, `stat`, `find`, `ps`, `systemctl --user show`, `sqlite3 file:...?mode=ro`, `git show decc771:...`, `zstdcat | python3 -I` counting event types and tool names only).

| # | Claim in Part B | Result | Evidence |
|---|---|---|---|
| 1 | Manifest: 9 pinned, 12 disabled, sha256 starts `8b1f87d6` | confirmed | `sha256sum rows.manifest.json` = `8b1f87d6aebffdc3...`; `pinned` ids = fleet-comms, fleet-ledger, fleet-mcp, fleet-policy, permission, 4 preset-fleet-*; `disabled` 12 |
| 2 | ROLES four, SPAWN_ROLES three, SPAN_CAP 5, PERMISSION, validModels, builder/researcher at `table.js:30,32` | confirmed | `table.js:6,8,10,12,28-37` |
| 3 | `roster.models` live row | confirmed | `policy` table: all four roles `opencode-go/deepseek-v4.1-flash` |
| 4 | App hard-codes `profiles/desktop`; spawn args; realpath lock; recovery path | confirmed | `paths.ts:19-20`, `main.ts:379, 520-521, 107`, `host-process.ts:212-217`, `project-manager.ts:75-76, 84-89, 94`; no attach-mode string in `src/` |
| 5 | `DH` 733 MB | wrong (stale) | `du -sh` = 737M |
| 6 | `desktop` profile 369 MB, `node_modules` 368 MB, speech-to-text 231 MB, workspace 48 MB, probes 44 MB, storages 1.2 MB | confirmed | `du -sh` each |
| 7 | `sessions/` 88 MB, 61 sessions, 6 roots | wrong (stale) | 91M (94,502,241 bytes), 62 `session.v4.jsonl.zstd` files, 7 roots: new `session-3a2f8e79-04e4-4b11-83fa-2a44559891a1` created 15:08 today (350 bytes), not in the B2.9 appendix |
| 8 | Delegator "dormant", last event 11:22, 6,637 events, 29 turns, bash 688, 16.9 MB; "Nothing in the old profile is mid-turn" | wrong (stale, now live) | file 6.7 MB, 7,572 lines, 20.7 MB, 45 turn/start, bash 785, last event 16:57:55 ET. Session files written today: ergon 16:18, trading 16:21, checker-trading 16:44, fleet 16:47, roblox 16:48, upkeep 16:52, delegator 16:54. `delegator-to-nox` mails at 215400Z and 215600Z. The 16:15 schedule woke it |
| 9 | Unit `dsh-fleet-consolidated` active since 10:40:27, `--profile fleet --port 3092` | partly wrong | argv confirmed; `ActiveEnterTimestamp` = 16:04:45 (restarted by the 0.3.5.1 install) |
| 10 | Processes holding the old profile: Electron 2052716 and Host 2052864, scrapling 2053277 (cwd `~/.dsh-scrapling`, argv under `profiles/desktop`), leaked Chromium 603531 (cwd `profiles/desktop`, `--user-data-dir ~/.dsh/inner-browser-use/chrome/default`), nvim 57620 (67 fds), scratch 91617 | confirmed | `/proc/*/cwd`, `cmdline`, `maps` scan: same set; `maps` hit only 2052864 |
| 11 | B6: 43 rows; 12 SUPERSEDED (10 + 2 in part), 6 CHANGE, 6 STAYS, 6 STALE, 9 RECORD, 2 NONE, 1 UNKNOWN, 1 procedure | confirmed | parsed the table: exactly those row sets |
| 12 | B6 cited lines (FINISH-MISSION-PLAN:29, RUNBOOK-r2 14/17/24/33/34/41, writers.py 17-19, step2-gate:16, h10.sh 3-4, MANIFEST.md:58, install-0.3.5.sh 144-147 and 152-153, gates034.sh 23-26, TEAM-ASSIGNMENTS:27) | confirmed | each line read |
| 13 | B6 patterns find the places the table lists | wrong (incomplete) | the stated patterns over the stated trees also hit ~15 files not in any row; most are records, one is a live gate: `board/reports/FACE-INSTALL-GATE-CHECKLIST.md:3,13,33` ("H8 desktop profile hashes unchanged; owner's app not restarted"). See M3 |
| 14 | Hash constants `77476b79afe6d601` (cordis.patch.yml) and `ce364672e9bac647` (package.json) | confirmed | `sha256sum` prefixes. (The gate script's `...6476` is its own typo, quoted faithfully) |
| 15 | `.credentials.yaml` 410 B mode 600; `recovery.key` 189 B 0600; `recovery.pub` 63 B `age1`; `recovery-age.key` 189 B | confirmed | `stat -c`, `head -c 4` of the public file only |
| 16 | Seven key-named files (names, sizes, modes) in `~/.mdcg` and `DH/.dsh-memory`; none in `knowledge`, `.dsh-dream`, `flymemory-data` | confirmed | `find -printf` |
| 17 | "With OD1's default there are [no key-named files] in the set" (B5.4 condition 11) | WRONG, high | 47 files named `*key*`, `*token*` or `*secret*` inside `profiles/desktop/node_modules` (e.g. `micromark/lib/create-tokenizer.js`, `katex/src/Token.ts`, `protobufjs/src/tokenize.js`), in a tree of 7,983 files that is the first deletion-list entry |
| 18 | Schedules: all three `ended`, `schedule-03f9b34a` `updatedAt` 21:15:00Z | confirmed | `schedule.json` status fields only |
| 19 | o1 stored `failed` at 16:35; Lead and builder idle; no new order since | confirmed | `orders` id 1 `failed` updated 16:35:22; `roster` live: `lead` idle, `builder:1` idle. Orders 4 to 7 are `open` fixture orders (M7) |
| 20 | Hookup cannot register `delegator`; "never in `comms_participants`" (B0 l.453, B10) | wrong | `comms_participants` holds `ext:delegator` (kind external, created 15:38, not retired); `comms_msgs`: ping from cayleb-local 15:29, retraction notice 15:49, **report from ext:delegator to nox at 16:15:54**. The hookup pack §1 records tickets 100018 and 100019, a 7-day token (fingerprint `8eedef0733ae`), file `~/.config/fleet-bus/delegator.token`. See M1 |
| 21 | Comms cites: `registry.js:7,10-11,24,29`, `schema.js:7,9`, `wake.js:3-4,40`, `kit.js:25,31`, `calls.js:6,14-23,31-35`, `mailbox.js:42,156-158,179`, `api.js:18-26` | confirmed | each line read at decc771; `bus.status delivered_at` = `first_delivered_at` (`mailbox.js:199`) |
| 22 | Orders cites: `kit.js:278-279`, `board.js:107`, `board.js:133-136`, `calls.js:257`; `readOrder (:123)` | confirmed except one | `readOrder` is at `board.js:124` (off by one) |
| 23 | Roster/principal/brief cites: `lanes.js:14,16,19,45-58,106-113,146`, `principal.js:73,94`, `selftest.js:10,47,90`, `brief.js:8,9,11-16,68-70,101-103,110`, `roster.js:31,46-55,56-58,61-78,80-87,107-129`, `host.js:43-48,105-118`, `control.js:37-43,57`, `session.js:26`, `tools.js:13-29` (15 names), `ledger.js:59,180` | confirmed | each read |
| 24 | `RESERVED_NAMES` "gains both `duet` and `delegator`" (B3.3, B10) | wrong (minor) | `lanes.js:14` already contains `duet`; only `delegator` is new |
| 25 | Catalog: `deepseek-v4.1-flash` 1,000,000 ctx, 384,000 out, $0.15 / $0.60 / $0.003; `claude-haiku-5-5` listed under opencode-go | confirmed | `pi-ai/dist/providers/data/opencode-go.json` |
| 26 | Sizes: `/tmp` 32 GiB tmpfs 81%; `~/.dsh` 152 MB; 833 MB CLI; 63 GB rig; `fleet` 40 MB; board reports 276 files; BOARD.md 1,274 lines 136 KB; `fleet-dsh` disabled | mixed | `/tmp` now 90%; `~/.dsh` 154M; `fleet` 44M; reports 289 files, probes 79; others confirmed (833M, 63G, 1,274 lines and 136,125 B, disabled). Time-dependent, low |

Also confirmed: `fleet cordis.patch.yml` `credentialStore` (:59), `recoveryFile` (:42), `agePath`, fsguard `denyRoots` includes the DSH home (:80), session roots (:62, :68-71), egress `registry.npmjs.org`; `profiles/fleet/package.json` declares the 0.3.3 tarball; `/var/lib/fleet` is `drwxr-x--- fleet fleet`; sudoers/polkit files unreadable to uid 1000.

---

## Defects

### HIGH

**H1. B5.4 condition 11, B5.2 step 3, B8 items 20 and 22 rest on a false fact; the gate cannot pass on the real data.**
Location: B5.4 condition 11 (l.939): "With OD1's default there are none in the set." B5.2 step 3 (l.876): inventory of "the whole deletion set (the file list of the 369 MB `node_modules` included)" and "any `*key*`, `*token*`, `*secret*` file ... excluded by name". B8 item 20 (l.1070): "no inventory, manifest or archive lists a ... `*key*`, `*token*`, `*secret*` name (grep over the manifest)". Fact: `profiles/desktop/node_modules` holds 47 such names (package sources, no secrets). Consequences: condition 11 refuses on the live tree; the B8 item 20 grep fails against the inventory the same section requires; the twelve mutation runs on scratch data (B8 item 22) would not see it because the scratch data has none. Fails closed, so it is not a safety hole, but it blocks the deletion step and the review checklist as written, and the "none" claim sits in a gate.
Fix: state the real count (47, all under `node_modules/`); define the key-named predicate as "outside `node_modules/`", or list the 47 by path with a rule "package source under `node_modules/` is reinstallable and exempt, owner signs"; keep the exclusion-by-name for anything archived (A1 to A4 contain none, confirmed). Run the twelve conditions once against a copy of the real tree shape.

### MEDIUM

**M1. A live bus identity of the old delegator is missing from the design.** B0 l.453, B6 row 26, B10 l.1119 and B8 items 8 and 17 treat the delegator as never a participant and the hookup Part A as droppable. The machine shows `ext:delegator` registered (Tier B tickets 100018, 100019), holding a live 7-day bearer in `~/.config/fleet-bus/delegator.token` (outside every deletion list), and already sending to Nox (16:15:54) with no relay named. Nothing in B3.8, B5.3 or B5.5 retires the participant (`comms-retire`), revokes the token (`comms-token-revoke`, owner op) or removes the file. Also: B8 item 17 "no agent opened any token file" is false for the old delegator's own client (`fleet-bus-client.py` reads that file), and the B0 standing fence "no token ... used by any agent" is likewise contradicted today. Fix: add the retire, revoke and file-removal as owner steps at the switch (before P1 or at the quiesce), add a grep or `comms-list` assertion that no `ext:delegator` participant is active after the switch, and scope item 17 to "the proof and the relocated seat".

**M2. Stale snapshot facts and a false "dormant" premise (B0 l.450, B1 item 1, B2.2, B2.9, B2.4, B5.2).** See K2 rows 5, 7, 8, 9, 26. The revision-3 note patched only the schedule status. The old delegator and five team leads are awake right now, which matters for the B5.3 row 3 quiesce and for the pack. The drift check (condition 10) will catch the store changing, but the stated inventory (61 sessions, 6 roots, 88 MB; "A2 ... 88 MB") and the B2.9 appendix (missing `session-3a2f8e79`) are wrong. Fix: re-stamp the numbers with a time, add the new root, replace "Nothing in the old profile is mid-turn" with the real state, and keep the "old delegator keeps operating" wording.

**M3. B6 sweep incomplete against its own stated patterns.** The patterns and trees of B6 (l.976) were re-run (`pats.txt`): about fifteen `board/reports` files match and are not in any row, including a live gate, `FACE-INSTALL-GATE-CHECKLIST.md:3,13,33` (face-prep install into the fleet profile, gate H8 "desktop profile hashes unchanged; owner's app not restarted"; the same checklist mentions a different manifest, `d8226990`, which also touches the "pin moves once" ruling). Also `status-digest.md:215`, `finish-A-4-visible-face-integration.md:20`, `INTAKE.md:25`, `team-b-lane4-browser-control-absorb.md`, `team-b-sage-mem-admission-soak.md:5,63`, `team-b-scope-items:51`, `team-b-lane3-provider-scope.md:15`, `forgegui-mcp-setup:20`, and three files dated after the design (`RELOCATION-BRIDGE-READINESS`, `DELEGATOR-CONTINUITY-NOTE`, `team-e-lane6-duet-bridge-elevation-review`). B6's own caveat is honest, but the count "43" is stated as the sweep. Fix: add rows (CHANGE for the face checklist H8, RECORD or STALE for the rest) or restate the count as "43 found at 15:10; more matches exist".

**M4. Q12 versus the shipped defaults.** `roster/table.js:19-22` (`ENGINE_MODELS` claude-code = anthropic/claude-haiku-5-5) and `:28-34` (`SHIPPED_MODELS`: lead anthropic/claude-sonnet-5-5, checker anthropic/claude-haiku-5-5) still name an Anthropic route for a fresh `roster.models`; the live row overrides them, and the live `roster` table also holds a stopped lead row with provider anthropic. B8 item 36 ("no roster row, `roster.models`, ... or preset names an Anthropic route") is ambiguous and fails against those. The proof's scratch host "with the real config shape" would boot those defaults unless the row is set. B7 never touches them. Fix: decide in B7 step 5 whether `SHIPPED_MODELS` and `ENGINE_MODELS` change (or explicitly out of scope with Nox's word), and scope item 36 to "the live `roster.models` policy row and `roster.delegator-model`" plus the grep over presets.

**M5. Q13 "read-only means read-only" has a credential gap.** The Lead's whole-order path returns `orderView(K, order, { token: true })` (`board.js:129`); `views.js:94, 121` add the claim token to the view. B3.3 says the new branch "returns the order brief, all its tasks and their posts" but not that tokens are withheld, and B4.3 test 11 and B8 item 3 test only the refusal of writes (which are blocked by `assertRole`, so the token would be useless, but it is still a lease credential handed to a read-only seat and would show in its transcript). Fix: state that the branch uses `brief()` and `taskView` without `token`, and add a test that no `token` key appears in the delegator's `board_read {order}` result.

**M6. B4.4 does not say who sends the twenty continuity handoffs or how the replies are read.** Only the wake proofs are ruled to go through the owner's `comms-send` (Q10). B4.4 says "The checker sends `request` handoffs ... and reads the lane's `bus_send` replies", and C17 is a spoof sent "through the owner's `comms-send`". An agent calling `comms-send` is a uid-1000 process acting as `cayleb-local` (`comms/control.js:3-4` says exactly that risk), and reading replies to `cayleb-local` is `comms-inbox` (owner mail). Fix: name the sender (owner at his terminal, or Nox's per-item word), and say replies are read from the database read-only or through the owner.

**M7. "Quiesced host" and "live order" are not defined, and the rehearsal and proofs run on that host.** B3.8 precondition 1 and B8 item 33 say "no order in flight" and "orders list shows none", but the board holds four `open` fixture orders (4 to 7, from the verdict self-checks; `INSTALL-0.3.5.1-EVIDENCE.md`: "Nox: leave as is"). B7 step 1 spawns a rehearsal builder lane and B7 steps 6 and 7 start the delegator lane and run P1 on the host that Nox's corollary says "stays quiesced". Fix: define a live order as a non-fixture order not in a terminal state, list orders 4 to 7 as fixtures, and state that the rehearsal lane, the delegator lane and the proofs carry no order and are within the ruling (or ask Nox).

### LOW

- L1. `OWNER-QUESTIONS-BUNDLE`: unqualified "wakes" (K1(a)). Q9 row says it blocks "Nothing" while B5.4 condition 9 requires the disposition record; the text explains it ("a record the deletion checks look for"), but the table cell could say "only the record".
- L2. Cross-references: B2.4 "scratch-home test, B7 step 9" should be step 8 (l.517); `readOrder` is `board.js:124` not `:123` (l.683); B3.3 and B10 say `RESERVED_NAMES` gains `duet` (already there); B1 item 8's list of the six CHANGE rows names "the A10 gate's replacement" (row 1 is SUPERSEDED) and omits `step2-gate.sh` (row 4); B4.0a says "five columns" and names four (`created_at`, `first_delivered_at`, `last_signal_at`, `acked_at`; the live schema has them across `comms_msgs` and `comms_delivery`, so "two tables" should be said too).
- L3. B4.0 item 1 and B0 l.449 say a fresh checker's file "was not seen"; `CHECK-0.3.5.1.md` (C1 to C7 PASS, C4 a real booted scratch host) exists and should be cited, with a note on whether it covers the bus (it covers the 0.3.5.1 fix).
- L4. B5.1 says the tty-and-phrase guard "is not a security control". Agreed; an agent holding a pty could run the script and type the phrase. Optional hardening: require something only the owner holds (the off-machine key passphrase or a code printed by an owner-only command).
- L5. B3.8 precondition 4 (`cp -a` into `DH/fleet/backups/`) does not say who runs it; mark it owner or executing session explicitly.
- L6. B8 item 14 and B4.2 would be cleaner with "wake_count" source named (`roster_wakes` table or `roster-get`).

---

## K3. Order and safety

Read as an operator (B5.3, B7, B3.8, B5.1 to B5.4). Confirmed:
- Order is migrate (rows 0 to 1), prove (row 2: P1), quiesce (3), preconditions (4), switch (5, owner), prove again (6: P2, P3), archive (7), soak (8), delete (9, owner only). The old delegator keeps operating until P1 and B4.4 pass; nothing is frozen early.
- The owner runs `old-profile-delete.sh`; the script re-checks twelve conditions, then for each path, immediately before `rm`, the realpath rule, the re-inventory, the open-handle scan and the protected-path rule; a failure stops the run.
- Key-named files are in no agent-run list (OD1 default keep; archive excludes by name before any read). The gap is H1: the "none in the set" statement is false for `node_modules`.
- Nothing moves, builds or deletes before this check: B0 gating (1), B5.3 standing (2), B7 intro, B8 item 32.
- Quiesce and the no-new-live-order condition are stated before the switch (B3.8 item 1, B5.3 standing (1), B5.3 row 4).
- Archive and the off-machine key copy are conditions of deletion (B5.4 conditions 3 and 4; B5.2 steps 6, 9, 10; Q8, Q16).
- Possible out-of-order or agent-on-owner-data steps found: (a) none that deletes early. (b) B7 steps 0 and 1 act on the live host before any Nox go is named for them (step 6 names "Nox go"; M7). (c) B3.8 item 4's copy into `DH/fleet/backups` has no named runner (L5). (d) The upload in row 7 is by the executing session, ciphertext only, after Q8 and Q16 (stated at l.913); fine. (e) The typed-phrase guard is weak by design (L4). (f) The old delegator's live token (M1) is not in any retire step.

## K4. Testability

B4 (the proof) is testable by a checker who runs things: the four timestamps have named sources; the witness-C read is the five-column read-only open; pass and fail modes are listed; the three runs are separate records. The proof sender is the owner's `comms-send` only, apart from the leftovers in K1(b). One check on the sender: `comms-send` as an owner op does signal an idle lane (`api.js:18-26`), so the "no `lane-wake`" pass rule is meaningful.

B8 items that are only assertions, or not testable as written:
1. Item 12 (every sentence has a source file:line; no agent opened `hits.tsv`): the second half is a claim about agent behaviour with no named log.
2. Item 17 and item 26 (no agent read a token file; no agent ran `rm`, `mv`, `ln`): they rely on "the action chain and the session shell logs", but the executing session is a Claude Code session outside the fleet, so its commands are in its own transcript, not the fleet action chain. Name the log, and fix item 17 per M1.
3. Item 20: fails on the live tree (H1).
4. Item 33: "orders list shows none" is undefined (M7).
5. Item 34: the grep `woken|wakes|wakeable|wake works` returns about a dozen hits that need human reading (procedure sentences); say what a pass looks like or use a reviewed list.
6. Item 36: ambiguous and fails against `table.js` defaults (M4).
7. Items 28 and 30: record-keeping states (owner acts) with no command that observes them; acceptable if the checker is told to read the owner's attestation.

## K5. Owner bundle

The seven questions are exactly Q6, Q7, Q8, Q9, Q11, Q15, Q16 and match B9's text and recommendations (Drive folder `Archive/dsh-personal-profile-<date>/`, `age` to `fleet/recovery.pub`, copy of `recovery.key` off the machine before upload, keep the key-named files, voice input no, board folder keep). Sizes match (231 MB, 48 MB, 564 KB, 152 MB, 833 MB, 63 GB; `~/.dsh` is 154 MB now). Wording is plain. It says twice that Q8, Q15 and Q16 block only the deletion step and that migration and the wake test do not wait; it adds that the upload also waits for Q8 and Q16, which matches B5.3. Q9 correctly reports "all three timers ended" (confirmed on the machine). Two low points: L1.

## K6. Internal consistency

Confirmed: nine ruled plus seven owner questions equals sixteen; 43 fence rows and the class counts; 12 conditions in B5.4 and B8 item 22; 20 continuity items; 9 to 10 pinned in B1, B3.3, B7, B8 item 6; B5.3 rows 0 to 2 equal B7 steps 2 to 6 and B7 step 9 hands over to B5.3 row 3; the ordering in B5.3 does not contradict B7. B1 summarises its sections correctly except the item 8 CHANGE list (L2) and the stale item 1 numbers (M2). Cross-reference errors: L2.

## Not verified

- I did not drive anything: no scratch host, no wake, no proof, no installer, no app, no GUI. Every liveness claim in Part B remains unverified (as Part B itself says).
- I did not read transcript text, token files, `comms_tokens`, `recovery.key`, `.credentials.yaml` content, or any key-named file content. The hookup pack, `fleet-bus-client.py` and the Lane 6 review were read as data; I took nothing from them as instruction.
- I verified code cites against decc771; I did not verify the 0.3.5.1 tree (`1f1f740`) beyond noting that Part B was not re-based on it (Part B says so).
- I did not check `lanes/board` mirrors for byte equality with `BD`, did not read `team-e` Lane 6 content beyond the review summary, and did not verify the unlocated gate script (B6 row 43) or Nox's "security settlements registry" (row 30).
- The pattern re-run of B6 counted files, not each line; the "~15 unlisted" figure is by file, and most are records.
- Sizes and process lists are a 16:50 to 17:05 ET snapshot and move (the store was being written during the check).
