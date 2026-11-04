from: roblox-day-lead
to: nox
kind: REPLIES — merges done, margin + void fraction frozen values, one ask
---

Nox — this is the single batched reply Team D owed you since 3 Nov ~12:57, brought current (the card merges have
executed since that report, so this is not a re-send). Every number below was re-measured tonight from the artifact on
disk, not copied from the earlier report; where a number is only a report claim, the line says so. Nothing was merged
past a gate, nothing uploaded or published, no Studio launched, no Plays spent.

State in one line: **4 merges done in your order (street 2, card 2), the committed look freeze is defective and
PROVISIONAL, recommended bounds are fleck `all` ≤ 270 and cyan `all` ≤ 0.63 (20 % under the baseline worst view) plus
a void bound of 0.05 scoped to `edge-*` views only, and the fix is already built and held on one branch waiting for your
yes.**

## 1 · Merges — done, in the order you named

**Street `main`** (both, `--no-ff`):
- `536e3c8` = merge of `prep/moving-association-20261103` @ `18d01a2`;
- `300b632` = merge of `prep/atlas-edge-diag-20261103` @ `bfc28ab`.
- `git rev-list --parents -n1 300b632` returns exactly `536e3c8` then `bfc28ab`; `git diff --stat 536e3c8 HEAD -- src`
  is empty, so both merges add tests/tools/docs and change no gameplay.
- Proof on the merged trees: Python `tests/tools` **369 OK**, Lune unit **943 passed / 0 failed (130 specs)**, Lune
  edge **10 passed / 0 failed**; sealed places byte-unchanged `84324d43…` (`release.rbxlx`) and `c6651793…`
  (`test.rbxlx`).
- *basis:* `~/Work/projects/untitled-street-game` git graph read directly tonight; the suite counts and hashes are in
  `lanes/board/reports/20261104-roblox-stream-lead-20261104T0353Z.md` §1, independently reproduced in
  `…-checker-verdict.md` items 1–5 (I did not re-run the 369/Lune suites myself; I did re-run the branch suite, §5).

**Card `master`** (both, `--no-ff`):
- `c15d534` = merge of `prep/card-game-design-spec` @ `e4e4746` into `04031c0` (the only card trunk — there is no card
  `main`);
- `9c3606b` = merge of `prep/balance-pass-20261103` @ `956b4bc` on top of it.
- Trunk head is now **`a22c17e`** — 9 commits on top of the merges (the two gate-defect fixes, the art-suite
  portability fix, docs), `git diff --stat 9c3606b a22c17e -- src` empty, so no gameplay moved. Proof on the trunk: 7 suites green on Lua+Lune, `verify.sh --skip-toolkit` **22 steps / 0 failures**,
  place `99ded6a6d701…`, stable 135-input fingerprint, and zero uploads — no `ops/release/target.json`, no
  `audio-bindings.json`, no `assets/images/roblox-ids.json`, 0 asset ids, `universeId=0/placeId=0`.
- *basis:* `git rev-list --parents -n1 c15d534 / 9c3606b` tonight; numbers from
  `20261104-roblox-stream-lead-20261104T0353Z.md` §2 + `…-checker-verdict.md` items 10–13.

**Which option was taken: A — the whole stack.** Your order named the two branches, and only the candidate stack can
run them. Measured, not argued: `956b4bc`'s parent is `3d21456` (candidate stack), and `3d21456` edits
`tests/integration/test_free_reachability.luau`, which **does not exist** on `04031c0` — so the balance commit cannot
land on a docs-only trunk. Taking the order literally = the whole stack.

**Docs-only alternative still built and available:** `prep/card-docs-only-20261104` @ `7b50877` = old `master` + the
four spec commits — **one file, +1134 lines, no code** (`git diff --stat 04031c0 7b50877`). If you meant the spec only,
that branch is the answer and the A trunk is not needed. Δ2 and Δ16 stay open either way.

Both trunks are **local only** — neither repo has a remote configured, so nothing was pushed.

## 2 · The freeze defect — Team D's, owned here

`docs/art/thresholds.json` on `main` (added by `bfc28ab`, merged at `300b632`, sha256 `137ec201…`, `frozenAt`
2026-11-03T17:30:00Z) was frozen from packs that were **not the same size**. Its two measured bounds prove it:

| bound in the file | value | where that value exists |
| --- | --- | --- |
| `cyanFraction.all` max | 0.785 | scout pack `20261103T125806Z-atlas-scout`, view `city-road-n_3_2__n_4_2-forward` = 0.784638 — **1837×744** |
| `fleckEnergy.all` max | 8253 | edge pack `20261103T165617Z-atlas-edge`, view `edge-city-S` = **8252.84** — **282×100** |

- In the two 296×398 edge packs the same view is 67.57 / 62.17 and the packs' overall fleck max is **80.02**. So 8253
  is a 282×100 artifact, not a street number — exactly as the earlier report said.
- The packs the freeze names: scout at **1837×744** (all 30 sidecars record it), edge `161812Z` and `163557Z` at
  **296×398**, edge `165617Z` at **282×100**. A fourth edge pack that opening (`154642Z`) is also 296×398. All 18 edge
  views in the 282×100 pack still passed as "valid" — `capture_validity` judges one capture against its own sidecar,
  with no minimum size and no one-size-per-pack rule.
- Cause: this Studio now launches as a native Wayland window `studiocycle` cannot see, so its window-fit step never
  ran. The viewport is whatever the window opened at.
- **Consequence: the file is PROVISIONAL and must not be used as a gate.** Concretely, run against its own baseline it
  already fails 7 of 18 street views on `roadLuminance.midOverNear` (0.6) and the facade bound (13 > 1) — those two are
  declared goals in `THRESHOLDS.md`, so the file blocks the unchanged baseline rather than protecting it.
- *basis:* `docs/art/thresholds.json`; `docs/art/THRESHOLDS.md` (which names the four packs); the packs' own
  `views/*.json` sidecars for the viewports; and a fresh `tools/look_metrics.py measure` run tonight over each pack —
  reproduction artifacts `/tmp/opencode/sliceb/*-measured.json`, command in §8.

## 3 · Margin values to freeze (baseline: scout, 1837×744, 18 valid street views)

| metric (road mask, `all`) | measured baseline | bound to freeze at 20 % margin |
| --- | --- | --- |
| `fleckEnergy.all` | max **336.9** (`city-garage`), p90 **199**, median **75.1** | **270** |
| `cyanFraction.all` | max **0.785** (`city-road-n_3_2__n_4_2-forward`), p90 **0.107**, median **0.006** | **0.63** |
| `roadLuminance.midOverNear` | **7 of 18** views under 0.6, worst **0.35** (`city-junction-n_12_12`) | stays `min 0.6` — a goal, not a no-worse bound |
| `facadeRepetition.worstWindowCount` | worst **13** (`image/inkcity/facade_1`) | stays `max 1` — a goal, not a no-worse bound |
| `voidFraction`, street views | **0.028–0.902**, median **0.21** | see §4 — this is a street number, not an edge number |

- *basis:* re-measured tonight, `tools/look_metrics.py` @ `8be563a8…` over pack `20261103T125806Z-atlas-scout`
  (18 of 30 valid; the 12 invalid are 3 street views with no adjacent-road asphalt and 9 normal-fog overheads, by
  design), build `build/main-places/test-main-0c325f8.rbxlx` (`2df9cdb4…`). Output `scout-measured.json`. p90 is
  nearest-rank, which is the method that reproduces the earlier report's 199 / 0.107 (linear interpolation gives
  166.3 / 0.082 — same data, different convention; no bound depends on it).

**Why 20 %:** the metric has no run-to-run jitter — where two runs produced a byte-identical image, the metrics are
byte-identical (2 views across the two 296×398 packs) — so the margin does not have to cover noise. It has to demand a
visible improvement and stop a no-op or a different shot (window size, opening, pose) from reading as a gain.
- **0 % protects nothing**: measured, the unchanged baseline fails 0 of 18 views at margin 0 — the bound is passed by
  the art as it stands.
- **50 % is wrong**: measured, 336.9 → 168.4 fails **2 of 18** street views at baseline (`city-garage`,
  `city-road-n_6_5__n_6_6-forward`; 8 of 27 measured views including overheads) and sets a bar before anyone knows the
  road material's ceiling. *Note:* the earlier report said "5 of 18" — I could not reproduce 5 from the pack; the
  direction holds, the count there was wrong.
- **20 % is the recommendation**: fleck 270 fails only `city-garage` (1 of 18), cyan 0.63 fails only the one
  outlier road view (1 of 18) — i.e. it asks the two known offenders to come down and lets every other valid view pass.

## 4 · Void fraction — recommend max 0.05, scoped to `edge-*` views, provisional

- **Recommendation: `voidFraction` max 0.05 at every `edge-*` view**, provisional. `ART_DIRECTION.md` line 79 says
  exactly "Void fraction (near-black, low-variance pixels below the horizon) near zero at every edge view"; a view with
  5 % of its below-horizon pixels near-black and flat shows a visible hole at the world edge.
- **No valid-viewport edge baseline exists**, so this is provisional by definition. The edge numbers we do have are
  invalid-viewport: across the 54 edge views of the three packs the freeze names, void runs **0.023–0.999**, median
  **0.31**, worst `edge-access-w-bend` **0.999** (282×100). In the 282×100 pack **0 of 18** views meet 0.05 today — the
  bound is an aspiration the horizon-edge work has to reach, and it is stated as such.
- **Schema requirement (this is the important one):** the threshold file is flat. `check_thresholds` applies every
  bound in `metrics` to every judged view, and a bound whose value is missing also fails as "not measured". So a flat
  void bound of 0.05 would fail the street baseline too — measured, only **1 of 18** street views is at or below 0.05
  (min 0.028); the other 17 run 0.085–0.902. **A void bound must therefore be scoped to edge views** (a small view
  pattern in the threshold schema) or it fails every street view. Add the scoping with the refreeze, not as a separate
  step.
- *basis:* `tools/look_metrics.py` (`check_thresholds`, read tonight); `docs/ART_DIRECTION.md` line 79; my
  `edge-*-measured.json` + `scout-measured.json` reproductions.

## 5 · The one ask — yes to guard + scoped void bound + refreeze, on a branch

**Ask: yes.** With your yes, Team D does viewport guard + scoped void bound + refreeze from comparable captures, **on a
branch, not merged**, with tests and a fresh independent checker. It is already prepared and held:

- `prep/look-metrics-viewport-guard` @ **`618316f`** — **one viewport per pack** (a pack that mixes sizes judges 0 views
  and the reason names the sizes; a pack with no recorded viewport is still left to `capture_validity`) plus
  `--min-viewport WxH` **off by default**, because the minimum size is your ruling parameter, not the tool's decision.
- Diff vs `main`: 2 files, +107/−3 (`tools/look_metrics.py`, `tests/tools/test_look_metrics.py`). **No threshold value
  touched, no refreeze, no re-capture** — I checked the diff.
- Tests: I re-ran the suite on the branch tonight — **Ran 372 tests, OK (skipped=1)**; `main` is 369, the branch adds 3.
- Switching the guard on is one flag, no code change.
- Successor item accepted: the **studiocycle Wayland window-finder gap** (window fit and window close both depend on
  it). It needs Studio closed to test, so it is first in the queue behind this ruling.

## 6 · Studio state

- **No Studio is running**: `pgrep -f '^[^ ]*/RobloxStudioBeta\.exe'` → 0 (anchored pattern; the unanchored form
  self-counts). The contained instance Team D left open (opening B, 282×100) is **closed**.
- **No Plays spent.** `~/.cache/usg/vinegar-plays.json` still reads `plays: 1` (of 3), last written 2026-11-03
  11:56:23 EST — the Play Team D's opening B used (Play 36); nothing has moved the counter since, hours before this lane
  started. The day lane launched nothing, and `studiocycle cycle` was not run (it must not be: close finds no Wayland
  window, then open launches a second Studio).
- The re-capture in §5 needs Studio open, so it needs you present: 3 parts × `--edge-part I/3`, about 3 Plays.

## 7 · What you must rule — one line each

1. **Viewport guard**: yes/no, and the minimum size (Team D proposed **1600×700**; it stays off until you name one).
2. **Void bound scoped to `edge-*` views at max 0.05, provisional**: yes/no.
3. **Margin freeze — fleck `all` ≤ 270, cyan `all` ≤ 0.63**: yes/no.
4. **Refreeze on a branch (not merged) from comparable edge captures, tests + fresh checker**: yes/no — and note it
   spends ~3 Plays and needs you at the machine.
5. **Card trunk**: A (whole stack, now `a22c17e`) or docs-only (`7b50877`)?

## 8 · What is measured and what is only claimed

So you can tell a frozen baseline from a claimed one:
- **Reproduced by me tonight** (command, ~10 s per pack, no Studio, no writes to any repo):
  `python3 tools/look_metrics.py measure --pack docs/verify/2026-10-19/vinegar/<run> --out /tmp/opencode/sliceb/<run>.json
  --build build/main-places/test-main-0c325f8.rbxlx --thresholds docs/art/thresholds.json`
  → `scout-measured.json`, `edge-282x100-measured.json`, `20261103T161812Z…json`, `20261103T163557Z…json`.
- **Not reproduced from an artifact on this machine** — these stay report claims and no recommended bound depends on
  them: (a) "deterministic to 3–4 digits across three Plays" — the packs hold different shots (only 2 of 18 views share
  a byte-identical image across the two 296×398 runs, and those two measure identically), so the render-determinism
  claim cannot be tested from what is on disk; (b) the "5 of 18 views fail fleck at 50 %" count — I measure 2 of 18;
  (c) Play numbers 33–36 — no artifact records a Play number, the run dirs are timestamp-named only.
- **Missing entirely** (searched: repo-wide text search for `fleckEnergy` in the JSON/Markdown of
  `untitled-street-game`, plus the `street-*` folders under `Work/session-artifacts/`, which hold only 19 Oct
  artifacts): no measurement output from the 3 Nov freeze run was ever filed — the frozen file cites no report and
  `THRESHOLDS.md` cites only run-dir names. That is part of why it is provisional: its numbers can be re-derived (as
  above) but the freeze run's own output is not on disk.

## Provenance

Files I opened to write this (sha256 prefix, mtime, path). Repo paths are under `~/Work/projects/`.

| sha256 | mtime | file |
| --- | --- | --- |
| `791c8db51c00b60d…` | 2026-11-04 05:44:23 | `fleet-scratch/overnight/roblox-day/BRIEF-B.md` |
| `d918e791494386e3…` | 2026-11-04 05:45:08 | `fleet-scratch/overnight/roblox-progress.md` |
| `ecd3e9d6ff3b97e4…` | 2026-11-03 13:05:57 | `Work/shared-workflows/lanes/board/reports/claude-team-d-rulings-merges-margin-20261103.md` |
| `99166458c6aa4a74…` | 2026-11-03 14:38:58 | `Work/shared-workflows/lanes/board/reports/claude-team-d-PICKUP-PACK-for-deepseek-20261103.md` |
| `6489ccef603e02da…` | 2026-11-03 23:50:44 | `Work/shared-workflows/lanes/board/reports/20261104-roblox-stream-lead-20261104T0353Z.md` |
| `cc6631ca78ebc284…` | 2026-11-03 23:42:12 | `Work/shared-workflows/lanes/board/reports/20261104-roblox-viewport-branch-20261104.md` |
| `423fd47fea0a56d9…` | 2026-11-04 00:05:06 | `Work/shared-workflows/lanes/board/reports/20261104-roblox-addendum-20261104.md` |
| `1e509b87cb88c237…` | 2026-11-04 00:05:06 | `Work/shared-workflows/lanes/board/reports/20261104-roblox-stream-lead-checker-verdict.md` (head + verdict table) |
| `137ec201339015da…` | 2026-11-03 12:59:42 | `untitled-street-game/docs/art/thresholds.json` |
| `73287d35e8263e16…` | 2026-11-03 12:59:42 | `untitled-street-game/docs/art/THRESHOLDS.md` |
| `3aec0149f988db01…` | 2026-10-20 21:35:36 | `untitled-street-game/docs/ART_DIRECTION.md` (lines 21, 69, 79, 80) |
| `8be563a8e1021ea6…` | 2026-11-02 22:52:35 | `untitled-street-game/tools/look_metrics.py` (usage, `check_thresholds`, `capture_validity`) |
| — | 2026-11-02 23:18 | `untitled-street-game/build/main-places/test-main-0c325f8.rbxlx` (used as `--build`; sha `2df9cdb4…` per each run's `preflight.json`, not re-hashed by me) |
| — | 2026-11-03 07:58 | run dir `untitled-street-game/docs/verify/2026-10-19/vinegar/20261103T125806Z-atlas-scout/` — `views/*.json` (30), `originals.json`, `preflight.json` |
| — | 2026-11-03 16:56 | run dir `…/20261103T165617Z-atlas-edge/` — `views/*.json` (18), `originals.json`, `preflight.json` |
| — | 2026-11-03 16:18 | run dir `…/20261103T161812Z-atlas-edge/` — same |
| — | 2026-11-03 16:35 | run dir `…/20261103T163557Z-atlas-edge/` — same |
| — | 2026-11-03 14:41 / 15:46 / 15:57 | run dirs `…/20261103T144130Z-`, `…T154642Z-`, `…T155744Z-atlas-edge/` — sidecar viewports only |
| — | 2026-11-03 14:28 | `…/20261103T142804Z-moving/report.json` (viewport check) |
| — | 2026-10-19 15:09 | `untitled-street-game/docs/verify/2026-10-19/look-metrics-baseline-034.json` (checked, **not** the basis: 19 Oct, 1594×762, 0 valid views) |
| — | 2026-11-03 11:56:23 | `~/.cache/usg/vinegar-plays.json` (Play counter) |
| — | branch `618316f` | `untitled-street-game` git: `log`/`rev-list --parents`/`diff main...prep/look-metrics-viewport-guard`; worktree `~/Work/tmp/usg-viewport-wt` for the 372-test run |
| — | master `a22c17e` | `untitled-anime-card-game` git: `log`/`rev-list --parents`/`cat-file -e 04031c0:tests/integration/test_free_reachability.luau`/`diff --stat 04031c0 prep/card-docs-only-20261104` |

My reproduction artifacts (regenerate with §8's command): `/tmp/opencode/sliceb/scout-measured.json`
(`555c1edec6b4c182…`), `edge-282x100-measured.json` (`342cc249509a714b…`), `20261103T161812Z-atlas-edge-measured.json`
(`88d7f1de170ddd14…`), `20261103T163557Z-atlas-edge-measured.json` (`26bf4886d4586453…`).

Not filed to the board by me (the lead files it). Not touched: `docs/art/thresholds.json`, any product source, Studio,
the board.
