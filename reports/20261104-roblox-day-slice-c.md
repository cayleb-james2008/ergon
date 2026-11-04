# Slice C — card-game D12 bundle, up to the owner gate

Date: 2026-11-04
Source: `fleet-scratch/overnight/roblox-day/SLICE-C-BUNDLE.md` (condensed)
Fences: board only. No `--live`, no key, no `target.json`, no upload/publish/Studio/`bind_art --live`/audio upload, no `src/` change, no balance move, no push/merge to `master`.

## Branch and commits

- Worktree `~/Work/tmp/card-d12-wt`, branch `prep/d12-bundle-20261104` cut from `master` @ `a22c17e`. No merge, no push; the lead decides what lands.
- `93edd0d` — `docs/D12-SPEC-DELTAS.md` (new, 302 lines): Δ2 and Δ16 to the Δ1–Δ15 standard.
- `7ebbb9a` — `docs/GAME-DESIGN-SPEC.md`, `docs/D12-ANSWER-SHEET.md`, `docs/D12-PACKAGE-20261104.md` made to agree (Δ2 settled in-the-15, Δ16 labelled not-in-the-15, line cites and stale heads corrected).
- `git diff --stat a22c17e HEAD` = 4 files, +351/−18, **all under `docs/`**; status clean (untracked `.tools` symlink is gitignored). **Trunk at `7ebbb9a`.**

## Δ2 — settled as one of the 15, from code

One sentence: the skill multiplier is never handed to the battle service and card XP is paid flat, so the 3 points a rebirth grants genuinely do nothing; the only claim needing narrowing is "the saved allocation is not loaded after a refresh" — that is a **client draft** (`src/client/App.luau:47`) never hydrated, even though the server persists and returns it. Player-visible: after a refresh the Train screen shows `0/N` while the server holds the ranks, and one Confirm tap sends `{}`, accepted as a free respec. That's why hydration is part of Δ2, the 15 stays 15, and the old answer sheet's "not in the 15" was wrong. Key evidence: `BattleService.luau:39-41,115-117,252-253`; `Bootstrap.luau:102-104`; `Battles.luau:27,31,47-48`; `Progression.luau:111-146`; `SkillService.luau:84-88,118-127`; `Rebirth.luau:191-207,233`; `Config.luau:110`; `Remotes.luau:266-268`.

## Δ16 — drafted, not in the 15

Rule: unlocked-and-uncleared → **normal encounters before bosses** → **lowest enemy level first** → **tie-break = list order**. Fixes the first-15-minutes case (after enc-3, `Next gate: World 2 · Encounter 1`, Continue starts `w2-enc-1`, which the starters win 200/200 at level 3). **Client-only change** (route rows gain `enemyLevel` + one `pickNext`; display order unchanged). §8 interaction stated, not papered over: `w3-enc-1` still gets picked (it *is* the next gate; a "skip what your deck can't beat" heuristic would need a strength estimate the client doesn't have). Invalidated tests named from the tree; Δ16 **adds** three cases (`test_route_states.luau` pins nothing today) rather than breaking one. Not implemented — drafted only.

## Gate proof — docs-only change moved nothing

Command (twice): `TOOLKIT_ROOT=<empty dir> bash tools/verify.sh --skip-toolkit --out <dir>` in `~/Work/tmp/card-d12-wt`.

- Run 1 (before edits, `a22c17e`): **22 steps, 0 failures** (`toolkit-53` skipped as designed), build twice-identical `99ded6a6d701bc68cda2f260f6686a8a82292df071b5341d9c3aa179ff2b8400`, 135-input fingerprint, head `fae24d6f693d`.
- Run 2 (with all doc edits): **22 steps, 0 failures**, identical place hash, identical fingerprint head, 135 inputs.

Why the hash cannot have moved: the build's mapped inputs are 56 files (`default.project.json` + `src/**`), zero `docs/` entries; the diff is four `docs/` files. Run 2 executed with doc bytes in the working tree before the commits (commit step changed no content; status clean after) — graded bytes are the committed bytes.

## Owner ask list (one line each)

1. **The spec** — approve, approve with strikes, or send back (now includes Δ1–Δ15, Δ2 among them).
2. **Δ2** — leave it in the 15, or strike it and remove the skill tree from the UI.
3. **Δ16** — add the guided routing rule, or leave the button walking new players into boss-1.
4. **If Δ2 ships: the §8 wall** — re-lock `w3-enc-1` at enemy level 10, or accept the first deck opens World 3 ~75% of the time at level 8.
5. **D12 (a)** one line: "yes, upload for binding only".
6. **D12 (b)** a brand-new Private experience + `universeId`, `placeId`, `userId`/`groupId`.
7. **D12 (c)** listen to the five audio files, yes or a named replacement.
8. **D12 (d)** re-approve `assets/images/replacements/BK-GUTS-01-clean-candidate.png` (path corrected here; it previously named the Roblox-rejected file).
9. **D12 (e)** Open Cloud key through the secure write-only route, never chat.
10. **D12 (f)** a signed-in Studio window when binding checks are wanted.
11. **Rights (A-04)** — accept the moderation risk for the 24 portraits, or hold.
12. **Later:** the playtest verdict (C-PLAY), and A-31 (ranks 6–8 add nothing) if he wants it raised.

## Found wrong, not fixed

Appendix D-1's slipped line cite (dated log, left alone) · `test_battles.luau` cannot be invalidated by Δ2 (grep) · `test_balance_pass.luau` is the real Δ2 hazard (`:57` hard-codes `{atk=1,hp=1}`, `:71` asserts the World-3 gate — stays green while §8's claim goes false once skills apply) · "costs the same 16 duels" rebirth figure removed, not verified (2 Nov playtest, not re-derivable) · `test_route_states.luau` pins no guided target (the three cases are specified, not added) · older D-3 defects untouched · §3.12 row order (V22 above V18) cosmetic, not renumbered · C-P1b proposed, not added to §6.
