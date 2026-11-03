# Playtest report: "untitled anime card game" (candidate 43574a5, build 55cef5d3...)

Role: playtester. Method: I played the real server command layer and the real client screens (the repo's own App, Actions, UiView and the combat resolver) in Lua 5.4, pressing buttons by name and reading the texts the UI sets. No Roblox Studio, no network, nothing modified in the card-game project. All files are in `.../scratchpad/playtest-cg/`.

---------------------------------------------------------------------------
## PART 1. Plain-language summary (for the owner)

**What it feels like.** The first three duels are fine: the cards fight automatically, each hit is a visible lunge and contact, and a duel takes about 20 seconds. After that, a new player who just presses the big "next" button hits a wall. The fourth button press sends them against World 1's boss, which they cannot beat, and the game keeps offering the same boss. In 9 of 10 simulated 15-minute sessions the player was stuck there from minute 3 to minute 15 with about 20 straight losses and no explanation. The one session that did not get stuck had rolled a rare card by luck, and cleared the whole game (79 wins, all 12 routes) in 15 minutes.

**The four things you asked me to check.**
1. Level-10 starters beat everything: **confirmed**. 72,000 of 72,000 simulated fights and 288 of 288 fights through the real server won. It takes exactly 45 wins to get there.
2. Cheapest first boss win is boss-2 at card level 6: **confirmed**. It takes 16 duels. The reason is that the route map lets you skip over boss-1 (boss-1 needs level 10).
3. Phones in portrait see about 31% of each card: **confirmed** (31.2% on 390x844, 31.1% on 375x812). The arithmetic is in Part 3.
4. Skill points do nothing: **confirmed**. I spent them in 20 matched pairs of sessions (and in 10 more with a maximum allocation seeded by me) and not one number changed: not the fighter stats, not a winner, not a single event, not XP.

**The biggest surprises beyond your four points.**
- **Rolling matters a lot early, then never.** One Rare card in the fourth slot beats most of the game at level 1. A player who just rolls for 3.5 to 4 minutes reaches the first boss win in about 5 minutes. The cheapest grinding route takes about 8 minutes, and the "press the obvious button" route takes 60 to 80 minutes (median 64.5 minutes of duels plus tapping). Once starters are level 10 nothing is left to win.
- **The game never shows card levels, card XP or run level anywhere.** The thing that makes you stronger is invisible. The "+20 XP" in the reward text is the useless "run XP", while the card XP that actually matters (+25 per win, +5 even per loss) is never mentioned, and a loss says "+0 XP".
- **Rebirth gives nothing.** It wipes levels, coins and progress. Run 2 took exactly the same 16 duels again. The only reward is 3 skill points, which do nothing.
- **Several things that look broken to a player:** the Auto-roll button can never be switched on; the "deck saved" confirmation is instantly overwritten; the intended "your four are ready, enter the first duel" step never fires; Skip, Pause and Speed are off-screen on a phone.

---------------------------------------------------------------------------
## PART 2. What I played, how, and what a real Studio session would add

**Harness** (`harness.lua`, `play.lua`): real `Bootstrap.build` server over the repo's in-memory adapter, with a seeded random generator and a controllable clock. The real client `App.boot` ran on the repo's own strict fake Roblox runtime (extracted verbatim from `tests/presentation/test_app.luau`, in `fakert.lua`). The player can only press buttons that exist on the current screen, and a button the UI greys out does nothing. One fix of mine: the repo's fake signal has a no-op `Disconnect`, which stacks frame listeners across duels and made later duels play too fast. I replaced it with a stepper that really disconnects, and playback now matches the computed durations exactly (23.3 s etc.).

**Time model (assumptions):** a tap costs 1.5 s, reading a screen costs 3 s, rolls wait the 6 s cooldown (6.2 s), duels play in real time at the 1x speed using the client's own Timeline/Geometry durations (strike = 0.13 wind-up + 0.15 lunge + 0.05 contact + 0.22 recoil = 0.55 s; damage 0.13 s; KO/replacement 0.40 s; turn marker 0.05 s). "Skip" skips playback entirely.

**Sessions played** (logs in the folder):
| Session | What | Seeds |
|---|---|---|
| A. First 15 minutes (`s_first15.lua`, `log_first15_seed3.txt`) | curious reader: roll, deck, follow the guided buttons | 10 |
| B1. Cheapest honest route (`s_cheapest.lua`, `log_cheapest_seed1_*.txt`) | fresh profile to first boss win to skills to first rebirth, using outside knowledge of the maths | 30 watched + 30 skipped |
| B2. "Press the obvious button" (`s_obvious.lua`) | only the guided buttons, until the first boss win | 30 |
| C. Deliberate getting-stuck (`s_stuck.lua`, `log_stuck.txt`) | cooldown, rate limits, locked routes, empty deck, shop, rank, rebirth, pending battle, reconnect, double claim, away | 1 long session |
| D. Roller (`s_roller.lua`) | roll N times first, best-4 deck, then guided | 5 values of N x 20 seeds |
| E. Skills A/B (`s_skills.lua`) | matched sessions with and without spending | 20 + 10 |
| F. Level-10 check (`s_l10.lua`) and direct resolver sweeps (`a1`, `a2`, `a3`, `e_events`) | win-rate tables | 8 played + 1000 per cell |
| Misc (`s_misc.lua`, `log_misc.txt`) | no-deck duel, auto-roll, visible-level scan, shop, rebirth and run 2 | 1 |

**What a real signed-in Studio session would add that I could not see:** actual rendering (clipping, text overflow, overlapping labels, how the card art looks), real input (touch scroll of the horizontal strips, controller focus), audio (the client itself says "Sound unavailable"), animation smoothness and frame pacing, DataStore saves and reconnect with real latency, the server-side auto-roll loop (it only exists in the Roblox runtime), and the Supporter store prompts. Everything below about layout is reasoning from the code's constants, not a screenshot.

---------------------------------------------------------------------------
## PART 3. The four owner findings, re-verified by play

### (a) "After about 45 wins a level-10 starter deck beats every one of the 12 routes 200/200": CONFIRMED (and slightly refined)
- Played: farming enc-1 with the 3 starters, every one of 8 seeds reached level 10 after **exactly 45 wins** (`45,45,45,45,45,45,45,45`). Card XP is +25 per win for every deck card; level n to n+1 costs 25n; level 10 needs 1125 XP.
- Then 12 routes x 3 repeats per seed through the real UI/server: **288/288 won**.
- Direct resolver sweep (the same snapshot code the server uses): 3 starters, all 6 deck orders, 12 routes, 1000 seeds each: **72,000 / 72,000**.
- Refinement: level 10 is the exact threshold, because of boss-1. Win rate of the 3 starters (400 seeds) at levels 8 / 9 / 10 on boss-1 is 8% / 68% / 100%. The other 11 routes need level 8 or less. Per-route minimum level for 100%: enc-1..3 level 1; w2-enc-1 3; w2-enc-2 4; w2-enc-3 5; boss-2 6 or 7 (order dependent); w3-enc-1 5; w3-enc-2 4-5; w3-enc-3 4; boss-3 8; boss-1 10.
- Why the grind is optional: the chart below (300 seeds) shows how little a few rare cards need.

  win % by route (e1 e2 e3 B1 | w2e1 w2e2 w2e3 B2 | w3e1 w3e2 w3e3 B3)
  | deck | e1 | e2 | e3 | B1 | w2e1 | w2e2 | w2e3 | B2 | w3e1 | w3e2 | w3e3 | B3 |
  |---|---|---|---|---|---|---|---|---|---|---|---|---|
  | 3 starters, level 1 | 100 | 100 | 100 | 0 | 94 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
  | 3 starters, level 10 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 |
  | starters + 1 Rare (Axe Berserker), level 1 | 100 | 100 | 100 | 0 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 88 |
  | starters + 1 Epic (Black Swordsman), level 1 | 100 | 100 | 100 | 0 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 |
  | starters + 1 Mythic, level 1 | 100 on all 12 | | | | | | | | | | | |
  | 4 Rare cards, level 1 | 100 on all 12 | | | | | | | | | | | |

  Lowest level at which a deck wins all 12 routes 200/200: 3 starters 10; starters + Common 9; starters + Epic 4; starters + Mythic 1; any four Rare-or-better cards 1.
- Verdict wording: "collecting stops mattering once starters are level 10" is true, and the mirror is also true: collecting a few Rare+ cards **replaces** the 45-win grind completely. Whichever comes first makes the other pointless.

### (b) "The cheapest first boss win is boss-2 at card level 6": CONFIRMED
- Why: the route chain never passes through bosses. Boss-N and the next world's first encounter both only require the previous world's **third encounter**. The route map shows it ("World 2 · Encounter 1 ... Clear World 1 · Encounter 3 first"). So boss-1 (Kakashi) is an optional side door.
- Win rate of 3 starters, 400 seeds: boss-2 at level 5 = 7%, level 6 = **99%**, level 7 = 100%. Boss-1: level 8 = 8%, level 9 = 68%, level 10 = 100%. Boss-3: level 6 = 35%, level 7 = 94%, level 8 = 100%.
- Played route (30 seeds watched, 30 seeds with Skip): **16 duels, 0 losses in all 60 sessions** (6 mandatory route clears + 9 farm wins on enc-1 + the boss-2 win). Cheapest requirements before each step: w2-enc-1 level 2, w2-enc-2 level 3, w2-enc-3 level 4, boss-2 level 6.

### (c) "Phone portrait shows only about 31% of each card": CONFIRMED (31.2% / 31.1%)
Source: `BattleScene.cameraSpec()` (camera at (0, 6.5, 11) looking at (0, 3.5, 0), field of view 55, which Roblox applies vertically by default) and `Geometry` (card 2.6 wide x 3.6 tall, centres at x = -3.2 and +3.2, y = 4, z = 0).
- Camera forward axis = (0,-3,-11)/11.4. Depth of the card plane along that axis is about 11.27. Vertical half-extent at that depth = 11.27 x tan(27.5 deg) = 5.87. The aspect ratio of 390/844 = 0.462 gives horizontal half-extent = 5.87 x 0.462 = **2.71 world units** (375/812 = 0.462, the same).
- Card A at rest spans x = -4.5 to -1.9. Only x >= -2.71 is on screen, so visible = (-1.9 - -2.71) / 2.6 = 0.81 / 2.6 = **31%**. Card B is the mirror image. The script projects every point of the card with y-dependent depth and gets **31.2% on 390x844 and 31.1% on 375x812**. In pixels, card A spans screen x = -129 to +58 on a 390-wide screen (only the inner 58 px), and card B spans 332 to 519.
- Only the inner edge strip of each resting card is visible; a card's full face appears only while it lunges to the middle (attacker at contact: 100% visible; at wind-up 14%). Excluding the HUD bands as well it drops to about 30%.
- Desktop 1920x1080: both cards 100% visible but small (about 239 px wide, 12% of the screen width each). Phone landscape 844x390: 100% visible, but by the code's own layout numbers the top HUD stack (to about y=152 px) and the control bar (top at about y=148 px) leave no free band, so the cards would sit under the chrome (unverified without rendering).
- Related phone issues from the same code: the battle control strip is 832 px wide inside a 358 px (390 phone) or 343 px (375 phone) bar. Buttons sit at x = 12 to 140 (Begin), 148 to 252 (Claim), 260 to 364 (Status, clipped), 372 to 476 (Speed), 484 to 588 (Pause), 596 to 700 (Skip), 708 to 812 (Choose another route). So **Speed, Pause and Skip are off-screen** until the player swipes the strip. The top navigation has 7 tabs of 86 px in a 342 px strip, so only START, SUMMON, CARDS, TEAM show; WORLDS, SHOP and MORE need a swipe. The battle title row (y = 60 to 104) and the HUD top label (y = 8 to about 94) overlap vertically by the code's numbers (unverified visually).

### (d) "Skill points currently do nothing": CONFIRMED (nothing changed at all)
Method: same seed, identical presses up to the first boss win and rank claim, then one session spends points and confirms ("with") and one does not ("without"). Then 16 identical duels in each (boss-2 x3, w3-enc-1..3, boss-3 x2, boss-1 x4, enc-1 x4), recording for each duel the fighter snapshot (HP/ATK of every card), winner, turns, event hash (a fingerprint of the whole event stream), reward XP and coins, card levels and card XP, run XP and level, coins.
- Honest play (3 points: prep-atk-1 x2, prep-hp-1 x1 = +3% attack, +1.5% health if they worked): **20 seeds x 16 duels = 320 duel pairs; 0 differences** in any field.
- Fixture (flagged: I seeded 16 points directly into a test profile, since real play gives 3 points per rebirth) and spent everything including card-XP nodes: **10 seeds x 16 = 160 pairs; 0 differences**. Even the +20% card XP node changed no card XP.
- What did change: only the server-stored allocation (`prep-atk-1=2,prep-hp-1=1`) and one line of text on the Upgrade screen ("Points used 3 / earned 3" instead of "0 / earned 3"). No battle outcome, stat, or XP gain.
- Cause (by code): `Bootstrap.build` never passes a `skillMultFn` to `BattleService`, so stats use a default multiplier of 1; card XP is never scaled by the XP nodes; nothing reads `skillAlloc`.
- Also, the Upgrade screen's buttons are raw ids (`prep-atk-1 0/2`) and nothing says what a node does. The client starts every session with an empty local allocation, so pressing "Confirm skills" after a reconnect without re-clicking everything would overwrite the saved allocation with nothing (by code, not played).

---------------------------------------------------------------------------
## PART 4. Times and action counts to each milestone

"Time" is my model (1.5 s per tap, 3 s per read), with duels watched at 1x unless marked Skip. Actions = button presses.

**Cheapest honest route** (medians, 30 seeds each; ranges were within a few seconds):
| Milestone | Watch each duel (1x) | Skip each duel |
|---|---|---|
| First win claimed | 5 actions, 0:33 | 6 actions, 0:12 |
| First card level-up (invisible to the player) | 5 actions, 0:33 | 6 actions, 0:12 |
| First run level-up, run level 2 (invisible, does nothing) | 17 actions, about 2:55 | 22 actions, 0:36 |
| First boss win claimed | 50 actions, about 7:41 | 66 actions, 1:42 |
| First rebirth confirmed | 57 actions, about 8:03 | 73 actions, 2:04 |

Total: 16 duels, 0 losses. With Skip the action count is higher because the Skip button is itself a press.

**"Press the obvious button" route** (30 seeds, Skip used to run it fast; durations of what watching would cost are computed from the real event stream):
- Wins enc-1, enc-2, enc-3 (about 2.5 minutes), then every "Enter the next duel" or "Continue" press sends the cards at boss-1. All 30 seeds lost their way to a first boss win: **129 to 169 duels in total (median 138), 125 to 165 losses (median 134), 390 to 510 presses (median 417)**. Watching every duel at 1x: **59.8 to 80.7 minutes of playback (median 64.5)**, about 75 minutes with taps. Even with Skip, about 10 minutes of pure losing.
- Each loss gives the cards +5 XP and the reward says "+0 XP / +0 currency", so the climb to level 9-10 (1050 XP above the 75 gained from the first three wins) is about 165 losses by arithmetic and 125 to 165 in play (some wins at 8% and 68% shorten it).

**Curious first-15-minutes player** (10 seeds): rolls 4 cards, builds a 4-card deck, follows the guided buttons. Reached boss-1 at duel 4 (about 2:30 to 3:10). **9 of 10 sessions: 3 wins, 19 to 21 losses, 61 to 65 presses, cards at level 4, stuck behind boss-1 for the rest of the 15 minutes.** 1 of 10 (seed 8, rolled the Legendary Twin-Element Prodigy) won boss-1 on duel 4 and cleared all 12 routes: 79 wins, cards level 13, 175 presses. First win in these sessions at about 1:38 (roll detour included).

**Roller** (roll N times first at the 6 s cooldown, best-4 deck, then guided; 20 seeds each; time = roll clock + duel playback at 1x):
| N rolls | first boss win | duels | note |
|---|---|---|---|
| 20 | median 3.9 min (3.8 to 6.7) | 4 (max 7) | no losses in most runs |
| 30 | median 5.2 min (5.1 to 5.5) | 4 | 0 losses in all 20 |
| 60 | median 8.9 min | 4 | rolled more than needed |
| 5 | median 9.2 min (1.9 to 16.5) | 8 (max 12) | 16 of 20 had to roll more after losing 4 in a row |
| 10 | median 9.7 min (2.6 to 23.9) | 8 (max 16) | 12 of 20 had to roll more |
Probability of at least one Rare-or-better in 20 rolls = 1 - 0.845^20 = about 97%.

---------------------------------------------------------------------------
## PART 5. Getting stuck on purpose: what the player is told

(From `log_stuck.txt`; "shown" = text the player sees.)
| Attempt | What happened | Understandable? |
|---|---|---|
| Tap roll twice in a row | "cooldown active" | Partly. No timer, no seconds, the button never greys. |
| Spam roll 6 times in 8 seconds | 1 roll, then "cooldown active", then "rate limited" x3 | Poor. "Rate limited" reads like a penalty; the server knows the wait (retryAfterS) but the UI never shows it. The cooldown rejections also use up the rate-limit allowance. |
| Locked boss / route | button greyed; tapping shows "Locked — clear World 1 · Encounter 3 first" | **Good.** Names exactly what to do. |
| Server-side attempt at a locked route (tampered client) | "route locked: clear enc-3" | Fine (internal id, not player-facing). |
| Shop with 0 coins | "insufficient earned currency: need 100 Run Coins" | Good (balance also shown on the shop screen). |
| Claim rank with no boss win | "no rank eligible (need boss milestone, next rung, unclaimed)" | Jargon; the intended wording "clear a boss" is only partly there. |
| Spend skill points with 0 earned | "Skill cap reached: 0 earned points spent — clear a boss rank to earn more" | OK. |
| Rebirth confirm without preview | "Refresh preview first" | OK. |
| Rebirth preview before a boss win | "rebirth blocked: boss milestone not committed" | Jargon ("milestone"); does not say "win any boss". |
| Duel with no saved deck | "deck illegal: deck size must be 1..4" | Confusing: players do not know they never saved a deck. |
| Remove all cards and press Save | **No error. The game silently saves the first four owned cards in alphabetical id order** and shows them as "your deck" | Misleading: your choice is replaced without a message, and alphabetical can drop your best cards. |
| Leave a duel mid-way and start another | "battle already pending" | Unclear. (A friendlier client message exists but never triggers because it checks a field that is a boolean.) |
| Quit with an unclaimed duel, return | reconnect screen "Pending 1:1 on enc-1 — Resume replays server events, never re-grants." then Resume replays the duel and Claim works | **Works well.** |
| Claim twice (second tap; and a second claim sent directly) | button says "No battle to claim"; direct re-claim returns AlreadyCommitted; XP and coins unchanged (20 / 10 after both) | **Safe.** |
| Away: claim right away | "Away reward claimed: +1 Run XP." | Trivial. |
| Away 3 h, claim, claim again | +180 Run XP, then "no away reward earned yet" | Clear. |
| Away 12 h | +480 (capped at 8 h) | Clear; but Run XP does nothing (below). |
| Change deck while a duel is pending | server: "deck locked during pending battle" | fine |
Also: **the status line is stale across screens**. A message like "insufficient earned currency: need 100 Run Coins" stayed on screen after leaving the shop, and "rebirth blocked: boss milestone not committed" was still showing on the Deck screen four taps later.

---------------------------------------------------------------------------
## PART 6. Where a new player is confused or bored; what is dishonest or unclear

**Ranked player-facing problems** (worst first):
1. **The guided path leads into an unwinnable wall.** The primary "Enter the next duel" / "Continue" button always picks the first unlocked, uncleared route, which after enc-3 is boss-1 (needs level 10). It never offers the easier unlocked World 2 routes, which the map itself shows as ready. Result: 20 to 165 losses in a row, each saying "Continue to the next unlocked route" and "+0 XP". 9 of 10 first-15-minute sessions were stuck.
2. **Progress is invisible.** No card level, card XP, run level or run XP is shown anywhere (I scanned all 13 screens; the only "level" word is "level-1 copy" in the shop). The duel uses leveled stats (Spirited Ninja in a chapter-3 duel at level 6 has HP 75 / ATK 11) while the Cards screen shows base stats (HP 58 / ATK 9), unexplained. A loser cannot know that losing "trains" a little.
3. **The XP numbers are misleading.** "+20 XP / +10 currency" is run XP, which affects nothing; card XP (+25 win, +5 loss) is not reported; a loss says "+0 XP" although cards gained XP. Away "Run XP" (60/hour, up to 480 per claim) is advertised as progress but also does nothing.
4. **Skill points and rebirth are fake progression.** Points do nothing (finding d); rebirth wipes everything for 3 useless points. Run 2 cost the same 16 duels again and rank 2 added 3 more points that do nothing. Nothing tells the player this.
5. **Rolling is all-or-nothing luck.** Rare+ cards skip the entire grind; a Common roll helps little. The roll screen has no timer, no hint which new card is better, and no suggestion to roll when stuck. Duplicates do nothing (no merge, no level), so rolls after the first few Rares are pointless.
6. **Broken or dead controls:** Auto-roll "switch on" can never turn on (the client always sends `enabled = not prefs.autoRoll` with a local flag that starts true and is never updated; I pressed it 3 times and the server flag stayed false). The first-deck guided jump ("Your four are ready. Enter the first available duel.") never fires. Save-deck success text ("Deck saved — confirmed") is replaced within the same tap by "Your saved run is ready"; the same hides the shop pack result, the rank-claim result and the rebirth-done message. After Rebirth the screen simply returns to the same START text as a new game with no "new run" message.
7. **Phone usability:** only about 31% of each resting card is visible in portrait; Speed, Pause and Skip are off-screen to the right; Worlds, Shop and More tabs are off-screen. Reaching Rebirth/Upgrade/Boss relies on in-flow buttons or the More tab.
8. **Confusing first screen:** the highlighted button is "Take a free roll", while the text says "NEXT MOVE / Build your first deck". Two competing instructions.
9. **Rebirth preview text is raw:** "Kept: " (empty), "Locked reset: " (empty), "rank reward nil", "Rank pending nil", "Resets: level, xp, currency, deck, cardLevels, cardXp, encounters, bossClaimed, cooldownReadyAt, autoRoll", "Preview 1:38:74". It does not say that levels, coins and progress are lost in plain words, or that the benefit is zero.
10. **Boredom after the first minute:** 12 routes, every enemy is a single card, duels are 10 to 51 seconds of the same two-card exchange, and there is nothing to choose in a duel (no manual play, a 4-card deck that is just an order). After the first boss + level 10 nothing remains. Starter-only fights mostly have no status effects at all (3,600 sampled duels: no shield/stun/burn events).
11. **Duel HUD labels** read "YOUR A:46 HP RIVAL B:0 HP" (internal A/B labels) and show only the active cards' HP.
12. **Stale status text** across screens (Part 5).

**What worked honestly:** server rejections are real and safe (double claim, tampered ids, locked routes); reconnect/resume works; exact odds and the 1800-roll Mythic guarantee are shown; the Store says plainly "cannot be bought with Robux"; the locked-route message is clear.

---------------------------------------------------------------------------
## PART 7. Duel feel, judged from the event stream (3,600 sampled duels, `out_events.txt`)
- Per round: 2 strikes + 2 damage events + 1 turn marker. 73% of presentation time is a strike animation (wind-up, lunge to edge contact, recoil), 37.2% of events are strikes and 37.2% damage; ko 2.7%, replacement 1.5%.
- Average duel: **23.2 s at 1x, 11.6 s at 2x, 5.8 s at 4x**. Range by route and level: 9.7 s (enc-3, level 10) to 54.3 s (w2-enc-2, level 1). Strikes per duel 13 to 70.
- A strike is 0.55 s with contact at 0.28 s (the card's edge meets the other card; HP numbers update at that beat). That reads as "cards hitting cards", and only one card per side is on stage. Damage is `max(1, atk - def +/- 1)`, so duels are nearly deterministic: the outcome is decided by stats and level, not by anything the player sees or chooses.
- Downsides: the on-screen result is only text HP in the top label; no floating damage numbers or impact effects exist in the code; on phones the resting card is only a 31% strip.

---------------------------------------------------------------------------
## PART 8. Ratings (1 to 5)
- **First 15 minutes were fun: 2.** Three decent minutes, then a wall of repeated losses for most players; only a lucky rare roll saves it.
- **Rolling and collecting still mattered once starters were level 10: 1.** Level-10 starters win 100% everywhere; rolls change nothing after that.
- **A duel felt like cards hitting cards: 4.** Distinct lunge, edge contact and recoil per strike, 73% of the time in strikes (desktop); phone view of the resting cards is poor.
- **Nothing was dishonest or confusing: 2.** Invisible levels, misleading XP, do-nothing skill points and rebirth, overwritten confirmations, and a dead Auto-roll button.
- **Phone and desktop both felt fine (from layout code): 2.** Desktop is fine (4); phone portrait shows 31% of cards, hides Skip/Pause/Speed and three nav tabs off-screen (2).

---------------------------------------------------------------------------
## PART 9. What I could not verify
- Anything visual: real rendering, clipping, text overflow or overlap, art, effects, animation smoothness. All layout figures come from code constants (assumes Roblox's default vertical field of view and a full-screen 3D viewport; a top-bar inset of about 36 px would change the numbers only slightly).
- Touch/gamepad behaviour, audio, real DataStore durability, network latency or "Pending" states, the Roblox-only server auto-roll loop, the Supporter store.
- Human timings are my model (1.5 s taps, 3 s reads); the duel durations are the client's own constants applied to the real event stream, but real frame rate can stretch them.
- The keyboard/controller path and the "reduced motion / flash" options were not played.
- Whether the owner's intended design makes the invisible stats visible somewhere I did not reach: I scanned all 13 built screens and the dynamic rows after a full run.

## Comparison with the design brief (read last, `docs/brief.md`)
The brief promises "unaffordable/locked actions show the requirement" (true for routes and shop), "finite skill choices read actual points and caps" (true, but the choices have no effect), "rebirth ... all level resets" shown in the preview (shown only as raw field names), "honest states" and "one primary action per screen" (the first screen has two competing prompts). It also calls first-session timing an unmeasured target: my numbers (about 33 s to the first win; about 8 minutes of watching to the first boss win by the cheapest honest path; over an hour by the guided path) are a first measurement. The brief says a loop of "upgrade ... rebirth" exists; in play both are empty rewards.

## Files
`harness.lua`, `play.lua`, `fakert.lua` (drivers); sessions `s_*.lua`; analysis `a1_winrates.lua`, `a2_thresholds.lua`, `a3_rarity.lua`, `c_phone.lua`, `e_events.lua`, `lib.lua`; outputs `out_*.txt`, `log_*.txt`.
