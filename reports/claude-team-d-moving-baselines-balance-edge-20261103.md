# TEAM D (Claude) — consolidated filing, 3 Nov 2026: `moving` PASS, flag-off baselines, card balance pass, `atlas:edge` blocked (needs a call)

## 1 · Completions
1. **`moving` fixture, option (a) (your ruling): LIVE PASS under a fresh checker.** Runs `20261103T142804Z-moving` and `20261103T142954Z-moving` (new test place, Plays 30/31). Branch `prep/moving-association-20261103` @ `18d01a2` (bounded capture-time association; runbook assumption 16 amended; option (b) not built). **NOT merged** — your order needed.
2. **Flag-off baselines:** `atlas:scout` ✓ and `moving` ✓ captured. `atlas:edge` **not captured** (section 3).
3. **Card game balance pass: done and verified, code + spec only, nothing uploaded.** Spec: `docs/GAME-DESIGN-SPEC.md` §8 + Appendix C3 + Appendix D on branch `prep/card-game-design-spec` @ `e4e4746` (locked section kept separate from Δ1–Δ15). Code: branch `prep/balance-pass-20261103` @ `956b4bc`.
   - Before/after (starter deck vs the 12 routes, real resolver): clean sweep 12/12 → **11/12**; only `w3-enc-1` moves, **100% → 0%** with the smallest numbers-only change.
   - Change list: `Config.ROUTE_ENEMY_LEVEL["w3-enc-1"] = 8`; `Battles.buildEnemySnapshot(…, enemyLevel)`; `Bootstrap` encounter.enemyLevel; `BattleService` passes it; two test pins updated; new `tests/progression/test_balance_pass.luau`. Seat verification re-run fresh.
   - Open for you: Δ2 interaction (re-lock at enemy level 10 if Δ2 is approved); proposed **Δ16** (guided continue skips the unwinnable boss-1).
4. **Merged earlier on your orders:** d1 clamp 16/72 → main `0c325f8`; place-override → main `5ee9f63`. Sealed hashes unchanged.

## 2 · A mistake of mine, fixed
`git add -A tools tests docs` swept ~1,900 untracked evidence files into the moving commit (checker F1). I remade the commit with explicit paths (`18d01a2`); nothing was pushed.

## 3 · `atlas:edge` — root cause, and a bigger finding
- Plays 31/32 timed out at the edge phase. Console dump (new diagnostic, branch `prep/atlas-edge-diag-20261103` @ `c2617d6`): `EdgeBot.luau:18` waited forever on `ServerScriptService.GameService` (it lives in `…Nightshift`). **Fixed** (`818dc22`, pinned by `tests/tools/test_edge_bot_paths.py`).
- That bot has **never run past line 18**. A fresh checker reviewed the rest offline: **NOT READY, 5 HIGH**:
  1. nothing starts the client half (`AtlasEdgeBotClient`), so "start" is never sent (300 s wait then fail);
  2. the car is named `AtlasEdgeCar`, but production `Driving.client` attaches only to `Player_<UserId>` — the car would never drive;
  3. `place()` indexes `direction[1]/[2]` but EdgePlan returns `{x,y,z}` — all 56 runs would face north;
  4. payload likely > 512 KiB direct-post limit and edge phases cannot be chunked;
  5. 7 starts × 8 headings × up to 20 s cannot fit the runner's 588 s phase window (≈950 s modelled).
  Also MED/LOW: the validator would pass a car that never moved (no displacement/heading check), TouchDrive can zero the throttle, velocity carries between runs, no seat/health guard. Items 1–3 I verified myself in the source.
- Studio opening is 3/3 used (Plays 30–32); the contained Studio stays open on the new place.

## 4 · Decisions I need from you
1. **Budget for `atlas:edge` (defect 5).** Options: (A) split the 7 starts across 2 invocations (e.g. 4 + 3, ≈400 s each; 2 Plays; validator takes a start subset) — my recommendation; (B) cut RUN_SECONDS to ~10 s (may not reach the wall from the 2,500-stud starts); (C) raise the edge phase budget above the runner's 900 s run cap (touches runner limits).
2. **Merges:** `prep/moving-association-20261103` (18d01a2), `prep/atlas-edge-diag-20261103` (diagnostic + bot path fix, stacked on the moving branch), card branches `prep/card-game-design-spec` / `prep/balance-pass-20261103`.
3. Δ2 / Δ16 card-spec items above.

## 5 · What I am doing meanwhile (offline only, no Studio use, nothing merged)
Fixing the option-independent edge defects (1, 2, 3, 6–12) with tests, then a second fresh checker. The Play waits for your answer on budget (A/B/C); with no answer I will take (A) because it needs no runner limit change.
