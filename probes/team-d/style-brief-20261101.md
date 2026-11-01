# Style brief — the 24 portraits and 8 cues (derived from the game's own records)
Prepared 2026-11-01 by Team D from `docs/specs/spec-untitled-anime-card-game/roster.md`, `docs/VISUAL-DIRECTION-20260926.md`, `src/shared/content/Cards.luau`, `assets/images/manifest.json`, `docs/ART-PROVENANCE-CURRENT.md` and `assets/audio/manifest.json`. **This brief invents no cast**: the franchise list, per-franchise characters and the 24 ids already exist in the records. No image was generated; binding/verification of the existing 24 is technical work (Lead ruling D13).

## 0 · Identity rules (binding, from Cayleb)
- The game proceeds **UNTITLED with a fresh identity**. Never name or brand it after an earlier project.
- It must **never read, write or migrate legacy profile data** (absolute; any design needing legacy data is rejected).
- Discarded identities found in the repo (e.g. `build/helikon-candidate/`) are evidence to explain, never a name to adopt.
- Rights stay **USER-ATTESTED, not independently cleared** — keep that wording on every record and preview.

## 1 · World and card treatment (from `docs/VISUAL-DIRECTION-20260926.md`)
- Reference: a bright, open-air **Skybridge** battle scene; two upright **flat** cards meeting at centre; small player/rival health panels above; an ordered four-card reserve strip with Auto, speed, Skip and settings below. Dark and Light variants exist.
- **Cards stay flat cards.** The card face is **monochrome**; family colour lives in the world/UI, never on the card face. The previous family-coloured geometric treatment and the centred "Duel Archive" shell are superseded.
- Unbound art shows an explicit **ART PENDING** state — never a guessed placeholder portrait.

## 2 · Portrait brief (all 24)
- Format **740 × 1024** portrait crop — the existing set's exact size, kept so the card view's aspect does not change (a different aspect is a UI change, not an art change). PNG RGB, 8-bit.
- **One figure per portrait**, single canonical form per the roster's per-character instruction (Bleach: one Soul Reaper-era costume; Naruto: one Shippuden-era set; Dragon Ball: fixed Z-era base forms; Berserk: Guts Black Swordsman, Casca Golden Age, Serpico one verified later-era outfit; Vinland Saga: Thorfinn one first edition; Vagabond: manga pipeline, unverified colours labelled proposed; MHA: one hero-costume revision each; Black Clover: one early-series base outfit each). Transformations are separate editions, never merged into one portrait.
- Read as a **flat card illustration, not a poster**: centred figure, quiet background that survives the card's monochrome treatment, no baked text, no series logo, no watermark, no UI furniture.
- Consistency is enforced across all 24 by **one treatment pass**: same crop, same lighting key, same contrast envelope, same line weight, same background value range. A portrait that needs a different treatment to read is **rejected, not special-cased**.
- Provenance per image is mandatory (roster's own image-job contract): canonical character id, series, form id, reference-set version, display alias, reference path + hash + conditioning role, model/graph revision hashes, prompt/seed/sampler/steps/CFG, output purpose and crop, rights status, licence snapshot, output hash, validation results, rejection reasons.
- **Display alias is separate from the canonical name**: the card shows the alias; the data names the canonical id. The binding test compares *the id the card shows* to *the id the data names* — not a pretty label.

## 3 · The 24 ids and the portrait each one names (the cast, as the records define it)
The id abbreviation encodes the franchise; the data rows carry the canonical/display names, and the file is bound 1:1 by card id. Character-level names come from the roster (above); the data layer deliberately does not repeat them, so the character↔portrait confirmation stays an eye check in Studio, card by card.

| # | Card id | Franchise | Canonical (data) | Display alias | Portrait file | sha256 (first 16) | Bound? |
|---|---|---|---|---|---|---|---|
| 1 | `BL-ICHIGO-01` | Bleach | Substitute Soul Reaper — Blade Striker | Blade Striker | `assets/images/BL-ICHIGO-01.png` | `6a788b17a6ed366d` | no (ArtId `""`) |
| 2 | `NR-NARUTO-01` | Naruto | Nine-Tailed Jinchuriki — Spirited Ninja | Spirited Ninja | `assets/images/NR-NARUTO-01.png` | `8b8a93da94dcd952` | no (ArtId `""`) |
| 3 | `DB-GOKU-01` | Dragon Ball | Spiky-Haired Saiyan Warrior — Base Form | Saiyan Warrior | `assets/images/DB-GOKU-01.png` | `1b902cf89fd3206a` | no (ArtId `""`) |
| 4 | `BK-GUTS-01` | Berserk | Wandering Black Swordsman — Dragonslayer | Black Swordsman | `assets/images/BK-GUTS-01.png` | `877902fe39889b85` | no (ArtId `""`) |
| 5 | `VS-THORFINN-01` | Vinland Saga | Pacifist Ex-Warrior — Dual Daggers | Dual Daggers | `assets/images/VS-THORFINN-01.png` | `a9d0d3bc91214a45` | no (ArtId `""`) |
| 6 | `VG-MUSASHI-01` | Vagabond | Wandering Swordsman — Two-Sword Style | Two-Sword Saint | `assets/images/VG-MUSASHI-01.png` | `a9e887b42d5196fd` | no (ArtId `""`) |
| 7 | `MHA-DEKU-01` | My Hero Academia | One-For-All Successor — Smash Support | Smash Successor | `assets/images/MHA-DEKU-01.png` | `59e299705cd10aff` | no (ArtId `""`) |
| 8 | `BC-ASTA-01` | Black Clover | Magicless Knight — Anti-Magic Blades | Anti-Magic Knight | `assets/images/BC-ASTA-01.png` | `59451c09724ae3f6` | no (ArtId `""`) |
| 9 | `NR-KAKASHI-01` | Naruto | Copy-Ninja Mentor — Lightning Blade | Masked Mentor | `assets/images/NR-KAKASHI-01.png` | `29033080c0f2b735` | no (ArtId `""`) |
| 10 | `BL-BYAKUYA-01` | Bleach | Noble Clan Head — Thousand Blades | Thousand-Blade Noble | `assets/images/BL-BYAKUYA-01.png` | `e9c97ce755aa0208` | no (ArtId `""`) |
| 11 | `BL-FROST-01` | Bleach | Ice-Petal Reaper — Frost Captain | Frost Captain | `assets/images/BL-FROST-01.png` | `9700ab9092b17ceb` | no (ArtId `""`) |
| 12 | `NR-SHADOW-01` | Naruto | Avenger of the Fallen Clan — Rival Striker | Lone Avenger | `assets/images/NR-SHADOW-01.png` | `ec7040d01c7f8a0b` | no (ArtId `""`) |
| 13 | `DB-PRINCE-01` | Dragon Ball | Proud Saiyan Prince — Gallic Warrior | Saiyan Prince | `assets/images/DB-PRINCE-01.png` | `1c519892a3b8a0d9` | no (ArtId `""`) |
| 14 | `DB-NAMEK-01` | Dragon Ball | Green Namekian Mentor — Regenerating Sage | Namekian Sage | `assets/images/DB-NAMEK-01.png` | `2a11fd9baf82a134` | no (ArtId `""`) |
| 15 | `BK-FALCON-01` | Berserk | Falcon Band Captain — Hawk Commander | Hawk Commander | `assets/images/BK-FALCON-01.png` | `08e0a9369eaef18c` | no (ArtId `""`) |
| 16 | `BK-WIND-01` | Berserk | Feathered Wind Rogue — Rapier Duelist | Wind Duelist | `assets/images/BK-WIND-01.png` | `dc40e62a5d518504` | no (ArtId `""`) |
| 17 | `VS-BEAR-01` | Vinland Saga | Bear-Slayer Warlord — Axe Berserker | Axe Berserker | `assets/images/VS-BEAR-01.png` | `34e4f15c2494aa22` | no (ArtId `""`) |
| 18 | `VS-SNAKE-01` | Vinland Saga | Cunning Sea Captain — Serpent Strategist | Serpent Strategist | `assets/images/VS-SNAKE-01.png` | `0e30d7765732696d` | no (ArtId `""`) |
| 19 | `VG-RIVAL-01` | Vagabond | Deaf Swordsman Rival — Longblade Prodigy | Longblade Prodigy | `assets/images/VG-RIVAL-01.png` | `86d36a4fdbe9bcf6` | no (ArtId `""`) |
| 20 | `VG-HEAD-01` | Vagabond | Dojo Head Successor — Wooden-Sword Teacher | Dojo Teacher | `assets/images/VG-HEAD-01.png` | `afbabdf807c605e1` | no (ArtId `""`) |
| 21 | `MHA-BLAST-01` | My Hero Academia | Explosion Quirk Rival — Howitzer Striker | Howitzer Striker | `assets/images/MHA-BLAST-01.png` | `a2f92d4a46523df2` | no (ArtId `""`) |
| 22 | `MHA-FROSTBURN-01` | My Hero Academia | Half-Hot Half-Cold Prodigy — Twin Element | Twin-Element Prodigy | `assets/images/MHA-FROSTBURN-01.png` | `1278aa26d0b1b02d` | no (ArtId `""`) |
| 23 | `BC-WIND-01` | Black Clover | Four-Leaf Wind Mage — Gale Captain | Gale Mage | `assets/images/BC-WIND-01.png` | `0109dd689df37c97` | no (ArtId `""`) |
| 24 | `BC-SEA-01` | Black Clover | Royal Sea Mage — Water Barrier | Sea Barrier | `assets/images/BC-SEA-01.png` | `5d336d1bf3a2968c` | no (ArtId `""`) |

## 4 · Audio brief (the 8 finals)
- All original; SFX **dry and isolated** (no speech, no music baked in); UI/contact cues short (1–2 s); music 40–45 s and loop-verified by test; contact cues fire at the actual visual contact.
- Separate saved Music/SFX/UI volume buses; manifest per asset with the Roblox id only when real (null while pending).
- Cue map (purpose → local file): contact_normal → contact_tap_v1.wav; contact_light → contact_tap_light_v1.wav; contact_heavy → contact_thud_heavy_v1.wav; contact_critical → contact_crack_critical_v1.wav; card_move → card_slide_whoosh_v1.wav; ui_click → ui_click_v1.wav; menu_music → menu_theme_v1.flac; battle_music → battle_theme_v1.flac.
- Every cue still carries "listening NOT PERFORMED, Studio NOT RUN" — the finals must beat the engine-bundled stand-ins in a listening check before they are called approved.

## 5 · The binding identity rule (what the Studio check must prove)
For each card: the picture the card shows must be the picture whose file is named for that same card id in `assets/images/manifest.json`, and the id the running card reports must equal the id whose `ArtIds.luau` row was bound. For each cue: the sound that fires on the mapped trigger must be the uploaded final for that purpose. A data-file assertion alone is **explicitly not admissible** (Job 17 verification clause).
