# TEAM D — independent re-check verdict: art-binding / audio-upload tools after the checker-HOLD fix round (2026-11-01)

**Checker:** fresh-context independent re-check (did not write the tools, the fixes, or the prior verdict).
**Reports to:** Lead only.
**Window:** 2026-11-01 11:05–11:17 ET (`America/New_York`).
**Authority used:** read-only on the repo and its branches; execution only inside throwaway `/tmp` copies of the
named commit; write only to `/tmp` and to this file.
**Not done, by design:** no live upload, no network request, no real or test credential read or printed, no Studio
action, no publication, no repo/branch modification, no `target.json` created in the repo.

**Targets**

| Item | Value |
|---|---|
| Repo | `~/Work/projects/untitled-anime-card-game` |
| HEAD (both prep branches) | `cd91a883f16dd0523c35b66bb1ea50c359e0afff`, parent `ddcdd79627364c31ae3bdd617c1eb8755f6d9f8b` |
| Branch refs | `prep/art-binding-20261101` → `cd91a88`; `prep/audio-upload-20261101` → `cd91a88`; `preserve/candidate-55cef5d3` → `43574a5` (untouched) |
| `tools/bind_art.py` | 764 lines, sha256 `a65eb218a27adc24a600e36a87f8829691beb5d1a2463e9c420cb435b2a83d66` |
| `tools/upload_audio.py` | 567 lines, sha256 `1b630bede1e61032b578bf93664e0bc113aaae1ab3fdff4b9942b3b1c1d34165` |
| `ops/release/publish.py` | sha256 `87dc2c1c…20604` — **identical to the prior check**, i.e. the fix commit did not touch it |
| `tools/apply_art_ids.py` | sha256 `285e7297…89b7` (unchanged) |
| Fix commit scope | `git diff --stat ddcdd79 cd91a88` → `tools/bind_art.py` +152/−8, `tools/upload_audio.py` +9/−3 only |
| Candidate of record | `build/UntitledAnimeCardGame.rbxlx` = `55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9` (repo + every copy) |
| Repo status | clean (`git status --porcelain` = 0 lines) before and after; HEAD/branches unchanged; no `docs/verification/art-upload*` dirs in the repo |
| Repo `ops/release/target.json` | **absent** (as designed) — every `--live` control-flow run below used a synthetic target inside a `/tmp` copy only |

Method: `git archive cd91a88 | tar -x` into `/tmp/uacg-rechk2` (execution) and `/tmp/uacg-pristine2` (zero-write
inventory), plus `/tmp/uacg-mut` (mutation test). **My own** harnesses (not the maker's, not the prior checker's):
`/tmp/uacg-harness2/spy.py` (denies `socket.socket.connect`, `socket.create_connection`, `socket.getaddrinfo`;
spies every `os.environ` read of the key), `stub_live.py` (in-process transport, **no `FIX_OPID` code path exists at
all**), `stub_valueerror.py`, `stub_audio.py`, `body_probe.py`. The key sentinel is a fake string
`UACG-RECHK-SENTINEL-4a7d2c1e-NOT-A-REAL-KEY`; it was never a credential.

---

## Verdict summary

| # | Item | Verdict |
|---|---|---|
| **D1** | `publish.operation_id_of` gone; inline `response.get("operationId")` with `AMBIGUOUS` journal; stub-transport `--selftest` case drives `upload_one()` end-to-end | **PASS** |
| **D2** | `upload_audio.py` catches `ValueError` at both HTTP sites; key pre-validated before any request with a non-echoing message; same pre-check in `bind_art.py` | **PASS** |
| **D3** | Absolute/`..` evidence paths refused in `main()` **and** in `write_json()` before `mkdir`; no directory created | **PASS** |
| **D4** | `--rebuild-receipts` re-verifies `raw-operation.json` (path, sha256, card id, `done`, asset id) and refuses every mismatch | **PASS** |
| **D5** | Doc line count corrected (764; 620 pre-fix; audio 567) | **PASS** |
| **D6** | Audio bare `--live` documented (companion §3.5) | **PASS** (see observation O2) |
| **R1** | `--selftest` PASS against the real `publish.validate_mapping_receipt`; and a mutation test proves the new stub case is not vacuous | **PASS** |
| **R2** | Dry-run POST counts unchanged: 24 / 6 / 6 / 12 / 3 | **PASS** |
| **R3** | Sentinel key: 0 occurrences in every output and every file; key read only once, inside `--live`, never printed | **PASS** |
| **R4** | No automatic retry: ambiguous → rc 3, one POST; unresolved → rc 3, one POST + 8 GETs; missing `operationId` → rc 3, one POST, `AMBIGUOUS` | **PASS** |
| **R5** | Candidate `55cef5d3…` unchanged in repo and copies; `git status` clean; no stray writes from any offline mode | **PASS** |
| **R6** | Static completeness: no other missing `publish.*` attribute; `publish.py` untouched; no retry library | **PASS** |
| **L** | The real `--live` HTTP path (auth, status codes, wire-format moderation fields, quota) | **UNVERIFIED — cannot be verified offline; block a (6 cards) is the first proof** |

### Overall: **PASS** — safe to run `--live` in the owner window under the stated sequence.

Every defect from the prior HOLD is closed and re-proven independently; no new defect was found; the regression
suite is intact. The tool now **fails closed** where it previously crashed (missing `operationId`), refuses a
malformed key before any request, refuses out-of-project evidence paths before creating anything, and refuses to
re-emit receipts from a raw file it cannot re-verify.

**Explicitly unchanged and still UNVERIFIED: the `--live` HTTP path itself.** No real key exists and no socket was
opened in any run here; auth-header acceptance, HTTP status semantics, the real moderation field names as Roblox
actually returns them, and the 10-audio/month quota are all outside what an offline check can prove. **Block a
(6 cards: Bleach + Naruto) is the first proof**, in this order: dry `--block a` → `--live --block a` → one
`apply_art_ids.py --receipt … --no-build` per PASS mapping receipt → only then blocks b and c, and the final build.

---

## D1 (P0) — missing `publish.operation_id_of` — PASS

**Static.** `grep -rn operation_id_of tools/bind_art.py ops/release/publish.py` → **no hits** (rc 1). The only
definition left in the tree is the audio tool's own local helper (`upload_audio.py:322`). `bind_art.py:318-321`:

```python
operation_id = response.get("operationId") if isinstance(response, dict) else None
if not isinstance(operation_id, str) or not operation_id.strip():
    raise RuntimeError("upload response has no operationId")
operation_id = operation_id.strip()
```
and `:322-327` catches `(RuntimeError, UnicodeDecodeError, json.JSONDecodeError, ValueError)`, journals
`AMBIGUOUS`, and re-raises a generic `RuntimeError` — so an absent/unusable `operationId` is an rc-3 block, not a
crash, and it happens **after** the POST is journaled as `SUBMISSION_STARTED` and updated to `AMBIGUOUS`.

**`--selftest` now drives `upload_one()` end to end.** `run_selftest()` calls `stub_live_path_failures()`
(`bind_art.py:614`), which builds a `/tmp` tree, replaces `publish.request_bytes` in-process, calls
`upload_one(...)` with a stub key (`:675`), then asserts `calls == ["POST","GET"]` (`:676`), a mapping receipt that
the **real** `publish.validate_mapping_receipt` accepts (`:682`), journal state `ASSET_ID_RETURNED` (`:687`), and an
ambiguous branch with exactly one POST and journal `AMBIGUOUS` (`:689-710`). No socket is reachable from it.

**Dynamic — stub transport, `FIX_OPID` unset** (my harness has no such variable; printed
`FIX_OPID set? -> <unset>`), 2026-11-01T11:14:19-05:00:

```
$ cd /tmp/uacg-rechk2 && PYTHONDONTWRITEBYTECODE=1 STUB_ROOT=/tmp/uacg-rechk2 STUB_MODE=ok \
    ROBLOX_OPEN_CLOUD_API_KEY='STUB-OK-KEY' python3 /tmp/uacg-harness2/stub_live.py --live --cards BC-ASTA-01
LIVE: 1 card(s) in this block, evidence under docs/verification/art-upload-20261101/; no publication and no experience change.
BC-ASTA-01: asset_id=1234567890 moderation=PASS mapping=yes
STUB_RESULT {"rc": 0, "net": 0, "mode": "ok", "methods": ["POST", "GET"],
  "calls": [{"method": "POST", "url": "https://apis.roblox.com/assets/v1/assets",
             "header_names": ["Content-Type", "x-api-key"], "body_len": 1488719},
            {"method": "GET", "url": "https://apis.roblox.com/assets/v1/operations/op-stub-1",
             "header_names": ["x-api-key"], "body_len": 0}],
  "journal": {"BC-ASTA-01": "ASSET_ID_RETURNED"}, "assets": []}
```
- `AttributeError` occurrences in that run: **0**.
- Four evidence files on disk: `raw-operation.json`, `upload-receipt.json`, `moderation-receipt.json`,
  `mapping-receipt.json`; journal entry carries `asset_id`, `moderation_state`, `raw_operation` `{file, sha256}`,
  `receipts`.
- The POST body still carries the approved PNG **verbatim**: body 1,488,719 B; PNG 1,488,158 B, sha256
  `59451c09…`, `png in body == True`; `Content-Type` boundary matches the body markers (11:17:0x, `body_probe.py`).
  That PNG hash equals the journal's `source_sha256`.

**Downstream binder on the emitted receipt** (2026-11-01T11:14:35-05:00):

```
$ python3 /tmp/uacg-harness2/spy.py tools/apply_art_ids.py \
    --receipt docs/verification/art-upload-20261101/BC-ASTA-01/mapping-receipt.json --no-build
Validated offline mapping: BC-ASTA-01 → 1234567890
Bound fields: current image hash, exact creator/universe/place, returned ID, upload receipt, PASS moderation, franchise permission, commercial model licence.
Mapping recorded; release remains blocked until the source manifest, build, and build receipt are refreshed.
HARNESS rc=0 net_attempts=0 key_reads=0 key_access=[]
```
Build hash before and after: `55cef5d3…d4c9`.

**The absent-`operationId` branch (the exact D1 regression shape)** — stub returns a 200 body with no
`operationId`, 2026-11-01T11:15:25-05:00:

```
FAILED BC-ASTA-01: BC-ASTA-01: upload outcome is ambiguous (or its operationId is unusable); journalled AMBIGUOUS and NEVER retried
BLOCKED: 1 failure(s); unresolved: BC-ASTA-01. Resolve by hand before any further upload.
STUB_RESULT {"rc": 3, "net": 0, "mode": "no_opid", "methods": ["POST"], …, "journal": {"BC-ASTA-01": "AMBIGUOUS"}}
```

**Mutation test — the selftest would have caught the original P0** (2026-11-01T11:15:48-05:00). In a separate copy
I replaced the inline parse with the old `publish.operation_id_of(response)` call and ran `--selftest`:

```
$ python3 /tmp/uacg-harness2/spy.py tools/bind_art.py --selftest
SELFTEST FAIL: stub live path raised AttributeError: module 'ops.release.publish' has no attribute 'operation_id_of'
HARNESS rc=1 net_attempts=0 key_reads=0 key_access=[]
```
So the new case is a genuine regression test, not a vacuous PASS.

## D2 (MEDIUM) — credential echo on a malformed key — PASS

**Static.** `upload_audio.py` now carries `ValueError` in **both** HTTP call sites:
`except (RuntimeError, ValueError, UnicodeDecodeError, json.JSONDecodeError)` at `:380` (POST) and `:398` (poll GET);
`run_live` catches `(RuntimeError, ValueError)` at `:463`; `main` has a generic `except ValueError` at `:561`
(message: “a value could not be used; details withheld (no credential is ever echoed)”, rc 3). The pre-check is at
`:454-456`, **before** `load_target()` and before any request. `bind_art.py` has the identical pre-check at
`:384-386` and the same generic `except ValueError` in `main` (`:758-760`). The diff for this fix is 9 added / 3
removed lines and touches nothing else.

**Dynamic — newline in the key, sentinel scan** (2026-11-01T11:12:25-05:00), both tools, socket-denied:

```
bind_art.py --live --cards BC-ASTA-01:
BLOCKED: the owner-entered key contains whitespace, control or non-ASCII characters and was refused before any request; the value is never echoed
HARNESS rc=3 net_attempts=0 key_reads=1 key_access=['get']
SENTINEL HITS: 0

upload_audio.py --live:
BLOCKED: the owner-entered key contains whitespace, control or non-ASCII characters and was refused before any request; the value is never echoed
HARNESS rc=3 net_attempts=0 key_reads=1 key_access=['get']
SENTINEL HITS: 0
```
Also proven for a **tab** and a **non-ASCII** (`é`) key: rc 3, `net_attempts=0`, sentinel 0, for both tools
(11:16:43 / 11:16:56).

**The catch sites themselves** (independent of the pre-check, so the `except` is proven, not assumed). A stub
raises `ValueError("Invalid header value b'<sentinel>\nINJECT'")` — exactly the CPython `putheader` shape — at each
site:

| Tool / site | Result | Sentinel hits |
|---|---|---|
| `bind_art` POST (`:322`) | rc 3, `["POST"]` once, journal `AMBIGUOUS`, generic message | **0** |
| `bind_art` poll (escapes to `main`'s `ValueError`) | rc 3, `["POST","GET"]`, generic “details withheld” | **0** |
| `upload_audio` POST (`:380`) | rc 3, 8 × one POST, each cue `AMBIGUOUS`, generic | **0** |
| `upload_audio` poll (`:398`) | rc 3, 8 × `POST,GET`, each cue `AMBIGUOUS`, generic | **0** |

(11:13:35 and 11:13:55; logs `d2-bindart-value_*.out`, `d2-audio-value_*.out`.)

## D3 (LOW) — absolute / `..` evidence paths — PASS

**Static.** `main()` rejects at `bind_art.py:740-741`: `if evidence_dir.is_absolute() or ".." in evidence_dir.parts`.
`write_json()` calls `project_relative()` (`:176`) — which refuses absolute or `..` (`:167-172`) — **before**
`destination.parent.mkdir(...)` (`:179`). The old guard `if not evidence_dir.is_absolute() and ".." in ...` is gone
from the diff.

**Dynamic — CLI** (2026-11-01T11:11:42-05:00), target dir absent before and after:

```
$ python3 spy.py tools/bind_art.py --evidence-dir /tmp/uacg-abs-escape-r2            → rc 2
BLOCKED: evidence dir must stay inside the project
HARNESS rc=2 net_attempts=0 key_reads=0 key_access=[]
abs dir after dry: absent
$ python3 spy.py tools/bind_art.py --live --block a --evidence-dir /tmp/uacg-abs-escape-r2  → rc 2
abs dir after live: absent
$ python3 spy.py tools/bind_art.py --live --block a --evidence-dir ../escape          → rc 2
escape dir exists outside copy? absent
```

**Dynamic — `write_json` directly**, bypassing `main` (2026-11-01T11:12:04-05:00), because the prior defect was
that the `mkdir` ran before the refusal:

```
absolute: BLOCKED evidence path must stay inside the project
dotdot:   BLOCKED evidence path must stay inside the project
ok:       WROTE ('docs/verification/probe-r3/card/upload-receipt.json', '2cc60c50…')
abs_dir_present: False
dotdot_dir_present: False
ok_dir_present: True
```
No `/tmp/uacg-abs-escape-r3` and no `/tmp/escape-r3` was created.

## D4 (LOW) — `--rebuild-receipts` re-verification — PASS

**Static.** `verify_recorded_raw(card_id, attempt)` (`bind_art.py:424-447`) checks, in order: `file`/`sha256` present;
`project_relative()` (no absolute/`..`); file exists; sha256 equals the journal; JSON parses; `card_id` matches;
`operation_result.done` is true; `publish.operation_asset_id(result)` equals the journal `asset_id`. It is called
for every card in `run_rebuild` (`:463`) before `emit_receipts`; a refusal prints `SKIP …` and the run returns 2.

**Dynamic — six tamper cases** on a registry that had just produced a valid receipt set (11:14:48 and 11:15:06),
each run socket-denied:

| Tamper | Result |
|---|---|
| control: untouched | rc 0 — `BC-ASTA-01: receipts rebuilt …`, `rebuilt 1 card receipt set(s); no network call was made` |
| one byte appended to raw (sha mismatch) | rc 2 — `SKIP … recorded raw operation sha256 does not match the journal` |
| raw says `done:false`, journal sha updated to match | rc 2 — `SKIP … recorded raw operation is not a completed operation` |
| raw `card_id` changed to another card, sha updated | rc 2 — `SKIP … recorded raw operation belongs to a different card` |
| raw asset id changed, sha updated | rc 2 — `SKIP … recorded raw operation asset id does not match the journal` |
| raw file deleted | rc 2 — `SKIP … recorded raw operation file is missing: …` |
| recorded `file` set absolute | rc 2 — `SKIP … evidence path must stay inside the project` |

Every refusal ran with `net_attempts=0` and `key_reads=0`. The control case proves the happy path still works.

## D5 / D6 (DOC / OBS) — PASS

- **D5:** companion `board/reports/team-d-g1-g3-companion.md:14` now says **“764 lines (620 before the fix round)”**;
  actual `wc -l tools/bind_art.py` = **764**; `tools/upload_audio.py` = **567** (matches the probe file). The old
  “561” is gone from the body and appears only inside the §3.5 table as the correction.
- **D6:** companion `:96` documents it: “audio `--live` has no explicit selection gate (bare `--live` uploads all 8) —
  left as designed (the cue map is all 8); documented here and to the Lead — not a defect.” The window runbook
  (`board/probes/team-d/d5-window-day-runbook-20261101.md:98`) mentions the audio tool but not the all-8 warning —
  see observation O2.

## Regression

### R1 — `--selftest` against the real validator — PASS
2026-11-01T11:11:12-05:00, sentinel key exported (and never read):

```
SELFTEST temp root: /tmp/uacg-bindart-selftest-b6m6pvw9
SELFTEST PASS: receipt shapes accepted/refused exactly as publish.validate_mapping_receipt demands, and the stub-transport live path completes POST->poll->receipt with no AttributeError; no repo write, no network, no key read.
HARNESS rc=0 net_attempts=0 key_reads=0 key_access=[]
```
The validator is the real function (`publish.validate_mapping_receipt`, `:593` / `:682`), not a mock. Mutation test
(§D1) proves the stub case fails when the P0 is reintroduced.

### R2 — dry-run POST counts unchanged — PASS
2026-11-01T11:11:31-05:00, every line `x-api-key: <redacted>` and `state=ok`, `net_attempts=0`, `key_reads=0`:

| Selection | `^POST ` lines |
|---|---|
| default (all) | **24** |
| `--block a` | **6** |
| `--block b` | **6** |
| `--block c` | **12** |
| `--franchise Bleach` | **3** |

### R3 — sentinel key: 0 occurrences everywhere — PASS
`grep -rl 'UACG-RECHK-SENTINEL-4a7d2c1e'` over the execution copy (including the four evidence files and
`assets/images/roblox-ids.json`), the pristine copy, and every log: **0 files, 0 lines**. The only env read is one
`os.environ.get` at `bind_art.py:380` / `upload_audio.py:450`, both inside `--live`; dry/status/selftest/guards show
`key_reads=0`. Static scan finds no `print`/f-string interpolation of the key value.

### R4 — no automatic retry — PASS
2026-11-01T11:15:25-05:00, stub transport, socket-denied:

| Scenario | Calls | Journal | rc |
|---|---|---|---|
| POST transport failure | `["POST"]` **once** | `AMBIGUOUS` | 3 |
| no `operationId` in the POST response | `["POST"]` **once** | `AMBIGUOUS` | 3 |
| operation never `done` | `["POST", GET×8]` (bounded, `MAX_POLL_ATTEMPTS=8`) | `AMBIGUOUS` | 3 |

Static: `publish.request_bytes` makes exactly one `urlopen` (`grep -c urlopen publish.py` = 1) and converts every
failure to `RuntimeError`; no `requests`/`urllib3`/retry helper anywhere. Audio has the same shape
(`8 × one POST` in the failure stub; `8 × POST,GET` in the poll-failure stub — one attempt per cue, never a repeat).

### R5 — candidate bytes, repo cleanliness, zero writes — PASS
- Repo at 2026-11-01T11:16:23-05:00: `HEAD cd91a88…`, `git status --porcelain` **0 lines**, branches as above,
  `sha256sum build/UntitledAnimeCardGame.rbxlx` = `55cef5d3…d4c9`.
- Same hash in `/tmp/uacg-rechk2` and `/tmp/uacg-pristine2` after all runs.
- Pristine-copy whole-tree inventory (every file path/size/mtime hash + every directory hash) **identical** before
  and after `--selftest`, all five dry selections, `--status`, the absolute/`..` guards, and the audio dry run:
  `files=2902ca06…54f00`, `dirs=43470fc7…103a` both times. `0` stray `docs/verification/art-upload*` dirs;
  no directory created outside the copy; repo has `0` such dirs.

### R6 — static completeness — PASS
AST scan of every `publish.<attr>` reference: `bind_art.py` 21 attributes, `upload_audio.py` 12, **missing = []**.
`publish.py` is byte-identical to the pre-fix commit, so the fix is self-contained in the two tools. The only
removed lines in the fix commit are the defect itself and the weaker old guards/messages (no check was deleted).

---

## New defects found

**None.** The fix round is minimal and self-contained; every removed line is either the defect or a superseded guard
message. Residual observations only:

- **O1 (inherent, must be stated):** the `--live` HTTP path is **UNVERIFIED**. Auth-header acceptance, HTTP status
  semantics, the real moderation field names, and quota can only be proven by the owner's first block.
- **O2 (documentation placement):** D6's all-8-cue warning lives in the companion §3.5 but not in the window
  runbook. One line in `d5-window-day-runbook-20261101.md` next to the audio step would remove the last chance of a
  surprise. Not a code defect and not a blocker.
- **O3 (cosmetic):** in the one-card stub run the tool prints the `apply_art_ids.py` command without `--no-build`
  (correct: a single/last card is the build step). In a 6-card block, the first five get `--no-build` and the last
  builds — as designed.

## Boundaries of this re-check

- No real credential was ever present; the sentinel is a fake string. No socket was opened in any run
  (`net_attempts=0` everywhere; the transport was replaced in-process, and my harness has no `FIX_OPID` path).
- The `--live` control flow was exercised with a **synthetic** `target.json` written only into the `/tmp` copy
  (`universeId 1234567890`, `placeId 9876543210`, `creator.userId 1170918805`). The repo's `target.json` remains
  absent; no real id was fabricated anywhere that matters.
- The repo was read only (`git rev-parse`, `git log`, `git diff`, `git archive`, `sha256sum`, `git status`); it was
  never modified. All `/tmp` copies are throwaway.
- The owner-window sequence is unchanged: fill `target.json` from the owner's W2/D12/U0 values → export the key
  through the secure route → `python3 tools/bind_art.py --block a` (dry) → `--live --block a` → one
  `apply_art_ids.py --receipt <mapping> --no-build` per PASS receipt → blocks b and c → final build. **Block a's six
  real POSTs + six polls + six receipt pairs are the first proof of the HTTP path.**

## Appendix — evidence index (`/tmp/uacg-harness2/logs`, all 2026-11-01)

`selftest.out` 11:11:12 · `dry-all.out`/`block-a|b|c.out`/`fr-bleach.out` 11:11:31 · `d3-abs-dry.out`,
`d3-abs-live.out`, `d3-dotdot.out` 11:11:42 · `d3-writejson.out` 11:12:04 · `d2-bindart-newline.out`,
`d2-audio-newline.out` 11:12:25 · `d2-bindart-value_post|value_poll.out` 11:13:35 · `d2-audio-value_*.out`
11:13:55 · `d1-stub-ok.out` 11:14:19 · `d1-apply-nobuild.out` 11:14:35 · `d4-clean.out`, `d4-bytetamper.out`,
`d4-notdone.out` 11:14:48 · `d4-wrongcard.out`, `d4-wrongasset.out`, `d4-missing.out`, `d4-abspath.out` 11:15:06 ·
`d1-ambiguous.out`, `d1-noopid.out`, `d1-neverdone.out` 11:15:25 · `mutation-selftest.out` 11:15:48 ·
`key-*-*.out`, `audio-dry2.out` 11:16:43–11:16:56 · `body-probe.out` 11:17.

Harnesses (throwaway, mine): `/tmp/uacg-harness2/spy.py` (socket deny + env spy), `stub_live.py` (transport stub,
no `FIX_OPID`), `stub_valueerror.py`, `stub_audio.py`, `body_probe.py`, `write_json_probe.py`.
