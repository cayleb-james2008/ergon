# Studio evidence capture sheet — per binding and per audio trigger

For the card game's proof round (Job 17 item 1/2) and reusable for the street game's native cells.
**Rule this sheet enforces:** a binding asserted from the data files alone does **not** count. Every row must be
backed by a capture from a running Studio, on a named build, with the exact bytes preserved.

## 0 · Session header (fill once per Studio opening)

| Field | Value |
|---|---|
| Date / time (ET + UTC) | |
| Machine | `outis-desktop` (Vinegar flatpak 1.9.4) |
| Studio version (from Studio's UI) | `0.745.0.7451137` ⟨confirm in-window⟩ |
| Signed-in account (from Studio's UI) | `Chrono_H3X` (1170918805) ⟨confirm⟩ |
| Place opened | `build/UntitledAnimeCardGame.rbxlx` |
| Build sha256 (before opening) | `55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9` ⟨or the new bound build⟩ |
| ArtIds state in that build | 24/24 empty ⟨or 24/24 bound after U5⟩ |
| Capture root | `docs/verification/<date>-binding/` |
| Who drove | agent ⟨name⟩ · who observed | |
| StudioMCP handshake file | `board/probes/team-d/mcp-handshake-<stamp>.txt` |

**Naming convention for every capture:** `<type>_<card_id|purpose>_<build8>.png` / `.log` / `.txt`
(e.g. `card_BC-ASTA-01_55cef5d3.png`, `cue_contact_light_55cef5d3.log`). No spaces, no re-use across builds.

## 1 · Per-card binding (24 rows — one row per ArtId / card)

For each card, in the order of `ArtIds.luau`:

1. Open the deck/garage view that shows the card (the view that draws the card face).
2. Screenshot **the card face** (crop to the card, so the portrait fills the frame).
3. Read back, from the **running game** (not from the file), the card id and the asset id the game resolved.
4. Record the pair. If the id resolves to `""`, the card shows the ART PENDING state — record that honestly as
   "unbound", do not skip the row.

| # | card_id | data-named asset id (read from running game) | portrait visibly correct? (Y/N + what you see) | screenshot file | sha256 of screenshot | note |
|---|---|---|---|---|---|---|
| 1 | `BC-ASTA-01` | | | | | |
| 2 | `BC-SEA-01` | | | | | |
| 3 | `BC-WIND-01` | | | | | |
| 4 | `BK-FALCON-01` | | | | | |
| 5 | `BK-GUTS-01` | | | | | |
| 6 | `BK-WIND-01` | | | | | |
| 7 | `BL-BYAKUYA-01` | | | | | |
| 8 | `BL-FROST-01` | | | | | |
| 9 | `BL-ICHIGO-01` | | | | | |
| 10 | `DB-GOKU-01` | | | | | |
| 11 | `DB-NAMEK-01` | | | | | |
| 12 | `DB-PRINCE-01` | | | | | |
| 13 | `MHA-BLAST-01` | | | | | |
| 14 | `MHA-DEKU-01` | | | | | |
| 15 | `MHA-FROSTBURN-01` | | | | | |
| 16 | `NR-KAKASHI-01` | | | | | |
| 17 | `NR-NARUTO-01` | | | | | |
| 18 | `NR-SHADOW-01` | | | | | |
| 19 | `VG-HEAD-01` | | | | | |
| 20 | `VG-MUSASHI-01` | | | | | |
| 21 | `VG-RIVAL-01` | | | | | |
| 22 | `VS-BEAR-01` | | | | | |
| 23 | `VS-SNAKE-01` | | | | | |
| 24 | `VS-THORFINN-01` | | | | | |

**Pass condition:** 24/24 rows filled, every `portrait visibly correct?` = Y, every screenshot hash recorded, and
every data-named asset id non-empty and matching the upload registry. One "N" = the round is failed and named.
The character-vs-portrait judgement is exactly the human/eye step the file-level D7 check cannot do — say what you see
in words ("the masked mentor with a scar and the headband", not "looks right").

## 2 · Per audio cue (8 rows — one per cue, trigger-verified)

For each cue: trigger it in the **real flow** (not a test harness call), listen with headphones on that machine, and
capture the play log line + a waveform-ish confirmation if the tool provides one.

| # | purpose | trigger used (exact in-game action) | played on trigger? (Y/N) | audible / correct? (Y/N + one sentence) | log line | capture file | note |
|---|---|---|---|---|---|---|---|
| 1 | `contact_normal` | play a normal contact card | | | | | |
| 2 | `contact_light` | play a light contact card | | | | | |
| 3 | `contact_heavy` | play a heavy contact card | | | | | |
| 4 | `contact_critical` | land a critical strike | | | | | |
| 5 | `card_move` | draw / move a card | | | | | |
| 6 | `ui_click` | press a UI button | | | | | |
| 7 | `menu_music` | enter the menu | | | | | |
| 8 | `battle_music` | start a battle | | | | | |

**Pass condition:** 8/8 fired on the right trigger AND a listening verdict recorded. "Listening NOT PERFORMED" stays
in the record if nobody listened — it is not rounded up. The five `menu/loop` questions: does the music loop without a
seam; does a repeated SFX fatigue; do two contacts overlapping sound like a mess; does the mix sit under the voice/UI;
does a scene change cut a cue cleanly.

## 3 · Supporting captures (one per round, not per item)

| Capture | How | Why it is needed |
|---|---|---|
| The running build's own report | the game's Studio self-report (it names the build hash it executed) | ties every screenshot to a hash |
| `ArtIds` readback from the running game | the game's own resolved table | proves the data the card used |
| Place hash before/after | `sha256sum build/*.rbxlx` at open and close | proves nothing changed mid-round |
| Studio log for the session | the `…/Roblox/logs/0.745…log` file, copied into the capture root | the raw record if a claim is disputed |
| Screenshot of the artifact/moderation rows | the creator dashboard rows for the 32 assets | links asset id ↔ moderation PASS |
| Any failure frame | full screenshot + the log lines around it | failures are kept, never re-run to pass |

## 4 · Close-out (must happen before the round can be called green)

- [ ] Session header complete, build hash recorded at open **and** close (identical).
- [ ] 24/24 card rows filled with screenshot files + hashes.
- [ ] 8/8 cue rows filled with triggers + listening verdict.
- [ ] Negative control: one deliberately wrong binding (e.g. point one card at another card's asset id in a scratch
      build) is shown to **fail** the visual check — proves the check can fail.
- [ ] Nothing asserted from a JSON file alone; every row has a capture path that opens.
- [ ] `verify-clean` (street repo) / place-hash check (card repo) exit 0.
- [ ] Round written to `docs/verification/<date>-binding/README.md` with the totals, the failures, and the honest
      limits ("what this does not prove").

**What this sheet does not prove:** that the art is legal to publish (rights stay USER-ATTESTED), that the game is
fun, that moderation will accept it on a re-review, or that the street game's cells pass — those are separate records.
