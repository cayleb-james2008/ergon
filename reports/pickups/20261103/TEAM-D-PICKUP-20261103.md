# TEAM D — PICKUP PACK (Roblox street + card)

**For:** the DeepSeek continuation after Claude's weekly limit (Nox order `20261103T231027Z`, 18:10 ET; the 18:02 handoff replies were cut off mid-write).
**Assembled:** 2026-11-03 19:30 ET, read-only. **Refreshes** `reports/claude-team-d-PICKUP-PACK-for-deepseek-20261103.md` (written 14:38 ET). No board push — the Lead files pointers.
**Owner:** Team D / `roblox-lead`. The Claude Team D lane went quiet after its 14:45 ET handoff (`20261103T194500Z`).

## Bottom line
- **Street:** both ruled merges are on **local `main` @ `300b632`** (`536e3c8` moving, `300b632` atlas:edge); nothing pushed (no upstream configured); sealed hashes intact. The open Studio is on the **new place** (`2df9cdb4…`), still at **282×100**, still open, 2 of 3 Plays left.
- **Card:** the merge is **still undecided** — options **A/B/C** await Nox; the spec branch vs `master` is 840 files / +39,800 lines because it carries the unapproved candidate/audio/bind-art prep. **Δ2** and **Δ16** await the owner's sitting.
- **Thresholds on `main` are provisional**: the edge baselines were captured at 296×398 / 282×100, so the committed fleck max (8253) is a 282×100 outlier. The fix (viewport guard + scoped void bound + refreeze) needs Nox's yes.
- **studiocycle cannot see the native Wayland Studio window** (it looks for class `steam_proton`; the window class is `robloxstudiobeta.exe`). Never `studiocycle cycle`; close/open only from a known state.
- **Post-14:38 refresh:** no new Team D commits, runs, or Nox rulings found. The only later filing is the 14:45 ET handoff (`20261103T194500Z`); the Studio is the same PID from 11:53.

## What this is
- `untitled-street-game`: the street game, its build/seal hashes, native-Play runner + contained Vinegar Studio, and the `look_metrics` threshold freeze.
- `untitled-anime-card-game`: the card game plus the owner-facing design spec and the numbers-only balance pass.
- Board record: `reports/claude-team-d-*.md`; run evidence under `docs/verify/2026-10-19/vinegar/`.

## Current state (with evidence)

### Street repo and merges
- `main` @ **`300b632`** (2026-11-03 12:59:42 ET) ← `536e3c8` (12:58:33) ← `bfc28ab`; branches `prep/moving-association-20261103` @ `18d01a2`, `prep/atlas-edge-diag-20261103` @ `bfc28ab`.
- **Local only:** `git rev-parse --abbrev-ref @{u}` → "no upstream configured for branch 'main'". Nothing pushed.
- Merged-tree claims re-checked: `git diff --stat 536e3c8 HEAD -- src` is **empty** (no production `src/` change in either merge).
- Sealed hashes re-computed: `build/test.rbxlx` **`c6651793ace0…fe98`**, `build/release.rbxlx` **`84324d4342…f6ce`**, sealed slot `…/data/usg/usg-desktop.rbxlx` **`c6651793…`** — unchanged.
- New-place slot `…/data/usg-main/usg-desktop.rbxlx` = **`2df9cdb49bbf…1946`**, 2,408,111 bytes, built from `main 0c325f8` (2026-11-02 23:18). The merges touch no `src/`, so this place still matches main's sources.
- Working tree: 870 untracked evidence files, **no tracked modifications** (`git status --porcelain | grep -v '^??'` empty).

### Studio (read-only process state)
- One real Studio: **PID 291883**, started **11:53:32**, elapsed 7h37m, contained via `bwrap` (PID 291758) → `vinegar` (291769); `RobloxStudioBeta.exe … data/usg-main/usg-desktop.rbxlx`.
- One Wine crash-handler process (PID 292012) is a companion of that launch, not a second Studio.
- No `studiocycle` process; no listener on 18101/18102 seen in `ss -ltn`. (A separate finish-lane fake MCP process exists under `fleet-scratch/W6B-MUT` — not Team D's; do not touch.)
- Current opening: **1 of 3 Plays used** (Play 36, per the 14:38 pack); viewport `[282,100,0]` in the last edge run (`docs/verify/2026-10-19/vinegar/20261103T165617Z-atlas-edge/report.json`). The pack says the 2 remaining Plays are useless for viewport-sensitive images.
- `studiocycle` is a symlink to `Work/tools/winctl/studiocycle.py`; `DEFAULT_PLACE = data/usg/usg-desktop.rbxlx`; `studio_windows()` matches class **`steam_proton`** (line ~129), which is why the native Wayland window is invisible to it.

### Threshold / edge-look defect
- `docs/art/thresholds.json` on main (`bfc28ab`) mixed scout/moving at **1837×744** with edge parts at **296×398** and **282×100**; `look_metrics` has no viewport rule, so all 18 edge views passed as "valid".
- Consequence: fleck max **8253** is a 282×100 outlier; edge images are **not a usable look baseline**. The **wall-containment drive-in PASS stands** (56 runs; it does not depend on image size).
- Recommended (not applied): fleck `all` max **270**, cyan `all` max **0.63** (20% under the 1837×744 baseline worst: 337 / 0.785); void max **0.05 at every `edge-*` view**, provisional and **scoped**, else it would fail every street view (0.03–0.9).
- Tests/validator: the corrected departure validator is `dae2abc`; the three part runs `20261103T165617Z`, `161812Z`, `163557Z` are the live PASS data.

### Card repo
- Current checkout `prep/art-binding-20261101` @ `cd91a88`; trunk **`master` @ `04031c0`**; `prep/card-game-design-spec` @ **`e4e4746`** (worktree `~/Work/tmp/card-spec-wt`); `prep/balance-pass-20261103` @ **`956b4bc`** (worktree `~/Work/tmp/card-balance-wt`).
- Verified `git diff --shortstat master prep/card-game-design-spec`: **840 files, +39,800 / -628**. Per the 13:05 report (not re-tested here): the 4 docs commits (`84c4018`, `6b9c985`, `448cec3`, `e4e4746`) cherry-pick cleanly onto master; the balance code (`3d21456`, `956b4bc`) does not (needs the candidate stack).
- Balance result: starter deck vs 12 routes clean sweep **12/12 → 11/12**; only `w3-enc-1` moves **100% → 0%** (`Config.ROUTE_ENEMY_LEVEL["w3-enc-1"] = 8`).
- **Δ2** = skill points/rebirth are empty rewards (plus "saved skill allocation not loaded after refresh" folded in). **Δ16** = guided Continue should skip a route the deck cannot realistically win (boss-1). Both are owner decisions, not in the 15.

### What changed after 14:38 ET
- 14:45 ET filing: `inbox/2026-11/20261103T194500Z-claude-team-d-to-nox.md` — "pickup pack filed … No further Claude-side work from this lane."
- No new commits: street main still `300b632` (12:59); no run dirs after 12:05; card branches unchanged.
- No **filed** Nox ruling found after `20261103T134600Z` (option (a); place-override approved) on the card A/B/C, margin/void, or the Studio close. If a ruling came by chat it is not on the board — UNKNOWN.
- Studio unchanged: same PID 291883 from 11:53:32; not closed by SSH.

## Running now
- **One contained Vinegar Studio** (PID 291883, bwrap 291758, vinegar 291769) idle on the new place; 2 Plays left; viewport 282×100.
- **No** studiocycle, no runner, no Studio MCP, no Team D service or timer.
- The other lanes' W6BS scratch processes may appear in `pgrep`; they are not Team D's and must not be touched.

## Owed / decisions
1. **Card merge A/B/C — Nox.** (A) whole stack incl. unapproved candidate/audio/bind-art; (B) docs-only cherry-pick of the 4 commits; (C) leave all card branches. Team D recommends **B or C**. Δ2/Δ16 stay open in every case.
2. **Δ2 / Δ16 — owner** (sitting bundle).
3. **Viewport guard + scoped void bound + refreeze — Nox's yes.** Then: one viewport per pack (minimum ~1600×700), re-capture the 3 edge parts at 1837×744, fresh checker, branch only (not merged).
4. **studiocycle Wayland window-finder gap — successor item**, test only with Studio closed; fit and close both depend on it.
5. **Open Studio — Nox/owner close decision.** The lane recommends closing it (2 Plays add nothing at 282×100); it cannot close it itself (`flatpak kill` refused; `studiocycle` blind).
6. **No pushes:** both street merges and all card branches are local-only, by ruling.

## Next steps to continue (ordered)
1. **Do not touch the open Studio.** Confirm the count with the anchored gate `pgrep -f '^[^ ]*/RobloxStudioBeta\.exe'` (the unanchored/normal `pgrep -f` self-counts the shell).
2. Get Nox's close decision. If yes: close via the owner/Nox route **from the known-open state**, verify the process is gone, then open only from a **known-closed** Studio. Never `studiocycle cycle`.
3. Once closed: fix the studiocycle Wayland finder (class/`robloxstudiobeta.exe`), test close/open offline, then use it.
4. On Nox's yes to the threshold fix: branch from `300b632`, add the viewport guard + scoped void bound, refreeze from 1837×744 captures, tests, fresh checker; then re-capture the 3 edge parts at 1837×744 (~3 Plays) and re-run `look_metrics`.
5. Card: on Nox's A/B/C, either cherry-pick the 4 docs commits onto `master` (B) or leave (C); do not merge the candidate stack without explicit approval.
6. Δ2/Δ16: prepare the two owner-decision lines, no code until the owner rules.
7. File pointers through the Lead; no board push from this pack.

## Constraints / do-not
- **Never run `studiocycle cycle`.** Its close cannot see the native Wayland window ("no-studio-window"), then open launches a **second** Studio.
- **Close/open only from a known state** — in practice, open only from a **known-closed** Studio; close only when you know exactly which Studio/window you are closing. Record the viewport in every filing.
- Do not touch other lanes' scratch Studios; wait for the one-Studio gate rather than killing anything.
- Do not alter the sealed place or its hashes (`build/test.rbxlx c6651793…`, `build/release.rbxlx 84324d43…`, `data/usg/usg-desktop.rbxlx c6651793…`).
- No publish/upload/push; no Roblox place id (the place is a local unpublished `.rbxlx`).
- Do not use unanchored `pgrep -f RobloxStudioBeta.exe` for the gate — it matches its own shell.
- Do not reuse the 8253 fleck threshold as a gate; it is provisional and known-defective.

## Where things live
- Street repo: `/home/cayleb/Work/projects/untitled-street-game` (`build/`, `docs/art/thresholds.json`, `docs/verify/2026-10-19/vinegar/`, `tools/`).
- Card repo: `/home/cayleb/Work/projects/untitled-anime-card-game`; worktrees `~/Work/tmp/card-spec-wt`, `~/Work/tmp/card-balance-wt`.
- Studio slots: `~/.var/app/org.vinegarhq.Vinegar/data/usg/usg-desktop.rbxlx` (sealed), `…/usg-main/usg-desktop.rbxlx` (new place).
- Board: `reports/claude-team-d-PICKUP-PACK-for-deepseek-20261103.md` (14:38), `claude-team-d-rulings-merges-margin-20261103.md`, `claude-team-d-moving-baselines-balance-edge-20261103.md`, `claude-team-d-new-test-place-20261103.md`, `claude-team-d-item1-merge-and-successor-20261103.md`; inbox `20261103T134600Z-nox-to-claude-team-d.md`, `20261103T194500Z-claude-team-d-to-nox.md`.
- Run dirs: `docs/verify/2026-10-19/vinegar/20261103T165617Z-atlas-edge/` (viewport proof), `…/20261103T170553Z-verify-clean/`.

## Open questions for Nox/owner
1. Card merge A/B/C — no filed ruling on the board after `134600Z`; is one pending in chat? (UNKNOWN)
2. Is the Studio to be closed now, and by which route? It has been open 7h37m with 2 Plays that the pack says are useless.
3. Approve the threshold fix as scoped (viewport guard, void bound scoped to `edge-*`, refreeze from 1837×744)?
4. Δ2/Δ16: owner's yes/no, and the Δ2 re-lock-at-level-10 interaction?
5. Who owns the studiocycle Wayland fix after the Studio closes — Team D or the fleet tooling lane?
6. Is the 2df9cdb4 new place still the intended Play place after the two merges, or should it be rebuilt from `300b632` before the next opening?
