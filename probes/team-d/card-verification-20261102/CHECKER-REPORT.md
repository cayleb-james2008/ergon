# CHECKER REPORT: GAME-DESIGN-SPEC (985 lines) vs candidate 43574a5 (place 55cef5d3)

Role: independent checker. Read-only on /home/cayleb/Work/projects/untitled-anime-card-game (nothing in it was modified by me; `git status` there shows only an untracked `.team-d-wsh/` that I did not create). All runs used the export at `.../scratchpad/cg-candidate/`. No Studio, network, uploads, commits or Delta work.
Scripts and raw outputs are in this folder (`harness.lua/.out`, `extras.*`, `sweep.*`, `rounds.*`, `fourcard.*`, `claims.*`, `camera.py`, `time15.lua`, `dump.lua/.out`, `t-*.out`).

## Verdict: PASS-WITH-FINDINGS
The freeze describes the candidate accurately. Every number in the data tables, config block, balance table and quoted figures reproduced exactly. There is no HIGH defect. 19 minor defects follow; the most decision-relevant are D1, D2, D3, D4.

## 1. Identity
| Item | Result | Evidence |
|---|---|---|
| Build hash | PASS | `sha256sum build/UntitledAnimeCardGame.rbxlx` = 55cef5d3ea71173de62b7c1070543d566e6f26674eb64083833ea5590466d4c9 |
| Source = place | PASS | 55 of 56 `src/**/*.luau` files appear byte-for-byte inside the rbxlx. The exception is `src/shared/offers/OfferConfig.luau`, which is not mapped in `default.project.json` and is required only by `tests/offers/*` (see D8) |
| Commit relation | PASS (note) | `git diff cd91a88 43574a5 -- src` is empty, so "code at cd91a88" == the candidate's src. cd91a88 is NOT an ancestor of 43574a5, and the candidate lacks `tools/bind_art.py` and `tools/upload_audio.py` (D9) |
| 24 cards, 8 franchises x 3 | PASS | `claims.out`: 24 editions, 8 franchises x 3; rarity counts C4 U5 R7 E5 L2 M1; stat bounds all in range; Cards.luau:32-225 |
| 3 worlds / 12 routes, prerequisites, chapters | PASS | `Bootstrap.luau:50-82` (`previousRouteId` advances only in the encounter loop, so bosses are side branches and the next world's enc-1 requires enc-3). All 12 `chapter/enemy/requiresClear` triples in spec 3.0b equal the code (`dump.out`). Caps 6/10/15/21/30 at `Config.luau:97`; only chapters 1-3 exist |
| 13 screens | PASS | `Screens.luau:33-38`, ids and titles match spec 4.1. One primary label differs (D6) |
| STARTER_IDS | PASS | `Config.luau:149` |
| 3.0 / 3.0b tables | PASS, 0 mismatches | Independent diff of all 24 cards x 13 fields, 11 skill nodes, 12 routes, 16 rate limits, 8 ranks, about 45 Config scalars, DEFAULT_VOLUMES, weights, caps, Statuses constants (2/3/2/1), Resolver 50/1000, Validate team 1-6. The spec's own `check_data.py` also prints "24 cards checked; mismatches: 0". Only non-code value: `versions.rules = "provisional-2"` (Delta-1 value, D13) |
| Trained-stat formula | PASS (note) | Code uses float math, `Battles.luau:46-48`. All 720 combinations (24 cards x eff 1-15 x hp/atk) equal the spec's integer-bp formula, 0 mismatches. Spirited Ninja eff 6/10/15 = 75/11, 89/13, 106/16; L1 +6% = 61/9; L10 +6% = 94/14, as stated |

## 2. [AB] and as-built claims
Checked 85 as-built claims, covering everything in 3.1-3.12 and samples from sections 1, 2, 4, 5, 6 and 7. 6 are FALSE or stale (D1-D4 and others below). The rest are PASS. The table lists them by claim, with evidence.

| # | Claim (spec section) | Verdict | Evidence |
|---|---|---|---|
| 1 | Roles are labels only (3.1) | PASS | `role` is read by no combat or progression code |
| 2 | Per-card odds 15 / 5 / 1.43 / 0.9 / 0.225 / 0.05 % (3.1) | PASS | 6000/2500/1000/450/45/5 split over 4/5/7/5/2/1 |
| 3 | First roll prefers unowned while < 4 distinct owned; Common is then Dojo Teacher (2.1, 3.1) | PASS | `RollService.luau:53-93` (preferMissing = distinct < DECK_MAX), `Rolls.luau:73-86` |
| 4 | Rates, pity fires at the 1800th draw (3.2) | PASS | `Rolls.luau:35` (pityFails >= 1799) |
| 5 | P(reach pity)=40.67 %, E[draws]=1187, 3.0 h (3.2) | PASS | computed 0.40668, 1187.04, 10800 s |
| 6 | Pity shared by free rolls and Field Packs; resets on any Mythic (3.2) | PASS | one `applyCardRoll`, `RollService.luau:53`; `Rolls.luau:108` |
| 7 | RNG drawn outside the commit retry (3.2) | PASS | `RollService.luau:137-139`; `buyEarnedPack` same |
| 8 | ~143 rolls / 14 min to collect 4 of 5 Epics (2.2) | PASS | 6.4167 Epic draws / 0.045 = 142.6 rolls = 14.3 min |
| 9 | Duel order, damage, shield-after-defence, entry on replacements only, ties, 50-round tiebreak (3.3) | PASS | `Resolver.luau:143-361`, `Statuses.luau:84-101` |
| 10 | Burn is dead content (3.3 item 7) | PASS | `applyBurn` has no caller; `Statuses.luau:56` |
| 11 | Enemy decks are one card; stun matters only for the Mythic and Masked Mentor on the player side (3.3) | PASS | `Bootstrap.luau:74,78` (`enemyIds = { id }`); only Kakashi and Byakuya have `entry = stun` |
| 12 | Route table, names, caps (3.4) | PASS | `World.SCENES` names; `Progression.luau:84-97` |
| 13 | Replays pay every win, no decay (3.4) | PASS | `BattleService.luau:137,245-248` |
| 14 | 240 coins / 480 XP first full clear; Field Pack 10 wins or 2 boss wins (3.5) | PASS | 9x10+3x50; 9x20+3x100 |
| 15 | Field Pack has no cooldown, no purchase limit, blocked while a duel or reward is pending (3.5) | PASS | `RollService.luau:165-172` (earnedPackBlockReason), `buyEarnedPack` never touches `cooldownReadyAt` |
| 16 | Cumulative XP L6=375, L10=1125, L15=2625; +25 win / +5 loss or tie to deck cards (3.6) | PASS | computed; `BattleService.luau:252-269` (tie falls into the CARD_XP_LOSS branch) |
| 17 | Skill table, ceilings, 14 points to all three ceilings, 16 usefully spendable (3.6) | PASS | atk nodes 1200 bp vs 600; 4+4+8 ranks, flex route 14 |
| 18 | Rank rule: runNumber >= N, boss win needed, claim or auto at rebirth (3.6) | PASS | `SkillService.luau:40-53`; `Rebirth.luau:187-205`. "One rank per run" holds in normal play because rebirth auto-claims; a player who never rebirths could in theory catch up several ranks in one later run (INFO) |
| 19 | Run level not shown; account level has no XP source (3.6, V12) | PASS | no client reader of `level`; `accountXp` is written nowhere outside Profile and Rebirth |
| 20 | "Effective level ... shown to the player" (3.6) | FALSE as-built (D4) | no client code shows any level or XP |
| 21 | Rebirth kept/reset/granted/keepsake rules (3.7) | PASS | `Rebirth.luau:49-232` |
| 22 | Away formula, rollback = 0, one claim per session, Run XP only (3.8) | PASS | `AwayProgress.luau:31-45`; `AwayProgressService.luau:92-123` |
| 23 | "profile screen shows the pending amount" (3.8) | FALSE (D5) | `Actions.luau:344-372` writes it to the Worlds screen |
| 24 | Server owns outcomes; v2 identity; lease 300 s; epoch fence (3.10) | PASS | `Store.luau:65-86,416-450`, `Remotes.luau:333-345` |
| 25 | Lease: `Store.renew` has no caller; owner check ignores expiry (V9) | PASS | `Store.luau:323`, `Store.luau:90-95` (`owns` compares epoch only); only call is `tests/progression/test_repair.luau:135` |
| 26 | Scope UACG_DEV_v1; production refuses unless ids non-zero and matching; key `profile:<uid>` (3.10) | PASS | `Config.luau:224`, `Bootstrap.luau:340-352`, `ReleaseConfig.luau:5-8`, `DataStoreAdapter.luau:46` |
| 27 | Studio probe `uacg-dev-probe`, key `probe:<time>`, `Bootstrap.luau:302`; memory lane 275-318 (V18) | PASS | `Bootstrap.luau:296-318` |
| 28 | No OrderedDataStore or MemoryStore (3.10) | PASS | grep |
| 29 | Analytics events emitted outside Studio (3.10) | PASS | `Bootstrap.luau:195-200,213-223` |
| 30 | No test records DataStore names (3.10) | PASS | only a scope-agnostic fake at `tests/progression/test_repair.luau:96` |
| 31 | `docs/persistence.md` calls the production path UACG_DEV_v1 (7) | PASS | `docs/persistence.md:122` |
| 32 | OfferConfig: 3 dormant rows, all disabled; Supporter pass id 0; checkout fails closed (3.11) | PASS (note D8) | `OfferConfig.luau:129-175`, `ReleaseConfig.luau:8`, `SupporterPass.luau:25-28` |
| 33 | V1 entry stun never skips a strike | PASS | scratch duel, `claims.out`: entry_effect stun at turn 1, then `status_expired` at turn 2 and the stunned card still strikes; 0 `stun-skip` events. Cause: `Statuses.luau:40-46` decays it in `tickStartOfTurn` before any strike (`Resolver.luau:157`) |
| 34 | V2 skill bonuses and XP bonus do nothing | PASS | see owner finding (d) |
| 35 | V3 world hidden, dark arena for all three worlds | PASS | `BattlePresenter.luau:19-45,58-78,231`; Skybridge = 33 Block parts + 2 PointLights (`World.luau:23-68`) |
| 36 | V4 fall and replacement are instant swaps; `defeatKeyframes` unused | PASS | `BattlePresenter.luau:~392-420`; only caller is `tests/presentation/test_geometry.luau:65`. The 0.40 s ko/replacement timeline slots do exist (`Timeline.luau:~165-170`) |
| 37 | V5 auto-roll toggle | PASS | `State.luau:40` (client default true), `Actions.luau:1128` (sends `not prefs.autoRoll`), `Profile.luau` default false; nothing writes `prefs.autoRoll` from the server, so the toggle always sends `false` |
| 38 | V6 three accents; server rejects absent accent | PASS | `State.luau:36,245`; `Profile.luau` "#A78BFA"; `Actions.luau:910-911` "#62E6AD"; `Progression.luau:180` |
| 39 | V7 rail width computed once | PASS | single read `UiView.luau:~639`; no ViewportSize hook anywhere |
| 40 | V8 ambience slider with no ambience sound | PASS | `Screens.luau:193`; `AudioCatalog` has no ambience cue |
| 41 | V10 `status_expired` always "burn"; damage event names the attacker as `target` | PASS | `Resolver.luau:166-168`, `Resolver.luau:261-265` |
| 42 | V11 five `starter` flags, three granted | PASS | `claims.out` |
| 43 | V13 card level, XP, effective level, trained stats invisible | PASS | no client reader; the duel card shows base stats (`BattlePresenter.luau` passes `ident.atk/hp/def` from `Cards`) |
| 44 | V14 portrait camera and HUD overlap | PASS | see owner finding (c); overlap numbers reproduced |
| 45 | V15 inspector prints `canonical`; inventory, catalogue, keepsake and rebirth preview print raw ids; preview omits catalogue and settings | PASS | `Actions.luau:214-218,226-244,1412-1476`; `Rebirth.luau:73-93` (`kept` holds only mythic, legacy and keepsake) |
| 46 | V16 raw server refusals reach the player | PASS | `Actions.luau:1244-1250` shows `detail` verbatim; strings at `RollService.luau:134,152,168,376`, `SkillService.luau:77` |
| 47 | V17 contact sound dropped while the previous clip plays; one Sound per cue | PASS | `AudioEngine.luau:283-285` |
| 48 | V19 only 5 cues have triggers | PASS | `Actions.luau:112,685-775,864`; `BattlePresenter.luau:543`; `AudioEngine.luau:360-385` |
| 49 | V20 interval 0 = unlimited | PASS | `RateLimit.luau:37-44`; `Config.luau:211,216` |
| 50 | V21 rarity letters and status glyphs undrawn | PASS | `Theme.RARITY/STATUS` have no consumer; `rarityColor` is used only at `UiView.luau:112` |
| 51 | HUD pixel layout, control bar 0.16H at 0.95H-160, reserve bottom -100, status -28, top 8 px, 0.05H+44 | PASS | `UiView.luau:559-566,828-845` |
| 52 | Camera pos / lookAt / FOV; anchors; card size | PASS | `BattleScene.luau:20-28`; `Geometry.luau:15-22` |
| 53 | Motion timings (0.13/0.15/0.05/0.22, contact at 0.28, defeat 0.40, tilt 12, back 0.45/0.35) | PASS | `Geometry.luau:25-37`, `Timeline.luau:~100-110` |
| 54 | Reduced motion ±0.35 stud travel | PASS (loose) | lunge 0.35, wind-up 0.175, recoil 0.175 |
| 55 | Flash pulse 0.13 normal / 0.08 reduced | PASS | `BattlePresenter.luau:~478-482` |
| 56 | Speed cycle 1,2,4,1; volume step 10 wrapping to 0; pause cancels effect sounds | PASS | `Actions.luau:~920-985` |
| 57 | Button sizes tab 44 / secondary 48 / primary 58 / route 62 / controls 48; fonts Bold >= 22 or primary | PASS | `UiView.luau:390-396,700-706`, `EncounterRoutes.luau:~170` |
| 58 | Theme hex tokens, rarity chips | PASS | `Theme.luau:15-52` |
| 59 | Rail 0.4W-24 (max 460) when W >= 900, else W-24 | PASS | `UiView.luau:39-44` |
| 60 | MORE opens Options with "Go:" buttons; 7 tabs | PASS | `UiView.luau:~697-700,800-806` |
| 61 | Music map: menu on all screens except encounter/boss/rewards | PASS | `AudioEngine.luau:360-366` (the stale `AudioCatalog.MENU_SCREENS` list is unused) |
| 62 | No keyboard shortcuts | PASS | no UserInputService or KeyCode in `src` |
| 63 | Honest-state strings in 4.7 (as-built ones) | PASS | `Actions.luau:459,489,1176,893,1086,321,1238`, `StoreView.luau:55,67,68,83`, `EncounterRoutes.luau:111,138`, `State.luau:57` |
| 64 | Art: 24 PNG 740x1024, manifest sha 398cb105..., replacement sha 6d8bf9f5...4cf3a, ArtIds all empty | PASS | `file`, `sha256sum`, `ArtIds.luau:3-26` |
| 65 | Audio durations 2.0/1.0/1.0/45/40, WAV 16-bit 44.1 kHz stereo, FLAC 16-bit 48 kHz, v2 files (33.4 s battle v2) | PASS | `ffprobe` |
| 66 | Stand-ins are `rbxasset://` incl. `oof.ogg` for critical | PASS | `AudioCatalog.luau:25-83` (but see D17) |
| 67 | verify.sh has 23 steps; test.sh runs only the 3 combat files; ~11,000 test lines | PASS | `tools/verify.sh` `run_step` count; `tools/test.sh`; 11,220 lines of .luau under tests |
| 68 | "grep provisional-1 src returns 20 files under src" (7) | FALSE (D2) | 23 files |
| 69 | A-42 "(>= 5.0 studs half-width)" | CONTRADICTS 3.0/4.2 (D1) | 3.0 and 4.2 say 5.4 |
| 70 | Status words "OK, Stun, Shield as S2, KO, Confirming..." in the HUD (4.5) | FALSE (D3) | `BattlePresenter.luau:~396-403,571-587` |

(Further PASS items covered by the same reads: 24-card tables, deck rules 1-4 unique owned, `DECK FULL / LEADS THE BATTLE / RESERVE n` strings, Reconnect screen auto-open (`Actions.luau:~147-157`), rate-limit semantics, economy freeze while a preview is open, Legacy grant on first boss-3 win (`BattleService.luau:283-292`), telemetry, controller selectability.)

## 3. Variances V1-V21
All 21 behave today as the spec says. V1, V5, V10 were also reproduced by running code; V2 by trace; V14 by projection. Only qualifier: V15's list of raw-id surfaces is incomplete (D11).

## 4. Appendix C and C2 re-run (Lua 5.4, ROOT edited to the export)
- Full level x route table: 60 of 60 win cells equal spec 3.9 (`harness.out` vs the table; script diff printed "60 cells compared; diffs 0"). Durations match every quoted number: 23.0 / 21.2 / 17.6 s at L1 for enc-1..3; ~15 s at L10 for enc-1; 4x = 4.4-5.8 s.
- Average rounds (`rounds.out`): W1 encounters 12.4-15.9 at L1, 5.0-10.8 at L10-15; boss-1 15.0 -> 18.6 -> 12.4 (L1/L10/L15); slowest cell w2-enc-2 L1 = 35.1. All as quoted.
- C2: L7-L10 starters vs boss-1 = 0/16/134/200; starters+Dojo vs boss-1 = 1/180/200/200; starters vs boss-3 = 192/200/200/200. Four-Epic deck 200/200/200 vs boss-1/2/3; four Commons 0/0/0. 45-wins timings: enc-1 12.4, enc-2 9.8, enc-3 8.7 min (spec says 9-12; 8.7 rounds to 9).
- Four-card figures re-run (`fourcard.out`), including w3-enc-1 which the spec quotes but the C2 script does not print: four Epics 200 on all four probes; four Commons 0 on all four; L10 {Dojo, Ninja, Wind Duelist, Gale Mage} beats boss-3 200/200 (and boss-1 200/200). Also: no single Epic alone wins any boss except Saiyan Prince (boss-2 196, boss-3 167).
- `check_data.py` (spec's own): 0 mismatches. My wider diff (section 1): 0 mismatches.
- 15 wins timing (`time15.lua`): 3.6 min (enc-3) to 4.7 min (enc-1), so "about 3-5 min" is right for the farming only; see D12.

## 5. The four owner findings
**(a) CONFIRMED.** Level-10 starters (STARTER_IDS order) win 200/200 on all 12 routes (`harness.out` L10 row). It holds for all 6 deck orders (`sweep.out`, last block). It does not hold at L9 (boss-1 134/200).

**(b) CONFIRMED, with refinements.** Starters only, STARTER_IDS order, 200 duels, `sweep.out` (L1..L15):
| Boss | first win (>0) | first 200/200 |
|---|---|---|
| boss-1 | L8 (16/200) | L10 |
| boss-2 | L5 (16/200) | L6 (L5 = 16, L4 = 0) |
| boss-3 | L6 (68/200) | L8 (L7 = 192) |
The cheapest first boss win is boss-2: L6 = 15 wins. The route to it is reliable at L6 (every W2 route 200/200 from L4). Refinements: (i) boss-2 @ L6 is 200/200 only in STARTER_IDS order; in 4 of 6 orders it is 199/200 (and 11-17/200 at L5). (ii) With the real first deck (starters + Dojo Teacher) boss-2 is beaten 124/200 at L4, 199/200 at L5, 200/200 at L6, so it is even cheaper than the spec says (spec does not mention this). (iii) Starters+Dojo vs boss-1: L7 1, L8 180, L9 200; vs boss-3: L5 5, L6 200.

**(c) CONFIRMED.** Code: `BattleScene.luau:20-28` (camera at (0,6.5,11) looking at (0,3.5,0), FOV 55) applied unchanged in `BattlePresenter.luau:149-160` on every device. Roblox FOV is vertical, so horizontal half-extent = depth x tan(27.5 deg) x (W/H). Pitch = 15.26 deg; depth to the card plane (z=0) at card mid-height y=4: 11.27; tan(27.5 deg)=0.5206. 390x844: aspect 0.4621 -> half-width 11.27 x 0.5206 x 0.4621 = 2.711 studs. Each resting card spans |x| from 1.9 to 4.5 (centre 3.2, width 2.6); visible strip = 2.711-1.9 = 0.811 -> 0.811/2.6 = **31.2 %**. This is 31 % of the card's width and, since the full card height is on screen (card y 2.2-5.8 projects to -0.205..+0.395 of NDC height), 31.2 % of its on-screen area (numerically integrated, `camera.py`), i.e. the inner strip nearest the centre line; the outer ~69 % is off-screen. It varies from 35.5 % at the card top to 26.8 % at its bottom (depth changes with the pitch). Other sizes: 375x812 = 31.1 %; 360x800 = 28.5 %; 430x932 = 31.0 %. In the wind-up pose (centre moves out 0.45) it drops to 13.9 % at 390x844. Landscape (844x390, 1280x720, 1440x900) = 100 %. HUD overlaps reproduced: card bottom vs control-bar top = 25 px overlap at H=720, 2 px at 844, none at 900+; at 390 tall 87 px (matches spec 4.2 to the pixel, assuming zero GUI inset). Caveat the spec never raises: the built place has no StarterGui/ScreenOrientation property (`grep` on the rbxlx; only 8 class types are present), so the engine default orientation applies; whether a phone actually runs portrait is UNVERIFIED (no Studio/device).

**(d) CONFIRMED, effect lost in two places, never reaches combat or XP.** Trace: client picks a node `Actions.luau:825-841` -> `app.skillAlloc` (local table, `App.luau:47`) -> `Actions.luau:1151` `SetSkills{alloc}` -> `Remotes.luau:357-358` -> `SkillService.setSkills` (`SkillService.luau:97-130`, validated by `Progression.checkSkills` `:111-147`, stored at `SkillService.luau:127` `account.skillAlloc`). Only readers of `skillAlloc`: `Remotes.luau:267` (profile echo), `Rebirth.luau:233` (carried to the new run) and the client's own local draft; nothing converts it to a multiplier. Combat side: `BattleService.luau:115` calls `svc.skillMultFn(data)`; `skillMultFn` defaults to `{atk=1,hp=1}` (`BattleService.luau:39-41`) because `Bootstrap.build` constructs the service without it (`Bootstrap.luau:100-102`); `Battles.buildSnapshot` would apply a multiplier if given (`Battles.luau:26-48`). Gap 1: the wiring is missing (above). Gap 2: no function computing a multiplier from `skillAlloc` exists anywhere, and no test references `skillMultFn`. XP bonus: `xpBp` is read only by `checkSkills` (ceiling check); `BattleService.luau:252` pays flat `Config.CARD_XP_WIN`. Extra finding: the client never loads the saved allocation back (`skillAlloc` is assigned only at `App.luau:47`), so after a refresh the Train screen shows 0 ranks and "Confirm skills" would overwrite the saved allocation with the new empty draft.

## 6. Candidate's own tests (Lua 5.5.1 `lua` and Lune 0.10.5 borrowed read-only; run in the export)
| Suite | lua | lune |
|---|---|---|
| P2 combat (3 files) | PASS | PASS |
| P3 progression + security (15 files) | PASS | PASS |
| P4 presentation (14 files) | PASS | PASS |
| P6 integration (3 files) | PASS | PASS |
| server concurrency (2 files) | PASS | PASS |
| offers (2 files) | PASS (132 assertions) | PASS |
| starter journey | PASS (1331 assertions) | PASS (1331) |
| studio checks | NOT_RUN by design (no Studio on this host) | NOT_RUN |
All 7 runnable groups exit 0 under both runtimes; no failures. Exact per-file lines are in `t-*.out` (assertion counts differ between runtimes, so I do not quote a total). NOT run: `ops/release/preflight.py` (selftest + report), `tests/test_audio_connector.py`, `tests/art/*`, `tests/release/test_publish.py`, toolkit pytest baseline, the two Rojo builds (no rojo; I instead checked source-vs-place byte equality above), all of `tools/verify.sh` as a script (it writes into the tree and needs the sibling toolkit).

## 7. Defects (all minor; no HIGH)
| # | Sev | Defect |
|---|---|---|
| D1 | MED | A-42 says the Delta-12 camera needs "(>= 5.0 studs half-width)"; 3.0 (`stage_min_half_width_studs_delta12`), 4.2 and C-B15 say 5.4. A builder can read either. |
| D2 | LOW | 7/Delta-1 says `grep -rn provisional-1 src tests` "returns 20 files under src"; it is 23 (27 including tests). Following "update every one" off the number 20 would miss files. |
| D3 | MED | 4.5 says status shows as words "OK, Stun, Shield as S2, KO, Confirming...". The HUD prints the raw event string: "STATUS OK", then `shield2`, `stun`, `stun-skip` (`BattlePresenter.luau:396-403,571-587`). "Stun", "KO" and "Confirming..." never appear in that field; `Theme.STATUS` has no consumer. Same defect family as V10 but not listed there; the Delta-1 label fix would not cover it. |
| D4 | LOW | 3.6 "Effective level ... shown to the player" is untagged and reads as as-built; it is not (next bullet and V13 say so). |
| D5 | LOW | 3.8 "the profile screen shows the pending amount": there is no profile screen; the line is on Worlds (`Actions.luau:344-372`). |
| D6 | LOW | 4.1 lists the Boss Gate primary action as "Go to Worlds"; the label is "Choose an unlocked boss" (`Screens.luau:161`; action is nav-worlds). Other primaries are paraphrased but equal in meaning. |
| D7 | LOW | 4.4 "side margin 16 px": the rail uses 12 px (`UiView.luau:~625,641`); 16 applies to battle chrome. 4.5 "Corner 8 px" is the token; the UI uses 8/10/12/14 radii (`decorate(...)` calls). |
| D8 | LOW | 3.11 presents `OfferConfig` as part of the game; it is not mapped into the place and no runtime code requires it (tests only). The dormant paid surface is thinner than described. |
| D9 | LOW | 5.1/5.2/5.5 cite `tools/bind_art.py` and `tools/upload_audio.py`; they exist at cd91a88 but not at the candidate commit 43574a5. src is identical, tooling is not. |
| D10 | MED (omission) | "Enter the next duel" / "Next gate" choose the first unlocked uncleared route in list order (`EncounterRoutes.luau:152,181-191`); boss-1 is listed before w2-enc-1, so after enc-3 the guided path sends a level-1 deck at boss-1 (0/200) instead of World 2. The spec mentions the boss as a side branch but not that the UI steers into it. |
| D11 | MED (omission) | Delta-13 scope misses internal ids on the Train screen (`node.id` buttons "prep-atk-1 0/2", "Points=N", `Actions.luau:286,825-829`), the "Go: inventory/store/..." hub buttons (`UiView.luau:~802`), and "Locked reset:", "Revision" and "Preview <token>" lines in the rebirth preview. Plus the unhydrated skillAlloc (finding d). |
| D12 | LOW | 2.2 "cheapest first boss win ... about 3-5 min": that is the 15 farming wins only. Reaching and beating boss-2 also needs the 2 unlock wins and 4 World-2 duels (my path: 306 s = 5.1 min after the 15 wins, plus the two unlock wins, about 6 min). Meanwhile the real first deck makes boss-2 cheaper (finding b ii). |
| D13 | LOW | 3.0 header says every value equals the code unless marked; `versions.rules = "provisional-2"` is a proposal and is not marked as Delta. |
| D14 | LOW | 4.2 precision: control-bar overlap numbers assume zero GUI top inset; the code uses `IgnoreGuiInset=false` (`UiView.luau:~548`), so on devices with an inset the true numbers differ. Spec does say "computed; unverified". |
| D15 | LOW | Orientation: the freeze treats portrait as a given (A-26); the place sets no ScreenOrientation (see finding c). If the engine default is a landscape lock, Delta-12's portrait case would not occur on phones by default. UNVERIFIED. |
| D16 | LOW | 3.9 "boss-2 at level 6 200/200": true in STARTER_IDS order only (199/200 in 4 of 6 orders). The spec states the order, but headline text ("cheapest first boss win ... 200/200") invites reading it as order-independent. |
| D17 | LOW | 5.2 says the game "plays engine-bundled stand-ins"; `ops/audio/status.json` records an in-Studio test where 0 of 8 `rbxasset` ids loaded (2026-09-21, before the 2026-09-26 rebinding). Whether the current stand-ins play is not shown anywhere (Studio NOT_RUN). |
| D18 | INFO | 2.2/3.6 "at most one rank per run": see claim 18; true via rebirth auto-claim, not as a per-run cap in `nextRank`. |
| D19 | INFO | Stale internal comments (not spec defects): `Screens.luau:10` lists only 4 route ids; `UiView.luau:808` states the status line occupies "-28..0" while it occupies -56..-28. |

## 8. Contradictions between candidate and freeze / dishonesty
No claim is dishonest. The freeze marks its own limits (unmeasured durations, Studio NOT_RUN, unverified DataStore-in-private-place, missing evidence directory). Spot contradictions are D1, D3, D5, D6. Anything outside the checked claims, unchanged.

## 9. Could not verify
- Anything needing Studio or a device: actual portrait orientation, actual HUD overlap with real GUI insets, whether stand-in sounds load, DataStore behaviour in a private place, art binding, moderation state.
- The 2026-09-27 gate PASS rows (evidence directory absent, as the spec says); the Job-17 owner rule text (outside the repo).
- Python suites and the release/preflight/build steps (see section 6).
- That the rbxlx loads in Roblox: only text-level source equality was checked.
- Whether the real-time pace of a duel matches the computed seconds (C-B14).
