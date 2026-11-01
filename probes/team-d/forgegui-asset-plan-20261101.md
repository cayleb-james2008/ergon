# ForgeGUI asset plan — bounded batch proposal (prepared, NOT executed)

Status: **plan only; 0 credits spent; no generation, no `library_search` until the Lead says the MCP is live.**
Authority: Cayleb's envelope **~100,000 ForgeGUI credits** (prepaid, no purchase). Free `library_search` first.
Key handling: Cayleb creates the key; it lands in a 0600 file; agents use MCP tools only, never see/log/ask for the
value. No publication via ForgeGUI. Assets return as files/URLs and enter Roblox only through the Studio MCP /
Open Cloud routes we already control.

## 1 · What the current jobs actually need (from the projects' own records)

| Job | Assets named by the records | Current state | New generation needed? |
|---|---|---|---|
| **17 card game** — portraits | 24 PNGs, 740×1024, ids in `ArtIds.luau`/`manifest.json` | **24/24 exist, file-verified**, 0 bound (binding/uploads are the open work) | **No** — Job 17's done-condition is "24/24 portraits bound and seen correct", not "produced"; D13 ruled no new art is needed for binding/verification |
| **17 card game** — audio | 8 finals (6 SFX WAV + 2 music FLAC) | **8/8 exist** as local finals; `roblox_id` null (upload pending) | **No** |
| **16 street game** — assets | store icon 512², 4 thumbnails 1920×1080, in-game art generated from the source tree | icon + 4 thumbs exist and pass `store_check` (3/3 PASS 09:36 ET); 27 runtime audio files exist | **No** for the playtest gate; look defects named in the records are code/flag work (road flecks, cyan sheen, dark mid-distance, blank overheads), not missing assets |
| **UI/GUI** | both games ship their own UI code; no missing screens named | — | **No** |

**Conclusion: ForgeGUI is NOT on the critical path for Jobs 16/17 as defined.** Nothing in either job's
done-condition needs a new generated asset; the open work is binding/uploading existing art, audio upload, and the
Studio test set. Generating more art now would spend credits against no acceptance criterion.

## 2 · When generation would be justified (candidate list, not authorised)

Only after the Studio test set/paytest produces a concrete missing-asset finding, e.g.:
1. A card portrait that the per-card Studio check proves wrong/unreadable **and** cannot be re-cropped from the
   existing masters (the 24 files are the approved set — re-shooting one is a *new* art decision).
2. A device-specific UI asset that the phone/tablet cells prove unreadable (needs evidence first).
3. Store-page thumbnail variants if Cayleb asks for a new marketing set (not part of these jobs).

Free-library-first rule for each candidate: `library_search` → reuse/adjust (0 credits) → only if absent, generate.

## 3 · Bounded-run protocol (to use only on the Lead's go)

1. One batch, one list. For each asset: purpose, target file, style constraints (from
   `style-brief-20261101.md` for the card game), and the acceptance check that will verify it.
2. `library_search` every item first; record hits/credits (free) in the ledger.
3. Generate in one authorised run with **per-asset job IDs**; record credits per job + running total in the ledger
   below; stop at the envelope (never exceeded; warn at 80%).
4. **No automatic retries** on `outcome_unknown`: check the existing job's status first, then decide with the Lead.
5. Outputs land as files/URLs; import only through Studio MCP (`insert_asset`/`upload_image`) or the existing
   Open Cloud binding tooling; no publication; moderation results recorded per asset.
6. Every asset gets a receipt row (job id, credits, model/settings if returned, source file/hash, target, status).

## 4 · Credit ledger (empty until live)

| # | Asset | Route (library/generate) | Job ID | Credits | Running total | Status |
|---|---|---|---|---|---|---|
| — | free `library_search` inventory (6 queries: anime card portrait, card frame UI, racing HUD icon, neon city background, garage/showroom UI, store icon) | library_search (free) | none (no generation job) | **0** | **0 / 100,000** | complete 2026-11-01 12:43 ET — all six returned `items:[]`, `search_status:complete` (no published match); see `forgegui-library-inventory-20261101.md` |
| — | (no generation requested) | — | — | 0 | **0 / 100,000** | generation held at 0 until a real gap is proven and a bounded costed run is authorised |

## 5 · Recommendation

Hold all ForgeGUI spend at **0** for now. When the MCP is live, the only zero-cost action worth doing is a
`library_search` inventory against the two games' named asset needs (24 portrait style refs, UI icons, store art) so
the fleet knows what is available **before** any credit is spent. Ask the Lead to authorise that free inventory as
the first ForgeGUI batch; generation waits for an evidence-based missing-asset list.
