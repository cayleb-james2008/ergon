# Upload plan — 24 portraits + 8 audio finals (PLAN ONLY; nothing uploaded)

Prepared 2026-10-31 by `roblox-team` (Team D). **Not executed.** Blocked on: Cayleb's upload authorisation (Q6),
`ops/release/target.json` (absent), Roblox moderation, and — for the audio — the absence of any project upload tool.

## 0 · What the project's own tooling demands (read before anything)

`ops/release/publish.py` is the upload path (`upload-art`), and it fails closed by design. Its binding gate
(`validate_mapping_receipt`) requires, per card, **all** of:

| Requirement | Where it lives | Status today |
|---|---|---|
| `schema` = the mapping-receipt schema | receipt JSON | to be produced per card |
| `card_id` exact | receipt | 24 ids known (`ArtIds.luau`) |
| `source_sha256` = the **current approved image** bytes | receipt vs disk | 24/24 known (D7 PASS) |
| `asset_id` matching `\d{6,19}` | returned by Roblox | none yet |
| `target` = `target.json`'s creator/universe/place, **exactly** | `ops/release/target.json` | **ABSENT** — only `target.example.json` exists |
| `franchise` present in `Cards.luau` | `Cards.luau` | present for all 24 |
| `upload_receipt` evidence (`uacg-upload-receipt/v1`, status `COMPLETE`, `returned_on`) | attached evidence | none |
| `moderation` evidence (`uacg-moderation-receipt/v1`, verdict `PASS`, `moderated_on`, fresh within window) | attached evidence | none |

Also enforced: `require_current_build()` (the build receipt must match the bytes being used — it does:
`55cef5d3…`), duplicate upload refused, and **"a prior upload for this card is unresolved; automatic re-upload is
refused"** — so an interrupted attempt must be resolved by hand, not retried blindly.
`tools/apply_art_ids.py --receipt <JSON>` performs the binding and refuses "bare copied IDs".

**Audio has no route at all.** `grep -i "upload.*audio|sound" ops/release/publish.py` → nothing. So the 8 finals
cannot be uploaded by the project's own tooling; the options are (a) Studio Asset Manager manual import (in a window,
by the owner/agent), (b) a small new upload script (code work, later, needs the same key/target), or (c) keep the
engine-bundled stand-ins and defer finals. **Recommendation: (a) for the first pass, (b) only if the card game's
binding pass repeats.**

**Credential rule:** the Open Cloud key is never printed, copied, or committed. If the tool asks for a key that is not
already available to it, **stop — that is Cayleb's secure paste**, not an agent step.

## 1 · Order of work (each block is one sitting)

| Block | Items | Why this order |
|---|---|---|
| **U0** | Fill `ops/release/target.json` from the experience's own settings (universe, place, creator). Verify by reading it back through the tool's own `canonical_target`. | every upload and every binding is refused without it |
| **U1** | **8 audio finals** (small, no character-IP in the file names, easiest moderation) | fails fast, and the card game's cue map is independently blocked on them |
| **U2** | **8 portraits — franchise block 1** (Bleach + Naruto) | small blast radius if moderation behaves unexpectedly |
| **U3** | **8 portraits — franchise block 2** (Dragon Ball + Berserk) | same |
| **U4** | **8 portraits — franchise block 3** (Vinland Saga + Vagabond + MHA + Black Clover) | same; note this block mixes four franchises, so 12 if balanced by franchise: 3+3+3+3+3+3+3+3 (8 blocks) |
| **U5** | Rebuild + bind + verify | see `studio-evidence-sheet.md` |

Blocks of 2–3 franchises (6–9 images) are deliberate: Roblox moderation of character-lookalike art can reject a
whole batch's *review*, and a small block keeps the failure attributable.

**Per-image upload order inside a block:** the order of the `ArtIds.luau` table (alphabetical by card id) — it is the
order the registry and the receipt generator use, so a retry resumes predictably.

## 2 · Naming (moderation-aware, and it must be boring)

Uploads carry a name in the creator's asset list. **Do not put character or franchise names in an asset name** — that
invites IP review and tells Roblox's moderation nothing useful. Use a neutral, stable scheme that maps 1:1 to the id:

- portraits: `uacg-art-<card_id>` → e.g. `uacg-art-BC-ASTA-01`
- audio: `uacg-cue-<purpose>` → `uacg-cue-contact_light`, `uacg-cue-menu_music`
- description: `Untitled card game — internal art, not for public use. Source <card_id> / <sha256 first 12>.`

The card id already encodes franchise (`BC`, `BK`, `BL`, `DB`, `MHA`, `NR`, `VG`, `VS`), so the mapping stays exact
without spelling out a trademark in the asset list.

## 3 · Privacy / listing settings and what must be true

- The experience is a **local, unlisted candidate** — no universe is authorised yet (`target.json` absent). Uploads
  are made **to the creator account**, not published to a game page. **No publication of the experience.**
- Roblox image assets are usable by the uploading owner; they must **not** be made public/marketed, and the store
  listing must not be touched. **Record the asset's privacy/ownership state as returned**; if the API offers a
  visibility/“public” choice, choose the most restrictive allowed.
- Audio: uploaded audio is restricted to the owner's own experiences on Roblox's side; confirm what the response
  actually says and record it — do **not** claim the finals are usable by anyone else.
- Rights stay **USER-ATTESTED**, not cleared. The upload does not change that sentence; the plan's records must keep
  saying so.
- **No money**: uploads are not purchases; no Robux, no subscription, no paid asset. If any step asks for money,
  stop — it is outside the envelope ($0).

## 4 · What to record per upload (the receipt that makes binding legal)

One row per asset, in `docs/verification/<date>/upload-registry.json`, fields exactly as the tool's schemas need:

| Field | Meaning |
|---|---|
| `card_id` / `purpose` | which ArtId row (or cue) this fills |
| `source_sha256` | sha256 of the exact local file that was uploaded (must equal the approved image/manifest hash) |
| `source_file` | path as uploaded (`assets/images/<id>.png`, `assets/audio/<file>`) |
| `asset_id` | the 6–19 digit id Roblox returned |
| `upload_receipt` | `{schema: uacg-upload-receipt/v1, status: COMPLETE, returned_on: <UTC>}` + request id if returned |
| `moderation` | `{schema: uacg-moderation-receipt/v1, verdict: PASS, moderated_on: <UTC>, raw_status: <as returned>}` |
| `target` | creator/userId + universeId + placeId, copied from `target.json` (exact) |
| `franchise` | from `Cards.luau` (the tool cross-checks it) |
| `asset_name` | the neutral upload name used |
| `operator` | who pressed go (agent/human) + timestamp |

**A pending or failed moderation is recorded as pending/failed — never rounded up to PASS.** A blocked asset keeps its
row and blocks its card; the block is reported, not worked around.

## 5 · Failure handling

| Symptom | Action |
|---|---|
| `target.json` mismatch / absent | stop; U0 is the fix, not a code change |
| duplicate-id refusal | the card is already bound — verify, don't force |
| "prior upload unresolved" | resolve that one attempt (find its asset in the creator list); do **not** re-upload automatically |
| moderation not PASS | keep the row, mark the card blocked, report; do **not** re-upload the same bytes hoping for a pass |
| rate limit / 429 | back off, resume from the registry; uploads are idempotent by `card_id` |
| any request for a credential the tool does not already hold | **stop — Cayleb's secure paste** |

## 6 · Done-condition for this plan (when it is finally run)

24/24 portraits uploaded → moderation PASS recorded → `apply_art_ids.py --receipt` binds each → `ArtIds.luau` has 24
non-empty ids → rebuild reproduces **new** bytes → 8/8 audio finals uploaded/imported with ids recorded → then the
Studio evidence round (`studio-evidence-sheet.md`) proves each card shows the right picture and each cue fires on its
trigger. Nothing short of that counts.
