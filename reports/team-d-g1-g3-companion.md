# TEAM D — G1-G3 companion built and proven offline (2026-11-01 09:45–09:55 ET)

Authorisation: Lead ruling C, 2026-11-01 — "write the small companion/extension at the upload step, offline and
dry-run only until the window. It must emit evidence receipts, persist moderationState, and implement the block
filter; it must never handle or log key values (keys are owner-entered), no network calls before the window, no
publication. Prove it on the pinned candidate without changing candidate bytes (55cef5d3… must stay the same after a
dry run)." Tier A, **$0**, no publication, no upload, no credential touched.

## 1 · What was built

`tools/bind_art.py` (card repo `~/Work/projects/untitled-anime-card-game`), branch
**`prep/art-binding-20261101`**, commit **`cd91a88`** (checker-fix commit; parent `ddcdd79`, whose parent is `e7f885c` — the
audio tool commit; the preservation branch `preserve/candidate-55cef5d3` is untouched at `43574a5`). Both prep branches
(`prep/art-binding-20261101`, `prep/audio-upload-20261101`) now point at `cd91a88`. 764 lines (620 before the fix round),
stdlib + the project's own `ops/release/publish.py`
helpers; it does NOT modify `publish.py`.

It closes the three gaps found in D-3:

| Gap | Closure |
|---|---|
| **G1** no evidence receipts | Per card it writes `upload-receipt.json` (`uacg-upload-receipt/v1`, `status: COMPLETE`, `returned_on`), `moderation-receipt.json` (`uacg-moderation-receipt/v1`, verdict exactly as returned), and — **only on PASS** — `mapping-receipt.json` (`uacg-asset-mapping/v1`) whose two evidence blocks carry `reference` + project-relative `file` + `sha256`, exactly the shape `publish.validate_mapping_receipt()` demands for `tools/apply_art_ids.py` |
| **G2** moderation not captured | Every POST response and every operation-poll result is written raw to `raw-operation.json` adjacent to the evidence; `moderationState` is persisted into `assets/images/roblox-ids.json` (`moderation_state`, `moderation_verdict`, `raw_operation`, `receipts`); PENDING/REJECTED are recorded as such, never rounded to PASS |
| **G3** no block filter | `--block a|b|c` (a = Bleach+Naruto 6, b = Dragon Ball+Berserk 6, c = Vinland Saga+Vagabond+MHA+Black Clover 12), plus `--cards`, `--franchise`, and an explicit `--all`; `--live` refuses without an explicit selection |

Design guarantees kept from the project's own rules: duplicate/already-bound refusal, unresolved-attempt refusal
(SUBMISSION_STARTED / AMBIGUOUS / OPERATION_PENDING are never retried), ambiguous outcome journalled then **never
retried**, `require_current_build()` before any upload, target identity from `ops/release/target.json`, neutral asset
names `uacg-art-<card_id>`, no publication, no spend. A `--rebuild-receipts` mode re-emits evidence from a recorded
raw operation without network if a run is interrupted between upload and receipt write (never re-uploads).

Modes: **default = dry run** (no writes, no network, key not read) · `--live` (owner's go; needs the owner-entered
`ROBLOX_OPEN_CLOUD_API_KEY` in the process environment and `target.json`) · `--status` · `--rebuild-receipts` ·
`--selftest`.

## 2 · Evidence (all commands run 2026-11-01 09:45–09:53 ET; full log
`board/probes/team-d/g1-g3-tooling-20261101.txt`)

Selftest — the receipt builder is exercised against the **real validator** in a throwaway `/tmp` tree (manifest
row, the actual PNG, the actual `Cards.luau`, a synthetic target that exists only in /tmp; no repo write, no network):

```
$ python3 tools/bind_art.py --selftest
SELFTEST PASS: PASS/PENDING/REJECTED receipt shapes accepted/refused exactly as
publish.validate_mapping_receipt demands; no repo write, no network, no key read.   rc=0
```

Dry run on the pinned candidate:

```
$ python3 tools/bind_art.py            # all 24
DRY RUN: 24 card(s); no network, no writes, no key read.
target.json: ABSENT/BLOCKED — … (U0 still the precondition)
build receipt: current (55cef5d3ea71173d…)
… 24 POST lines, each "x-api-key: <redacted>" …
24 planned upload(s); 0 not uploadable as-is.   rc=0
```

Block filter counts (measured): **a = 6, b = 6, c = 12** POST lines; `--franchise Bleach` = 3.

Guards (all exit 2, nothing sent): `--live` without an explicit block; `--live --all` without the key; unknown card
id; `--evidence-dir ../escape` (path traversal refused). `--rebuild-receipts` with no finished upload correctly
refuses (it needs `target.json` for the identity).

Key discipline: with a sentinel value in `ROBLOX_OPEN_CLOUD_API_KEY`, a dry run's whole output contains
**0 occurrences** of the sentinel. The only place the tool reads the environment is inside `--live`, immediately
before building the request header; the value is never printed, written, stored or echoed, and no dry-run/status/
selftest code path touches it. (No `strace` on this host, so "no network" is proven by code path + the no-key/no-
target refusals + zero writes rather than a packet capture.)

Candidate unchanged: `sha256sum build/UntitledAnimeCardGame.rbxlx` = **`55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9`** before AND after every test; `git status --short` shows only the new untracked tool before commit; no evidence directories were created by the dry runs.

## 3 · What is proven and what is not

**Proven:** the receipt generator produces documents the project's real binding validator accepts (and refuses
non-PASS verdicts); the block filter selects 6/6/12; the dry run writes nothing, calls no network and never reveals a
key; the pinned candidate bytes do not move; the tool is committed on a separate prep branch.

**Not proven (honest):** the `--live` HTTP path has never run — no key, no `target.json`, and the owner window has
not opened. It is the same Assets API request shape `publish.py` already uses for Decals, plus raw capture and
polling copied from it; the first live block (recommended `a`, 6 cards) will be its proof, and its first
`apply_art_ids.py --receipt … --no-build` run must pass the validator before anything else proceeds.

## 3.5 · Checker HOLD and the fix round (2026-11-01 10:45–10:58 ET)

The fresh tool checker's verdict was **HOLD** (`board/reports/team-d-binding-tool-checker-verdict.md`, 424 lines,
per-item evidence). Fixes shipped in commit **`cd91a88`** (both prep branch refs point there):

| ID | Sev | Defect | Fix |
|---|---|---|---|
| D1 | **P0** | `bind_art.py` called the non-existent `publish.operation_id_of` → the first real card would upload, then crash before any receipt/asset id was recorded (`SUBMISSION_STARTED` stuck, manual reconciliation) | inline `response.get("operationId")` (as `publish.py:727-731` / `upload_audio.py` do), journal `AMBIGUOUS` when absent; **added a stub-transport case to `--selftest`** that drives `upload_one()` end to end (POST→poll→four evidence files→real validator) and the ambiguous branch (one POST, no retry) |
| D2 | MED | `upload_audio.py` did not catch `ValueError`; a key containing a newline was echoed verbatim in an uncaught traceback | `ValueError` caught at both HTTP call sites; the key is pre-validated (whitespace/control/non-ASCII) before any request with a message that never contains the value; `run_live`/`main` catch `ValueError` generically; same pre-check added to `bind_art.py` |
| D3 | LOW | absolute `--evidence-dir` bypassed the `..` guard and created a directory outside the project before refusing | absolute and `..` paths refused in `main` and again in `write_json` before any `mkdir` |
| D4 | LOW | `--rebuild-receipts` re-emitted receipts without re-verifying the recorded `raw-operation.json` | new `verify_recorded_raw()` checks project path, sha256 vs journal, card identity, completed operation and asset id; any mismatch refuses |
| D5 | DOC | this report said "561 lines" (that was the audio tool) | corrected to 764 (620 pre-fix) |
| D6 | OBS | audio `--live` has no explicit selection gate (bare `--live` uploads all 8) | left as designed (the cue map is all 8); documented here and to the Lead — not a defect |

Re-checked with the checker's own offline harnesses (fresh `/tmp/uacg-recheck` copy of `cd91a88`; evidence
`board/probes/team-d/checker-fixes-20261101.txt`):

```
selftest (spy harness):  SELFTEST PASS … HARNESS rc=0 net_attempts=0 key_reads=0 key_access=[]
stub harness, FIX_OPID UNSET:  STUB_RESULT {"rc": 0, "net": 0, "methods": ["POST", "GET"], …}
   → 4 evidence files (raw-operation, upload-receipt, moderation-receipt, mapping-receipt), journal ASSET_ID_RETURNED
   → python3 tools/apply_art_ids.py --receipt … --no-build  →  "Validated offline mapping: BC-ASTA-01 → 1234567890"
ambiguous (POST fails):  rc 3, methods ["POST"] exactly once, journal AMBIGUOUS
unresolved (never done):  rc 3, methods ["POST", GET×8], no retry
newline-key (audio, spy): rc 3, net 0, SENTINEL HITS 0
newline-key (bind_art, spy): rc 3, SENTINEL HITS 0
absolute --evidence-dir: rc 2, no directory created
dry-run POST counts unchanged: 24 / 6 / 6 / 12 / 3
candidate sha256: 55cef5d3… unchanged in the repo and the re-check copy
```

Status: **fix round delivered; awaiting the checker's re-check before any `--live`.** The live HTTP semantics
(auth, status codes, real moderation field names, quota) remain UNVERIFIED until the owner window's first block.

## 4 · Next / successor

1. When the owner channel returns W2 + D12 + U0 target + P9 key: fill `target.json` (§1 of D-3 — never fabricate
   ids), then run `python3 tools/bind_art.py --block a` (dry) → `--live` on the owner's go; repeat b, c; then the
   emitted `apply_art_ids.py` commands (23 × `--no-build`, last one builds) → rebuilt candidate with 24/24 ArtIds.
2. Then the per-card Studio evidence round (`studio-evidence-sheet.md`) and the 8 audio cues
   (`tools/upload_audio.py`, same window authorisation).
3. Nothing in this file authorises an upload; the tool refuses until the human gate (target + key + explicit block)
   is satisfied. `$0`, no publication.
