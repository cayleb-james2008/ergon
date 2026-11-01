# D10 — Audio import path (DESIGN ONLY; nothing uploaded)

Prepared 2026-10-31 by `roblox-team`. **Not integrated, not executed.** Answers one question plainly: **does a real
Open Cloud route for audio exist, or must it fall back to Studio Asset Manager import?**
Short answer: **A real API route exists** — `assetType: "Audio"` on the same Assets API the project already uses for
images — so the script is a sibling of `upload_image`, not new machinery. Asset Manager import stays as the manual
fallback.

## 0 · The evidence (read-only official docs, fetched 2026-10-31)

Source: <https://create.roblox.com/docs/cloud/open-cloud/usage-assets> (external page, treated as data).

> "The [Assets API] of Open Cloud allows you to upload and update assets with a single HTTP request rather than
> manually importing them to Studio."

Audio appears in the supported-types table with:

> Asset type **Audio** · Format **`.mp3` `.ogg` `.wav` `.flac`** · Content type **`audio/mpeg` `audio/ogg` `audio/wav`
> `audio/flac`** · Restrictions **"Up to 7 minutes of duration." / "Up to 100 uploads per month if you're ID-verified."
> / "Up to 10 total uploads per month if you aren't ID-verified." / "Not available for updating."**

Create-asset request shape (same endpoint the project uses for Decals):

```
POST https://apis.roblox.com/assets/v1/assets
header: x-api-key: <key>
multipart form:
  request     = {"assetType":"Audio","displayName":"…","description":"…",
                 "creationContext":{"creator":{"userId":"<id>" or "groupId":"<id>"}}}
  fileContent = <the audio bytes>; type=audio/wav (or audio/mpeg | audio/ogg | audio/flac)
→ response { "operationId": … }   (or path "operations/…")
GET https://apis.roblox.com/assets/v1/operations/{operationId}    (x-api-key)
→ { "done": true, "response": { "assetId":"…", "moderationResult": {"moderationState":"MODERATION_STATE_APPROVED"} } }
```

**The project already implements this exact pattern for images** — `ops/release/publish.py`:
`":700"` sets `{"assetType": "Decal", "displayName": card_id, …}`; `":721"` POSTs to `/assets/v1/assets`;
`":732"`–`":740"` poll `/assets/v1/operations/{id}` with a deadline and `MAX_POLL_ATTEMPTS`. So the audio script
reuses a proven code path; the only real differences are the `assetType`, the file's content type, and the
non-updatable restriction.

## 1 · What the 8 cues need against those limits

| cue | file | format | duration | ≤7 min | size ≤20 MB | updatable |
|---|---|---|---|---|---|---|
| contact_normal | `contact_tap_v1.wav` | wav | 2.0 s | yes | 353 KB | no (by type) |
| contact_light | `contact_tap_light_v1.wav` | wav | 2.0 s | yes | 353 KB | no |
| contact_heavy | `contact_thud_heavy_v1.wav` | wav | 2.0 s | yes | 353 KB | no |
| contact_critical | `contact_crack_critical_v1.wav` | wav | 2.0 s | yes | 353 KB | no |
| card_move | `card_slide_whoosh_v1.wav` | wav | 1.0 s | yes | 176 KB | no |
| ui_click | `ui_click_v1.wav` | wav | 1.0 s | yes | 176 KB | no |
| menu_music | `menu_theme_v1.flac` | flac | 45.0 s | yes | 1.7 MB | no |
| battle_music | `battle_theme_v1.flac` | flac | 40.0 s | yes | 4.3 MB | no |

**Quota is the binding constraint, not duration.** 8 uploads is one pass; **if the account is not ID-verified the
audio cap is 10 per month**, so a rejected/re-shot cue leaves at most one more try — and audio **cannot be updated**,
so a bad upload can only be replaced by a second upload that burns quota. Therefore: **validate everything locally
before the first network call**, and treat every upload as final.

## 2 · The script design (not written into the repo; this is the spec)

**File (proposed):** `tools/upload_audio.py` — mirrors `ops/release/publish.py`'s structure and reuses
`multipart_body`, `request_bytes`, `read_target`/`canonical_target`, `Blocked`.

**(a) Input.** `assets/audio/manifest.json` (8 rows, `roblox_id: null`). The script reads only rows whose
`roblox_id` is null or empty; a row with a real id is refused as a duplicate.

**(b) Pre-upload gate — local, no network, all must pass or nothing is sent.**
1. extension ∈ {`.mp3`, `.ogg`, `.wav`, `.flac`} → content type map (`audio/mpeg`, `audio/ogg`, `audio/wav`, `audio/flac`).
2. **actual** duration read from the file (ffprobe-free fallback: WAV header; for FLAC/MP3 use the manifest only if
   the tool can read it — otherwise record "duration from manifest, not re-measured" honestly) ≤ 420 s, and equal to
   the manifest `duration_s` within ±0.1 s.
3. size ≤ 20 MB; sha256 recorded (this is the identity later bound).
4. `roblox_id` is null (not already bound); no unresolved attempt for this purpose in the registry.
5. `ops/release/target.json` exists and validates (creator for `creationContext`). **Still a precondition** — the
   audio route does not remove U0.

**(c) Request.** Exactly the table in §0 with
`displayName = "uacg-cue-" + purpose`, `description = "Untitled card game — original cue <purpose>; source sha256 <12>"`,
creator from `target.json` (userId **or** groupId, exactly one — same rule as images).
No franchise/character words in the name (moderation hygiene, same rule as the image plan).

**(d) Idempotence and crash safety — copy the image path literally.** Before the POST, write
`upload_attempts[purpose] = {state: SUBMISSION_STARTED, source_sha256, target, started_utc}` into
`ops/release/audio-bindings.json`; after an ambiguous response → `AMBIGUOUS` and **never retry automatically** (audio
is non-updatable and quota-metered; a blind retry could burn both). This is the one design decision that must not be
softened.

**(e) Polling.** `GET /assets/v1/operations/{id}` until `done`, deadline 60 s, bounded attempts — identical to
`upload_image`. On `done`, record raw `assetId` and `moderationResult.moderationState`.

**(f) Moderation handling.** `MODERATION_STATE_APPROVED` → eligible to bind. `PENDING` → keep the attempt open and
poll again later (audio review can lag; do not call it passed). `REJECTED` → keep the row, mark the cue **blocked**,
report; **do not re-upload the same bytes** (quota + finality). Never round PENDING/REJECTED up.

**(g) Binding.** On APPROVED, write `roblox_id = "rbxassetid://<assetId>"` into `AudioCatalog.CUES[...]` for that
`purpose` — the stand-ins are exactly `roblox_id = "rbxasset://sounds/…"` in `src/client/AudioCatalog.luau`, so the
edit is one line per cue. Then `bash tools/build.sh`, then the Studio trigger check. (A generated
`src/shared/content/AudioIds.luau` + catalog read-through is the cleaner long-term shape; it is a bigger edit and
should be the Lead's call.)

**(h) Receipt per cue** (`uacg-audio-upload/v1`), mirroring the image receipt so a future binder behaves like
`apply_art_ids.py` (refuses bare ids):
`{schema, purpose, source_file, source_sha256, bytes, duration_s, target, asset_name, asset_id, upload_receipt{schema,status:COMPLETE,returned_on}, moderation{schema,verdict:PASS|PENDING|REJECTED,moderated_on,raw_state}, operator, utc}`

**(i) Dry-run.** `python3 tools/upload_audio.py` with no `--live` prints the 8 POST lines with
`x-api-key: <redacted>` and **makes no network request** — exactly the shape proven for images
(`board/probes/team-d/u9-upload-dryrun.txt`). `--live` requires the key + target and is Cayleb's go.

## 3 · Files/decisions that stay the Lead's or Cayleb's

- **U0 target** (`target.json`): still absent; the audio route needs a creator context. Cayleb picks/creates the
  unlisted experience (D12).
- **Credential:** the script never reads, prints or stores the key beyond passing it to the request header; if the key
  is not already available to the tool, that is Cayleb's secure paste.
- **Quota:** 8 audio uploads against a possible 10/month non-ID-verified cap — one shot. Say so to Cayleb before he
  authorises.

## 4 · Fallback (still valid, no script)

**Studio Asset Manager import**: in the window, import the 8 files from `assets/audio/`, copy each asset id, and
record the same receipt rows by hand. Slower, manual, but no new code and no key handling beyond Studio's own sign-in.
Recommendation: **try the script path first** (it is a sibling of code already in the repo and is dry-runnable), keep
Asset Manager for the case where the API refuses or the window is the only chance.

**Verdict for the Lead:** a real Open Cloud audio route exists; it is the same endpoint the project already polls for
Decals; the only new risks are the non-updatable restriction and the small monthly quota, both handled by
local-validation-first and no-automatic-retry. Nothing here has been uploaded or written into the repo.
