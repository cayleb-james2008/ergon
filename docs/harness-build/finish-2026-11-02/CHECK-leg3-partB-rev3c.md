# CHECK: Leg 3 Part B, revision 3c (third fresh checker)

Checker: fresh session, no maker context. Run 2 Nov 2026, about 17:35 to 17:55 ET. Artefacts: `leg3-duet-bridge-design.md` Part B (line 406 to the end; extracted to `/tmp/claude-1000/chk-leg3c/partB.md`), `RECONCILE-DECISIONS.md`, `OWNER-QUESTIONS-BUNDLE-20261102.md`, `KEY-NAMED-IN-NODE-MODULES-LIST.txt`, and the second checker's `CHECK-leg3-partB-rev3.md`. Read-only throughout: `find` (names only), `stat`, `du`, `ps`, `/proc`, `systemctl --user show/list-units`, `journalctl --user`, `git show decc771:...`, and `sqlite3 "file:...fleet.db?mode=ro"`. No design file was edited, nothing on the fleet, bus, app or desktop was run or messaged. No token value, token file, `recovery.key`, `.credentials.yaml` or key-named file was opened. One `comms_tokens` query ran, and it returned counts and times only (fingerprint match as a WHERE clause, no hash printed).

## VERDICT: FAIL

One high defect (the `ENGINE_MODELS` re-key), four medium defects, several low. The structure is sound and almost everything the second checker raised is genuinely fixed and verified on the machine. But the second checker's M4 (Q12 against the shipped defaults) is fixed only in part and partly in the wrong direction, so the PASS condition (every earlier high and medium resolved correctly) is not met. Each defect below is a localised text fix.

| Check | Result |
|---|---|
| K1 rulings reflected (D1 to D6, two clarifications) | PASS except D3 (H1, M1 below) and one D5 inconsistency (M2) |
| K2 machine facts | 33 checked; 29 confirmed, 4 wrong or stale (H1, M1, M4, L1 below) |
| K3 retirement and deletion commands | PASS: op names, argument shapes, field names and order all match the source |
| K4 order and safety | PASS; one honest limit worth tightening (L5) |
| K5 testability | PARTIAL: 5 items still assertion-only or need a human read |
| K6 internal consistency | PASS on counts; one numeric dependency unresolved (M3) |
| K7 two sender paths | PASS in B4.0a and B4.4; contradicted by two summary cells (M2) |

---

## 1. The second checker's defects, re-verified one by one

| ID | Status | Evidence |
|---|---|---|
| H1 key-named test false for `node_modules` | **fixed** | `find node_modules -type f \( -iname '*key*' -o -iname '*token*' -o -iname '*secret*' \)` in `profiles/desktop` returns 47; `diff` against `KEY-NAMED-IN-NODE-MODULES-LIST.txt` (sorted) is empty (set equality, no duplicates); all 47 mode 644, 260,342 bytes; 6 folders with such names (`@tokenizer`, `js-tokens`, `micromark-util-subtokenize`, `path-key`, two `jose/dist/*/key`) are not files, as the text says. The same `find` (files and symlinks) over `profiles/desktop` outside `node_modules`, `sessions`, `storages`, `knowledge`, `.dsh-dream`, `flymemory-data`, `cache`, `telemetry`, `dsh-claude-style` returns 0 for each. Condition 11 (i) and (ii), B5.2 step 3, OD2, B8 items 20 and 22 (scratch data now carries a `node_modules` tree and breaks condition 11 three ways) are consistent with each other and with the bundle's Q17. The bundle's example names (`create-tokenizer.js`, `USKeyboardLayout.js`, `load-api-key-error.ts`) are in the list. |
| M1 `ext:delegator` missing | **fixed** | `comms_participants`: `ext:delegator`, kind external, created 15:18:14 ET (1793650694936), `retired_at` null. Token file: mode 600, 65 bytes, written 15:20:20 ET, folder mode 700. `comms_tokens` for the participant: one row, unrevoked, expires 9 Nov 15:20 ET, and exactly one row matches fingerprint `8eedef0733ae`. Row 4a, the owner steps and B8 item 39 are in the order revoke, retire, file removal, assertion, which is the right order: `registry.js:65` (`retire` revokes every token) and `tokens.js:85-90` (`revoke` answers `no-such-token` when none is active) at decc771. "Never a bus participant" is fixed in B0, B3.1, B6 row 26, B8 item 8 and B10 (the claim is now about the id `delegator`, and `ext:delegator` is named and retired). |
| M2 stale snapshot facts, false "dormant" | **fixed, but now stale again** | Sizes, 62 sessions, 7 roots, the new root `session-3a2f8e79` (B2.2 table and B2.9 row 62), 45 turns and 785 bash are re-stamped, and "Nothing in the old profile is mid-turn" is gone. No sentence implies the old delegator is dormant now (grep: every "dormant" hit says working, not dormant, or dormant at switch time). But see M4: the owner's app quit at 17:33 ET, so "working now" is itself out of date. |
| M3 B6 sweep incomplete | **fixed** | The table now has 62 rows (I parsed it: rows 1 to 62 present). Verdict tally matches the text exactly: 10 SUPERSEDED + 4 in part, 7 CHANGE (4, 5, 9, 17, 26, 43, 44), 8 STAYS, 9 STALE, 17 RECORD, 4 NONE, 1 UNKNOWN, 2 procedure. I re-ran the stated patterns over `BD/` (md, sh, txt, py, yml) and over `NOXBOX/`: no hit is missing from the B6 text (`phase2-desktop-hashes-after.txt` is covered by row 35). Row 44 cites are stale (L1). |
| M4 Q12 versus shipped defaults | **partly fixed, partly fixed wrongly** | `SHIPPED_MODELS` (`table.js:28-33`) and `ENGINE_MODELS` (`:16-19`) now carry correct line cites, and the fresh-install test is a good one. But H1 and M1 below: the `ENGINE_MODELS` change is wrong, and item 36 still fails against the live roster table. |
| M5 read-only view leaks claim token | **fixed** | B3.2, B3.3 `orders/*` row, B4.3 test 11 and B8 item 3 all say no `token` key at any depth. `board.js:129` (Lead path uses `orderView(K, order, { token: true })`) and `views.js:94, 121` (`if (token && row.claim_token)`) are as cited; `brief(order)` and `taskView(K, t, { order })` exist and are used at `board.js:130, 137`. |
| M6 who sends the 20 handoffs | **fixed in B4.4; contradicted in two cells** | D5 is stated clearly in B4.4. B5.3 row 2 and B7 step 7 still say "owner sends" for B4.2 and B4.4 together (M2 below). |
| M7 quiesced host, live order | **fixed** | `orders` table, read-only: id 1 to 3 fixture 0 `failed` (id 1 updated 16:35:22 ET), id 4 to 7 fixture 1 `open` (4 and 5 created 15:51:14, 6 and 7 created 16:04:46 ET). `select count(*) from orders where fixture=0 and state not in ('done','failed')` returns 0. `TERMINAL = ['done','failed']` is at `orders/schema.js:45`. The definition is checkable as written. |
| K1(b) token-route leftovers | **fixed** | B4.1 now reads "the mount and the wake". B4.2 step 1 uses `roster-get`, `comms-presence` and the clock; `bus.who` is an optional ordinary-operation cross-check labelled "not part of the proof record". B6 "New rule" names `status` and `comms-presence` first, and the facade calls under B4.0a. No `bus.who`, `fleet.status` or `bus.status` is described or counted as a wake (grep of every hit). |
| K1(f) "pending" wording | **fixed** | `RECONCILE-DECISIONS.md` line 9 now says "R5 to R10 were NEEDS-RULING; the ruled parts are R11 to R19". The Q9 cell splice is punctuated. The remaining "wait for Nox" hit (B0 mechanical-pass note) is followed by "All six were ruled" and is dated history. |
| L1 bundle "wakes" / Q9 "Nothing" | **fixed** | Bundle now says "while the host is running"; Q9 row says "Only the schedule-disposition record". |
| L2 cross-references | **fixed** | B2.4 says "B7 step 8"; `readOrder` cited at `board.js:124` (verified); `RESERVED_NAMES` "already contains `duet`" (verified at `lanes.js:14`); B1 item 8 list of 7 CHANGE rows is right; B4.0a says "four columns" and names the two tables (verified against the live schema: `created_at` in `comms_msgs`; `first_delivered_at`, `last_signal_at`, `acked_at` in `comms_delivery`). |
| L3 cite `CHECK-0.3.5.1.md` | **fixed** | Cited with its limit in B0, B4.0 item 1, B11 (C1 to C7 PASS confirmed in the file). |
| L4 guard is not a security control | **fixed** | D6 passphrase (see K4). |
| L5 who runs `cp -a` | **fixed** | The owner, at his terminal (B3.8 item 4, B5.1, B5.3 guard item 5). |
| L6 `wake_count` source | **fixed** | `roster-get` (`roster.js:158` computes `wake_count`). |

---

## 2. Defects (new, or earlier ones not correctly closed)

### HIGH

**H1. D3 re-keys `ENGINE_MODELS`, which is the ledger gate's price key; this opens a fail-closed gate and misprices the engines.**
Where: B0 revision-3c D3 bullet; B3.3 `roster/table.js` row; B7 step 5; B8 item 36 (the check "`SHIPPED_MODELS` and `ENGINE_MODELS` name only `opencode-go/deepseek-v4.1-flash`"; the mutation test covers only the lead default).
Evidence: `table.js:16-19` is `claude-code: anthropic/claude-haiku-5-5` and `codex: openai/gpt-x`, the latter commented "unpriced until the Codex decision (unknown-model fails closed)". `roster/engines.js:222-223` prices an engine lane with `ENGINE_MODELS[engine]` and calls `ledger.gate`; `ledger.js:179` returns `unknown-model` for an unpriced key. The engine file header (lines 1 to 8) says the engines run their own CLIs, bill outside `llm/stream`, and their usage is recorded "priced as the vendor's model". So (a) re-keying `codex` to DeepSeek turns a deliberate refusal into a pass; (b) re-keying `claude-code` leaves the Anthropic route exactly where it is (it still needs `ANTHROPIC_API_KEY`, `ENGINE_SPEC`) and just records its spend at DeepSeek rates, so Q12 is not met, only hidden. Nox's D3 ruling is about "shipped roster defaults ... every seat"; an engine is not a seat. B11 admits the reading of `ENGINE_MODELS` as the price key was "from source, not run".
Fix: leave `ENGINE_MODELS` alone in B3.3, B7 step 5 and B8 item 36 (say so, with the reason above) and restrict D3's change and test to `SHIPPED_MODELS` plus `roster-get roles` plus the `roster.delegator-model` default; if Nox wants the Anthropic engine route closed, put that as a separate question (disable `claude-code` in `ENGINES`, or leave it unpriced so it fails closed), not as a re-key.

### MEDIUM

**M1. B8 item 36 and the B7 step 5 data step miss a second stopped Anthropic roster row.**
`roster` (live, read-only) has two rows with provider `anthropic`: the stopped `lead` (`claude-sonnet-5-5`) and the stopped `checker:cad5f8fe0117b` (`claude-haiku-5-5`, run 2 Nov 10:51 to 11:08 ET). The text says "the one stopped `lead` row" and item 36 opens with "no ... roster row ... names an Anthropic route", which fails on the checker row. Fix: say two rows; either migrate both in the named data step or state that stopped historical rows are records and exempt, and scope the check to "no non-stopped roster row" plus "no row with a live state". (The live `roster.models` policy row is all `opencode-go/deepseek-v4.1-flash`, confirmed; the two non-stopped lanes are `lead` and `builder:1`, both idle.)

**M2. K7: two summary cells still say the owner sends the continuity handoffs.**
B5.3 row 2 ("B4.1, B4.2 and B4.4 on the current host | owner sends (`comms-send`, witness C)") and B7 step 7 ("P1: B4.2 and B4.4 ... | owner sends; fresh checker witnesses") contradict B4.4, B8 item 40 and D5, where the executing session sends the 20 handoffs and the owner sends only the wake proof. Fix both cells: "owner sends the wake proof (witness C); the executing session sends the 20 continuity handoffs (D5, transitional)".

**M3. The "10 pinned rows" figure is not settled against the face install.**
B3.3, B4.0 item 2, B7 step 5 and B8 item 6 state 10 flatly (9 plus the delegator row). The live `FACE-INSTALL-GATE-CHECKLIST.md` gate H2 expects "manifest `d8226990…`; 10 pinned / 12 disabled; `fleet-face` present exactly once", i.e. 9 plus the face row. If the face leg is in the "all legs" integration commit, or installs first, the delegator row makes 11 and B8 item 6 fails as written. Row 44 admits this is "not decided here" but the other four places do not. Fix: state the figure as "baseline plus one" with the baseline taken from the manifest at the integration commit, or get the face-leg pin decision before step 5; the test then asserts the delegator row is pinned and the count is baseline plus one.

**M4. The machine changed at 17:33 ET; "the old delegator is working now" and the process list are stale.**
`journalctl --user` shows the app scope `app-deepseek-ai-dsh-desktop-2052716.scope` and `dsh-supervised-relaunch-...a11y.scope` ending at 17:33:27 ET (after 6 h 11 min); the delegator's session file was last written 17:32:33 and no Electron, Host, scrapling or forgegui process remains (pids 2052716, 2052864, 2053277, 2053312 are gone). Still live: `nvim` 57620 and the leaked Chromium 603531 (both cwd under `profiles/desktop`), scratch pid 91617, and the unit (active since 16:04:45). The app also rewrote `.credentials.yaml` (mtime 17:32:27; size still 410, mode 600) and wrote `storages` and `dsh-claude-style` at 17:32:41 to 17:32:43. Stale present-tense statements: B0 "Two hosts run today", "Where the system stands" (delegator "working now"), B2.10, B3.4 ("it is working, not dormant"), B5.3 row 3 and the clarification-2 wording that the old delegator "keeps operating". Nox's clarification 2 says design for the machine as it is. Fix: add a dated line (as read 17:38 ET: app and Host not running, delegator not turning; who closed the app is not established), keep the clarification (quiesce at the switch is what makes it dormant) and make the switch steps conditional on whether the app is running when the owner gets there. Nothing in the order or the commands changes; B7 step 0 does not depend on it.

### LOW

- **L1. B6 row 44 cites are shifted and the row is half-handled.** `FACE-INSTALL-GATE-CHECKLIST.md` was edited at 17:12:56 ET (a "Fence scope note" at lines 2 to 4 says the H8 and precondition-5 fences apply to the pre-relocation install only). The row cites `:3, 13, 33`; the lines are now `:9` (scope), `:19` (precondition 5), `:39` (H8). Update the cites and say the note was added and the post-switch replacement (A10′, A12, the extended scan, the quit-the-app rule) is still owed.
- **L2. Assertion-only B8 items remain.** Item 12 (no agent opened `hits.tsv`: no named log); item 26 ("the action chain and the session shell logs show none": the executing session's commands are in its own transcript, as item 17 already says); item 34 (the grep returns about a dozen procedural hits that need a human; give a reviewed allow-list); items 28 and 30 (owner attestations, acceptable if the checker is told to read the attestation). Items 17, 33, 36 (after M1), 39, 40, 41 and 42 are now testable by a command.
- **L3. B4.0a does not name the tool that makes the executing session's facade calls.** B0 (line 408) says the session did not run `busprobe.py` because it reads the MCP token file, while the clarification lets it make `bus.who` and `fleet.status` calls "in-process". Name the tool (or say the calls are made only by the installer's self-check, the post-switch gate script and the owner's deletion script), otherwise B8 items 17 and 42 cannot be audited.
- **L4. T1 source.** `last_signal_at` is the last signal; if a lane is signalled twice, T1 is later than the first wake. The `comms_signals` table exists; use its first row for the handoff, or state that a re-signal makes the run a fail to re-run.
- **L5. Guard and list tightening.** An agent running as uid 1000 can overwrite `~/.config/dsh-fleet/delete-guard` with a hash of its own passphrase; the text says so ("could edit the guard"), which is honest, but "the passphrase guard cannot be satisfied by an agent" is then not literally true. Cheap hardening: the owner types the first 8 characters of the guard file's hash (or the hash itself is recorded in his off-machine attestation) and the script compares. Likewise, Q17's sign-off should record the sha256 of the list file (today `ee8ae9f9544744f1…`), so the list the script compares against is the list he read.
- **L6. `RECONCILE-DECISIONS.md` line 54** still says "the sixteen one-line questions Q1 to Q16" (history of section 2b; now 24 questions). Add "(Q17, Q18 and D1 to D6 are in 2d)".

---

## 3. K1 detail: rulings reflected

- **D1** consistent in B5.2 step 3, OD2, B5.4 condition 11 (i) and (ii), B8 items 20 and 22, B9 D1 and Q17, the bundle (Q17), and `KEY-NAMED...txt`. Both deletion-checker checks are present. Predicate stated three ways ("outside `profiles/desktop/node_modules/`", "outside `.../desktop.retired-<UTC>/node_modules/`", "outside `node_modules`") and they agree. Machine: 47 found, 47 presented, 0 outside.
- **D2** steps and assertions verified against source (K3 below). "Explicit, never assumed from the 7-day expiry" appears in row 4a, the owner-steps block, B8 item 39 and the bundle. The old delegator's own token is the one named exception to the standing fence.
- **D3** rule and the fresh-install test are in place (B3.3, B7 step 5, B8 item 36, B9). **Wrong in part: H1, M1.**
- **D4** definition, the SQL, ids 4 and 5 (Nox) and 6 and 7 (self-check, flag 1) all verified on the machine. Lanes are stated not to be orders.
- **D5** correct in B4.4, B8 item 40, B9 D5; **contradicted by M2**. Item 40's check (20 `request` rows from `cayleb-local`, none a wake proof) is a database query. Note `comms_msgs` has a new row, seq 4, a `notice` from `cayleb-local` to `ext:delegator` at 17:34:50 ET; it post-dates the text and is not a design fault.
- **D6** passphrase from `/dev/tty`, scrypt hash with salt in an owner-only file, refusal `PREDELETE-REFUSED: 0 passphrase` before the phrase, owner runs the backup: consistent across B1 item 5, B3.8 item 4, B5.1, B5.3, B5.4, B8 items 22 and 41, B9, the bundle. The guard file does not exist yet (checked: `~/.config/dsh-fleet/` is mode 700, no `delete-guard`), as the text says ("not yet written"). The limit is stated honestly (L5).
- **Clarification 1** is stated once in B4.0a, referenced from B4.0, B4.2, B6, B8 items 17 and 42, B11, B9 Q10 and R26. No bearer call is called a wake. The word "wake" is qualified "while the host is running" at every claim (lines 49, 68, 392, 447 and the B3.2 Wake row); the remaining hits are negations or counts.
- **Clarification 2**: no sentence implies the old delegator is dormant now. See M4 for the opposite staleness.
- Q1 to Q5, Q10, Q12 to Q14 and the three process conditions are unchanged from what the second checker accepted, and still read as ruled.

## 4. K2 machine facts (33 checked; the second checker did not check these)

Confirmed: 47 key-named files, set equality with the list, modes and 260,342 bytes; 0 in each of the 9 other deletion paths; the 6 folders; the 7 OD1 key-named names, modes and sizes; orders 1 to 7 states, fixture flags and times; the live-order SQL (0); `ext:delegator` created 15:18:14 not retired, token file mode and time, fingerprint match, 7-day expiry; the 16:15:54 report (seq 3); `control.js:55-58` op lines at decc771; `registry.js:65`; `tokens.js:85-90`; `table.js:16-19, 28-33`; `engines.js:222-223`; `orders/schema.js:45`; `roster.js:158-159`; `kit.js:278-279`; `board.js:107, 124, 129, 133-136`; `views.js:94, 121`; `calls.js:257`; `lanes.js:14, 51`; `core/principal.js:94`; `selftest.js:10`; `mcp/session.js:26`; `mailbox.js:156-158, 199`; `api.js:18-26`; the B6 table (62 rows and tally); `gates034.sh:23-26`; `writers.py:17-19`; `step2-gate.sh:16`; `RUNBOOK-r2.md:17, 33, 34, 41`; `install-0.3.5.sh:144-147, 152-153`; `INSTALL-0.3.5-run.log` (9 pinned, 12 disabled, `8b1f87d6…`); `rows.manifest.json` sha256 `8b1f87d6aebf`; `package.json` `ce364672e9bac647`; `cordis.patch.yml` `77476b79afe6d601`; credentials 410 B mode 600, `recovery.key` 189 B 0600, `recovery.pub` 63 B, `recovery-age.key` 189 B; schedules all `ended`, third `updatedAt` 21:15:00Z; unit active since 16:04:45 with `--profile fleet --port 3092`; 62 sessions and 7 roots; `TEAM-ASSIGNMENTS.md:27` "Haiku 5.5 via Anthropic provider"; BOARD.md 1,274 lines and 136,125 bytes; `scratch-tb-l4` 63G; `~/.dsh` 154M; CLI shim passes `op` to core and `DSH_FLEET_SOCKET` is read; `control.sock` exists; deletion-list paths all exist.
Wrong or stale: H1 and M1 (roster and engine facts), M4 (process list), L1 (row 44 cites). Sizes drift as the text warns (`DH` 741M, sessions 92M and 96,202,052 bytes now); not a defect.

## 5. K3 the retirement and deletion commands, read as an operator

The block in B5.3 (and the bundle's copy) is right as written. `dsh-fleet.mjs op <name> '<json>'` is the shim path (`bin/dsh-fleet.mjs` forwards to core; `op` parses `[opName, text='{}']`, one JSON object). `comms-token-revoke {participant, fingerprint}`, `comms-retire {id}`, `comms-list` (none), `comms-tokens {participant}` match `control.js:55-58`. They are "plain" ops (not Tier B), gated by `isOwner`. Result shapes match the assertions: `{ok, result:{participants:[...retired_at...]}}` and `{result:{tokens:[{fingerprint, revoked_at,...}]}}` (`tokens.js:94-96` returns `fingerprint` and `revoked_at`). Style matches `owner-sitting/confirm-100019.sh` (`DH`, `P`, `DSH_FLEET_SOCKET=/run/user/1000/fleet/control.sock`, `op()` with `timeout 40`). The path `.../fleet/control.sock` exists. The "no-such-participant / no-such-token means already done" note is correct. The deletion script's list, realpath rule and per-path re-check are coherent. The only command-level risk is the unverified socket path under the app's Host, which the text already handles by running the steps first.

## 6. K4 order and safety

Migrate (rows 0 to 1), prove P1 (row 2), quiesce (3), preconditions (4), retire `ext:delegator` (4a, before the unit stop), switch (5, owner), prove P2 and P3 (6), archive (7), soak (8), delete (9, owner only, behind the passphrase, twelve conditions and the typed phrase). No agent runs a step that mutates the owner's data; the executing session only prepares scripts, reads for hashing and encrypts to the public recipient. The passphrase never touches an agent; the F1 residual (L5) is stated. The quiesce and no-live-order condition are before the switch and checkable by SQL (B3.8 item 1, B8 item 33). The deletion gate on Q8, Q15, Q16, Q17, Q18 does not block migration or proof. Archive precedes deletion; the off-machine key copy is a condition.

## 7. K5 testability

Runnable by a checker: every B4 proof step (four database columns, session log, named pass and fail modes), B8 items 1 to 11, 13 to 16, 18 to 25, 27, 29, 31 to 33, 35 to 42 (item 36 once M1 and H1 are fixed). Assertion-only or human-read: items 12, 26, 28, 30, 34 (L2).

## 8. K6 consistency

Nine owner questions (Q6, Q7, Q8, Q9, Q11, Q15, Q16, Q17, Q18) and fifteen ruled (Q1 to Q5, Q10, Q12 to Q14, D1 to D6) agree across B0, B9, RECONCILE (R20 to R27) and the bundle. 62 B6 rows; 12 deletion conditions; 20 continuity items; 42 B8 items; B5.3 rows 0 to 2 equal B7 steps 2 to 7, and B7 step 9 hands to B5.3 row 3 onward including row 4a. Open numeric: M3.

## 9. K7 the two sender paths

Wake proofs: the owner's `comms-send`, witness C, four database columns, stated in B4.0a, B4.2, the witnesses paragraph. Continuity handoffs: the executing session through `comms-send` on Nox's standing word, replies read from `comms_msgs` read-only, not `comms-inbox`, stated in B4.4 and B8 item 40, with "never counted as a wake proof". Distinct and each clear, except the two summary cells of M2.

## Not verified

- No scratch host, wake, proof, installer, app start or GUI was run; every liveness claim in Part B stays unverified (as Part B says).
- I did not read session transcripts, token values or files, `recovery.key`, `.credentials.yaml` or any key-named file. The token check was a count by fingerprint match only.
- Code cites were verified at decc771, not on the 0.3.5.1 tree (`1f1f740`); the control.js line numbers in this worktree's HEAD differ only if that file changed, and I did not diff it.
- I did not check who closed the app at 17:33 or why, and did not read the 17:34:50 notice.
- The B6 re-sweep covered `BD/` and `NOXBOX/` for the stated patterns; I did not re-run `finish-rel`, `finish-int`, `EV/` or the scratchpad, and a different phrasing can still hold a fence.
- I did not verify B2.9 row by row, the `lanes/board` mirrors, the unlocated gate script (row 43) or Nox's "security settlements registry" (row 30).
- Whether the app's Host serves the same control-socket path after the switch, and whether a symlinked profile behaves, remain unobserved.
