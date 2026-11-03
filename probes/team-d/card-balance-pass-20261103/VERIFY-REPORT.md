# Independent verification: locked balance pass (spec section 8, branch prep/card-game-design-spec @ 448cec3; code prep/balance-pass-20261103 @ 3d21456)

Verdict: **PASS-WITH-FINDINGS**. The numbers in 8.3, 8.4 and the opener rows of 8.6 reproduce (every cell within 0.3 pp of the spec); the code change does what it says and only that; the wall cannot be bypassed on any path I found. But three sentences in section 8 are stronger than the evidence ("any Uncommon-or-better card"), Appendix C was not updated and now measures the pre-pass game, and the wall is conditional on skill multiplier = 1 (true as built, false under proposed Δ2).

Method. Own harness `h.lua` (+ h2/h3/h4) in this directory, plain Lua 5.4, run with cwd = code root. It loads each tree's own Config, Battles, Cards, Resolver, and **Bootstrap.contentTables(Cards)** (the shipped content path: that is where enemyLevel is attached to each route). Player snapshot = `Battles.buildSnapshot(deck, editionsById, levels, encounter.chapter, {atk=1,hp=1})`; enemy snapshot = `Battles.buildEnemySnapshot(encounter.enemyIds, editionsById, encounter.enemyLevel)`, exactly the call in BattleService.luau:121-122; rules `Config.RULES_VERSION`; seeds s*7919, s=1..1000. The same script ran unmodified against BEFORE (cg-candidate) and AFTER (card-balance-wt). Author files were not read before the numbers existed. Raw outputs: out_BEF_main.txt, out_AFT_main.txt, out_BEF_collect.txt, out_AFT_collect.txt, out_alldecks.txt, suites_lua.txt, suites_lune.txt.

## 1. BEFORE: PASS
Starters (NARUTO, THORFINN, ASTA) and first deck (+VG-HEAD-01), 1000 fights/cell, route order enc-1 enc-2 enc-3 boss-1 w2-1 w2-2 w2-3 boss-2 w3-1 w3-2 w3-3 boss-3:
- starters L10 and L30: 100 on all 12. first deck L10 and L30: 100 on all 12. (12/12 each.)
- At L6 (below the ceiling the claim is about): starters 0 on boss-1, 99.7 boss-2, 32.8 boss-3, rest 100; first deck 0 boss-1, 99.9 boss-3, rest 100.
- Four-card claim in 8.1 ("even a Common, sweeps every route"): first deck is the all-Common deck, 12/12 at L10/L30. Confirmed.

## 2. AFTER vs BEFORE: PASS
Only one cell family differs: **w3-enc-1 100 -> 0** for all six rows (starters and first deck at L6, L10, L30). Every other of the 66 other cells is identical to the decimal between trees. Full rows (AFTER, order as above):
- starters L6: 100 100 100 0 100 100 100 99.7 **0** 100 100 32.8
- first deck L6: 100 100 100 0 100 100 100 100 **0** 100 100 99.9
- starters L10 / L30: 100 x8, **0**, 100 x3 (both)
- first deck L10 / L30: 100 x8, **0**, 100 x3 (both)
Comparison with spec 8.4 (all 72 before/after cells): **largest difference 0.3 pp** (starters L6 boss-2 99.7 vs 100, boss-3 32.8 vs 33; first deck L6 boss-3 99.9 vs 100). No cell differs by more than 2 pp. Spec's "routes won >= 90%" column (10/12, 9/12, 12/12, 11/12, 11/12 ...) matches.

## 3. Collecting, starters + one card at L30, AFTER: PASS for the cards tested
All 12 routes 100.0 for Goku, Musashi, Sea, Ichigo, Namek, Bear, Guts, Kakashi, except Goku on w3-enc-1 = 99.9. Matches 8.6. (BEFORE the same decks are 100 everywhere.)
Beyond the spec's list (h2 addall N=200, h3 N=1000, enemy level 8, deck at ceiling):
| card added to starters | rarity | w3-enc-1 win % |
| BK-WIND-01 | Uncommon | **7.6** |
| VS-SNAKE-01 | Uncommon | **50.3** |
| BC-WIND-01 | Uncommon | **50.3** |
| MHA-DEKU-01 | Rare | 91.2 |
| BL-FROST-01 / BK-FALCON-01 | Rare | 99 / 99.5 |
| all other non-Common cards | | 100 |
8.6 omits these (it lists only the openers). See defect 1.
Sweep of 8.3 table (enemy level vs deck, N=1000): reproduces within 0.5 pp (starters 100/90.1/16.1/0/0/0/0; first deck 100/100/99.5/46.7/3.8/0/0; Goku 100 100 100 100 100 99.9 35.2; Musashi ... 27.8; Sea ... 56.6; Ichigo ... 73; Bear/Namek/Guts/Kakashi 100 throughout).
All 10,626 four-card decks, 20 fights, >=90%: Common 0/1, Uncommon **91/125** (spec 88), Rare **1660/1694** (spec 1656), Epic 4165/4165, Legendary 2870/2870, Mythic 1771/1771. Differences are the borderline-90% decks; equivalent. Spec's own "70%" for Uncommon means 30% of Uncommon-best decks do NOT open the door.
All 24 orderings of the Common 4-deck, plus 1-card and 3-card sub-decks, at L30: max 0% on w3-enc-1.

## 4. Enemy-level mechanics from the AFTER code: PASS
- Config.ROUTE_ENEMY_LEVEL = { ["w3-enc-1"] = 8 }; Bootstrap.contentTables sets `enemyLevel = Config.ROUTE_ENEMY_LEVEL[routeId] or 1` for the 9 encounters and 3 bosses. Printed from the shipped builder: w3-enc-1 level 8, every other route level 1.
- Shipped snapshot for w3-enc-1: BK-GUTS-01 hp=110 maxHp=110 atk=21 def=5 speed=8 shield=0 status none entry none (8.3 says hp 110, atk 21, def 5, speed 8: exact).
- Other 11 routes: enemy snapshot lines are byte-identical to BEFORE (compared the printed snapshots, fields hp/maxHp/atk/def/spd/shield/status/entry). BEFORE has enemyLevel=nil, AFTER has 1, giving the same raw stats.
- Cards.luau and the whole src/shared/content dir: `diff` identical to BEFORE. 24 cards, 3 worlds, 12 routes, no stat change.
- buildEnemySnapshot validates the new argument (whole number >= 1); default 1.

## 5. Reachability (Bootstrap.luau, printed): PASS
requiresClear: enc-1 nil; enc-2<-enc-1; enc-3<-enc-2; boss-1<-enc-3; w2-enc-1<-enc-3; w2-enc-2<-w2-enc-1; w2-enc-3<-w2-enc-2; boss-2<-w2-enc-3; **w3-enc-1<-w2-enc-3**; w3-enc-2<-w3-enc-1; w3-enc-3<-w3-enc-2; boss-3<-w3-enc-3. So w3-enc-1 gates w3-enc-2, w3-enc-3 and boss-3 (4 routes). Enforced on the public lane (Remotes -> startFromRemote -> enforceRoutePrerequisites=true, BattleService.luau:95,205-207); `BattleService.start` (no prerequisite check) is the trusted internal call, not referenced by Remotes. boss-2 needs only w2-enc-3 (World 2), starters at L6 win it 99.7% (first deck 100), so the first rebirth (needs any boss win) stays free without World 3. Starters beat boss-1 from L10 only (0% at L6), as spec says.

## 6. Repo suites on AFTER: PASS (with test-strength findings)
lua5.4 and lune: progression, presentation (JOURNEY STATE SWEEP PASS states=233 routes=12), integration, server, offers, tests/run_all, test_starter_journey: **all green, exit 0, in both runtimes**. `git diff prep/card-game-design-spec~1 3d21456 --stat`: 6 files (4 src, 2 tests: test_free_reachability, test_journey_states). Nothing else in tests/ changed. git status clean after running.
Mutation testing (scratch copies of the AFTER tree, never the real one):
- wall removed (table empty): both updated tests FAIL on "starter-only deck does not open w3-enc-1". Good: the wall assertion is real.
- collected card dropped from the deck (starters only): both FAIL ("not cleared in 20 tries"). Good: the collection path is real.
- wall at enemy level **7, 9, and 6**: both tests still **PASS**. So the tests do not pin 8, and (they use the 3-starter deck only) they do not test the first-deck claim at all (level 6 lets the first deck win 47%, and the tests still pass).
- Generosity: the seeded LCG (20261103) makes the first free roll a Rare, **DB-NAMEK-01**, in both tests (I instrumented copies). So the collecting path is proven only for one of the strongest openers; no test covers a weak Uncommon (BK-WIND, SNAKE, BC-WIND) that cannot open the door. The roll helper also lets a non-Common of any kind be "collected" and then requires the clear to succeed, so a different seed could fail the test, which is fine, but it means the test is seed-fitted.
- "winner ~= A" is the only wall check (5 starter-only attempts in one, 1 in the other); adequate since the resolver is near-deterministic.

## 7. Spec hygiene
- Diff 6b9c985..448cec3: only (a) Decisions item 7 added, (b) one sentence in 3.4, (c) a blockquote banner at the top of 3.9, (d) new section 8. 3.12, section 7 (propagation) and the Δ1-Δ15 list are untouched. Section 8 states it is separate from the Δs. PASS.
- "Numbers only" holds: Cards.luau identical, the only src change is one Config table plus 3 plumbing hunks (Bootstrap, BattleService, Battles optional argument).
- Section 8 claims vs my numbers: 8.1, 8.2 (CHAPTER_CAPS[3] 15->8 on BEFORE: starters boss-1 6.8% (spec 8), first deck boss-1 87.0% (spec 88), starters+X sweep 12/12; matches within 1.2 pp), 8.3 table, 8.4, 8.5 (8 of 12, boss-2 at L6, 15 training battles: 233-188=45=15x3), 8.6 opener rows all reproduce. Failures are in defects 1-4 below.
- Section 8 header says a checker record is "at the end of §8"; there is none yet (this report is that record).

## 8. Weaker than claimed? Bypass search
Callers of buildEnemySnapshot (src, tests, tools, ops, plugins): BattleService.luau:121 (the only runtime caller; takes encounter.enemyLevel) and tests/progression/test_validation.luau:403 (2-arg, level 1 default). StudioDemoDrill uses Bootstrap.build (same encounters). Client has no resolver/enemy builder: EncounterRoutes.luau only mirrors unlocks, BattlePresenter plays server events. Replay/reconnect (`BattleService.status`) re-resolves from the stored snapshotA/B, it does not rebuild. Away progress touches XP only; `run.encounters` is written only by claim from `firstClear` (decided at start from the server result) and reset by rebirth. A stale pre-pass copy of src exists under session-artifacts/codex-skybridge-20260927/private-art-test (raw buildEnemySnapshot) but default.project.json maps only src/. No bypass found with: all 24 orderings of the Common deck, 1/3-card decks, L30 (capped at 6 by chapter 1).
Conditions under which the wall is weaker than stated: see defects 1, 3, 7.

## Defects (HIGH = a spec claim is false or the wall can be bypassed)
1. **HIGH (false claim).** Section 8.7 (C-BAL2) "starters + any Uncommon-or-better card at ceiling win 100% on all twelve", 8.3 "lets a single collected Uncommon-or-better card open the door", and Decisions item 7 "a collected Uncommon-or-better card can". False: starters + BK-WIND-01 wins w3-enc-1 7.6%, + VS-SNAKE-01 50.3%, + BC-WIND-01 50.3% (all Uncommon), + MHA-DEKU-01 (Rare) 91.2%. The spec's own 8.3 "Uncommon 70%" contradicts the sentences. 8.6 lists only the cards that open. Practical effect: with the shipped odds 37.5% of "first non-Common" free rolls (3 of 5 Uncommons x 62.5% of non-Common rolls) will not open the door; the player keeps rolling every 6 s, so it is friction, not a lockout. Fix the wording ("an Uncommon-or-better card, 2 of 5 Uncommons and all 7 Rares open it") and add the weak rows to 8.6.
2. **MEDIUM.** Appendix C (and acceptance C-BAL1) was not updated: it calls `Battles.buildEnemySnapshot({enemy}, by)` with no level. I ran Appendix C verbatim against the AFTER code at L10: w3-enc-1 = **200/200**, i.e. it reproduces the pre-pass game and cannot detect the wall. C-BAL2 cites a "balance.lua-style harness" that is not in the spec. Add the level argument (use Bootstrap.contentTables) to Appendix C.
3. **MEDIUM (conditional).** The wall holds only while the skill multiplier is 1, which is as built (Bootstrap passes no skillMultFn; Δ2 is a proposal). Under proposed Δ2 (up to +6% atk, +6% hp): first deck wins w3-enc-1 **75.2%** at +6/+6, **58.8%** with +6% atk only, 3.5% at +5/+5; 3 starters stay 0% until about +15%. Section 8 never mentions this, and 8.5 "reaching World 3 needs a card from outside the starter set" would be false after Δ2. Rules version `provisional-2` (Δ1) was not simulated: UNVERIFIED.
4. **MEDIUM (stale text).** Decisions item 4 still says "I did **not** fix that", the §3.9 "It is **not fixed here**" paragraph and A-12 (Appendix A and the §7 line) still say enemy decks stay single-card at raw stats and the lever is deferred to the verdict. Contradicts item 7 / section 8 / the changed 3.4 sentence.
5. **LOW.** Updated tests do not pin the lock: they pass with the wall at enemy level 6, 7, 9; they never test the first-deck claim; the collecting path runs on one seeded outcome (a Rare, DB-NAMEK-01 on roll 1).
6. **LOW.** Section 8 header promises a checker record at the end of 8; none is present.
7. **LOW/INFO.** The wall bites in run 1: rebirth keeps Mythic, Legacy entitlements and one keepsake (Rebirth.luau); boss-3 pays a Legacy Goku (Config.LEGACY_STORY_REWARD_ID), and any keepsake that opens the door carries over, so later runs start past the wall. 8.6 "matters once" is right but unstated. Also: a pending battle stored before this change keeps its raw w3-enc-1 snapshot and could first-clear it on claim (unpublished game, theoretical).
8. **INFO.** Spec rounds a few cells (99.7 shown as 100, 99.9 as 100, 99.5 as 100 at first-deck enemy L5). All within 0.5 pp.

## Could not verify
Human play; Roblox Studio/device behaviour; stored-document migration of pending battles (not exercised); rules version provisional-2 (Δ1) effect on the wall; deck order outside the permutations tested (all 24 orderings tested only for the all-Common deck; the 10,626-deck sweep uses one fixed order per combination); the author's claim "20 fights each" in the all-deck sweep was reproduced with my own 20-fight pass (not their seeds); the author's harness/tables were not consulted.
