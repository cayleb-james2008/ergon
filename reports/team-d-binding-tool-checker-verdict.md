# TEAM D — independent checker verdict: art-binding / audio-upload tools (2026-11-01)

**Checker:** fresh-context independent checker (did not write either tool). **Reports to:** Lead only.
**Window:** 2026-11-01 10:07–10:39 ET (`America/New_York`).
**Authority used:** read-only on the repo and its branches; write only to `/tmp` throwaway copies and this file.
**Not done, by design:** no live upload, no real network request, no real or test credential read or printed,
no Studio action, no publication, no repo/branch modification.

**Targets**

| Item | Value |
|---|---|
| Repo | `~/Work/projects/untitled-anime-card-game` |
| Branch / HEAD | `prep/art-binding-20261101` @ `ddcdd79627364c31ae3bdd617c1eb8755f6d9f8b` (parent `e7f885ca5274d773fc0026cb6c51d1161b71ae84`) |
| Audio branch | `prep/audio-upload-20261101` @ `e7f885c` (`tools/upload_audio.py` verified byte-identical between the two commits) |
| Files | `tools/bind_art.py` 620 lines, sha256 `b8ba4bb4…f0d8d`; `tools/upload_audio.py` 561 lines, sha256 `8ee2ad50…b43cc`; `ops/release/publish.py` 885 lines, sha256 `87dc2c1c…20604`; `tools/apply_art_ids.py` sha256 `285e7297…89b7` |
| Candidate of record | `build/UntitledAnimeCardGame.rbxlx` = `55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9` |
| Repo status | clean (`git status --porcelain` empty) before and after the check; HEAD/branch unchanged |

Method: `rsync -a --exclude .git` (twice-checked against `git archive ddcdd79`) into seven `/tmp/uacg-chk/*`
throwaway copies; every run wrapped in a harness that (a) denies `socket.socket.connect`,
`socket.create_connection`, `socket.getaddrinfo`, and (b) spies every `os.environ` access to the key variable.
A second harness replaces `publish.request_bytes` with an in-process stub (no socket at all) to exercise `--live`
control flow offline. Harness sources: Appendix. Raw logs: `/tmp/uacg-chk/logs/*.out` (throwaway; all decisive
output quoted verbatim below).

---

## Verdict summary

| # | Item | Verdict |
|---|---|---|
| 1 | `--selftest` PASS against the **real** `publish.validate_mapping_receipt`; APPROVED accepted; PENDING/REJECTED emit no mapping receipt and record the verdict | **PASS** |
| 2 | Dry run on the pinned candidate: 24 POST lines, key redacted, `target.json` ABSENT blocks `--live`, zero writes, zero network, key not read | **PASS** |
| 3 | Block counts 6/6/12; guards refuse `--live` without block, without key, unknown card, evidence-dir traversal | **PASS** (one guard nit, D3) |
| 4 | Sentinel key: zero occurrences in all output/files; key read only inside `--live`, never printed/written/stored/echoed | **PASS for `bind_art.py`** — including the malformed-key path |
| 5 | No automatic retry anywhere; ambiguous outcomes journalled then blocked; `--rebuild-receipts` re-emits without re-uploading | **PASS** (evidence-integrity nit, D4) |
| 6 | Candidate bytes `55cef5d3…` before and after every test; no evidence dirs from dry runs | **PASS** |
| 7 | Audio tool: 8 POST lines, zero network, measured durations, quota/no-retry gates, candidate bytes stable | **PASS on the listed gates; FAIL on key discipline** (D2: key echoed in a traceback) |
| 8 | `--live` HTTP path | **UNVERIFIED** (as required to state) — and **FAIL as shipped**: D1 crashes the first live card *after* its POST |

### Overall: **HOLD**

Not PASS. The dry-run/selftest/selection/evidence machinery is careful and, offline, sound — the emitted mapping
receipt is accepted not only by `publish.validate_mapping_receipt` but by the real downstream binder
`tools/apply_art_ids.py`. But the shipped `--live` path cannot complete one card: `tools/bind_art.py:309` calls
`publish.operation_id_of`, **which does not exist**, and the failure happens *after* the asset POST has been sent.
Do not run `--live` in the owner window until D1 is fixed and re-checked (see *Re-check procedure*). The audio
tool is usable only after D2's three-line fix, or with a key proven to contain no whitespace.

---

## 1 · `--selftest` against the real validator — PASS

```
$ cd /tmp/uacg-chk/art && PYTHONDONTWRITEBYTECODE=1 ROBLOX_OPEN_CLOUD_API_KEY='UACG-SENTINEL-7f3c9b1e-NOT-A-REAL-KEY' \
    python3 /tmp/uacg-chk/tools/spy.py tools/bind_art.py --selftest
SELFTEST temp root: /tmp/uacg-bindart-selftest-0_s833t3
SELFTEST PASS: PASS/PENDING/REJECTED receipt shapes accepted/refused exactly as publish.validate_mapping_receipt demands; no repo write, no network, no key read.
HARNESS rc=0 net_attempts=0 key_reads=0 key_access=[]
```
(2026-11-01T10:19:28-05:00; tool rc 0; temp dir removed afterwards — `ls -d /tmp/uacg-bindart-selftest-*` → none.)

Code path read: `run_selftest()` (bind_art.py:513-576) builds a `/tmp` tree from the **real** `manifest.json`,
`Cards.luau` and `BC-ASTA-01.png`, swaps `publish.ROOT`, and calls `publish.validate_mapping_receipt` — the real
function, not a mock (`:549`). APPROVED → receipt accepted (`:549-551`); PENDING/REJECTED → `mapping_receipt is
None` and the moderation receipt's `verdict` must equal `PENDING`/`REJECTED` (`:554-566`).

Because a self-test can pass vacuously, I added an independent negative test of the same validator
(`/tmp/uacg-chk/tools/negative_validator.py`, run 2026-11-01T10:19:41-05:00):

```
OK       1 PASS mapping receipt accepted by publish.validate_mapping_receipt :: 1234567890
BLOCKED  2 tampered asset_id refused :: upload_receipt evidence does not match asset_id
BLOCKED  3 wrong source hash refused :: mapping source hash is not the current approved image for BC-ASTA-01
BLOCKED  4 byte-tampered upload receipt refused :: upload_receipt evidence hash mismatch
OK       5 PENDING emits no mapping receipt :: mapping_receipt=None
OK       6 PENDING verdict recorded verbatim :: verdict=PENDING raw_state=MODERATION_STATE_PENDING
BLOCKED  7 hand-built mapping over PENDING moderation refused :: moderation evidence verdict is not PASS
OK       8 REJECTED emits no mapping receipt :: mapping_receipt=None
```

The validator is genuinely strict (schema, per-file sha256, receipt_id, identity fields, verdict, `returned_on` /
`moderated_on` dates, 30-day freshness). Caveat: the selftest itself contains no negative case; item 1's "refused"
direction is proven by my test, not by the tool.

## 2 · Dry run on the pinned candidate — PASS

```
$ python3 /tmp/uacg-chk/tools/spy.py tools/bind_art.py            # 2026-11-01T10:20:11-05:00
DRY RUN: 24 card(s); no network, no writes, no key read.
target.json: ABSENT/BLOCKED — ops/release/target.json evidence file is missing or outside the project
build receipt: current (55cef5d3ea71173d…)
evidence dir (live): docs/verification/art-upload-20261101/
blocks: a = Bleach+Naruto (6) · b = Dragon Ball+Berserk (6) · c = … (12). --live requires --block/--cards/--franchise, or --all explicitly.
POST https://apis.roblox.com/assets/v1/assets (x-api-key: <redacted>) card=BC-SEA-01 franchise=Black Clover name=uacg-art-BC-SEA-01 sha256=5d336d1bf3a2968c state=ok
…
24 planned upload(s); 0 not uploadable as-is. Audio cues are a separate path (tools/upload_audio.py).
HARNESS rc=0 net_attempts=0 key_reads=0 key_access=[]
```

- 24 `^POST ` lines, all with the literal `x-api-key: <redacted>`; 24/24 `state=ok`; no unredacted header anywhere.
- `target.json` is genuinely absent (`ls ops/release/target.json` → missing) and `--live --block a` refuses on it
  (`BLOCKED: ops/release/target.json evidence file is missing or outside the project`, HARNESS rc=2,
  net_attempts=0, 2026-11-01T10:20:50-05:00) — i.e. live is blocked before any POST.
- **Zero writes:** whole-tree inventory (`find` of every file's path/size/mtime + every directory, sorted, hashed)
  was `e0558a52ad01b4bc8f9341c2ae1223ee1a144419bed04509eb225515761a1638` before the run and identical after
  selftest + 24-card dry + blocks a/b/c + franchise + `--status` + five guard runs. No
  `docs/verification/art-upload-*` directory exists in the copy or the repo. Re-checked at the end of the whole
  session (10:39): `ART_FINAL_INV` is still `e0558a52…a1638`.
- **Zero network:** all sockets were denied before the tool imported; `net_attempts=0` in every dry run. Read of
  the code path agrees: plan mode only calls `read_images`, `current_image`, `current_card_franchises`,
  `load_target`, `require_current_build`, `read_asset_registry`.
- **Key not read:** the key variable *was* set in the environment for these runs, yet the env spy recorded
  `key_reads=0`. The single `env.get` in the tool is bind_art.py:367, inside `run_live`.

## 3 · Block counts and guards — PASS

| Selection | `^POST ` lines | harness |
|---|---|---|
| (default, all) | **24** | rc 0, net 0, key_reads 0 |
| `--block a` | **6** | rc 0, net 0, key_reads 0 |
| `--block b` | **6** | rc 0, net 0, key_reads 0 |
| `--block c` | **12** | rc 0, net 0, key_reads 0 |
| `--franchise Bleach` | **3** | rc 0, net 0, key_reads 0 |

All at 2026-11-01T10:20:11-05:00. Counts match the companion's claim exactly.

Guards (2026-11-01T10:20:30-05:00, all rc 2, `net_attempts=0`, nothing sent):

```
--live (no selection)      BLOCKED: a live run needs an explicit block: --block a|b|c, --cards, --franchise, or --all (the default mode is the dry run)      key_reads=0
--live --all (no key)      BLOCKED: environment variable ROBLOX_OPEN_CLOUD_API_KEY is missing; … (never printed or stored by this tool)                    key_reads=1
--live --cards NOPE-01     BLOCKED: unknown card id(s): NOPE-01                                                                                          key_reads=0
--live --block a --evidence-dir ../escape   BLOCKED: evidence dir must stay inside the project                                                             key_reads=0
--live --franchise Nope    BLOCKED: unknown franchise(s): nope; known: Berserk, Black Clover, …                                                            key_reads=0
--block z                  argparse: invalid choice (rc 2)
```
`--live --block a --cards` ordering is also correct: selection and card validation run before the key is read.

**Guard nit (D3, LOW):** the traversal check in `main()` is
`if not evidence_dir.is_absolute() and ".." in evidence_dir.parts` (bind_art.py:599). An **absolute**
`--evidence-dir /tmp/...` passes it, and in a writing mode `write_json()` calls
`destination.parent.mkdir(parents=True, exist_ok=True)` (bind_art.py:170) *before* `publish.atomic_write_project`
refuses the absolute path. Demonstrated (2026-11-01T10:36:0x): `bind_art.write_json(Path('/tmp/uacg-chk/abs-escape-probe/card/upload-receipt.json'), …)`
→ `BLOCKED: evidence receipt output path must stay inside the project`, but
`outside dir created anyway: True | children: ['card']`, `file written? False`. No data escapes; an empty directory
outside the project does. Dry runs are unaffected (no `mkdir` in plan mode). Fix: reject absolute evidence dirs in
`main()` and move the `mkdir` behind the guard.

## 4 · Sentinel-key test (mine) — PASS for `bind_art.py`

Sentinel used: `UACG-SENTINEL-7f3c9b1e-NOT-A-REAL-KEY` (a fake string, not a credential). It was set in
`ROBLOX_OPEN_CLOUD_API_KEY` for every art-tool run, including the dry runs, `--status`, `--selftest`, all guards,
and the stubbed live runs (where the header is actually built).

- `grep -ro 'UACG-SENTINEL-7f3c9b1e'` over all five art copies and all logs: **0 hits** — including the stubbed
  `--live` runs that built `{'x-api-key': <key>, 'Content-Type': …}`, the written `raw-operation.json`, the
  upload/moderation/mapping receipts and `assets/images/roblox-ids.json`.
- `raw-operation.json` records `{card_id, operation_result, post_response, recorded_utc}` only — no request
  headers, no key.
- Env spy: `key_reads=0` in dry/status/selftest/guard paths; `key_reads=1` (one `env.get`) in every `--live` run.
  Static check agrees: `os.environ` is touched in this repo only at `bind_art.py:367`, `upload_audio.py:450`,
  `publish.py:138/779/812`, and no code writes the value anywhere.
- Malformed-key path (the one that matters): with a **newline inside** the key, CPython raises
  `ValueError("Invalid header value b'<KEY>'")` inside `http.client.putheader` *before* connecting.
  `bind_art.py` catches `ValueError` in the POST `except` (`:310`) and replaces the message:
  `FAILED BK-GUTS-01: … upload outcome is ambiguous; journalled AMBIGUOUS and NEVER retried`, rc 3,
  **0 sentinel hits** (2026-11-01T10:38:36-05:00). `upload_audio.py` does **not** — see D2.

Wording nit: the key is read once at `run_live` entry (`:367`), not "immediately before the header" (`:304-306`);
still `--live`-only and never re-read. No security impact.

## 5 · No automatic retry; ambiguous outcomes blocked; `--rebuild-receipts` — PASS

Static: `publish.request_bytes` (publish.py:578-586) makes exactly one `urlopen` and converts every failure to a
bare `RuntimeError`; no `requests`/`urllib3`/retry helper is imported anywhere in the project; the only loops over
the network are bounded operation polls (`MAX_POLL_ATTEMPTS = 8`, `POLL_BACKOFF_SECONDS` 0.5…8 s), which re-read
one operation and never re-POST.

Dynamic (stub transport, no socket, sentinel key, 2026-11-01T10:24:52–10:27:08):

| Scenario | Calls observed | Journal | rc |
|---|---|---|---|
| `--live --cards BC-ASTA-01` (shipped code) | `["POST"]` | `SUBMISSION_STARTED` | harness 97 = uncaught `AttributeError` (a real shell prints the traceback and exits 1) |
| re-run of the same card | `[]` | unchanged | 3 — `prior upload unresolved (SUBMISSION_STARTED); resolve by hand — never retry` |
| POST raises (transport failure) | `["POST"]` **once** | `AMBIGUOUS` | 3 |
| poll never returns `done` | `["POST", GET×8]` (31.5 s) | `AMBIGUOUS` | 3 — `operation result is ambiguous; journalled AMBIGUOUS and NEVER retried` |

So: no retry of a failed or unresolved upload anywhere, and every unresolved outcome stops the run with rc 3.

`--rebuild-receipts` (2026-11-01T10:23:35, synthetic journal because no real upload exists): re-emitted
`upload-receipt.json` + `moderation-receipt.json` + `mapping-receipt.json` from the recorded raw operation with
`net_attempts=0`, `key_reads=0`, no POST. The emitted receipt was then accepted by the real downstream binder:
`python3 tools/apply_art_ids.py --receipt …/mapping-receipt.json --dry-run` →
`Validated offline mapping: BC-ASTA-01 → 1234567890`, rc 0.

**Nit (D4, LOW):** `run_rebuild` never re-reads the recorded raw file. With `raw-operation.json` deliberately
deleted (2026-11-01T10:23:59) it still emitted PASS receipts and a mapping receipt that `apply_art_ids` accepted,
whose `raw_operation` pointer (`docs/verification/rebuild-probe/BC-ASTA-01/raw-operation.json`,
sha `ec7bb394…`) no longer exists. Only the upload/moderation receipts are hash-verified downstream. Fix: re-hash
(or refuse on a missing) recorded raw file before re-emitting.

## 6 · Candidate bytes and stray writes — PASS

`sha256sum build/UntitledAnimeCardGame.rbxlx` = `55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9`
**before and after every test**, in every copy, and in the repo:

```
art 55cef5d3…d4c9   audio 55cef5d3…d4c9   rebuild 55cef5d3…d4c9
live 55cef5d3…d4c9  live2 55cef5d3…d4c9   chain 55cef5d3…d4c9   leak 55cef5d3…d4c9
repo: prep/art-binding-20261101 @ ddcdd796… ; 55cef5d3…d4c9 ; git status --porcelain empty
```
No `docs/verification/art-upload-*` directory was created by any dry run (repo: 0 such dirs).

## 7 · Audio tool (`tools/upload_audio.py`) — PASS on the listed gates, FAIL on key discipline

```
$ env -u ROBLOX_OPEN_CLOUD_API_KEY python3 /tmp/uacg-chk/tools/spy.py tools/upload_audio.py      # 10:29:22
DRY RUN: 8 audio upload(s); no network requests are made and the key is not read.
target.json: BLOCKED (live only): ops/release/target.json is absent — U0 … is the precondition
POST … (x-api-key: <redacted>) cue=contact_normal … duration=2.0s (measured)
… 8 lines, every one "(measured)" …
QUOTA: audio is non-updatable; if this account is not ID-verified only 10 audio uploads/month are allowed. …
HARNESS rc=0 net_attempts=0 key_reads=0 key_access=[]
```
- **8** `^POST ` lines, **8/8** durations `(measured)` (6 WAV via RIFF header, 2 FLAC via STREAMINFO — 45.0 s,
  40.0 s), 0 network attempts, key not read, whole-tree snapshot identical before/after, candidate `55cef5d3…`.
- Guards: `--live` without key → rc 2 `environment variable … is missing` (net 0); `--live --cue nope` → rc 2
  `unknown cue(s): nope`; `--live` with sentinel key but no `target.json` → rc 2 on target, net 0.
- No-retry gates (stub, no socket): PASS path → `["POST","GET"]`, rc 0, receipt `asset_id=1234567890`,
  `verdict=PASS`, `duration_note=measured`, state `ASSET_ID_RETURNED`; transport failure → `["POST"]` once,
  `AMBIGUOUS`, rc 3; never-done → `["POST", GET×8]`, rc 3, state `OPERATION_PENDING`. Same fail-closed shape as
  the art tool. All nine audio file hashes are distinct, so the duplicate-bytes quota warning never fired.
- **Quota gate is informational only:** the 10/month text is printed in the dry run; nothing counts or refuses a
  live run. There is also **no explicit selection gate** for audio (a bare `--live` sends all 8 cues), unlike
  `bind_art.py` which refuses `--live` without `--block/--cards/--franchise/--all`. Acceptable if the owner intends
  all eight, but it deserves one line in the runbook.

**D2 (MEDIUM, credential echo):** the audio tool's POST `except` is
`except (RuntimeError, UnicodeDecodeError, json.JSONDecodeError)` (`upload_audio.py:380`) — **`ValueError` is not
caught**, unlike `bind_art.py:310`. `run_live` catches only `RuntimeError` (`:460`), `main` only
`Blocked`/`RuntimeError` (`:552-557`). A key containing a newline (a classic paste/`$(cat)` artifact) makes
`http.client.putheader` raise `ValueError('Invalid header value %r' % value)` — **the value is in the message** —
and Python prints the uncaught traceback to stderr. Reproduced 2026-11-01T10:37:55 with the sentinel plus a
newline:

```
ValueError: Invalid header value b'UACG-SENTINEL-7f3c9b1e\nINJECT'
SENTINEL HITS IN OUTPUT: 1
```
No socket was opened (`net_attempts=0`) and no upload occurred (journal `SUBMISSION_STARTED`), but the credential
was echoed to the terminal/log — the one thing the design says must never happen. Fix: add `ValueError` to the
`except` at `:380`, and ideally pre-validate the key in `run_live` (reject whitespace/non-latin-1 with a message
that never includes the value). With a well-formed key the sentinel count is **0**.

## 8 · The `--live` HTTP path — UNVERIFIED, and blocked by D1

**UNVERIFIED, honestly:** no real key exists and the owner window has not opened, so no real HTTP request was made.
What *is* verified offline (stub transport, no socket) is the control flow, the request body, and the evidence chain:

- The multipart body is well-formed and bound to the approved bytes: captured POST body (1,404,596 B) parses as
  exactly two segments, `Content-Type: multipart/form-data; boundary=----uacg-art-…` matches the body markers,
  preamble empty / epilogue `--\r\n`; request JSON is
  `{"assetType":"Decal","displayName":"uacg-art-BK-GUTS-01","description":"… Source BK-GUTS-01 / 877902fe3988","creationContext":{"creator":{"userId":"1170918805"}}}`
  and the file segment is **byte-identical** to `assets/images/BK-GUTS-01.png`, matching the manifest sha256.
  (2026-11-01T10:32:03)
- **D1 (P0, blocks live use).** `tools/bind_art.py:309` calls `publish.operation_id_of(response)`, but
  `ops/release/publish.py` (both commits) has no such attribute — only `operation_asset_id` (`:670`), and no
  module `__getattr__`. It is the *only* missing one of the 22 `publish.*` attributes `bind_art.py` uses (AST scan);
  `upload_audio.py` has all 12 and carries its own local `operation_id_of` (`upload_audio.py:322`). `publish.py`
  parses the id inline at `:727-731`. Runtime proof (2026-11-01T10:24:52, stub transport, no socket):

```
  File "tools/bind_art.py", line 309, in upload_one
    operation_id = publish.operation_id_of(response)
AttributeError: module 'ops.release.publish' has no attribute 'operation_id_of'
STUB_RESULT {"rc": 97, "net": 0, "methods": ["POST"], "calls": [{"method": "POST", … "header_names": ["Content-Type", "x-api-key"], "body_len": 1488719}]}
journal: {"BC-ASTA-01": {"state": "SUBMISSION_STARTED", …}}
receipts on disk: none
```
  Impact in the owner window: the **first card's asset is really uploaded**, then the tool dies before it can read
  `operationId`, capture the raw operation, or write any receipt — the asset id exists only in Roblox, the journal
  is stuck at `SUBMISSION_STARTED`, and `--rebuild-receipts` refuses (it requires `ASSET_ID_RETURNED`), so recovery
  is fully manual. The remaining cards in the block are not sent (fail-closed). `AttributeError` is not in the
  POST `except` (`:310`) nor in `run_live`'s (`:383`), so this is a hard crash, not an rc-3 block.
  Fix (3 lines): inline `response.get("operationId")` exactly as `publish.py:727-731` does, journaling `AMBIGUOUS`
  when absent — or copy `upload_audio.operation_id_of`. Then add a stub-transport `--selftest` case so this class
  of defect is caught offline.
- With only that one line supplied (fix simulation, no repo edit), the rest of the path is coherent:
  `--live --cards BC-SEA-01` → `["POST","GET"]`, rc 0, `moderation=PASS`, four evidence files, state
  `ASSET_ID_RETURNED` (10:26:11), and the emitted receipt passes the real binder
  `apply_art_ids.py --receipt … --no-build` → `Validated offline mapping: BC-SEA-01 → 1234567890`,
  `Mapping recorded; …`, registry `BOUND`, ArtIds updated, build still `55cef5d3…` (10:35:14).

**What the first live block must prove (unchanged, and currently impossible to reach):** block **a = 6 cards**
(Bleach+Naruto) is the smallest true test; it must show 6 real POSTs + 6 operation polls, six
`upload-receipt.json`/`moderation-receipt.json` pairs, six `mapping-receipt.json` only for `PASS` verdicts, and
`apply_art_ids.py --receipt <mapping> --no-build` must pass `publish.validate_mapping_receipt` for each before
anything else proceeds (then blocks b and c, then the final build).

---

## Defects

| ID | Sev | Where | Effect | Fix |
|---|---|---|---|---|
| **D1** | **P0** | `tools/bind_art.py:309` → missing `publish.operation_id_of` | First live card uploads, then crashes before any receipt/asset id is recorded; manual reconciliation only | Inline `response.get("operationId")` as `publish.py:727-731`, journal `AMBIGUOUS` if absent; add stub-transport selftest |
| **D2** | **MED** | `tools/upload_audio.py:380` (missing `ValueError`) + `:460`, `:552` | A key with a newline/control char is echoed verbatim in an uncaught `ValueError` traceback (sentinel hit = 1, reproduced) | Add `ValueError` to the `except`; pre-validate the key without echoing it |
| **D3** | LOW | `bind_art.py:599` guard + `write_json` `mkdir` at `:170` | Absolute `--evidence-dir` bypasses the `..` check; an empty directory outside the project is created before the write is refused | Reject absolute evidence dirs; move `mkdir` behind the guard |
| **D4** | LOW | `bind_art.py:408-438` (`run_rebuild`) | Re-emits PASS receipts without re-verifying the recorded `raw-operation.json`; a dangling raw pointer is still accepted downstream | Re-hash/refuse on the recorded raw file |
| **D5** | DOC | companion `team-d-g1-g3-companion.md` §1 | Says `bind_art.py` is "561 lines"; it is **620** (561 is `upload_audio.py`). The probe file says 620 correctly | Correct the number |
| **D6** | OBS | `upload_audio.py:548` | `--live` has no explicit selection gate: a bare `--live` uploads all 8 cues; quota is printed, not enforced | Document, or mirror the art tool's explicit-selection guard |

Nothing else was found: no automatic retry, no unredacted key in any dry/status/selftest/guard output, no writes by
dry runs, no candidate-byte movement, no branch or repo modification, no network attempt in any run.

## Re-check procedure after the D1/D2 fixes (offline, ~3 min)

1. `rsync -a --exclude .git ~/Work/projects/untitled-anime-card-game/ /tmp/uacg-recheck/`
2. Re-run the socket-deny/env-spy harness (Appendix A) on `--selftest`, the two dry runs and the guards; expect
   `net_attempts=0`, `key_reads=0`, identical POST counts 24/6/6/12/3.
3. Re-run the stub harness (Appendix B) with `FIX_OPID` unset (the real code must no longer need it):
   `--live --cards <one card>` must give `["POST","GET"]`, rc 0, four evidence files, journal
   `ASSET_ID_RETURNED`; then `apply_art_ids.py --receipt … --no-build` must print `Validated offline mapping`.
4. Re-run the audio newline-key sentinel test (§7): after the `ValueError` fix it must exit rc 3 with
   `SENTINEL HITS IN OUTPUT: 0`.
5. Confirm `sha256sum build/UntitledAnimeCardGame.rbxlx` is unchanged throughout.

## Boundaries of this check

- No real credential was ever present; the sentinel is a fake string. No socket was opened in any run
  (`net_attempts=0` everywhere; the stub replaced the transport in-process).
- `--live` HTTP semantics (auth, status codes, moderation field names as actually returned, quota) remain
  **UNVERIFIED** and can only be proven by the owner's first block.
- All `/tmp/uacg-chk` copies are throwaway. The repo was read only: branch, HEAD, file hashes, `git show`, `git status`
  — all unchanged at the end of the check, candidate still `55cef5d3…`.

## Appendix A — socket-deny / env-spy harness (`/tmp/uacg-chk/tools/spy.py`)

```python
import os, sys, runpy, socket, traceback
KEY_NAME = os.environ.get("SPY_KEY_NAME", "ROBLOX_OPEN_CLOUD_API_KEY")
state = {"net": 0, "key_access": []}
_real_env = os.environ
class EnvSpy:
    def __getitem__(self, k):
        if k == KEY_NAME: state["key_access"].append("getitem")
        return _real_env[k]
    def get(self, k, default=None):
        if k == KEY_NAME: state["key_access"].append("get")
        return _real_env.get(k, default)
    def __contains__(self, k):
        if k == KEY_NAME: state["key_access"].append("contains")
        return k in _real_env
    def __setitem__(self, k, v): _real_env[k] = v
    def __delitem__(self, k): del _real_env[k]
    def __iter__(self): return iter(_real_env)
    def __len__(self): return len(_real_env)
    def keys(self): return _real_env.keys()
    def items(self): return _real_env.items()
    def values(self): return _real_env.values()
    def copy(self): return _real_env.copy()
    def setdefault(self, k, d=None): return _real_env.setdefault(k, d)
    def pop(self, k, *a): return _real_env.pop(k, *a)
    def update(self, *a, **k): return _real_env.update(*a, **k)
    def __repr__(self): return "<env spy>"
os.environ = EnvSpy()
class NetAttempt(RuntimeError): pass
def _deny(*a, **k):
    state["net"] += 1
    raise NetAttempt("network attempt blocked by checker harness")
def _deny_connect(self, address):
    state["net"] += 1
    raise NetAttempt("network attempt blocked by checker harness")
# patch methods, not the class: ssl.SSLSocket subclasses socket.socket at import time
socket.socket.connect = _deny_connect
socket.create_connection = _deny
socket.getaddrinfo = _deny
script, tool_argv = sys.argv[1], sys.argv[2:]
rc = 0
sys.argv = [script] + tool_argv
try:
    runpy.run_path(script, run_name="__main__")
except SystemExit as exc:
    rc = exc.code if isinstance(exc.code, int) else 1
except NetAttempt as exc:
    print(f"HARNESS: NETWORK ATTEMPT ESCAPED: {exc}", file=sys.stderr); rc = 98
except BaseException:
    print("HARNESS: UNCAUGHT EXCEPTION IN TOOL:", file=sys.stderr); traceback.print_exc(); rc = 97
print(f"HARNESS rc={rc} net_attempts={state['net']} key_reads={len(state['key_access'])} key_access={state['key_access']}")
sys.exit(rc)
```

## Appendix B — offline `--live` stub (`/tmp/uacg-chk/tools/stub_live.py`)

Same socket denial as Appendix A, plus before the tool runs:

```python
from ops.release import publish
MODE, STATE = os.environ.get("STUB_MODE", "ok"), os.environ.get("STUB_STATE", "MODERATION_STATE_APPROVED")
def fake_request_bytes(url, method, body=None, headers=None, timeout=30):
    calls.append({"method": method, "url": url, "header_names": sorted((headers or {}).keys()), "body_len": len(body or b"")})
    if method == "POST":
        if MODE == "fail_post": raise RuntimeError("Open Cloud request failed")
        return json.dumps({"operationId": "op-stub-0001"}).encode()
    if MODE == "never_done": return json.dumps({"done": False}).encode()
    return json.dumps({"done": True, "response": {"assetId": "1234567890",
            "moderationResult": {"moderationState": STATE}}}).encode()
publish.request_bytes = fake_request_bytes
# FIX_OPID=1 injects the missing helper in memory only, to show what the shipped path would do after D1 is fixed
```
Then `runpy.run_path("tools/bind_art.py", run_name="__main__")` with `sys.argv` rewritten; prints
`STUB_RESULT {rc, net, methods, calls}` (header **names** only; header values never recorded).

## Appendix C — evidence index (`/tmp/uacg-chk/logs`, 2026-11-01)

`selftest.out` 10:19:28 · `negval.out` 10:19:41 · `dry-all.out` 10:20:11 (24 POSTs) · `block-a/b/c.out`,
`fr-bleach.out` 10:20:11 (6/6/12/3) · `g1…g7.out` 10:20:30–10:33:32 · `status.out` 10:20:50 ·
`rebuild3.out` 10:23:35 · `apply-dry3.out` 10:23:35 · `rebuild-dangling.out`/`apply-dangling.out` 10:23:59 ·
`stub-s1.out` 10:24:52 (D1) · `stub-s2.out` 10:25:35 · `fixsim-ok.out` 10:26:11 · `fixsim-failpost.out` 10:26:39 ·
`fixsim-neverdone.out` 10:27:08 · `audio-dry2.out` 10:29:22 · `audio-stub-ok.out` 10:30:02 ·
`audio-failpost.out` 10:30:22 · `audio-neverdone.out` 10:30:23 · `body-capture3.out` 10:32:03 ·
`chain1.out`/`chain2.out` 10:35:14 · `leak-audio2.out` 10:37:55 (D2) · `leak-bindart.out` 10:38:36.
