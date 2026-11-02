# CHECK 0.3.5.1 — independent checker report

Artefact: `/home/cayleb/workspace/harness-build/finish-rel`, commit `1f1f74024ab41eab4c78bb2736dc7424e04d8855` (release/0.3.5.1), parent 831032d (fix), base decc771 (installed 0.3.5 tree).
Checker did not build this. Everything below was run; commands and output tails are from this session (2 Nov 2026, ~20:50–21:00 UTC). Scratch only under `/tmp/claude-1000/chk0351`; logs kept there (`c4b.log`, `suite-*.txt`, `c6-*.txt`).

## Verdict: PASS (C1 to C7 all PASS; three low observations, no blocking defect)

| Check | Result |
|---|---|
| C1 Scope | PASS |
| C2 Decoder tests, mutation, suite compare | PASS |
| C3 Real failing t11 payload | PASS |
| C4 Real booted scratch host (a, b, c, d) | PASS |
| C5 Installer negative cases | PASS |
| C6 Live system untouched | PASS |
| C7 Self-check itself | PASS |

## C1 Scope — PASS
- `git diff decc771..1f1f740 --name-status`: `M package/src/orders/validate.js`, `A package/src/orders/test/string-criteria.test.mjs`, and under `release/` only `A FILES-0.3.5.1.sha256, PRE-0.3.5.1.sha256, gen-lists-0.3.5.1.sh, install-0.3.5.1.sh, verdict-selfcheck.mjs`. Nothing else (7 files, +528/-1).
- validate.js diff: adds `MAX_CRITERIA_TEXT = 2 MiB` and exported `criteriaFromText(value)`, and in `validateVerdict` replaces `Array.isArray(input.criteria) ? input.criteria : []` with `const given = criteriaFromText(input.criteria); const rows = Array.isArray(given) ? given : []`. That is the only change to validation logic.
  - It returns a decoded value only when `JSON.parse` yields an array (`Array.isArray(decoded)`); otherwise it returns its input unchanged, so a non-array becomes `[]` and is refused `criteria-required` exactly as before. Non-strings, empty strings and strings over 2 MiB are returned untouched before any parse (test proves JSON.parse never saw the oversized text). The second try (`\"` to `"`) runs only when the text is not JSON as sent. Row validation, `criteria-incomplete`, `pass-needs-all-met`, `quote-required` etc. are untouched (verified by the 4th test and by reading the unchanged lines).
- `FILES-0.3.5.1.sha256` has one line: `b68ce88c...bedc22  src/orders/validate.js`; `sha256sum package/src/orders/validate.js` = `b68ce88c...bedc22` (equal). `PRE-0.3.5.1.sha256` = `bbeacee7...532dd`; `git show decc771:package/src/orders/validate.js | sha256sum` = `bbeacee7...532dd` (equal). The live file's hash is also `bbeacee7...` (mode 644).
- Cosmetic (low): the new const and function were inserted between validateVerdict's JSDoc block ("A Checker verdict ...") and the function, so that doc comment now sits above `MAX_CRITERIA_TEXT`. No behaviour effect.

## C2 Decoder tests and mutation — PASS
- `node --test package/src/orders/test/string-criteria.test.mjs` on the release tree: `tests 5, pass 5, fail 0`.
- Mutation (made `criteriaFromText` return its input, in my copy): `pass 0, fail 5`, all five tests failed. Restored; hash back to `b68ce88c...`, copy tree clean.
- Whole orders suite (`cd package/src/orders && node --test test/*.test.mjs`, fresh TMPDIR):
  - release 1f1f740: tests 196, pass 193, fail 3
  - baseline decc771: tests 191, pass 188, fail 3
  - failing names identical in both (diff of sorted sets is empty), so the release adds 5 passing tests and no new failure:
    1. `R5-6 (contract F2, shapehost): the tool schemas type every post id as a string, so a JSON number fails the schema on the host (bad-args)...`
    2. `selftest with 'agent': distinct ids are distinct claimants through the real tools pipeline; the loser gets already-claimed; one work root per id`
    3. `test/finisher.test.mjs` (file-level failure)
  - Their cause was not investigated (pre-existing on the baseline).

## C3 The real failing payload — PASS
- Session log read with `zstd -dc` (read only). The last `verdict` tool call is event 351 (`callId call_5c51...`), arguments keys `task, result, criteria`, `result: pass`, `criteria` a **string** (1663 chars) that is plain JSON (4 rows, parses as an array as sent; no backslash-quotes). Its matching tool/result (matched by `toolCallId`): `Error: verdict-invalid: the verdict is not evidence: criteria: criteria-required` (isError true). No claim token was printed (the call carries none).
- t11 spec from `fleet.db` (mode=ro): `done_when` rubric, 4 criteria. The payload's 4 rows' `criterion` texts equal t11's 4 criteria exactly, all `met: true`.
- Built a rubric task with exactly those 4 criteria through the orders test scaffolding (`helpers.mjs`: `world`, `startedOrder`, `claimTask`, `submitBuilt`, `beginCheck`) and called `recordVerdict` with the payload's exact `criteria` string and `result`:
  - release tree: `RESULT: ACCEPTED {"verdict":{"round":1,"result":"pass",...` and `task state: done`
  - baseline tree: `RESULT: REFUSED code=verdict-invalid problems=criteria-required | task state: checking`

## C4 Real booted scratch host — PASS
Built with the release tree's own `release/lib/scratch.mjs` (`makeScratchHome`, `startHost`, `stopHost`, `freePort`): read-only copy of the live layout, own port and sockets, no systemd, `install-0.3.5.sh --no-service --derive-live-artefacts` to reach the 0.3.5 state (exit 0; scratch `validate.js` = PRE hash `bbeacee7...`, mode 644). Driver `c4driver.mjs`, 28/28 checks, final log `c4b.log`.
- (a) Unfixed host: `node release/verdict-selfcheck.mjs --url ... --admin-socket ...` exit 1, `VERDICT SELF-CHECK: FAIL (4 of 11)` (both shapes: "the real string verdict is accepted" and "the harness applied it" FAIL with `criteria-required`; the non-JSON refusal checks PASS). `bus-selfcheck.mjs`: `BUS SELF-CHECK: PASS`.
- (b) `install-0.3.5.1.sh --no-service --pkg --home --profile-dir --cli --rundir`: exit 0, `replace pkg/src/orders/validate.js`, backup `rel0351-<UTC>` written, `installed 1 file(s); every installed artefact verified`, manifest check, `compose check: dump-config mounts @cayleb/dsh-fleet/comms once`, `INSTALL PASS`. Booted again: `VERDICT SELF-CHECK: PASS` (11 of 11, both shapes accepted and `task_state done`, non-JSON string refused and applied nothing) and `BUS SELF-CHECK: PASS`.
  - Tree diff of the whole profile+package tree across the install: exactly one entry, `.../src/orders/validate.js: bbeacee7... 644 -> b68ce88c... 644`. Overlay, rows manifest, presets, patch files untouched.
- (c) `install-0.3.5.1.sh --rollback --no-service`: `restored 1 file(s) byte for byte`; the whole profile+package tree (sha256 and mode of every file) is identical to its pre-install snapshot (tree hash `16eea63524ce` both). Booted: verdict-selfcheck FAILS again (exit 1), bus-selfcheck PASS.
- (d) Second install run: `already installed: nothing to do`, exit 0, tree unchanged, no second backup. After a rollback, re-install passes and the next run again says already installed.
- All hosts stopped (`hosts still alive after stop: 0`), scratch dirs removed; no process matching chk0351 remains.
- Extra, not in the brief: automatic rollback. With a deliberately failing `--cli` (exits 1), the installer replaced the file, then `FAILED: dump-config failed: boom`, `FAILED before the start: rolling every replaced file back`, exit 1, and the tree equals the pre-install tree; a following good install passed.

## C5 Installer negative cases — PASS (scratch, --no-service)
- `--dry-run`: exit 0, `0 already installed, 1 to install`, `dry run: nothing written`; profile+package tree hash identical before/after and no `rel0351-*` backup created.
- Release file hash not matching the list (tampered copy of the release tree): `REFUSED: release file src/orders/validate.js does not match FILES-0.3.5.1.sha256 ...`, nothing changed.
- Target neither expected pre-install nor release file (appended a line): `REFUSED: pkg/src/orders/validate.js is neither the expected live file nor the release file ... nothing was changed`; no backup, tree unchanged; `--dry-run` refuses the same way.
- Tampered backup copy: `--rollback` gives `REFUSED: the backup copy of pkg/src/orders/validate.js does not match its recorded hash; not rolling back`; installed file untouched.
- Installed file edited after install: `--rollback` gives `REFUSED: ... is neither the installed nor the backed-up file (changed since the install)`.
- `--rollback --dry-run` writes nothing.
- Modes: with the real mode 644, mode is 644 before and after install and after rollback. With a 640 original, rollback restores 640 byte-for-byte. Observation (low): install does not preserve a non-644 mode, it sets 644 (`install -m 0644`); the live file is 644 so there is no effect on the real install.

## C6 Live system untouched — PASS
Before (20:54 UTC) and after (21:00 UTC) values, identical:
- package dir `find -type f | sort | xargs sha256sum | sha256sum` = `20b515965225031962edf6e418a5ddccd4e2701127e3c2d6238932d05f130362` (both)
- user unit `dsh-fleet-consolidated`: `active` (both)
- `policy_tickets` rows: 19 (both); `max(events.seq)`: 1744 (both, so no event growth during my run)
- The only live command I ran: `bash release/install-0.3.5.1.sh --dry-run` (read-only): `replace pkg/src/orders/validate.js`, `0 already installed, 1 to install`, `dry run: nothing written, no service touched`, exit 0. sqlite opened with `mode=ro`.
- Observation for the reader: the live `fleet.db` already holds two fixture orders titled `verdict-selfcheck shape A/B ...` (order ids 4 and 5, created 20:51:14 UTC, events seq 1682 to 1706, still `in_progress`) and `core.selftest` events up to seq 1744 (20:52:10). These were created before my C6 baseline (20:54:10) and I never addressed port 8790, so they are not mine; someone ran the self-check against the live (unfixed) host earlier. Not a defect of the artefact, but those two fixture orders are left open on the live fleet.

## C7 The self-check itself — PASS
Read `release/verdict-selfcheck.mjs`: the bearer is read inside the process (`readTokenFile` / `adminToken`); every printed detail passes `scrub(detail, token)` (replaces the token, any `Bearer ...`, `token=` query values and 64-hex strings); the facade version line prints only server name and version. It creates only orders with `fixture: true` through `board.op order-create` and drives them with `fleet.selftest` principals named `selftest-vs-<rand>`. It exits 1 on any failed check and on any exception (catch records a FAIL), 2 on bad usage. C4(a) and C4(c) show it does fail on a host without the fix (4 of 11 FAIL, exit 1), and C4(b) that it passes with it. Its output in the runs had no 64-hex or Bearer string.

## Not verified
- No install on the live system and no live verdict drive (not permitted); the installer's service path (`stop_unit`, `systemd-run` start, and its built-in `selfcheck()` call using `--token-file $home/fleet/state/mcp/token`) was not exercised because scratch runs use `--no-service`. I ran the same two self-check scripts by hand with `--admin-socket` instead.
- Only the two criteria shapes in the contract were tested (plain JSON string, quotes carrying literal backslashes); other malformed encodings were covered only by the 8 refusal strings in the unit test.
- The cause of the 3 baseline suite failures; the lane/fleet behaviour of the live host after a real install; any wake or bus-bridge behaviour.
- The scratch hosts ran the 0.3.5 profile composed from a copy of the live profile at 20:5x UTC; later live changes were not reflected.
