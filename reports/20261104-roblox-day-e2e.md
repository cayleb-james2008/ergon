# Roblox day — end-to-end (street + card, nothing past gates)

Date: 2026-11-04
Sources: `fleet-scratch/overnight/roblox-day/INVENTORY-STREET.md`, `INVENTORY-CARD.md`; slices A/B/C/D reports alongside this one.
Fences: board only. No `--live`, no key reads, no upload/publish, no Studio, no product merges by this slice. No credentials.

## Street — owner gates (per INVENTORY-STREET)

Repo `untitled-street-game`, trunk `main` @ `a43b7e4`; last code merge `300b632` (`src/` unchanged since `536e3c8`); RC `build/rc/rc-300b632.rbxlx` (sha `c34e4cc2…`); sealed places unchanged `84324d43…` / `c6651793…`.

Publication is the owner's gate (AGENTS.md release gate + STATE "owner decisions"); nothing publishes without his explicit yes:

- Playtest route choice: A) Studio Play on `rc-300b632.rbxlx` (no upload, Studio verdict only) or B) `tools/usg publish --published --restart` (still PRIVATE, full phone/tablet sheet) — sheet `docs/launch/OWNER_PLAYTEST.md`.
- C2 private upload: he runs the two `publish` commands; agent verifies.
- C3 his device/driving verdict (phone + tablet join, leave-and-rejoin, 15-min drive, audio reel) — gates any fix lane.
- D3 Creator Hub steps 1–7 (description, icon/thumbs, maturity questionnaire, age/regions, voice OFF) + `tools/launch_readback.py` check.
- D4/E1 PUBLIC_FLIP_CHECKLIST green + his Public yes in his own words, recorded in RELEASE_LOG; he flips Public. Rollback decision is his.

## Street — agent-side remainder

All offline green (Lune 943/0 across 130 specs, Python 369 OK — checker 14 PASS / 3 FAIL are claims-not-build-state). Native truth unproven: nothing has run in real Studio since 2026-10-18:

1. World look in motion on GPU: road flecks, cyan sheen, dark mid-distance, void at edges, lamps, repeated frontage, blank overheads (Stage B3).
2. Native truth: city blocked-grade, junction, handling matrix, cadence gaps (gui014/015/020/030), first-place race gate, traffic/rival feel (B1/B2/B4).
3. Thresholds in `docs/art/thresholds.json` are provisional — viewport defect; Team D refreeze awaiting Nox's yes (B1 freeze). Frozen values + corrections: Slice B report.
4. Never done: 27 audio files unlistened, two-client, live persistence + receipt retries, perf budgets, truthful rating (B4/C1).
5. Backlog post-launch (not blocking): PvP sprints, car meet, rain nights, ramps, photo mode, crews, traffic rows 12/13/17.

## Card — owner gates (per INVENTORY-CARD)

Main worktree HEAD `da27e6b` on `prep/art-binding-20261101` (no D12 docs on disk there); trunk `master` @ `7ebbb9a` = the D12 bundle, local only, not pushed (see Slice C report). Candidate at `7ebbb9a`: `99ded6a6…` + manifest `49c4338b…`, verify 22/0 twice-identical (Linux domain only, never Studio evidence). Gate counts: **3 PASS · 3 NOT_RUN · 5 UNVERIFIED · 3 BLOCKED**; overall preflight STAGING. Nothing uploaded/published/spent.

| Item | Status |
|---|---|
| D12 sitting: spec v0.1 approve/strike; 29 uploads; new private experience IDs; listen to 5 audio files; re-approve `BK-GUTS-01-clean-candidate.png`; Open Cloud key via secure route; signed-in Studio window | awaiting owner's reply (answer sheet at `7ebbb9a`) |
| Rights A-04: accept moderation risk for 24 portraits or hold art/audio | decision only |
| Δ2 (in the 15, approve/strike) and Δ16 (separate add/leave) | `D12-SPEC-DELTAS.md` at `7ebbb9a` |
| Studio sign-in + Play on the exact candidate hash | NOT_RUN |
| Creator/universe/place authorization; `target.json` | BLOCKED — absent |
| Art upload + moderation; 24 `ArtIds` | BLOCKED — empty; gated on D12 + target |
| Physical device/mobile + sustained-session measurements | UNVERIFIED |
| Studio audio playback (listen & record) | UNVERIFIED |
| Rights status USER-ATTESTED | UNVERIFIED |
| Paid-product identity + exact price (offers stay disabled) | BLOCKED |
| Separate publish sign-off for exact candidate + action | NOT_RUN |

## Card — agent-side remainder

- Fresh receipt on a named candidate, committed in this repo (source is one fix ahead of the build `55cef5d3…96b64083833ea5590466d4c9`).
- AC-15 close-out: review run receipts + final diff for forbidden actions.
- P4 carry-forward: disabled offers keep `UNCONFIGURED` status regardless of enabled state.
- Reference-game battle observations (0/10 complete) — rules still provisional.
- Independent art-fidelity assessment.
- Re-run local verify chain to replace the missing 2026-09-27 receipts (receipt dir absent from this machine).

## Slice D status

Branch `prep/slice-d-qa-20261104`, worktree `~/Work/tmp/usg-slice-d-wt`, **14 tests pass, uncommitted** (see Slice D report). Tools ready for review; commit + merge are lead-gated until the street playtest verdict / Nox review. Next bounded step: Nox/lead review, then decide landing into `main` (docs+tools only) and wiring into the inspect stage.

## Nothing past gates

Per-merge proof (A) bound the green claims to their commits and to later fixes where required. The Nox reply (B) is filed and its values stay provisional pending his yes. The D12 bundle (C) stops at the owner gate — no `--live`, no upload, no publish. Slice D tools are uncommitted and ungated by nothing except the lead/Nox review. No Studio run, no Play spent, no sealed place rebuilt, no card artifact uploaded or published, no key read or written.
