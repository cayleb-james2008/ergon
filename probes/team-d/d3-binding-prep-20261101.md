# D-3 — Job 17 binding prep: exact owner screen, upload/bind steps, style brief (2026-11-01)

Prepared by `roblox-lead` (Team D). Tier A, `$0`, **nothing uploaded, nothing published, no credential touched.**
Companions: `upload-plan.md` (ordering/privacy rules), `audio-import-design.md` (audio route), `studio-evidence-sheet.md`
(per-card/per-cue proof), `d3-cast-table-20261101.json` (24-row data), `style-brief-20261101.md` (the art brief).

## 0 · What is actually blocking Job 17 today (honest, in order)

| # | Block | State | Whose |
|---|---|---|---|
| B1 | Upload authorisation ("upload for binding only, no publication") | in the owner channel (ASK D12) | Cayleb, via Nox |
| B2 | `ops/release/target.json` — the experience's `universeId`/`placeId` | **absent**; both `ReleaseConfig.luau` files hold `universeId = 0, placeId = 0` ("keep the published-server path closed until the owner selects the experience") | Cayleb selects/creates the unlisted experience (one screen, §1) |
| B3 | Open Cloud key with Assets API write, under the owner's account | absent from this machine; must arrive through the secure write-only route (never chat) | Cayleb (P9) |
| B4 | **Tooling gap found this batch**: the project's own uploader does not record moderation results and has no evidence-receipt generator; `apply_art_ids.py` refuses to bind without two hash-pinned evidence files per card | new, precise | Lead ruling: minute-scope code at the upload step (§3.4) |
| B5 | Studio per-card/per-cue verification | needs the shared Studio window (D-5) | Cayleb window + agent |

Art/audio data itself is ready: 24/24 portraits file-verified (740×1024, manifest hashes match, 0 bound), 8/8 cues
`roblox_id: null`, rights remain **USER-ATTESTED** (never claimed cleared).

## 1 · U0 — the exact owner screen for the experience selection

The team will **not** fabricate `universeId`/`placeId`. The owner's smallest act:

1. In the desktop browser, open **https://create.roblox.com/dashboard/creations** and sign in only if asked.
   Check the name in the top-right corner: it must be **Chrono_H3X** (the account Studio is signed into). If a
   different account is shown, stop and say so.
2. Click **Create new experience** → choose the **Baseplate** template → name it anything neutral
   (e.g. `uacg-internal-target`) → **Create**. This is a **private** experience draft: do **not** publish it, do not
   add passes/products, do not invite players, do not change any public setting. (If he prefers to reuse an existing
   experience he owns, that is fine — same next step.)
3. When the experience page opens, **copy the whole address bar** and send it back through Nox/the Lead. It looks
   like `https://create.roblox.com/dashboard/creations/experiences/<UNIVERSE_ID>/places/<PLACE_ID>/…`.
   If the URL carries only the universe id, click the **Places** tab, open the baseplate place, and copy that URL
   instead — both numbers must be visible.
4. That is all. The team then writes (never commits) `ops/release/target.json`:

```json
{"universeId": "<from URL>", "placeId": "<from URL>", "creator": {"userId": "1170918805"}}
```

`creator.userId = 1170918805` is already recorded in the street game's `docs/STATE.md` ("owner Chrono_H3X
(1170918805)") — the dashboard must show the same account, or we stop. Verification after writing (agent, no
network): `python3 ops/release/publish.py status` shows `target.json: valid` and the dry run
`python3 ops/release/publish.py upload-art` still prints 24 candidate POSTs with `x-api-key: <redacted>`.
`publish-place` must stay blocked/unauthorised — publication is not authorised.

## 2 · P9 — the Open Cloud key (owner act, secure route only)

Owner: Creator Dashboard → **Credentials / API Keys** → create a key **on his own account**, restricted to the
**Assets API with Read+Write** (nothing else; no groups, no datastores, no place publishing). The key value is shown
once; he pastes it through the **secure write-only route** the programme provides — **never into chat, never to the
team, never in a file the team reads**. The team only ever sees "variable present: yes/no". If the tool says the
variable is missing, that is his paste step, not an agent improvisation.

Quota facts to state before he authorises (both matter): audio uploads are **non-updatable**, and the audio cap is
**10/month if the account is not ID-verified** (100 if verified) — 8 cues is one shot. Image/decal uploads have a
separate cap; treat moderation outcomes as final.

## 3 · Upload → bind → verify, exact steps (after B1/B2/B3)

### 3.1 Order (small blast radius)

| Step | What | Count | Command / artefact |
|---|---|---|---|
| U0 | `target.json` written + validated | 1 | see §1 |
| U1 | **Audio finals** (fail fast, smallest) | 8 | `python3 tools/upload_audio.py` (dry) → `--live` only on the owner's go |
| U2 | Portraits, franchise block A (Bleach + Naruto) | 6 | `ops/release/publish.py upload-art` — see gap G3 below |
| U3 | Portraits, block B (Dragon Ball + Berserk) | 6 | same |
| U4 | Portraits, block C (Vinland Saga + Vagabond + MHA + Black Clover) | 12 | same (or 2×6) |
| U5 | Receipts → `apply_art_ids.py` → rebuild → Studio evidence sheet | 24 + 8 | §3.3–3.5 |

Neutral asset names only (moderation hygiene): `uacg-art-<card_id>`, `uacg-cue-<purpose>`; descriptions
"Untitled card game — internal art/cue, not for public use. Source <id> / <sha256 first 12>." No franchise or
character words in asset names. Most restrictive privacy the API allows; no publication, no store listing, no money.

### 3.2 Per-upload record (the row that makes binding legal)

For every asset: `card_id`/`purpose`, `source_sha256` (exact bytes uploaded), `source_file`, returned `asset_id`
(6–19 digits), `target` (exact creator/universe/place), `franchise` (from `Cards.luau`), `asset_name`, raw
upload/moderation response, `operator`, UTC timestamp, **status as returned** (a PENDING/REJECTED moderation is
recorded as such — never rounded to PASS).

### 3.3 Evidence files the binding gate actually demands (new, precise)

`publish.validate_mapping_receipt()` requires, **per card**, two project-relative JSON evidence files whose bytes are
SHA-256-pinned inside the mapping receipt:
`upload_receipt` → `schema: uacg-upload-receipt/v1`, `status: COMPLETE`, `returned_on: YYYY-MM-DD`, `receipt_id`
referenced, and matching `card_id/source_sha256/target/franchise/asset_id`;
`moderation` → `schema: uacg-moderation-receipt/v1`, `verdict: PASS`, `moderated_on: YYYY-MM-DD` (fresh ≤30 days),
same identity fields. Then `tools/apply_art_ids.py --receipt <mapping.json>` binds one card
(`schema: uacg-asset-mapping/v1`, `card_id`, `source_sha256`, `asset_id`, `target`, plus the two evidence blocks)
and rebuilds.

**Gaps found in the project's own tooling (recorded, not smoothed over):**
- **G1 — no receipt generator.** `publish.py upload-art --live` returns asset ids but writes no upload/moderation
  evidence files; nothing in the repo generates the two files above.
- **G2 — moderation is not captured.** The uploader polls operations and returns `assetId`; it does not persist
  `moderationResult.moderationState`. Without it a PASS verdict cannot be honestly recorded.
- **G3 — no block filter.** `upload-art --live` loops all 24 cards in one invocation; the "franchise blocks" in
  `upload-plan.md` are not supported (an already-bound card aborts the loop). Either upload all 24 in one run and
  treat any stop as a full stop, or add a 10-line `--cards` filter.
- **Recommendation (Lead ruling):** authorise one small companion at the upload step — either extend
  `publish.py` to persist the raw responses + emit the two evidence files, or a `tools/bind_art.py` that wraps the
  same API calls, records raw JSON, and writes the receipts. It is the difference between "bindable" and "a 48-file
  manual transcription with no raw evidence". Until that ruling, U5 cannot be completed honestly.

**UPDATE 2026-11-01 — RULED AND BUILT.** The Lead authorised the companion; `tools/bind_art.py` is written and
proven offline (branches `prep/art-binding-20261101` and `prep/audio-upload-20261101`, commit `cd91a88`; evidence
`board/probes/team-d/g1-g3-tooling-20261101.txt`). It closes G1 (hash-pinned upload + moderation evidence and the
mapping receipt in the exact shape `publish.validate_mapping_receipt` accepts), G2 (persists `moderationState`
into `assets/images/roblox-ids.json`), G3 (block filter `--block a|b|c` = 6/6/12, plus `--cards`/`--franchise`/
explicit `--all`). It is **dry-run only** until the window: the default mode prints 24 POST lines with
`x-api-key: <redacted>`, writes nothing, makes no network call and does not read the key; `--selftest` exercises
the receipt builder against the real validator in a throwaway /tmp tree; the pinned candidate stayed `55cef5d3…`
before/after. The three gaps are closed at the tooling level; the human gate (owner authorisation + target +
key) still stands.

### 3.4 Failure handling (unchanged, binding)

| Symptom | Action |
|---|---|
| duplicate id refusal | card already bound — verify, never force |
| "prior upload unresolved" | resolve by hand (find the asset in the creator list); **no automatic retry** |
| moderation not PASS | keep the row, mark the card blocked, report; never re-upload the same bytes hoping |
| ambiguous response | stop, record AMBIGUOUS, human looks — never blindly retry (audio is non-updatable; quota is finite) |
| 429 / rate limit | back off, resume from the recorded rows |
| any credential request the tool does not already hold | stop — owner's secure paste |
| any money prompt | stop — not in this job |

### 3.5 Studio verification (the only admissible binding proof)

Per `studio-evidence-sheet.md`: for each of 24 cards open the card view in Studio, screenshot, and record **the card
id + the asset id the running game reports**; the pictured portrait must be the one the data names (data-file
assertions do not count). For each of 8 cues: trigger it in the real flow and record the log line + a listening
verdict. A deliberately wrong binding must fail the check (negative control), proving the check can fail. Done =
24/24 screenshots + 8/8 trigger records, each naming id, asset id, screenshot hash, log line, Studio version.
Cues also need a human listening verdict — the current record still says "listening NOT PERFORMED"; that stays true
until it runs.

## 4 · Audio-side current state (authorised this batch)

`tools/upload_audio.py` is written and committed: branch `prep/audio-upload-20261101`, commit `e7f885c` (repo
`~/Work/projects/untitled-anime-card-game`). It implements the D10 spec: local validation first (extension/content
type, **measured** WAV/FLAC duration, ≤420 s, ≤20 MB, sha256, duplicate/robo-id/unresolved-attempt refusals), the
same Assets API request shape `publish.py` already uses, ambiguity journalled and **never retried**, moderation
recorded exactly as returned, and a `--bind` step for APPROVED receipts.

Dry-run proof (zero network, key never read):

```
$ python3 tools/upload_audio.py
DRY RUN: 8 audio upload(s); no network requests are made and the key is not read.
target.json: BLOCKED (live only): ops/release/target.json is absent — U0 (the owner selects the target experience) is the precondition
POST https://apis.roblox.com/assets/v1/assets (x-api-key: <redacted>) cue=contact_normal name=uacg-cue-contact_normal file=assets/audio/contact_tap_v1.wav content-type=audio/wav bytes=352878 sha256=74d7938b8af960d3 duration=2.0s (measured)
… (8 lines, one per cue; menu 45.0 s FLAC, battle 40.0 s FLAC) …
QUOTA: audio is non-updatable; if this account is not ID-verified only 10 audio uploads/month are allowed. Treat each upload as final; there is no retry path and no update path.
```

Candidate bytes unaffected (proof: tree `43574a5` + the tool rebuilds to `55cef5d3…`/520,846 B twice). Not run live;
`--live` refuses without key + target, exactly as intended.

## 5 · Inventory (as of 2026-11-01, verified this batch)

| Item | Value |
|---|---|
| Portraits on disk | 24 PNG, 740×1024, 8-bit RGB; ids/files/hashes in `d3-cast-table-20261101.json`; 24/24 manifest-hash match (D7, 31 Oct) |
| Bound art ids | **0/24** — `ArtIds.luau` all `""` |
| Audio cues | 8, all `roblox_id: null`; files in `assets/audio/` (4×2 s WAV, 2×1 s WAV, 45 s FLAC, 40 s FLAC); all unique sha256 |
| Target | `ops/release/target.json` absent; `ReleaseConfig` universe/place = 0 |
| Rights | USER-ATTESTED, not cleared — keep that wording |
| Publication | not authorised — nothing public, no store, no money |

## 6 · Done-condition for this prep (when the window opens)

U0 + key + authorisation → U1..U4 uploads with per-asset records → evidence files + `apply_art_ids.py` binds 24/24 →
rebuild reproduces **new** bytes → per-card screenshots + per-cue trigger records in Studio → legacy-profile safety
proof by test (definition already ruled: everything except `UACG_DEV_v1` / `uacg-dev-probe`) → test set green →
playtest package + one-paragraph note. Nothing less counts; a data-file assertion alone is explicitly inadmissible.
